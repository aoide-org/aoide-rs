// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// Re-exports
pub(super) use aoide::desktop_app::track::repo_search::*;
use aoide::{
    api::{
        filtering::StringPredicate,
        tag::search::{FacetsFilter, Filter as TagFilter},
        track::search::{Filter, PhraseFieldFilter, StringField},
    },
    tag::FacetKey,
    track::tag::{
        FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING, FACET_ID_ISRC,
        FACET_ID_MOOD, FACET_ID_STYLE, FACET_ID_VIBE, FACET_ID_XID,
    },
};

const HASHTAG_LABEL_PREFIX: &str = "#";

pub(super) fn parse_filter_from_input(input: &str) -> Option<Filter> {
    debug_assert_eq!(input, input.trim());
    if input.is_empty() {
        return None;
    }
    let phrase_fields = [StringField::Publisher];
    let predefined_tag_facets = [
        FacetKey::from(FACET_ID_COMMENT),
        FacetKey::from(FACET_ID_GROUPING),
        FacetKey::from(FACET_ID_GENRE),
        FacetKey::from(FACET_ID_MOOD),
        FacetKey::from(FACET_ID_STYLE),
        FacetKey::from(FACET_ID_DESCRIPTION),
        FacetKey::from(FACET_ID_VIBE),
        FacetKey::from(FACET_ID_XID),
        FacetKey::from(FACET_ID_ISRC),
    ];
    let facets_filter = FacetsFilter::AnyOf(predefined_tag_facets.to_vec());
    // The size of the filter and as a consequence the execution time
    // scales linearly with the number of terms in the input.
    let all_filters: Vec<_> = input
        .split_whitespace()
        .map(|term| {
            if let Some(hashtag_label) = term.strip_prefix(HASHTAG_LABEL_PREFIX) {
                if !hashtag_label.is_empty() {
                    // Exclude predefined facets from the hashtag search.
                    let facets = FacetsFilter::NoneOf(predefined_tag_facets.to_vec());
                    let label = StringPredicate::StartsWith(hashtag_label.to_owned().into());
                    return Filter::Tag(TagFilter {
                        facets: Some(facets),
                        label: Some(label),
                        ..Default::default()
                    });
                }
            }
            let title_phrase = Filter::TitlePhrase(aoide::api::track::search::TitlePhraseFilter {
                name_terms: vec![term.to_owned()],
                ..Default::default()
            });
            let actor_phrase = Filter::ActorPhrase(aoide::api::track::search::ActorPhraseFilter {
                name_terms: vec![term.to_owned()],
                ..Default::default()
            });
            let field_phrase = Filter::Phrase(PhraseFieldFilter {
                fields: phrase_fields.to_vec(),
                terms: vec![term.to_owned()],
            });
            let tag = Filter::Tag(aoide::api::tag::search::Filter {
                facets: Some(facets_filter.clone()),
                label: Some(StringPredicate::Contains(term.to_owned().into())),
                ..Default::default()
            });
            Filter::Any(vec![title_phrase, actor_phrase, field_phrase, tag])
        })
        .collect();
    debug_assert!(!all_filters.is_empty());
    Some(Filter::All(all_filters))
}
