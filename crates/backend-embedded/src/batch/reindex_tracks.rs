// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::num::NonZeroU64;

use diesel::Connection as _;
use tantivy::{
    IndexWriter,
    query::{AllQuery, Query as _},
};

use aoide_repo::{collection::EntityRepo as _, track::CollectionRepo as _, track::EntityRepo as _};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::prelude::*;

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
        .spawn_blocking_write_task_flattened(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
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
            connection.transaction::<_, anyhow::Error, _>(|connection| {
                let mut repo = aoide_repo_sqlite::Connection::new(connection);
                let collection_id = repo.resolve_collection_id(&collection_uid)?;
                // The number of visited tracks
                let mut visited = 0;
                // Keyset pagination
                let mut after = None;
                // Last timestamp to consider for updates
                let mut last_updated_at = None;
                let mut finished = false;
                while !finished {
                    let headers = repo.load_recently_updated_track_headers(
                        collection_id,
                        after,
                        batch_size.get(),
                    )?;
                    let Some((last_record_header, _)) = headers.last() else {
                        break;
                    };
                    after = Some((last_record_header.updated_at, last_record_header.id));
                    let mut ids_to_index = Vec::with_capacity(headers.len());
                    for (record_header, entity_header) in &headers {
                        match mode {
                            IndexingMode::All => (),
                            IndexingMode::RecentlyUpdated => {
                                if let Some(rev) = track_fields
                                    .find_rev_by_uid(&index_searcher, &entity_header.uid)?
                                {
                                    if rev < entity_header.rev {
                                        let term = track_fields.uid_term(&entity_header.uid);
                                        index_writer.delete_term(term);
                                    } else {
                                        debug_assert_eq!(rev, entity_header.rev);
                                        // After approaching the first unmodified entity all entities
                                        // with an updated_at timestamp strictly less than the current
                                        // one are guaranteed to be unmodified. But we still need to
                                        // consider all entities with the same timestamp to be sure.
                                        if let Some(last_updated_at) = last_updated_at {
                                            if record_header.updated_at < last_updated_at {
                                                // No more updated entities expected to follow
                                                finished = true;
                                                break;
                                            }
                                        } else {
                                            // Initialize the high watermark that will eventually
                                            // terminate the loop.
                                            last_updated_at = Some(record_header.updated_at);
                                        }
                                        // Skip and continue with next entity
                                        visited += 1;
                                        continue;
                                    }
                                }
                            }
                        }
                        ids_to_index.push(record_header.id);
                        visited += 1;
                    }
                    // Only load the entities that need to be (re-)indexed.
                    for (_, entity) in repo.load_track_entities(&ids_to_index)? {
                        // TODO: Load play counter
                        let play_counter = None;
                        let doc = track_fields.create_document(
                            Some(&collection_uid),
                            &entity,
                            play_counter,
                        );
                        index_writer.add_document(doc)?;
                    }
                    if !finished {
                        progress_fn(visited);
                    }
                }
                index_writer.commit()?;
                Ok(visited)
            })
        })
        .await
}
