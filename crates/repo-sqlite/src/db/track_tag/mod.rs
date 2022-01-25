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

pub mod models;
pub mod schema;

use crate::prelude::*;

use aoide_core::tag::*;

use aoide_repo::track::RecordId;

#[derive(Debug)]
pub struct Record {
    pub track_id: RecordId,
    pub facet_id: Option<FacetId>,
    pub label: Option<Label>,
    pub score: Score,
}

impl From<Record> for (Option<FacetId>, PlainTag) {
    fn from(from: Record) -> Self {
        let Record {
            track_id: _,
            facet_id,
            label,
            score,
        } = from;
        let plain_tag = PlainTag { label, score };
        (facet_id, plain_tag)
    }
}
