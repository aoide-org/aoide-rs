// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::num::NonZeroU64;

use diesel::Connection as _;
use tantivy::{
    query::{AllQuery, Query as _},
    IndexWriter,
};

use aoide_core::util::clock::OffsetDateTimeMs;
use aoide_core_api::{
    track::search::{SortField, SortOrder},
    Pagination, SortDirection,
};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::{prelude::*, track::EntityCollector};

#[derive(Debug, Clone)]
pub enum IndexingMode {
    /// Delete all existing documents and re-populate the index.
    All,

    /// Add or replace only documents that have recently been
    /// updated and stop when no more updated documents are
    /// expected.
    RecentlyUpdated,
}

#[cfg(feature = "tantivy")]
/// Re-index all or recently updated tracks
///
/// This task cannot be aborted, otherwise the terminate condition
/// no longer holds! Moreover the database must not be modified while
/// this task is running!
///
/// The `mode` defaults to `RecentlyUpdated` if unspecified. It is
/// irrelevant and ignored if the index is empty.
pub async fn reindex_tracks(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    track_fields: aoide_search_index_tantivy::TrackFields,
    batch_size: NonZeroU64,
    mode: Option<IndexingMode>,
    mut index_writer: IndexWriter,
    mut progress_fn: impl FnMut(u64) + Send + 'static,
) -> anyhow::Result<u64> {
    // Obtain an exclusive database connection by pretending to
    // write although we only read. The connection is locked
    // for the whole operation!
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            let search_params = aoide_core_api::track::search::Params {
                ordering: vec![SortOrder {
                    field: SortField::UpdatedAt,
                    direction: SortDirection::Descending,
                }],
                ..Default::default()
            };
            let index_searcher = index_writer.index().reader()?.searcher();
            let mode = if AllQuery.count(&index_searcher)? > 0 {
                mode.unwrap_or(IndexingMode::RecentlyUpdated)
            } else {
                IndexingMode::All
            };
            match mode {
                IndexingMode::All => {
                    index_writer.delete_all_documents()?;
                }
                IndexingMode::RecentlyUpdated => (),
            }
            let mut offset = 0;
            #[allow(clippy::cast_possible_truncation)]
            let mut collector = EntityCollector::new(Vec::with_capacity(batch_size.get() as usize));
            // Last timestamp to consider for updates
            let mut last_updated_at: Option<OffsetDateTimeMs> = None;
            connection.transaction::<_, anyhow::Error, _>(|connection| {
                'batch_loop: loop {
                    let pagination = Pagination {
                        offset: Some(offset),
                        limit: Some(batch_size.get()),
                    };
                    aoide_usecases_sqlite::track::search::search(
                        connection,
                        &collection_uid,
                        &search_params,
                        &pagination,
                        &mut collector,
                    )?;
                    let mut entities = collector.finish();
                    if entities.is_empty() {
                        break;
                    }
                    for entity in &entities {
                        match mode {
                            IndexingMode::All => (),
                            IndexingMode::RecentlyUpdated => {
                                if let Some(rev) = track_fields
                                    .find_rev_by_uid(&index_searcher, &entity.hdr.uid)?
                                {
                                    if rev < entity.hdr.rev {
                                        let term = track_fields.uid_term(&entity.hdr.uid);
                                        index_writer.delete_term(term);
                                    } else {
                                        debug_assert_eq!(rev, entity.hdr.rev);
                                        // After approaching the first unmodified entity all entities
                                        // with an updated_at timestamp strictly less than the current
                                        // one are guaranteed to be unmodified. But we still need to
                                        // consider all entities with the same timestamp to be sure.
                                        if let Some(last_updated_at) = &last_updated_at {
                                            if entity.body.updated_at < *last_updated_at {
                                                // No more updated entities expected to follow
                                                break 'batch_loop;
                                            }
                                        } else {
                                            // Initialize the high watermark that will eventually
                                            // terminate the loop.
                                            last_updated_at = Some(entity.body.updated_at.clone());
                                        }
                                        // Skip and continue with next entity
                                        offset += 1;
                                        continue;
                                    }
                                }
                            }
                        }
                        // TODO: Load play counter
                        let play_counter = None;
                        let doc = track_fields.create_document(
                            Some(&collection_uid),
                            entity,
                            play_counter,
                        );
                        index_writer.add_document(doc)?;
                        offset += 1;
                    }
                    // Reuse the capacity of the allocated entities for the next round
                    entities.clear();
                    collector = EntityCollector::new(entities);
                    progress_fn(offset);
                }
                index_writer.commit()?;
                Ok(offset)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
