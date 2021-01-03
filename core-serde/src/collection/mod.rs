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

pub mod playlist;
pub mod track;

use super::*;

use aoide_core::util::clock::{TickInstant, TickType, Ticks};

mod _core {
    pub use aoide_core::{
        collection::{playlist, track, *},
        entity::EntityHeader,
    };
}

///////////////////////////////////////////////////////////////////////
// Collection
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Collection {
    #[serde(rename = "nam")]
    name: String,

    #[serde(rename = "dsc", skip_serializing_if = "Option::is_none")]
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
// CollectionEntry
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct CollectionEntry<T> {
    #[serde(rename = "add")]
    added_at: TickType,

    #[serde(flatten)]
    item: T,
}

impl<T, U> From<CollectionEntry<T>> for _core::CollectionEntry<U>
where
    T: Into<U>,
{
    fn from(from: CollectionEntry<T>) -> Self {
        let CollectionEntry { item, added_at } = from;
        Self {
            added_at: TickInstant(Ticks(added_at)),
            item: item.into(),
        }
    }
}

impl<T, U> From<_core::CollectionEntry<T>> for CollectionEntry<U>
where
    T: Into<U>,
{
    fn from(from: _core::CollectionEntry<T>) -> Self {
        let _core::CollectionEntry { item, added_at } = from;
        Self {
            added_at: (added_at.0).0,
            item: item.into(),
        }
    }
}

pub type SingleTrackEntry = CollectionEntry<track::ItemBody>;

pub type TrackEntry = CollectionEntry<track::Item>;

pub type PlaylistEntry = CollectionEntry<playlist::Item>;

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
