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

use num_traits::{FromPrimitive as _, ToPrimitive as _};

use aoide_core::{media::SourcePath, util::clock::DateTime};

use aoide_core_ext::media::tracker::{DirTrackingStatus, DirectoriesStatus};

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::source::RecordId as MediaSourceId,
    media::{tracker::*, DigestBytes},
};

use crate::{
    db::{
        media_source::schema::*,
        media_tracker::{models::*, schema::*},
    },
    prelude::*,
};

#[derive(QueryableByName)]
struct StatusCountRow {
    #[sql_type = "diesel::sql_types::SmallInt"]
    status: i16,

    #[sql_type = "diesel::sql_types::BigInt"]
    count: i64,
}

impl<'db> Repo for crate::prelude::Connection<'db> {
    fn media_tracker_update_directories_status(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
        old_status: Option<DirTrackingStatus>,
        new_status: DirTrackingStatus,
    ) -> RepoResult<usize> {
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(sql_column_substr_prefix_eq("path", path_prefix));
        let mut query = diesel::update(target)
            .set((
                media_tracker_directory::row_updated_ms.eq(updated_at.timestamp_millis()),
                media_tracker_directory::status.eq(new_status.to_i16().expect("new_status")),
            ))
            .into_boxed();
        if let Some(old_status) = old_status {
            query = query.filter(
                media_tracker_directory::status.eq(old_status.to_i16().expect("old_status")),
            );
        }
        query.execute(self.as_ref()).map_err(repo_error)
    }

    fn media_tracker_untrack(
        &self,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
        status: Option<DirTrackingStatus>,
    ) -> RepoResult<usize> {
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(sql_column_substr_prefix_eq("path", path_prefix));
        let subselect = target.clone().select(media_tracker_directory::row_id);
        if let Some(status) = status {
            // Filter by status
            let status_filter =
                media_tracker_directory::status.eq(status.to_i16().expect("status"));
            diesel::delete(media_tracker_source::table.filter(
                media_tracker_source::directory_id.eq_any(subselect.filter(status_filter)),
            ))
            .execute(self.as_ref())
            .map_err(repo_error)?;
            diesel::delete(target.filter(status_filter))
                .execute(self.as_ref())
                .map_err(repo_error)
        } else {
            // Any status
            diesel::delete(
                media_tracker_source::table
                    .filter(media_tracker_source::directory_id.eq_any(subselect)),
            )
            .execute(self.as_ref())
            .map_err(repo_error)?;
            diesel::delete(target)
                .execute(self.as_ref())
                .map_err(repo_error)
        }
    }

