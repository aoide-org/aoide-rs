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

use aoide_core::{collection::*, util::clock::DateTime};

use aoide_core_ext::collection::*;

use crate::prelude::*;

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

pub trait EntityRepo {
    entity_repo_trait_common_functions!(RecordId, Entity, Collection);

    fn insert_collection_entity(
        &self,
        created_at: DateTime,
        created_entity: &Entity,
    ) -> RepoResult<RecordId>;

    fn load_collection_entities(
        &self,
        kind: Option<&str>,
        with_summary: bool,
        pagination: Option<&Pagination>,
        collector: &mut dyn ReservableRecordCollector<
            Header = RecordHeader,
            Record = (Entity, Option<Summary>),
        >,
    ) -> RepoResult<()>;

    fn load_collection_summary(&self, id: RecordId) -> RepoResult<Summary>;
}
