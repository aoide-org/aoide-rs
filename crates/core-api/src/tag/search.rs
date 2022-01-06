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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SortField {
    FacetId,
    Label,
    Score,
    Count,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SortOrder {
    pub field: SortField,
    pub direction: SortDirection,
}
