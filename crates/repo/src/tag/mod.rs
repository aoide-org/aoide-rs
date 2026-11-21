// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::tag::FacetId;
use aoide_core_api::tag::search::SortOrder;

fn dedup_facets(facets: &mut Vec<FacetId>) {
    facets.sort_unstable();
    facets.dedup();
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectTags {
    /// Both faceted and non-faceted tags.
    #[default]
    All,
    /// Only faceted tags.
    ///
    /// Excludes all non-faceted tags.
    Faceted,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CountParams {
    pub tags: SelectTags,
    pub include_facets: Option<Vec<FacetId>>,
    pub exclude_facets: Vec<FacetId>,
    pub order: Vec<SortOrder>,
}

impl CountParams {
    #[must_use]
    pub const fn all(order: Vec<SortOrder>) -> Self {
        Self {
            tags: SelectTags::All,
            include_facets: None,
            exclude_facets: Vec::new(),
            order,
        }
    }

    pub fn dedup_facets(&mut self) {
        if let Some(include_facets) = &mut self.include_facets {
            dedup_facets(include_facets);
        }
        dedup_facets(&mut self.exclude_facets);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FacetCountParams {
    pub include_facets: Option<Vec<FacetId>>,
    pub exclude_facets: Vec<FacetId>,
    pub order: Vec<SortOrder>,
}

impl FacetCountParams {
    #[must_use]
    pub const fn all(order: Vec<SortOrder>) -> Self {
        Self {
            include_facets: None,
            exclude_facets: Vec::new(),
            order,
        }
    }

    pub fn dedup_facets(&mut self) {
        if let Some(include_facets) = &mut self.include_facets {
            dedup_facets(include_facets);
        }
        dedup_facets(&mut self.exclude_facets);
    }
}

#[cfg(test)]
mod tests;
