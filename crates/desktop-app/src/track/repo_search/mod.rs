// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_backend_embedded::track::search;
use aoide_core::{collection::EntityUid as CollectionUid, track::Entity as TrackEntity};
use aoide_core_api::{track::search::Params, Pagination};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use discro::{new_pubsub, Publisher, Ref, Subscriber};

pub mod tasklet;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Context {
    pub collection_uid: Option<CollectionUid>,
    pub params: Params,
}

#[derive(Debug, Default, PartialEq)]
pub enum FetchState {
    #[default]
    Initial,
    Ready {
        fetched: Vec<TrackEntity>,
        can_fetch_more: bool,
    },
    Pending {
        fetched_before: Option<Vec<TrackEntity>>,
    },
    Failed {
        fetched_before: Option<Vec<TrackEntity>>,
        err_msg: String,
    },
}

impl FetchState {
    #[must_use]
    pub fn is_initial(&self) -> bool {
        matches!(self, Self::Initial)
    }

    #[must_use]
    pub fn is_idle(&self) -> bool {
        match self {
            Self::Initial | Self::Ready { .. } | Self::Failed { .. } => true,
            Self::Pending { .. } => false,
        }
    }

    #[must_use]
    pub fn fetched(&self) -> Option<&[TrackEntity]> {
        match self {
            Self::Initial => None,
            Self::Ready { fetched, .. } => Some(fetched),
            Self::Pending { fetched_before, .. } | Self::Failed { fetched_before, .. } => {
                fetched_before.as_deref()
            }
        }
    }

    #[must_use]
    pub fn can_fetch_more(&self) -> Option<bool> {
        match self {
            Self::Initial => Some(true),
            Self::Pending { .. } | Self::Failed { .. } => None,
            Self::Ready { can_fetch_more, .. } => Some(*can_fetch_more),
        }
    }

    pub fn reset(&mut self) -> bool {
        if matches!(self, Self::Initial) {
            return false;
        }
        *self = Self::Initial;
        log::debug!("Reset: {self:?}");
        true
    }

    pub fn fetch_more_succeeded(
        &mut self,
        offset: usize,
        fetched: Vec<TrackEntity>,
        can_fetch_more: bool,
    ) -> bool {
        if let Self::Pending { fetched_before } = self {
            let expected_offset = fetched_before.as_ref().map(Vec::len).unwrap_or(0);
            if offset != expected_offset {
                log::warn!("Mismatching offset after fetching succeeded: expected = {expected_offset}, actual = {offset}");
                return false;
            }
            let fetched = if let Some(mut fetched_before) = fetched_before.take() {
                if fetched_before.is_empty() {
                    fetched
                } else {
                    let mut fetched = fetched;
                    fetched_before.append(&mut fetched);
                    std::mem::take(&mut fetched_before)
                }
            } else {
                fetched
            };
            *self = Self::Ready {
                fetched,
                can_fetch_more,
            };
            log::debug!("Fetching succeeded: {self:?}");
            true
        } else {
            log::error!("Illegal state when fetching succeeded: {self:?}");
            log::warn!(
                "Discarding {num_fetched} fetched entities",
                num_fetched = fetched.len()
            );
            false
        }
    }

