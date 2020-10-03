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

use crate::entity::EntityUid;

use aoide_core::util::clock::{TickInstant, TickType, Ticks};

mod _core {
    pub use aoide_core::collection::track::*;
}

///////////////////////////////////////////////////////////////////////
// ItemBody
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct ItemBody {
    #[serde(rename = "plc", skip_serializing_if = "Option::is_none")]
    play_count: Option<usize>,

    #[serde(rename = "plt", skip_serializing_if = "Option::is_none")]
    last_played_at: Option<TickType>,
}

impl From<_core::ItemBody> for ItemBody {
    fn from(from: _core::ItemBody) -> Self {
        let _core::ItemBody {
            play_count,
            last_played_at,
        } = from;
        Self {
            play_count: play_count.map(Into::into),
            last_played_at: last_played_at.map(|last_played_at| (last_played_at.0).0),
        }
    }
}

impl From<ItemBody> for _core::ItemBody {
    fn from(from: ItemBody) -> Self {
        let ItemBody {
            play_count,
            last_played_at,
        } = from;
        Self {
            play_count: play_count.map(Into::into),
            last_played_at: last_played_at.map(|last_played_at| TickInstant(Ticks(last_played_at))),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Item
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Item(EntityUid, ItemBody);

impl From<Item> for _core::Item {
    fn from(from: Item) -> Self {
        let Item(uid, body) = from;
        Self {
            uid: uid.into(),
            body: body.into(),
        }
    }
}

impl From<_core::Item> for Item {
    fn from(from: _core::Item) -> Self {
        let _core::Item { uid, body } = from;
        Self(uid.into(), body.into())
    }
}
