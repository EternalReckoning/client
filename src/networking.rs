use std::sync::mpsc::Sender;
use std::net::{Ipv4Addr, SocketAddr};

use futures::channel::mpsc;
use futures::stream::{
    StreamExt,
    SplitStream,
    SplitSink,
};
use futures::sink::SinkExt;
use tokio::net::{lookup_host, UdpSocket};
use tokio_util::udp::UdpFramed;
use uuid::Uuid;

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

#[tokio::main]
pub async fn connect(
    address: &String,
    update_rx: mpsc::UnboundedReceiver<Update>,
    event_tx: Sender<Event>,
)
{
    let socket = UdpSocket::bind((Ipv4Addr::new(127, 0, 0, 1), 0)).await.unwrap();

    let mut addr = None;
    for try_addr in lookup_host(&address[..]).await.unwrap() {
        if socket.connect(&try_addr).await.is_ok() {
            addr = Some(try_addr);
            break;
        }
    }

    let addr = match addr {
        Some(addr) => addr,
        None => panic!("Failed to connect to server"),
    };
    
    log::info!("Connected to server: {}", addr);

    let mut framed = UdpFramed::new(socket, EternalReckoningCodec);
    let uuid = handshake(&mut framed, addr).await.unwrap();

    event_tx.send(Event::ConnectionEvent(
        ConnectionEvent::Connected(uuid)
    )).unwrap();

    let (writer, reader) = framed.split();
    
    tokio::spawn(async move {
        read_connection(reader, event_tx, uuid).await;
    });

    tokio::spawn(async move {
        write_connection(writer, update_rx, addr).await;
    });
}

async fn handshake(framed: &mut UdpFramed<EternalReckoningCodec>, addr: SocketAddr) -> Option<Uuid> {
    framed.send((Operation::ClConnectMessage(operation::ClConnectMessage), addr)).await.unwrap();
    match framed.next().await {
        Some(Ok((op, _addr))) => match op {
            Operation::SvConnectResponse(operation::SvConnectResponse { uuid }) => Some(uuid),
            _ => None,
        }
        _ => None,
    }
}

async fn read_connection(
    mut framed: SplitStream<UdpFramed<EternalReckoningCodec>>,
    event_tx: Sender<Event>,
    uuid: Uuid,
) {
    while let Some(frame) = framed.next().await {
        match frame {
            Ok((op, _addr)) => {
                match op {
                    Operation::SvUpdateWorld(_) => {
                        event_tx.send(Event::NetworkEvent(op.clone())).unwrap();
                    },
                    _ => {
                        log::warn!("Unexpected server message received, ignoring");
                    }
                }
            },
            Err(err) => {
                log::error!("Read failed: {}", err);
                break;
            },
        }
    }

    // EOF
    log::warn!("Disconnected from server");
    event_tx.send(Event::ConnectionEvent(
        ConnectionEvent::Disconnected(uuid)
    )).unwrap();
}

async fn write_connection(
    mut framed: SplitSink<UdpFramed<EternalReckoningCodec>, (Operation, SocketAddr)>,
    mut update_rx: mpsc::UnboundedReceiver<Update>,
    addr: std::net::SocketAddr,
) {
    loop {
        match update_rx.next().await {
            Some(update) => {
                match update {
                    simulation::event::Update::PositionUpdate(data) => {
                        let result = framed.send((
                            Operation::ClMoveSetPosition(
                                operation::ClMoveSetPosition {
                                    pos: data.position.clone(),
                                }
                            ),
                            addr
                        )).await;
                        if let Err(err) = result {
                            log::error!("Send failed: {}", err);
                            return;
                        }
                    },
                    _ => (),
                }
            },
            None => {
                log::warn!("Update channel closed");
                return;
            },
        }
    }
}