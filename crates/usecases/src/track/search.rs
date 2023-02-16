// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Instant;

use aoide_core::track::Entity;

use aoide_core_api::track::search::*;

use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    track::{CollectionRepo as TrackCollectionRepo, RecordHeader},
};

use crate::collection::vfs::RepoContext;

use super::*;

pub fn search<Repo>(
    repo: &mut Repo,
    collection_id: CollectionId,
    pagination: &Pagination,
    filter: Option<Filter>,
    ordering: Vec<SortOrder>,
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
        vfs_content_path_root_url,
        filter,
        ordering,
    } = params;
    let resolve_url_from_content_path = vfs_content_path_root_url.is_some();
    let collection_ctx =
        RepoContext::resolve_override(repo, collection_uid, None, vfs_content_path_root_url)?;
    let collection_id = collection_ctx.record_id;
    if resolve_url_from_content_path {
        #[cfg(not(target_family = "wasm"))]
        {
            let Some(vfs_ctx) = collection_ctx.content_path.vfs else {
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
