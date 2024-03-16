// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, hash::Hash as _, num::NonZeroUsize, time::Instant};

use highway::{HighwayHash, HighwayHasher, Key};

use aoide_backend_embedded::track::search;
use aoide_core::{
    track::{Entity, EntityHeader},
    CollectionUid,
};
use aoide_core_api::{track::search::Params, Pagination};

use crate::{Handle, JoinedTask, Observable, ObservableReader, ObservableRef};

pub mod tasklet;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Context {
    pub collection_uid: Option<CollectionUid>,
    pub params: Params,
}

#[derive(Debug, Clone)]
pub struct FetchedEntity {
    pub offset_hash: u64,
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FetchedEntitiesMemo {
    pub offset: usize,
    pub last_offset_hash: u64,
}

#[must_use]
pub fn last_offset_hash_of_fetched_entities<'a>(
    fetched_entities: impl Into<Option<&'a [FetchedEntity]>>,
) -> u64 {
    fetched_entities
        .into()
        .and_then(<[FetchedEntity]>::last)
        .map_or(INITIAL_OFFSET_HASH_SEED, |fetched_entity| {
            fetched_entity.offset_hash
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FetchStateMemo {
    #[default]
    Initial,
    Ready,
    Pending {
        pending_since: Instant,
    },
    Failed,
}

impl FetchStateMemo {
    /// Check whether the state is pending.
    #[must_use]
    const fn pending_since(&self) -> Option<Instant> {
        match self {
            Self::Initial | Self::Ready { .. } | Self::Failed { .. } => None,
            Self::Pending { pending_since, .. } => Some(*pending_since),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FetchMemo {
    pub state: FetchStateMemo,
    pub fetched_entities: Option<FetchedEntitiesMemo>,
}

#[derive(Debug, Default)]
enum FetchState {
    #[default]
    Initial,
    Ready {
        fetched_entities: Vec<FetchedEntity>,
        can_fetch_more: bool,
    },
    Pending {
        fetched_entities_before: Option<Vec<FetchedEntity>>,
        pending_since: Instant,
    },
    Failed {
        fetched_entities_before: Option<Vec<FetchedEntity>>,
        error: anyhow::Error,
    },
}

impl FetchState {
    #[must_use]
    const fn fetch_state_memo(&self) -> FetchStateMemo {
        match self {
            Self::Initial => FetchStateMemo::Initial,
            Self::Ready { .. } => FetchStateMemo::Ready,
            Self::Failed { .. } => FetchStateMemo::Failed,
            Self::Pending { pending_since, .. } => FetchStateMemo::Pending {
                pending_since: *pending_since,
            },
        }
    }

    #[must_use]
    const fn pending_since(&self) -> Option<Instant> {
        self.fetch_state_memo().pending_since()
    }

    #[must_use]
    fn fetched_entities(&self) -> Option<&[FetchedEntity]> {
        match self {
            Self::Initial => None,
            Self::Ready {
                fetched_entities, ..
            } => Some(fetched_entities),
            Self::Pending {
                fetched_entities_before,
                ..
            }
            | Self::Failed {
                fetched_entities_before,
                ..
            } => fetched_entities_before.as_deref(),
        }
    }

    #[must_use]
    fn fetched_entities_memo(&self) -> Option<FetchedEntitiesMemo> {
        let fetched_entities = self.fetched_entities()?;
        let offset = fetched_entities.len();
        let last_offset_hash = last_offset_hash_of_fetched_entities(fetched_entities);
        Some(FetchedEntitiesMemo {
            offset,
            last_offset_hash,
        })
    }

    #[must_use]
    fn memo(&self) -> FetchMemo {
        let state = self.fetch_state_memo();
        let fetched_entities = self.fetched_entities_memo();
        FetchMemo {
            state,
            fetched_entities,
        }
    }

    #[must_use]
    const fn should_prefetch(&self) -> bool {
        matches!(self, Self::Initial)
    }

    #[must_use]
    const fn can_fetch_more(&self) -> Option<bool> {
        match self {
            Self::Initial => Some(true), // always, i.e. try at least once
            Self::Pending { .. } | Self::Failed { .. } => None, // undefined
            Self::Ready { can_fetch_more, .. } => Some(*can_fetch_more), // maybe
        }
    }

    #[must_use]
    const fn last_error(&self) -> Option<&anyhow::Error> {
        match self {
            Self::Initial | Self::Pending { .. } | Self::Ready { .. } => None,
            Self::Failed { error, .. } => Some(error),
        }
    }

    fn try_reset(&mut self) -> bool {
        if matches!(self, Self::Initial) {
            // No effect
            return false;
        }
        *self = Default::default();
        debug_assert!(matches!(self, Self::Initial));
        true
    }

    fn try_fetch_more(&mut self) -> bool {
        debug_assert_eq!(Some(true), self.can_fetch_more());
        let fetched_entities_before = match self {
            Self::Initial => None,
            Self::Pending { .. } | Self::Failed { .. } => {
                // Not applicable
                return false;
            }
            Self::Ready {
                fetched_entities, ..
            } => Some(std::mem::take(fetched_entities)),
        };
        *self = Self::Pending {
            fetched_entities_before,
            pending_since: Instant::now(),
        };
        true
    }

    fn fetch_more_succeeded(
        &mut self,
        offset: usize,
        offset_hash: u64,
        fetched_entities: Vec<Entity>,
        can_fetch_more: bool,
    ) -> bool {
        let num_fetched_entities = fetched_entities.len();
        log::debug!("Fetching succeeded with {num_fetched_entities} newly fetched entities");

        let Self::Pending {
            fetched_entities_before,
            pending_since: _,
        } = self
        else {
            // Not applicable
            log::error!("Not pending when fetching succeeded");
            return false;
        };
        let expected_offset = fetched_entities_before.as_ref().map_or(0, Vec::len);
        let expected_offset_hash =
            last_offset_hash_of_fetched_entities(fetched_entities_before.as_deref());
        if offset != expected_offset || offset_hash != expected_offset_hash {
            // Not applicable
            log::warn!(
                "Mismatching offset/hash after fetching succeeded: expected = \
                 {expected_offset}/{expected_offset_hash}, actual = {offset}/{offset_hash}"
            );
            return false;
        }
        let mut offset = offset;
        let mut offset_hash_seed = offset_hash;
        let mut fetched_entities_before =
            std::mem::take(fetched_entities_before).unwrap_or_default();
        fetched_entities_before.reserve(fetched_entities.len());
        fetched_entities_before.extend(fetched_entities.into_iter().map(|entity| {
            let offset_hash = hash_entity_header_at_offset(offset_hash_seed, offset, &entity.hdr);
            offset_hash_seed = offset_hash;
            offset += 1;
            FetchedEntity {
                offset_hash,
                entity,
            }
        }));

        let fetched_entities = fetched_entities_before;
        let num_cached_entities = fetched_entities.len();

        *self = Self::Ready {
            fetched_entities,
            can_fetch_more,
        };

        debug_assert!(num_fetched_entities <= num_cached_entities);
        if num_fetched_entities < num_cached_entities {
            log::debug!(
                "Caching {num_cached_entities_before} + {num_fetched_entities} fetched entities",
                num_cached_entities_before = num_cached_entities - num_fetched_entities
            );
        } else {
            log::debug!("Caching {num_fetched_entities} fetched entities");
        }
        true
    }

    #[allow(clippy::needless_pass_by_value)]
    fn fetch_more_failed(&mut self, error: anyhow::Error) -> bool {
        log::warn!("Fetching failed: {error}");
        let Self::Pending {
            fetched_entities_before,
            pending_since: _,
        } = self
        else {
            // No effect
            log::error!("Not pending when fetching failed");
            return false;
        };
        let fetched_entities_before = std::mem::take(fetched_entities_before);
        *self = Self::Failed {
            fetched_entities_before,
            error,
        };
        true
    }

    fn fetch_more_aborted(&mut self) -> bool {
        log::debug!("Fetching aborted");
        let Self::Pending {
            fetched_entities_before,
            pending_since: _,
        } = self
        else {
            // No effect
            log::error!("Not pending when fetching aborted");
            return false;
        };
        let fetched_entities_before = std::mem::take(fetched_entities_before);
        if let Some(fetched_entities) = fetched_entities_before {
            *self = Self::Ready {
                fetched_entities,
                can_fetch_more: true,
            };
        } else {
            *self = Self::Initial;
        }
        true
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FetchedEntitiesDiff {
    /// Replace all fetched entities.
    ///
    /// This is the fallback and safe default.
    #[default]
    Replace,

    /// Append entities after more have been fetched.
    ///
    /// This is a common case that enables optimizations. It allows
    /// consumers to avoid rebuilding and re-rendering the entire
    /// list of fetched entities.
    Append,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoDiff {
    Unchanged,
    Changed {
        fetched_entities: FetchedEntitiesDiff,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Memo {
    pub default_params: Params,
    pub context: Context,
    pub fetch: FetchMemo,
}

impl Memo {
    pub fn apply_delta(&mut self, delta: MemoDelta) -> &mut Self {
        let Self {
            default_params,
            context,
            fetch,
        } = self;
        let MemoDelta {
            default_params: new_default_params,
            context: new_context,
            fetch: new_fetch,
        } = delta;
        if let Some(new_default_params) = new_default_params {
            *default_params = new_default_params;
        }
        if let Some(new_context) = new_context {
            *context = new_context;
        }
        if let Some(new_fetch) = new_fetch {
            *fetch = new_fetch;
        }
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MemoDelta {
    pub default_params: Option<Params>,
    pub context: Option<Context>,
    pub fetch: Option<FetchMemo>,
}

#[derive(Debug, Default)]
pub struct State {
    default_params: Params,
    context: Context,
    fetch: FetchState,
}

impl State {
    #[must_use]
    pub fn new(default_params: Params) -> Self {
        let context = Context {
            params: default_params.clone(),
            ..Default::default()
        };
        Self {
            default_params,
            context,
            fetch: Default::default(),
        }
    }

    #[must_use]
    pub const fn default_params(&self) -> &Params {
        &self.default_params
    }

    #[must_use]
    pub const fn context(&self) -> &Context {
        &self.context
    }

    #[must_use]
    pub const fn pending_since(&self) -> Option<Instant> {
        self.fetch.pending_since()
    }

    #[must_use]
    pub const fn should_prefetch(&self) -> bool {
        self.context.collection_uid.is_some() && self.fetch.should_prefetch()
    }

    #[must_use]
    pub const fn can_fetch_more(&self) -> Option<bool> {
        if self.context.collection_uid.is_none() {
            // Not applicable
            return Some(false);
        }
        self.fetch.can_fetch_more()
    }

    /// The error of the last fetch operation.
    ///
    /// Only set if the last fetch operation failed, i.e. if the state tag is `Failed`.
    #[must_use]
    pub const fn last_fetch_error(&self) -> Option<&anyhow::Error> {
        self.fetch.last_error()
    }

    #[must_use]
    pub fn fetched_entities(&self) -> Option<&[FetchedEntity]> {
        self.fetch.fetched_entities()
    }

    #[must_use]
    pub fn fetched_entities_len(&self) -> Option<usize> {
        self.fetch.fetched_entities().map(<[_]>::len)
    }

    #[must_use]
    pub fn fetched_entities_memo(&self) -> Option<FetchedEntitiesMemo> {
        self.fetch.fetched_entities_memo()
    }

    fn clone_memo(&self) -> Memo {
        let Self {
            default_params,
            context,
            fetch,
        } = self;
        Memo {
            default_params: default_params.clone(),
            context: context.clone(),
            fetch: fetch.memo(),
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Never panics
    pub fn update_memo_delta(&self, memo: &Memo) -> (MemoDelta, MemoDiff) {
        let Self {
            default_params,
            context,
            fetch,
        } = self;
        let Memo {
            default_params: memo_default_params,
            context: memo_context,
            fetch: memo_fetch,
        } = memo;
        let mut delta = MemoDelta::default();
        if memo_default_params != default_params {
            delta.default_params = Some(default_params.clone());
        }
        if memo_context != context {
            delta.context = Some(context.clone());
        }
        let fetch = fetch.memo();
        if delta != Default::default() {
            delta.fetch = Some(fetch);
            debug_assert_eq!(memo.clone().apply_delta(delta.clone()), &self.clone_memo());
            return (
                delta,
                MemoDiff::Changed {
                    fetched_entities: FetchedEntitiesDiff::Replace,
                },
            );
        }
        if *memo_fetch == fetch {
            debug_assert_eq!(memo.clone().apply_delta(delta.clone()), &self.clone_memo());
            return (delta, MemoDiff::Unchanged);
        }
        let mut fetched_entities_diff = FetchedEntitiesDiff::Replace;
        if let (Some(fetched_entities), Some(memo_fetched_entities)) =
            (&fetch.fetched_entities, &memo_fetch.fetched_entities)
        {
            let FetchedEntitiesMemo {
                offset,
                last_offset_hash: _,
            } = fetched_entities;
            let FetchedEntitiesMemo {
                offset: memo_offset,
                last_offset_hash: memo_last_offset_hash,
            } = memo_fetched_entities;
            debug_assert_eq!(*offset, self.fetched_entities().unwrap().len());
            if *memo_offset > 0
                && memo_offset <= offset
                && *memo_last_offset_hash
                    == self
                        .fetched_entities()
                        .unwrap()
                        .get(*memo_offset - 1)
                        .unwrap()
                        .offset_hash
            {
                // Optimize update by only appending the newly fetched entities.
                fetched_entities_diff = FetchedEntitiesDiff::Append;
            }
        }
        delta.fetch = Some(fetch);
        debug_assert_eq!(memo.clone().apply_delta(delta.clone()), &self.clone_memo());
        (
            delta,
            MemoDiff::Changed {
                fetched_entities: fetched_entities_diff,
            },
        )
    }

    #[must_use]
    pub fn update_memo(&self, memo: &mut Memo) -> MemoDiff {
        let (delta, diff) = self.update_memo_delta(memo);
        memo.apply_delta(delta);
        diff
    }

    fn try_reset(&mut self) -> bool {
        // Cloning the default params once for pre-creating the target state
        // is required to avoid redundant code for determining in advance if
        // the state would actually change or not.
        let reset = Self::new(self.default_params.clone());
        let Self {
            default_params: _,
            context: reset_context,
            fetch: reset_fetch,
        } = reset;
        debug_assert!(matches!(reset_fetch, FetchState::Initial));
        if self.context == reset_context && matches!(self.fetch, FetchState::Initial) {
            // No effect.
            log::debug!("State doesn't need to be reset");
            return false;
        }
        self.context = reset_context;
        self.fetch = reset_fetch;
        debug_assert!(!self.should_prefetch());
        log::info!("State has been reset");
        true
    }

    /// Update the collection UID
    ///
    /// Consumed the argument when returning `true`.
    fn try_update_collection_uid(&mut self, collection_uid: &mut Option<CollectionUid>) -> bool {
        if collection_uid.as_ref() == self.context.collection_uid.as_ref() {
            // No effect.
            log::debug!("Collection UID unchanged: {collection_uid:?}");
            return false;
        }
        self.context.collection_uid = collection_uid.take();
        self.fetch.try_reset();
        if let Some(uid) = &self.context.collection_uid {
            log::info!("Collection UID updated: {uid}");
        } else {
            log::info!("Collection UID updated: <none>");
        }
        true
    }

    /// Update the search parameters
    ///
    /// Consumed the argument when returning `true`.
    fn try_update_params(&mut self, params: &mut Params) -> bool {
        if params == &self.context.params {
            // No effect.
            log::debug!("Params unchanged: {params:?}");
            return false;
        }
        self.context.params = std::mem::take(params);
        self.fetch.try_reset();
        log::info!("Params updated: {params:?}", params = self.context.params);
        true
    }

    fn try_fetch_more(&mut self) -> bool {
        debug_assert_eq!(Some(true), self.can_fetch_more());
        self.fetch.try_fetch_more()
    }

    fn fetch_more_task_joined(
        &mut self,
        joined_tasked: JoinedTask<FetchMoreResult>,
        continuation: FetchMoreTaskContinuation,
    ) -> bool {
        let FetchMoreTaskContinuation {
            context,
            offset,
            offset_hash,
            limit,
        } = continuation;
        if context != self.context {
            log::warn!(
                "Mismatching context after fetching succeeded: expected = {expected_context:?}, \
                 actual = {context:?}",
                expected_context = self.context
            );
            // No effect.
            return false;
        }
        match joined_tasked {
            JoinedTask::Cancelled => self.fetch.fetch_more_aborted(),
            JoinedTask::Completed(Ok(fetched_entities)) => {
                let can_fetch_more = if let Some(limit) = limit {
                    limit.get() <= fetched_entities.len()
                } else {
                    false
                };
                self.fetch.fetch_more_succeeded(
                    offset,
                    offset_hash,
                    fetched_entities,
                    can_fetch_more,
                )
            }
            JoinedTask::Completed(Err(err)) => self.fetch.fetch_more_failed(err.into()),
            JoinedTask::Panicked(err) => self.fetch.fetch_more_failed(err),
        }
    }

    fn try_reset_fetched(&mut self) -> bool {
        self.fetch.try_reset()
    }
}

#[derive(Debug)]
pub struct FetchMoreTaskContinuation {
    context: Context,
    offset: usize,
    offset_hash: u64,
    limit: Option<NonZeroUsize>,
}

pub type FetchMoreResult = aoide_backend_embedded::Result<Vec<Entity>>;

fn try_fetch_more_task(
    handle: &Handle,
    state: &mut State,
    fetch_limit: Option<NonZeroUsize>,
) -> Option<(
    impl Future<Output = FetchMoreResult> + Send + 'static,
    FetchMoreTaskContinuation,
)> {
    if state.can_fetch_more() != Some(true) || !state.try_fetch_more() {
        // Not modified.
        return None;
    }

    let Context {
        collection_uid,
        params,
    } = &state.context;
    let collection_uid = collection_uid.clone()?;
    let params = params.clone();
    let offset = state
        .fetched_entities()
        .and_then(|slice| slice.len().try_into().ok());
    let limit = fetch_limit.and_then(|limit| limit.get().try_into().ok());
    let pagination = Pagination { limit, offset };
    let handle = handle.clone();
    let task =
        async move { search(handle.db_gatekeeper(), collection_uid, params, pagination).await };

    let context = state.context.clone();
    let offset = offset.unwrap_or(0) as usize;
    let offset_hash = last_offset_hash_of_fetched_entities(state.fetched_entities());
    let limit = fetch_limit;
    let continuation = FetchMoreTaskContinuation {
        context,
        offset,
        offset_hash,
        limit,
    };

    Some((task, continuation))
}

pub type StateSubscriber = discro::Subscriber<State>;

/// Manages the mutable, observable state
#[derive(Debug, Default)]
pub struct ObservableState(Observable<State>);

impl ObservableState {
    #[must_use]
    pub fn new(initial_state: State) -> Self {
        Self(Observable::new(initial_state))
    }

    #[must_use]
    pub fn read(&self) -> ObservableStateRef<'_> {
        self.0.read()
    }

    #[must_use]
    pub fn subscribe_changed(&self) -> StateSubscriber {
        self.0.subscribe_changed()
    }

    pub fn set_modified(&self) {
        self.0.set_modified();
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_reset(&self) -> bool {
        self.0.modify(State::try_reset)
    }

    pub fn try_update_collection_uid(&self, collection_uid: &mut Option<CollectionUid>) -> bool {
        self.0
            .modify(|state| state.try_update_collection_uid(collection_uid))
    }

    pub fn try_update_params(&self, params: &mut Params) -> bool {
        self.0.modify(|state| state.try_update_params(params))
    }

    #[must_use]
    pub fn try_fetch_more_task(
        &self,
        handle: &Handle,
        fetch_limit: Option<NonZeroUsize>,
    ) -> Option<(
        impl Future<Output = FetchMoreResult> + Send + 'static,
        FetchMoreTaskContinuation,
    )> {
        let mut maybe_fetch_more = None;
        self.0.modify(|state| {
            let Some(fetch_more) = try_fetch_more_task(handle, state, fetch_limit) else {
                return false;
            };
            maybe_fetch_more = Some(fetch_more);
            true
        });
        maybe_fetch_more
    }

    #[allow(clippy::must_use_candidate)]
    pub fn fetch_more_task_joined(
        &self,
        joined_task: JoinedTask<FetchMoreResult>,
        continuation: FetchMoreTaskContinuation,
    ) -> bool {
        self.0
            .modify(|state| state.fetch_more_task_joined(joined_task, continuation))
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_reset_fetched(&self) -> bool {
        self.0.modify(State::try_reset_fetched)
    }
}

pub type ObservableStateRef<'a> = ObservableRef<'a, State>;

impl ObservableReader<State> for ObservableState {
    fn read_lock(&self) -> ObservableStateRef<'_> {
        self.0.read_lock()
    }
}

const INITIAL_OFFSET_HASH_SEED: u64 = 0;

const fn hash_key_for_offset(seed: u64, offset: usize) -> Key {
    let offset_u64 = offset as u64;
    Key([seed, offset_u64, seed, offset_u64])
}

fn hash_entity_header_at_offset(seed: u64, offset: usize, entity_header: &EntityHeader) -> u64 {
    debug_assert_eq!(seed == INITIAL_OFFSET_HASH_SEED, offset == 0);
    let mut hasher = HighwayHasher::new(hash_key_for_offset(seed, offset));
    offset.hash(&mut hasher);
    // Ugly workaround, because the tag type does not implement `Hash`. No allocations.
    entity_header.clone().into_untyped().hash(&mut hasher);
    hasher.finalize64()
}

#[cfg(test)]
mod tests {
    use highway::Key;

    use crate::track::repo_search::{hash_key_for_offset, INITIAL_OFFSET_HASH_SEED};

    #[test]
    fn default_hash_key_equals_offset_zero() {
        assert_eq!(
            Key::default().0,
            hash_key_for_offset(INITIAL_OFFSET_HASH_SEED, 0).0
        );
    }
}
