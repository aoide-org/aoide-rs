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

use std::{io::BufReader, sync::atomic::AtomicBool};

use super::*;

use aoide_core::{entity::EntityUid, track::Track, util::clock::DateTime};

use aoide_media::{
    fmt::{flac, mp3, mp4, ogg},
    fs::{dir_digest, open_local_file_url_for_reading},
    io::import::*,
    util::guess_mime_from_url,
};

use aoide_repo::{
    collection::EntityRepo as _,
    media::{
        dir_tracker::{Repo as _, UpdateOutcome},
        source::Repo as _,
    },
};

use url::Url;

///////////////////////////////////////////////////////////////////////

pub use aoide_repo::media::dir_tracker::TrackingStatusAggregated;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DirScanSummary {
    pub current: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DirScanStatus {
    Finished,
    Aborted,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DirScanOutcome {
    pub status: DirScanStatus,
    pub summary: DirScanSummary,
}

pub fn digest_directories_recursively(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    root_dir_url: &Url,
    max_depth: Option<usize>,
    abort_flag: &AtomicBool,
) -> Result<DirScanOutcome> {
    let db = RepoConnection::new(connection);
    if root_dir_url.scheme() != "file" {
        return Err(Error::Media(
            anyhow::format_err!("Unsupported URL scheme '{}'", root_dir_url.scheme()).into(),
        ));
    }
    if !root_dir_url.as_str().ends_with('/') {
        return Err(Error::Media(
            anyhow::format_err!("URL path does not end with a trailing slash").into(),
        ));
    }
    let root_path = match root_dir_url.to_file_path() {
        Ok(file_path) => file_path,
        Err(()) => {
            return Err(Error::Media(
                anyhow::format_err!("URL is not a file path '{}'", root_dir_url).into(),
            ));
        }
    };
    if !root_path.is_absolute() {
        return Err(Error::Media(
            anyhow::format_err!("Root file path is not absolute: {}", root_path.display()).into(),
        ));
    }
    Ok(db.transaction::<_, DieselRepoError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let outdated_count = db.media_dir_tracker_mark_entries_outdated(
            DateTime::now_utc(),
            collection_id,
            root_dir_url.as_str(),
        )?;
        log::debug!(
            "Marked {} current cache entries as outdated",
            outdated_count
        );
        let mut summary = DirScanSummary::default();
        let status = dir_digest::digest_directories_recursively::<_, anyhow::Error, _, _, _>(
            &root_path,
            max_depth,
            abort_flag,
            blake3::Hasher::new,
            |path, digest| {
                debug_assert!(path.is_relative());
                let full_path = root_path.join(&path);
                debug_assert!(full_path.is_absolute());
                let url = Url::from_directory_path(&full_path).expect("URL");
                debug_assert!(url.as_str().starts_with(root_dir_url.as_str()));
                match db
                    .media_dir_tracker_update_entry_digest(
                        DateTime::now_utc(),
                        collection_id,
                        url.as_str(),
                        &digest.into(),
                    )
                    .map_err(anyhow::Error::from)?
                {
                    UpdateOutcome::Current => {
                        summary.current += 1;
                    }
                    UpdateOutcome::Inserted => {
                        log::debug!("Found added directory: {}", full_path.display());
                        summary.added += 1;
                    }
                    UpdateOutcome::Updated => {
                        log::debug!("Found modified directory: {}", full_path.display());
                        summary.modified += 1;
                    }
                    UpdateOutcome::Skipped => {
                        log::debug!("Skipped directory: {}", full_path.display());
                        summary.skipped += 1;
                    }
                }
                Ok(dir_digest::AfterDirFinished::Continue)
            },
            |progress| {
                log::trace!("{:?}", progress);
            },
        )
        .map_err(anyhow::Error::from)
        .map_err(RepoError::from)
        .and_then(|outcome| {
            let dir_digest::Outcome {
                status,
                progress: _,
            } = outcome;
            match status {
                dir_digest::FinalStatus::Finished => {
                    // Mark all remaining entries that are unreachable and
                    // have not been visited as orphaned.
                    summary.orphaned = db.media_dir_tracker_mark_entries_orphaned(
                        DateTime::now_utc(),
                        collection_id,
                        root_dir_url.as_str(),
                    )?;
                    debug_assert!(summary.orphaned <= outdated_count);
                    Ok(DirScanStatus::Finished)
                }
                dir_digest::FinalStatus::Aborted => {
                    // All partial results up to now can safely be committed.
                    Ok(DirScanStatus::Aborted)
                }
            }
        })?;
        Ok(DirScanOutcome { status, summary })
    })?)
}

pub fn digest_directories_aggregate_status(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    root_dir_url: &Url,
) -> Result<TrackingStatusAggregated> {
    let db = RepoConnection::new(connection);
    Ok(db.transaction::<_, DieselRepoError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        Ok(db
            .media_dir_tracker_update_load_aggregate_status(collection_id, root_dir_url.as_str())?)
    })?)
}

pub fn import_track_from_url(
    url: &Url,
    config: &ImportTrackConfig,
    flags: ImportTrackFlags,
) -> Result<Track> {
    let file = open_local_file_url_for_reading(url)?;
    let file_metadata = file.metadata().map_err(MediaError::from)?;
    let mime = guess_mime_from_url(url)?;
    let collected_at = DateTime::now_local();
    let synchronized_at = file_metadata
        .modified()
        .map(DateTime::from)
        .unwrap_or_else(|_| {
            log::debug!("Using current time instead of inaccessible last modification time");
            DateTime::now_utc()
        });
    let input = NewTrackInput {
        collected_at,
        synchronized_at,
    };
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let track = input.try_from_url_into_new_track(url, &mime)?;
    match mime.as_ref() {
        "audio/flac" => Ok(flac::ImportTrack.import_track(config, flags, track, &mut reader)?),
        "audio/mpeg" => Ok(mp3::ImportTrack.import_track(config, flags, track, &mut reader)?),
        "audio/m4a" | "audio/mp4" => {
            Ok(mp4::ImportTrack.import_track(config, flags, track, &mut reader)?)
        }
        "audio/ogg" => Ok(ogg::ImportTrack.import_track(config, flags, track, &mut reader)?),
        _ => Err(Error::Media(MediaError::UnsupportedContentType(mime))),
    }
}

pub fn relocate_collected_sources(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    old_uri_prefix: &str,
    new_uri_prefix: &str,
) -> Result<usize> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselRepoError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let updated_at = DateTime::now_utc();
        Ok(db.relocate_media_sources_by_uri_prefix(
            updated_at,
            collection_id,
            old_uri_prefix,
            new_uri_prefix,
        )?)
    })
    .map_err(Into::into)
}
