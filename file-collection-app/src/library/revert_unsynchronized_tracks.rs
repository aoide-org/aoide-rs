// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Discard all modifications of unsynchronized tracks by restoring them from their files.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::anyhow;
use aoide::{backend_embedded::media::predefined_import_track_config, desktop_app::ActionEffect};

// Re-exports
pub use aoide::backend_embedded::batch::revert_unsynchronized_tracks::{
    Completion, Outcome, Progress,
};

use super::{Event, EventEmitter, Library, State};

impl Library {
    /// Spawn a task for restoring all unsynchronized tracks from their files.
    ///
    /// Emits [`Event::RevertUnsynchronizedTracksProgress`] while running and
    /// [`Event::RevertUnsynchronizedTracksFinished`] when finished.
    pub fn revert_unsynchronized_tracks<E>(
        &mut self,
        rt: &tokio::runtime::Handle,
        event_emitter: &E,
    ) -> (ActionEffect, anyhow::Result<()>)
    where
        E: EventEmitter + Clone + 'static,
    {
        if !matches!(self.state, State::Idle) {
            let rejected = anyhow!("already/still pending");
            return (ActionEffect::Unchanged, Err(rejected));
        }
        let Some(collection_uid) = self.shared_state.collection.read().entity_uid().cloned() else {
            return (ActionEffect::Unchanged, Err(anyhow!("no collection")));
        };
        let abort_flag = Arc::new(AtomicBool::new(false));
        rt.spawn({
            let env = Arc::clone(self.env());
            let event_emitter = event_emitter.clone();
            let abort_flag = Arc::clone(&abort_flag);
            async move {
                let report_progress_fn = {
                    let event_emitter = event_emitter.clone();
                    move |progress| {
                        drop(
                            event_emitter
                                .emit_event(Event::RevertUnsynchronizedTracksProgress(progress)),
                        );
                    }
                };
                let result = aoide::backend_embedded::batch::revert_unsynchronized_tracks::revert_unsynchronized_tracks(
                    env.db_gatekeeper(),
                    collection_uid,
                    predefined_import_track_config(),
                    report_progress_fn,
                    abort_flag,
                )
                .await
                .map_err(Into::into);
                drop(event_emitter.emit_event(Event::RevertUnsynchronizedTracksFinished(
                    Box::new(result),
                )));
            }
        });
        self.state = State::RevertingUnsynchronizedTracks { abort_flag };
        (ActionEffect::Changed, Ok(()))
    }

    /// Abort the pending task after the current track has been restored.
    pub fn revert_unsynchronized_tracks_abort(&self) -> ActionEffect {
        let State::RevertingUnsynchronizedTracks { abort_flag } = &self.state else {
            log::info!("Not reverting unsynchronized tracks");
            return ActionEffect::Unchanged;
        };
        log::info!("Aborting revert of unsynchronized tracks");
        abort_flag.store(true, Ordering::Relaxed);
        ActionEffect::MaybeChanged
    }

    pub(crate) fn revert_unsynchronized_tracks_finished(&mut self) -> ActionEffect {
        if !matches!(self.state, State::RevertingUnsynchronizedTracks { .. }) {
            return ActionEffect::Unchanged;
        }
        self.state = State::Idle;
        ActionEffect::Changed
    }
}
