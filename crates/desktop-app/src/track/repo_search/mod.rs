// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{hash::Hash as _, num::NonZeroUsize};

use aoide_backend_embedded::track::search;
use aoide_core::{
    track::{Entity, EntityHeader},
    CollectionUid,
};
use aoide_core_api::{track::search::Params, Pagination};
use discro::{Publisher, Ref, Subscriber};
use highway::{HighwayHash, HighwayHasher, Key};

use crate::environment::Handle;

pub mod tasklet;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Context {
    pub collection_uid: Option<CollectionUid>,
    pub params: Params,
}

/// A light-weight tag that denotes the [`State`] variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FetchStateTag {
    #[default]
    Initial,
    Ready,
    Pending,
    Failed,
}

impl FetchStateTag {
    /// Check whether the state is stable.
    #[must_use]
    pub const fn is_idle(&self) -> bool {
        match self {
            Self::Initial | Self::Ready | Self::Failed => true,
            Self::Pending => false,
        }
    }

    #[must_use]
    pub const fn should_prefetch(&self) -> bool {
        matches!(self, Self::Initial)
    }
}

#[derive(Debug, Clone)]
pub struct FetchedEntity {
    pub offset_hash: u64,
    pub entity: Entity,
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
    },
    Failed {
        fetched_entities_before: Option<Vec<FetchedEntity>>,
        _err_msg: String,
    },
}

impl FetchState {
    #[must_use]
    const fn state_tag(&self) -> FetchStateTag {
        match self {
            Self::Initial => FetchStateTag::Initial,
            Self::Ready { .. } => FetchStateTag::Ready,
            Self::Failed { .. } => FetchStateTag::Failed,
            Self::Pending { .. } => FetchStateTag::Pending,
        }
    }

