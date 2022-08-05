// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::future::Future;

use discro::{tasklet::OnChanged, Subscriber};

use super::State;

pub fn on_should_fetch_more_trigger(
    subscriber: Subscriber<State>,
    mut on_trigger: impl FnMut() -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        |state| state.should_fetch_more_trigger(),
        |should_fetch_more_trigger, state| {
            *should_fetch_more_trigger != state.should_fetch_more_trigger()
        },
        move |_| on_trigger(),
    )
}

pub fn on_is_idle_changed(
    subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(bool) -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        |state| state.is_idle(),
        |is_idle, state| *is_idle != state.is_idle(),
        move |is_idle| on_changed(*is_idle),
    )
}
