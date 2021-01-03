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

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "track_tag"]
pub struct QueryableRecord {
    pub id: RowId,
    pub track_id: RowId,
    pub facet: Option<String>,
    pub label: Option<String>,
    pub score: f64,
}

impl From<QueryableRecord> for (RecordId, Record) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            id,
            track_id,
            facet,
            label,
            score,
        } = from;
        let record = Record {
            track_id: track_id.into(),
            facet: facet.map(Into::into),
            label: label.map(Into::into),
            score: score.into(),
        };
        (id.into(), record)
    }
}

#[derive(Debug, Insertable)]
#[table_name = "track_tag"]
pub struct InsertableRecord<'a> {
    pub track_id: RowId,
    pub facet: Option<&'a str>,
    pub label: Option<&'a str>,
    pub score: f64,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(track_id: RecordId, facet: &'a Option<Facet>, plain_tag: &'a PlainTag) -> Self {
        let PlainTag { label, score } = plain_tag;
        Self {
            track_id: track_id.into(),
            facet: facet.as_ref().map(Facet::as_ref),
            label: label.as_ref().map(Label::as_ref),
            score: score.into_inner(),
        }
    }
}
