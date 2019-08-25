// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::{actor::*, title::*};

///////////////////////////////////////////////////////////////////////
// Album
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Album {
    pub titles: Vec<Title>,

    pub actors: Vec<Actor>,

    pub compilation: Option<bool>,
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
pub enum AlbumValidation {
    Titles(TitlesValidation),
    Actors(ActorsValidation),
}

impl Validate for Album {
    type Validation = AlbumValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.map_and_merge_result(
            Titles::validate(self.titles.iter()),
            AlbumValidation::Titles,
        );
        context.map_and_merge_result(
            Actors::validate(self.actors.iter()),
            AlbumValidation::Actors,
        );
        context.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
