// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::Track;
use lofty::ogg::VorbisComments;

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags},
        import::Importer,
    },
    util::artwork::EditEmbeddedArtworkImage,
};

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

pub(crate) fn export_track_to_tag(
    tag: &mut VorbisComments,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) {
    *tag = super::split_export_merge_track_to_tag(
        std::mem::take(tag),
        config,
        track,
        edit_embedded_artwork_image,
    );

    #[cfg(feature = "serato-markers")]
    if config.flags.contains(ExportTrackFlags::SERATO_MARKERS) {
        log::warn!("TODO: Export Serato markers");
    }
}
