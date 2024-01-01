// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use nonicle::{CanonicalOrd, Canonicalize, IsCanonical};
use regex::{Regex, RegexBuilder};
use strum::FromRepr;

use crate::prelude::*;

///////////////////////////////////////////////////////////////////////
// Role
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, FromRepr)]
#[repr(u8)]
pub enum Role {
    #[default]
    Artist = 0,
    Arranger = 1,
    Composer = 2,
    Conductor = 3,
    MixDj = 4,
    Engineer = 5,
    Lyricist = 6,
    MixEngineer = 7,
    Performer = 8,
    Producer = 9,
    Director = 10,
    Remixer = 11,
    Writer = 12,
}

///////////////////////////////////////////////////////////////////////
// Kind
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, FromRepr)]
#[repr(u8)]
pub enum Kind {
    #[default]
    Summary = 0, /* unspecified for display, may mention multiple actors with differing kinds and roles */
    Individual = 1, // single persons or group/band names
    Sorting = 2,
}

///////////////////////////////////////////////////////////////////////
// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Actor {
    pub role: Role,

    pub kind: Kind,

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
    SummaryActorAmbiguous(Role),
    SortingActorAmbiguous(Role),
    SummaryNameInconsistentWithIndividualNames(Role),
    MainActorUndefined(Role),
}

pub const ANY_ROLE_FILTER: Option<Role> = None;
pub const ANY_RANK_FILTER: Option<Kind> = None;

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
                    Self::filter_kind_role(actors.clone(), Kind::Summary, role);
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
                        !Self::filter_kind_role(actors.clone(), Kind::Individual, role)
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
                        Self::filter_kind_role(actors.clone(), Kind::Individual, role).count() != 1,
                    );
                    context = context.invalidate_if(
                        Self::filter_kind_role(actors.clone(), Kind::Individual, role).count() != 1,
                        ActorsInvalidity::MainActorUndefined(role),
                    );
                }
                // At most one sorting entry exists for each role.
                context = context.invalidate_if(
                    Self::filter_kind_role(actors.clone(), Kind::Sorting, role).count() > 1,
                    ActorsInvalidity::SortingActorAmbiguous(role),
                );
            }
        }
        context.into()
    }

    pub fn filter_kind_role<'a, I>(
        actors: I,
        kind: impl Into<Option<Kind>>,
        role: impl Into<Option<Role>>,
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

    pub fn summary_actor<'a, I>(actors: I, role: Role) -> Option<&'a Actor>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        Self::filter_kind_role(actors, Kind::Summary, role).next()
    }

    pub fn singular_individual_actor<'a, I>(actors: I, role: Role) -> Option<&'a Actor>
    where
        I: Iterator<Item = &'a Actor> + Clone,
    {
        let mut iter = Self::filter_kind_role(actors, Kind::Individual, role);
        let first = iter.next();
        if first.is_some() && iter.next().is_none() {
            first
        } else {
            // Missing or ambiguous
            None
        }
    }

    // The singular summary actor or if none exists then the singular individual actor
    pub fn main_actor<'a, I>(actors: I, role: Role) -> Option<&'a Actor>
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

#[derive(Debug)]
pub struct ActorNamesSummarySplitter {
    name_separator_regex: Regex,
    protected_names: AhoCorasick,
}

impl ActorNamesSummarySplitter {
    /// Creates a new actor name splitter.
    ///
    /// Both `separators` and `protected_names` are case-insensitive.
    ///
    /// Space characters in are replaced by whitespace matching regexes.
    #[allow(clippy::missing_panics_doc)] // never panics
    pub fn new<'a>(
        name_separators: impl IntoIterator<Item = &'a str>,
        protected_names: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        let name_separator_pattern = format!(
            r"({})",
            name_separators
                .into_iter()
                .map(|separator| regex::escape(separator).replace(' ', r"\s+"))
                .collect::<Vec<_>>()
                .join("|")
        );
        let name_separator_regex = RegexBuilder::new(&name_separator_pattern)
            .case_insensitive(true)
            .build()
            .unwrap();
        let protected_names = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .match_kind(aho_corasick::MatchKind::LeftmostFirst)
            .build(protected_names)
            .unwrap();
        Self {
            name_separator_regex,
            protected_names,
        }
    }

    fn split_next<'a>(&self, mut name: &'a str) -> Option<(&'a str, Option<&'a str>)> {
        let mut skipped_leading_separator = false;
        loop {
            let name_trimmed = name.trim_start();
            if let Some(first_match) = self.protected_names.find(name_trimmed) {
                if first_match.start() == 0 {
                    let (first_name, rest) = name_trimmed.split_at(first_match.end());
                    let rest = rest.trim_end();
                    let rest = if rest.is_empty() { None } else { Some(rest) };
                    return Some((first_name, rest));
                }
            }
            if skipped_leading_separator {
                break;
            }
            if let Some(separator_match) = self.name_separator_regex.find(name) {
                debug_assert!(separator_match.end() > 0);
                if separator_match.start() == 0 {
                    // Skip leading separator
                    name = &name[separator_match.end()..];
                    debug_assert!(!skipped_leading_separator);
                    skipped_leading_separator = true;
                    // Try to find another protected name after the separator
                    continue;
                }
            }
            break;
        }
        let mut regex_split_iter = self.name_separator_regex.splitn(name, 2);
        let first_name = regex_split_iter.next()?.trim();
        let rest = regex_split_iter.next();
        Some((first_name, rest))
    }

    pub fn split_all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        let mut rest = Some(name);
        std::iter::from_fn(move || loop {
            let (first_name, next_rest) = self.split_next(rest?)?;
            rest = next_rest;
            if first_name.is_empty() {
                continue;
            }
            return Some(first_name);
        })
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
