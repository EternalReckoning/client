use std::sync::mpsc::{
    Receiver,
    Sender,
};

use failure::Error;
use tokio::io::{
    ReadHalf,
    WriteHalf,
};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::codec::{
    FramedRead,
    FramedWrite,
};

use eternalreckoning_core::net::{
    codec::EternalReckoningCodec,
    operation::{
        self,
        Operation,
    },
};
use crate::simulation::{
    self,
    event::{
        Event,
        Update,
    },
};

enum ReadConnectionState {
    AwaitConnectionResponse,
    Connected,
}

struct ReadConnection {
    frames: FramedRead<ReadHalf<TcpStream>, EternalReckoningCodec>,
    state: ReadConnectionState,
}

pub fn connect(address: &String, update_rx: Receiver<Update>, update_tx: Sender<Event>) {
    let addr = address.parse().unwrap();

    let client = TcpStream::connect(&addr)
        .map(|stream| {
            log::info!("Connected to server");

            let (reader, writer) = stream.split();

            let framed_reader = FramedRead::new(reader, EternalReckoningCodec);
            let read_connection = ReadConnection::new(framed_reader)
                .map_err(|err| {
                    log::error!("Receive failed: {:?}", err);
                });
            tokio::spawn(read_connection);
            
            let framed_writer = FramedWrite::new(writer, EternalReckoningCodec);
            let write_connection = WriteConnection::new(framed_writer, update_rx)
                .map_err(|err| {
                    log::error!("Write failed: {:?}", err);
                });
            tokio::spawn(write_connection);
        })
        .map_err(|err| {
            log::error!("Failed to connect to server: {:?}", err);
        });

    tokio::run(client);
}

impl ReadConnection {
    pub fn new(frames: FramedRead<ReadHalf<TcpStream>, EternalReckoningCodec>)
        -> ReadConnection
    {
        ReadConnection {
            frames,
            state: ReadConnectionState::AwaitConnectionResponse,
        }
    }

    fn await_connection_response(&mut self, packet: &Operation)
        -> Result<(), Error>
    {
        match packet {
            Operation::SvConnectResponse(_) => (),
            _ => {
                return Err(failure::format_err!(
                    "unexpected message from server: {}",
                    packet
                ));
            }
        };

        log::debug!("Server handshake completed");
        self.state = ReadConnectionState::Connected;

        Ok(())
    }
}

impl Future for ReadConnection {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        while let Async::Ready(frame) = self.frames.poll()? {
            if let Some(packet) = frame {
                match self.state {
                    ReadConnectionState::AwaitConnectionResponse => {
                        self.await_connection_response(&packet)?;
                    },
                    _ => (),
                };
            } else {
                // EOF
                return Ok(Async::Ready(()));
            }
        }

        Ok(Async::NotReady)
    }
}

enum WriteConnectionState {
    SendConnectionRequest,
    Sending,
    Connected,
}

struct WriteConnection {
    frames: FramedWrite<WriteHalf<TcpStream>, EternalReckoningCodec>,
    update_rx: Receiver<Update>,
    state: WriteConnectionState,
}

impl WriteConnection {
    pub fn new(
        frames: FramedWrite<WriteHalf<TcpStream>, EternalReckoningCodec>,
        update_rx: Receiver<Update>,
    ) -> WriteConnection
    {
        WriteConnection {
            frames,
            update_rx,
            state: WriteConnectionState::SendConnectionRequest,
        }
    }

    fn send(&mut self, packet: Operation) -> Result<(), Error> {
        log::trace!("Starting send for packet: {}", packet);
        self.frames.start_send(packet)?;
        self.state = WriteConnectionState::Sending;
        Ok(())
    }
}

impl Future for WriteConnection {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        loop {
            match self.state {
                WriteConnectionState::SendConnectionRequest => {
                    self.send(Operation::ClConnectMessage(operation::ClConnectMessage))?;
                },
                WriteConnectionState::Sending => {
                    log::trace!("Finishing write");
                    futures::try_ready!(self.frames.poll_complete());
                    self.state = WriteConnectionState::Connected;
                },
                WriteConnectionState::Connected => {
                    // FIXME: blocking io call
                    match self.update_rx.recv() {
                        Ok(update) => {
                            match update.event {
                                simulation::event::UpdateEvent::PositionUpdate(data) => {
                                    self.send(Operation::ClMoveSetPosition(
                                        operation::ClMoveSetPosition {
                                            pos: data.position.clone(),
                                        }
                                    ))?;
                                },
                            }
                        },
                        Err(err) => {
                            log::error!("Failed to read update channel: {:?}", err);
                            return Ok(Async::Ready(()))
                        },
                    };
                },
            }
        }
    }
}