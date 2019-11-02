use std::sync::mpsc::Sender;

use failure::{
    format_err,
    Error,
};
use futures::sync::mpsc;
use futures::stream::{
    Stream,
    SplitStream,
    SplitSink,
};
use tokio::net::{
    UdpSocket,
    UdpFramed,
};
use tokio::prelude::*;

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
    let client = tokio_dns::resolve_sock_addr(&address[..])
        .from_err()
        .and_then(move |addr_vec| {
            let socket = UdpSocket::bind(&([127, 0, 0, 1], 0).into())
                .map_err(|err| {
                    format_err!("Failed to bind udp socket: {}", err)
                })
                .unwrap();

            let mut addr = None;
            for try_addr in addr_vec {
                if socket.connect(&try_addr).is_ok() {
                    addr = Some(try_addr);
                    break;
                }
            }
            
            let addr = match addr {
                Some(addr) => addr,
                None => {
                    panic!("Failed to connect to server");
                }
            };

            log::info!("Connected to server: {}", addr);

            UdpFramed::new(socket, EternalReckoningCodec)
                .send((Operation::ClConnectMessage(operation::ClConnectMessage), addr))
                .and_then(|framed| {
                    framed.into_future().map_err(|(err, _stream)| err)
                })
                .map(move |(op, stream)| {
                    if op.is_none() {
                        return futures::future::err(
                            failure::format_err!("connection closed")
                        );
                    }
                    let op = op.unwrap().0;

                    if let Operation::SvConnectResponse(operation::SvConnectResponse { uuid }) = op {
                        event_tx.send(Event::ConnectionEvent(
                            ConnectionEvent::Connected(uuid)
                        )).unwrap();

                        let (writer, reader) = stream.split();

                        tokio::spawn(
                            ReadConnection::new(reader, event_tx.clone())
                                .map_err(move |err| {
                                    log::error!("Receive failed: {:?}", err);
                                    event_tx.send(Event::ConnectionEvent(
                                        ConnectionEvent::Disconnected(uuid)
                                    )).unwrap();
                                })
                        );

                        tokio::spawn(
                            WriteConnection::new(writer, addr, update_rx)
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
    frames: SplitStream<UdpFramed<EternalReckoningCodec>>,
    event_tx: Sender<Event>,
}

impl ReadConnection {
    pub fn new(
        frames: SplitStream<UdpFramed<EternalReckoningCodec>>,
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
                log::trace!("Packet: {}", &packet.0);
                self.process_data(&packet.0)?;
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
    frames: SplitSink<UdpFramed<EternalReckoningCodec>>,
    addr: std::net::SocketAddr,
    update_rx: mpsc::UnboundedReceiver<Update>,
    state: WriteConnectionState,
}

impl WriteConnection {
    pub fn new(
        frames: SplitSink<UdpFramed<EternalReckoningCodec>>,
        addr: std::net::SocketAddr,
        update_rx: mpsc::UnboundedReceiver<Update>,
    ) -> WriteConnection
    {
        WriteConnection {
            frames,
            addr,
            update_rx,
            state: WriteConnectionState::Connected,
        }
    }

    fn send(&mut self, packet: Operation) -> Result<(), Error> {
        self.frames.start_send((packet, self.addr))?;
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