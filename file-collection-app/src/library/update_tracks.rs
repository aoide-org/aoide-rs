// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Edit multiple tracks at once.

use std::sync::Arc;

use aoide::{TrackUid, desktop_app::ActionEffect};

// Re-exports
pub use aoide::backend_embedded::batch::update_tracks::{Completion, Outcome, Progress};

use super::{
    Event, EventEmitter, Library, batch_task::BatchTaskKind, track_edit::export_params,
    ui::MultiTrackEdit,
};

impl Library {
    /// Spawn a task for modifying multiple tracks.
    ///
    /// Emits [`Event::UpdateTracksProgress`] while running and
    /// [`Event::UpdateTracksFinished`] when finished.
    pub fn update_tracks<E>(
        &mut self,
        rt: &tokio::runtime::Handle,
        event_emitter: &E,
        entity_uids: Vec<TrackUid>,
        edit: MultiTrackEdit,
        write_files: bool,
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
            BatchTaskKind::UpdateTracks,
            |env, abort_flag| async move {
                let report_progress_fn = {
                    let event_emitter = event_emitter.clone();
                    move |progress| {
                        drop(event_emitter.emit_event(Event::UpdateTracksProgress(progress)));
                    }
                };
                let edit = Arc::new(edit);
                let modify_track_fn = move |track: &mut aoide::Track| {
                    edit.apply(track);
                };
                let result = aoide::backend_embedded::batch::update_tracks::update_tracks(
                    env.db_gatekeeper(),
                    collection_uid,
                    entity_uids,
                    modify_track_fn,
                    write_files.then(export_params),
                    report_progress_fn,
                    abort_flag,
                )
                .await
                .map_err(Into::into);
                drop(event_emitter.emit_event(Event::UpdateTracksFinished(Box::new(result))));
            },
        )
    }

    /// Abort the pending task after the current track has been updated.
    pub fn update_tracks_abort(&self) -> ActionEffect {
        self.abort_batch_task(BatchTaskKind::UpdateTracks)
    }

    pub(crate) fn update_tracks_finished(&mut self) -> ActionEffect {
        self.batch_task_finished(BatchTaskKind::UpdateTracks)
    }
}
