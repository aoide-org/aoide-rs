// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::File;

use anyhow::anyhow;
use lofty::{iff::aiff::AiffFile, AudioFile};

use aoide_core::track::Track;

use crate::{
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
    util::artwork::EditEmbeddedArtworkImage,
    Error, Result,
};

use super::{id3v2, parse_options};

pub(crate) fn import_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    aiff_file: AiffFile,
    track: &mut Track,
) {
    // Pre-processing

    #[cfg(feature = "serato-markers")]
    let serato_tags = config
        .flags
        .contains(ImportTrackFlags::SERATO_MARKERS)
        .then(|| aiff_file.id3v2())
        .flatten()
        .and_then(|tag| super::id3v2::import_serato_markers(importer, tag));

    // Generic import
    let tagged_file = aiff_file.into();
    super::import_tagged_file_into_track(importer, config, tagged_file, track);

    // Post-processing

    #[cfg(feature = "serato-markers")]
    if let Some(serato_tags) = &serato_tags {
        super::import_serato_tags(track, serato_tags);
    }
}

pub(crate) fn export_track_to_file(
    file: &mut File,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    let mut aiff_file = <AiffFile as AudioFile>::read_from(file, parse_options())?;
    if aiff_file.text_chunks().is_some() {
        return Err(Error::Metadata(anyhow!(
            "Exporting metadata into AIFF files with text chunks is not supported"
        )));
    }
    let mut id3v2 = aiff_file
        .id3v2_mut()
        .map(std::mem::take)
        .unwrap_or_default();

    id3v2::export_track_to_tag(&mut id3v2, config, track, edit_embedded_artwork_image);

    aiff_file.set_id3v2(id3v2);
    aiff_file.save_to(file)?;

    Ok(())
}
