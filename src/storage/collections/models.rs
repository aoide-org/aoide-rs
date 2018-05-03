// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use super::schema::collections_entity;

use chrono::{DateTime, Utc};
use chrono::naive::NaiveDateTime;

use diesel::prelude::*;
use diesel;

use aoide_core::domain::entity::{EntityUid, EntityHeader};
use aoide_core::domain::collection::*;

use storage::*;

use usecases::*;

#[derive(Debug, Insertable)]
#[table_name = "collections_entity"]
pub struct InsertableCollectionsEntity<'a> {
    pub uid: &'a str,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> InsertableCollectionsEntity<'a> {
    pub fn bind(entity: &'a CollectionEntity) -> Self {
        Self {
            uid: entity.header().uid().as_str(),
            rev_ordinal: entity.header().revision().ordinal() as i64,
            rev_timestamp: entity.header().revision().timestamp().naive_utc(),
            name: &entity.body().name,
            description: entity.body().description.as_ref().map(|s| s.as_str()),
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "collections_entity"]
pub struct UpdatableCollectionsEntity<'a> {
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> UpdatableCollectionsEntity<'a> {
    pub fn bind(next_revision: &EntityRevision, body: &'a CollectionBody) -> Self {
        Self {
            rev_ordinal: next_revision.ordinal() as i64,
            rev_timestamp: next_revision.timestamp().naive_utc(),
            name: &body.name,
            description: body.description.as_ref().map(|s| s.as_str()),
        }
    }
}

#[derive(Debug, Queryable)]
pub struct QueryableCollectionsEntity {
    pub id: StorageId,
    pub uid: String,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: String,
    pub description: Option<String>,
}

impl From<QueryableCollectionsEntity> for CollectionEntity {
    fn from(from: QueryableCollectionsEntity) -> Self {
        let uid: EntityUid = from.uid.into();
        let revision = EntityRevision::new(
            from.rev_ordinal as u64,
            DateTime::from_utc(from.rev_timestamp, Utc),
        );
        let header = EntityHeader::new(uid, revision);
        let body = CollectionBody {
            name: from.name,
            description: from.description,
        };
        Self::new(header, body)
    }
}
