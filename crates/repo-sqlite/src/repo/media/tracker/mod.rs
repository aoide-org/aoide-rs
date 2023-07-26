// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{media::content::ContentPath, util::clock::OffsetDateTimeMs};
use aoide_core_api::media::tracker::{DirTrackingStatus, DirectoriesStatus};
use aoide_repo::{
    collection::RecordId as CollectionId,
    media::{source::RecordId as MediaSourceId, tracker::*, DigestBytes},
};

use crate::{
    db::{
        media_source::schema::*,
        media_tracker::{
            decode_dir_tracking_status, encode_dir_tracking_status, models::*, schema::*,
        },
    },
    prelude::*,
};

impl<'db> Repo for crate::prelude::Connection<'db> {
    fn media_tracker_update_directories_status(
        &mut self,
        updated_at: OffsetDateTimeMs,
        collection_id: CollectionId,
        path_prefix: &ContentPath<'_>,
        old_status: Option<DirTrackingStatus>,
        new_status: DirTrackingStatus,
    ) -> RepoResult<usize> {
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(sql_column_substr_prefix_eq(
                "content_path",
                path_prefix.as_str(),
            ));
        let mut query = diesel::update(target)
            .set((
                media_tracker_directory::row_updated_ms.eq(updated_at.timestamp_millis()),
                media_tracker_directory::status.eq(encode_dir_tracking_status(new_status)),
            ))
            .into_boxed();
        if let Some(old_status) = old_status {
            query = query
                .filter(media_tracker_directory::status.eq(encode_dir_tracking_status(old_status)));
        }
        query.execute(self.as_mut()).map_err(repo_error)
    }

    fn media_tracker_untrack_directories(
        &mut self,
        collection_id: CollectionId,
        path_prefix: &ContentPath<'_>,
        status: Option<DirTrackingStatus>,
    ) -> RepoResult<usize> {
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(sql_column_substr_prefix_eq(
                "content_path",
                path_prefix.as_str(),
            ));
        let subselect = target.clone().select(media_tracker_directory::row_id);
        if let Some(status) = status {
            // Filter by status
            let status_filter =
                media_tracker_directory::status.eq(encode_dir_tracking_status(status));
            diesel::delete(media_tracker_source::table.filter(
                media_tracker_source::directory_id.eq_any(subselect.filter(status_filter)),
            ))
            .execute(self.as_mut())
            .map_err(repo_error)?;
            diesel::delete(target.filter(status_filter))
                .execute(self.as_mut())
                .map_err(repo_error)
        } else {
            // Any status
            diesel::delete(
                media_tracker_source::table
                    .filter(media_tracker_source::directory_id.eq_any(subselect)),
            )
            .execute(self.as_mut())
            .map_err(repo_error)?;
            diesel::delete(target)
                .execute(self.as_mut())
                .map_err(repo_error)
        }
    }

