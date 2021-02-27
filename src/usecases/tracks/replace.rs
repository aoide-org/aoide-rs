// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;

use crate::usecases::media::{import_track_from_url, ImportMode, ImportTrackFromFileOutcome};

use aoide_core::util::clock::DateTime;
use aoide_media::{
    fs::local_file_path_from_url,
    io::import::{ImportTrackConfig, ImportTrackFlags},
};
use aoide_repo::{
    collection::{EntityRepo as _, RecordId as CollectionId},
    media::source::{RecordId as MediaSourceId, Repo},
    track::{ReplaceMode, ReplaceOutcome},
};

use media::SynchronizedImportMode;
use std::{
    fs::read_dir,
    sync::atomic::{AtomicBool, Ordering},
};
use url::Url;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Completion {
    Finished,
    Aborted,
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub completion: Completion,
    pub summary: Summary,
    pub media_source_ids: Vec<MediaSourceId>,
}

#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<String>,
    pub not_imported: Vec<String>,
    pub not_created: Vec<Track>,
    pub not_updated: Vec<Track>,
}

fn replace_collected_track_by_media_source_uri(
    summary: &mut Summary,
    db: &RepoConnection<'_>,
    collection_id: CollectionId,
    replace_mode: ReplaceMode,
    preserve_collected_at: bool,
    track: Track,
) -> RepoResult<Option<MediaSourceId>> {
    let media_source_uri = track.media_source.uri.clone();
    let outcome = db
        .replace_collected_track_by_media_source_uri(
            collection_id,
            preserve_collected_at,
            replace_mode,
            track,
        )
        .map_err(|err| {
            log::warn!(
                "Failed to replace track by URI {}: {}",
                media_source_uri,
                err
            );
            err
        })?;
    let media_source_id = match outcome {
        ReplaceOutcome::Created(media_source_id, _, entity) => {
            debug_assert_ne!(ReplaceMode::UpdateOnly, replace_mode);
            log::trace!("Created {}: {:?}", entity.body.media_source.uri, entity.hdr);
            summary.created.push(entity);
            media_source_id
        }
        ReplaceOutcome::Updated(media_source_id, _, entity) => {
            debug_assert_ne!(ReplaceMode::CreateOnly, replace_mode);
            log::trace!("Updated {}: {:?}", entity.body.media_source.uri, entity.hdr);
            summary.updated.push(entity);
            media_source_id
        }
        ReplaceOutcome::Unchanged(media_source_id, _, entity) => {
            log::trace!("Unchanged: {:?}", entity);
            summary.unchanged.push(entity.body.media_source.uri);
            media_source_id
        }
        ReplaceOutcome::NotCreated(track) => {
            debug_assert_eq!(ReplaceMode::UpdateOnly, replace_mode);
            log::trace!("Not created: {:?}", track);
            summary.not_created.push(track);
            return Ok(None);
        }
        ReplaceOutcome::NotUpdated(media_source_id, _, track) => {
            debug_assert_eq!(ReplaceMode::CreateOnly, replace_mode);
            log::trace!("Not updated: {:?}", track);
            summary.not_updated.push(track);
            media_source_id
        }
    };
    Ok(Some(media_source_id))
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
fn import_and_replace_by_media_source_url(
    summary: &mut Summary,
    media_source_ids: &mut Vec<MediaSourceId>,
    db: &RepoConnection<'_>,
    collection_id: CollectionId,
    url: &Url,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    replace_mode: ReplaceMode,
) -> RepoResult<()> {
    let uri = url.to_string();
    let (media_source_id, last_synchronized_at) = db
        .resolve_media_source_id_synchronized_at_by_uri(collection_id, &uri)
        .optional()?
        .map(|(media_source_id, synchronized_at)| (Some(media_source_id), synchronized_at))
        .unwrap_or((None, None));
    match import_track_from_url(
        &url,
        SynchronizedImportMode::new(import_mode, last_synchronized_at),
        import_config,
        import_flags,
        DateTime::now_local(),
    ) {
        Ok(ImportTrackFromFileOutcome::Imported(track)) => {
            debug_assert_eq!(track.media_source.uri, uri);
            if let Some(media_source_id) = replace_collected_track_by_media_source_uri(
                summary,
                &db,
                collection_id,
                replace_mode,
                true,
                track,
            )? {
                media_source_ids.push(media_source_id);
            }
        }
        Ok(ImportTrackFromFileOutcome::SkippedSynchronized(_synchronized_at)) => {
            debug_assert!(media_source_id.is_some());
            debug_assert!(last_synchronized_at.is_some());
            debug_assert!(_synchronized_at <= last_synchronized_at.unwrap());
            summary.unchanged.push(uri);
            media_source_ids.push(media_source_id.unwrap());
        }
        Ok(ImportTrackFromFileOutcome::SkippedDirectory) => {
            // Nothing to do
        }
        Err(err) => {
            log::warn!("Failed to import track from file URL {}: {}", url, err);
            summary.not_imported.push(uri);
        }
    };
    Ok(())
}

pub fn replace_by_media_source_uri(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    replace_mode: ReplaceMode,
    tracks: impl Iterator<Item = Track>,
) -> Result<Summary> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselRepoError, _>(|| {
        let mut summary = Summary::default();
        let collection_id = db.resolve_collection_id(collection_uid)?;
        for track in tracks {
            replace_collected_track_by_media_source_uri(
                &mut summary,
                &db,
                collection_id,
                replace_mode,
                false,
                track,
            )?;
        }
        Ok(summary)
    })
    .map_err(Into::into)
}

