use crate::{
    compat::{CompatibilityError, CompatibilityInfo},
    FromPacketBytes, IntoPacketBytes,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerPacket {
    // This first variant should never change
    Init(ServerInitPacket),
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

impl FromPacketBytes for ServerPacket {}
