//! # Vdir client
//!
//! calendula's wrapper around [`io_vdir`]'s std client, for the
//! protocol-specific subcommands.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use io_vdir::{client::VdirClient as Inner, path::VdirPath};

use crate::{cli::resolve_account, config::VdirConfig};

/// A vdir client rooted at the configured home directory.
pub struct VdirClient {
    inner: Inner,
}

impl VdirClient {
    /// Builds the client rooted at the configured home directory.
    ///
    /// The path arrives shell-expanded, [`VdirConfig::home_dir`] doing
    /// it at deserialize.
    pub fn new(config: VdirConfig) -> Self {
        let root = VdirPath::new(config.home_dir.to_string_lossy().into_owned());

        Self {
            inner: Inner::new(root),
        }
    }
}

impl Deref for VdirClient {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VdirClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Loads the configuration, picks the active account, then opens it.
///
/// Bails when the account carries no `[vdir]` block.
pub fn build_vdir_client(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<VdirClient> {
    let (_, name, mut account_config) = resolve_account(config_paths, account_name)?;

    let vdir_config = account_config
        .vdir
        .take()
        .ok_or_else(|| anyhow!("vdir configuration is missing for account `{name}`"))?;

    Ok(VdirClient::new(vdir_config))
}
