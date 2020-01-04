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
    pub use aoide_core::{collection::*, entity::EntityHeader};
}

///////////////////////////////////////////////////////////////////////
// Collection
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Collection {
    #[serde(rename = "n")]
    name: String,

    #[serde(rename = "d")]
    description: Option<String>,
}

impl From<Collection> for _core::Collection {
    fn from(from: Collection) -> Self {
        let Collection { name, description } = from;
        Self { name, description }
    }
}

impl From<_core::Collection> for Collection {
    fn from(from: _core::Collection) -> Self {
        let _core::Collection { name, description } = from;
        Self { name, description }
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

pub type Entity = crate::entity::Entity<Collection>;

impl From<Entity> for _core::Entity {
    fn from(from: Entity) -> Self {
        Self::new(from.0, from.1)
    }
}

impl From<_core::Entity> for Entity {
    fn from(from: _core::Entity) -> Self {
        Self(from.hdr.into(), from.body.into())
    }
}
