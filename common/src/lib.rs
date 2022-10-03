use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};

//

pub mod client;
pub mod compat;
pub mod server;

//

pub trait IntoPacketBytes: Serialize + Sized {
    fn into_bytes(self) -> Bytes {
        bincode::serialize(&self)
            .map(|v| Bytes::from(v))
            .unwrap_or_default() // if encoding fails, just don't fucking care
    }
}

pub trait FromPacketBytes: DeserializeOwned {
    fn from_bytes(bytes: Bytes) -> Option<Self> {
        // if decoding fails, we report is as an invalid packet
        // and possibly kick the client
        bincode::deserialize(&bytes[..]).ok()
    }
}
