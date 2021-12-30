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

use std::path::PathBuf;

use aoide_core::{entity::EntityUid, media::SourcePath, util::url::BaseUrl};

use aoide_media::resolver::{SourcePathResolver, VirtualFilePathResolver};

use aoide_repo::collection::{EntityRepo, RecordId};

use super::*;

fn load_virtual_file_path_resolver<Repo>(
    repo: &Repo,
    collection_id: RecordId,
    override_root_url: Option<BaseUrl>,
) -> Result<Option<VirtualFilePathResolver>>
where
    Repo: EntityRepo,
{
    let (_, entity) = repo.load_collection_entity(collection_id)?;
    let (_path_kind, root_url) = entity.body.media_source_config.source_path.into();
    let root_url = if let Some(root_url) = root_url {
        root_url
    } else {
        return Ok(None);
    };
    let resolver = VirtualFilePathResolver::with_root_url(override_root_url.unwrap_or(root_url));
    Ok(Some(resolver))
}

fn resolve_collection_id_for_virtual_file_path<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    override_root_url: Option<BaseUrl>,
) -> Result<(RecordId, Option<VirtualFilePathResolver>)>
where
    Repo: EntityRepo,
{
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    let resolver = load_virtual_file_path_resolver(repo, collection_id, override_root_url)?;
    Ok((collection_id, resolver))
}

fn resolve_path_prefix_from_base_url(
    source_path_resolver: &impl SourcePathResolver,
    url_path_prefix: &BaseUrl,
) -> Result<SourcePath> {
    source_path_resolver
        .resolve_path_from_url(url_path_prefix)
        .map_err(|err| anyhow::format_err!("Invalid URL path prefix: {}", err).into())
}

#[derive(Debug)]
pub struct RepoContext {
    pub record_id: RecordId,
    pub vfs: Option<VfsContext>,
}

#[derive(Debug)]
pub struct VfsContext {
    pub source_path_resolver: VirtualFilePathResolver,
    pub root_path: SourcePath,
    pub root_url: BaseUrl,
}

impl RepoContext {
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
        let (record_id, source_path_resolver) =
            resolve_collection_id_for_virtual_file_path(repo, uid, override_root_url)?;
        let vfs = if let Some(source_path_resolver) = source_path_resolver {
            let root_path = root_url
                .map(|url| resolve_path_prefix_from_base_url(&source_path_resolver, url))
                .transpose()?
                .unwrap_or_default();
            let root_url = source_path_resolver
                .resolve_url_from_path(&root_path)
                .map_err(anyhow::Error::from)?;
            Some(VfsContext {
                source_path_resolver,
                root_path,
                root_url: BaseUrl::new(root_url),
            })
        } else {
            None
        };
        Ok(Self { record_id, vfs })
    }

    pub fn root_path_prefix_str<'a, 'b>(
        &'a self,
        default_root_url: Option<&'b BaseUrl>,
    ) -> Option<&'a str>
    where
        'b: 'a,
    {
        self.vfs
            .as_ref()
            .map(|vfs| vfs.root_path.as_str())
            .or_else(|| default_root_url.map(|root_url| root_url.as_str()))
            .filter(|root_path_prefix| !root_path_prefix.is_empty())
    }
}

impl VfsContext {
    pub fn build_root_file_path(&self) -> PathBuf {
        self.source_path_resolver.build_file_path(&self.root_path)
    }
}
