// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{hash::Hash, num::NonZeroUsize, sync::Arc, time::Instant};

use discro::Publisher;
use highway::{HighwayHash as _, HighwayHasher, Key};
use tokio::task::AbortHandle;

use aoide_backend_embedded::track::search;
use aoide_core::{
    CollectionUid,
    track::{Entity, EntityHeader},
};
use aoide_core_api::{Pagination, track::search::Params};

use crate::{ActionEffect, Environment, JoinedTask, modify_shared_state_action_effect};

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

const HIGHWAY_HASH_KEY: Key = Key([0, 0, 0, 0]);

const INITIAL_OFFSET_HASH_SEED: u64 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FetchedEntitiesMemo {
    offset: usize,
    offset_hash: u64,
}

impl FetchedEntitiesMemo {
    #[must_use]
    const fn empty() -> Self {
        Self {
            offset: 0,
            offset_hash: INITIAL_OFFSET_HASH_SEED,
        }
    }

    #[must_use]
    fn next(&self, entity_header: &EntityHeader) -> Self {
        let offset = self.offset + 1;
        let offset_hash = {
            let mut hasher = HighwayHasher::new(HIGHWAY_HASH_KEY);
            self.hash(&mut hasher);
            entity_header.hash(&mut hasher);
            hasher.finalize64()
        };
        Self {
            offset,
            offset_hash,
        }
    }

