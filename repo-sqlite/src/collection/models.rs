// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::{collection::*, util::clock::*};

use aoide_repo::RepoId;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable)]
#[table_name = "tbl_collection"]
pub struct InsertableEntity<'a> {
    pub uid: &'a [u8],
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub name: &'a str,
    pub desc: Option<&'a str>,
}

impl<'a> InsertableEntity<'a> {
    pub fn bind(entity: &'a Entity) -> Self {
        Self {
            uid: entity.hdr.uid.as_ref(),
            rev_no: entity.hdr.rev.no as i64,
            rev_ts: (entity.hdr.rev.ts.0).0,
            name: &entity.body.name,
            desc: entity.body.description.as_deref(),
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "tbl_collection"]
pub struct UpdatableEntity<'a> {
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub name: &'a str,
    pub desc: Option<&'a str>,
}

impl<'a> UpdatableEntity<'a> {
    pub fn bind(next_revision: &EntityRevision, body: &'a Collection) -> Self {
        Self {
            rev_no: next_revision.no as i64,
            rev_ts: (next_revision.ts.0).0,
            name: &body.name,
            desc: body.description.as_deref(),
        }
    }
}

#[derive(Debug, Queryable)]
pub struct QueryableEntity {
    pub id: RepoId,
    pub uid: Vec<u8>,
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub name: String,
    pub desc: Option<String>,
}

impl From<QueryableEntity> for Entity {
    fn from(from: QueryableEntity) -> Self {
        let uid = EntityUid::from_slice(&from.uid);
        let rev = EntityRevision {
            no: from.rev_no as u64,
            ts: TickInstant(Ticks(from.rev_ts)),
        };
        let header = EntityHeader { uid, rev };
        let body = Collection {
            name: from.name,
            description: from.desc,
        };
        Self::new(header, body)
    }
}
