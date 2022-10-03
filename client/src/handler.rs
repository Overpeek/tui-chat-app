use std::time::Duration;

use eznet::{packet::Packet, socket::Socket};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};
use tui_chat_app_common::{
    client::{ClientChatPacket, ClientInitPacket, ClientPacket},
    compat::COMPAT_INFO,
    server::{ServerInitPacket, ServerPacket},
    FromPacketBytes, IntoPacketBytes,
};

//

pub async fn handler(socket: Socket, recv: Receiver<ClientPacket>, send: Sender<ServerPacket>) {
    if handler_try(socket, recv, send).await.is_none() {
        eprintln!("Closed");
    }
}

async fn handler_try(
    mut socket: Socket,
    mut recv: Receiver<ClientPacket>,
    send: Sender<ServerPacket>,
) -> Option<()> {
    // Init state

    let init = ClientInitPacket::ClientInfo(COMPAT_INFO).into_bytes();
    socket.send(Packet::ordered(init, None)).await?;
    match recv_packet(&mut socket).await? {
        ServerPacket::Init(ServerInitPacket::Success(i)) => {
            if let Err(err) = i.compatible(COMPAT_INFO) {
                eprintln!("{err}");
                return None;
            }
        }
        ServerPacket::Init(ServerInitPacket::Fail { reason }) => {
            eprintln!("{reason}");
            return None;
        }
        _ => {
            eprintln!("Invalid state");
            return None;
        }
    };

    let mut hb = Instant::now() + Duration::SECOND;

    loop {
        tokio::select! {
            _ = tokio::time::sleep_until(hb) => {
                socket.send(Packet::ordered(ClientChatPacket::KeepAlive.into_bytes(), None)).await?;
                hb = Instant::now() + Duration::SECOND;
            }
            Some(to_send) = recv.recv() => {
                socket.send(Packet::ordered(to_send.into_bytes(), None)).await?;
            }
            Some(to_send) = recv_packet(&mut socket) => {
                send.send(to_send).await.ok()?;
            }
        }
    }
}

async fn recv_packet(socket: &mut Socket) -> Option<ServerPacket> {
    ServerPacket::from_bytes(socket.recv().await?.bytes)
}
