// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use lofty::ogg::{OpusFile, VorbisComments};

use aoide_core::{media::artwork::EditEmbeddedArtworkImage, track::Track};

use crate::{
    Error, Result,
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
};

use super::vorbis::export_track_to_tag;

pub(crate) fn import_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    opus_file: OpusFile,
    track: &mut Track,
) {
    // Pre-processing
    #[cfg(feature = "serato-markers")]
    let serato_tags = config
        .flags
        .contains(ImportTrackFlags::SERATO_MARKERS)
        .then(|| {
            super::vorbis::import_serato_markers(
                importer,
                opus_file.vorbis_comments(),
                triseratops::tag::TagFormat::Ogg,
            )
        })
        .flatten();

    // Generic import
    let tagged_file = opus_file.into();
    super::import_tagged_file_into_track(importer, config, tagged_file, track);

    // Post-processing
    #[cfg(feature = "serato-markers")]
    if let Some(serato_tags) = &serato_tags {
        super::import_serato_tags(track, serato_tags);
    }
}

pub(crate) fn export_track_to_vorbis_comments(
    vorbis_comments: &mut VorbisComments,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    if track.media_source.content.r#type.essence_str() != "audio/opus" {
        return Err(Error::UnsupportedContentType(
            track.media_source.content.r#type.clone(),
        ));
    }
    export_track_to_tag(vorbis_comments, config, track, edit_embedded_artwork_image);
    Ok(())
}