    fn media_tracker_update_directory_digest(
        &mut self,
        updated_at: OffsetDateTimeMs,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
        digest: &DigestBytes,
    ) -> RepoResult<DirUpdateOutcome> {
        debug_assert!(!content_path.is_terminal());
        // Try to mark outdated entry as current if digest is unchanged (most likely)
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::content_path.eq(content_path.as_str()))
            .filter(media_tracker_directory::digest.eq(&digest[..]))
            // Filtering by DirTrackingStatus::Outdated allows to safely trigger a rescan even
            // if entries that have previously been marked as added or modified are still
            // pending for subsequent processing, e.g. (re-)importing their metadata.
            // Those entries will finally be skipped (see below).
            .filter(
                media_tracker_directory::status
                    .eq(encode_dir_tracking_status(DirTrackingStatus::Outdated))
                    .or(media_tracker_directory::status
                        .eq(encode_dir_tracking_status(DirTrackingStatus::Orphaned))),
            );
        let query = diesel::update(target).set((
            media_tracker_directory::row_updated_ms.eq(updated_at.timestamp_millis()),
            media_tracker_directory::status
                .eq(encode_dir_tracking_status(DirTrackingStatus::Current)),
        ));
        let rows_affected = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            return Ok(DirUpdateOutcome::Current);
        }
        // Try to mark existing entry (with any status) as modified if digest has changed (less likely)
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::content_path.eq(content_path.as_str()))
            .filter(media_tracker_directory::digest.ne(&digest[..]));
        let query = diesel::update(target).set((
            media_tracker_directory::row_updated_ms.eq(updated_at.timestamp_millis()),
            media_tracker_directory::status
                .eq(encode_dir_tracking_status(DirTrackingStatus::Modified)),
            media_tracker_directory::digest.eq(&digest[..]),
        ));
        let rows_affected = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            return Ok(DirUpdateOutcome::Updated);
        }
        // Try to add a new entry (least likely)
        let insertable = InsertableRecord::bind(
            updated_at,
            collection_id,
            content_path.as_str(),
            DirTrackingStatus::Added,
            digest,
        );
        let query =
            diesel::insert_or_ignore_into(media_tracker_directory::table).values(&insertable);
        let rows_affected = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            return Ok(DirUpdateOutcome::Inserted);
        }
        // Skip entries that have previously been marked as either added or
        // modified if their digest didn't change.
        Ok(DirUpdateOutcome::Skipped)
    }

    fn media_tracker_replace_directory_sources(
        &mut self,
        collection_id: CollectionId,
        directory_path: &ContentPath<'_>,
        media_source_ids: &[MediaSourceId],
    ) -> RepoResult<(usize, usize)> {
        let directory_id = media_tracker_directory::table
            .select(media_tracker_directory::row_id)
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::content_path.eq(directory_path.as_str()))
            .first::<RowId>(self.as_mut())
            .map_err(repo_error)?;
        let target =
            media_tracker_source::table.filter(media_tracker_source::directory_id.eq(directory_id));
        let removed = diesel::delete(target)
            .execute(self.as_mut())
            .map_err(repo_error)?;
        let mut added = 0;
        for media_source_id in media_source_ids {
            added += diesel::insert_into(media_tracker_source::table)
                .values((
                    media_tracker_source::directory_id.eq(directory_id),
                    media_tracker_source::source_id.eq(RowId::from(*media_source_id)),
                ))
                .execute(self.as_mut())
                .map_err(repo_error)?;
        }
        debug_assert_eq!(media_source_ids.len(), added);
        Ok((removed, added))
    }

    fn media_tracker_confirm_directory(
        &mut self,
        updated_at: OffsetDateTimeMs,
        collection_id: CollectionId,
        directory_path: &ContentPath<'_>,
        digest: &DigestBytes,
    ) -> RepoResult<bool> {
        debug_assert!(!directory_path.is_terminal());
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::content_path.eq(directory_path.as_str()))
            .filter(media_tracker_directory::digest.eq(&digest[..]));
        let query = diesel::update(target).set((
            media_tracker_directory::row_updated_ms.eq(updated_at.timestamp_millis()),
            media_tracker_directory::status
                .eq(encode_dir_tracking_status(DirTrackingStatus::Current)),
        ));
        let rows_affected = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        Ok(rows_affected > 0)
    }

    fn media_tracker_load_directory_tracking_status(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<DirTrackingStatus> {
        debug_assert!(!content_path.is_terminal());
        media_tracker_directory::table
            .select(media_tracker_directory::status)
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::content_path.eq(content_path.as_str()))
            .first::<i16>(self.as_mut())
            .map_err(repo_error)
            .and_then(decode_dir_tracking_status)
    }

    fn media_tracker_aggregate_directories_tracking_status(
        &mut self,
        collection_id: CollectionId,
        content_path_prefix: &ContentPath<'_>,
    ) -> RepoResult<DirectoriesStatus> {
        media_tracker_directory::table
            .group_by(media_tracker_directory::status)
            .select((media_tracker_directory::status, diesel::dsl::count_star()))
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(sql_column_substr_prefix_eq(
                "content_path",
                content_path_prefix.as_str(),
            ))
            .load::<(i16, i64)>(self.as_mut())
            .map_err(repo_error)
            .map(|v| {
                v.into_iter().fold(
                    DirectoriesStatus::default(),
                    |mut aggregate_status, (status, count)| {
                        let status = match decode_dir_tracking_status(status) {
                            Ok(status) => status,
                            Err(err) => {
                                log::error!("{err}");
                                return aggregate_status;
                            }
                        };
                        let count = (count as u64) as usize;
                        match status {
                            DirTrackingStatus::Current => {
                                debug_assert_eq!(aggregate_status.current, 0);
                                aggregate_status.current = count;
                            }
                            DirTrackingStatus::Outdated => {
                                debug_assert_eq!(aggregate_status.outdated, 0);
                                aggregate_status.outdated = count;
                            }
                            DirTrackingStatus::Added => {
                                debug_assert_eq!(aggregate_status.added, 0);
                                aggregate_status.added = count;
                            }
                            DirTrackingStatus::Modified => {
                                debug_assert_eq!(aggregate_status.modified, 0);
                                aggregate_status.modified = count;
                            }
                            DirTrackingStatus::Orphaned => {
                                debug_assert_eq!(aggregate_status.orphaned, 0);
                                aggregate_status.orphaned = count;
                            }
                        }
                        aggregate_status
                    },
                )
            })
    }

    fn media_tracker_load_directories_requiring_confirmation(
        &mut self,
        collection_id: CollectionId,
        content_path_prefix: &ContentPath<'_>,
        pagination: &Pagination,
    ) -> RepoResult<Vec<TrackedDirectory>> {
        let mut query = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            // Status is pending
            .filter(
                media_tracker_directory::status
                    .eq(encode_dir_tracking_status(DirTrackingStatus::Added))
                    .or(media_tracker_directory::status
                        .eq(encode_dir_tracking_status(DirTrackingStatus::Modified))),
            )
            // Oldest first
            .order_by(media_tracker_directory::row_updated_ms)
            // then order by URI for disambiguation
            .then_order_by(media_tracker_directory::content_path)
            .into_boxed();
        if !content_path_prefix.is_empty() {
            query = query.filter(sql_column_substr_prefix_eq(
                "content_path",
                content_path_prefix.as_str(),
            ));
        }

        // Pagination
        //FIXME: Extract into generic function crate::util::apply_pagination()
        let (limit, offset) = pagination_to_limit_offset(pagination);
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let records = query
            .load::<QueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        let (valid, errors): (Vec<_>, _) = records
            .into_iter()
            .map(TryFrom::try_from)
            .partition(Result::is_ok);
        if let Some(err) = errors.into_iter().map(Result::unwrap_err).next() {
            return Err(RepoError::Other(err));
        }
        let valid: Vec<_> = valid.into_iter().map(Result::unwrap).collect();
        Ok(valid)
    }

    fn media_tracker_relink_source(
        &mut self,
        old_source_id: MediaSourceId,
        new_source_id: MediaSourceId,
    ) -> RepoResult<bool> {
        // Drop all references to old_source_id that are obsolete and
        // could cause conflicts during the following update
        let rows_deleted = diesel::delete(
            media_tracker_source::table
                .filter(media_tracker_source::source_id.eq(RowId::from(old_source_id))),
        )
        .execute(self.as_mut())
        .map_err(repo_error)?;
        debug_assert!(rows_deleted <= 1);
        // Replace all references of new_source_id with old_source_id
        let target = media_tracker_source::table
            .filter(media_tracker_source::source_id.eq(RowId::from(new_source_id)));
        let query = diesel::update(target)
            .set(media_tracker_source::source_id.eq(RowId::from(old_source_id)));
        let rows_affected = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        Ok(rows_affected > 0)
    }

    fn media_tracker_find_untracked_sources(
        &mut self,
        collection_id: CollectionId,
        path_prefix: &ContentPath<'_>,
    ) -> RepoResult<Vec<MediaSourceId>> {
        let untracked_sources_query = media_source::table
            .select(media_source::row_id)
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(sql_column_substr_prefix_eq(
                "media_source.content_link_path",
                path_prefix.as_str(),
            ))
            .filter(
                media_source::row_id
                    .ne_all(media_tracker_source::table.select(media_tracker_source::source_id)),
            );
        untracked_sources_query
            .load::<RowId>(self.as_mut())
            .map_err(repo_error)
            .map(|v| v.into_iter().map(MediaSourceId::new).collect())
    }

    fn media_tracker_resolve_source_id_synchronized_at_by_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(MediaSourceId, Option<u64>)> {
        debug_assert!(content_path.is_terminal());
        let tracked_source_query = media_source::table
            .select((media_source::row_id, media_source::content_link_rev))
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(media_source::content_link_path.eq(content_path.as_str()))
            .filter(
                media_source::row_id
                    .eq_any(media_tracker_source::table.select(media_tracker_source::source_id)),
            );
        tracked_source_query
            .first::<(RowId, Option<i64>)>(self.as_mut())
            .map_err(repo_error)
            .map(|(row_id, content_link_rev)| {
                (row_id.into(), content_link_rev.map(|rev| rev as u64))
            })
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
