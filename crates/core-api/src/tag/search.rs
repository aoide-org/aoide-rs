// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::tag::FacetKey;

use crate::{filtering::*, sorting::*};

/// Filter by facets.
///
/// Both an empty vector or a default element inside a non-empty
/// vector match all unfaceted tags, i.e. tags without a facet.
#[derive(Clone, Debug, PartialEq)]
pub enum FacetsFilter<'a> {
    Prefix(FacetKey<'a>),
    AnyOf(Vec<FacetKey<'a>>),
    /// Not [`AnyOf`](Self::AnyOf).
    NoneOf(Vec<FacetKey<'a>>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    pub modifier: Option<FilterModifier>,

    pub facets: Option<FacetsFilter<'static>>,

    pub label: Option<StringPredicate<'static>>,

    pub score: Option<NumericPredicate>,
}

impl Filter {
    #[must_use]
    pub const fn any_facet() -> Option<Vec<String>> {
        None
    }

    #[must_use]
    pub fn no_facet() -> Option<Vec<String>> {
        Some(Vec::default())
    }

    #[must_use]
    pub const fn any_term() -> Option<StringPredicate<'static>> {
        None
    }

    #[must_use]
    pub const fn any_score() -> Option<NumericPredicate> {
        None
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SortField {
    FacetId,
    Label,
    Score,
    Count,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SortOrder {
    pub field: SortField,
    pub direction: SortDirection,
}
