// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{TrackUid, media::content::resolver::vfs::VfsResolver};
use aoide_media_file::{io::export::ExportTrackConfig, util::artwork::EditEmbeddedArtworkImage};
use aoide_repo::track::EntityRepo as _;
use aoide_repo_sqlite::DbConnection;
use aoide_usecases as uc;

use crate::{RepoConnection, Result};

pub fn export_metadata_into_file(
    connection: &mut DbConnection,
    track_uid: &TrackUid,
    content_path_resolver: &VfsResolver,
    config: &ExportTrackConfig,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    let (_, mut track_entity) = repo.load_track_entity_by_uid(track_uid)?;
    uc::media::export_track_metadata_into_file(
        &mut track_entity.body.track,
        content_path_resolver,
        config,
        edit_embedded_artwork_image,
    )
    .map_err(Into::into)
}
