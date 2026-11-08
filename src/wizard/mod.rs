//! # Wizard
//!
//! Interactive configuration: one input, the discovered routes to the
//! calendars with the best one picked and reviewed, a sign-in tested as soon
//! as it is typed, and the converters from answers to [`crate::config`].

pub mod configure;
pub mod discover;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
pub mod local;
#[cfg(feature = "caldav")]
pub mod plan;
#[cfg(feature = "caldav")]
pub mod search;
#[cfg(feature = "caldav")]
pub mod secret;
#[cfg(feature = "caldav")]
pub mod sign_in;
