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
    Framed,
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
        ConnectionEvent,
    },
};

pub fn connect(
    address: &String,
    update_rx: mpsc::UnboundedReceiver<Update>,
    event_tx: Sender<Event>,
)
{
    let client = tokio_dns::TcpStream::connect(&address[..])
        .from_err()
        .and_then(move |stream| {
            log::info!("Connected to server");

            Framed::new(stream, EternalReckoningCodec)
                .send(Operation::ClConnectMessage(operation::ClConnectMessage))
                .and_then(|framed| {
                    framed.into_future().map_err(|(err, _stream)| err)
                })
                .map(|(op, stream)| {
                    if op.is_none() {
                        return futures::future::err(
                            failure::format_err!("connection closed")
                        );
                    }
                    let op = op.unwrap();

                    if let Operation::SvConnectResponse(operation::SvConnectResponse { uuid }) = op {
                        event_tx.send(Event::ConnectionEvent(
                            ConnectionEvent::Connected(uuid)
                        )).unwrap();

                        let (reader, writer) = stream.into_inner().split();

                        let framed_reader = FramedRead::new(reader, EternalReckoningCodec);
                        tokio::spawn(
                            ReadConnection::new(framed_reader, event_tx.clone())
                                .map_err(move |err| {
                                    log::error!("Receive failed: {:?}", err);
                                    event_tx.send(Event::ConnectionEvent(
                                        ConnectionEvent::Disconnected(uuid)
                                    )).unwrap();
                                })
                        );

                        let framed_writer = FramedWrite::new(writer, EternalReckoningCodec);
                        tokio::spawn(
                            WriteConnection::new(framed_writer, update_rx)
                                .map_err(|err| {
                                    log::error!("Write failed: {:?}", err);
                                })
                        );

                        return futures::future::ok(());
                    }

                    futures::future::err(
                        failure::format_err!("unexpected response from server")
                    )
                })
                .map_err(|err| {
                    failure::format_err!("handshake failed: {:?}", err)
                })
        })
        .map(|_| ())
        .map_err(|err| {
            log::error!("Failed to connect to server: {:?}", err);
        });

    tokio::run(client);
}

struct ReadConnection {
    frames: FramedRead<ReadHalf<TcpStream>, EternalReckoningCodec>,
    event_tx: Sender<Event>,
}

impl ReadConnection {
    pub fn new(
        frames: FramedRead<ReadHalf<TcpStream>, EternalReckoningCodec>,
        event_tx: Sender<Event>,
    ) -> ReadConnection
    {
        ReadConnection {
            frames,
            event_tx,
        }
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
                self.process_data(&packet)?;
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
            state: WriteConnectionState::Connected,
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
                WriteConnectionState::Sending => {
                    futures::try_ready!(self.frames.poll_complete());
                    self.state = WriteConnectionState::Connected;
                },
                WriteConnectionState::Connected => {
                    match self.update_rx.poll() {
                        Ok(Async::Ready(Some(update))) => {
                            match update {
                                simulation::event::Update::PositionUpdate(data) => {
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