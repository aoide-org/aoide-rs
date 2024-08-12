// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;

use derive_more::{Display, Error};
use url::Url;

use super::{ContentPath, ContentPathKind};

#[cfg(not(target_family = "wasm"))]
pub mod vfs;

#[derive(Debug, Error, Display)]
pub enum ResolveFromPathError {
    #[display("invalid path: {_0}")]
    InvalidPath(#[error(ignore)] String),

    #[cfg(not(target_family = "wasm"))]
    #[display("invalid file path: {}", _0.display())]
    InvalidFilePath(#[error(ignore)] PathBuf),

    #[error]
    Other(anyhow::Error),
}

#[derive(Debug, Error, Display)]
pub enum ResolveFromUrlError {
    #[display("invalid URL")]
    InvalidUrl,

    Other(anyhow::Error),
}

pub trait ContentPathResolver {
    fn path_kind(&self) -> ContentPathKind;
    fn resolve_path_from_url(
        &self,
        content_url: &Url,
    ) -> Result<Option<ContentPath<'static>>, ResolveFromUrlError>;
    fn resolve_url_from_path(
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
        content_url: &Url,
    ) -> Result<Option<ContentPath<'static>>, ResolveFromUrlError> {
        Ok(Some(content_url.to_string().into()))
    }

    fn resolve_url_from_path(
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
        content_url: &Url,
    ) -> Result<Option<ContentPath<'static>>, ResolveFromUrlError> {
        if content_url.scheme() != FILE_URL_SCHEME {
            return Err(ResolveFromUrlError::InvalidUrl);
        }
        UrlResolver.resolve_path_from_url(content_url)
    }

    fn resolve_url_from_path(
        &self,
        content_path: &ContentPath<'_>,
    ) -> Result<Url, ResolveFromPathError> {
        let content_url = UrlResolver.resolve_url_from_path(content_path)?;
        if content_url.scheme() != FILE_URL_SCHEME {
            return Err(ResolveFromPathError::InvalidPath(
                content_path.to_borrowed().into_owned().into(),
            ));
        }
        Ok(content_url)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
