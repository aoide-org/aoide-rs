// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Write the metadata of all unsynchronized tracks into their files.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::anyhow;
use aoide::desktop_app::ActionEffect;

// Re-exports
pub use aoide::backend_embedded::batch::export_unsynchronized_tracks::{
    Completion, Outcome, Progress,
};

use super::{Event, EventEmitter, Library, State, track_edit::export_params};

impl Library {
    /// Spawn a task for writing the metadata of all unsynchronized tracks into their files.
    ///
    /// Emits [`Event::ExportUnsynchronizedTracksProgress`] while running and
    /// [`Event::ExportUnsynchronizedTracksFinished`] when finished.
    pub fn export_unsynchronized_tracks<E>(
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
                                .emit_event(Event::ExportUnsynchronizedTracksProgress(progress)),
                        );
                    }
                };
                let result = aoide::backend_embedded::batch::export_unsynchronized_tracks::export_unsynchronized_tracks(
                    env.db_gatekeeper(),
                    collection_uid,
                    export_params(),
                    report_progress_fn,
                    abort_flag,
                )
                .await
                .map_err(Into::into);
                drop(event_emitter.emit_event(Event::ExportUnsynchronizedTracksFinished(
                    Box::new(result),
                )));
            }
        });
        self.state = State::ExportingUnsynchronizedTracks { abort_flag };
        (ActionEffect::Changed, Ok(()))
    }

    /// Abort the pending task after the current track has been written.
    pub fn export_unsynchronized_tracks_abort(&self) -> ActionEffect {
        let State::ExportingUnsynchronizedTracks { abort_flag } = &self.state else {
            log::info!("Not exporting unsynchronized tracks");
            return ActionEffect::Unchanged;
        };
        log::info!("Aborting export of unsynchronized tracks");
        abort_flag.store(true, Ordering::Relaxed);
        ActionEffect::MaybeChanged
    }

    pub(crate) fn export_unsynchronized_tracks_finished(&mut self) -> ActionEffect {
        if !matches!(self.state, State::ExportingUnsynchronizedTracks { .. }) {
            return ActionEffect::Unchanged;
        }
        self.state = State::Idle;
        ActionEffect::Changed
    }
}
