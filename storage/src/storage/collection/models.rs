// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::{schema::tbl_collection, *};

use crate::api::entity::StorageId;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable)]
#[table_name = "tbl_collection"]
pub struct InsertableCollectionsEntity<'a> {
    pub uid: &'a [u8],
    pub rev_ordinal: i64,
    pub rev_instant: TickType,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> InsertableCollectionsEntity<'a> {
    pub fn bind(entity: &'a CollectionEntity) -> Self {
        Self {
            uid: entity.header().uid().as_ref(),
            rev_ordinal: entity.header().revision().ordinal() as i64,
            rev_instant: (entity.header().revision().instant().0).0,
            name: &entity.body().name,
            description: entity.body().description.as_ref().map(|s| s.as_str()),
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "tbl_collection"]
pub struct UpdatableCollectionsEntity<'a> {
    pub rev_ordinal: i64,
    pub rev_instant: TickType,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> UpdatableCollectionsEntity<'a> {
    pub fn bind(next_revision: &EntityRevision, body: &'a Collection) -> Self {
        Self {
            rev_ordinal: next_revision.ordinal() as i64,
            rev_instant: (next_revision.instant().0).0,
            name: &body.name,
            description: body.description.as_ref().map(|s| s.as_str()),
        }
    }
}

#[derive(Debug, Queryable)]
pub struct QueryableCollectionsEntity {
    pub id: StorageId,
    pub uid: Vec<u8>,
    pub rev_ordinal: i64,
    pub rev_instant: TickType,
    pub name: String,
    pub description: Option<String>,
}

impl From<QueryableCollectionsEntity> for CollectionEntity {
    fn from(from: QueryableCollectionsEntity) -> Self {
        let uid = EntityUid::from_slice(&from.uid);
        let revision = EntityRevision::new(
            from.rev_ordinal as u64,
            TickInstant(Ticks(from.rev_instant)),
        );
        let header = EntityHeader::new(uid, revision);
        let body = Collection {
            name: from.name,
            description: from.description,
        };
        Self::new(header, body)
    }
}
