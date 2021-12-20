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

use std::path::PathBuf;

use aoide_media::io::import::{load_embedded_artwork_image_from_file_path, LoadedArtworkImage};

use aoide_repo::collection::RecordId as CollectionId;

use aoide_core::{entity::EntityUid, media::SourcePath};

use uc::collection::resolve_collection_id_for_virtual_file_path;

use super::*;

pub fn resolve_file_path(
    db: &RepoConnection<'_>,
    collection_uid: &EntityUid,
    source_path: &SourcePath,
) -> Result<(CollectionId, PathBuf)> {
    resolve_collection_id_for_virtual_file_path(db, collection_uid, None)
        .map_err(transaction_error)
        .map_err(Into::into)
        .map(|(collection_id, source_path_resolver)| {
            let file_path = source_path_resolver.build_file_path(source_path);
            (collection_id, file_path)
        })
}

pub fn load_embedded_artwork_image(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    source_path: &SourcePath,
) -> Result<(CollectionId, Option<LoadedArtworkImage>)> {
    let db = RepoConnection::new(connection);
    resolve_file_path(&db, collection_uid, source_path).and_then(|(collection_id, file_path)| {
        let loaded_artwork_image = load_embedded_artwork_image_from_file_path(&file_path)?;
        Ok((collection_id, loaded_artwork_image))
    })
}
