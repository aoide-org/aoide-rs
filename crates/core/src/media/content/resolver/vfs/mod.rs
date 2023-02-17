// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, path::PathBuf};

use anyhow::anyhow;
use url::Url;

use crate::{
    media::content::ContentPathKind,
    util::url::{is_valid_base_url, BaseUrl},
};

use super::{
    ContentPath, ContentPathResolver, ResolveFromPathError, ResolveFromUrlError, FILE_URL_SCHEME,
};

#[derive(Debug, Clone, Default)]
pub struct VfsResolver {
    pub root_url: Option<BaseUrl>,
    root_file_path: Option<PathBuf>,
    root_slash_path: Option<String>,
}

fn has_trailing_path_separator(path: &std::path::Path) -> Option<bool> {
    // Path::ends_with() cannot be used for this purpose. This
    // function is only used in a debug assertion and performance
    // doesn't matter.
    path.to_str()
        .map(|s| s.ends_with(std::path::MAIN_SEPARATOR))
}

#[cfg(not(target_family = "wasm"))]
fn path_to_slash(path: &std::path::Path) -> Option<std::borrow::Cow<'_, str>> {
    let slash_path = path_slash::PathExt::to_slash(path);
    debug_assert_eq!(
        has_trailing_path_separator(path),
        slash_path
            .as_deref()
            .map(|slash_path| slash_path.ends_with('/'))
    );
    slash_path
}

#[cfg(not(target_family = "wasm"))]
impl VfsResolver {
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
        debug_assert!(root_file_path
            .as_deref()
            .ok()
            .and_then(has_trailing_path_separator)
            .unwrap_or(true));
        let root_url = Some(root_url);
        debug_assert_eq!(
            root_url,
            root_file_path
                .as_ref()
                .ok()
                .and_then(|path| Url::from_directory_path(path).ok())
                .and_then(|url| BaseUrl::try_from(url).ok())
        );
        let root_slash_path = root_file_path
            .as_deref()
            .ok()
            .and_then(path_to_slash)
            .map(std::borrow::Cow::into_owned);
        debug_assert!(root_slash_path
            .as_ref()
            .map_or(true, |path| path.ends_with('/')));
        debug_assert_eq!(root_file_path.is_ok(), root_slash_path.is_some());
        Self {
            root_url,
            root_file_path: root_file_path.ok(),
            root_slash_path,
        }
    }

    #[must_use]
    pub fn build_file_path(&self, content_path: &ContentPath<'_>) -> PathBuf {
        let path_suffix = path_slash::PathBufExt::from_slash(content_path.as_str());
        if let Some(root_file_path) = &self.root_file_path {
            let mut path_buf = PathBuf::with_capacity(
                root_file_path.as_os_str().len() + content_path.as_str().len(),
            );
            path_buf.push(root_file_path);
            path_buf.push(path_suffix);
            path_buf
        } else {
            path_suffix
        }
    }
}

#[cfg(not(target_family = "wasm"))]
impl ContentPathResolver for VfsResolver {
    fn path_kind(&self) -> ContentPathKind {
        ContentPathKind::VirtualFilePath
    }

    fn resolve_path_from_url(
        &self,
        url: &Url,
    ) -> Result<ContentPath<'static>, ResolveFromUrlError> {
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
                    if let Some(slash_path) = path_to_slash(&file_path) {
                        if let Some(root_slash_path) = &self.root_slash_path {
                            let stripped_path = slash_path.strip_prefix(root_slash_path);
                            if let Some(stripped_path) = stripped_path {
                                return Ok(stripped_path.to_owned().into());
                            }
                        } else {
                            return Ok(slash_path.into_owned().into());
                        }
                    }
                }
                Err(ResolveFromUrlError::InvalidUrl)
            }
            Err(()) => Err(ResolveFromUrlError::InvalidUrl),
        }
    }

    fn resolve_url_from_content_path(
        &self,
        content_path: &ContentPath<'_>,
    ) -> Result<Url, ResolveFromPathError> {
        let file_path = self.build_file_path(content_path);
        let url = if content_path.is_terminal() {
            Url::from_file_path(&file_path)
        } else {
            // Preserve the trailing slash
            Url::from_directory_path(&file_path)
        }
        .map_err(|()| ResolveFromPathError::InvalidFilePath(file_path))?;
        debug_assert!(content_path.is_terminal() != url.as_str().ends_with('/'));
        debug_assert!(!content_path.is_empty() || is_valid_base_url(&url));
        Ok(url)
    }
}

#[derive(Debug)]
pub struct RemappingVfsResolver {
    /// The canonical root URL if it differs from the path resolver.
    root_url: Option<BaseUrl>,

