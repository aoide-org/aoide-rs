// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fs::File, path::Path, sync::atomic::AtomicBool};

use anyhow::anyhow;

use aoide_core::util::clock::OffsetDateTimeMs;
use aoide_core::{
    CollectionUid, EntityRevision, TrackEntity,
    media::content::{ContentPath, ContentRevision},
    track::EntityHeader,
};
use aoide_core_api::media::SyncMode;
use aoide_media_file::io::{
    export::ExportTrackConfig,
    import::{ImportTrack, ImportTrackConfig},
};
use aoide_repo::{
    CollectionId, RepoError,
    collection::EntityRepo as CollectionRepo,
    track::{
        CollectionRepo as TrackCollectionRepo, EntityRepo as TrackEntityRepo, ReplaceMode,
        ReplaceParams,
    },
};

use crate::{
    Error, Result,
    collection::vfs::RepoContext,
    media::{
        ImportTrackFromFileOutcome, SyncModeParams, export_track_metadata_into_file,
        import_track_from_file_path,
    },
};

use super::{ValidatedInput, import_and_replace, replace, validate_input};

#[derive(Debug, Clone)]
pub struct ExportParams {
    /// Export the updated metadata into the file.
    pub export_config: ExportTrackConfig,

    /// Re-import the file after exporting the metadata to mark
    /// the track as synchronized.
    pub import_config: ImportTrackConfig,
}

#[derive(Debug, Clone, Default)]
pub struct Params {
    /// Write the updated metadata into the file (optional).
    ///
    /// Otherwise the track is only updated in the database and
    /// will become unsynchronized.
    pub export: Option<ExportParams>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conflict {
    /// The track has been modified in the database in the meantime.
    EntityRevision {
        expected: EntityRevision,
        actual: EntityRevision,
    },

    /// The file has been modified on disk since it has been imported
    /// for the last time.
    ContentRevision {
        expected: Option<ContentRevision>,
        actual: Option<ContentRevision>,
    },
}

#[derive(Debug)]
#[expect(clippy::large_enum_variant)]
pub enum Outcome {
    /// The track has been updated in the database and the file
    /// has been written if requested.
    ///
    /// Also returned if the track itself has not been modified.
    Updated { entity: TrackEntity, exported: bool },

    /// The update has been rejected and nothing has been modified.
    Rejected(Conflict),
}

/// Update a single track that has been edited.
///
/// The update is rejected if either the track entity has been modified
/// since it has been loaded (`expected_entity_hdr`) or if the file has
/// been modified on disk since it has been imported for the last time.
///
/// The media source of the track must not be changed. All other properties
/// of the track are replaced.
///
/// After writing the file, it is re-imported to update the content revision
/// and to mark the track as synchronized. Metadata that could not be written
/// into the file is not preserved.
pub fn update_one<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    expected_entity_hdr: &EntityHeader,
    track: ValidatedInput,
    params: &Params,
) -> Result<Outcome>
where
    Repo: CollectionRepo + TrackCollectionRepo + TrackEntityRepo,
{
    let ValidatedInput(mut track) = track;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, None)?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(Error::Other(anyhow!(
            "unsupported path kind: {path_kind:?}"
        )));
    };
    let content_path_resolver = resolver.canonical_resolver();
    let collection_id = collection_ctx.record_id;

    let current_entity = load_collected_entity(
        repo,
        collection_id,
        expected_entity_hdr,
        &track.media_source.content.link.path,
    )?;
    let current_media_source = &current_entity.body.track.media_source;
    let file_path = content_path_resolver.build_file_path(&current_media_source.content.link.path);
    if let Some(conflict) = check_for_conflict(expected_entity_hdr, &current_entity, &file_path)? {
        log::info!(
            "Rejecting update of track {uid}: {conflict:?}",
            uid = expected_entity_hdr.uid,
        );
        return Ok(Outcome::Rejected(conflict));
    }

    // Prevent unintended modifications of the media source, e.g. the
    // content revision that is managed internally.
    track.media_source.clone_from(current_media_source);
    let content_path = track.media_source.content.link.path.clone();
    let replace_params = ReplaceParams {
        mode: ReplaceMode::UpdateOnly,
        preserve_collected_at: true,
        update_last_synchronized_rev: false,
    };
    let outcome = replace::replace_collected_track_by_media_source_content_path(
        repo,
        collection_id,
        replace_params,
        ValidatedInput(track),
    )?;
    let entity = match outcome {
        replace::Outcome::Unchanged(_, entity) | replace::Outcome::Updated(_, entity) => entity,
        replace::Outcome::NotCreated(_) | replace::Outcome::NotUpdated(_, _) => {
            // Unreachable for an existing track in mode `UpdateOnly`.
            return Err(Error::Other(anyhow!("track {content_path} not updated")));
        }
        replace::Outcome::Created(_, _) => unreachable!(),
    };

    let Some(ExportParams {
        export_config,
        import_config,
    }) = params.export.as_ref()
    else {
        return Ok(Outcome::Updated {
            entity,
            exported: false,
        });
    };

    let mut exported_track = entity.body.track.clone();
    export_track_metadata_into_file(
        &mut exported_track,
        content_path_resolver,
        export_config,
        None,
    )?;
    log::debug!(
        "Exported metadata of track {uid} into file {file_path}",
        uid = entity.hdr.uid,
        file_path = file_path.display(),
    );

    let import_params = import_and_replace::Params {
        sync_mode: SyncMode::Always,
        import_config: import_config.clone(),
        replace_mode: ReplaceMode::UpdateOnly,
    };
    let import_outcome = import_and_replace::import_and_replace_many_by_local_file_path(
        repo,
        collection_uid,
        &import_params,
        std::iter::once(content_path.clone()),
        Some(1),
        &std::convert::identity,
        &AtomicBool::new(false),
    )?;
    let summary = &import_outcome.summary;
    if !summary.failed.is_empty() || !summary.skipped.is_empty() {
        return Err(Error::Other(anyhow!(
            "failed to re-import track {content_path} after exporting metadata"
        )));
    }
    let (_, entity) = repo.load_track_entity_by_uid(&entity.hdr.uid)?;
    Ok(Outcome::Updated {
        entity,
        exported: true,
    })
}