    #[must_use]
    const fn is_idle(&self) -> bool {
        self.state_tag().is_idle()
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
    const fn should_prefetch(&self) -> bool {
        self.state_tag().should_prefetch()
    }

    #[must_use]
    const fn can_fetch_more(&self) -> Option<bool> {
        match self {
            Self::Initial => Some(true),                                 // always
            Self::Pending { .. } | Self::Failed { .. } => None,          // undefined
            Self::Ready { can_fetch_more, .. } => Some(*can_fetch_more), // maybe
        }
    }

    fn reset(&mut self) -> bool {
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
        log::debug!(
            "Fetching succeeded with {num_fetched_entities} newly fetched entities",
            num_fetched_entities = fetched_entities.len()
        );
        let Self::Pending {
            fetched_entities_before,
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
        let num_fetched_entities = fetched_entities.len();
        *self = Self::Ready {
            fetched_entities,
            can_fetch_more,
        };
        log::debug!("Caching {num_fetched_entities} fetched entities");
        true
    }

    #[allow(clippy::needless_pass_by_value)]
    fn fetch_more_failed(&mut self, err: anyhow::Error) -> bool {
        log::warn!("Fetching failed: {err}");
        let Self::Pending {
            fetched_entities_before,
        } = self
        else {
            // No effect
            log::error!("Not pending when fetching failed");
            return false;
        };
        let fetched_entities_before = std::mem::take(fetched_entities_before);
        *self = Self::Failed {
            fetched_entities_before,
            _err_msg: err.to_string(),
        };
        true
    }
}

#[derive(Debug)]
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
    pub const fn fetch_state_tag(&self) -> FetchStateTag {
        self.fetch.state_tag()
    }

    #[must_use]
    pub const fn is_idle(&self) -> bool {
        self.fetch.is_idle()
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

    #[must_use]
    pub fn fetched_entities(&self) -> Option<&[FetchedEntity]> {
        self.fetch.fetched_entities()
    }

    pub fn reset(&mut self) -> bool {
        // Cloning the default params once for pre-creating the target state
        // is required to avoid redundant code for determining in advance if
        // the state would actually change or not.
        let reset = Self::new(self.default_params.clone());
        if self.context == reset.context && self.fetch.state_tag() == reset.fetch.state_tag() {
            // No effect.
            log::debug!("State doesn't need to be reset");
            return false;
        }
        *self = reset;
        debug_assert!(!self.should_prefetch());
        log::debug!("State has been reset");
        true
    }

    /// Update the collection UID
    ///
    /// Consumed the argument when returning `true`.
    pub fn update_collection_uid(&mut self, collection_uid: &mut Option<CollectionUid>) -> bool {
        if collection_uid.as_ref() == self.context.collection_uid.as_ref() {
            // No effect.
            log::debug!("Collection UID unchanged: {collection_uid:?}");
            return false;
        }
        self.context.collection_uid = collection_uid.take();
        self.fetch.reset();
        log::debug!(
            "Collection UID updated: {collection_uid:?}",
            collection_uid = self.context.collection_uid
        );
        true
    }

    /// Update the search parameters
    ///
    /// Consumed the argument when returning `true`.
    pub fn update_params(&mut self, params: &mut Params) -> bool {
        if params == &self.context.params {
            // No effect.
            log::debug!("Params unchanged: {params:?}");
            return false;
        }
        self.context.params = std::mem::take(params);
        self.fetch.reset();
        log::debug!("Params updated: {params:?}", params = self.context.params);
        true
    }

    pub fn try_fetch_more(&mut self) -> bool {
        debug_assert_eq!(Some(true), self.can_fetch_more());
        self.fetch.try_fetch_more()
    }

    pub fn fetch_more_succeeded(&mut self, succeeded: FetchMoreSucceeded) -> bool {
        let FetchMoreSucceeded {
            context,
            offset,
            offset_hash,
            fetched,
            can_fetch_more,
        } = succeeded;
        if context != self.context {
            log::warn!(
                "Mismatching context after fetching succeeded: expected = {expected_context:?}, \
                 actual = {context:?}",
                expected_context = self.context
            );
            // No effect.
            return false;
        }
        self.fetch
            .fetch_more_succeeded(offset, offset_hash, fetched, can_fetch_more)
    }

    pub fn fetch_more_failed(&mut self, err: anyhow::Error) -> bool {
        self.fetch.fetch_more_failed(err)
    }

    pub fn reset_fetched(&mut self) -> bool {
        self.fetch.reset()
    }
}

#[derive(Debug)]
pub struct FetchMoreSucceeded {
    context: Context,
    offset: usize,
    offset_hash: u64,
    fetched: Vec<Entity>,
    can_fetch_more: bool,
}

#[allow(clippy::cast_possible_truncation)]
pub async fn fetch_more(
    handle: &Handle,
    context: Context,
    offset_hash: u64,
    pagination: Pagination,
) -> anyhow::Result<FetchMoreSucceeded> {
    let Context {
        collection_uid,
        params,
    } = &context;
    let collection_uid = if let Some(collection_uid) = collection_uid {
        collection_uid.clone()
    } else {
        anyhow::bail!("cannot fetch more without collection");
    };
    let params = params.clone();
    let offset = pagination.offset.unwrap_or(0) as usize;
    let limit = pagination.limit;
    let fetched = search(handle.db_gatekeeper(), collection_uid, params, pagination).await?;
    let can_fetch_more = if let Some(limit) = limit {
        limit <= fetched.len() as u64
    } else {
        false
    };
    Ok(FetchMoreSucceeded {
        context,
        offset,
        offset_hash,
        fetched,
        can_fetch_more,
    })
}

#[derive(Debug)]
struct FetchMore {
    context: Context,
    offset_hash: u64,
    pagination: Pagination,
}

fn on_fetch_more(state: &mut State, fetch_limit: Option<NonZeroUsize>) -> Option<FetchMore> {
    if state.can_fetch_more() != Some(true) || !state.try_fetch_more() {
        // Not modified.
        return None;
    }
    let context = state.context().clone();
    let offset_hash = last_offset_hash_of_fetched_entities(state.fetched_entities());
    let offset = state.fetched_entities().map(|slice| slice.len() as u64);
    let limit = fetch_limit.map(|limit| limit.get() as u64);
    let pagination = Pagination { limit, offset };
    Some(FetchMore {
        context,
        offset_hash,
        pagination,
    })
}

/// Manages the mutable, observable state
#[derive(Debug)]
pub struct ObservableState {
    state_pub: Publisher<State>,
}

impl ObservableState {
    #[must_use]
    pub fn new(initial_state: State) -> Self {
        let state_pub = Publisher::new(initial_state);
        Self { state_pub }
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, State> {
        self.state_pub.read()
    }

    #[must_use]
    pub fn subscribe(&self) -> Subscriber<State> {
        self.state_pub.subscribe()
    }

    #[allow(clippy::must_use_candidate)]
    pub fn modify(&self, modify_state: impl FnOnce(&mut State) -> bool) -> bool {
        self.state_pub.modify(modify_state)
    }

    #[allow(clippy::must_use_candidate)]
    pub fn reset(&self) -> bool {
        self.modify(State::reset)
    }

    pub fn update_collection_uid(&self, collection_uid: &mut Option<CollectionUid>) -> bool {
        self.modify(|state| state.update_collection_uid(collection_uid))
    }

    pub fn update_params(&self, params: &mut Params) -> bool {
        self.modify(|state| state.update_params(params))
    }

    fn on_fetch_more(&self, fetch_limit: Option<NonZeroUsize>) -> Option<FetchMore> {
        let mut maybe_fetch_more = None;
        self.modify(|state| {
            let Some(fetch_more) = on_fetch_more(state, fetch_limit) else {
                return false;
            };
            maybe_fetch_more = Some(fetch_more);
            true
        });
        maybe_fetch_more
    }

    pub async fn fetch_more(&self, handle: &Handle, fetch_limit: Option<NonZeroUsize>) -> bool {
        let Some(FetchMore {
            context,
            offset_hash,
            pagination,
        }) = self.on_fetch_more(fetch_limit)
        else {
            // No effect.
            return false;
        };
        let res = self::fetch_more(handle, context, offset_hash, pagination).await;
        self.modify(|state| match res {
            Ok(succeeded) => state.fetch_more_succeeded(succeeded),
            Err(err) => state.fetch_more_failed(err),
        })
    }

    #[allow(clippy::must_use_candidate)]
    pub fn reset_fetched(&self) -> bool {
        self.modify(State::reset_fetched)
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
