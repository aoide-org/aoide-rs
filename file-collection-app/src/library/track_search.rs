// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{num::NonZeroUsize, time::Instant};

use discro::{Ref, Subscriber};

use aoide::{
    api::{
        SortDirection,
        filtering::{NumericPredicate, ScalarFieldFilter, StringPredicate},
        media::source::ResolveUrlFromContentPath,
        tag::search::{FacetsFilter, Filter as TagFilter},
        track::search::{Filter, NumericField, PhraseFieldFilter, SortOrder, StringField},
    },
    desktop_app::track,
    track::tag::{
        FACET_KEY_COMMENT, FACET_KEY_DESCRIPTION, FACET_KEY_GENRE, FACET_KEY_GROUPING,
        FACET_KEY_ISRC, FACET_KEY_MOOD, FACET_KEY_XID,
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

const DEFAULT_PREFETCH_LIMIT_USIZE: usize = 250;
pub(super) const DEFAULT_PREFETCH_LIMIT: NonZeroUsize =
    NonZeroUsize::MIN.saturating_add(DEFAULT_PREFETCH_LIMIT_USIZE - 1);

#[derive(Debug)]
pub enum Event {
    StateChanged,
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

        log::debug!("Suspending watch_state");
        if subscriber.changed().await.is_err() {
            log::info!("Stop watching track search state after publisher has been dropped");
            break;
        }
        log::debug!("Resuming watch_state");
    }
}

#[derive(Debug)]
pub enum MemoState {
    Ready(Memo),
    Pending {
        memo: Memo,
        memo_delta: MemoDelta,
        state_changed_again: bool,
        pending_since: Instant,
    },
}

impl MemoState {
    #[must_use]
    fn new_pending(memo: Memo, memo_delta: MemoDelta) -> Self {
        Self::Pending {
            memo,
            memo_delta,
            state_changed_again: false,
            pending_since: Instant::now(),
        }
    }

    pub fn try_start_pending<'a>(
        &'a mut self,
        shared_state: &SharedState,
    ) -> Option<(&'a Memo, MemoDiff)> {
        let (memo, memo_delta, memo_diff) = {
            let memo = match self {
                Self::Ready(memo) => memo,
                Self::Pending {
                    state_changed_again,
                    ..
                } => {
                    log::debug!("State changed again");
                    *state_changed_again = true;
                    return None;
                }
            };
            let (memo_delta, memo_diff) = {
                let state = shared_state.read();
                state.update_memo_delta(memo)
            };
            let memo = std::mem::take(memo);
            (memo, memo_delta, memo_diff)
        };
        *self = Self::new_pending(memo, memo_delta);
        let Self::Pending { memo, .. } = self else {
            unreachable!();
        };
        Some((memo, memo_diff))
    }

    #[must_use]
    pub const fn pending_since(&self) -> Option<Instant> {
        match self {
            Self::Ready(_) => None,
            Self::Pending { pending_since, .. } => Some(*pending_since),
        }
    }

    pub fn abort(&mut self) -> bool {
        match self {
            Self::Ready(_) => {
                // Unchanged.
                false
            }
            MemoState::Pending { memo, .. } => {
                *self = Self::Ready(std::mem::take(memo));
                true
            }
        }
    }

    pub fn complete(&self) -> Result<(&Memo, &MemoDelta), MemoStateCompletionError> {
        match self {
            MemoState::Ready(_) => Err(MemoStateCompletionError::NotPending),
            MemoState::Pending {
                memo,
                memo_delta,
                state_changed_again,
                pending_since,
            } => {
                log::debug!(
                    "Memo state pending completed after {elapsed_ms} ms",
                    elapsed_ms = pending_since.elapsed().as_secs_f64() * 1000.0
                );
                if *state_changed_again {
                    Err(MemoStateCompletionError::AbortPendingAndRetry)
                } else {
                    Ok((memo, memo_delta))
                }
            }
        }
    }
}

impl Default for MemoState {
    fn default() -> Self {
        Self::Ready(Default::default())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MemoStateCompletionError {
    NotPending,
    AbortPendingAndRetry,
}

const HASHTAG_LABEL_PREFIX: &str = "#";

const BPM_GREATER_OR_EQUAL_PREFIX: char = '+';
const BPM_LESS_OR_EQUAL_PREFIX: char = '-';

#[derive(Debug)]
enum BpmFilter {
    GreaterOrEqual,
    LessOrEqual,
}

pub(super) fn parse_filter_from_input(input: &str) -> Option<Filter> {
    debug_assert_eq!(input, input.trim());
    if input.is_empty() {
        return None;
    }
    let phrase_fields = [StringField::Publisher];
    let predefined_tag_facets = [
        FACET_KEY_COMMENT.clone(),
        FACET_KEY_GROUPING.clone(),
        FACET_KEY_GENRE.clone(),
        FACET_KEY_MOOD.clone(),
        FACET_KEY_DESCRIPTION.clone(),
        FACET_KEY_XID.clone(),
        FACET_KEY_ISRC.clone(),
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
            let mut bpm_filter = BpmFilter::GreaterOrEqual;
            if let Some(bpm) = term
                .strip_prefix(BPM_GREATER_OR_EQUAL_PREFIX)
                .or_else(|| {
                    bpm_filter = BpmFilter::LessOrEqual;
                    term.strip_prefix(BPM_LESS_OR_EQUAL_PREFIX)
                })
                .and_then(|bpm_digits| bpm_digits.parse::<u16>().ok())
            {
                let predicate = match bpm_filter {
                    BpmFilter::GreaterOrEqual => NumericPredicate::GreaterOrEqual(bpm.into()),
                    BpmFilter::LessOrEqual => NumericPredicate::LessOrEqual(bpm.into()),
                };
                return Filter::Numeric(ScalarFieldFilter {
                    field: NumericField::MusicTempoBpm,
                    predicate,
                });
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
