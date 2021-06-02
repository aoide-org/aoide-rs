// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    convert::TryFrom,
    fmt,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use ::url::Url;
use thiserror::Error;

/// An absolute URL that ends with a trailing path separator
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct BaseUrl(Url);

#[derive(Error, Debug)]
pub enum BaseUrlError {
    #[error("cannot be a base")]
    CannotBeABase,

    #[error("no leading path separator")]
    NoLeadingPathSeparator,

    #[error("no trailing path separator")]
    NoTrailingPathSeparator,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
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

pub fn is_file_url(url: &Url) -> bool {
    url.scheme() == FILE_SCHEME
}

impl BaseUrl {
    pub const fn new_valid(valid_base_url: Url) -> Self {
        Self(valid_base_url)
    }

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

    pub fn is_file(&self) -> bool {
        is_file_url(self)
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
            format!("{}/", s).parse()
        }
        .map_err(anyhow::Error::from)?;
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

impl DerefMut for BaseUrl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
