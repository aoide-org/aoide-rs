// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_json::tag::FacetKey;

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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<FacetKey>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringPredicate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<NumericPredicate>,
}

#[cfg(feature = "backend")]
impl From<Filter> for _inner::Filter {
    fn from(from: Filter) -> Self {
        let Filter {
            modifier,
            facets,
            label,
            score,
        } = from;
        Self {
            modifier: modifier.map(Into::into),
            facets: facets.map(|facets| facets.into_iter().map(Into::into).collect()),
            label: label.map(Into::into),
            score: score.map(Into::into),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::Filter> for Filter {
    fn from(from: _inner::Filter) -> Self {
        let _inner::Filter {
            modifier,
            facets,
            label,
            score,
        } = from;
        Self {
            modifier: modifier.map(Into::into),
            facets: facets.map(|facets| facets.into_iter().map(Into::into).collect()),
            label: label.map(Into::into),
            score: score.map(Into::into),
        }
    }
}
