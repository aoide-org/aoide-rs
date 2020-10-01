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

pub mod playlist;
pub mod track;

use super::*;

use crate::util::clock::TickInstant;

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
pub enum CollectionItem {
    Track(track::Item),
    Playlist(playlist::Item),
}

#[derive(Copy, Clone, Debug)]
pub enum CollectionItemInvalidity {
    Track(track::ItemInvalidity),
    Playlist(playlist::ItemInvalidity),
}

impl Validate for CollectionItem {
    type Invalidity = CollectionItemInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        use CollectionItem::*;
        match self {
            Track(ref track) => context.validate_with(track, Self::Invalidity::Track),
            Playlist(ref playlist) => context.validate_with(playlist, Self::Invalidity::Playlist),
        }
        .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionEntry {
    pub added_at: TickInstant,

    pub item: CollectionItem,
}

#[derive(Copy, Clone, Debug)]
pub enum CollectionEntryInvalidity {
    Item(CollectionItemInvalidity),
}

impl Validate for CollectionEntry {
    type Invalidity = CollectionEntryInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.item, Self::Invalidity::Item)
            .into()
    }
}
