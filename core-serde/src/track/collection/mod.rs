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

use super::*;

mod _core {
    pub use aoide_core::track::collection::*;
}

use aoide_core::util::clock::{TickInstant, TickType, Ticks};

use crate::{entity::EntityUid, util::color::ColorRgb};

///////////////////////////////////////////////////////////////////////
// Collection
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    #[serde(rename = "u")]
    uid: EntityUid,

    #[serde(rename = "s")]
    since: TickType,

    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    color: Option<ColorRgb>,

    #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
    play_count: Option<usize>,
}

impl From<_core::Collection> for Collection {
    fn from(from: _core::Collection) -> Self {
        let _core::Collection {
            uid,
            since,
            color,
            play_count,
        } = from;
        Self {
            uid: uid.into(),
            since: (since.0).0,
            color: color.map(Into::into),
            play_count,
        }
    }
}

impl From<Collection> for _core::Collection {
    fn from(from: Collection) -> Self {
        let Collection {
            uid,
            since,
            color,
            play_count,
        } = from;
        Self {
            uid: uid.into(),
            since: TickInstant(Ticks(since)),
            color: color.map(Into::into),
            play_count,
        }
    }
}
