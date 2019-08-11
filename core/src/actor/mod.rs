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

use crate::validate::{self, Validate};

///////////////////////////////////////////////////////////////////////
// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActorRole {
    Artist = 0, // default
    Arranger = 1,
    Composer = 2,
    Conductor = 3,
    DjMixer = 4,
    Engineer = 5,
    Lyricist = 6,
    Mixer = 7,
    Performer = 8,
    Producer = 9,
    Publisher = 10,
    Remixer = 11,
    Writer = 12,
}

impl Default for ActorRole {
    fn default() -> ActorRole {
        ActorRole::Artist
    }
}

///////////////////////////////////////////////////////////////////////
// ActorPrecedence
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActorPrecedence {
    Summary = 0, // default
    Primary = 1,
    Secondary = 2,
}

impl Default for ActorPrecedence {
    fn default() -> ActorPrecedence {
        ActorPrecedence::Summary
    }
}

///////////////////////////////////////////////////////////////////////
// Actor
///////////////////////////////////////////////////////////////////////

const MIN_NAME_LEN: usize = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Actor {
    pub name: String,

    pub role: ActorRole,

    pub precedence: ActorPrecedence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorValidation {
    Name,
}

impl Validate<ActorValidation> for Actor {
    fn validate(&self) -> ValidationResult<ActorValidation> {
        let mut errors = ValidationErrors::default();
        if self.name.len() < MIN_NAME_LEN {
            errors.add_error(
                ActorValidation::Name,
                Violation::TooShort(validate::Min(MIN_NAME_LEN)),
            );
        }
        errors.into_result()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Actors;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorsValidation {
    Actor(ActorValidation),
    SummaryActor,
    MainActor,
}

pub const ANY_ROLE_FILTER: Option<ActorRole> = None;
pub const ANY_PRECEDENCE_FILTER: Option<ActorPrecedence> = None;

impl Actors {
    pub fn validate<'a, I>(actors: I) -> ValidationResult<ActorsValidation>
    where
        I: IntoIterator<Item = &'a Actor> + Copy,
    {
        let mut errors = ValidationErrors::default();
        let mut at_least_one_actor = false;
        for actor in actors.into_iter() {
            errors.map_and_merge_result(actor.validate(), ActorsValidation::Actor);
            at_least_one_actor = true;
        }
        if errors.is_empty() {
            let mut roles: Vec<_> = actors.into_iter().map(|actor| actor.role).collect();
            roles.sort();
            roles.dedup();
            let mut summary_missing = false;
            let mut summary_too_many = false;
            for role in roles {
                // A summary entry exists if more than one primary entry exists for disambiguation
                if Self::filter_role_precedence(actors, role, ActorPrecedence::Primary).count() > 1
                    && Self::filter_role_precedence(actors, role, ActorPrecedence::Summary).count()
                        == 0
                {
                    summary_missing = true;
                }
                // At most one summary entry exists for each role
                if Self::filter_role_precedence(actors, role, ActorPrecedence::Summary).count() > 1
                {
                    summary_too_many = true;
                }
            }
            if summary_missing {
                errors.add_error(ActorsValidation::SummaryActor, Violation::Missing);
            }
            if summary_too_many {
                errors.add_error(
                    ActorsValidation::SummaryActor,
                    Violation::TooMany(validate::Max(1)),
                );
            }
        }
        if errors.is_empty()
            && at_least_one_actor
            && Self::main_actor(actors, ActorRole::Artist).is_none()
        {
            errors.add_error(ActorsValidation::MainActor, Violation::Missing);
        }
        errors.into_result()
    }

    pub fn filter_role_precedence<'a, I>(
        actors: I,
        role: impl Into<Option<ActorRole>>,
        precedence: impl Into<Option<ActorPrecedence>>,
    ) -> impl Iterator<Item = &'a Actor>
    where
        I: IntoIterator<Item = &'a Actor>,
    {
        let role = role.into();
        let precedence = precedence.into();
        actors.into_iter().filter(move |actor| {
            (role == ANY_ROLE_FILTER || role == Some(actor.role))
                && (precedence == ANY_PRECEDENCE_FILTER || precedence == Some(actor.precedence))
        })
    }

    // The singular summary actor or if none exists then the singular primary actor
    pub fn main_actor<'a, I>(actors: I, role: ActorRole) -> Option<&'a Actor>
    where
        I: IntoIterator<Item = &'a Actor> + Copy,
    {
        Self::filter_role_precedence(actors, role, ActorPrecedence::Summary)
            .nth(0)
            .or_else(|| Self::filter_role_precedence(actors, role, ActorPrecedence::Primary).nth(0))
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
