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

use super::{schema::*, *};

use num_traits::FromPrimitive as _;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "track_actor"]
pub struct QueryableRecord {
    pub id: RowId,
    pub track_id: RowId,
    pub scope: i16,
    pub kind: i16,
    pub name: String,
    pub role: i16,
    pub role_notes: Option<String>,
}

impl From<QueryableRecord> for (RecordId, Record) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            id,
            track_id,
            scope,
            kind,
            name,
            role,
            role_notes,
        } = from;
        let actor = Actor {
            kind: ActorKind::from_i16(kind).unwrap_or_else(|| {
                log::error!("Invalid actor kind value: {}", kind);
                Default::default()
            }),
            name,
            role: ActorRole::from_i16(role).unwrap_or_else(|| {
                log::error!("Invalid actor role value: {}", role);
                Default::default()
            }),
            role_notes,
        };
        let record = Record {
            track_id: track_id.into(),
            scope: Scope::from_i16(scope).unwrap_or_else(|| {
                log::error!("Invalid scope value: {}", scope);
                Scope::Track
            }),
            actor,
        };
        (id.into(), record)
    }
}

#[derive(Debug, Insertable)]
#[table_name = "track_actor"]
pub struct InsertableRecord<'a> {
    pub track_id: RowId,
    pub scope: i16,
    pub kind: i16,
    pub name: &'a str,
    pub role: i16,
    pub role_notes: Option<&'a str>,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(track_id: RecordId, scope: Scope, actor: &'a Actor) -> Self {
        let Actor {
            kind,
            name,
            role,
            role_notes,
        } = actor;
        Self {
            track_id: track_id.into(),
            scope: scope as i16,
            kind: *kind as i16,
            name: name.as_str(),
            role: *role as i16,
            role_notes: role_notes.as_ref().map(String::as_str),
        }
    }
}
