use crate::{compat::CompatibilityInfo, FromPacketBytes, IntoPacketBytes};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientPacket {
    /// This first variant should never change
    Init(ClientInitPacket),

    Chat(ClientChatPacket),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientInitPacket {
    /// This first variant should never change
    ClientInfo(CompatibilityInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientChatPacket {
    RequestMembers,
    RequestSelfMember,

    SendMessage { message_id: Uuid, message: String },
    EditMessage { message_id: Uuid, message: String },
    RemoveMessage { message_id: Uuid },

    KeepAlive,
}

//

impl IntoPacketBytes for ClientPacket {}

impl IntoPacketBytes for ClientInitPacket {
    fn into_bytes(self) -> Bytes {
        ClientPacket::Init(self).into_bytes()
    }
}

impl IntoPacketBytes for ClientChatPacket {
    fn into_bytes(self) -> Bytes {
        ClientPacket::Chat(self).into_bytes()
    }
}

impl FromPacketBytes for ClientPacket {}
