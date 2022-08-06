// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, sync::Arc};

use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use discro::{tasklet::OnChanged, Subscriber};

use crate::collection;

use super::{FetchStateTag, ObservableState, State};

pub fn on_should_prefetch_trigger(
    subscriber: Subscriber<State>,
    mut on_trigger: impl FnMut() -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        |_| (),
        |(), state| {
            // Keep nagging the listener until should_prefetch() returns false
            state.should_prefetch()
        },
        move |_| on_trigger(),
    )
}

pub fn on_should_prefetch_trigger_async<T>(
    subscriber: Subscriber<State>,
    mut on_trigger: impl FnMut() -> T + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static
where
    T: Future<Output = OnChanged> + Send + 'static,
{
    discro::tasklet::capture_changes_async(
        subscriber,
        |_| (),
        |(), state| {
            // Keep nagging the listener until should_prefetch() returns false
            state.should_prefetch()
        },
        move |()| on_trigger(),
    )
}

pub async fn on_should_prefetch(
    db_gatekeeper: Arc<Gatekeeper>,
    observable_state: Arc<ObservableState>,
    prefetch_limit: Option<usize>,
) {
    log::debug!("Starting on_should_prefetch_prefetch");
    on_should_prefetch_trigger_async(observable_state.subscribe(), {
        move || {
            let db_gatekeeper = Arc::clone(&db_gatekeeper);
            let observable_state = Arc::clone(&observable_state);
            async move {
                let should_prefetch = observable_state.read().should_prefetch();
                if should_prefetch {
                    log::debug!("Prefetching...");
                    observable_state
                        .fetch_more(&*db_gatekeeper, prefetch_limit)
                        .await;
                }
                OnChanged::Continue
            }
        }
    })
    .await;
    log::debug!("Stopping on_should_prefetch_prefetch");
}

pub fn on_fetch_state_tag_changed(
    subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(FetchStateTag) -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        |state| state.fetch_state_tag(),
        |fetch_state_tag, state| *fetch_state_tag != state.fetch_state_tag(),
        move |fetch_state_tag| on_changed(*fetch_state_tag),
    )
}

pub fn on_fetch_state_tag_changed_async<T>(
    subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(FetchStateTag) -> T + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static
where
    T: Future<Output = OnChanged> + Send + 'static,
{
    discro::tasklet::capture_changes_async(
        subscriber,
        |state| state.fetch_state_tag(),
        |fetch_state_tag, state| *fetch_state_tag != state.fetch_state_tag(),
        move |fetch_state_tag| on_changed(*fetch_state_tag),
    )
}

pub async fn on_collection_changed(
    collection_state: Arc<collection::ObservableState>,
    observable_state: Arc<ObservableState>,
) {
    log::debug!("Starting on_collection_changed");
    collection::tasklet::on_state_tag_changed(collection_state.subscribe(), {
        let observable_state = Arc::clone(&observable_state);
        move |_| {
            let mut collection_uid = collection_state.read().entity_uid().map(Clone::clone);
            // Argument is consumed when updating succeeds
            if !observable_state.update_collection_uid(&mut collection_uid) {
                log::debug!("Collection not updated: {collection_uid:?}");
            }
            OnChanged::Continue
        }
    })
    .await;
    log::debug!("Stopping on_collection_changed");
}
