// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{MediaSource, media::content::ContentPath, util::clock::UtcDateTimeMs};
use aoide_core_api::filter::StringPredicate;

use crate::{CollectionId, RepoResult};

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

pub trait Repo {
    fn update_media_source(
        &mut self,
        id: RecordId,
        updated_at: UtcDateTimeMs,
        updated_media_source: &MediaSource,
    ) -> RepoResult<()>;

    fn purge_media_source(&mut self, id: RecordId) -> RepoResult<()>;

    fn load_media_source(&mut self, id: RecordId) -> RepoResult<(RecordHeader, MediaSource)>;
}

pub trait CollectionRepo {
    fn resolve_media_source_id_synchronized_at_by_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(RecordId, Option<u64>)>;

    fn resolve_media_source_ids_by_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<Vec<RecordId>>;

    fn insert_media_source(
        &mut self,
        collection_id: CollectionId,
        created_at: UtcDateTimeMs,
        created_media_source: &MediaSource,
    ) -> RepoResult<RecordHeader>;

    fn load_media_source_by_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(RecordHeader, MediaSource)>;

    fn relocate_media_sources_by_content_path_prefix(
        &mut self,
        collection_id: CollectionId,
        updated_at: UtcDateTimeMs,
        old_content_path_prefix: &ContentPath<'_>,
        new_content_path_prefix: &ContentPath<'_>,
    ) -> RepoResult<usize>;

    fn purge_media_sources_by_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<usize>;

    fn purge_orphaned_media_sources_by_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<usize>;

    fn purge_orphaned_media_sources(&mut self, collection_id: CollectionId) -> RepoResult<usize> {
        self.purge_orphaned_media_sources_by_content_path_predicate(
            collection_id,
            StringPredicate::Prefix("".into()),
        )
    }

    fn purge_untracked_media_sources_by_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<usize>;

    fn purge_untracked_media_sources(&mut self, collection_id: CollectionId) -> RepoResult<usize> {
        self.purge_untracked_media_sources_by_content_path_predicate(
            collection_id,
            StringPredicate::Prefix("".into()),
        )
    }
}
