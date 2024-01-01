// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    media::{content::ContentPath, Source},
    util::clock::OffsetDateTimeMs,
};
use aoide_repo::{collection::RecordId as CollectionId, media::source::*};

use crate::{
    db::{
        media_source::{models::*, schema::*, select_row_id_filtered_by_content_path_predicate},
        media_tracker::schema::*,
        track::schema::*,
    },
    prelude::*,
};

impl<'db> Repo for crate::prelude::Connection<'db> {
    fn update_media_source(
        &mut self,
        id: RecordId,
        updated_at: OffsetDateTimeMs,
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

    fn purge_media_source(&mut self, id: RecordId) -> RepoResult<()> {
        let target = media_source::table.filter(media_source::row_id.eq(RowId::from(id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn load_media_source(&mut self, id: RecordId) -> RepoResult<(RecordHeader, Source)> {
        media_source::table
            .filter(media_source::row_id.eq(RowId::from(id)))
            .first::<QueryableRecord>(self.as_mut())
            .map_err(repo_error)
            .and_then(|record| record.try_into().map_err(Into::into))
    }
}

impl<'db> CollectionRepo for crate::prelude::Connection<'db> {
    fn resolve_media_source_id_synchronized_at_by_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(RecordId, Option<u64>)> {
        debug_assert!(!content_path.as_str().ends_with('/'));
        media_source::table
            .select((media_source::row_id, media_source::content_link_rev))
            .filter(media_source::collection_id.eq(RowId::from(collection_id)))
            .filter(media_source::content_link_path.eq(content_path.as_str()))
            .first::<(RowId, Option<i64>)>(self.as_mut())
            .map(|(row_id, content_link_rev)| {
                (row_id.into(), content_link_rev.map(|rev| rev as u64))
            })
            .map_err(repo_error)
    }

    fn resolve_media_source_ids_by_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<Vec<RecordId>> {
        media_source::table
            .select(media_source::row_id)
            // Reuse the tested subselect with reliable predicate filtering
            // even if it might be slightly less efficient! The query optimizer
            // should detect this.
            .filter(
                media_source::row_id.eq_any(select_row_id_filtered_by_content_path_predicate(
                    collection_id,
                    content_path_predicate,
                )),
            )
            .load::<RowId>(self.as_mut())
            .map_err(repo_error)
            .map(|v| v.into_iter().map(RecordId::new).collect())
    }

    fn relocate_media_sources_by_content_path_prefix(
        &mut self,
        collection_id: CollectionId,
        updated_at: OffsetDateTimeMs,
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
                    "'{}' || substr(content_link_path,{})",
                    escape_single_quotes(new_content_path_prefix.as_ref()),
                    old_content_path_prefix.len() + 1
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
        let insertable = InsertableRecord::bind(created_at, collection_id, created_source);
        let query = diesel::insert_into(media_source::table).values(&insertable);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert_eq!(1, rows_affected);
        let (id, _) = self.resolve_media_source_id_synchronized_at_by_content_path(
            collection_id,
            &created_source.content.link.path,
        )?;
        Ok(RecordHeader {
            id,
            created_at,
            updated_at: created_at,
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
            .first::<QueryableRecord>(self.as_mut())
            .map_err(repo_error)
            .and_then(|record| record.try_into().map_err(Into::into))
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
