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

use super::{actor::*, title::*};

use crate::prelude::*;

use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
pub enum AlbumKind {
    Album = 0,
    Single = 1,
    Compilation = 2,
}

#[derive(Clone, Debug, Default, Eq)]
pub struct Album {
    pub titles: Vec<Title>,

    pub actors: Vec<Actor>,

    pub kind: Option<AlbumKind>,
}

impl Album {
    pub fn main_title<'a, 'b>(&'a self) -> Option<&'a Title>
    where
        'b: 'a,
    {
        Titles::main_title(self.titles.iter())
    }

    pub fn main_actor(&self, role: ActorRole) -> Option<&Actor> {
        Actors::main_actor(self.actors.iter(), role)
    }

    pub fn main_artist(&self) -> Option<&Actor> {
        Actors::main_actor(self.actors.iter(), ActorRole::Artist)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AlbumInvalidity {
    Titles(TitlesInvalidity),
    Actors(ActorsInvalidity),
}

impl Validate for Album {
    type Invalidity = AlbumInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .merge_result_with(
                Titles::validate(self.titles.iter()),
                AlbumInvalidity::Titles,
            )
            .merge_result_with(
                Actors::validate(self.actors.iter()),
                AlbumInvalidity::Actors,
            )
            .into()
    }
}

impl Canonicalize for Album {
    fn canonicalize(&mut self) {
        let Self { actors, titles, .. } = self;
        sort_slice_canonically(actors);
        sort_slice_canonically(titles);
    }

    fn is_canonicalized(&self) -> bool {
        let Self { actors, titles, .. } = self;
        is_slice_sorted_canonically(actors) && is_slice_sorted_canonically(titles)
    }
}

impl PartialEq for Album {
    fn eq(&self, other: &Album) -> bool {
        debug_assert!(self.is_canonicalized());
        let Self {
            kind,
            titles,
            actors,
        } = self;
        kind.eq(&other.kind) && titles.eq(&other.titles) && actors.eq(&other.actors)
    }
}

#[cfg(test)]
mod tests;
