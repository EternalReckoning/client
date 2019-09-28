use std::sync::mpsc::{
    Receiver,
    Sender,
};

use bytes::Bytes;
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
    packet::{
        Packet,
        Operation,
    },
};
use crate::simulation::event::{
    Event,
    Update,
};

enum ReadConnectionState {
    AwaitConnectionResponse,
    Connected,
}

struct ReadConnection {
    frames: FramedRead<ReadHalf<TcpStream>, EternalReckoningCodec>,
    state: ReadConnectionState,
}

enum WriteConnectionState {
    SendConnectionRequest,
    Connected,
}

struct WriteConnection {
    frames: FramedWrite<WriteHalf<TcpStream>, EternalReckoningCodec>,
    state: WriteConnectionState,
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
            let write_connection = WriteConnection::new(framed_writer)
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
    pub fn new(frames: FramedRead<ReadHalf<TcpStream>, EternalReckoningCodec>) -> ReadConnection {
        ReadConnection {
            frames,
            state: ReadConnectionState::AwaitConnectionResponse,
        }
    }

    fn await_connection_response(&mut self, packet: &Packet) -> Result<(), Error> {
        if packet.operation != Operation::ConnectRes {
            return Err(failure::format_err!(
                "unexpected message from server: {}",
                packet.operation
            ));
        }

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

impl WriteConnection {
    pub fn new(frames: FramedWrite<WriteHalf<TcpStream>, EternalReckoningCodec>) -> WriteConnection {
        WriteConnection {
            frames,
            state: WriteConnectionState::SendConnectionRequest,
        }
    }

    fn send_connection_request(&mut self) -> Result<(), Error> {
        log::trace!("Starting send for packet: {}", Operation::ConnectReq);
        self.frames.start_send(Packet {
            operation: Operation::ConnectReq,
        })?;
        self.state = WriteConnectionState::Connected;

        Ok(())
    }
}

impl Future for WriteConnection {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        loop {
            match self.state {
                WriteConnectionState::SendConnectionRequest => self.send_connection_request()?,
                WriteConnectionState::Connected => {
                    log::trace!("Finishing write");
                    return self.frames.poll_complete();
                },
            }
        }
    }
}