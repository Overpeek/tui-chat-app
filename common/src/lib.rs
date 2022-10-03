use bincode::Options;
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use std::mem::size_of;
use uuid::Uuid;

//

pub mod client;
pub mod compat;
pub mod server;

//

pub static MAX_MEMBERS: usize = u16::MAX as usize;
/// maximum size of [`ServerChatPacket::Members`] packet + some extra
pub static MAX_PACKET_BYTES: usize = 100 + size_of::<Uuid>() * 2 * MAX_MEMBERS;

//

pub trait IntoPacketBytes: Serialize + Sized {
    fn into_bytes(self) -> Bytes {
        bincode::DefaultOptions::new()
            .with_limit(MAX_PACKET_BYTES as u64) // no support 128 bit operating systems unfortunately :(
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .serialize(&self)
            .map(|v| Bytes::from(v))
            .unwrap_or_default() // if encoding fails, just don't fucking care
    }
}

pub trait FromPacketBytes: DeserializeOwned {
    fn from_bytes(bytes: Bytes) -> Option<Self> {
        // if decoding fails, we report is as an invalid packet
        // and possibly kick the client

        bincode::DefaultOptions::new()
            .with_limit(MAX_PACKET_BYTES as u64)
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .deserialize(&bytes[..])
            .ok()
    }
}
