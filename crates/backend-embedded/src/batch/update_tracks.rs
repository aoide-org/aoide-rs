// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Modify multiple tracks at once.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use diesel::Connection as _;

use aoide_core::{Track, TrackEntity, TrackUid};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_usecases::track::{
    update::{Conflict, ExportParams, Outcome as UpdateOutcome, Params as UpdateParams},
    validate_input,
};

use crate::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    #[default]
    Finished,
    Aborted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Progress {
    /// The number of tracks to update.
    pub total: usize,

    /// The number of tracks that have been visited.
    pub finished: usize,
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub completion: Completion,

    /// Tracks that have been modified or whose file has been written.
    pub updated: Vec<TrackEntity>,

    /// Tracks that have not been modified.
    pub unchanged: Vec<TrackUid>,

    /// Tracks that have not been updated due to a conflict, i.e. if the
    /// file has been modified on disk since it has been imported.
    pub skipped: Vec<(TrackUid, Conflict)>,

    /// Tracks that could not be updated.
    pub failed: Vec<(TrackUid, String)>,
}

/// Modify multiple tracks and optionally write their files.
///
/// Each track is loaded, modified, and saved in a separate transaction,
/// i.e. modifications are applied to the current revision of each track.
/// Saving a track is rejected if its file has been modified on disk since
/// it has been imported for the last time, see
/// [`aoide_usecases::track::update::update_one()`].
///
/// Failures of individual tracks are collected in the outcome and don't
/// abort the batch operation.
pub async fn update_tracks<ModifyTrackFn, ReportProgressFn>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    entity_uids: Vec<TrackUid>,
    modify_track_fn: ModifyTrackFn,
    export: Option<ExportParams>,
    mut report_progress_fn: ReportProgressFn,
    abort_flag: Arc<AtomicBool>,
) -> Result<Outcome>
where
    ModifyTrackFn: Fn(&mut Track) + Clone + Send + 'static,
    ReportProgressFn: FnMut(Progress),
{
    let mut progress = Progress {
        total: entity_uids.len(),
        finished: 0,
    };
    report_progress_fn(progress);

    let mut outcome = Outcome::default();
    for entity_uid in entity_uids {
        if abort_flag.load(Ordering::Relaxed) {
            log::info!(
                "Aborting update of tracks after {finished} of {total} track(s)",
                finished = progress.finished,
                total = progress.total
            );
            outcome.completion = Completion::Aborted;
            return Ok(outcome);
        }
        match update_one(
            db_gatekeeper,
            collection_uid.clone(),
            entity_uid.clone(),
            modify_track_fn.clone(),
            export.clone(),
        )
        .await
        {
            Ok(None) => {
                outcome.unchanged.push(entity_uid);
            }
            Ok(Some(UpdateOutcome::Updated { entity, .. })) => {
                outcome.updated.push(entity);
            }
            Ok(Some(UpdateOutcome::Rejected(conflict))) => {
                log::info!("Skipped update of track {entity_uid}: {conflict:?}");
                outcome.skipped.push((entity_uid, conflict));
            }
            Err(err) => {
                log::warn!("Failed to update track {entity_uid}: {err}");
                outcome.failed.push((entity_uid, err.to_string()));
            }
        }
        progress.finished += 1;
        report_progress_fn(progress);
    }
    Ok(outcome)
}

/// Returns `None` if the track is unchanged and the file doesn't need to be written.
async fn update_one<ModifyTrackFn>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    entity_uid: TrackUid,
    modify_track_fn: ModifyTrackFn,
    export: Option<ExportParams>,
) -> Result<Option<UpdateOutcome>>
where
    ModifyTrackFn: Fn(&mut Track) + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                let entity = aoide_usecases_sqlite::track::load::load_one(connection, &entity_uid)?;
                let mut track = entity.body.track.clone();
                modify_track_fn(&mut track);
                if track == entity.body.track && export.is_none() {
                    return Ok(None);
                }
                let (track, _invalidities) =
                    validate_input(track).map_err(aoide_usecases::Error::from)?;
                let params = UpdateParams { export };
                aoide_usecases_sqlite::track::update::update_one(
                    connection,
                    &collection_uid,
                    &entity.hdr,
                    track,
                    &params,
                )
                .map(Some)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