const UNKNOWN_FILE_URI_COUNT: usize = 256;

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_media_source_uri(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    replace_mode: ReplaceMode,
    file_uris: impl Iterator<Item = String>,
    file_uri_count: Option<usize>,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselRepoError, _>(|| {
        let mut summary = Summary::default();
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let mut media_source_ids =
            Vec::with_capacity(file_uri_count.unwrap_or(UNKNOWN_FILE_URI_COUNT));
        for file_uri in file_uris {
            if abort_flag.load(Ordering::Relaxed) {
                log::debug!("Aborting import of {}", file_uri);
                return Ok(Outcome {
                    completion: Completion::Aborted,
                    summary,
                    media_source_ids,
                });
            }
            let url: Url = match file_uri.parse() {
                Ok(url) => url,
                Err(err) => {
                    log::warn!("Failed to import track from file URI {}: {}", file_uri, err);
                    summary.not_imported.push(file_uri);
                    continue;
                }
            };
            import_and_replace_by_media_source_url(
                &mut summary,
                &mut media_source_ids,
                &db,
                collection_id,
                &url,
                import_mode,
                import_config,
                import_flags,
                replace_mode,
            )?;
        }
        Ok(Outcome {
            completion: Completion::Finished,
            summary,
            media_source_ids,
        })
    })
    .map_err(Into::into)
}

const EXPECTED_NUMBER_OF_DIR_ENTRIES: usize = 256;

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_media_source_uri_from_directory(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    dir_url: &Url,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    replace_mode: ReplaceMode,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let dir_path = local_file_path_from_url(dir_url)?;
    let dir_entries = read_dir(dir_path)?;
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselRepoError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let mut summary = Summary::default();
        let mut media_source_ids = Vec::with_capacity(EXPECTED_NUMBER_OF_DIR_ENTRIES);
        for dir_entry in dir_entries {
            let dir_entry = match dir_entry {
                Ok(dir_entry) => dir_entry,
                Err(err) => {
                    log::warn!("Failed to access directory entry: {}", err);
                    // Skip entry and keep going
                    continue;
                }
            };
            if abort_flag.load(Ordering::Relaxed) {
                log::debug!(
                    "Aborting import before visiting {}",
                    dir_entry.path().display()
                );
                return Ok(Outcome {
                    completion: Completion::Aborted,
                    summary,
                    media_source_ids,
                });
            }
            let url = match Url::from_file_path(dir_entry.path()) {
                Ok(url) => url,
                Err(()) => {
                    log::warn!(
                        "Failed to obtain URL from file path {}",
                        dir_entry.path().display()
                    );
                    // Skip entry and keep going
                    continue;
                }
            };
            import_and_replace_by_media_source_url(
                &mut summary,
                &mut media_source_ids,
                &db,
                collection_id,
                &url,
                import_mode,
                import_config,
                import_flags,
                replace_mode,
            )?;
        }
        Ok(Outcome {
            completion: Completion::Finished,
            summary,
            media_source_ids,
        })
    })
    .map_err(Into::into)
}
