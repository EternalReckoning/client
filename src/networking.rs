use std::sync::mpsc::Sender;

use failure::Error;
use futures::sync::mpsc;
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
    event_tx: Sender<Event>,
}

pub fn connect(
    address: &String,
    update_rx: mpsc::UnboundedReceiver<Update>,
    event_tx: Sender<Event>,
)
{
    let client = tokio_dns::TcpStream::connect(&address[..])
        .map(|stream| {
            log::info!("Connected to server");

            let (reader, writer) = stream.split();

            let framed_reader = FramedRead::new(reader, EternalReckoningCodec);
            let read_connection = ReadConnection::new(framed_reader, event_tx)
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
    pub fn new(
        frames: FramedRead<ReadHalf<TcpStream>, EternalReckoningCodec>,
        event_tx: Sender<Event>
    ) -> ReadConnection
    {
        ReadConnection {
            frames,
            event_tx,
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

    fn process_data(&mut self, packet: &Operation)
        -> Result<(), Error> {
        match packet {
            Operation::SvUpdateWorld(_) => {
                self.event_tx.send(Event::NetworkEvent(packet.clone()))?;
            },
            _ => {
                log::warn!("Unexpected server message received, ignoring");
            }
        };
        Ok(())
    }
}

impl Future for ReadConnection {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        while let Async::Ready(frame) = self.frames.poll()? {
            if let Some(packet) = frame {
                log::trace!("Packet: {}", &packet);
                match self.state {
                    ReadConnectionState::AwaitConnectionResponse => {
                        self.await_connection_response(&packet)?;
                    },
                    ReadConnectionState::Connected => {
                        self.process_data(&packet)?;
                    },
                };
            } else {
                // EOF
                log::warn!("Disconnected from server");
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
    update_rx: mpsc::UnboundedReceiver<Update>,
    state: WriteConnectionState,
}

impl WriteConnection {
    pub fn new(
        frames: FramedWrite<WriteHalf<TcpStream>, EternalReckoningCodec>,
        update_rx: mpsc::UnboundedReceiver<Update>,
    ) -> WriteConnection
    {
        WriteConnection {
            frames,
            update_rx,
            state: WriteConnectionState::SendConnectionRequest,
        }
    }

    fn send(&mut self, packet: Operation) -> Result<(), Error> {
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
                    futures::try_ready!(self.frames.poll_complete());
                    self.state = WriteConnectionState::Connected;
                },
                WriteConnectionState::Connected => {
                    match self.update_rx.poll() {
                        Ok(Async::Ready(Some(update))) => {
                            match update.event {
                                simulation::event::UpdateEvent::PositionUpdate(data) => {
                                    self.send(Operation::ClMoveSetPosition(
                                        operation::ClMoveSetPosition {
                                            pos: data.position.clone(),
                                        }
                                    ))?;
                                },
                                _ => (),
                            }
                        },
                        Err(err) => {
                            log::warn!("Update channel closed: {:?}", err);
                            return Ok(Async::Ready(()));
                        }
                        _ => {
                            return Ok(Async::NotReady);
                        },
                    };
                },
            }
        }
    }
}