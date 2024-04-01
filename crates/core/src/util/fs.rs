// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Cow,
    ops::Deref,
    path::{Path, PathBuf},
};

/// A `Cow<'_, Path>` with more restrictive/sensitive `PartialEq`/`Eq` semantics.
///
/// Distinguishes paths with/-out trailing slashes.
#[derive(Debug, Clone, Default)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct DirPath<'p>(Cow<'p, Path>);

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
    pub const fn from_owned(path_buf: PathBuf) -> DirPath<'static> {
        DirPath(Cow::Owned(path_buf))
    }

    #[must_use]
    pub fn borrowed(&self) -> DirPath<'_> {
        let Self(inner) = self;
        DirPath::from_borrowed(inner)
    }

    #[must_use]
    pub fn into_owned(self) -> DirPath<'static> {
        let Self(inner) = self;
        DirPath(Cow::Owned(inner.into_owned()))
    }
}

impl AsRef<Path> for DirPath<'_> {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<PathBuf> for DirPath<'static> {
    fn from(from: PathBuf) -> Self {
        Self::from_owned(from)
    }
}

impl From<DirPath<'static>> for PathBuf {
    fn from(from: DirPath<'static>) -> Self {
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
