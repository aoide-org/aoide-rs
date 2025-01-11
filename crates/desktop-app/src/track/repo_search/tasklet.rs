// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    future::Future,
    num::NonZeroUsize,
    sync::{Arc, Weak},
};

use discro::{tasklet::OnChanged, Subscriber};
use unnest::{some_or_break, some_or_return_with};

use crate::{collection, Environment};

use super::{SharedState, State};

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
    rt: tokio::runtime::Handle,
    env: Weak<Environment>,
    this: &Arc<SharedState>,
    prefetch_limit: Option<NonZeroUsize>,
) -> impl Future<Output = ()> + Send + 'static + use<> {
    let subscriber = this.subscribe_changed();
    let this = Arc::downgrade(this);
    async move {
        log::debug!("Starting on_should_prefetch");
        on_should_prefetch_trigger_async(subscriber, move || {
            let rt = rt.clone();
            let env = Weak::clone(&env);
            let this = Weak::clone(&this);

            async move {
                log::debug!("Resuming on_should_prefetch");
                let this = some_or_return_with!(this.upgrade(), OnChanged::Abort);
                let should_prefetch = this.read().should_prefetch();
                if should_prefetch {
                    let env = some_or_return_with!(env.upgrade(), OnChanged::Abort);
                    let effect = this.spawn_fetching_more_task(&rt, &env, prefetch_limit);
                    log::debug!("Prefetching: {effect:?}");
                }
                log::debug!("Suspending on_should_prefetch");
                OnChanged::Continue
            }
        })
        .await;
    }
}

pub fn on_collection_state_changed(
    collection_state: &collection::SharedState,
    this: Weak<SharedState>,
) -> impl Future<Output = ()> + Send + 'static + use<> {
    let mut collection_state_sub = collection_state.subscribe_changed();
    async move {
        log::debug!("Starting on_collection_state_changed");
        loop {
            log::debug!("Suspending on_collection_state_changed");
            if collection_state_sub.changed().await.is_err() {
                // No publisher(s).
                break;
            }
            log::debug!("Resuming on_collection_state_changed");

            let this = some_or_break!(this.upgrade());

            let mut collection_uid = {
                let state = collection_state_sub.read_ack();
                // We are only interested in the collection UID if the collection is ready,
                // even though it is available in other states as well.
                match &*state {
                    collection::State::Ready { entity, .. } => Some(entity.hdr.uid.clone()),
                    _ => None,
                }
            };
            let _ = this.update_collection_uid(&mut collection_uid);
        }
    }
}
