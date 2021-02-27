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

use aoide_core::{media::Source as MediaSource, track::Track};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::{source::Repo as MediaSourceRepo, tracker::Repo as MediaTrackerRepo},
    track::EntityRepo as TrackRepo,
};

/// Relink a moved track.
///
/// Replace the track referenced by `old_source_uri` with the replacement
/// referenced by `new_source_uri`. Afterwards the replacement track is
/// deleted which requires that is not yet referenced in the collection,
/// e.g. as a playlist entry.
///
/// The `collected_at` timestamp of the old track is preserved while all
/// other properties are copied from the replacement track.
///
/// The media tracker is also updated, i.e. it will reference the updated
/// old media source instead of the new media source that is removed.
pub fn relink_collected_track_by_media_source_uri<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    old_source_uri: &str,
    new_source_uri: &str,
) -> RepoResult<()>
where
    Repo: CollectionRepo + TrackRepo + MediaSourceRepo + MediaTrackerRepo,
{
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    let (old_source_id, old_header, old_entity) =
        repo.load_track_entity_by_media_source_uri(collection_id, old_source_uri)?;
    let (new_source_id, new_header, new_entity) =
        repo.load_track_entity_by_media_source_uri(collection_id, new_source_uri)?;
    let updated_track = Track {
        media_source: MediaSource {
            // Preserve the collected_at field from the old source
            collected_at: old_entity.body.media_source.collected_at,
            ..new_entity.body.media_source
        },
        ..new_entity.body
    };
    // Relink the sources in the media tracker
    repo.media_tracker_relink_source(old_source_id, new_source_id)?;
    // Delete the soon obsolete track and source records to prevent
    // constraint violations during the update. This only works as
    // long as the track is not referenced elsewhere, e.g. playlists!
    repo.delete_track_entity(new_header.id)?;
    repo.delete_media_source(new_source_id)?;
    // Finish with updating the old track
    if updated_track != old_entity.body {
        let updated_at = DateTime::now_utc();
        if old_entity.body.media_source != updated_track.media_source {
            repo.update_media_source(old_source_id, updated_at, &updated_track.media_source)?;
            debug_assert_eq!(
                updated_track.media_source,
                repo.load_media_source_by_uri(collection_id, new_source_uri)?
                    .1
            );
        }
        let updated_entity = Entity::new(old_entity.hdr, updated_track);
        repo.update_track_entity(old_header.id, updated_at, old_source_id, &updated_entity)?;
        debug_assert_eq!(
            updated_entity.body,
            repo.load_track_entity_by_media_source_uri(collection_id, new_source_uri)?
                .2
                .body
        );
    }
    Ok(())
}
