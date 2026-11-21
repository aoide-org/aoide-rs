// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;
use lofty::iff::aiff::AiffFile;

use aoide_core::{media::artwork::EditEmbeddedArtworkImage, track::Track};

use crate::{
    Error, Result,
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
};

use super::id3v2;

pub(crate) fn import_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    aiff_file: AiffFile,
    track: &mut Track,
) {
    // Pre-processing
    let import = config
        .flags
        .contains(ImportTrackFlags::METADATA)
        .then(|| aiff_file.id3v2())
        .flatten()
        .map(|tag| id3v2::Import::build(importer, config, tag));

    // Import generic metadata
    let tagged_file = aiff_file.into();
    super::import_tagged_file_into_track(importer, config, tagged_file, track);

    // Post-processing
    if let Some(import) = import {
        import.finish(track);
    }
}

pub(crate) fn export_track_to_file_id3v2(
    aiff_file: &mut AiffFile,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    if aiff_file.text_chunks().is_some() {
        return Err(Error::Metadata(anyhow!(
            "Exporting metadata into AIFF files with text chunks is not supported"
        )));
    }

    let mut id3v2 = aiff_file.remove_id3v2().unwrap_or_default();
    id3v2::export_track_to_tag(&mut id3v2, config, track, edit_embedded_artwork_image);
    aiff_file.set_id3v2(id3v2);

    Ok(())
}
