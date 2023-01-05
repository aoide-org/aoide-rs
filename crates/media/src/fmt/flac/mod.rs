// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, path::Path};

use aoide_core::track::Track;

use crate::{io::export::ExportTrackConfig, Error, Result};

use super::vorbis;

impl vorbis::CommentWriter for metaflac::Tag {
    fn overwrite_single_value(&mut self, key: Cow<'_, str>, value: &'_ str) {
        if self.get_vorbis(&key).is_some() {
            self.write_single_value(key, value.into());
        }
    }
    fn write_multiple_values(&mut self, key: Cow<'_, str>, values: Vec<String>) {
        if values.is_empty() {
            self.remove_vorbis(&key);
        } else {
            self.set_vorbis(key, values);
        }
    }
    fn remove_all_values(&mut self, key: &str) {
        self.remove_vorbis(key);
    }
}

fn map_metaflac_err(err: metaflac::Error) -> Error {
    let metaflac::Error { kind, description } = err;
    match kind {
        metaflac::ErrorKind::Io(err) => Error::Io(err),
        kind => Error::Other(anyhow::Error::from(metaflac::Error { kind, description })),
    }
}

pub fn export_track_to_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
) -> Result<bool> {
    let mut metaflac_tag = match metaflac::Tag::read_from_path(path) {
        Ok(metaflac_tag) => metaflac_tag,
        Err(err) => {
            let content_path = &track.media_source.content.link.path;
            log::warn!("Failed to parse metadata from media source '{content_path}': {err}");
            return Err(map_metaflac_err(err));
        }
    };

    let vorbis_comments_orig = metaflac_tag.vorbis_comments().map(ToOwned::to_owned);
    vorbis::export_track(config, track, &mut metaflac_tag);

    if metaflac_tag.vorbis_comments() == vorbis_comments_orig.as_ref() {
        // Unmodified
        return Ok(false);
    }
    metaflac_tag.write_to_path(path).map_err(map_metaflac_err)?;
    // Modified
    Ok(true)
}