    pub fn fetch_more_failed(&mut self, err: anyhow::Error) -> bool {
        if let Self::Pending { fetched_before } = self {
            let fetched_before = std::mem::take(fetched_before);
            *self = Self::Failed {
                fetched_before,
                err_msg: err.to_string(),
            };
            log::debug!("Fetching failed: {self:?}");
            true
        } else {
            log::error!("Illegal state when fetching failed: {self:?}");
            false
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct State {
    context: Context,
    fetch: FetchState,
    initial_fetch_trigger: usize,
}

impl State {
    #[must_use]
    pub fn context(&self) -> &Context {
        &self.context
    }

    #[must_use]
    pub fn initial_fetch_trigger(&self) -> usize {
        self.initial_fetch_trigger
    }

    #[must_use]
    pub fn is_fetch_initial(&self) -> bool {
        self.fetch.is_initial()
    }

    #[must_use]
    pub fn is_fetch_idle(&self) -> bool {
        self.fetch.is_idle()
    }

    #[must_use]
    pub fn can_fetch_more(&self) -> Option<bool> {
        if self.context.collection_uid.is_none() {
            return Some(false);
        }
        self.fetch.can_fetch_more()
    }

    #[must_use]
    pub fn fetched(&self) -> Option<&[TrackEntity]> {
        self.fetch.fetched()
    }

    pub fn reset(&mut self) -> bool {
        let reset = Self::default();
        if *self == reset {
            return false;
        }
        *self = reset;
        true
    }

    /// Update the collection UID
    ///
    /// Consumed the argument when returning `true`.
    pub fn update_collection_uid(&mut self, collection_uid: &mut Option<CollectionUid>) -> bool {
        if collection_uid.as_ref() == self.context.collection_uid.as_ref() {
            return false;
        }
        self.context.collection_uid = collection_uid.take();
        self.fetch.reset();
        if self.context.collection_uid.is_some() {
            self.initial_fetch_trigger = self.initial_fetch_trigger.wrapping_add(1);
        }
        log::debug!("Collection UID updated: {self:?}");
        true
    }

    /// Update the search parameters
    ///
    /// Consumed the argument when returning `true`.
    pub fn update_params(&mut self, params: &mut Params) -> bool {
        if params == &self.context.params {
            return false;
        }
        self.context.params = std::mem::take(params);
        self.fetch.reset();
        self.initial_fetch_trigger = self.initial_fetch_trigger.wrapping_add(1);
        log::debug!("Params updated: {self:?}");
        true
    }

    pub fn fetch_more_succeeded(&mut self, succeeded: FetchMoreSucceeded) -> bool {
        let FetchMoreSucceeded {
            context,
            offset,
            fetched,
            can_fetch_more,
        } = succeeded;
        if context != self.context {
            log::warn!("Mismatching context after fetching succeeded: expected = {expected_context:?}, actual = {context:?}",
            expected_context = self.context);
            return false;
        }
        self.fetch
            .fetch_more_succeeded(offset, fetched, can_fetch_more)
    }

    pub fn fetch_more_failed(&mut self, err: anyhow::Error) -> bool {
        self.fetch.fetch_more_failed(err)
    }
}

#[derive(Debug)]
pub struct FetchMoreSucceeded {
    context: Context,
    offset: usize,
    fetched: Vec<TrackEntity>,
    can_fetch_more: bool,
}

pub async fn fetch_more(
    db_gatekeeper: &Gatekeeper,
    context: Context,
    pagination: Pagination,
) -> anyhow::Result<FetchMoreSucceeded> {
    let Context {
        collection_uid,
        params,
    } = &context;
    let collection_uid = if let Some(collection_uid) = collection_uid {
        collection_uid.clone()
    } else {
        anyhow::bail!("Cannot fetch more without collection");
    };
    let params = params.clone();
    let offset = pagination.offset.unwrap_or(0) as usize;
    let limit = pagination.limit;
    let fetched = search(
        db_gatekeeper,
        collection_uid,
        params,
        pagination,
    )
    .await?;
    let can_fetch_more = if let Some(limit) = limit {
        limit <= fetched.len() as u64
    } else {
        false
    };
    Ok(FetchMoreSucceeded {
        context,
        offset,
        fetched,
        can_fetch_more,
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
        let (state_pub, _) = new_pubsub(initial_state);
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
        self.modify(|state| state.reset())
    }

    pub fn update_collection_uid(&self, collection_uid: &mut Option<CollectionUid>) -> bool {
        self.modify(|state| state.update_collection_uid(collection_uid))
    }

    pub fn update_params(&self, params: &mut Params) -> bool {
        self.modify(|state| state.update_params(params))
    }

    pub async fn fetch_more(&self, db_gatekeeper: &Gatekeeper, fetch_limit: Option<usize>) -> bool {
        let (context, pagination) = {
            let state = self.read();
            let context = state.context().clone();
            let limit = fetch_limit.map(|limit| limit as u64);
            let offset = state.fetched().map(|slice| slice.len() as u64);
            let pagination = Pagination { offset, limit };
            (context, pagination)
        };
        let res = self::fetch_more(db_gatekeeper, context, pagination).await;
        self.modify(|state| match res {
            Ok(succeeded) => state.fetch_more_succeeded(succeeded),
            Err(err) => state.fetch_more_failed(err),
        })
    }
}

impl Default for ObservableState {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
