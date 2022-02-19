// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::entity::EntityUid;

use aoide_media::{io::export::ExportTrackConfig, resolver::VirtualFilePathResolver};
use aoide_repo::track::EntityRepo as _;

use super::*;

pub fn export_metadata_into_file(
    connection: &SqliteConnection,
    track_uid: &EntityUid,
    content_path_resolver: &VirtualFilePathResolver,
    config: &ExportTrackConfig,
) -> Result<bool> {
    let repo = RepoConnection::new(connection);
    let (_, mut track_entity) = repo.load_track_entity_by_uid(track_uid)?;
    uc::media::export_track_metadata_into_file(
        content_path_resolver,
        config,
        &mut track_entity.body.track,
    )
    .map_err(Into::into)
}
