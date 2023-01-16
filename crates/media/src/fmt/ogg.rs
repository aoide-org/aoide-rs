// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::File;

use aoide_core::{track::Track, util::canonical::Canonical};
use lofty::{ogg::VorbisFile, AudioFile};

use crate::{
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
    util::artwork::EditEmbeddedArtworkImage,
    Result,
};

use super::{parse_options, vorbis::export_track_to_tag};

pub(crate) fn import_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    vorbis_file: VorbisFile,
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
                vorbis_file.vorbis_comments(),
                triseratops::tag::TagFormat::Ogg,
            )
        })
        .flatten();

    // Generic import
    let tagged_file = vorbis_file.into();
    super::import_tagged_file_into_track(importer, config, tagged_file, track);

    // Post-processing
    #[cfg(feature = "serato-markers")]
    if let Some(serato_tags) = serato_tags {
        track.cues = Canonical::tie(crate::util::serato::import_cues(&serato_tags));
        track.color = crate::util::serato::import_track_color(&serato_tags);
    }
}

pub(crate) fn export_track_to_file(
    file: &mut File,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<bool> {
    let mut ogg_file = <VorbisFile as AudioFile>::read_from(file, parse_options())?;

    let vorbis_comments = ogg_file.vorbis_comments_mut();
    let vorbis_comments_orig = vorbis_comments.clone();

    export_track_to_tag(vorbis_comments, config, track, edit_embedded_artwork_image);

    let modified = *vorbis_comments != vorbis_comments_orig;
    if modified {
        ogg_file.save_to(file)?;
    }
    Ok(modified)
}
