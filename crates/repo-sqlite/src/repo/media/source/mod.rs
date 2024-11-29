// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::prelude::*;

use aoide_core::{
    media::{content::ContentPath, Source},
    util::clock::OffsetDateTimeMs,
};
use aoide_core_api::filtering::StringPredicate;
use aoide_repo::{
    media::source::{CollectionRepo, RecordHeader, Repo},
    CollectionId, MediaSourceId, RepoError, RepoResult,
};

use crate::{
    db::{
        media_source::{models::*, schema::*, select_row_id_filtered_by_content_path_predicate},
        media_tracker::schema::*,
        track::schema::*,
    },
    repo_error,
    util::{escape_single_quotes, sql_column_substr_prefix_eq},
    Connection, RowId,
};

impl Repo for Connection<'_> {
    fn update_media_source(
        &mut self,
        id: MediaSourceId,
        updated_at: &OffsetDateTimeMs,
        updated_source: &Source,
    ) -> RepoResult<()> {
        let updatable = UpdatableRecord::bind(updated_at, updated_source);
        let target = media_source::table.filter(media_source::row_id.eq(RowId::from(id)));
        let query = diesel::update(target).set(&updatable);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn purge_media_source(&mut self, id: MediaSourceId) -> RepoResult<()> {
        let target = media_source::table.filter(media_source::row_id.eq(RowId::from(id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn load_media_source(&mut self, id: MediaSourceId) -> RepoResult<(RecordHeader, Source)> {
        media_source::table
            .filter(media_source::row_id.eq(RowId::from(id)))
            .get_result::<QueryableRecord>(self.as_mut())
            .map_err(repo_error)
            .and_then(|record| record.try_into().map_err(RepoError::Other))
    }
}

impl CollectionRepo for Connection<'_> {
    fn resolve_media_source_id_synchronized_at_by_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(MediaSourceId, Option<u64>)> {
        debug_assert!(!content_path.as_str().ends_with('/'));
        media_source::table
            .select((media_source::row_id, media_source::content_link_rev))
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(media_source::content_link_path.eq(content_path.as_str()))
            .get_result::<(RowId, Option<i64>)>(self.as_mut())
            .map(|(row_id, content_link_rev)| {
                (row_id.into(), content_link_rev.map(|rev| rev as u64))
            })
            .map_err(repo_error)
    }

    fn resolve_media_source_ids_by_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<Vec<MediaSourceId>> {
        let query = media_source::table
            .select(media_source::row_id)
            // Reuse the tested subselect with reliable predicate filtering
            // even if it might be slightly less efficient! The query optimizer
            // should detect this.
            .filter(
                media_source::row_id.eq_any(select_row_id_filtered_by_content_path_predicate(
                    collection_id,
                    content_path_predicate,
                )),
            );
        let rows = query
            .load_iter::<RowId, _>(self.as_mut())
            .map_err(repo_error)?;
        rows.map(|row| row.map_err(repo_error).map(MediaSourceId::new))
            .collect::<RepoResult<_>>()
    }

    fn relocate_media_sources_by_content_path_prefix(
        &mut self,
        collection_id: CollectionId,
        updated_at: &OffsetDateTimeMs,
        old_content_path_prefix: &ContentPath<'_>,
        new_content_path_prefix: &ContentPath<'_>,
    ) -> RepoResult<usize> {
        let target = media_source::table
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(sql_column_substr_prefix_eq(
                "content_link_path",
                old_content_path_prefix.as_ref(),
            ));
        diesel::update(target)
            .set((
                media_source::row_updated_ms.eq(updated_at.timestamp_millis()),
                media_source::content_link_path.eq(diesel::dsl::sql(&format!(
                    "'{escaped}' || substr(content_link_path,{len})",
                    escaped = escape_single_quotes(new_content_path_prefix.as_ref()),
                    len = old_content_path_prefix.len() + 1
                ))),
            ))
            .execute(self.as_mut())
            .map_err(repo_error)
    }

    fn purge_media_sources_by_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<usize> {
        // Reuse the tested subselect with reliable predicate filtering
        // even if it might be slightly less efficient! The query optimizer
        // should detect this.
        diesel::delete(media_source::table.filter(media_source::row_id.eq_any(
            select_row_id_filtered_by_content_path_predicate(collection_id, content_path_predicate),
        )))
        .execute(self.as_mut())
        .map_err(repo_error)
    }

    fn purge_orphaned_media_sources_by_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<usize> {
        // Reuse the tested subselect with reliable predicate filtering
        // even if it might be slightly less efficient! The query optimizer
        // should detect this.
        diesel::delete(
            media_source::table
                .filter(media_source::row_id.eq_any(
                    select_row_id_filtered_by_content_path_predicate(
                        collection_id,
                        content_path_predicate,
                    ),
                ))
                // Restrict to orphaned media sources without a track
                .filter(media_source::row_id.ne_all(track::table.select(track::media_source_id))),
        )
        .execute(self.as_mut())
        .map_err(repo_error)
    }

    fn purge_untracked_media_sources_by_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<usize> {
        // Reuse the tested subselect with reliable predicate filtering
        // even if it might be slightly less efficient! The query optimizer
        // should detect this.
        diesel::delete(
            media_source::table
                .filter(media_source::row_id.eq_any(
                    select_row_id_filtered_by_content_path_predicate(
                        collection_id,
                        content_path_predicate,
                    ),
                ))
                // Restrict to untracked media sources
                .filter(
                    media_source::row_id.ne_all(
                        media_tracker_source::table.select(media_tracker_source::source_id),
                    ),
                ),
        )
        .execute(self.as_mut())
        .map_err(repo_error)
    }

    fn insert_media_source(
        &mut self,
        collection_id: CollectionId,
        created_at: OffsetDateTimeMs,
        created_source: &Source,
    ) -> RepoResult<RecordHeader> {
        let insertable = InsertableRecord::bind(&created_at, collection_id, created_source);
        let query = insertable.insert_into(media_source::table);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert_eq!(1, rows_affected);
        let (id, _) = self.resolve_media_source_id_synchronized_at_by_content_path(
            collection_id,
            &created_source.content.link.path,
        )?;
        let updated_at = created_at.clone();
        Ok(RecordHeader {
            id,
            created_at,
            updated_at,
        })
    }

    fn load_media_source_by_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(RecordHeader, Source)> {
        media_source::table
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(media_source::content_link_path.eq(content_path.as_str()))
            .get_result::<QueryableRecord>(self.as_mut())
            .map_err(repo_error)
            .and_then(|record| record.try_into().map_err(RepoError::Other))
    }

    fn purge_orphaned_media_sources(&mut self, collection_id: CollectionId) -> RepoResult<usize> {
        let target = media_source::table
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(media_source::row_id.ne_all(track::table.select(track::media_source_id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        Ok(rows_affected)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
