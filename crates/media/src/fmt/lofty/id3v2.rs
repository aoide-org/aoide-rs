// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// FIXME

use lofty::id3::v2::ID3v2Tag;

use aoide_core::track::album::Kind as AlbumKind;

use crate::{io::import::Importer, util::trim_readable};

pub(super) fn import_album_kind(importer: &mut Importer, tag: &ID3v2Tag) -> Option<AlbumKind> {
    let value = tag.get_text("TCMP");
    value
        .as_ref()
        .and_then(|compilation| trim_readable(compilation).parse::<u8>().ok())
        .and_then(|compilation| match compilation {
            0 => Some(AlbumKind::NoCompilation),
            1 => Some(AlbumKind::Compilation),
            _ => {
                importer.add_issue(format!(
                    "Unexpected tag value: TCMP = '{}'",
                    value.expect("unreachable")
                ));
                None
            }
        })
}

#[cfg(feature = "serato-markers")]
#[must_use]
pub(super) fn import_serato_markers(
    importer: &mut crate::io::import::Importer,
    tag: &ID3v2Tag,
) -> Option<triseratops::tag::TagContainer> {
    let mut serato_tags = triseratops::tag::TagContainer::new();
    let mut parsed = false;

    if let Some(frame) =
        tag.get(<triseratops::tag::Markers as triseratops::tag::format::id3::ID3Tag>::ID3_TAG)
    {
        if let lofty::id3::v2::FrameValue::Binary(data) = frame.content() {
            match serato_tags.parse_markers(data, triseratops::tag::TagFormat::ID3) {
                Ok(()) => {
                    parsed = true;
                }
                Err(err) => {
                    importer.add_issue(format!("Failed to parse Serato Markers: {err}"));
                }
            }
        } else {
            importer.add_issue(format!("Unexpected Serato Markers frame: {frame:?}"));
        }
    }
    if let Some(frame) =
        tag.get(<triseratops::tag::Markers2 as triseratops::tag::format::id3::ID3Tag>::ID3_TAG)
    {
        if let lofty::id3::v2::FrameValue::Binary(data) = frame.content() {
            match serato_tags.parse_markers(data, triseratops::tag::TagFormat::ID3) {
                Ok(()) => {
                    parsed = true;
                }
                Err(err) => {
                    importer.add_issue(format!("Failed to parse Serato Markers2: {err}"));
                }
            }
        } else {
            importer.add_issue(format!("Unexpected Serato Markers2 frame: {frame:?}"));
        }
    }

    parsed.then_some(serato_tags)
}
