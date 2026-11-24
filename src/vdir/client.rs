//! # Vdir client
//!
//! calendula's wrapper around [`io_vdir`]'s std client, for the
//! protocol-specific subcommands.

use std::{
    fs,
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
use pimalaya_cli::error::CodedError;

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
    /// The path arrives shell-expanded, [`VdirConfig::root`] doing
    /// it at deserialize.
    pub fn new(config: VdirConfig) -> Self {
        let root = VdirPath::new(config.root.to_string_lossy().into_owned());

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

/// The directory of the collection `id` under `root`, when `id` is one
/// directory name naming a directory that exists.
pub(crate) fn collection_dir(root: &VdirPath, id: &str) -> Option<VdirPath> {
    let path = root.join(id);
    (is_dir_name(id) && Path::new(path.as_str()).is_dir()).then_some(path)
}

/// Refuses to rename the calendar `id` to `new_id` when `root` holds
/// `new_id` already, coded `calendar-duplicate`: the rename would fail on
/// a full directory with the raw OS error, and replace an empty one. A
/// `new_id` that is not one directory name is left to io-vdir.
pub(crate) fn check_rename_target_absent(root: &VdirPath, id: &str, new_id: &str) -> Result<()> {
    let target = root.join(new_id);

    if is_dir_name(new_id) && fs::symlink_metadata(target.as_str()).is_ok() {
        let message = format!("Cannot rename calendar `{id}` to `{new_id}`: it exists already");
        return Err(CodedError::new(ErrorCode::CalendarDuplicate, message).into());
    }

    Ok(())
}

/// Refuses to `verb` the item `id` into `target`, named `target_name`, when
/// it holds an item of that id already, coded `item-duplicate`: io-vdir
/// replaces the target's file, and its event would be lost.
pub(crate) fn check_item_absent(
    client: &VdirClient,
    target: &VdirPath,
    target_name: &str,
    id: &str,
    verb: &str,
) -> Result<()> {
    match client.locate_item(target.clone(), id) {
        Ok(_) => {
            let message = format!(
                "Cannot {verb} item `{id}` into `{target_name}`: it holds an item of that id already"
            );
            Err(CodedError::new(ErrorCode::ItemDuplicate, message).into())
        }
        Err(VdirClientError::VdirItemLocate(VdirItemLocateError::NotFound(_))) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// Whether `id` is one directory name: a single normal path component that
/// is the whole of `id`, so neither empty, `.` nor `..`, and holding no
/// separator, a trailing one included.
pub(crate) fn is_dir_name(id: &str) -> bool {
    let mut components = Path::new(id).components();

    match (components.next(), components.next()) {
        (Some(Component::Normal(name)), None) => name == id,
        _ => false,
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
