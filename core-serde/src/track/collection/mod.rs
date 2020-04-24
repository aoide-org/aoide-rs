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
#[serde(deny_unknown_fields)]
pub struct Collection {
    #[serde(rename = "uid")]
    uid: EntityUid,

    #[serde(rename = "add")]
    added_at: TickType,

    #[serde(rename = "col", skip_serializing_if = "Option::is_none")]
    color: Option<ColorRgb>,

    #[serde(rename = "plc", skip_serializing_if = "Option::is_none")]
    play_count: Option<usize>,

    #[serde(rename = "plt", skip_serializing_if = "Option::is_none")]
    last_played_at: Option<TickType>,
}

impl From<_core::Collection> for Collection {
    fn from(from: _core::Collection) -> Self {
        let _core::Collection {
            uid,
            added_at,
            color,
            play_count,
            last_played_at,
        } = from;
        Self {
            uid: uid.into(),
            added_at: (added_at.0).0,
            color: color.map(Into::into),
            play_count,
            last_played_at: last_played_at.map(|last_played_at| (last_played_at.0).0),
        }
    }
}

impl From<Collection> for _core::Collection {
    fn from(from: Collection) -> Self {
        let Collection {
            uid,
            added_at,
            color,
            play_count,
            last_played_at,
        } = from;
        Self {
            uid: uid.into(),
            added_at: TickInstant(Ticks(added_at)),
            color: color.map(Into::into),
            play_count,
            last_played_at: last_played_at.map(|last_played_at| TickInstant(Ticks(last_played_at))),
        }
    }
}
