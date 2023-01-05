// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use aoide_core::track::Track;

use crate::{io::export::ExportTrackConfig, Error, Result};

use super::id3::{export_track as export_track_into_id3_tag, map_id3_err};

pub fn export_track_to_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
) -> Result<bool> {
    let id3_tag_orig = id3::Tag::read_from_path(path).map_err(map_id3_err)?;

    let mut id3_tag = id3_tag_orig.clone();
    export_track_into_id3_tag(config, track, &mut id3_tag)
        .map_err(|err| Error::Other(anyhow::anyhow!("Failed to export ID3 tag: {err:?}")))?;

    if id3_tag == id3_tag_orig {
        // Unmodified
        return Ok(false);
    }
    id3_tag
        .write_to_path(path, id3::Version::Id3v24)
        .map_err(map_id3_err)?;
    // Modified
    Ok(true)
}
