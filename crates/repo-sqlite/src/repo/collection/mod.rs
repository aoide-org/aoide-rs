// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;
use aoide_core::{
    collection::EntityHeader,
    media::content::{ContentPath, ContentPathConfig, VirtualFilePathConfig},
    util::clock::*,
    Collection, CollectionEntity, CollectionUid, EncodedEntityUid, EntityRevision,
};
use aoide_core_api::collection::{
    EntityWithSummary, LoadScope, MediaSourceSummary, PlaylistSummary, Summary, TrackSummary,
};
use aoide_repo::collection::*;
use diesel::{connection::DefaultLoadingMode, dsl::count_star};

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

fn load_vfs_excluded_content_paths(
    db: &mut Connection<'_>,
    id: RecordId,
) -> RepoResult<Vec<ContentPath<'static>>> {
    let query = collection_vfs::table
        .select(collection_vfs::excluded_content_path)
        .filter(collection_vfs::collection_id.eq(RowId::from(id)));
    let rows = query
        .load_iter::<String, DefaultLoadingMode>(db.as_mut())
        .map_err(repo_error)?;
    rows.map(|row| row.map_err(repo_error).map(Into::into))
        .collect::<RepoResult<_>>()
}

fn purge_vfs_excluded_content_paths<'a>(
    db: &mut Connection<'_>,
    id: RecordId,
    keep: impl IntoIterator<Item = &'a ContentPath<'a>>,
) -> RepoResult<()> {
    let target = collection_vfs::table
        .filter(collection_vfs::collection_id.eq(RowId::from(id)))
        .filter(
            collection_vfs::excluded_content_path.ne_all(keep.into_iter().map(ContentPath::as_str)),
        );
    let query = diesel::delete(target);
    let rows_affected: usize = query.execute(db.as_mut()).map_err(repo_error)?;
    log::debug!("Purged {rows_affected} VFS record(s) of collection {id:?}");
    Ok(())
}

fn restore_vfs(
    db: &mut Connection<'_>,
    id: RecordId,
    collection: &mut Collection,
) -> RepoResult<()> {
    let ContentPathConfig::VirtualFilePath(VirtualFilePathConfig { excluded_paths, .. }) =
        &mut collection.media_source_config.content_path
    else {
        return Ok(());
    };
    debug_assert!(excluded_paths.is_empty());
    *excluded_paths = load_vfs_excluded_content_paths(db, id)?;
    Ok(())
}

