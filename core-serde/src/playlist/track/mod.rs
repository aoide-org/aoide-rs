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

mod _core {
    pub use aoide_core::playlist::track::*;
}

///////////////////////////////////////////////////////////////////////
// Item
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Item {
    #[serde(rename = "uid")]
    uid: EntityUid,
}

impl From<Item> for _core::Item {
    fn from(from: Item) -> Self {
        let Item { uid } = from;
        Self { uid: uid.into() }
    }
}

impl From<_core::Item> for Item {
    fn from(from: _core::Item) -> Self {
        let _core::Item { uid } = from;
        Self { uid: uid.into() }
    }
}
