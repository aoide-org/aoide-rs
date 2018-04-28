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

use domain::entity::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionBody {
    pub name: String,
}

impl CollectionBody {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionEntity {
    header: EntityHeader,

    body: CollectionBody,
}

impl CollectionEntity {
    pub fn new(header: EntityHeader, body: CollectionBody) -> Self {
        Self { header, body }
    }

    pub fn with_body(body: CollectionBody) -> Self {
        let uid = EntityUidGenerator::generate_uid();
        let header = EntityHeader::with_uid(uid);
        Self { header, body }
    }

    pub fn is_valid(&self) -> bool {
        self.header.is_valid() && self.body.is_valid()
    }

    pub fn header<'a>(&'a self) -> &'a EntityHeader {
        &self.header
    }

    pub fn body<'a>(&'a self) -> &'a CollectionBody {
        &self.body
    }

    pub fn body_mut<'a>(&'a mut self) -> &'a mut CollectionBody {
        &mut self.body
    }

    pub fn update_revision(&mut self, next_revision: EntityRevision) {
        self.header.update_revision(next_revision);
    }
}

pub type CollectionUid = EntityUid;

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