fn store_vfs(db: &mut Connection<'_>, id: RecordId, collection: &Collection) -> RepoResult<()> {
    let ContentPathConfig::VirtualFilePath(VirtualFilePathConfig { excluded_paths, .. }) =
        &collection.media_source_config.content_path
    else {
        return purge_vfs_excluded_content_paths(db, id, std::iter::empty());
    };
    purge_vfs_excluded_content_paths(db, id, excluded_paths)?;
    for excluded_path in excluded_paths {
        let record = UpsertableVfsExcludedContentPathRecord {
            collection_id: RowId::from(id),
            excluded_content_path: excluded_path.as_str(),
        };
        let query = diesel::insert_or_ignore_into(collection_vfs::table).values(&record);
        let rows_affected = query.execute(db.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
    }
    Ok(())
}

impl<'db> EntityRepo for crate::Connection<'db> {
    fn resolve_collection_entity_revision(
        &mut self,
        uid: &CollectionUid,
    ) -> RepoResult<(RecordHeader, EntityRevision)> {
        collection::table
            .select((
                collection::row_id,
                collection::row_created_ms,
                collection::row_updated_ms,
                collection::entity_rev,
            ))
            .filter(collection::entity_uid.eq(EncodedEntityUid::from(uid).as_str()))
            .get_result::<(RowId, TimestampMillis, TimestampMillis, i64)>(self.as_mut())
            .map_err(repo_error)
            .map(|(row_id, row_created_ms, row_updated_ms, entity_rev)| {
                let header = RecordHeader {
                    id: row_id.into(),
                    created_at: OffsetDateTimeMs::from_timestamp_millis(row_created_ms),
                    updated_at: OffsetDateTimeMs::from_timestamp_millis(row_updated_ms),
                };
                (header, decode_entity_revision(entity_rev))
            })
    }

    fn insert_collection_entity(
        &mut self,
        created_at: &OffsetDateTimeMs,
        created_entity: &CollectionEntity,
    ) -> RepoResult<RecordId> {
        let insertable = InsertableRecord::bind(created_at, created_entity);
        let query = insertable.insert_into(collection::table);
        let rows_affected = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert_eq!(1, rows_affected);
        let id = self.resolve_collection_id(&created_entity.hdr.uid)?;
        store_vfs(self, id, &created_entity.body)?;
        Ok(id)
    }

    fn touch_collection_entity_revision(
        &mut self,
        entity_header: &EntityHeader,
        updated_at: &OffsetDateTimeMs,
    ) -> RepoResult<(RecordHeader, EntityRevision)> {
        let EntityHeader { uid, rev } = entity_header;
        let next_rev = rev
            .next()
            .ok_or_else(|| RepoError::Other(anyhow!("no next revision")))?;
        let touchable = TouchableRecord::bind(updated_at, next_rev);
        let encoded_uid = EncodedEntityUid::from(uid);
        let target = collection::table
            .filter(collection::entity_uid.eq(encoded_uid.as_str()))
            .filter(collection::entity_rev.eq(encode_entity_revision(*rev)));
        let query = diesel::update(target).set(&touchable);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        let resolved = self.resolve_collection_entity_revision(uid)?;
        if rows_affected < 1 {
            // Resolved by UID, but not touched due to revision conflict
            return Err(RepoError::Conflict);
        }
        Ok(resolved)
    }

    fn update_collection_entity(
        &mut self,
        id: RecordId,
        updated_at: &OffsetDateTimeMs,
        updated_entity: &CollectionEntity,
    ) -> RepoResult<()> {
        let updatable =
            UpdatableRecord::bind(updated_at, updated_entity.hdr.rev, &updated_entity.body);
        let target = collection::table.filter(collection::row_id.eq(RowId::from(id)));
        let query = diesel::update(target).set(&updatable);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        store_vfs(self, id, &updated_entity.body)?;
        Ok(())
    }

    fn load_collection_entity(
        &mut self,
        id: RecordId,
    ) -> RepoResult<(RecordHeader, CollectionEntity)> {
        let (record_header, mut entity) = collection::table
            .filter(collection::row_id.eq(RowId::from(id)))
            .get_result::<QueryableRecord>(self.as_mut())
            .map_err(repo_error)
            .and_then(|record| record.try_into().map_err(RepoError::Other))?;
        restore_vfs(self, id, &mut entity.body)?;
        Ok((record_header, entity))
    }

    #[allow(clippy::too_many_lines)] // TODO
    fn load_collection_entities(
        &mut self,
        kind_filter: Option<KindFilter<'_>>,
        media_source_root_url: Option<&MediaSourceRootUrlFilter>,
        load_scope: LoadScope,
        pagination: Option<&Pagination>,
        collector: &mut dyn ReservableRecordCollector<
            Header = RecordHeader,
            Record = EntityWithSummary,
        >,
    ) -> RepoResult<()> {
        let kind_filter = kind_filter.as_ref();
        let fetch = move |db: &mut Connection<'_>, pagination: Option<&_>| {
            let mut target = collection::table
                .order_by(collection::row_updated_ms.desc())
                .into_boxed();

            if let Some(kind_filter) = kind_filter {
                match kind_filter {
                    KindFilter::IsNone => {
                        // Note: collection::kind.eq(None) does not match NULL!
                        // <https://github.com/diesel-rs/diesel/issues/1306>
                        target = target.filter(collection::kind.is_null());
                    }
                    KindFilter::Equal(kind) => {
                        target = target.filter(collection::kind.eq(kind));
                    }
                    KindFilter::NotEqual(kind) => {
                        target = target.filter(collection::kind.ne(kind));
                    }
                }
            }

            // Media source root URL
            if let Some(media_source_root_url) = media_source_root_url {
                match media_source_root_url {
                    MediaSourceRootUrlFilter::IsNone => {
                        target = target.filter(collection::media_source_root_url.is_null());
                    }
                    MediaSourceRootUrlFilter::Equal(root_url) => {
                        target =
                            target.filter(collection::media_source_root_url.eq(root_url.as_str()));
                    }
                    MediaSourceRootUrlFilter::NotEqual(root_url) => {
                        target =
                            target.filter(collection::media_source_root_url.ne(root_url.as_str()));
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
                //FIXME: Extract into generic function crate::util::apply_pagination()
                let (limit, offset) = pagination_to_limit_offset(pagination);
                if let Some(limit) = limit {
                    target = target.limit(limit);
                }
                if let Some(offset) = offset {
                    target = target.offset(offset);
                }
            }

            target
                .load::<QueryableRecord>(db.as_mut())
                .map_err(repo_error)
        };

        let filter_map = move |db: &mut Connection<'_>, record: QueryableRecord| {
            let (record_header, mut entity) = record.try_into().map_err(RepoError::Other)?;
            if let Some(media_source_root_url) = media_source_root_url {
                match media_source_root_url {
                    MediaSourceRootUrlFilter::IsNone => {
                        debug_assert!(entity
                            .body
                            .media_source_config
                            .content_path
                            .root_url()
                            .is_none());
                    }
                    MediaSourceRootUrlFilter::Equal(root_url) => {
                        debug_assert_eq!(
                            Some(root_url),
                            entity.body.media_source_config.content_path.root_url()
                        );
                    }
                    MediaSourceRootUrlFilter::NotEqual(root_url) => {
                        debug_assert_ne!(
                            Some(root_url),
                            entity.body.media_source_config.content_path.root_url()
                        );
                    }
                    MediaSourceRootUrlFilter::Prefix(prefix_url) => {
                        debug_assert_eq!(
                            Some(true),
                            entity
                                .body
                                .media_source_config
                                .content_path
                                .root_url()
                                .map(|root_url| root_url.as_str().starts_with(prefix_url.as_str()))
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
            restore_vfs(db, record_header.id, &mut entity.body)?;
            let summary = match load_scope {
                LoadScope::Entity => None,
                LoadScope::EntityWithSummary => Some(db.load_collection_summary(record_header.id)?),
            };
            Ok(Some((record_header, EntityWithSummary { entity, summary })))
        };

        fetch_and_collect_filtered_records(self, pagination, fetch, filter_map, collector)
    }

    fn load_collection_summary(&mut self, id: RecordId) -> RepoResult<Summary> {
        let media_source_count = media_source::table
            .select(count_star())
            .filter(media_source::collection_id.eq(RowId::from(id)))
            .get_result::<i64>(self.as_mut())
            .map_err(repo_error)?;
        debug_assert!(media_source_count >= 0);
        let media_source_summary = MediaSourceSummary {
            total_count: media_source_count as u64,
        };
        let media_source_id_subselect = select_media_source_id_filtered_by_collection_id(id);
        let track_count = track::table
            .select(count_star())
            .filter(track::media_source_id.eq_any(media_source_id_subselect))
            .get_result::<i64>(self.as_mut())
            .map_err(repo_error)?;
        debug_assert!(track_count >= 0);
        let track_summary = TrackSummary {
            total_count: track_count as u64,
        };
        let playlist_count = playlist::table
            .select(count_star())
            .filter(playlist::collection_id.eq(RowId::from(id)))
            .get_result::<i64>(self.as_mut())
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

    fn purge_collection_entity(&mut self, id: RecordId) -> RepoResult<()> {
        let target = collection::table.filter(collection::row_id.eq(RowId::from(id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn load_all_kinds(&mut self) -> RepoResult<Vec<String>> {
        collection::table
            .select(collection::kind.assume_not_null())
            .filter(collection::kind.is_not_null())
            .distinct()
            .load::<String>(self.as_mut())
            .map_err(repo_error)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
