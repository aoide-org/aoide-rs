// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::File;

use lofty::{flac::FlacFile, AudioFile};

use aoide_core::{track::Track, util::canonical::Canonical};

use crate::{
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
    util::artwork::ReplaceEmbeddedArtworkImage,
    Result,
};

use super::{parse_options, vorbis::export_track_to_tag};

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
            super::vorbis::import_serato_markers(
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
    if let Some(serato_tags) = serato_tags {
        track.cues = Canonical::tie(crate::util::serato::import_cues(&serato_tags));
        track.color = crate::util::serato::import_track_color(&serato_tags);
    }
}

pub(crate) fn export_track_to_file(
    file: &mut File,
    config: &ExportTrackConfig,
    track: &mut Track,
    replace_embedded_artwork_image: Option<ReplaceEmbeddedArtworkImage>,
) -> Result<bool> {
    let mut flac_file = <FlacFile as AudioFile>::read_from(file, parse_options())?;

    let vorbis_comments = if let Some(vorbis_comments) = flac_file.vorbis_comments_mut() {
        vorbis_comments
    } else {
        flac_file.set_vorbis_comments(Default::default());
        flac_file.vorbis_comments_mut().expect("VorbisComments")
    };
    let vorbis_comments_orig = vorbis_comments.clone();

    export_track_to_tag(
        vorbis_comments,
        config,
        track,
        replace_embedded_artwork_image,
    );

    let modified = *vorbis_comments != vorbis_comments_orig;
    if modified {
        // Prevent inconsistencies by stripping all other, secondary tags
        flac_file.remove_id3v2();
        flac_file.save_to(file)?;
    }
    Ok(modified)
}
