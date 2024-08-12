// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, ops::Deref, str::FromStr};

use ::url::Url;
use derive_more::{Display, Error};

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub mod fs;

/// An absolute URL that ends with a trailing path separator
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
pub struct BaseUrl(Url);

#[derive(Debug, Display, Error)]
pub enum BaseUrlError {
    #[display("cannot be a base")]
    CannotBeABase,

    #[display("no leading path separator")]
    NoLeadingPathSeparator,

    #[display("no trailing path separator")]
    NoTrailingPathSeparator,

    #[error]
    Other(anyhow::Error),
}

pub fn validate_base_url(url: &Url) -> Result<(), BaseUrlError> {
    if url.cannot_be_a_base() {
        return Err(BaseUrlError::CannotBeABase);
    }
    // Only absolute paths are permitted
    if !url.path().starts_with('/') {
        return Err(BaseUrlError::NoLeadingPathSeparator);
    }
    // Not only the path but the whole URL must end with
    // a trailing path separator!
    if !url.as_str().ends_with('/') {
        return Err(BaseUrlError::NoTrailingPathSeparator);
    }
    Ok(())
}

#[must_use]
pub fn is_valid_base_url(url: &Url) -> bool {
    validate_base_url(url).is_ok()
}

impl TryFrom<Url> for BaseUrl {
    type Error = BaseUrlError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        validate_base_url(&url)?;
        Ok(Self(url))
    }
}

const FILE_SCHEME: &str = "file";

#[must_use]
pub fn is_file_url(url: &Url) -> bool {
    url.scheme() == FILE_SCHEME
}

impl BaseUrl {
    #[must_use]
    pub const fn new_valid(valid_base_url: Url) -> Self {
        Self(valid_base_url)
    }

    #[must_use]
    pub fn new(url: Url) -> Self {
        debug_assert!(is_valid_base_url(&url));
        Self::new_valid(url)
    }

    pub fn try_autocomplete_from(url: Url) -> Result<Self, BaseUrlError> {
        if url.as_str().ends_with('/') {
            return Self::try_from(url);
        }
        // FromStr applies the autocompletion
        url.as_str().parse()
    }

    #[must_use]
    pub fn is_file(&self) -> bool {
        is_file_url(self)
    }

    pub fn parse_strict(s: &str) -> Result<Self, BaseUrlError> {
        let url = Url::parse(s)
            .map_err(anyhow::Error::from)
            .map_err(BaseUrlError::Other)?;
        url.try_into()
    }

    pub fn parse_lazy(s: &str) -> Result<Self, BaseUrlError> {
        let url = Url::parse(s)
            .map_err(anyhow::Error::from)
            .map_err(BaseUrlError::Other)?;
        Self::try_autocomplete_from(url)
    }
}

impl From<BaseUrl> for Url {
    fn from(from: BaseUrl) -> Self {
        from.0
    }
}

impl FromStr for BaseUrl {
    type Err = BaseUrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: Url = if s.ends_with('/') {
            s.parse()
        } else {
            // Autocomplete the string before parsing
            format!("{s}/").parse()
        }
        .map_err(anyhow::Error::from)
        .map_err(BaseUrlError::Other)?;
        Self::try_from(url)
    }
}

impl AsRef<Url> for &BaseUrl {
    fn as_ref(&self) -> &Url {
        &self.0
    }
}

impl Deref for BaseUrl {
    type Target = Url;

    fn deref(&self) -> &Url {
        &self.0
    }
}

impl fmt::Display for BaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
