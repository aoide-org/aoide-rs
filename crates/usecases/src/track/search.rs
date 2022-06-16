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

use std::time::Instant;

use aoide_core::track::Entity;

use aoide_core_api::{media::source::ResolveUrlFromContentPath, track::search::*};

use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    track::{CollectionRepo as TrackCollectionRepo, RecordHeader},
};

use crate::collection::vfs::RepoContext;

use super::*;

pub fn search<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    pagination: &Pagination,
    filter: Option<SearchFilter>,
    ordering: Vec<SortOrder>,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> RepoResult<usize>
where
    Repo: TrackCollectionRepo,
{
    let timed = Instant::now();
    let num_tracks = repo.search_tracks(collection_id, pagination, filter, ordering, collector)?;
    log::debug!(
        "Search returned {num_tracks} track(s) and took {} ms",
        (timed.elapsed().as_micros() / 1000) as f64,
    );
    Ok(num_tracks)
}

pub fn search_with_params<Repo>(
    repo: &Repo,
    collection_uid: &CollectionUid,
    params: Params,
    pagination: &Pagination,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<usize>
where
    Repo: CollectionRepo + TrackCollectionRepo,
{
    let Params {
        resolve_url_from_content_path,
        filter,
        ordering,
    } = params;
    let collection_ctx = RepoContext::resolve_ext(
        repo,
        collection_uid,
        None,
        resolve_url_from_content_path
            .as_ref()
            .and_then(ResolveUrlFromContentPath::override_root_url)
            .map(ToOwned::to_owned),
    )?;
    let collection_id = collection_ctx.record_id;
    if resolve_url_from_content_path.is_some() {
        #[cfg(not(target_family = "wasm"))]
        {
            let vfs_ctx = if let Some(vfs_ctx) = collection_ctx.content_path.vfs {
                vfs_ctx
            } else {
                let path_kind = collection_ctx.content_path.kind;
                return Err(anyhow::anyhow!("Unsupported path kind: {path_kind:?}").into());
            };
            let mut collector = super::vfs::ResolveUrlFromVirtualFilePathCollector {
                content_path_resolver: vfs_ctx.path_resolver,
                collector,
            };
            search(
                repo,
                collection_id,
                pagination,
                filter,
                ordering,
                &mut collector,
            )
        }
        #[cfg(target_family = "wasm")]
        {
            // TODO: Support relative paths for URLs?
            log::warn!("Ignoring unsupported parameter {resolve_url_from_content_path:?}");
            search(repo, collection_id, pagination, filter, ordering, collector)
        }
    } else {
        search(repo, collection_id, pagination, filter, ordering, collector)
    }
    .map_err(Into::into)
}