/// Write the current metadata of a track into the file.
///
/// Intended for synchronizing tracks that have only been modified
/// in the database.
///
/// Same as [`update_one()`] with the current, unmodified track and
/// writing the file, including all checks for conflicts.
pub fn export_one<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    expected_entity_hdr: &EntityHeader,
    export: ExportParams,
) -> Result<Outcome>
where
    Repo: CollectionRepo + TrackCollectionRepo + TrackEntityRepo,
{
    let (_, entity) = repo.load_track_entity_by_uid(&expected_entity_hdr.uid)?;
    // Unchanged invalidities have already been reported when importing the track.
    let (track, _invalidities) = validate_input(entity.raw.body.track)?;
    let params = Params {
        export: Some(export),
    };
    update_one(repo, collection_uid, expected_entity_hdr, track, &params)
}

#[derive(Debug)]
#[expect(clippy::large_enum_variant)]
pub enum RevertOutcome {
    /// The metadata of the track is identical to the file and the
    /// track is synchronized.
    Reverted(TrackEntity),

    /// The track has been modified in the database in the meantime
    /// and nothing has been modified.
    Rejected(Conflict),
}

/// Discard all modifications of a track by restoring it from the file.
///
/// The track is rebuilt from the file instead of importing the file into the
/// current track. Otherwise modifications of properties that are missing in
/// the file would survive, e.g. all of them if the file has no tags at all.
///
/// The current file is authoritative, even if it has been modified on disk
/// since it has been imported for the last time. It is never written.
///
/// Cue points and artwork are preserved if the file doesn't provide them.
/// Otherwise reverting would lose data that could not be edited.
pub fn revert_one<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    expected_entity_hdr: &EntityHeader,
    import_config: &ImportTrackConfig,
) -> Result<RevertOutcome>
where
    Repo: CollectionRepo + TrackCollectionRepo + TrackEntityRepo,
{
    let collection_ctx = RepoContext::resolve(repo, collection_uid, None)?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(Error::Other(anyhow!(
            "unsupported path kind: {path_kind:?}"
        )));
    };
    let content_path_resolver = resolver.canonical_resolver();
    let collection_id = collection_ctx.record_id;

    let (_, current_entity) = repo.load_track_entity_by_uid(&expected_entity_hdr.uid)?;
    let content_path = current_entity
        .body
        .track
        .media_source
        .content
        .link
        .path
        .clone_owned();
    let current_entity =
        load_collected_entity(repo, collection_id, expected_entity_hdr, &content_path)?;
    if current_entity.hdr.rev != expected_entity_hdr.rev {
        return Ok(RevertOutcome::Rejected(Conflict::EntityRevision {
            expected: expected_entity_hdr.rev,
            actual: current_entity.hdr.rev,
        }));
    }

    let current_track = &current_entity.body.track;
    let collected_at: OffsetDateTimeMs = current_track.media_source.collected_at;
    let imported = import_track_from_file_path(
        ImportTrack::NewTrack { collected_at },
        content_path_resolver,
        &content_path,
        &SyncModeParams::Always,
        import_config,
    )?;
    let ImportTrackFromFileOutcome::Imported { mut track, issues } = imported else {
        return Err(Error::Other(anyhow!(
            "failed to import {content_path} from file: {imported:?}"
        )));
    };
    if !issues.is_empty() {
        log::warn!("Issues while reverting track {content_path}: {issues:?}");
    }
    if track.cues.is_empty() {
        track.cues.clone_from(&current_track.cues);
    }
    if track.media_source.artwork.is_none() {
        track
            .media_source
            .artwork
            .clone_from(&current_track.media_source.artwork);
    }

    // Imported tracks are often invalid to some extent and reverting must not be prevented.
    let (validated_input, _invalidities) = validate_input(track)?;
    let replace_params = ReplaceParams {
        mode: ReplaceMode::UpdateOnly,
        preserve_collected_at: true,
        update_last_synchronized_rev: true,
    };
    let outcome = replace::replace_collected_track_by_media_source_content_path(
        repo,
        collection_id,
        replace_params,
        validated_input,
    )?;
    match outcome {
        replace::Outcome::Unchanged(_, entity) | replace::Outcome::Updated(_, entity) => {
            Ok(RevertOutcome::Reverted(entity))
        }
        replace::Outcome::NotCreated(_) | replace::Outcome::NotUpdated(_, _) => {
            // Unreachable for an existing track in mode `UpdateOnly`.
            Err(Error::Other(anyhow!("track {content_path} not reverted")))
        }
        replace::Outcome::Created(_, _) => unreachable!(),
    }
}

