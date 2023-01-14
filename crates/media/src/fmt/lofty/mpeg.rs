// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use lofty::mpeg::MPEGFile;

use aoide_core::track::Track;

use crate::io::import::{ImportTrackConfig, ImportTrackFlags, Importer};

use super::id3v2::Import;

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
