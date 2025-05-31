// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    if track.media_source.content.r#type.essence_str() != "audio/aiff" {
        return Err(Error::UnsupportedContentType(
            track.media_source.content.r#type.clone(),
        ));
    }

    let id3v2 = if let Some(id3v2) = aiff_file.id3v2_mut() {
        id3v2
    } else {
        aiff_file.set_id3v2(Default::default());
        aiff_file.id3v2_mut().expect("Some")
    };
    id3v2::export_track_to_tag(id3v2, config, track, edit_embedded_artwork_image);

    Ok(())
}