    #[must_use]
    fn from_slice(fetched_entities: &[FetchedEntity]) -> Self {
        fetched_entities
            .last()
            .map_or_else(Self::empty, |last| Self {
                offset: fetched_entities.len(),
                offset_hash: last.offset_hash,
            })
    }

    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FetchStateMemo {
    #[default]
    Initial,
    Pending {
        since: Instant,
    },
    Ready,
    Failed,
}

impl FetchStateMemo {
    /// Check whether the state is pending.
    #[must_use]
    const fn pending_since(&self) -> Option<Instant> {
        match self {
            Self::Initial | Self::Ready { .. } | Self::Failed { .. } => None,
            Self::Pending { since, .. } => Some(*since),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FetchMemo {
    pub state: FetchStateMemo,
    pub fetched_entities: Option<FetchedEntitiesMemo>,
}

#[derive(Debug, Default)]
enum FetchState {
    #[default]
    Initial,
    Pending {
        fetched_entities_before: Option<Vec<FetchedEntity>>,
        since: Instant,
        task: AbortHandle,
    },
    Ready {
        fetched_entities: Vec<FetchedEntity>,
        can_fetch_more: bool,
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
            Self::Pending { since, .. } => FetchStateMemo::Pending { since: *since },
        }
    }

    #[must_use]
    const fn pending_since(&self) -> Option<Instant> {
        self.fetch_state_memo().pending_since()
    }

    #[must_use]
    const fn is_pending(&self) -> bool {
        self.pending_since().is_some()
    }

    fn abort_pending_task(&self) -> ActionEffect {
        match self {
            Self::Initial | Self::Ready { .. } | Self::Failed { .. } => ActionEffect::Unchanged,
            Self::Pending { task, .. } => {
                task.abort();
                ActionEffect::MaybeChanged
            }
        }
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
    fn memo(&self) -> FetchMemo {
        let state = self.fetch_state_memo();
        let fetched_entities = self.fetched_entities().map(FetchedEntitiesMemo::from_slice);
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

    fn reset(&mut self) -> ActionEffect {
        if matches!(self, Self::Initial) {
            // No effect
            return ActionEffect::Unchanged;
        }
        *self = Default::default();
        debug_assert!(matches!(self, Self::Initial));
        ActionEffect::Changed
    }

    fn fetching_more_succeeded(&mut self, fetched_entities: Vec<Entity>, can_fetch_more: bool) {
        let num_fetched_entities = fetched_entities.len();
        log::debug!("Fetching more succeeded with {num_fetched_entities} newly fetched entities");

        let Self::Pending {
            fetched_entities_before,
            since: _,
            task,
        } = self
        else {
            unreachable!();
        };
        debug_assert!(task.is_finished());
        let mut fetched_entities_before =
            std::mem::take(fetched_entities_before).unwrap_or_default();
        fetched_entities_before.reserve(fetched_entities.len());
        {
            let mut last_memo = FetchedEntitiesMemo::from_slice(&fetched_entities_before);
            fetched_entities_before.extend(fetched_entities.into_iter().map(|entity| {
                last_memo = last_memo.next(&entity.hdr);
                FetchedEntity {
                    offset_hash: last_memo.offset_hash,
                    entity,
                }
            }));
        }

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
    }

    fn fetching_more_failed(&mut self, error: anyhow::Error) {
        log::warn!("Fetching more failed: {error}");
        let Self::Pending {
            fetched_entities_before,
            since: _,
            task,
        } = self
        else {
            unreachable!();
        };
        debug_assert!(task.is_finished());
        let fetched_entities_before = std::mem::take(fetched_entities_before);
        *self = Self::Failed {
            fetched_entities_before,
            error,
        };
    }

    fn fetching_more_cancelled(&mut self) {
        log::debug!("Fetching more cancelled");
        let Self::Pending {
            fetched_entities_before,
            since: _,
            task,
        } = self
        else {
            unreachable!();
        };
        debug_assert!(task.is_finished());
        let fetched_entities_before = std::mem::take(fetched_entities_before);
        if let Some(fetched_entities) = fetched_entities_before {
            *self = Self::Ready {
                fetched_entities,
                can_fetch_more: true,
            };
        } else {
            *self = Self::Initial;
        }
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
    pub const fn is_pending(&self) -> bool {
        self.fetch.is_pending()
    }

    pub fn abort_pending_task(&self) -> ActionEffect {
        self.fetch.abort_pending_task()
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
    #[expect(clippy::missing_panics_doc)] // Never panics
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
                offset_hash: _,
            } = fetched_entities;
            let FetchedEntitiesMemo {
                offset: memo_offset,
                offset_hash: memo_offset_hash,
            } = memo_fetched_entities;
            if *memo_offset > 0
                && memo_offset <= offset
                && *memo_offset_hash
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

    fn reset(&mut self) -> ActionEffect {
        let Self {
            default_params: _,
            context,
            fetch,
        } = self;
        let reset_context = Default::default();
        let reset_fetch_effect = fetch.reset();
        if *context == reset_context && matches!(reset_fetch_effect, ActionEffect::Unchanged) {
            // No effect.
            log::debug!("State doesn't need to be reset");
            return ActionEffect::Unchanged;
        }
        self.context = reset_context;
        let reset_context_effect = ActionEffect::Changed;
        debug_assert!(!self.should_prefetch());
        log::debug!("State has been reset");
        reset_context_effect + reset_fetch_effect
    }

    /// Update the collection UID
    ///
    /// Consumed the argument when returning `true`.
    fn update_collection_uid(
        &mut self,
        collection_uid: &mut Option<CollectionUid>,
    ) -> ActionEffect {
        if collection_uid.as_ref() == self.context.collection_uid.as_ref() {
            // No effect.
            log::debug!("Collection UID unchanged: {collection_uid:?}");
            return ActionEffect::Unchanged;
        }
        self.context.collection_uid = collection_uid.take();
        log::debug!(
            "Collection UID updated: {uid:?}",
            uid = self.context.collection_uid
        );
        ActionEffect::Changed + self.fetch.reset()
    }

    /// Update the search parameters
    ///
    /// Consumed the argument when returning `true`.
    fn update_params(&mut self, params: &mut Params) -> ActionEffect {
        if params == &self.context.params {
            // No effect.
            log::debug!("Params unchanged: {params:?}");
            return ActionEffect::Unchanged;
        }
        self.context.params = std::mem::take(params);
        log::debug!("Params updated: {params:?}", params = self.context.params);
        ActionEffect::Changed + self.fetch.reset()
    }

    fn continue_after_fetching_more_task_joined(
        &mut self,
        joined: JoinedTask<FetchMoreResult>,
        continuation: FetchMoreTaskContinuation,
    ) -> ActionEffect {
        let Self {
            default_params,
            context,
            fetch:
                FetchState::Pending {
                    fetched_entities_before,
                    since: pending_since,
                    task,
                },
        } = self
        else {
            return ActionEffect::Unchanged;
        };
        debug_assert!(task.is_finished());

        let FetchMoreTaskContinuation {
            pending_since: continuation_pending_since,
            default_params: continuation_default_params,
            context: continuation_context,
            fetched_entities_before: continuation_fetched_entities_before_memo,
            limit,
        } = continuation;
        if continuation_pending_since != *pending_since
            || continuation_default_params != *default_params
            || continuation_context != *context
            || continuation_fetched_entities_before_memo
                != fetched_entities_before
                    .as_deref()
                    .map(FetchedEntitiesMemo::from_slice)
        {
            return ActionEffect::Unchanged;
        }
        match joined {
            JoinedTask::Completed(Ok(fetched_entities)) => {
                let can_fetch_more = if let Some(limit) = limit {
                    limit.get() <= fetched_entities.len()
                } else {
                    false
                };
                self.fetch
                    .fetching_more_succeeded(fetched_entities, can_fetch_more);
            }
            JoinedTask::Completed(Err(err)) => {
                self.fetch.fetching_more_failed(err.into());
            }
            JoinedTask::Cancelled => {
                self.fetch.fetching_more_cancelled();
            }
            JoinedTask::Panicked(err) => {
                self.fetch.fetching_more_failed(err);
            }
        }

        ActionEffect::MaybeChanged
    }

    fn reset_fetched(&mut self) -> ActionEffect {
        self.fetch.reset()
    }

    fn spawn_fetching_more_task(
        &mut self,
        this: &SharedState,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
        fetch_limit: Option<NonZeroUsize>,
    ) -> ActionEffect {
        let Some(collection_uid) = &self.context.collection_uid else {
            debug_assert!(self.can_fetch_more() != Some(true));
            return ActionEffect::Unchanged;
        };

        let fetched_entities_before = match &mut self.fetch {
            FetchState::Initial => None,
            FetchState::Pending { .. }
            | FetchState::Failed { .. }
            | FetchState::Ready {
                can_fetch_more: false,
                ..
            } => {
                debug_assert!(self.can_fetch_more() != Some(true));
                return ActionEffect::Unchanged;
            }
            FetchState::Ready {
                can_fetch_more: true,
                fetched_entities,
                ..
            } => Some(std::mem::take(fetched_entities)),
        };

        debug_assert!(self.can_fetch_more() == Some(true));
        let pending_since = Instant::now();

        let continuation = {
            let default_params = self.default_params.clone();
            let context = self.context.clone();
            let fetched_entities_before = fetched_entities_before
                .as_deref()
                .map(FetchedEntitiesMemo::from_slice);
            let limit = fetch_limit;
            FetchMoreTaskContinuation {
                pending_since,
                default_params,
                context,
                fetched_entities_before,
                limit,
            }
        };

        let worker_task = rt.spawn({
            let env = Arc::clone(env);
            let collection_uid = collection_uid.clone();
            let params = continuation.context.params.clone();
            let offset = continuation
                .fetched_entities_before
                .as_ref()
                .map(|memo| memo.offset.try_into().expect("convertible"));
            let limit = fetch_limit.map(|limit| limit.get().try_into().expect("convertible"));
            let pagination = Pagination { limit, offset };
            async move { search(env.db_gatekeeper(), collection_uid, params, pagination).await }
        });
        let abort_worker_task = worker_task.abort_handle();
        let _supervisor_task = rt.spawn({
            let this = this.clone();
            async move {
                let joined = JoinedTask::join(worker_task).await;
                let _ = this.continue_after_fetching_more_task_joined(joined, continuation);
            }
        });

        self.fetch = FetchState::Pending {
            fetched_entities_before,
            since: pending_since,
            task: abort_worker_task,
        };

        ActionEffect::MaybeChanged
    }
}

#[derive(Debug)]
pub struct FetchMoreTaskContinuation {
    pending_since: Instant,
    default_params: Params,
    context: Context,
    fetched_entities_before: Option<FetchedEntitiesMemo>,
    limit: Option<NonZeroUsize>,
}

pub type FetchMoreResult = aoide_backend_embedded::Result<Vec<Entity>>;

pub type SharedStateObserver = discro::Observer<State>;
pub type SharedStateSubscriber = discro::Subscriber<State>;

/// Shared, mutable state.
#[derive(Debug, Clone, Default)]
pub struct SharedState(Publisher<State>);

impl SharedState {
    #[must_use]
    pub fn new(initial_state: State) -> Self {
        Self(Publisher::new(initial_state))
    }

    #[must_use]
    pub fn read(&self) -> SharedStateRef<'_> {
        self.0.read()
    }

    #[must_use]
    pub fn observe(&self) -> SharedStateObserver {
        self.0.observe()
    }

    #[must_use]
    pub fn subscribe_changed(&self) -> SharedStateSubscriber {
        self.0.subscribe_changed()
    }

    pub fn reset(&self) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, State::reset)
    }

    pub fn update_collection_uid(
        &self,
        collection_uid: &mut Option<CollectionUid>,
    ) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, |state| {
            state.update_collection_uid(collection_uid)
        })
    }

    pub fn update_params(&self, params: &mut Params) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, |state| state.update_params(params))
    }

    pub fn spawn_fetching_more_task(
        &self,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
        fetch_limit: Option<NonZeroUsize>,
    ) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, |state| {
            state.spawn_fetching_more_task(self, rt, env, fetch_limit)
        })
    }

    fn continue_after_fetching_more_task_joined(
        &self,
        joined: JoinedTask<FetchMoreResult>,
        continuation: FetchMoreTaskContinuation,
    ) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, |state| {
            state.continue_after_fetching_more_task_joined(joined, continuation)
        })
    }

    pub fn reset_fetched(&self) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, State::reset_fetched)
    }
}

pub type SharedStateRef<'a> = discro::Ref<'a, State>;
