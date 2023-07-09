// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    music::tempo::TempoBpm,
    track::{metric::MetricsFlags, Track},
};
use lofty::id3::v2::{ExtendedTextFrame, Frame, FrameValue, Id3v2Tag};

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags},
        import::{ImportTrackConfig, ImportTrackFlags, ImportedTempoBpm, Importer},
    },
    util::{artwork::EditEmbeddedArtworkImage, format_validated_tempo_bpm, FormattedTempoBpm},
};

#[derive(Debug, Default)]
pub(super) struct Import {
    float_bpm: Option<ImportedTempoBpm>,

    #[cfg(feature = "serato-markers")]
    serato_tags: Option<triseratops::tag::TagContainer>,
}

impl Import {
    pub(super) fn build(
        importer: &mut Importer,
        config: &ImportTrackConfig,
        tag: &Id3v2Tag,
    ) -> Self {
        debug_assert!(config.flags.contains(ImportTrackFlags::METADATA));

        let float_bpm = tag.into_iter().find_map(|frame| {
            if let FrameValue::UserText(ExtendedTextFrame {
                description,
                encoding: _,
                content,
                ..
            }) = frame.content()
            {
                if description != "BPM" {
                    return None;
                }
                importer.import_tempo_bpm(content)
            } else {
                None
            }
        });

        #[cfg(feature = "serato-markers")]
        let serato_tags = config
            .flags
            .contains(ImportTrackFlags::SERATO_MARKERS)
            .then(|| import_serato_markers(importer, tag))
            .flatten();

        Self {
            float_bpm,
            #[cfg(feature = "serato-markers")]
            serato_tags,
        }
    }

    pub(super) fn finish(self, track: &mut Track) {
        let Self {
            float_bpm,
            #[cfg(feature = "serato-markers")]
            serato_tags,
        } = self;

        if let Some(float_bpm) = float_bpm {
            // Unconditionally assume that the value is fractional.
            track
                .metrics
                .flags
                .set(MetricsFlags::TEMPO_BPM_NON_FRACTIONAL, false);
            let old_tempo_bpm = &mut track.metrics.tempo_bpm;
            let new_tempo_bpm = TempoBpm::from(float_bpm);
            if let Some(old_tempo_bpm) = old_tempo_bpm {
                if *old_tempo_bpm != new_tempo_bpm {
                    log::debug!("Replacing tempo: {old_tempo_bpm} -> {new_tempo_bpm}");
                }
            }
            *old_tempo_bpm = Some(new_tempo_bpm);
        }

        #[cfg(feature = "serato-markers")]
        if let Some(serato_tags) = &serato_tags {
            super::import_serato_tags(track, serato_tags);
        }
    }
}

#[cfg(feature = "serato-markers")]
#[must_use]
pub(super) fn import_serato_markers(
    importer: &mut crate::io::import::Importer,
    tag: &Id3v2Tag,
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

pub(crate) fn export_track_to_tag(
    tag: &mut Id3v2Tag,
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

    // Post-processing: Export custom metadata

    // Music: Precise tempo BPM as a float value
    tag.retain(|frame| {
        if frame.id_str() != "TXXX" {
            return true;
        }
        let FrameValue::UserText(ExtendedTextFrame {
            description,
            ..
        }) = frame.content() else {
            return true;
        };
        if description != "BPM" {
            return true;
        }
        // Drop the custom BPM frame
        false
    });
    if let Some(formatted) = format_validated_tempo_bpm(
        &mut track.metrics.tempo_bpm,
        crate::util::TempoBpmFormat::Float,
    ) {
        if !track
            .metrics
            .flags
            .contains(MetricsFlags::TEMPO_BPM_NON_FRACTIONAL)
            || matches!(formatted, FormattedTempoBpm::Fractional(_))
        {
            track
                .metrics
                .flags
                .remove(MetricsFlags::TEMPO_BPM_NON_FRACTIONAL);
            let frame = FrameValue::UserText(ExtendedTextFrame {
                description: "BPM".to_owned(),
                encoding: lofty::TextEncoding::UTF8,
                content: formatted.into(),
            });
            tag.insert(Frame::new("TXXX", frame, Default::default()).expect("valid frame"));
        }
    }

    #[cfg(feature = "serato-markers")]
    if config.flags.contains(ExportTrackFlags::SERATO_MARKERS) {
        log::warn!("TODO: Export Serato markers");
    }
}
