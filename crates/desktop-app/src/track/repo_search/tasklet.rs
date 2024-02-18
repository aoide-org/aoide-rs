// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    future::Future,
    num::NonZeroUsize,
    sync::{Arc, Weak},
};

use discro::{tasklet::OnChanged, Subscriber};
use unnest::some_or_return_with;

use crate::{collection, WeakHandle};

use super::{ObservableState, State};

pub fn on_should_prefetch_trigger(
    subscriber: Subscriber<State>,
    mut on_trigger: impl FnMut() -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        (),
        |(), state| {
            // Keep nagging the listener until should_prefetch() returns false.
            state.should_prefetch()
        },
        move |()| on_trigger(),
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
        (),
        |(), state| {
            // Keep nagging the listener until should_prefetch() returns false.
            state.should_prefetch()
        },
        move |()| on_trigger(),
    )
}

pub fn on_should_prefetch(
    observable_state: &Arc<ObservableState>,
    handle: WeakHandle,
    prefetch_limit: Option<NonZeroUsize>,
) -> impl Future<Output = ()> + Send + 'static {
    let observable_state_sub = observable_state.subscribe_changed();
    let observable_state = Arc::downgrade(observable_state);
    async move {
        log::debug!("Starting on_should_prefetch");
        on_should_prefetch_trigger_async(observable_state_sub, move || {
            let observable_state = Weak::clone(&observable_state);
            let handle = handle.clone();
            async move {
                let observable_state =
                    some_or_return_with!(observable_state.upgrade(), OnChanged::Abort);
                let handle = some_or_return_with!(handle.upgrade(), OnChanged::Abort);
                let should_prefetch = observable_state.read().should_prefetch();
                if should_prefetch {
                    if let Some((task, continuation)) =
                        observable_state.try_fetch_more_task(&handle, prefetch_limit)
                    {
                        log::debug!("Prefetching");
                        let result = task.await;
                        observable_state.fetch_more_task_joined(result.into(), continuation);
                    }
                }
                OnChanged::Continue
            }
        })
        .await;
        log::debug!("Stopping on_should_prefetch");
    }
}

pub fn on_collection_state_changed(
    collection_state: &Arc<collection::ObservableState>,
    observable_state: Weak<ObservableState>,
) -> impl Future<Output = ()> + Send + 'static {
    let mut collection_state_sub = collection_state.subscribe_changed();
    async move {
        log::debug!("Starting on_collection_state_changed");
        loop {
            {
                let Some(observable_state) = observable_state.upgrade() else {
                    // Observable has been dropped.
                    break;
                };
                let mut collection_uid = {
                    let state = collection_state_sub.read_ack();
                    match &*state {
                        collection::State::Ready { entity, .. } => Some(entity.hdr.uid.clone()),
                        _ => None,
                    }
                };
                observable_state.try_update_collection_uid(&mut collection_uid);
                if observable_state.try_reset_fetched() {
                    log::debug!("Fetched results have been reset");
                }
            }
            if collection_state_sub.changed().await.is_err() {
                // Publisher has been dropped.
                break;
            }
        }
        log::debug!("Stopping on_collection_state_changed");
    }
}
