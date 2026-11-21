// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Modify multiple tracks at once.

use std::sync::{Arc, atomic::AtomicBool};

use diesel::Connection as _;

use aoide_core::{Track, TrackEntity, TrackUid};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_usecases::track::{
    update::{Conflict, ExportParams, Outcome as UpdateOutcome, Params as UpdateParams},
    validate_input,
};

use crate::prelude::*;

pub use super::{Completion, Progress};

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
    report_progress_fn: ReportProgressFn,
    abort_flag: Arc<AtomicBool>,
) -> Result<Outcome>
where
    ModifyTrackFn: Fn(&mut Track) + Clone + Send + 'static,
    ReportProgressFn: FnMut(Progress),
{
    let mut outcome = Outcome::default();
    outcome.completion = super::run_batch(
        "update of tracks",
        entity_uids,
        |entity_uid| {
            let updated = update_one(
                db_gatekeeper,
                collection_uid.clone(),
                entity_uid.clone(),
                modify_track_fn.clone(),
                export.clone(),
            );
            async move { (entity_uid, updated.await) }
        },
        |(entity_uid, result)| match result {
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
        },
        report_progress_fn,
        &abort_flag,
    )
    .await;
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
        .spawn_blocking_write_task_flattened(move |mut pooled_connection| {
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
}
