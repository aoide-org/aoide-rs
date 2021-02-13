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

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

use crate::{collection::RecordId as CollectionId, prelude::*};

use aoide_core::{media::Source, util::clock::DateTime};

pub trait Repo {
    fn resolve_media_source_id_synchronized_at_by_uri(
        &self,
        collection_id: CollectionId,
        uri: &str,
    ) -> RepoResult<(RecordId, Option<DateTime>)>;
    fn resolve_media_source_ids_by_uri_predicate(
        &self,
        collection_id: CollectionId,
        uri_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<Vec<RecordId>>;

    fn insert_media_source(
        &self,
        created_at: DateTime,
        collection_id: CollectionId,
        created_source: &Source,
    ) -> RepoResult<RecordHeader>;
    fn update_media_source(
        &self,
        id: RecordId,
        updated_at: DateTime,
        updated_source: &Source,
    ) -> RepoResult<()>;
    fn delete_media_source(&self, id: RecordId) -> RepoResult<()>;

    fn load_media_source(&self, id: RecordId) -> RepoResult<(RecordHeader, Source)>;
    fn load_media_source_by_uri(
        &self,
        collection_id: CollectionId,
        uri: &str,
    ) -> RepoResult<(RecordHeader, Source)>;

    fn relocate_media_sources_by_uri_prefix(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        old_uri_prefix: &str,
        new_uri_prefix: &str,
    ) -> RepoResult<usize>;

    fn purge_media_sources_by_uri_predicate(
        &self,
        collection_id: CollectionId,
        uri_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<usize>;

    fn purge_orphaned_media_sources_from_collection(
        &self,
        collection_id: CollectionId,
    ) -> RepoResult<usize>;

    fn purge_orphaned_media_sources_from_all_collections(&self) -> RepoResult<usize>;
}
