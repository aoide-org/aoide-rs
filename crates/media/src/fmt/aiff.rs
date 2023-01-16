// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::File;

use anyhow::anyhow;
use lofty::{iff::aiff::AiffFile, AudioFile};

use aoide_core::{track::Track, util::canonical::Canonical};

use crate::{
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
    util::artwork::ReplaceEmbeddedArtworkImage,
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
    let mut aiff_file = <AiffFile as AudioFile>::read_from(file, parse_options())?;
    if aiff_file.text_chunks().is_some() {
        return Err(Error::Metadata(anyhow!(
            "Exporting metadata into AIFF files with text chunks is not supported"
        )));
    }

    let id3v2 = if let Some(id3v2) = aiff_file.id3v2_mut() {
        id3v2
    } else {
        aiff_file.set_id3v2(Default::default());
        aiff_file.id3v2_mut().expect("ID3v2")
    };
    let id3v2_orig = id3v2.clone();

    id3v2::export_track_to_tag(id3v2, config, track, replace_embedded_artwork_image);

    let modified = *id3v2 != id3v2_orig;
    if modified {
        aiff_file.save_to(file)?;
    }
    Ok(modified)
}
