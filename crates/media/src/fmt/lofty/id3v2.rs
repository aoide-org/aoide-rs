// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use lofty::id3::v2::ID3v2Tag;

use aoide_core::{
    track::{metric::MetricsFlags, Track},
    util::canonical::Canonical,
};

use crate::io::import::{ImportTrackConfig, ImportTrackFlags, ImportedTempoBpm, Importer};

const FLOAT_BPM_FRAME_ID: &str = "TXXX:BPM";

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
        tag: &ID3v2Tag,
    ) -> Self {
        debug_assert!(config.flags.contains(ImportTrackFlags::METADATA));

        let float_bpm = tag
            .get_text(FLOAT_BPM_FRAME_ID)
            .and_then(|input| importer.import_tempo_bpm(&input));

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
            track.metrics.flags.set(
                MetricsFlags::TEMPO_BPM_NON_FRACTIONAL,
                float_bpm.is_non_fractional(),
            );
            track.metrics.tempo_bpm = Some(float_bpm.into());
        }

        #[cfg(feature = "serato-markers")]
        if let Some(serato_tags) = serato_tags {
            track.cues = Canonical::tie(crate::util::serato::import_cues(&serato_tags));
            track.color = crate::util::serato::import_track_color(&serato_tags);
        }
    }
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
