use eznet::{packet::Packet, socket::Socket};
use tui_chat_app_common::{
    client::ClientInitPacket,
    compat::COMPAT_INFO,
    server::{ServerInitPacket, ServerPacket},
    FromPacketBytes, IntoPacketBytes,
};

//

pub async fn handler(socket: Socket) {
    let _ = handler_try(socket).await;
}

async fn handler_try(mut socket: Socket) -> Option<()> {
    println!("Connecting to {}", socket.remote());

    // Init state

    let init = ClientInitPacket::ClientInfo(COMPAT_INFO).into_bytes();
    socket.send(Packet::ordered(init, None)).await?;
    match recv(&mut socket).await? {
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

    println!("Connected to {}", socket.remote());

    Some(())
}

async fn recv(socket: &mut Socket) -> Option<ServerPacket> {
    ServerPacket::from_bytes(socket.recv().await?.bytes)
}
