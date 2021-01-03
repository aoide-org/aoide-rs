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

use crate::util::clock::TickInstant;

use std::fmt::Debug;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Collection {
    pub name: String,

    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CollectionInvalidity {
    NameEmpty,
}

impl Validate for Collection {
    type Invalidity = CollectionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.name.trim().is_empty(), CollectionInvalidity::NameEmpty)
            .into()
    }
}

pub type Entity = crate::entity::Entity<CollectionInvalidity, Collection>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionEntry<T> {
    pub added_at: TickInstant,

    pub item: T,
}

#[derive(Copy, Clone, Debug)]
pub enum CollectionEntryInvalidity<T>
where
    T: Validate + Debug + 'static,
{
    Item(T::Invalidity),
}

impl<T> Validate for CollectionEntry<T>
where
    T: Validate + Debug + 'static,
{
    type Invalidity = CollectionEntryInvalidity<T>;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.item, Self::Invalidity::Item)
            .into()
    }
}

pub type SingleTrackEntry = CollectionEntry<track::ItemBody>;

pub type TrackEntry = CollectionEntry<track::Item>;

pub type PlaylistEntry = CollectionEntry<playlist::Item>;
