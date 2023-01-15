// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::Track;
use lofty::{ogg::VorbisComments, Tag, TagType};

use crate::io::{
    export::{ExportTrackConfig, ExportTrackFlags},
    import::Importer,
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

fn export_track_to_tag_generic(
    tag: &mut VorbisComments,
    config: &ExportTrackConfig,
    track: &mut Track,
) {
    // Collect keys that would survive a roundtrip
    let old_keys = VorbisComments::from(Tag::from(tag.clone()))
        .take_items()
        .map(|(key, _)| key)
        .collect::<Vec<_>>();
    // Export generic metadata
    let mut new_tag = Tag::new(TagType::VorbisComments);
    super::export_track_to_tag(&mut new_tag, config, track);
    let mut new_tag = VorbisComments::from(new_tag);
    // Merge generic metadata
    for key in old_keys {
        std::mem::forget(tag.remove(&key));
    }
    for (key, value) in new_tag.take_items() {
        tag.insert(key, value, false);
    }
}

pub(crate) fn export_track_to_tag(
    tag: &mut VorbisComments,
    config: &ExportTrackConfig,
    track: &mut Track,
) {
    export_track_to_tag_generic(tag, config, track);

    #[cfg(feature = "serato-markers")]
    if config.flags.contains(ExportTrackFlags::SERATO_MARKERS) {
        log::warn!("TODO: Export Serato markers");
    }
}
