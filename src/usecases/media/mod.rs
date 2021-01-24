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

use std::{
    io::BufReader,
    sync::atomic::{AtomicBool, Ordering},
};

use super::*;

use aoide_core::{entity::EntityUid, track::Track, util::clock::DateTime};

use aoide_media::{
    digest::{self as media_digest, DirScanOutcome as DigestDirScanOutcome, NextDirScanStep},
    guess_mime_from_url, mp4, open_local_file_url_for_reading, ImportTrack, ImportTrackConfig,
    ImportTrackOptions, NewTrackInput, Reader,
};

use aoide_repo::{
    collection::EntityRepo as _,
    media::{
        dir_cache::{CacheStatus, Repo as _, UpdateOutcome},
        source::Repo as _,
    },
};

use url::Url;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DirScanSummary {
    pub current: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DirScanOutcome {
    Finished(DirScanSummary),
    Aborted,
}

pub fn scan_directories_recursively(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    root_dir_url: &Url,
    max_depth: Option<usize>,
    aborted_flag: &AtomicBool,
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
    let mut summary = DirScanSummary::default();
    db.transaction::<_, DieselRepoError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let updated_at = DateTime::now_utc();
        let outdated_count = db.media_dir_cache_update_entries_status(
            updated_at,
            collection_id,
            root_dir_url.as_str(),
            Some(CacheStatus::Current),
            CacheStatus::Outdated,
        )?;
        log::debug!(
            "Marked {} current cache entries as outdated",
            outdated_count
        );
        media_digest::digest_directories_recursively::<_, _, anyhow::Error, _>(
            &root_path,
            max_depth,
            blake3::Hasher::new,
            |path, digest| {
                if aborted_flag.swap(false, Ordering::Relaxed) {
                    return Ok(NextDirScanStep::Abort);
                }
                debug_assert!(path.is_relative());
                let full_path = root_path.join(&path);
                debug_assert!(full_path.is_absolute());
                let url = Url::from_directory_path(&full_path).expect("URL");
                debug_assert!(url.as_str().starts_with(root_dir_url.as_str()));
                match db
                    .media_dir_cache_update_entry_digest(
                        updated_at,
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
                Ok(NextDirScanStep::Continue)
            },
        )
        .map_err(anyhow::Error::from)
        .map_err(RepoError::from)
        .and_then(|outcome| {
            match outcome {
                DigestDirScanOutcome::Finished(_) => Ok(()),
                DigestDirScanOutcome::Aborted => Err(RepoError::Aborted), // rollback
            }
        })?;
        summary.orphaned = db.media_dir_cache_update_entries_status(
            updated_at,
            collection_id,
            root_dir_url.as_str(),
            Some(CacheStatus::Outdated),
            CacheStatus::Orphaned,
        )?;
        debug_assert!(summary.orphaned <= outdated_count);
        Ok(())
    })?;
    Ok(DirScanOutcome::Finished(summary))
}

pub fn import_track_from_url(
    url: &Url,
    config: &ImportTrackConfig,
    options: ImportTrackOptions,
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
    let file_size = file_metadata.len();
    let track = input.try_from_url_into_new_track(url, &mime)?;
    if mime == "audio/m4a" {
        Ok(mp4::ImportTrack.import_track(config, options, track, &mut reader, file_size)?)
    } else {
        Err(Error::Media(MediaError::UnsupportedContentType(mime)))
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
