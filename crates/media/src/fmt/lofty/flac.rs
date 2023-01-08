// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use lofty::flac::FlacFile;

use aoide_core::{track::Track, util::canonical::Canonical};

use crate::io::import::{ImportTrackConfig, ImportTrackFlags, Importer};

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
