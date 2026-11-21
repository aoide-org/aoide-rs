// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Discard all modifications of unsynchronized tracks by restoring them from their files.

use aoide::{
    backend_embedded::{
        batch::revert_unsynchronized_tracks::revert_unsynchronized_tracks,
        media::predefined_import_track_config,
    },
    desktop_app::ActionEffect,
};

// Re-exports
pub use aoide::backend_embedded::batch::revert_unsynchronized_tracks::{
    Completion, Outcome, Progress,
};

use super::{Event, EventEmitter, Library, batch_task::BatchTaskKind};

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
        let collection_uid = match self.batch_task_collection_uid() {
            Ok(collection_uid) => collection_uid,
            Err(err) => return (ActionEffect::Unchanged, Err(err)),
        };
        let event_emitter = event_emitter.clone();
        self.spawn_batch_task(
            rt,
            BatchTaskKind::RevertUnsynchronizedTracks,
            |env, abort_flag| async move {
                let report_progress_fn = {
                    let event_emitter = event_emitter.clone();
                    move |progress| {
                        drop(
                            event_emitter
                                .emit_event(Event::RevertUnsynchronizedTracksProgress(progress)),
                        );
                    }
                };
                let result = revert_unsynchronized_tracks(
                    env.db_gatekeeper(),
                    collection_uid,
                    predefined_import_track_config(),
                    report_progress_fn,
                    abort_flag,
                )
                .await
                .map_err(Into::into);
                drop(
                    event_emitter
                        .emit_event(Event::RevertUnsynchronizedTracksFinished(Box::new(result))),
                );
            },
        )
    }

    /// Abort the pending task after the current track has been restored.
    pub fn revert_unsynchronized_tracks_abort(&self) -> ActionEffect {
        self.abort_batch_task(BatchTaskKind::RevertUnsynchronizedTracks)
    }

    pub(crate) fn revert_unsynchronized_tracks_finished(&mut self) -> ActionEffect {
        self.batch_task_finished(BatchTaskKind::RevertUnsynchronizedTracks)
    }
}
