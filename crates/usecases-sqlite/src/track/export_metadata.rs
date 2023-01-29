// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::media::content::resolver::VirtualFilePathResolver;

use aoide_media::{io::export::ExportTrackConfig, util::artwork::EditEmbeddedArtworkImage};

use aoide_repo::track::EntityRepo as _;

use super::*;

pub fn export_metadata_into_file(
    connection: &mut DbConnection,
    track_uid: &EntityUid,
    content_path_resolver: &VirtualFilePathResolver,
    config: &ExportTrackConfig,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<bool> {
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
