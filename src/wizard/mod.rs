#[cfg(feature = "caldav")]
pub mod caldav;
pub mod configure;
pub mod discover;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
pub mod local;
#[cfg(feature = "caldav")]
pub mod search;
#[cfg(feature = "caldav")]
pub mod secret;
