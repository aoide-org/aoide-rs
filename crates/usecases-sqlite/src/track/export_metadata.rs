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

use aoide_core::{entity::EntityUid, util::clock::DateTime};

use aoide_media::{io::export::ExportTrackConfig, resolver::VirtualFilePathResolver};
use aoide_repo::track::{EntityRepo as _, RecordTrail};

use super::*;

pub fn export_metadata_into_file(
    connection: &SqliteConnection,
    track_uid: &EntityUid,
    source_path_resolver: &VirtualFilePathResolver,
    config: &ExportTrackConfig,
    update_source_synchronized_at: bool,
) -> Result<Option<DateTime>> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
        let (record_header, mut track_entity) = db.load_track_entity_by_uid(track_uid)?;
        let RecordTrail {
            collection_id: _,
            media_source_id,
            media_source_path: _,
        } = db.load_track_record_trail(record_header.id)?;
        let media_source_synchronized_at = track_entity.body.media_source.synchronized_at;
        uc::media::export_track_metadata_into_file(
            source_path_resolver,
            config,
            &mut track_entity.body,
            update_source_synchronized_at,
        )
        .map_err(DieselTransactionError::new)?;
        let updated_at = DateTime::now_utc();
        if let Err(err) =
            db.update_track_entity(record_header.id, updated_at, media_source_id, &track_entity)
        {
            // Rolling back the transaction does not help after the metadata
            // has already been exported into the file and cannot be undone!
            tracing::error!(
                "Failed to update track in database after exporting metadata into file: {}",
                err
            );
            return Ok(media_source_synchronized_at);
        }
        Ok(track_entity.body.media_source.synchronized_at)
    })
    .map_err(Into::into)
}
