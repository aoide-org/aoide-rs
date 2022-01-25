// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use num_traits::FromPrimitive as _;

use super::{schema::*, *};

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "track_title"]
pub struct QueryableRecord {
    pub id: RowId,
    pub track_id: RowId,
    pub scope: i16,
    pub kind: i16,
    pub name: String,
}

impl TryFrom<QueryableRecord> for (RecordId, Record) {
    type Error = anyhow::Error;

    fn try_from(from: QueryableRecord) -> anyhow::Result<Self> {
        let QueryableRecord {
            id,
            track_id,
            scope,
            kind,
            name,
        } = from;
        let kind = TitleKind::from_i16(kind)
            .ok_or_else(|| anyhow::anyhow!("Invalid title kind value: {}", kind))?;
        let scope = Scope::from_i16(scope)
            .ok_or_else(|| anyhow::anyhow!("Invalid scope value: {}", scope))?;
        let record = Record {
            track_id: track_id.into(),
            scope,
            title: Title { kind, name },
        };
        Ok((id.into(), record))
    }
}

#[derive(Debug, Insertable)]
#[table_name = "track_title"]
pub struct InsertableRecord<'a> {
    pub track_id: RowId,
    pub scope: i16,
    pub kind: i16,
    pub name: &'a str,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(track_id: RecordId, scope: Scope, title: &'a Title) -> Self {
        let Title { kind, name } = title;
        Self {
            track_id: track_id.into(),
            scope: scope as i16,
            kind: *kind as i16,
            name: name.as_str(),
        }
    }
}
