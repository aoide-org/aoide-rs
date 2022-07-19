// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Cow,
    ops::Deref,
    path::{Path, PathBuf},
    str::Utf8Error,
};

use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};

/// Decode percent-encoded RFD path
///
/// The returned path seems to be percent-encoded, e.g. space characters
/// are replaced by %20!?
///
/// # Errors
///
/// Fails if the path is not a valid UTF-8 string.
fn percent_decode_path(path: &Path) -> Result<PathBuf, Utf8Error> {
    let encoded = path.display().to_string();
    percent_decode_str(&encoded)
        .decode_utf8()
        .map(|decoded| Path::new(decoded.as_ref()).to_path_buf())
}

pub async fn choose_directory(dir_path: impl Into<Option<&Path>>) -> Option<OwnedDirPath> {
    log::debug!("Open rfd::AsyncFileDialog");
    let mut file_dialog = rfd::AsyncFileDialog::new();
    if let Some(dir_path) = dir_path.into() {
        file_dialog = file_dialog.set_directory(dir_path);
    }
    let dir_handle = file_dialog.pick_folder().await;
    log::debug!("rfd::AsyncFileDialog closed");
    dir_handle.and_then(|file_handle| {
        percent_decode_path(file_handle.path())
            .map_err(|err| {
                // TODO: Replace with inspect_err()
                log::warn!("Failed to decode path: {err}");
                err
            })
            .map(DirPath::from_owned)
            .ok()
    })
}

/// A `Cow<'_, Path>` with more restrictive/sensitive `PartialEq`/`Eq` semantics.
///
/// Distinguishes paths with/-out trailing slashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirPath<'p>(Cow<'p, Path>);

pub type OwnedDirPath = DirPath<'static>;

impl<'p> DirPath<'p> {
    #[must_use]
    pub const fn new(inner: Cow<'p, Path>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub const fn from_borrowed(path: &'p Path) -> Self {
        Self(Cow::Borrowed(path))
    }

    #[must_use]
    pub const fn from_owned(path_buf: PathBuf) -> OwnedDirPath {
        DirPath(Cow::Owned(path_buf))
    }

    #[must_use]
    pub fn borrowed(&self) -> DirPath<'_> {
        let Self(inner) = self;
        DirPath::from_borrowed(inner)
    }

    #[must_use]
    pub fn into_owned(self) -> OwnedDirPath {
        let Self(inner) = self;
        DirPath(Cow::Owned(inner.into_owned()))
    }
}

impl From<PathBuf> for OwnedDirPath {
    fn from(from: PathBuf) -> Self {
        Self::from_owned(from)
    }
}

impl From<OwnedDirPath> for PathBuf {
    fn from(from: OwnedDirPath) -> Self {
        let DirPath(inner) = from;
        inner.into_owned()
    }
}

impl<'p> From<&'p Path> for DirPath<'p> {
    fn from(from: &'p Path) -> Self {
        Self::from_borrowed(from)
    }
}

impl PartialEq for DirPath<'_> {
    // Using Path::as_os_str() is required to handle trailing slashes consistently!
    // https://www.reddit.com/r/rust/comments/ooh5wn/damn_trailing_slash/
    fn eq(&self, other: &Self) -> bool {
        self.as_os_str().eq(other.as_os_str())
    }
}

impl Eq for DirPath<'_> {}

impl Deref for DirPath<'_> {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
