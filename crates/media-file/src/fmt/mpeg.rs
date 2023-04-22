// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::File;

use aoide_core::track::Track;
use lofty::{mpeg::MPEGFile, AudioFile};

use super::{
    id3v2::{export_track_to_tag, Import},
    parse_options,
};
use crate::{
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
    util::artwork::EditEmbeddedArtworkImage,
    Result,
};

pub(crate) fn import_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    mpeg_file: MPEGFile,
    track: &mut Track,
) {
    // Pre-processing
    let import = config
        .flags
        .contains(ImportTrackFlags::METADATA)
        .then(|| mpeg_file.id3v2())
        .flatten()
        .map(|tag| Import::build(importer, config, tag));

    // Import generic metadata
    let tagged_file = mpeg_file.into();
    super::import_tagged_file_into_track(importer, config, tagged_file, track);

    // Post-processing
    if let Some(import) = import {
        import.finish(track);
    }
}

pub(crate) fn export_track_to_file(
    file: &mut File,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    let mut mpeg_file = <MPEGFile as AudioFile>::read_from(file, parse_options())?;

    let mut id3v2 = mpeg_file
        .id3v2_mut()
        .map(std::mem::take)
        .unwrap_or_default();

    export_track_to_tag(&mut id3v2, config, track, edit_embedded_artwork_image);

    mpeg_file.set_id3v2(id3v2);
    mpeg_file.save_to(file)?;

    Ok(())
}
