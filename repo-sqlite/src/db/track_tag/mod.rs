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

pub mod models;
pub mod schema;

use crate::prelude::*;

use aoide_core::tag::*;

use aoide_repo::track::RecordId;

#[derive(Debug)]
pub struct Record {
    pub track_id: RecordId,
    pub facet: Option<Facet>,
    pub label: Option<Label>,
    pub score: Score,
}

impl From<Record> for (FacetKey, PlainTag) {
    fn from(from: Record) -> Self {
        let Record {
            track_id: _,
            facet,
            label,
            score,
        } = from;
        let facet_key = FacetKey::new(facet);
        let plain_tag = PlainTag { label, score };
        (facet_key, plain_tag)
    }
}
