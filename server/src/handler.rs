use dashmap::DashSet;
use eznet::{packet::Packet, socket::Socket};
use std::{net::IpAddr, sync::Arc};
use tui_chat_app_common::{
    client::{ClientInitPacket, ClientPacket},
    compat::COMPAT_INFO,
    server::{ServerInitFailReason, ServerInitPacket},
    FromPacketBytes, IntoPacketBytes,
};

//

pub async fn handler(mut socket: Socket, connections: Arc<DashSet<IpAddr>>) {
    if !connections.insert(socket.remote().ip()) {
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

    if handler_try(&mut socket).await.is_none() {
        eprintln!("Client error");
    }

    println!("Disconnected {}", socket.remote());

    connections.remove(&socket.remote().ip());
}

async fn handler_try(socket: &mut Socket) -> Option<()> {
    // Init state

    let packet = recv(socket).await?;
    let response = match init_state(packet) {
        Ok(()) => ServerInitPacket::Success(COMPAT_INFO),
        Err(reason) => ServerInitPacket::Fail { reason },
    };

    socket
        .send(Packet::ordered(response.into_bytes(), None))
        .await?;

    Some(())
}

async fn recv(socket: &mut Socket) -> Option<ClientPacket> {
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
