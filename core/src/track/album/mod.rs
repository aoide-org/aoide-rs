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

use super::release::ReleaseYear;

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
    pub fn main_title<'a, 'b>(
        &'a self,
        default_language: impl Into<Option<&'b str>>,
    ) -> Option<&'a Title>
    where
        'b: 'a,
    {
        Titles::main_title(&self.titles, default_language)
    }

    pub fn main_actor(&self, role: ActorRole) -> Option<&Actor> {
        Actors::main_actor(&self.actors, role)
    }

    pub fn main_artist(&self) -> Option<&Actor> {
        Actors::main_actor(&self.actors, ActorRole::Artist)
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
        context.map_and_merge_result(Titles::validate(&self.titles), AlbumValidation::Titles);
        context.map_and_merge_result(Actors::validate(&self.actors), AlbumValidation::Actors);
        context.into_result()
    }
}

// TODO: Move into separate module with response types?
// Might not be needed in the core.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AlbumTracksCount {
    pub title: Option<String>,

    pub artist: Option<String>,

    pub release_year: Option<ReleaseYear>,

    pub count: usize,
}

impl AlbumTracksCount {
    pub fn new_for_album(
        album: &Album,
        release_year: impl Into<Option<ReleaseYear>>,
        count: usize,
    ) -> Self {
        let title = album.main_title(None).map(|title| title.name.to_string());
        let artist = album.main_artist().map(|actor| actor.name.to_string());
        let release_year = release_year.into();
        Self {
            title,
            artist,
            release_year,
            count,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
