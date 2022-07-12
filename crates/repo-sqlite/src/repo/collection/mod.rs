// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::dsl::count_star;

use aoide_core::{collection::*, entity::EntityRevision, util::clock::*};

use aoide_core_api::collection::{
    EntityWithSummary, MediaSourceSummary, PlaylistSummary, Summary, TrackSummary,
};

use aoide_repo::collection::*;

use crate::{
    db::{
        collection::{models::*, schema::*},
        media_source::{
            schema::*,
            select_row_id_filtered_by_collection_id as select_media_source_id_filtered_by_collection_id,
        },
        playlist::schema::*,
        track::schema::*,
    },
    prelude::*,
};

impl<'db> EntityRepo for crate::Connection<'db> {
    fn resolve_collection_entity_revision(
        &self,
        uid: &EntityUid,
    ) -> RepoResult<(RecordHeader, EntityRevision)> {
        collection::table
            .select((
                collection::row_id,
                collection::row_created_ms,
                collection::row_updated_ms,
                collection::entity_rev,
            ))
            .filter(collection::entity_uid.eq(uid.as_ref()))
            .first::<(RowId, TimestampMillis, TimestampMillis, i64)>(self.as_ref())
            .map_err(repo_error)
            .map(|(row_id, row_created_ms, row_updated_ms, entity_rev)| {
                let header = RecordHeader {
                    id: row_id.into(),
                    created_at: DateTime::new_timestamp_millis(row_created_ms),
                    updated_at: DateTime::new_timestamp_millis(row_updated_ms),
                };
                (header, entity_revision_from_sql(entity_rev))
            })
    }

    fn insert_collection_entity(
        &self,
        created_at: DateTime,
        created_entity: &Entity,
    ) -> RepoResult<RecordId> {
        let insertable = InsertableRecord::bind(created_at, created_entity);
        let query = diesel::insert_into(collection::table).values(&insertable);
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert_eq!(1, rows_affected);
        self.resolve_collection_id(&created_entity.hdr.uid)
    }

    fn touch_collection_entity_revision(
        &self,
        entity_header: &EntityHeader,
        updated_at: DateTime,
    ) -> RepoResult<(RecordHeader, EntityRevision)> {
        let EntityHeader { uid, rev } = entity_header;
        let next_rev = rev
            .next()
            .ok_or_else(|| anyhow::anyhow!("no next revision"))?;
        let touchable = TouchableRecord::bind(updated_at, next_rev);
        let target = collection::table
            .filter(collection::entity_uid.eq(uid.as_ref()))
            .filter(collection::entity_rev.eq(entity_revision_to_sql(*rev)));
        let query = diesel::update(target).set(&touchable);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        let resolved = self.resolve_collection_entity_revision(uid)?;
        if rows_affected < 1 {
            // Resolved by UID, but not touched due to revision conflict
            return Err(RepoError::Conflict);
        }
        Ok(resolved)
    }

    fn update_collection_entity(
        &self,
        id: RecordId,
        updated_at: DateTime,
        updated_entity: &Entity,
    ) -> RepoResult<()> {
        let updatable =
            UpdatableRecord::bind(updated_at, updated_entity.hdr.rev, &updated_entity.body);
        let target = collection::table.filter(collection::row_id.eq(RowId::from(id)));
        let query = diesel::update(target).set(&updatable);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn load_collection_entity(&self, id: RecordId) -> RepoResult<(RecordHeader, Entity)> {
        collection::table
            .filter(collection::row_id.eq(RowId::from(id)))
            .first::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)
            .and_then(|record| record.try_into().map_err(Into::into))
    }

