// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::tag::*;

use aoide_core_api::tag::search::*;

fn dedup_facets(facets: &mut Vec<FacetId>) {
    facets.sort_unstable();
    facets.dedup();
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CountParams {
    pub facets: Option<Vec<FacetId>>,
    pub include_non_faceted_tags: Option<bool>,
    pub ordering: Vec<SortOrder>,
}

impl CountParams {
    pub fn dedup_facets(&mut self) {
        if let Some(ref mut facets) = self.facets {
            dedup_facets(facets);
        }
    }

    #[must_use]
    pub fn include_non_faceted_tags(&self) -> bool {
        self.include_non_faceted_tags.unwrap_or(true)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FacetCountParams {
    pub facets: Option<Vec<FacetId>>,
    pub ordering: Vec<SortOrder>,
}

impl FacetCountParams {
    pub fn dedup_facets(&mut self) {
        if let Some(ref mut facets) = self.facets {
            dedup_facets(facets);
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct FacetCount {
    pub facet_id: FacetId,
    pub total_count: usize,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct AvgScoreCount {
    pub facet_id: Option<FacetId>,
    pub label: Option<Label>,
    pub avg_score: Score,
    pub total_count: usize,
}
