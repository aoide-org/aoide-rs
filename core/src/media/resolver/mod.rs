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

use super::{SourcePath, SourcePathKind};

use path_slash::PathBufExt as _;
use std::path::PathBuf;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ResolveFromPathError {
    #[error("invalid path")]
    InvalidPath,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum ResolveFromUrlError {
    #[error("invalid URL")]
    InvalidUrl,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub trait SourcePathResolver {
    fn path_kind(&self) -> SourcePathKind;
    fn resolve_path_from_url(&self, url: &Url) -> Result<SourcePath, ResolveFromUrlError>;
    fn resolve_url_from_path(&self, path: &str) -> Result<Url, ResolveFromPathError>;
}

#[derive(Debug, Clone)]
pub struct UrlEncodedResolver;

impl SourcePathResolver for UrlEncodedResolver {
    fn path_kind(&self) -> SourcePathKind {
        SourcePathKind::UrlEncoded
    }

    fn resolve_path_from_url(&self, url: &Url) -> Result<SourcePath, ResolveFromUrlError> {
        Ok(url.to_string().into())
    }

    fn resolve_url_from_path(&self, path: &str) -> Result<Url, ResolveFromPathError> {
        Url::parse(path).map_err(|_| ResolveFromPathError::InvalidPath)
    }
}

const FILE_URL_SCHEME: &str = "file";

#[derive(Debug, Clone, Default)]
pub struct LocalFileResolver {
    base_url: Option<Url>,
    base_file_path: Option<PathBuf>,
    base_slash_path: Option<String>,
}

impl LocalFileResolver {
    pub const fn new() -> Self {
        Self {
            base_url: None,
            base_file_path: None,
            base_slash_path: None,
        }
    }

    pub fn is_valid_base_url(base_url: &Url) -> bool {
        !base_url.cannot_be_a_base()
            && base_url.scheme() == FILE_URL_SCHEME
            && base_url.as_str().ends_with('/')
    }

    pub fn with_base_url(base_url: Url) -> Self {
        debug_assert!(Self::is_valid_base_url(&base_url));
        let base_file_path = base_url.to_file_path();
        let base_url = Some(base_url);
        debug_assert_eq!(
            base_url,
            base_file_path
                .as_ref()
                .ok()
                .and_then(|path| Url::from_directory_path(path).ok())
        );
        let base_slash_path = base_file_path.as_ref().ok().and_then(|p| p.to_slash());
        debug_assert_eq!(base_file_path.is_ok(), base_slash_path.is_some());
        Self {
            base_url,
            base_file_path: base_file_path.ok(),
            base_slash_path,
        }
    }

    pub fn build_file_path(&self, slash_path: &str) -> PathBuf {
        if let Some(base_file_path) = &self.base_file_path {
            let mut path_buf =
                PathBuf::with_capacity(base_file_path.as_os_str().len() + slash_path.len());
            path_buf.push(base_file_path);
            path_buf.push(PathBuf::from_slash(slash_path));
            path_buf
        } else {
            PathBuf::from_slash(slash_path)
        }
    }
}

impl SourcePathResolver for LocalFileResolver {
    fn path_kind(&self) -> SourcePathKind {
        SourcePathKind::LocalFile
    }

    fn resolve_path_from_url(&self, url: &Url) -> Result<SourcePath, ResolveFromUrlError> {
        if let Some(base_url) = &self.base_url {
            if !url.as_str().starts_with(base_url.as_str()) {
                return Err(ResolveFromUrlError::InvalidUrl);
            }
        } else if url.scheme() != FILE_URL_SCHEME {
            return Err(ResolveFromUrlError::InvalidUrl);
        }
        match url.to_file_path() {
            Ok(file_path) => {
                if file_path.is_absolute() {
                    if let Some(slash_path) = file_path.to_slash() {
                        if let Some(base_slash_path) = &self.base_slash_path {
                            let stripped_path = slash_path.strip_prefix(base_slash_path);
                            if let Some(stripped_path) = stripped_path {
                                return Ok(stripped_path.to_owned().into());
                            }
                        } else {
                            return Ok(slash_path.into());
                        }
                    }
                }
                Err(ResolveFromUrlError::InvalidUrl)
            }
            Err(()) => Err(ResolveFromUrlError::InvalidUrl),
        }
    }

    fn resolve_url_from_path(&self, slash_path: &str) -> Result<Url, ResolveFromPathError> {
        Url::from_file_path(self.build_file_path(slash_path))
            .map_err(|()| ResolveFromPathError::InvalidPath)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
