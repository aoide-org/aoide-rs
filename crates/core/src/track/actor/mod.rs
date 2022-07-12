// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;

use num_derive::{FromPrimitive, ToPrimitive};

use crate::{
    prelude::*,
    util::canonical::{CanonicalOrd, Canonicalize, IsCanonical},
};

///////////////////////////////////////////////////////////////////////
// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
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
    Director = 10,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
pub enum ActorKind {
    Summary = 0, // unspecified for display, may mention multiple actors with differing kinds and roles
    Individual = 1, // single persons or group/band names
    Sorting = 2,
}

impl Default for ActorKind {
    fn default() -> ActorKind {
        ActorKind::Summary
    }
}

///////////////////////////////////////////////////////////////////////
// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

pub fn is_valid_actor_name(name: impl AsRef<str>) -> bool {
    let name = name.as_ref();
    let trimmed = name.trim();
    !trimmed.is_empty() && trimmed == name
}

pub fn is_valid_summary_individual_actor_name(
    summary_name: impl AsRef<str>,
    individual_name: impl AsRef<str>,
) -> bool {
    let summary_name = summary_name.as_ref();
    debug_assert!(is_valid_actor_name(summary_name));
    let individual_name = individual_name.as_ref();
    debug_assert!(is_valid_actor_name(individual_name));
    summary_name.contains(individual_name)
}

#[derive(Copy, Clone, Debug)]
pub enum ActorInvalidity {
    Name,
}

impl Validate for Actor {
    type Invalidity = ActorInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!is_valid_actor_name(&self.name), Self::Invalidity::Name)
            .into()
    }
}

#[derive(Debug)]
pub struct Actors;

#[derive(Copy, Clone, Debug)]
pub enum ActorsInvalidity {
    Actor(ActorInvalidity),
    SummaryActorAmbiguous(ActorRole),
    SortingActorAmbiguous(ActorRole),
    SummaryNameInconsistentWithIndividualNames(ActorRole),
    MainActorUndefined(ActorRole),
}

pub const ANY_ROLE_FILTER: Option<ActorRole> = None;
pub const ANY_RANK_FILTER: Option<ActorKind> = None;

impl Actors {
    pub fn validate<'a, I>(actors: &I) -> ValidationResult<ActorsInvalidity>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        // TODO (Optimization): Take a canonical slice of actors and iterate through
        // the different roles with .group_by(|lhs, rhs| lhs.role == rhs.role)
        // https://doc.rust-lang.org/std/primitive.slice.html#method.group_by
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
            for role in roles {
                let mut summary_actors_iter =
                    Self::filter_kind_role(actors.clone(), ActorKind::Summary, role);
                let summary_actor = summary_actors_iter.next();
                if let Some(summary_actor) = summary_actor {
                    debug_assert!(Self::main_actor(actors.clone(), role).is_some());
                    // At most one summary entry exists for each role.
                    context = context.invalidate_if(
                        summary_actors_iter.next().is_some(),
                        ActorsInvalidity::SummaryActorAmbiguous(role),
                    );
                    // All individual actors must be consistent with the summary actor
                    context = context.invalidate_if(
                        !Self::filter_kind_role(actors.clone(), ActorKind::Individual, role)
                            .map(|actor| &actor.name)
                            .all(|name| {
                                is_valid_summary_individual_actor_name(&summary_actor.name, name)
                            }),
                        ActorsInvalidity::SummaryNameInconsistentWithIndividualNames(role),
                    );
                } else {
                    // No summary actor
                    debug_assert_eq!(
                        Self::main_actor(actors.clone(), role).is_none(),
                        // Optimization to skip finding the missing summary actor again
                        Self::filter_kind_role(actors.clone(), ActorKind::Individual, role).count()
                            != 1,
                    );
                    context = context.invalidate_if(
                        Self::filter_kind_role(actors.clone(), ActorKind::Individual, role).count()
                            != 1,
                        ActorsInvalidity::MainActorUndefined(role),
                    );
                }
                // At most one sorting entry exists for each role.
                context = context.invalidate_if(
                    Self::filter_kind_role(actors.clone(), ActorKind::Sorting, role).count() > 1,
                    ActorsInvalidity::SortingActorAmbiguous(role),
                );
            }
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

    pub fn summary_actor<'a, I>(actors: I, role: ActorRole) -> Option<&'a Actor>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        Self::filter_kind_role(actors, ActorKind::Summary, role).next()
    }

    pub fn singular_individual_actor<'a, I>(actors: I, role: ActorRole) -> Option<&'a Actor>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        let mut iter = Self::filter_kind_role(actors, ActorKind::Individual, role);
        let first = iter.next();
        if first.is_some() && iter.next().is_none() {
            first
        } else {
            // Missing or ambiguous
            None
        }
    }

    // The singular summary actor or if none exists then the singular individual actor
    pub fn main_actor<'a, I>(actors: I, role: ActorRole) -> Option<&'a Actor>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        Self::summary_actor(actors.clone(), role)
            .or_else(|| Self::singular_individual_actor(actors, role))
    }

    pub fn collect_all_unique_actor_names<'a, I>(actors: I) -> Vec<&'a str>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        let mut actor_names: Vec<_> = actors.map(|actor| actor.name.as_str()).collect();
        actor_names.sort_unstable();
        actor_names.dedup();
        actor_names
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
