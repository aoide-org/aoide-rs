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

use audio::Duration;
use domain::entity::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionBody {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CollectionBody {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionStats {
    pub tracks: Option<CollectionTrackStats>,
}

impl CollectionStats {
    pub fn is_empty(&self) -> bool {
        self.tracks.is_none()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionTrackStats {
    pub total_count: usize,
    pub total_duration: Duration,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionEntity {
    header: EntityHeader,

    body: CollectionBody,

    #[serde(skip_serializing_if = "CollectionStats::is_empty", default)]
    pub stats: CollectionStats,
}

impl CollectionEntity {
    pub fn new(header: EntityHeader, body: CollectionBody) -> Self {
        Self {
            header,
            body,
            stats: CollectionStats::default(),
        }
    }

    pub fn with_body(body: CollectionBody) -> Self {
        Self::new(EntityHeader::initial(), body)
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
}

pub type CollectionUid = EntityUid;
