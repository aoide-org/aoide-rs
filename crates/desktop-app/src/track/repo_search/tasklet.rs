// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, sync::Weak};

use discro::{tasklet::OnChanged, Subscriber};
use unnest::{some_or_return, some_or_return_with};

use crate::{collection, environment::WeakHandle};

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
    observable_state: Weak<ObservableState>,
    handle: WeakHandle,
    prefetch_limit: Option<usize>,
) {
    let observable_state_sub = some_or_return!(observable_state.upgrade()).subscribe();
    log::debug!("Starting on_should_prefetch_prefetch");
    on_should_prefetch_trigger_async(observable_state_sub, move || {
        let observable_state = observable_state.clone();
        let handle = handle.clone();
        async move {
            let observable_state =
                some_or_return_with!(observable_state.upgrade(), OnChanged::Abort);
            let handle = some_or_return_with!(handle.upgrade(), OnChanged::Abort);
            let should_prefetch = observable_state.read().should_prefetch();
            if should_prefetch {
                log::debug!("Prefetching...");
                observable_state.fetch_more(&handle, prefetch_limit).await;
            }
            OnChanged::Continue
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
    collection_state: Weak<collection::ObservableState>,
    observable_state: Weak<ObservableState>,
) {
    let collection_state_sub = some_or_return!(collection_state.upgrade()).subscribe();
    log::debug!("Starting on_collection_changed");
    collection::tasklet::on_state_tag_changed(collection_state_sub, {
        move |_| {
            let collection_state =
                some_or_return_with!(collection_state.upgrade(), OnChanged::Abort);
            let observable_state =
                some_or_return_with!(observable_state.upgrade(), OnChanged::Abort);
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
