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

use aoide_core::entity::EntityUid;
use aoide_core_api::{media::source::ResolveUrlFromPath, track::search::*};

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
    let count = repo.search_tracks(collection_id, pagination, filter, ordering, collector)?;
    log::debug!(
        "Search returned {} tracks and took {} ms",
        count,
        (timed.elapsed().as_micros() / 1000) as f64,
    );
    Ok(count)
}

pub fn search_with_params<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: Params,
    pagination: &Pagination,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<usize>
where
    Repo: CollectionRepo + TrackCollectionRepo,
{
    let Params {
        resolve_url_from_path,
        filter,
        ordering,
    } = params;
    let collection_ctx = RepoContext::resolve_ext(
        repo,
        collection_uid,
        None,
        resolve_url_from_path
            .as_ref()
            .and_then(ResolveUrlFromPath::override_root_url)
            .map(ToOwned::to_owned),
    )?;
    let collection_id = collection_ctx.record_id;
    if resolve_url_from_path.is_some() {
        let vfs_ctx = if let Some(vfs_ctx) = collection_ctx.source_path.vfs {
            vfs_ctx
        } else {
            return Err(anyhow::anyhow!(
                "Unsupported path kind: {:?}",
                collection_ctx.source_path.kind
            )
            .into());
        };
        let mut collector = ResolveUrlFromVirtualFilePathCollector {
            source_path_resolver: vfs_ctx.path_resolver,
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
    } else {
        search(repo, collection_id, pagination, filter, ordering, collector)
    }
    .map_err(Into::into)
}
