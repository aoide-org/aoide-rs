// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::{
    filtering::{FilterModifier, NumericPredicate, StringPredicate},
    prelude::*,
};

mod _inner {
    pub(super) use crate::_inner::tag::search::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Filter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,

    // Facets are always matched with equals. Use an empty vector
    // for matching only tags without a facet_id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringPredicate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<NumericPredicate>,
}

#[cfg(feature = "backend")]
impl From<Filter> for _inner::Filter {
    fn from(from: Filter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            facets: from.facets,
            label: from.label.map(Into::into),
            score: from.score.map(Into::into),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::Filter> for Filter {
    fn from(from: _inner::Filter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            facets: from.facets,
            label: from.label.map(Into::into),
            score: from.score.map(Into::into),
        }
    }
}
