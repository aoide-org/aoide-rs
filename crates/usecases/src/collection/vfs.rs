// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::Not as _;

use aoide_core::{
    collection::EntityUid,
    media::content::{ContentPath, ContentPathKind},
    util::url::BaseUrl,
};

#[cfg(not(target_family = "wasm"))]
use aoide_core::media::content::resolver::{ContentPathResolver, VirtualFilePathResolver};

use aoide_repo::collection::{EntityRepo, RecordId};

use super::*;

#[cfg(not(target_family = "wasm"))]
fn resolve_path_prefix_from_base_url(
    content_path_resolver: &impl ContentPathResolver,
    url_path_prefix: &BaseUrl,
) -> Result<ContentPath> {
    content_path_resolver
        .resolve_path_from_url(url_path_prefix)
        .map_err(|err| anyhow::format_err!("Invalid URL path prefix: {err}").into())
}

struct RepoContextProps {
    record_id: RecordId,
    content_path_kind: ContentPathKind,
    root_url: Option<BaseUrl>,
}

impl RepoContextProps {
    fn load_from_repo<Repo>(repo: &Repo, uid: &EntityUid) -> Result<Self>
    where
        Repo: EntityRepo,
    {
        let record_id = repo.resolve_collection_id(uid)?;
        let (_, entity) = repo.load_collection_entity(record_id)?;
        let (content_path_kind, root_url) = entity.raw.body.media_source_config.content_path.into();
        Ok(Self {
            record_id,
            content_path_kind,
            root_url,
        })
    }
}

#[derive(Debug)]
pub struct RepoContext {
    pub record_id: RecordId,
    pub content_path: ContentPathContext,
}

impl RepoContext {
    fn new(
        props: RepoContextProps,
        root_url: Option<&BaseUrl>,
        override_root_url: impl Into<Option<BaseUrl>>,
    ) -> Result<Self> {
        let record_id = props.record_id;
        let content_path = ContentPathContext::new(props, root_url, override_root_url)?;
        Ok(Self {
            record_id,
            content_path,
        })
    }

    pub fn resolve(
        repo: &impl EntityRepo,
        uid: &EntityUid,
        root_url: Option<&BaseUrl>,
    ) -> Result<Self> {
        Self::resolve_ext(repo, uid, root_url, None)
    }

    pub fn resolve_ext(
        repo: &impl EntityRepo,
        uid: &EntityUid,
        root_url: Option<&BaseUrl>,
        override_root_url: Option<BaseUrl>,
    ) -> Result<Self> {
        let props = RepoContextProps::load_from_repo(repo, uid)?;
        Self::new(props, root_url, override_root_url)
    }

    #[must_use]
    pub fn root_path_prefix_str<'a, 'b>(
        &'a self,
        default_root_url: Option<&'b BaseUrl>,
    ) -> Option<&'a str>
    where
        'b: 'a,
    {
        self.content_path
            .vfs
            .as_ref()
            .map(|vfs| vfs.root_path.as_str())
            .or_else(|| default_root_url.map(|root_url| root_url.as_str()))
            .filter(|root_path_prefix| root_path_prefix.is_empty().not())
    }
}

#[derive(Debug)]
pub struct ContentPathContext {
    pub kind: ContentPathKind,
    pub vfs: Option<ContentPathVfsContext>,
}

impl ContentPathContext {
    #[cfg_attr(target_family = "wasm", allow(unused_variables))]
    fn new(
        repo_props: RepoContextProps,
        root_url: Option<&BaseUrl>,
        override_root_url: impl Into<Option<BaseUrl>>,
    ) -> Result<Self> {
        let RepoContextProps {
            record_id,
            content_path_kind: kind,
            root_url: repo_root_url,
        } = repo_props;
        let vfs = match kind {
            ContentPathKind::Url | ContentPathKind::Uri | ContentPathKind::FileUrl => None,
            #[cfg(not(target_family = "wasm"))]
            ContentPathKind::VirtualFilePath => {
                let repo_root_url = if let Some(repo_root_url) = repo_root_url {
                    repo_root_url
                } else {
                    return Err(
                        anyhow::anyhow!("Missing root URL for collection {record_id:?} with content path kind {kind:?}").into(),
                    );
                };
                let path_resolver = VirtualFilePathResolver::with_root_url(
                    override_root_url.into().unwrap_or(repo_root_url),
                );
                let root_path = root_url
                    .map(|url| resolve_path_prefix_from_base_url(&path_resolver, url))
                    .transpose()?
                    .unwrap_or_default();
                let root_url = path_resolver
                    .resolve_url_from_content_path(&root_path)
                    .map_err(anyhow::Error::from)?;
                Some(ContentPathVfsContext {
                    root_url: BaseUrl::new(root_url),
                    root_path,
                    path_resolver,
                })
            }
            #[cfg(target_family = "wasm")]
            ContentPathKind::VirtualFilePath => {
                return Err(anyhow::anyhow!("Unsupported content path kind: {kind:?}").into());
            }
        };
        Ok(Self { kind, vfs })
    }
}

#[derive(Debug)]
pub struct ContentPathVfsContext {
    pub root_path: ContentPath,
    pub root_url: BaseUrl,
    #[cfg(not(target_family = "wasm"))]
    pub path_resolver: VirtualFilePathResolver,
}

#[cfg(not(target_family = "wasm"))]
impl ContentPathVfsContext {
    #[must_use]
    pub fn build_root_file_path(&self) -> std::path::PathBuf {
        self.path_resolver.build_file_path(&self.root_path)
    }
}
