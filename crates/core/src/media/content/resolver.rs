// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;

use thiserror::Error;
use url::Url;

use super::{ContentPath, ContentPathKind};

#[cfg(not(target_family = "wasm"))]
pub mod vfs;

#[derive(Error, Debug)]
pub enum ResolveFromPathError {
    #[error("invalid path")]
    InvalidPath(String),

    #[cfg(not(target_family = "wasm"))]
    #[error("invalid file path")]
    InvalidFilePath(PathBuf),

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
    fn resolve_path_from_url(&self, url: &Url)
        -> Result<ContentPath<'static>, ResolveFromUrlError>;
    fn resolve_url_from_content_path(
        &self,
        content_path: &ContentPath<'_>,
    ) -> Result<Url, ResolveFromPathError>;
}

#[derive(Debug, Clone)]
pub struct UrlResolver;

impl ContentPathResolver for UrlResolver {
    fn path_kind(&self) -> ContentPathKind {
        ContentPathKind::Url
    }

    fn resolve_path_from_url(
        &self,
        url: &Url,
    ) -> Result<ContentPath<'static>, ResolveFromUrlError> {
        Ok(url.to_string().into())
    }

    fn resolve_url_from_content_path(
        &self,
        content_path: &ContentPath<'_>,
    ) -> Result<Url, ResolveFromPathError> {
        Url::parse(content_path.as_str())
            .map_err(|_| ResolveFromPathError::InvalidPath(content_path.clone_owned().into()))
    }
}

const FILE_URL_SCHEME: &str = "file";

#[derive(Debug, Clone)]
pub struct FileUrlResolver;

impl ContentPathResolver for FileUrlResolver {
    fn path_kind(&self) -> ContentPathKind {
        ContentPathKind::FileUrl
    }

    fn resolve_path_from_url(
        &self,
        url: &Url,
    ) -> Result<ContentPath<'static>, ResolveFromUrlError> {
        if url.scheme() != FILE_URL_SCHEME {
            return Err(ResolveFromUrlError::InvalidUrl);
        }
        UrlResolver.resolve_path_from_url(url)
    }

    fn resolve_url_from_content_path(
        &self,
        content_path: &ContentPath<'_>,
    ) -> Result<Url, ResolveFromPathError> {
        let url = UrlResolver.resolve_url_from_content_path(content_path)?;
        if url.scheme() != FILE_URL_SCHEME {
            return Err(ResolveFromPathError::InvalidPath(
                content_path.to_borrowed().into_owned().into(),
            ));
        }
        Ok(url)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
