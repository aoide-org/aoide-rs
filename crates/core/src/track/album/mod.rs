// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use nonicle::{Canonical, IsCanonical};
use semval::prelude::*;
use strum::FromRepr;

use super::{
    actor::{self, Actor, Actors, ActorsInvalidity},
    title::{Title, Titles, TitlesInvalidity},
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, FromRepr)]
#[repr(u8)]
pub enum Kind {
    #[default]
    NoCompilation = 0, // either Album or Single
    Compilation = 1,
    Album = 2,
    Single = 3,
}

#[derive(Clone, Debug, Default, Eq)]
pub struct Album {
    pub kind: Option<Kind>,

    pub titles: Canonical<Vec<Title>>,

    pub actors: Canonical<Vec<Actor>>,
}

impl Album {
    #[must_use]
    pub fn main_title<'a, 'b>(&'a self) -> Option<&'a Title>
    where
        'b: 'a,
    {
        Titles::main_title(self.titles.iter())
    }

    #[must_use]
    pub fn summary_actor(&self, role: actor::Role) -> Option<&Actor> {
        Actors::summary_actor(self.actors.iter(), role)
    }

    #[must_use]
    pub fn summary_artist(&self) -> Option<&Actor> {
        Actors::summary_actor(self.actors.iter(), actor::Role::Artist)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AlbumInvalidity {
    Titles(TitlesInvalidity),
    Actors(ActorsInvalidity),
}

impl Validate for Album {
    type Invalidity = AlbumInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .merge_result_with(
                Titles::validate(&self.titles.iter()),
                Self::Invalidity::Titles,
            )
            .merge_result_with(
                Actors::validate(&self.actors.iter()),
                Self::Invalidity::Actors,
            )
            .into()
    }
}

impl IsCanonical for Album {
    fn is_canonical(&self) -> bool {
        true
    }
}

impl PartialEq for Album {
    fn eq(&self, other: &Album) -> bool {
        debug_assert!(self.is_canonical());
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
