// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    future::Future,
    sync::{Arc, Weak},
};

use discro::{tasklet::OnChanged, Subscriber};
use unnest::some_or_break;

use super::{NestedMusicDirectoriesStrategy, ObservableState, State, StateTag};
use crate::{fs::DirPath, settings, Handle, WeakHandle};

fn capture_state_tag_changes(state_tag: &mut StateTag, new_state: &State) -> bool {
    if *state_tag == new_state.state_tag() {
        return false;
    }
    *state_tag = new_state.state_tag();
    true
}

pub fn on_state_tag_changed(
    mut subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(StateTag) -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    let initial_value = subscriber.read_ack().state_tag();
    discro::tasklet::capture_changes(
        subscriber,
        initial_value,
        capture_state_tag_changes,
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
        capture_state_tag_changes,
        move |state_tag| on_changed(*state_tag),
    )
}

async fn update_music_dir(
    settings_state: &settings::ObservableState,
    observable_state: &ObservableState,
    handle: Handle,
    music_dir: Option<DirPath<'static>>,
    collection_kind: Option<String>,
    create_new_entity_if_not_found: bool,
    nested_music_directories_strategy: NestedMusicDirectoriesStrategy,
) {
    if !observable_state
        .update_music_dir(
            &handle,
            collection_kind.map(Into::into),
            music_dir,
            create_new_entity_if_not_found,
            nested_music_directories_strategy,
        )
        .await
    {
        // Unchanged
        return;
    }
    // After succeeded read the actual music directory from the collection state
    // and feed it back into the settings state.
    let new_music_dir = {
        let state = observable_state.read();
        if !state.is_ready() {
            return;
        }
        state.music_dir().map(DirPath::into_owned)
    };
    if let Some(new_music_dir) = &new_music_dir {
        log::info!(
            "Updating music directory in settings: {new_music_dir}",
            new_music_dir = new_music_dir.display()
        );
    } else {
        log::info!("Resetting music directory in settings",);
    }
    settings_state.update_music_dir(new_music_dir.as_ref());
}

pub fn on_settings_state_changed(
    settings_state: &Arc<settings::ObservableState>,
    observable_state: Weak<ObservableState>,
    handle: WeakHandle,
    create_new_entity_if_not_found: bool,
    nested_music_directories_strategy: NestedMusicDirectoriesStrategy,
) -> impl Future<Output = ()> + Send + 'static {
    let mut settings_state_sub = settings_state.subscribe_changed();
    let settings_state = Arc::downgrade(settings_state);
    async move {
        log::debug!("Starting on_settings_state_changed");
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
                update_music_dir(
                    &settings_state,
                    &observable_state,
                    handle,
                    music_dir,
                    collection_kind,
                    create_new_entity_if_not_found,
                    nested_music_directories_strategy,
                )
                .await;
            }
            if settings_state_sub.changed().await.is_err() {
                // Publisher disappeared
                break;
            }
        }
        log::debug!("Stopping on_settings_state_changed");
    }
}
