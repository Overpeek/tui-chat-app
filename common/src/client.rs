use crate::{compat::CompatibilityInfo, FromPacketBytes, IntoPacketBytes};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

//

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientPacket {
    /// This first variant should never change
    Init(ClientInitPacket),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientInitPacket {
    /// This first variant should never change
    ClientInfo(CompatibilityInfo),
}

//

impl IntoPacketBytes for ClientPacket {}

impl IntoPacketBytes for ClientInitPacket {
    fn into_bytes(self) -> Bytes {
        ClientPacket::Init(self).into_bytes()
    }
}

impl FromPacketBytes for ClientPacket {}
