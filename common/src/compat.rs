use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

//

pub static COMPAT_INFO: CompatibilityInfo = CompatibilityInfo {
    magic_bytes: MagicBytes(0x3064396a3df83f1d),
    version: Version([0, 1, 0]),
};

//

/// This should never change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MagicBytes(pub u64);
/// This should never change

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version(pub [u16; 3]);

/// This struct should never change
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    /// These magic bytes are here
    /// to filter out accidental
    /// connections that are not
    /// compatible with this app.
    ///
    /// This is NOT to make sure
    /// the client is to be trusted.
    magic_bytes: MagicBytes,

    /// Filter out connections
    /// that have too different
    /// versions.
    ///
    /// For example let v1.1.43
    /// connect to v1.4.54.
    ///
    /// Server configuration can
    /// be used to disallow
    /// certain versions, like:
    /// too old, too new, ...
    ///
    /// Usually vX.Y.Z should be
    /// compatible with vX.W.U
    /// where the X is the same
    /// but Y and Z might not be
    /// and Z and U might not be.
    ///
    /// This is similar to semver.
    version: Version,
}

/*#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VersionMismatchPolicy {
    /// Major versions have to match.
    ///
    /// Minor and patch can differ.
    ///
    /// This is the default behavior.
    #[default]
    SameMajor,

    /// Both major and minor versions
    /// have to match.
    ///
    /// Patch can differ.
    SameMinor,

    /// All major, minor and patch
    /// versions have to match.
    ///
    /// Prefer [`Self::SameMinor`]
    /// as patches shouldn't contain
    /// any breaking changes.
    Same,
}*/

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Error)]
#[non_exhaustive]
pub enum CompatibilityError {
    #[error("Client is invalid (Invalid magic bytes)")]
    InvalidClient,

    #[error("The client is incompatible (server:{server} and client:{client})")]
    VersionMismatch { server: Version, client: Version },
}

//

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0[0], self.0[1], self.0[2])
    }
}

impl CompatibilityInfo {
    /// Test if the connection `self` is compatible
    /// with the connection `other`.
    ///
    /// `self` should be the server and
    /// `other` should be the client.
    ///
    /// Ok is compatible
    /// Err is not
    pub fn compatible(
        self,
        other: Self, /*policy: VersionMismatchPolicy*/
    ) -> Result<(), CompatibilityError> {
        if self.magic_bytes != other.magic_bytes {
            return Err(CompatibilityError::InvalidClient);
        }

        if self.version.0[0] != other.version.0[0] {
            return Err(CompatibilityError::VersionMismatch {
                server: self.version,
                client: other.version,
            });
        }

        Ok(())

        /* match policy {
            VersionMismatchPolicy::SameMajor => self.version.0[0] == other.version.0[0],
            VersionMismatchPolicy::SameMinor => self.version.0[0..2] == other.version.0[0..2],
            VersionMismatchPolicy::Same => self.version == other.version,
        } */
    }
}
