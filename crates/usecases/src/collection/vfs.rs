// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::Not as _;

use aoide_core::{collection::EntityUid, media::content::ContentPathKind, util::url::BaseUrl};

#[cfg(not(target_family = "wasm"))]
use aoide_core::media::content::resolver::vfs::RemappingVfsResolver;

use aoide_repo::collection::{EntityRepo, RecordId};

use super::*;

#[derive(Debug, Clone)]
struct RepoContextProps {
    record_id: RecordId,
    content_path_kind: ContentPathKind,
    root_url: Option<BaseUrl>,
}

impl RepoContextProps {
    fn load_from_repo<Repo>(repo: &mut Repo, uid: &EntityUid) -> Result<Self>
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
        override_root_url: Option<BaseUrl>,
    ) -> Result<Self> {
        let record_id = props.record_id;
        let content_path = ContentPathContext::new(props, root_url, override_root_url)?;
        Ok(Self {
            record_id,
            content_path,
        })
    }

    pub fn resolve(
        repo: &mut impl EntityRepo,
        uid: &EntityUid,
        root_url: Option<&BaseUrl>,
    ) -> Result<Self> {
        Self::resolve_override(repo, uid, root_url, None)
    }

    pub fn resolve_override(
        repo: &mut impl EntityRepo,
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
            .resolver
            .as_ref()
            .map(|vfs| vfs.root_path().as_str())
            .or_else(|| default_root_url.map(|root_url| root_url.as_str()))
            .filter(|root_path_prefix| root_path_prefix.is_empty().not())
    }
}

#[derive(Debug)]
pub struct ContentPathContext {
    pub kind: ContentPathKind,
    pub resolver: Option<RemappingVfsResolver>,
}

impl ContentPathContext {
    #[cfg_attr(target_family = "wasm", allow(unused_variables))]
    fn new(
        repo_props: RepoContextProps,
        root_url: Option<&BaseUrl>,
        override_root_url: Option<BaseUrl>,
    ) -> Result<Self> {
        let RepoContextProps {
            record_id,
            content_path_kind: kind,
            root_url: canonical_root_url,
        } = repo_props;
        let resolver = match kind {
            ContentPathKind::Url | ContentPathKind::Uri | ContentPathKind::FileUrl => None,
            #[cfg(not(target_family = "wasm"))]
            ContentPathKind::VirtualFilePath => {
                let Some(canonical_root_url) = canonical_root_url else {
                    return Err(
                        anyhow::anyhow!("Missing root URL for collection {record_id:?} with content path kind {kind:?}").into(),
                    );
                };
                Some(RemappingVfsResolver::new(
                    canonical_root_url,
                    root_url,
                    override_root_url,
                )?)
            }
            #[cfg(target_family = "wasm")]
            ContentPathKind::VirtualFilePath => {
                return Err(anyhow::anyhow!("Unsupported content path kind: {kind:?}").into());
            }
        };
        Ok(Self { kind, resolver })
    }
}
