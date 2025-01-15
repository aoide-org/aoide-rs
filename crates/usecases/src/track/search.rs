// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Instant;

use aoide_core::track::Entity;
use aoide_core_api::{
    track::search::{Filter, Params, SortOrder},
    Pagination,
};
use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    track::{CollectionRepo as TrackCollectionRepo, RecordHeader},
    CollectionId, RepoResult, ReservableRecordCollector,
};

use crate::Result;

pub fn search<Repo>(
    repo: &mut Repo,
    collection_id: CollectionId,
    pagination: &Pagination,
    filter: Option<&Filter>,
    ordering: &[SortOrder],
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> RepoResult<usize>
where
    Repo: TrackCollectionRepo,
{
    let timed = Instant::now();
    let num_tracks = repo.search_tracks(collection_id, pagination, filter, ordering, collector)?;
    log::debug!(
        "Search returned {num_tracks} track(s) and took {elapsed_millis} ms",
        elapsed_millis = timed.elapsed().as_secs_f64() * 1000.0,
    );
    Ok(num_tracks)
}

#[cfg(not(target_family = "wasm"))]
pub fn search_with_params<Repo>(
    repo: &mut Repo,
    collection_uid: &aoide_core::CollectionUid,
    params: &Params,
    pagination: &Pagination,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<usize>
where
    Repo: CollectionRepo + TrackCollectionRepo,
{
    use anyhow::anyhow;

    let Params {
        resolve_url_from_content_path,
        filter,
        ordering,
    } = params;
    let collection_ctx = crate::collection::vfs::RepoContext::resolve_override(
        repo,
        collection_uid,
        None,
        resolve_url_from_content_path
            .as_ref()
            .and_then(aoide_core_api::media::source::ResolveUrlFromContentPath::override_root_url)
            .map(ToOwned::to_owned),
    )?;
    let collection_id = collection_ctx.record_id;
    if resolve_url_from_content_path.is_some() {
        let Some(resolver) = collection_ctx.content_path.resolver else {
            let path_kind = collection_ctx.content_path.kind;
            return Err(crate::Error::Other(anyhow!(
                "unsupported path kind: {path_kind:?}"
            )));
        };
        let mut collector = super::vfs::ResolveUrlFromVirtualFilePathCollector {
            resolver,
            collector,
        };
        search(
            repo,
            collection_id,
            pagination,
            filter.as_ref(),
            ordering,
            &mut collector,
        )
    } else {
        search(
            repo,
            collection_id,
            pagination,
            filter.as_ref(),
            ordering,
            collector,
        )
    }
    .map_err(Into::into)
}

#[cfg(target_family = "wasm")]
pub fn search_with_params<Repo>(
    repo: &mut Repo,
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
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    if resolve_url_from_content_path.is_some() {
        // TODO: Support relative paths for URLs?
        log::warn!("Ignoring unsupported parameter {resolve_url_from_content_path:?}");
        search(repo, collection_id, pagination, filter, ordering, collector)
    } else {
        search(repo, collection_id, pagination, filter, ordering, collector)
    }
    .map_err(Into::into)
}