/// Load the current entity of a track in the collection.
///
/// The content path of the media source must not be changed.
fn load_collected_entity<Repo>(
    repo: &mut Repo,
    collection_id: CollectionId,
    expected_entity_hdr: &EntityHeader,
    content_path: &ContentPath<'_>,
) -> Result<TrackEntity>
where
    Repo: TrackCollectionRepo + TrackEntityRepo,
{
    let (_, current_entity) = repo.load_track_entity_by_uid(&expected_entity_hdr.uid)?;
    let current_content_path = &current_entity.body.track.media_source.content.link.path;
    // The track must belong to the collection.
    let (_, _, collected_entity_hdr) = repo
        .resolve_track_entity_header_by_media_source_content_path(
            collection_id,
            current_content_path,
        )?;
    if collected_entity_hdr.uid != expected_entity_hdr.uid {
        return Err(RepoError::NotFound.into());
    }
    if content_path != current_content_path {
        return Err(Error::Other(anyhow!(
            "media source content path must not be changed: expected \"{current_content_path}\", \
             actual \"{content_path}\""
        )));
    }
    Ok(current_entity)
}

fn check_for_conflict(
    expected_entity_hdr: &EntityHeader,
    current_entity: &TrackEntity,
    file_path: &Path,
) -> Result<Option<Conflict>> {
    if current_entity.hdr.rev != expected_entity_hdr.rev {
        return Ok(Some(Conflict::EntityRevision {
            expected: expected_entity_hdr.rev,
            actual: current_entity.hdr.rev,
        }));
    }
    let expected_content_rev = current_entity.body.track.media_source.content.link.rev;
    let actual_content_rev = ContentRevision::try_from_file(&File::open(file_path)?)?;
    // Without a content revision, unsynchronized modifications could not be detected.
    if expected_content_rev.is_none() || actual_content_rev != expected_content_rev {
        return Ok(Some(Conflict::ContentRevision {
            expected: expected_content_rev,
            actual: actual_content_rev,
        }));
    }
    Ok(None)
}
