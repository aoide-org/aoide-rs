// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{filtering::*, sorting::*};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    pub modifier: Option<FilterModifier>,

    // Facets identifiers are always matched with equals. Use an
    // empty vector for matching only tags without a facet.
    pub facets: Option<Vec<String>>,

    pub label: Option<StringPredicate>,

    pub score: Option<NumericPredicate>,
}

impl Filter {
    #[must_use]
    pub fn any_facet() -> Option<Vec<String>> {
        None
    }

    #[must_use]
    pub fn no_facet() -> Option<Vec<String>> {
        Some(Vec::default())
    }

    #[must_use]
    pub fn any_term() -> Option<StringPredicate> {
        None
    }

    #[must_use]
    pub fn any_score() -> Option<NumericPredicate> {
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
