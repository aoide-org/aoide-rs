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

use aoide_core::entity::*;

pub fn entity_revision_from_sql(rev: i64) -> EntityRevision {
    EntityRevision::from_inner(rev as EntityRevisionNumber)
}

pub fn entity_revision_to_sql(rev: EntityRevision) -> i64 {
    rev.to_inner() as i64
}

pub fn entity_header_from_sql(uid: &[u8], rev: i64) -> EntityHeader {
    EntityHeader {
        uid: EntityUid::from_slice(uid),
        rev: entity_revision_from_sql(rev),
    }
}