    fn load_collection_entities(
        &self,
        kind: Option<&str>,
        media_source_root_url: Option<&MediaSourceRootUrlFilter>,
        with_summary: bool,
        pagination: Option<&Pagination>,
        collector: &mut dyn ReservableRecordCollector<
            Header = RecordHeader,
            Record = EntityWithSummary,
        >,
    ) -> RepoResult<()> {
        let fetch = move |pagination: Option<&_>| {
            let mut target = collection::table
                .order_by(collection::row_updated_ms.desc())
                .into_boxed();

            // Kind
            if let Some(kind) = kind {
                target = target.filter(collection::kind.eq(kind));
            }

            // Media source root URL
            if let Some(media_source_root_url) = media_source_root_url {
                match media_source_root_url {
                    MediaSourceRootUrlFilter::Equal(root_url) => {
                        if let Some(root_url) = root_url {
                            target = target
                                .filter(collection::media_source_root_url.eq(root_url.as_str()));
                        } else {
                            target = target.filter(collection::media_source_root_url.is_null());
                        }
                    }
                    MediaSourceRootUrlFilter::Prefix(prefix_url) => {
                        target = target.filter(sql_column_substr_prefix_eq(
                            "collection.media_source_root_url",
                            prefix_url.as_str(),
                        ));
                    }
                    MediaSourceRootUrlFilter::PrefixOf(prefix_of_url) => {
                        if prefix_of_url.as_str().is_empty() {
                            // Nothing to do
                            return Ok(Default::default());
                        }
                        // Post-fetch filtering (see below)
                    }
                }
            }

            // Pagination
            if let Some(pagination) = pagination {
                target = apply_pagination(target, pagination);
            }

            target
                .load::<QueryableRecord>(self.as_ref())
                .map_err(repo_error)
        };

        let filter_map =
            move |record: QueryableRecord| {
                let (record_header, entity) = record.try_into()?;
                if let Some(media_source_root_url) = media_source_root_url {
                    match media_source_root_url {
                        MediaSourceRootUrlFilter::Equal(root_url) => {
                            debug_assert_eq!(
                                root_url.as_ref(),
                                entity.body.media_source_config.content_path.root_url()
                            );
                        }
                        MediaSourceRootUrlFilter::Prefix(prefix_url) => {
                            debug_assert_eq!(
                                Some(true),
                                entity.body.media_source_config.content_path.root_url().map(
                                    |root_url| root_url.as_str().starts_with(prefix_url.as_str())
                                )
                            );
                        }
                        MediaSourceRootUrlFilter::PrefixOf(prefix_of_url) => {
                            if let Some(root_url) =
                                entity.body.media_source_config.content_path.root_url()
                            {
                                if !prefix_of_url.as_str().starts_with(root_url.as_str()) {
                                    // Discard
                                    return Ok(None);
                                }
                            } else {
                                // Discard
                                return Ok(None);
                            }
                        }
                    }
                }
                let summary = if with_summary {
                    Some(self.load_collection_summary(record_header.id)?)
                } else {
                    None
                };
                Ok(Some((record_header, EntityWithSummary { entity, summary })))
            };

        fetch_and_collect_filtered_records(pagination, fetch, filter_map, collector)
    }

    fn load_collection_summary(&self, id: RecordId) -> RepoResult<Summary> {
        let media_source_count = media_source::table
            .select(count_star())
            .filter(media_source::collection_id.eq(RowId::from(id)))
            .first::<i64>(self.as_ref())
            .map_err(repo_error)?;
        debug_assert!(media_source_count >= 0);
        let media_source_summary = MediaSourceSummary {
            total_count: media_source_count as u64,
        };
        let media_source_id_subselect = select_media_source_id_filtered_by_collection_id(id);
        let track_count = track::table
            .select(count_star())
            .filter(track::media_source_id.eq_any(media_source_id_subselect))
            .first::<i64>(self.as_ref())
            .map_err(repo_error)?;
        debug_assert!(track_count >= 0);
        let track_summary = TrackSummary {
            total_count: track_count as u64,
        };
        let playlist_count = playlist::table
            .select(count_star())
            .filter(playlist::collection_id.eq(RowId::from(id)))
            .first::<i64>(self.as_ref())
            .map_err(repo_error)?;
        debug_assert!(playlist_count >= 0);
        let playlist_summary = PlaylistSummary {
            total_count: playlist_count as u64,
        };
        Ok(Summary {
            media_sources: media_source_summary,
            tracks: track_summary,
            playlists: playlist_summary,
        })
    }

    fn purge_collection_entity(&self, id: RecordId) -> RepoResult<()> {
        let target = collection::table.filter(collection::row_id.eq(RowId::from(id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn load_all_kinds(&self) -> RepoResult<Vec<String>> {
        collection::table
            .select(collection::kind)
            .distinct()
            .load::<Option<String>>(self.as_ref())
            .map_err(repo_error)
            .map(|v| v.into_iter().flatten().collect())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
