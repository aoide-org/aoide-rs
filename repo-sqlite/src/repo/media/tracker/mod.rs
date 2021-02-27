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

use crate::{
    db::media_tracker::{models::*, schema::*},
    prelude::*,
};

use aoide_core::util::clock::DateTime;

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::source::RecordId as MediaSourceId,
    media::{tracker::*, DigestBytes},
};

use num_traits::{FromPrimitive as _, ToPrimitive as _};

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
        uri_prefix: &str,
        old_status: Option<DirTrackingStatus>,
        new_status: DirTrackingStatus,
    ) -> RepoResult<usize> {
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(diesel::dsl::sql(&format!(
                "substr(uri,1,{})='{}'",
                uri_prefix.len(),
                escape_single_quotes(uri_prefix),
            )));
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
        uri_prefix: &str,
        status: Option<DirTrackingStatus>,
    ) -> RepoResult<usize> {
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(diesel::dsl::sql(&format!(
                "substr(uri,1,{})='{}'",
                uri_prefix.len(),
                escape_single_quotes(uri_prefix),
            )));
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
        uri: &str,
        digest: &DigestBytes,
    ) -> RepoResult<DirUpdateOutcome> {
        // Try to mark outdated entry as current if digest is unchanged (most likely)
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::uri.eq(uri))
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
            .filter(media_tracker_directory::uri.eq(uri))
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
            uri,
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
        uri: &str,
        digest: &DigestBytes,
        media_source_ids: &[MediaSourceId],
    ) -> RepoResult<bool> {
        let target = media_tracker_directory::table
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::uri.eq(uri))
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
                .filter(media_tracker_directory::uri.eq(uri))
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
        uri: &str,
    ) -> RepoResult<DirTrackingStatus> {
        media_tracker_directory::table
            .select(media_tracker_directory::status)
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(media_tracker_directory::uri.eq(uri))
            .first::<i16>(self.as_ref())
            .map_err(repo_error)
            .map(|val| DirTrackingStatus::from_i16(val).expect("DirTrackingStatus"))
    }

    fn media_tracker_aggregate_directories_tracking_status(
        &self,
        collection_id: CollectionId,
        uri_prefix: &str,
    ) -> RepoResult<DirectoriesStatusSummary> {
        // TODO: Remove with type-safe query when group_by() is available
        /*
        media_tracker_directory::table
            .select((media_tracker_directory::status, diesel::dsl::count_star))
            .filter(media_tracker_directory::collection_id.eq(RowId::from(collection_id)))
            .filter(diesel::dsl::sql(&format!(
                "substr(uri,1,{})='{}'",
                uri_prefix.len(),
                escape_single_quotes(uri_prefix),
            )))
            // TODO: Replace with group_by() when available
            .filter(diesel::dsl::sql("TRUE GROUP BY status ORDER BY status"))
            .load::<(i16, usize)>(self.as_ref())
        */
        let sql = format!(
            "SELECT status, COUNT(*) as count \
        FROM media_tracker_directory \
        WHERE collection_id={collection_id} AND \
        substr(uri,1,{uri_prefix_len})='{escaped_uri_prefix}' \
        GROUP BY status",
            collection_id = RowId::from(collection_id),
            uri_prefix_len = uri_prefix.len(),
            escaped_uri_prefix = escape_single_quotes(uri_prefix),
        );
        diesel::dsl::sql_query(sql)
            .load::<StatusCountRow>(self.as_ref())
            .map_err(repo_error)
            .map(|v| {
                v.into_iter().fold(
                    DirectoriesStatusSummary::default(),
                    |mut aggregate_status, row| {
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
                    },
                )
            })
    }

    fn media_tracker_load_directories_requiring_confirmation(
        &self,
        collection_id: CollectionId,
        uri_prefix: Option<&str>,
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
            .then_order_by(media_tracker_directory::uri)
            .into_boxed();
        if let Some(uri_prefix) = uri_prefix {
            query = query.filter(diesel::dsl::sql(&format!(
                "substr(uri,1,{})='{}'",
                uri_prefix.len(),
                escape_single_quotes(uri_prefix),
            )))
        }
        let query = apply_pagination(query, pagination);
        query
            .load::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)
            .map(|v| v.into_iter().map(Into::into).collect())
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
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
