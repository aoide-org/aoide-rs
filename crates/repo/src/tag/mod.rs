// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::tag::*;
use aoide_core_api::tag::search::*;

fn dedup_facets(facets: &mut Vec<FacetId<'_>>) {
    facets.sort_unstable();
    facets.dedup();
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CountParams<'a> {
    pub facets: Option<Vec<FacetId<'a>>>,
    pub include_non_faceted_tags: Option<bool>,
    pub ordering: Vec<SortOrder>,
}

impl<'a> CountParams<'a> {
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
pub struct FacetCountParams<'a> {
    pub facets: Option<Vec<FacetId<'a>>>,
    pub ordering: Vec<SortOrder>,
}

impl<'a> FacetCountParams<'a> {
    pub fn dedup_facets(&mut self) {
        if let Some(ref mut facets) = self.facets {
            dedup_facets(facets);
        }
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
