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

use std::{num::NonZeroU64, sync::Arc};

use diesel::Connection as _;
use tantivy::{
    query::{AllQuery, Query as _},
    IndexWriter,
};

use aoide_core::{entity::EntityUid, util::clock::DateTime};
use aoide_core_api::{
    sorting::SortDirection,
    track::search::{SortField, SortOrder},
    Pagination,
};
use aoide_index_tantivy::TrackFields;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::track::EntityCollector;

#[derive(Debug, Clone)]
pub enum IndexingMode {
    /// Add or replace all existing documents unconditionally
    All,

    /// Add or replace only documents that have recently been
    /// updated and stop when no more updated documents are
    /// expected.
    RecentlyUpdated,
}

/// Re-index all recently updated tracks
///
/// This task cannot be aborted, otherwise the terminate condition
/// no longer holds! Moreover the database must not be modified while
/// this task is running!
pub async fn reindex_tracks(
    track_fields: TrackFields,
    mut index_writer: IndexWriter,
    db_gatekeeper: Arc<Gatekeeper>,
    collection_uid: EntityUid,
    batch_size: NonZeroU64,
    mode: IndexingMode,
    mut progress_fn: impl FnMut(u64) + Send + 'static,
) -> anyhow::Result<u64> {
    // Obtain an exclusive database connection by pretending to
    // write although we only read. The connection is locked
    // for the whole operation!
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            let search_params = aoide_core_api::track::search::Params {
                ordering: vec![SortOrder {
                    field: SortField::UpdatedAt,
                    direction: SortDirection::Descending,
                }],
                ..Default::default()
            };
            let index_searcher = index_writer.index().reader()?.searcher();
            let index_was_empty = AllQuery.count(&index_searcher)? == 0;
            let mut offset = 0;
            let mut collector = EntityCollector::new(Vec::with_capacity(batch_size.get() as usize));
            // Last timestamp to consider for updates
            let mut last_updated_at: Option<DateTime> = None;
            connection.transaction::<_, anyhow::Error, _>(|| {
                'batch_loop: loop {
                    let pagination = Pagination {
                        offset: Some(offset),
                        limit: Some(batch_size.get()),
                    };
                    aoide_usecases_sqlite::track::search::search(
                        &*pooled_connection,
                        &collection_uid,
                        search_params.clone(),
                        &pagination,
                        &mut collector,
                    )?;
                    let mut entities = collector.finish();
                    if entities.is_empty() {
                        break;
                    }
                    for entity in &entities {
                        match mode {
                            IndexingMode::All => {
                                if !index_was_empty {
                                    // Ensure that the no document with this UID already exists
                                    let term = track_fields.uid_term(&entity.hdr.uid);
                                    index_writer.delete_term(term);
                                }
                            }
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
                                        if let Some(last_updated_at) = last_updated_at {
                                            if entity.body.updated_at < last_updated_at {
                                                // No more updated entities expected to follow
                                                break 'batch_loop;
                                            }
                                        } else {
                                            // Initialize the high watermark that will eventually
                                            // terminate the loop.
                                            last_updated_at = Some(entity.body.updated_at);
                                        }
                                        // Skip and continue with next entity
                                        continue;
                                    }
                                }
                            }
                        }
                        let doc = track_fields.create_document(entity);
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
