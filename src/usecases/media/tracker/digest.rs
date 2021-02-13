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

use aoide_core::{entity::EntityUid, util::clock::DateTime};

use aoide_media::fs::dir_digest;

use aoide_repo::media::tracker::DirUpdateOutcome;

use std::sync::atomic::AtomicBool;
use url::Url;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub current: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Outcome {
    pub completion: Completion,
    pub summary: Summary,
}

pub fn digest_recursively(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    root_dir_url: &Url,
    max_depth: Option<usize>,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let root_dir_path = root_dir_path_from_url(root_dir_url)?;
    let db = RepoConnection::new(connection);
    Ok(db.transaction::<_, DieselRepoError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let outdated_count = db.media_tracker_mark_current_directories_outdated(
            DateTime::now_utc(),
            collection_id,
            root_dir_url.as_str(),
        )?;
        log::debug!(
            "Marked {} current cache entries as outdated",
            outdated_count
        );
        let mut summary = Summary::default();
        let completion = dir_digest::digest_directories::<_, anyhow::Error, _, _, _>(
            &root_dir_path,
            max_depth,
            abort_flag,
            blake3::Hasher::new,
            |path, digest| {
                debug_assert!(path.is_relative());
                let full_path = root_dir_path.join(&path);
                debug_assert!(full_path.is_absolute());
                let url = Url::from_directory_path(&full_path).expect("URL");
                debug_assert!(url.as_str().starts_with(root_dir_url.as_str()));
                match db
                    .media_tracker_update_directory_digest(
                        DateTime::now_utc(),
                        collection_id,
                        url.as_str(),
                        &digest.into(),
                    )
                    .map_err(anyhow::Error::from)?
                {
                    DirUpdateOutcome::Current => {
                        summary.current += 1;
                    }
                    DirUpdateOutcome::Inserted => {
                        log::debug!("Found added directory: {}", full_path.display());
                        summary.added += 1;
                    }
                    DirUpdateOutcome::Updated => {
                        log::debug!("Found modified directory: {}", full_path.display());
                        summary.modified += 1;
                    }
                    DirUpdateOutcome::Skipped => {
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
                completion,
                progress: _,
            } = outcome;
            match completion {
                dir_digest::Completion::Finished => {
                    // Mark all remaining entries that are unreachable and
                    // have not been visited as orphaned.
                    summary.orphaned = db.media_tracker_mark_outdated_directories_orphaned(
                        DateTime::now_utc(),
                        collection_id,
                        root_dir_url.as_str(),
                    )?;
                    debug_assert!(summary.orphaned <= outdated_count);
                    Ok(Completion::Finished)
                }
                dir_digest::Completion::Aborted => {
                    // All partial results up to now can safely be committed.
                    Ok(Completion::Aborted)
                }
            }
        })?;
        Ok(Outcome {
            completion,
            summary,
        })
    })?)
}
