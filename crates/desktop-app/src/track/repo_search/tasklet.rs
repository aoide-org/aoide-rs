// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::future::Future;

use discro::{tasklet::OnChanged, Subscriber};

use super::{FetchStatus, State};

pub fn on_should_fetch_more_trigger(
    subscriber: Subscriber<State>,
    mut on_trigger: impl FnMut() -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        |_| (),
        |(), state| {
            // Keep nagging the listener until should_fetch_more() returns false
            state.should_fetch_more()
        },
        move |_| on_trigger(),
    )
}

pub fn on_fetch_status_changed(
    subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(FetchStatus) -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        |state| state.fetch_status(),
        |fetch_status, state| *fetch_status != state.fetch_status(),
        move |fetch_status| on_changed(*fetch_status),
    )
}
