// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::tag::{FacetId, Label, Score};
use aoide_core_api::tag::search::SortOrder;

fn dedup_facets(facets: &mut Vec<FacetId<'_>>) {
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
pub struct CountParams<'a> {
    pub tags: SelectTags,
    pub include_facets: Option<Vec<FacetId<'a>>>,
    pub exclude_facets: Vec<FacetId<'a>>,
    pub ordering: Vec<SortOrder>,
}

impl<'a> CountParams<'a> {
    #[must_use]
    pub const fn all(ordering: Vec<SortOrder>) -> Self {
        Self {
            tags: SelectTags::All,
            include_facets: None,
            exclude_facets: Vec::new(),
            ordering,
        }
    }

    #[must_use]
    pub const fn all_faceted(ordering: Vec<SortOrder>) -> Self {
        Self {
            tags: SelectTags::Faceted,
            include_facets: None,
            exclude_facets: Vec::new(),
            ordering,
        }
    }

    #[must_use]
    pub const fn all_non_faceted(ordering: Vec<SortOrder>) -> Self {
        Self {
            tags: SelectTags::All,
            include_facets: Some(vec![]),
            exclude_facets: Vec::new(),
            ordering,
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
pub struct FacetCountParams<'a> {
    pub include_facets: Option<Vec<FacetId<'a>>>,
    pub exclude_facets: Vec<FacetId<'a>>,
    pub ordering: Vec<SortOrder>,
}

impl<'a> FacetCountParams<'a> {
    #[must_use]
    pub const fn all(ordering: Vec<SortOrder>) -> Self {
        Self {
            include_facets: None,
            exclude_facets: Vec::new(),
            ordering,
        }
    }

    pub fn dedup_facets(&mut self) {
        if let Some(include_facets) = &mut self.include_facets {
            dedup_facets(include_facets);
        }
        dedup_facets(&mut self.exclude_facets);
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct FacetCount<'a> {
    pub facet_id: FacetId<'a>,
    pub total_count: usize,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct AvgScoreCount<'a> {
    pub facet_id: Option<FacetId<'a>>,
    pub label: Option<Label<'a>>,
    pub avg_score: Score,
    pub total_count: usize,
}

#[cfg(test)]
mod tests;
