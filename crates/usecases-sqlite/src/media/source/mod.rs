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

use std::path::PathBuf;

use aoide_media::io::import::{
    load_embedded_artwork_image_from_file_path, Importer, LoadedArtworkImage,
};

use aoide_repo::collection::RecordId as CollectionId;

use aoide_core::{entity::EntityUid, media::SourcePath};
use uc::collection::vfs::RepoContext;

use super::*;

pub mod purge_orphaned;
pub mod purge_untracked;
pub mod relocate;

pub fn resolve_file_path(
    db: &RepoConnection<'_>,
    collection_uid: &EntityUid,
    source_path: &SourcePath,
) -> Result<(CollectionId, PathBuf)> {
    let collection_ctx = RepoContext::resolve(db, collection_uid, None)?;
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.source_path.vfs {
        vfs_ctx
    } else {
        return Err(anyhow::anyhow!(
            "Unsupported path kind: {:?}",
            collection_ctx.source_path.kind
        )
        .into());
    };
    let file_path = vfs_ctx.path_resolver.build_file_path(source_path);
    Ok((collection_ctx.record_id, file_path))
}

pub fn load_embedded_artwork_image(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    source_path: &SourcePath,
) -> Result<(CollectionId, Option<LoadedArtworkImage>)> {
    let db = RepoConnection::new(connection);
    let mut importer = Importer::new();
    resolve_file_path(&db, collection_uid, source_path).and_then(|(collection_id, file_path)| {
        let loaded_artwork_image =
            load_embedded_artwork_image_from_file_path(&mut importer, &file_path)?;
        for issue_message in importer.finish().into_messages() {
            log::warn!("{}", issue_message);
        }
        Ok((collection_id, loaded_artwork_image))
    })
}