    /// The root path relative to the canonical root URL.
    ///
    /// Denotes a subtree within the tree starting at the canonical root URL.
    /// Empty for the whole tree.
    root_path: ContentPath<'static>,

    path_resolver: VfsResolver,
}

impl RemappingVfsResolver {
    pub fn new(
        canonical_root_url: BaseUrl,
        root_url: Option<&BaseUrl>,
        override_root_url: Option<BaseUrl>,
    ) -> anyhow::Result<Self> {
        let canonical_path_resolver = VfsResolver::with_root_url(canonical_root_url);
        let root_path = root_url
            .map(|root_url| resolve_path_prefix_from_base_url(&canonical_path_resolver, root_url))
            .transpose()?
            .unwrap_or_default();
        let vfs_context = if override_root_url == canonical_path_resolver.root_url {
            // Don't override
            RemappingVfsResolver {
                root_url: None,
                root_path,
                path_resolver: canonical_path_resolver,
            }
        } else if let Some(override_root_url) = override_root_url {
            let override_path_resolver = VfsResolver::with_root_url(override_root_url);
            RemappingVfsResolver {
                root_url: canonical_path_resolver.root_url,
                root_path,
                path_resolver: override_path_resolver,
            }
        } else {
            // Don't override
            RemappingVfsResolver {
                root_url: None,
                root_path,
                path_resolver: canonical_path_resolver,
            }
        };
        Ok(vfs_context)
    }

    #[must_use]
    pub fn canonical_root_url(&self) -> &BaseUrl {
        self.root_url
            .as_ref()
            .or(self.path_resolver.root_url.as_ref())
            .expect("Some")
    }

    #[must_use]
    pub fn root_path(&self) -> &ContentPath<'_> {
        &self.root_path
    }

    #[must_use]
    fn remap_content_path<'a>(&self, content_path: &'a ContentPath<'a>) -> ContentPath<'a> {
        // Remapping is only required if both root URLs differ.
        debug_assert!(self.root_url.is_some());
        debug_assert_ne!(self.root_url, self.path_resolver.root_url);
        debug_assert!(
            self.root_path.is_empty() || content_path.as_str().starts_with(self.root_path.as_str())
        );
        if self.root_path.as_str().is_empty()
            || !content_path.as_str().starts_with(self.root_path.as_str())
        {
            return content_path.as_borrowed();
        }
        let content_path_suffix = &content_path.as_str()[self.root_path.as_str().len()..];
        ContentPath::new(Cow::Borrowed(content_path_suffix))
    }

    #[must_use]
    pub fn build_file_path(&self, content_path: &ContentPath<'_>) -> std::path::PathBuf {
        if self.root_url.is_none() {
            self.path_resolver.build_file_path(content_path)
        } else {
            let remapped_content_path = self.remap_content_path(content_path);
            self.path_resolver.build_file_path(&remapped_content_path)
        }
    }

    #[must_use]
    pub fn canonical_resolver(&self) -> &VfsResolver {
        debug_assert!(self.root_url.is_none());
        &self.path_resolver
    }

    #[must_use]
    pub fn dismantle(self) -> (BaseUrl, ContentPath<'static>) {
        let Self {
            root_url,
            root_path,
            path_resolver,
        } = self;
        let root_url = root_url.or(path_resolver.root_url).expect("Some");
        (root_url, root_path)
    }
}

impl ContentPathResolver for RemappingVfsResolver {
    fn path_kind(&self) -> ContentPathKind {
        self.path_resolver.path_kind()
    }

    fn resolve_url_from_content_path(
        &self,
        content_path: &ContentPath<'_>,
    ) -> Result<url::Url, ResolveFromPathError> {
        if self.root_url.is_none() {
            self.path_resolver
                .resolve_url_from_content_path(content_path)
        } else {
            let remapped_content_path = self.remap_content_path(content_path);
            self.path_resolver
                .resolve_url_from_content_path(&remapped_content_path)
        }
    }

    fn resolve_path_from_url(
        &self,
        url: &Url,
    ) -> Result<ContentPath<'static>, ResolveFromUrlError> {
        if self.root_url.is_none() {
            self.path_resolver.resolve_path_from_url(url)
        } else {
            unimplemented!("inverse remapping from URL to content path is not supported");
        }
    }
}

fn resolve_path_prefix_from_base_url(
    content_path_resolver: &impl ContentPathResolver,
    url_path_prefix: &BaseUrl,
) -> anyhow::Result<ContentPath<'static>> {
    content_path_resolver
        .resolve_path_from_url(url_path_prefix)
        .map_err(|err| anyhow!("Invalid URL path prefix: {err}"))
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
