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

///////////////////////////////////////////////////////////////////////
// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ActorRole {
    Artist,
    Arranger,
    Composer,
    Conductor,
    DjMixer,
    Engineer,
    Lyricist,
    Mixer,
    Performer,
    Producer,
    Publisher,
    Remixer,
    Writer,
}

impl Default for ActorRole {
    fn default() -> ActorRole {
        ActorRole::Artist
    }
}

///////////////////////////////////////////////////////////////////////
// ActorPrecedence
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActorPrecedence {
    Summary,
    Primary,
    Secondary,
}

impl Default for ActorPrecedence {
    fn default() -> ActorPrecedence {
        ActorPrecedence::Summary
    }
}

///////////////////////////////////////////////////////////////////////
// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Actor {
    pub name: String,

    pub role: ActorRole,

    pub precedence: ActorPrecedence,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActorValidation {
    NameEmpty,
}

impl Validate for Actor {
    type Validation = ActorValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(self.name.trim().is_empty(), ActorValidation::NameEmpty);
        context.into_result()
    }
}

#[derive(Debug)]
pub struct Actors;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActorsValidation {
    Actor(ActorValidation),
    SummaryActorMissing,
    SummaryActorAmbiguous,
    MainActorMissing,
}

pub const ANY_ROLE_FILTER: Option<ActorRole> = None;
pub const ANY_PRECEDENCE_FILTER: Option<ActorPrecedence> = None;

impl Actors {
    pub fn validate<'a, I>(actors: I) -> ValidationResult<ActorsValidation>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        let mut context = ValidationContext::default();
        let mut at_least_one_actor = false;
        for actor in actors.clone() {
            context.map_and_merge_result(actor.validate(), ActorsValidation::Actor);
            at_least_one_actor = true;
        }
        if !context.has_violations() {
            let mut roles: Vec<_> = actors.clone().map(|actor| actor.role).collect();
            roles.sort_unstable();
            roles.dedup();
            let mut summary_missing = false;
            let mut summary_ambiguous = false;
            for role in roles {
                // A summary entry exists if more than one primary entry exists for disambiguation
                if Self::filter_role_precedence(actors.clone(), role, ActorPrecedence::Primary)
                    .count()
                    > 1
                    && Self::filter_role_precedence(actors.clone(), role, ActorPrecedence::Summary)
                        .count()
                        == 0
                {
                    summary_missing = true;
                }
                // At most one summary entry exists for each role
                if Self::filter_role_precedence(actors.clone(), role, ActorPrecedence::Summary)
                    .count()
                    > 1
                {
                    summary_ambiguous = true;
                }
            }
            context.add_violation_if(summary_missing, ActorsValidation::SummaryActorMissing);
            context.add_violation_if(summary_ambiguous, ActorsValidation::SummaryActorAmbiguous);
        }
        context.add_violation_if(
            !context.has_violations()
                && at_least_one_actor
                && Self::main_actor(actors, ActorRole::Artist).is_none(),
            ActorsValidation::MainActorMissing,
        );
        context.into_result()
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
        I: Iterator<Item = &'a Actor> + Clone,
    {
        // Try `Summary` first
        if let Some(actor) =
            Self::filter_role_precedence(actors.clone(), role, ActorPrecedence::Summary).nth(0)
        {
            return Some(actor);
        }
        // Otherwise try `Primary` as a fallback
        Self::filter_role_precedence(actors, role, ActorPrecedence::Primary).nth(0)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
