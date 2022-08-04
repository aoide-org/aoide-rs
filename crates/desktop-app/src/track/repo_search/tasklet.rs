// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::future::Future;

use discro::Subscriber;

use super::State;

pub fn on_initial_fetch_trigger(
    subscriber: Subscriber<State>,
    mut on_trigger: impl FnMut() -> bool + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        |state| state.initial_fetch_trigger(),
        |initial_fetch_trigger, state| *initial_fetch_trigger != state.initial_fetch_trigger(),
        move |_| on_trigger(),
    )
}

pub fn on_fetch_is_idle_changed(
    subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(bool) -> bool + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        |state| state.is_fetch_idle(),
        |is_fetch_idle, state| *is_fetch_idle != state.is_fetch_idle(),
        move |is_fetch_idle| on_changed(*is_fetch_idle),
    )
}
