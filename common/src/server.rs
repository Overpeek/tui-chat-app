use crate::{
    compat::{CompatibilityError, CompatibilityInfo},
    FromPacketBytes, IntoPacketBytes,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};
use thiserror::Error;
use uuid::Uuid;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerPacket {
    // This first variant should never change
    Init(ServerInitPacket),

    Chat(ServerChatPacket),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerInitPacket {
    // These two variants should never change
    Success(CompatibilityInfo),

    Fail {
        // Optional reason for why
        // the server declined the init.
        //
        // This could be an IP ban,
        // invalid magic_bytes,
        // version mismatch or ...
        reason: ServerInitFailReason,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerChatPacket {
    ServerInfo {
        name: String,
    },

    // member packets
    SelfMember {
        member_id: Uuid,
    },
    Members {
        member_ids: HashSet<Uuid>,
    },
    NewMember {
        member_id: Uuid,
    },
    RemoveMember {
        member_id: Uuid,
    },
    MemberInfo {
        members: HashMap<Uuid, MemberInfo>,
    },

    // message packets
    NewMessage {
        sender_id: Uuid,
        message_id: Uuid,
        message: String,
    },
    EditMessage {
        sender_id: Uuid,
        message_id: Uuid,
        message: String,
    },
    RemoveMessage {
        sender_id: Uuid,
        message_id: Uuid,
    },

    KeepAlive,

    InvalidState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberInfo {
    pub name: String,
    pub status: MemberStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberStatus {
    /// User is online (Active or Idle)
    Online,

    /// User is online (Do not disturb not Dungeons&Dragons)
    Dnd,

    /// User is offline
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[non_exhaustive]
pub enum ServerInitFailReason {
    // These 5 variants should never change
    #[error("Invalid state (desync)")]
    InvalidState,

    #[error("Invalid packet")]
    InvalidPacket,

    #[error(transparent)]
    CompatibilityError(#[from] CompatibilityError),

    #[error("Already connected")]
    AlreadyConnected,

    #[error("Server message: {0}")]
    Custom(Cow<'static, str>),
}

//

impl IntoPacketBytes for ServerPacket {}

impl IntoPacketBytes for ServerInitPacket {
    fn into_bytes(self) -> Bytes {
        ServerPacket::Init(self).into_bytes()
    }
}

impl IntoPacketBytes for ServerChatPacket {
    fn into_bytes(self) -> Bytes {
        ServerPacket::Chat(self).into_bytes()
    }
}

impl FromPacketBytes for ServerPacket {}
