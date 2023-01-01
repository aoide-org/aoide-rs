// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
