// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use lofty::flac::FlacFile;

use aoide_core::{media::artwork::EditEmbeddedArtworkImage, track::Track};

use crate::{
    Error, Result,
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
};

use super::vorbis;

pub(crate) fn import_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    flac_file: FlacFile,
    track: &mut Track,
) {
    // Pre-processing
    #[cfg(feature = "serato-markers")]
    let serato_tags = config
        .flags
        .contains(ImportTrackFlags::SERATO_MARKERS)
        .then(|| flac_file.vorbis_comments())
        .flatten()
        .and_then(|vorbis_comments| {
            vorbis::import_serato_markers(
                importer,
                vorbis_comments,
                triseratops::tag::TagFormat::FLAC,
            )
        });

    // Generic import
    let tagged_file = flac_file.into();
    super::import_tagged_file_into_track(importer, config, tagged_file, track);

    // Post-processing
    #[cfg(feature = "serato-markers")]
    if let Some(serato_tags) = &serato_tags {
        super::import_serato_tags(track, serato_tags);
    }
}

pub(crate) fn export_track_to_file_vorbis_comments(
    flac_file: &mut FlacFile,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    if track.media_source.content.r#type.essence_str() != "audio/flac" {
        return Err(Error::UnsupportedContentType(
            track.media_source.content.r#type.clone(),
        ));
    }

    let vorbis_comments = if let Some(vorbis_comments) = flac_file.vorbis_comments_mut() {
        vorbis_comments
    } else {
        flac_file.set_vorbis_comments(Default::default());
        flac_file.vorbis_comments_mut().expect("Some")
    };
    vorbis::export_track_to_tag(vorbis_comments, config, track, edit_embedded_artwork_image);

    Ok(())
}
