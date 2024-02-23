// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::num::NonZeroUsize;

use discro::{Ref, Subscriber};

use aoide::{
    api::{
        filtering::StringPredicate,
        media::source::ResolveUrlFromContentPath,
        sorting::SortDirection,
        tag::search::{FacetsFilter, Filter as TagFilter},
        track::search::{Filter, PhraseFieldFilter, SortOrder, StringField},
    },
    desktop_app::{collection::SynchronizeVfsTaskContinuation, track},
    tag::FacetKey,
    track::tag::{
        FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING, FACET_ID_ISRC,
        FACET_ID_MOOD, FACET_ID_STYLE, FACET_ID_VIBE, FACET_ID_XID,
    },
};

use crate::NoReceiverForEvent;

use super::EventEmitter;

// Re-exports
pub use track::repo_search::*;

// We always need the URL in addition to the virtual file path
const RESOLVE_TRACK_URL_FROM_CONTENT_PATH: Option<ResolveUrlFromContentPath> =
    Some(ResolveUrlFromContentPath::CanonicalRootUrl);

pub(super) fn default_params() -> aoide::api::track::search::Params {
    aoide::api::track::search::Params {
        resolve_url_from_content_path: RESOLVE_TRACK_URL_FROM_CONTENT_PATH.clone(),
        ordering: DEFAULT_SORT_ORDER.to_vec(),
        ..Default::default()
    }
}

// Show recently updated tracks first.
const DEFAULT_SORT_ORDER: &[SortOrder] = &[SortOrder {
    field: aoide::api::track::search::SortField::UpdatedAt,
    direction: SortDirection::Descending,
}];

const DEFAULT_PREFETCH_LIMIT_USIZE: usize = 1_000;
pub(super) const DEFAULT_PREFETCH_LIMIT: NonZeroUsize =
    NonZeroUsize::MIN.saturating_add(DEFAULT_PREFETCH_LIMIT_USIZE - 1);

#[derive(Debug)]
#[allow(dead_code)] // TODO
struct SynchronizeMusicDirCompleted {
    continuation: SynchronizeVfsTaskContinuation,
    result:
        Option<anyhow::Result<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>>,
}

#[derive(Debug)]
pub enum Event {
    StateChanged,
    FetchMoreTaskCompleted {
        result: FetchMoreResult,
        continuation: FetchMoreTaskContinuation,
    },
}

impl From<Event> for super::Event {
    fn from(event: Event) -> Self {
        Self::TrackSearch(event)
    }
}

pub type StateRef<'a> = Ref<'a, State>;
pub type StateSubscriber = Subscriber<State>;

pub(super) async fn watch_state<E>(mut subscriber: StateSubscriber, event_emitter: E)
where
    E: EventEmitter,
{
    // The first event is always emitted immediately.
    loop {
        drop(subscriber.read_ack());
        if let Err(NoReceiverForEvent) = event_emitter.emit_event(Event::StateChanged.into()) {
            log::info!("Stop watching track search state after event receiver has been dropped");
            break;
        };
        if subscriber.changed().await.is_err() {
            log::info!("Stop watching track search state after publisher has been dropped");
            break;
        }
    }
}

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
