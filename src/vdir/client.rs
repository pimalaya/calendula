//! # Vdir client
//!
//! calendula's wrapper around [`io_vdir`]'s std client, for the
//! protocol-specific subcommands.

use std::{
    ops::{Deref, DerefMut},
    path::{Component, Path, PathBuf},
};

use anyhow::{Result, anyhow};
use io_vdir::{
    client::{VdirClient as Inner, VdirClientError},
    item::{
        copy::VdirItemCopyError, delete::VdirItemDeleteError, get::VdirItemGetError,
        locate::VdirItemLocateError, r#move::VdirItemMoveError,
    },
    path::VdirPath,
};

use crate::{
    cli::resolve_account,
    config::VdirConfig,
    error::{ErrorCode, calendar_not_found},
};

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

    /// The directory of the collection `id`, refused `calendar-not-found`
    /// when the home holds none of that name.
    pub fn collection_path(&self, id: &str) -> Result<VdirPath> {
        collection_dir(self.root(), id)
            .ok_or_else(|| calendar_not_found(format!("Collection `{id}` not found")))
    }
}

/// The directory of the collection `id` under `root`, when `id` is one path
/// segment, neither `.` nor `..`, naming a directory that exists.
pub(crate) fn collection_dir(root: &VdirPath, id: &str) -> Option<VdirPath> {
    let mut components = Path::new(id).components();
    let segment = matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    );
    let path = root.join(id);

    (segment && Path::new(path.as_str()).is_dir()).then_some(path)
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

/// The code of a vdir failure in an error chain: an item id naming no file
/// of the collection, or naming none it could, is `item-not-found`.
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    let locate = match cause.downcast_ref::<VdirClientError>()? {
        VdirClientError::VdirItemLocate(err)
        | VdirClientError::VdirItemGet(VdirItemGetError::Locate(err))
        | VdirClientError::VdirItemDelete(VdirItemDeleteError::Locate(err))
        | VdirClientError::VdirItemCopy(VdirItemCopyError::Locate(err))
        | VdirClientError::VdirItemMove(VdirItemMoveError::Locate(err)) => err,
        _ => return None,
    };

    match locate {
        VdirItemLocateError::NotFound(_) | VdirItemLocateError::InvalidId(_) => {
            Some(ErrorCode::ItemNotFound)
        }
        _ => None,
    }
}
