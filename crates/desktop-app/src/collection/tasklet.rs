// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, sync::Arc};

use discro::{tasklet::OnChanged, Subscriber};

use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::{fs::DirPath, settings};

use super::{NestedMusicDirectoriesStrategy, ObservableState, State, StateTag};

pub fn on_state_tag_changed(
    subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(StateTag) -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    discro::tasklet::capture_changes(
        subscriber,
        |state| state.state_tag(),
        |state_tag, state| *state_tag != state.state_tag(),
        move |state_tag| on_changed(*state_tag),
    )
}

pub fn on_state_tag_changed_async<T>(
    subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(StateTag) -> T + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static
where
    T: Future<Output = OnChanged> + Send + 'static,
{
    discro::tasklet::capture_changes_async(
        subscriber,
        |state| state.state_tag(),
        |state_tag, state| *state_tag != state.state_tag(),
        move |state_tag| on_changed(*state_tag),
    )
}

pub async fn on_settings_changed(
    db_gatekeeper: Arc<Gatekeeper>,
    settings_state: Arc<settings::ObservableState>,
    observable_state: Arc<ObservableState>,
    nested_music_directories_strategy: NestedMusicDirectoriesStrategy,
    mut report_error: impl FnMut(anyhow::Error) + Send + 'static,
) {
    log::debug!("Starting on_settings_changed_update_state");
    let mut settings_state_sub = settings_state.subscribe();
    loop {
        let (music_dir, collection_kind) = {
            let settings_state = settings_state_sub.read_ack();
            let music_dir = settings_state.music_dir.clone();
            let collection_kind = settings_state.collection_kind.clone();
            (music_dir, collection_kind)
        };
        if let Err(err) = observable_state
            .update_music_dir(
                &db_gatekeeper,
                collection_kind.map(Into::into),
                music_dir,
                nested_music_directories_strategy,
            )
            .await
        {
            report_error(err);
            // Reset the music directory in the settings state. This will
            // reset the collection state subsequently.
            settings_state.modify(|settings| settings.update_music_dir(None));
        } else {
            // Get the actual music directory from the collection state
            // and feed it back into the settings state.
            let music_dir = observable_state.read().music_dir().map(DirPath::into_owned);
            settings_state.modify(|settings| settings.update_music_dir(music_dir.as_ref()));
        }
        if settings_state_sub.changed().await.is_err() {
            // Publisher disappeared
            break;
        }
    }
    log::debug!("Stopping on_settings_changed_update_state");
}
