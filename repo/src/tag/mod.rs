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

use aoide_core::{tag::*, usecases::tags::search::*};

fn dedup_facets(facets: &mut Vec<Facet>) {
    facets.sort_unstable();
    facets.dedup();
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CountParams {
    pub facets: Option<Vec<Facet>>,
    pub include_non_faceted_tags: Option<bool>,
    pub ordering: Vec<SortOrder>,
}

impl CountParams {
    pub fn dedup_facets(&mut self) {
        if let Some(ref mut facets) = self.facets {
            dedup_facets(facets);
        }
    }

    pub fn include_non_faceted_tags(&self) -> bool {
        self.include_non_faceted_tags.unwrap_or(true)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FacetCountParams {
    pub facets: Option<Vec<Facet>>,
    pub ordering: Vec<SortOrder>,
}

impl FacetCountParams {
    pub fn dedup_facets(&mut self) {
        if let Some(ref mut facets) = self.facets {
            dedup_facets(facets);
        }
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct FacetCount {
    pub facet: Facet,
    pub total_count: usize,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct AvgScoreCount {
    pub facet: Option<Facet>,
    pub label: Option<Label>,
    pub avg_score: Score,
    pub total_count: usize,
}
