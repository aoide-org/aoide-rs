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

use crate::prelude::*;

use num_derive::{FromPrimitive, ToPrimitive};
use std::{cmp::Ordering, iter::once};

///////////////////////////////////////////////////////////////////////
// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
pub enum ActorRole {
    Artist = 0,
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
// ActorKind
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
pub enum ActorKind {
    Summary = 0, // unspecified for display, may mention multiple actors with differing kinds and roles
    Primary = 1,
    Secondary = 2,
    Sorting = 3, // for sorting
}

impl Default for ActorKind {
    fn default() -> ActorKind {
        ActorKind::Summary
    }
}

///////////////////////////////////////////////////////////////////////
// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Actor {
    pub role: ActorRole,

    pub kind: ActorKind,

    pub name: String,

    /// A textual annotation for the role, e.g. the role of or
    /// the instrument played by the performer.
    pub role_notes: Option<String>,
}

impl CanonicalOrd for Actor {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        let Self {
            role: lhs_role,
            kind: lhs_kind,
            name: lhs_name,
            role_notes: _,
        } = self;
        let Self {
            role: rhs_role,
            kind: rhs_kind,
            name: rhs_name,
            role_notes: _,
        } = other;
        lhs_role
            .cmp(rhs_role)
            .then(lhs_kind.cmp(rhs_kind))
            .then(lhs_name.cmp(rhs_name))
    }
}

impl IsCanonical for Actor {
    fn is_canonical(&self) -> bool {
        true
    }
}

impl Canonicalize for Actor {
    fn canonicalize(&mut self) {
        debug_assert!(self.is_canonical());
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ActorInvalidity {
    NameEmpty,
}

impl Validate for Actor {
    type Invalidity = ActorInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.name.trim().is_empty(), ActorInvalidity::NameEmpty)
            .into()
    }
}

#[derive(Debug)]
pub struct Actors;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ActorsInvalidity {
    Actor(ActorInvalidity),
    SummaryActorMissing,
    SummaryActorAmbiguous,
    SortingActorAmbiguous,
    MainActorMissing,
}

pub const ANY_ROLE_FILTER: Option<ActorRole> = None;
pub const ANY_RANK_FILTER: Option<ActorKind> = None;

impl Actors {
    pub fn validate<'a, I>(actors: I) -> ValidationResult<ActorsInvalidity>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        let mut at_least_one_actor = false;
        let mut context = actors
            .clone()
            .fold(ValidationContext::new(), |context, actor| {
                at_least_one_actor = true;
                context.validate_with(actor, ActorsInvalidity::Actor)
            });
        if context.is_valid() {
            let mut roles: Vec<_> = actors.clone().map(|actor| actor.role).collect();
            roles.sort_unstable();
            roles.dedup();
            let mut summary_missing = false;
            let mut summary_ambiguous = false;
            let mut sorting_ambiguous = false;
            for role in roles {
                // A summary entry exists if more than one primary entry exists for disambiguation
                if Self::filter_kind_role(actors.clone(), ActorKind::Primary, role).count() > 1
                    && Self::filter_kind_role(actors.clone(), ActorKind::Summary, role).count() == 0
                {
                    summary_missing = true;
                }
                // At most one summary entry exists for each role
                if Self::filter_kind_role(actors.clone(), ActorKind::Summary, role).count() > 1 {
                    summary_ambiguous = true;
                }
                // At most one sorting entry exists for each role
                if Self::filter_kind_role(actors.clone(), ActorKind::Sorting, role).count() > 1 {
                    sorting_ambiguous = true;
                }
            }
            context = context
                .invalidate_if(summary_missing, ActorsInvalidity::SummaryActorMissing)
                .invalidate_if(summary_ambiguous, ActorsInvalidity::SummaryActorAmbiguous)
                .invalidate_if(sorting_ambiguous, ActorsInvalidity::SortingActorAmbiguous);
        }
        if context.is_valid() {
            context = context.invalidate_if(
                at_least_one_actor && Self::main_actor(actors, ActorRole::Artist).is_none(),
                ActorsInvalidity::MainActorMissing,
            );
        }
        context.into()
    }

    pub fn filter_kind_role<'a, I>(
        actors: I,
        kind: impl Into<Option<ActorKind>>,
        role: impl Into<Option<ActorRole>>,
    ) -> impl Iterator<Item = &'a Actor>
    where
        I: IntoIterator<Item = &'a Actor>,
    {
        let kind = kind.into();
        let role = role.into();
        actors.into_iter().filter(move |actor| {
            (kind == ANY_RANK_FILTER || kind == Some(actor.kind))
                && (role == ANY_ROLE_FILTER || role == Some(actor.role))
        })
    }

    // The singular summary actor or if none exists then the singular primary actor
    pub fn main_actor<'a, I>(actors: I, role: ActorRole) -> Option<&'a Actor>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        // Try `Summary` first
        if let Some(actor) = Self::filter_kind_role(actors.clone(), ActorKind::Summary, role).next()
        {
            return Some(actor);
        }
        // Otherwise try `Primary` as a fallback
        Self::filter_kind_role(actors, ActorKind::Primary, role).next()
    }

    // The singular summary actor or if none exists then the singular primary actor
    pub fn other_actors<'a, I>(actors: I, role: ActorRole) -> Option<&'a Actor>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        // Try `Summary` first
        if let Some(actor) = Self::filter_kind_role(actors.clone(), ActorKind::Summary, role).next()
        {
            return Some(actor);
        }
        // Otherwise try `Primary` as a fallback
        Self::filter_kind_role(actors, ActorKind::Primary, role).next()
    }

    pub fn set_main_actor(
        actors: &mut Vec<Actor>,
        role: ActorRole,
        name: impl Into<String>,
    ) -> bool {
        let name = name.into();
        if let Some(main_actor) = Self::main_actor(actors.iter(), role) {
            // Replace
            if main_actor.name == name {
                return false; // unmodified
            }
            let kind = main_actor.kind;
            let role = main_actor.role;
            let role_notes = main_actor.role_notes.clone();
            let old_actors = std::mem::take(actors);
            let new_actors = once(Actor {
                role,
                kind,
                name,
                role_notes,
            })
            .chain(
                old_actors
                    .into_iter()
                    .filter(|actor| actor.kind != kind && actor.role != role),
            )
            .collect();
            let _placeholder = std::mem::replace(actors, new_actors);
            debug_assert!(_placeholder.is_empty());
        } else {
            // Add
            actors.push(Actor {
                name,
                kind: ActorKind::Summary,
                role,
                role_notes: None,
            });
        }
        true // modified
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