    fn media_tracker_update_directory_digest(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        path: &SourcePath,
        digest: &DigestBytes,
    ) -> RepoResult<DirUpdateOutcome> {
        debug_assert!(!path.is_terminal());
        // Try to mark outdated entry as current if digest is unchanged (most likely)
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::path.eq(path.as_ref()))
            .filter(media_tracker_directory::digest.eq(&digest[..]))
            // Filtering by DirTrackingStatus::Outdated allows to safely trigger a rescan even
            // if entries that have previously been marked as added or modified are still
            // pending for subsequent processing, e.g. (re-)importing their metadata.
            // Those entries will finally be skipped (see below).
            .filter(
                media_tracker_directory::status
                    .eq(DirTrackingStatus::Outdated.to_i16().expect("outdated"))
                    .or(media_tracker_directory::status
                        .eq(DirTrackingStatus::Orphaned.to_i16().expect("orphaned"))),
            );
        let query = diesel::update(target).set((
            media_tracker_directory::row_updated_ms.eq(updated_at.timestamp_millis()),
            media_tracker_directory::status
                .eq(DirTrackingStatus::Current.to_i16().expect("current")),
        ));
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            return Ok(DirUpdateOutcome::Current);
        }
        // Try to mark existing entry (with any status) as modified if digest has changed (less likely)
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::path.eq(path.as_ref()))
            .filter(media_tracker_directory::digest.ne(&digest[..]));
        let query = diesel::update(target).set((
            media_tracker_directory::row_updated_ms.eq(updated_at.timestamp_millis()),
            media_tracker_directory::status
                .eq(DirTrackingStatus::Modified.to_i16().expect("modified")),
            media_tracker_directory::digest.eq(&digest[..]),
        ));
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            return Ok(DirUpdateOutcome::Updated);
        }
        // Try to add a new entry (least likely)
        let insertable = InsertableRecord::bind(
            updated_at,
            collection_id,
            path,
            DirTrackingStatus::Added,
            digest,
        );
        let query =
            diesel::insert_or_ignore_into(media_tracker_directory::table).values(&insertable);
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            return Ok(DirUpdateOutcome::Inserted);
        }
        // Skip entries that have previously been marked as either added or
        // modified if their digest didn't change.
        Ok(DirUpdateOutcome::Skipped)
    }

    fn media_tracker_confirm_directory(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        path: &SourcePath,
        digest: &DigestBytes,
        media_source_ids: &[MediaSourceId],
    ) -> RepoResult<bool> {
        debug_assert!(!path.is_terminal());
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::path.eq(path.as_ref()))
            .filter(media_tracker_directory::digest.eq(&digest[..]));
        let query = diesel::update(target).set((
            media_tracker_directory::row_updated_ms.eq(updated_at.timestamp_millis()),
            media_tracker_directory::status
                .eq(DirTrackingStatus::Current.to_i16().expect("current")),
        ));
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected > 0 {
            let directory_id = media_tracker_directory::table
                .select(media_tracker_directory::row_id)
                .filter(media_tracker_directory::path.eq(path.as_ref()))
                .first::<RowId>(self.as_ref())
                .map_err(repo_error)?;
            let target = media_tracker_source::table
                .filter(media_tracker_source::directory_id.eq(directory_id));
            let _rows_affected = diesel::delete(target)
                .execute(self.as_ref())
                .map_err(repo_error)?;
            for media_source_id in media_source_ids {
                diesel::insert_into(media_tracker_source::table)
                    .values((
                        media_tracker_source::directory_id.eq(directory_id),
                        media_tracker_source::source_id.eq(RowId::from(*media_source_id)),
                    ))
                    .execute(self.as_ref())
                    .map_err(repo_error)?;
            }
        }
        Ok(rows_affected > 0)
    }

    fn media_tracker_load_directory_tracking_status(
        &self,
        collection_id: CollectionId,
        path: &SourcePath,
    ) -> RepoResult<DirTrackingStatus> {
        debug_assert!(!path.is_terminal());
        media_tracker_directory::table
            .select(media_tracker_directory::status)
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::path.eq(path.as_ref()))
            .first::<i16>(self.as_ref())
            .map_err(repo_error)
            .map(|val| DirTrackingStatus::from_i16(val).expect("DirTrackingStatus"))
    }

    fn media_tracker_aggregate_directories_tracking_status(
        &self,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
    ) -> RepoResult<DirectoriesStatus> {
        // TODO: Remove with type-safe query when group_by() is available
        /*
        media_tracker_directory::table
            .select((media_tracker_directory::status, diesel::dsl::count_star))
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(sql_column_substr_prefix_eq("path", path_prefix))
            // TODO: Replace with group_by() when available
            .filter(diesel::dsl::sql("TRUE GROUP BY status ORDER BY status"))
            .load::<(i16, usize)>(self.as_ref())
        */
        let sql = format!(
            "SELECT status, COUNT(*) as count \
        FROM media_tracker_directory \
        WHERE collection_id={collection_id} AND \
        substr(path,1,{path_prefix_len})='{escaped_path_prefix}' \
        GROUP BY status",
            collection_id = RowId::from(collection_id),
            path_prefix_len = path_prefix.len(),
            escaped_path_prefix = escape_single_quotes(path_prefix),
        );
        diesel::dsl::sql_query(sql)
            .load::<StatusCountRow>(self.as_ref())
            .map_err(repo_error)
            .map(|v| {
                v.into_iter()
                    .fold(DirectoriesStatus::default(), |mut aggregate_status, row| {
                        let StatusCountRow { status, count } = row;
                        let status =
                            DirTrackingStatus::from_i16(status).expect("DirTrackingStatus");
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
                    })
            })
    }

    fn media_tracker_load_directories_requiring_confirmation(
        &self,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
        pagination: &Pagination,
    ) -> RepoResult<Vec<TrackedDirectory>> {
        let mut query = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            // Status is pending
            .filter(
                media_tracker_directory::status
                    .eq(DirTrackingStatus::Added.to_i16().unwrap())
                    .or(media_tracker_directory::status
                        .eq(DirTrackingStatus::Modified.to_i16().unwrap())),
            )
            // Oldest first
            .order_by(media_tracker_directory::row_updated_ms)
            // then order by URI for disambiguation
            .then_order_by(media_tracker_directory::path)
            .into_boxed();
        if !path_prefix.is_empty() {
            query = query.filter(sql_column_substr_prefix_eq("path", path_prefix));
        }
        let query = apply_pagination(query, pagination);
        let records = query
            .load::<QueryableRecord>(self.as_ref())
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
        &self,
        old_source_id: MediaSourceId,
        new_source_id: MediaSourceId,
    ) -> RepoResult<bool> {
        // Drop all references to old_source_id that are obsolete and
        // could cause conflicts during the following update
        let _rows_deleted = diesel::delete(
            media_tracker_source::table
                .filter(media_tracker_source::source_id.eq(RowId::from(old_source_id))),
        )
        .execute(self.as_ref())
        .map_err(repo_error)?;
        debug_assert!(_rows_deleted <= 1);
        // Replace all references of new_source_id with old_source_id
        let target = media_tracker_source::table
            .filter(media_tracker_source::source_id.eq(RowId::from(new_source_id)));
        let query = diesel::update(target)
            .set(media_tracker_source::source_id.eq(RowId::from(old_source_id)));
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        Ok(rows_affected > 0)
    }

    fn media_tracker_find_untracked_sources(
        &self,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
    ) -> RepoResult<Vec<MediaSourceId>> {
        let untracked_sources_query = media_source::table
            .select(media_source::row_id)
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(sql_column_substr_prefix_eq(
                "media_source.path",
                path_prefix.as_str(),
            ))
            .filter(
                media_source::row_id
                    .ne_all(media_tracker_source::table.select(media_tracker_source::source_id)),
            );
        untracked_sources_query
            .load::<RowId>(self.as_ref())
            .map_err(repo_error)
            .map(|v| v.into_iter().map(MediaSourceId::new).collect())
    }

    fn media_tracker_resolve_source_id_synchronized_at_by_path(
        &self,
        collection_id: CollectionId,
        path: &SourcePath,
    ) -> RepoResult<(MediaSourceId, Option<DateTime>)> {
        debug_assert!(path.is_terminal());
        let tracked_source_query = media_source::table
            .select((
                media_source::row_id,
                media_source::synchronized_at,
                media_source::synchronized_ms,
            ))
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(media_source::path.eq(path.as_ref()))
            .filter(
                media_source::row_id
                    .eq_any(media_tracker_source::table.select(media_tracker_source::source_id)),
            );
        tracked_source_query
            .first::<(RowId, Option<String>, Option<i64>)>(self.as_ref())
            .map_err(repo_error)
            .map(|(row_id, synchronized_at, synchronized_ms)| {
                (
                    row_id.into(),
                    parse_datetime_opt(synchronized_at.as_deref(), synchronized_ms),
                )
            })
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
