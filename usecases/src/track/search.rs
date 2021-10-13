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

use std::time::Instant;

use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    track::{EntityRepo, RecordHeader},
};

use aoide_core::{usecases::track::search::*, util::url::BaseUrl};

use crate::collection::load_virtual_file_path_resolver;

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
    Repo: EntityRepo,
{
    let timed = Instant::now();
    let count =
        repo.search_collected_tracks(collection_id, pagination, filter, ordering, collector)?;
    tracing::debug!(
        "Search returned {} tracks and took {} ms",
        count,
        (timed.elapsed().as_micros() / 1000) as f64,
    );
    Ok(count)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Params {
    pub resolve_url_from_path: bool,
    pub override_root_url: Option<BaseUrl>,
}

pub fn search_with_params<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    pagination: &Pagination,
    filter: Option<SearchFilter>,
    ordering: Vec<SortOrder>,
    params: Params,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<usize>
where
    Repo: EntityRepo + CollectionRepo,
{
    let Params {
        override_root_url,
        resolve_url_from_path,
    } = params;
    debug_assert!(resolve_url_from_path || override_root_url.is_none());
    if resolve_url_from_path {
        let source_path_resolver =
            load_virtual_file_path_resolver(repo, collection_id, override_root_url)?;
        let mut collector = ResolveUrlFromVirtualFilePathCollector {
            source_path_resolver,
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
        // Providing a base URL without using it to resolve virtual file paths
        // does no harm but doesn't make any sense and is probably unintended.
        debug_assert!(override_root_url.is_none());
        search(repo, collection_id, pagination, filter, ordering, collector)
    }
    .map_err(Into::into)
}
