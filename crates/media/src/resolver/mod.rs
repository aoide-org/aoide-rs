// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::path::PathBuf;

use path_slash::PathBufExt as _;
use thiserror::Error;
use url::Url;

use aoide_core::{
    media::content::{ContentPath, ContentPathKind},
    util::url::{is_valid_base_url, BaseUrl},
};

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

pub trait ContentPathResolver {
    fn path_kind(&self) -> ContentPathKind;
    fn resolve_path_from_url(&self, url: &Url) -> Result<ContentPath, ResolveFromUrlError>;
    fn resolve_url_from_content_path(&self, path: &str) -> Result<Url, ResolveFromPathError>;
}

#[derive(Debug, Clone)]
pub struct UrlResolver;

impl ContentPathResolver for UrlResolver {
    fn path_kind(&self) -> ContentPathKind {
        ContentPathKind::Url
    }

    fn resolve_path_from_url(&self, url: &Url) -> Result<ContentPath, ResolveFromUrlError> {
        Ok(url.to_string().into())
    }

    fn resolve_url_from_content_path(&self, path: &str) -> Result<Url, ResolveFromPathError> {
        Url::parse(path).map_err(|_| ResolveFromPathError::InvalidPath)
    }
}

const FILE_URL_SCHEME: &str = "file";

#[derive(Debug, Clone)]
pub struct FileUrlResolver;

impl ContentPathResolver for FileUrlResolver {
    fn path_kind(&self) -> ContentPathKind {
        ContentPathKind::FileUrl
    }

    fn resolve_path_from_url(&self, url: &Url) -> Result<ContentPath, ResolveFromUrlError> {
        if url.scheme() != FILE_URL_SCHEME {
            return Err(ResolveFromUrlError::InvalidUrl);
        }
        UrlResolver.resolve_path_from_url(url)
    }

    fn resolve_url_from_content_path(&self, path: &str) -> Result<Url, ResolveFromPathError> {
        let url = UrlResolver.resolve_url_from_content_path(path)?;
        if url.scheme() != FILE_URL_SCHEME {
            return Err(ResolveFromPathError::InvalidPath);
        }
        Ok(url)
    }
}

#[derive(Debug, Clone, Default)]
pub struct VirtualFilePathResolver {
    root_url: Option<BaseUrl>,
    root_file_path: Option<PathBuf>,
    root_slash_path: Option<String>,
}

impl VirtualFilePathResolver {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            root_url: None,
            root_file_path: None,
            root_slash_path: None,
        }
    }

    #[must_use]
    pub fn is_valid_root_url(root_url: &Url) -> bool {
        !root_url.cannot_be_a_base() && root_url.scheme() == FILE_URL_SCHEME
    }

    #[must_use]
    pub fn with_root_url(root_url: BaseUrl) -> Self {
        debug_assert!(Self::is_valid_root_url(&root_url));
        let root_file_path = root_url.to_file_path();
        let root_url = Some(root_url);
        debug_assert_eq!(
            root_url,
            root_file_path
                .as_ref()
                .ok()
                .and_then(|path| Url::from_directory_path(path).ok())
                .and_then(|url| BaseUrl::try_from(url).ok())
        );
        let root_slash_path = root_file_path.as_ref().ok().and_then(|p| p.to_slash());
        debug_assert_eq!(root_file_path.is_ok(), root_slash_path.is_some());
        Self {
            root_url,
            root_file_path: root_file_path.ok(),
            root_slash_path,
        }
    }

    #[must_use]
    pub fn build_file_path(&self, slash_path: &str) -> PathBuf {
        let path_suffix = PathBuf::from_slash(slash_path);
        if let Some(root_file_path) = &self.root_file_path {
            let mut path_buf =
                PathBuf::with_capacity(root_file_path.as_os_str().len() + slash_path.len());
            path_buf.push(root_file_path);
            path_buf.push(path_suffix);
            path_buf
        } else {
            path_suffix
        }
    }
}

impl ContentPathResolver for VirtualFilePathResolver {
    fn path_kind(&self) -> ContentPathKind {
        ContentPathKind::VirtualFilePath
    }

    fn resolve_path_from_url(&self, url: &Url) -> Result<ContentPath, ResolveFromUrlError> {
        if let Some(root_url) = &self.root_url {
            if !url.as_str().starts_with(root_url.as_str()) {
                return Err(ResolveFromUrlError::InvalidUrl);
            }
        } else if url.scheme() != FILE_URL_SCHEME {
            return Err(ResolveFromUrlError::InvalidUrl);
        }
        match url.to_file_path() {
            Ok(file_path) => {
                if file_path.is_absolute() {
                    if let Some(slash_path) = file_path.to_slash() {
                        if let Some(root_slash_path) = &self.root_slash_path {
                            let stripped_path = slash_path.strip_prefix(root_slash_path);
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

    fn resolve_url_from_content_path(&self, slash_path: &str) -> Result<Url, ResolveFromPathError> {
        let file_path = self.build_file_path(slash_path);
        let url = if slash_path.is_empty() || slash_path.ends_with('/') {
            // Preserve the trailing slash
            Url::from_directory_path(file_path)
        } else {
            Url::from_file_path(file_path)
        }
        .map_err(|()| ResolveFromPathError::InvalidPath)?;
        debug_assert!(
            slash_path.is_empty() || slash_path.ends_with('/') == url.as_str().ends_with('/')
        );
        debug_assert!(!slash_path.is_empty() || is_valid_base_url(&url));
        Ok(url)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
