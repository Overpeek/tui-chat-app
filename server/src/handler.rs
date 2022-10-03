use dashmap::DashSet;
use eznet::{packet::Packet, socket::Socket};
use std::{net::IpAddr, sync::Arc, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time::Instant,
};
use tui_chat_app_common::{
    client::{ClientChatPacket, ClientInitPacket, ClientPacket},
    compat::COMPAT_INFO,
    server::{ServerChatPacket, ServerInitFailReason, ServerInitPacket, ServerPacket},
    FromPacketBytes, IntoPacketBytes,
};
use uuid::Uuid;

//

pub async fn handler(
    mut socket: Socket,
    connections: Arc<DashSet<IpAddr>>,
    send: Arc<Sender<ServerPacket>>,
    recv: Receiver<ServerPacket>,
) {
    if false && !connections.insert(socket.remote().ip()) {
        // already connected from this ip
        let _ = socket
            .send(Packet::ordered(
                ServerInitPacket::Fail {
                    reason: ServerInitFailReason::AlreadyConnected,
                }
                .into_bytes(),
                None,
            ))
            .await;
        return;
    }

    println!("New connection from {}", socket.remote());

    if handler_try(&mut socket, send, recv).await.is_none() {
        eprintln!("Client error");
    }

    println!("Disconnected {}", socket.remote());

    connections.remove(&socket.remote().ip());
}

async fn handler_try(
    socket: &mut Socket,
    send: Arc<Sender<ServerPacket>>,
    mut recv: Receiver<ServerPacket>,
) -> Option<()> {
    // Init state

    let packet = recv_packet(socket).await?;
    let response = match init_state(packet) {
        Ok(()) => ServerInitPacket::Success(COMPAT_INFO),
        Err(reason) => ServerInitPacket::Fail { reason },
    };

    socket
        .send(Packet::ordered(response.into_bytes(), None))
        .await?;

    // Chat state

    let client = Uuid::new_v4();
    let mut hb = Instant::now() + Duration::SECOND;

    loop {
        tokio::select! {
            _ = tokio::time::sleep_until(hb) => {
                socket.send(Packet::ordered(ClientChatPacket::KeepAlive.into_bytes(), None)).await?;
                hb = Instant::now() + Duration::SECOND;
            }
            Some(packet) = recv_packet(socket) => handle_chat_client_recv(socket, send.clone(), packet, client).await?,
            Ok(packet) = recv.recv() => handle_chat_broadcast(socket, packet).await?,
        }
    }
}

async fn recv_packet(socket: &mut Socket) -> Option<ClientPacket> {
    ClientPacket::from_bytes(socket.recv().await?.bytes)
}

fn init_state(packet: ClientPacket) -> Result<(), ServerInitFailReason> {
    let compat = match packet {
        ClientPacket::Init(ClientInitPacket::ClientInfo(i)) => i,
        _ => return Err(ServerInitFailReason::InvalidState),
    };

    COMPAT_INFO.compatible(compat)?;

    Ok(())
}

async fn handle_chat_client_recv(
    socket: &mut Socket,
    send: Arc<Sender<ServerPacket>>,
    packet: ClientPacket,
    client: Uuid,
) -> Option<()> {
    let packet = match packet {
        ClientPacket::Chat(p) => p,
        _ => {
            socket
                .send(Packet::ordered(
                    ServerChatPacket::InvalidState.into_bytes(),
                    None,
                ))
                .await?;
            return None;
        }
    };

    match packet {
        ClientChatPacket::SendMessage {
            message_id,
            message,
        } => {
            send.send(ServerPacket::Chat(ServerChatPacket::NewMessage {
                sender_id: client,
                message_id,
                message: message.trim().to_string(),
            }))
            .ok()?;
        }
        ClientChatPacket::RequestSelfMember => {
            socket
                .send(Packet::ordered(
                    ServerPacket::Chat(ServerChatPacket::SelfMember { member_id: client })
                        .into_bytes(),
                    None,
                ))
                .await?;
        }
        _ => {}
    }

    Some(())
}

async fn handle_chat_broadcast(socket: &mut Socket, packet: ServerPacket) -> Option<()> {
    socket
        .send(Packet::ordered(packet.into_bytes(), None))
        .await
}
