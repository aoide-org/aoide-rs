// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use lofty::ogg::VorbisComments;

use aoide_core::track::album::Kind as AlbumKind;

use crate::{io::import::Importer, util::trim_readable};

#[must_use]
pub(super) fn import_album_kind(
    importer: &mut Importer,
    vorbis_comments: &VorbisComments,
) -> Option<AlbumKind> {
    let value = vorbis_comments.get("COMPILATION");
    value
        .and_then(|compilation| trim_readable(compilation).parse::<u8>().ok())
        .and_then(|compilation| match compilation {
            0 => Some(AlbumKind::NoCompilation),
            1 => Some(AlbumKind::Compilation),
            _ => {
                importer.add_issue(format!(
                    "Unexpected tag value: COMPILATION = '{}'",
                    value.expect("unreachable")
                ));
                None
            }
        })
}

#[cfg(feature = "serato-markers")]
#[must_use]
pub(super) fn import_serato_markers(
    importer: &mut Importer,
    vorbis_comments: &VorbisComments,
    format: triseratops::tag::TagFormat,
) -> Option<triseratops::tag::TagContainer> {
    let key = match format {
        triseratops::tag::TagFormat::FLAC => {
            <triseratops::tag::Markers2 as triseratops::tag::format::flac::FLACTag>::FLAC_COMMENT
        }
        triseratops::tag::TagFormat::Ogg => {
            <triseratops::tag::Markers2 as triseratops::tag::format::ogg::OggTag>::OGG_COMMENT
        }
        _ => {
            return None;
        }
    };
    let data = vorbis_comments.get(key)?;
    let mut serato_tags = triseratops::tag::TagContainer::new();
    serato_tags
        .parse_markers2(data.as_bytes(), format)
        .map_err(|err| {
            importer.add_issue(format!("Failed to import Serato Markers2: {err}"));
        })
        .ok()?;
    Some(serato_tags)
}
