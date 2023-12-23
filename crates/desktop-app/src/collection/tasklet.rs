// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, sync::Weak};

use discro::{tasklet::OnChanged, Subscriber};
use unnest::{some_or_break, some_or_return};

use super::{NestedMusicDirectoriesStrategy, ObservableState, State, StateTag};
use crate::{fs::DirPath, settings, WeakHandle};

pub fn on_state_tag_changed(
    mut subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(StateTag) -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    let initial_value = subscriber.read_ack().state_tag();
    discro::tasklet::capture_changes(
        subscriber,
        initial_value,
        |state_tag, state| (*state_tag != state.state_tag()).then(|| state.state_tag()),
        move |state_tag| on_changed(*state_tag),
    )
}

pub fn on_state_tag_changed_async<T>(
    mut subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(StateTag) -> T + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static
where
    T: Future<Output = OnChanged> + Send + 'static,
{
    let initial_value = subscriber.read_ack().state_tag();
    discro::tasklet::capture_changes_async(
        subscriber,
        initial_value,
        |state_tag, state| (*state_tag != state.state_tag()).then(|| state.state_tag()),
        move |state_tag| on_changed(*state_tag),
    )
}

pub fn on_settings_changed(
    settings_state: Weak<settings::ObservableState>,
    observable_state: Weak<ObservableState>,
    handle: WeakHandle,
    create_new_entity_if_not_found: bool,
    nested_music_directories_strategy: NestedMusicDirectoriesStrategy,
    mut report_error: impl FnMut(anyhow::Error) + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    let settings_state_sub = settings_state.upgrade().map(|observable| {
        let mut subscriber = observable.subscribe();
        // Enforce initial update.
        subscriber.mark_changed();
        subscriber
    });
    async move {
        let mut settings_state_sub = some_or_return!(settings_state_sub);
        log::debug!("Starting on_settings_changed");
        loop {
            {
                let settings_state = some_or_break!(settings_state.upgrade());
                let observable_state = some_or_break!(observable_state.upgrade());
                let handle = some_or_break!(handle.upgrade());
                let (music_dir, collection_kind) = {
                    let settings_state = settings_state_sub.read_ack();
                    let music_dir = settings_state.music_dir.clone();
                    let collection_kind = settings_state.collection_kind.clone();
                    (music_dir, collection_kind)
                };
                let new_music_dir = if let Err(err) = observable_state
                    .update_music_dir(
                        &handle,
                        collection_kind.map(Into::into),
                        music_dir,
                        create_new_entity_if_not_found,
                        nested_music_directories_strategy,
                    )
                    .await
                {
                    log::debug!("Resetting music directory in settings after error: {err}");
                    report_error(err);
                    // Reset the music directory in the settings state. This will reset
                    // the collection state subsequently to recover from the error.
                    None
                } else {
                    // Get the actual music directory from the collection state and feed it back
                    // into the settings state.
                    let new_music_dir =
                        observable_state.read().music_dir().map(DirPath::into_owned);
                    if let Some(new_music_dir) = &new_music_dir {
                        log::debug!(
                            "Updating music directory in settings: {new_music_dir}",
                            new_music_dir = new_music_dir.display()
                        );
                    } else {
                        log::debug!("Resetting music directory in settings");
                    }
                    new_music_dir
                };
                settings_state.modify(|settings| settings.update_music_dir(new_music_dir.as_ref()));
            }
            if settings_state_sub.changed().await.is_err() {
                // Publisher disappeared
                break;
            }
        }
        log::debug!("Stopping on_settings_changed");
    }
}
