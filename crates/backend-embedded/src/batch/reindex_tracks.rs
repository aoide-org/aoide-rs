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

use std::num::NonZeroU64;

use tantivy::{IndexWriter, Searcher, Term};

use aoide_core::{entity::EntityUid, util::clock::DateTime};
use aoide_core_api::{
    sorting::SortDirection,
    track::search::{SortField, SortOrder},
    Pagination,
};
use aoide_index_tantivy::{find_track_rev, TrackFields};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

pub async fn reindex_recently_updated_tracks(
    track_fields: &TrackFields,
    index_writer: &mut IndexWriter,
    searcher: &Searcher,
    db_gatekeeper: &Gatekeeper,
    collection_uid: &EntityUid,
    batch_size: NonZeroU64,
    mut progress_fn: impl FnMut(u64),
) -> anyhow::Result<u64> {
    let mut offset = 0;
    // Last timestamp to consider for updates
    let mut last_updated_at: Option<DateTime> = None;
    loop {
        let params = aoide_core_api::track::search::Params {
            ordering: vec![SortOrder {
                field: SortField::UpdatedAt,
                direction: SortDirection::Descending,
            }],
            ..Default::default()
        };
        let pagination = Pagination {
            offset: Some(offset),
            limit: Some(batch_size.get()),
        };
        let entities =
            crate::track::search(db_gatekeeper, collection_uid.to_owned(), params, pagination)
                .await?;
        if entities.is_empty() {
            break;
        }
        for entity in entities {
            if let Some(rev) = find_track_rev(searcher, track_fields, &entity.hdr.uid)? {
                if rev < entity.hdr.rev {
                    let term = Term::from_field_bytes(track_fields.uid, entity.hdr.uid.as_ref());
                    index_writer.delete_term(term);
                } else {
                    debug_assert_eq!(rev, entity.hdr.rev);
                    // After approaching the first unmodified entity all entities
                    // with an updated_at timestamp strictly less than the current
                    // one are guaranteed to be unmodified. But we still need to
                    // consider all entities with the same timestamp to be sure.
                    if let Some(last_updated_at) = last_updated_at {
                        if entity.body.updated_at < last_updated_at {
                            // No more modified entities to follow
                            return Ok(offset);
                        }
                    } else {
                        // Initialize the high watermark that will eventually
                        // terminate the loop.
                        last_updated_at = Some(entity.body.updated_at);
                    }
                    // Skip
                    continue;
                }
            }
            offset += 1;
            let doc = track_fields.create_document(&entity);
            index_writer.add_document(doc);
        }
        progress_fn(offset);
    }
    index_writer.commit()?;
    Ok(offset)
}
