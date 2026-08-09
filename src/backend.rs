//! The `--backend` selector.

use std::{fmt, str::FromStr};

use anyhow::{Error, bail};
use clap::Parser;

/// Selects which backend a cross-protocol command targets.
///
/// [`Auto`](Self::Auto) picks the first configured-and-compiled backend
/// in calendula's own priority order (vdir, pimdir, CalDAV, gcal), so a
/// local store is preferred over a network round-trip and a
/// protocol-standard server over a vendor API. A named variant pins the
/// command to that backend and bails when the account carries no
/// matching configuration block.
///
/// The protocol-specific subcommands ignore this flag entirely: each
/// already names its backend.
#[derive(Clone, Copy, Debug, Default, Parser, PartialEq, Eq)]
pub enum Backend {
    /// The first configured backend, in calendula's priority order.
    #[default]
    Auto,
    /// CalDAV only.
    #[cfg(feature = "caldav")]
    Caldav,
    /// The Google Calendar API only.
    #[cfg(feature = "gcal")]
    Gcal,
    /// The local pimdir store only.
    #[cfg(feature = "pimdir")]
    Pimdir,
    /// The local vdir home only.
    #[cfg(feature = "vdir")]
    Vdir,
}

impl Backend {
    /// Whether the CalDAV arm of a shared command may run.
    #[cfg(feature = "caldav")]
    pub fn allows_caldav(self) -> bool {
        matches!(self, Self::Auto | Self::Caldav)
    }

    /// Whether the gcal arm of a shared command may run.
    #[cfg(feature = "gcal")]
    pub fn allows_gcal(self) -> bool {
        matches!(self, Self::Auto | Self::Gcal)
    }

    /// Whether the pimdir arm of a shared command may run.
    #[cfg(feature = "pimdir")]
    pub fn allows_pimdir(self) -> bool {
        matches!(self, Self::Auto | Self::Pimdir)
    }

    /// Whether the vdir arm of a shared command may run.
    #[cfg(feature = "vdir")]
    pub fn allows_vdir(self) -> bool {
        matches!(self, Self::Auto | Self::Vdir)
    }
}

impl FromStr for Backend {
    type Err = Error;

    fn from_str(backend: &str) -> Result<Self, Self::Err> {
        match backend {
            "auto" => Ok(Self::Auto),
            #[cfg(feature = "caldav")]
            "caldav" => Ok(Self::Caldav),
            #[cfg(feature = "gcal")]
            "gcal" => Ok(Self::Gcal),
            #[cfg(feature = "pimdir")]
            "pimdir" => Ok(Self::Pimdir),
            #[cfg(feature = "vdir")]
            "vdir" => Ok(Self::Vdir),
            backend => bail!("Invalid backend `{backend}`"),
        }
    }
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            #[cfg(feature = "caldav")]
            Self::Caldav => write!(f, "caldav"),
            #[cfg(feature = "gcal")]
            Self::Gcal => write!(f, "gcal"),
            #[cfg(feature = "pimdir")]
            Self::Pimdir => write!(f, "pimdir"),
            #[cfg(feature = "vdir")]
            Self::Vdir => write!(f, "vdir"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_backend_round_trips_through_its_wire_spelling() {
        for backend in [
            Backend::Auto,
            #[cfg(feature = "caldav")]
            Backend::Caldav,
            #[cfg(feature = "gcal")]
            Backend::Gcal,
            #[cfg(feature = "pimdir")]
            Backend::Pimdir,
            #[cfg(feature = "vdir")]
            Backend::Vdir,
        ] {
            let spelling = backend.to_string();
            assert_eq!(Backend::from_str(&spelling).unwrap(), backend);
        }
    }

    #[test]
    fn auto_allows_every_arm_and_a_named_backend_allows_only_its_own() {
        #[cfg(feature = "vdir")]
        {
            assert!(Backend::Auto.allows_vdir());
            assert!(Backend::Vdir.allows_vdir());
            #[cfg(feature = "caldav")]
            assert!(!Backend::Vdir.allows_caldav());
        }

        assert!(Backend::from_str("nonsense").is_err());
    }
}
