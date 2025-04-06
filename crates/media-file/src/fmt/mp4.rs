// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, fs::File};

use lofty::{
    config::WriteOptions,
    file::AudioFile,
    mp4::{AtomData, AtomIdent, Ilst, Mp4File},
};

use aoide_core::{media::artwork::EditEmbeddedArtworkImage, track::Track};

use crate::{
    Result,
    io::{
        export::{ExportTrackConfig, ExportTrackFlags},
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
};

use super::parse_options;

#[cfg(feature = "serato-markers")]
const SERATO_MARKERS_IDENT: AtomIdent<'_> = AtomIdent::Freeform {
    mean: Cow::Borrowed(<triseratops::tag::Markers as triseratops::tag::format::mp4::MP4Tag>::MP4_ATOM_FREEFORM_MEAN),
    name: Cow::Borrowed(<triseratops::tag::Markers as triseratops::tag::format::mp4::MP4Tag>::MP4_ATOM_FREEFORM_NAME),
};

#[cfg(feature = "serato-markers")]
const SERATO_MARKERS2_IDENT: AtomIdent<'_> = AtomIdent::Freeform {
    mean: Cow::Borrowed(<triseratops::tag::Markers2 as triseratops::tag::format::mp4::MP4Tag>::MP4_ATOM_FREEFORM_MEAN),
    name: Cow::Borrowed(<triseratops::tag::Markers2 as triseratops::tag::format::mp4::MP4Tag>::MP4_ATOM_FREEFORM_NAME),
};

#[cfg(feature = "serato-markers")]
#[must_use]
fn import_serato_markers(
    importer: &mut Importer,
    ilst: &lofty::mp4::Ilst,
) -> Option<triseratops::tag::TagContainer> {
    let mut parsed = false;

    let mut serato_tags = triseratops::tag::TagContainer::new();

    if let Some(data) = ilst
        .get(&SERATO_MARKERS_IDENT)
        .and_then(|atom| atom.data().next())
    {
        match data {
            AtomData::UTF8(input) => {
                match serato_tags.parse_markers(input.as_bytes(), triseratops::tag::TagFormat::MP4)
                {
                    Ok(()) => {
                        parsed = true;
                    }
                    Err(err) => {
                        importer.add_issue(format!("Failed to parse Serato Markers: {err}"));
                    }
                }
            }
            data => {
                importer.add_issue(format!("Unexpected data for Serato Markers: {data:?}"));
            }
        }
    }

    if let Some(data) = ilst
        .get(&SERATO_MARKERS2_IDENT)
        .and_then(|atom| atom.data().next())
    {
        match data {
            AtomData::UTF8(input) => {
                match serato_tags.parse_markers2(input.as_bytes(), triseratops::tag::TagFormat::MP4)
                {
                    Ok(()) => {
                        parsed = true;
                    }
                    Err(err) => {
                        importer.add_issue(format!("Failed to parse Serato Markers2: {err}"));
                    }
                }
            }
            data => {
                importer.add_issue(format!("Unexpected data for Serato Markers2: {data:?}"));
            }
        }
    }

    parsed.then_some(serato_tags)
}

#[derive(Debug, Default)]
struct Import {
    #[cfg(feature = "serato-markers")]
    serato_tags: Option<triseratops::tag::TagContainer>,
}

impl Import {
    fn build(
        #[cfg(feature = "serato-markers")] importer: &mut Importer,
        #[cfg(not(feature = "serato-markers"))] _importer: &mut Importer,
        config: &ImportTrackConfig,
        ilst: &Ilst,
    ) -> Self {
        debug_assert!(config.flags.contains(ImportTrackFlags::METADATA));

        #[cfg(feature = "serato-markers")]
        let serato_tags = config
            .flags
            .contains(ImportTrackFlags::SERATO_MARKERS)
            .then(|| import_serato_markers(importer, ilst))
            .flatten();

        Self {
            #[cfg(feature = "serato-markers")]
            serato_tags,
        }
    }

    fn finish(self, track: &mut Track) {
        let Self {
            #[cfg(feature = "serato-markers")]
            serato_tags,
        } = self;

        #[cfg(feature = "serato-markers")]
        if let Some(serato_tags) = &serato_tags {
            super::import_serato_tags(track, serato_tags);
        }
    }
}

pub(crate) fn import_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    mp4_file: Mp4File,
    track: &mut Track,
) {
    // Pre-processing
    let import = config
        .flags
        .contains(ImportTrackFlags::METADATA)
        .then(|| mp4_file.ilst())
        .flatten()
        .map(|ilst| Import::build(importer, config, ilst));

    // Import generic metadata
    let tagged_file = mp4_file.into();
    super::import_tagged_file_into_track(importer, config, tagged_file, track);

    // Post-processing
    if let Some(import) = import {
        import.finish(track);
    }
}

pub(crate) fn export_track_to_file(
    file: &mut File,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    let mut mp4_file = <Mp4File as AudioFile>::read_from(file, parse_options())?;
    let mut ilst = mp4_file.ilst_mut().map(std::mem::take).unwrap_or_default();

    export_track_to_tag(&mut ilst, config, track, edit_embedded_artwork_image);

    mp4_file.set_ilst(ilst);
    mp4_file.save_to(file, WriteOptions::default())?;

    Ok(())
}

pub(crate) fn export_track_to_tag(
    ilst: &mut Ilst,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) {
    *ilst = super::split_export_merge_track_to_tag(
        std::mem::take(ilst),
        config,
        track,
        edit_embedded_artwork_image,
    );

    #[cfg(feature = "serato-markers")]
    if config.flags.contains(ExportTrackFlags::SERATO_MARKERS) {
        log::warn!("TODO: Export Serato markers");
    }
}
