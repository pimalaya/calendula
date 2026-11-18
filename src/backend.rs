//! # Backend
//!
//! The `--backend` selector, naming which backend the cross-protocol
//! commands target and how `auto` resolves.

use std::{fmt, str::FromStr};

use anyhow::{Error, bail};
use clap::{
    Parser,
    builder::{PossibleValue, PossibleValuesParser, TypedValueParser},
};

/// Every backend calendula knows, as `--backend` and its cargo feature
/// spell it, in the order `auto` tries them.
pub const ALL: [&str; 6] = ["vdir", "pimdir", "caldav", "jmap", "gcal", "msgraph"];

/// Why nothing reaches `backend`, one of [`ALL`] this build leaves out.
pub fn not_built(backend: &str) -> String {
    format!("This build leaves out the `{backend}` backend, rebuild with `--features {backend}`")
}

/// The parser of `--backend`: `auto` and every backend calendula knows,
/// built in or not, each with its help, so the help lists them and a
/// value none is spelled is refused listing them, as Himalaya's is. A
/// backend this build leaves out is refused naming its cargo feature.
pub fn parser() -> impl TypedValueParser<Value = Backend> {
    let auto = PossibleValue::new("auto")
        .help("Take the first backend the account configures, in this order");

    let values = ALL.map(|name| {
        let label = match name {
            "caldav" => "CalDAV",
            "jmap" => "JMAP",
            "gcal" => "Google Calendar",
            "msgraph" => "Microsoft Graph",
            name => name,
        };

        PossibleValue::new(name).help(format!("Pin the command to the account's {label} backend"))
    });

    PossibleValuesParser::new([auto].into_iter().chain(values))
        .try_map(|backend| Backend::from_str(&backend))
}

/// Selects which backend a cross-protocol command targets.
///
/// [`Auto`](Self::Auto) takes the first configured backend of calendula's
/// priority order (vdir, pimdir, CalDAV, JMAP, gcal, msgraph): a local store
/// before a network round-trip, a standard protocol before a vendor API,
/// whatever capability the command needs, one this build leaves out
/// refused. A named variant pins that one instead.
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
    /// JMAP for Calendars only.
    #[cfg(feature = "jmap")]
    Jmap,
    /// The Microsoft Graph API only.
    #[cfg(feature = "msgraph")]
    Msgraph,
    /// The local pimdir store only.
    #[cfg(feature = "pimdir")]
    Pimdir,
    /// The local vdir home only.
    #[cfg(feature = "vdir")]
    Vdir,
}

impl Backend {
    /// Whether the CalDAV arm of a unified command may run.
    #[cfg(feature = "caldav")]
    pub fn allows_caldav(self) -> bool {
        matches!(self, Self::Auto | Self::Caldav)
    }

    /// Whether the JMAP arm of a unified command may run.
    #[cfg(feature = "jmap")]
    pub fn allows_jmap(self) -> bool {
        matches!(self, Self::Auto | Self::Jmap)
    }

    /// Whether the gcal arm of a unified command may run.
    #[cfg(feature = "gcal")]
    pub fn allows_gcal(self) -> bool {
        matches!(self, Self::Auto | Self::Gcal)
    }

    /// Whether the msgraph arm of a unified command may run.
    #[cfg(feature = "msgraph")]
    pub fn allows_msgraph(self) -> bool {
        matches!(self, Self::Auto | Self::Msgraph)
    }

    /// Whether the pimdir arm of a unified command may run.
    #[cfg(feature = "pimdir")]
    pub fn allows_pimdir(self) -> bool {
        matches!(self, Self::Auto | Self::Pimdir)
    }

    /// Whether the vdir arm of a unified command may run.
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
            #[cfg(feature = "jmap")]
            "jmap" => Ok(Self::Jmap),
            #[cfg(feature = "msgraph")]
            "msgraph" => Ok(Self::Msgraph),
            #[cfg(feature = "pimdir")]
            "pimdir" => Ok(Self::Pimdir),
            #[cfg(feature = "vdir")]
            "vdir" => Ok(Self::Vdir),
            backend if ALL.contains(&backend) => bail!(not_built(backend)),
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
            #[cfg(feature = "jmap")]
            Self::Jmap => write!(f, "jmap"),
            #[cfg(feature = "msgraph")]
            Self::Msgraph => write!(f, "msgraph"),
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
    #[cfg_attr(
        not(any(
            feature = "vdir",
            feature = "caldav",
            feature = "jmap",
            feature = "pimdir",
            feature = "gcal",
            feature = "msgraph"
        )),
        allow(clippy::single_element_loop)
    )]
    fn every_named_backend_round_trips_through_its_wire_spelling() {
        for backend in [
            Backend::Auto,
            #[cfg(feature = "caldav")]
            Backend::Caldav,
            #[cfg(feature = "gcal")]
            Backend::Gcal,
            #[cfg(feature = "jmap")]
            Backend::Jmap,
            #[cfg(feature = "msgraph")]
            Backend::Msgraph,
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

    /// A backend left out of the build is named with the feature bringing
    /// it back, whichever it is.
    #[test]
    fn a_backend_left_out_names_its_feature() {
        for backend in ALL {
            match Backend::from_str(backend) {
                Ok(parsed) => assert_eq!(parsed.to_string(), backend),
                Err(err) => assert_eq!(err.to_string(), not_built(backend)),
            }
        }

        assert_eq!(
            not_built("gcal"),
            "This build leaves out the `gcal` backend, rebuild with `--features gcal`"
        );
    }
}
