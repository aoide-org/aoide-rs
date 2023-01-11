// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, fs::File};

use lofty::{
    mp4::{Atom, AtomData, AtomIdent, Ilst, Mp4File},
    AudioFile, ItemKey, Tag, TagType, TaggedFile, TaggedFileExt as _,
};

use aoide_core::{
    media::AdvisoryRating,
    track::{
        title::{Kind as TitleKind, Title, Titles},
        Track,
    },
    util::canonical::Canonical,
};

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags},
        import::{ImportTrackConfig, ImportTrackFlags, Importer},
    },
    util::{format_validated_tempo_bpm, ingest_title_from},
    Result,
};

use super::parse_options;

const ADVISORY_RATING_IDENT: AtomIdent<'_> = AtomIdent::Fourcc(*b"rtng");

const LEGACY_GENRE_IDENT: AtomIdent<'_> = AtomIdent::Fourcc(*b"gnre"); // numeric identifier

const WORK_NAME_IDENT: AtomIdent<'_> = AtomIdent::Fourcc(*b"\xa9wrk");

const COM_APPLE_ITUNES_FREEFORM_MEAN: &str = "----:com.apple.iTunes";

const FLOAT_BPM_IDENT: AtomIdent<'_> = AtomIdent::Freeform {
    mean: Cow::Borrowed(COM_APPLE_ITUNES_FREEFORM_MEAN),
    name: Cow::Borrowed("BPM"),
};

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

fn import_advisory_rating(advisory_rating: lofty::mp4::AdvisoryRating) -> AdvisoryRating {
    use lofty::mp4::AdvisoryRating::*;
    match advisory_rating {
        Inoffensive => AdvisoryRating::Unrated,
        Clean => AdvisoryRating::Clean,
        Explicit => AdvisoryRating::Explicit,
    }
}

fn export_advisory_rating(advisory_rating: AdvisoryRating) -> lofty::mp4::AdvisoryRating {
    use AdvisoryRating::*;
    match advisory_rating {
        Unrated => lofty::mp4::AdvisoryRating::Inoffensive,
        Clean => lofty::mp4::AdvisoryRating::Clean,
        Explicit => lofty::mp4::AdvisoryRating::Explicit,
    }
}

#[derive(Debug, Default)]
struct Import {
    advisory_rating: Option<AdvisoryRating>,

    work_title: Option<Title>,

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
        // TODO: Handle in generic import
        // See also: <https://github.com/Serial-ATA/lofty-rs/issues/99>
        let advisory_rating = config
            .flags
            .contains(ImportTrackFlags::METADATA)
            .then(|| ilst.advisory_rating().map(import_advisory_rating))
            .flatten();

        // Additional titles not (yet) supported by lofty-rs
        let work_title = config
            .flags
            .contains(ImportTrackFlags::METADATA)
            .then(|| ilst.atom(&WORK_NAME_IDENT))
            .flatten()
            .into_iter()
            .flat_map(Atom::data)
            .find_map(|data| {
                if let AtomData::UTF8(name) = data {
                    ingest_title_from(name, TitleKind::Work)
                } else {
                    None
                }
            });

        #[cfg(feature = "serato-markers")]
        let serato_tags = config
            .flags
            .contains(ImportTrackFlags::SERATO_MARKERS)
            .then(|| import_serato_markers(importer, ilst))
            .flatten();

        Self {
            advisory_rating,
            work_title,
            #[cfg(feature = "serato-markers")]
            serato_tags,
        }
    }

    fn finish(self, track: &mut Track) {
        let Self {
            advisory_rating,
            work_title,
            #[cfg(feature = "serato-markers")]
            serato_tags,
        } = self;

        debug_assert!(track.media_source.advisory_rating.is_none());
        track.media_source.advisory_rating = advisory_rating;

        if let Some(work_title) = work_title {
            let mut track_titles = track.titles.untie_replace(Default::default());
            debug_assert!(Titles::filter_kind(&track_titles, TitleKind::Work)
                .next()
                .is_none());
            track_titles.push(work_title);
            track.titles = Canonical::tie(track_titles);
        }

        #[cfg(feature = "serato-markers")]
        if let Some(serato_tags) = serato_tags {
            track.cues = Canonical::tie(crate::util::serato::import_cues(&serato_tags));
            track.color = crate::util::serato::import_track_color(&serato_tags);
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
) -> Result<bool> {
    // Export generic metadata
    let has_genre_text;
    let tag_ilst = {
        let mut tagged_file = TaggedFile::read_from(file, parse_options())?;
        let mut tag = tagged_file
            .remove(TagType::MP4ilst)
            .unwrap_or_else(|| Tag::new(TagType::MP4ilst));
        super::export_track_to_tag(&mut tag, config, track);
        has_genre_text = tag.get_string(&ItemKey::Genre).is_some();
        Ilst::from(tag)
    };

    // Post-processing: Export custom metadata
    let mut mp4_file = <Mp4File as AudioFile>::read_from(file, parse_options())?;
    let ilst = if let Some(ilst) = mp4_file.ilst_mut() {
        ilst
    } else {
        mp4_file.set_ilst(Default::default());
        mp4_file.ilst_mut().expect("ilst")
    };
    for atom in tag_ilst {
        ilst.replace_atom(atom);
    }

    // Preserve numeric legacy genres until overwritten by textual genres
    if has_genre_text {
        ilst.remove_atom(&LEGACY_GENRE_IDENT);
    }

    if let Some(advisory_rating) = track
        .media_source
        .advisory_rating
        .map(export_advisory_rating)
    {
        ilst.set_advisory_rating(advisory_rating);
    } else {
        ilst.remove_atom(&ADVISORY_RATING_IDENT);
    }

    // Music: Precise tempo BPM as a float value
    if let Some(formatted_bpm) = format_validated_tempo_bpm(
        &mut track.metrics.tempo_bpm,
        crate::util::TempoBpmFormat::Float,
    ) {
        let atom = Atom::new(FLOAT_BPM_IDENT, AtomData::UTF8(formatted_bpm));
        ilst.replace_atom(atom);
    } else {
        ilst.remove_atom(&FLOAT_BPM_IDENT);
    }

    // Additional titles not yet supported by lofty-rs
    if let Some(work) = Titles::filter_kind(track.titles.iter(), TitleKind::Work).next() {
        let atom = Atom::new(WORK_NAME_IDENT, AtomData::UTF8(work.name.clone()));
        ilst.insert_atom(atom);
    } else {
        ilst.remove_atom(&WORK_NAME_IDENT);
    }

    #[cfg(feature = "serato-markers")]
    if config.flags.contains(ExportTrackFlags::SERATO_MARKERS) {
        log::warn!("TODO: Export Serato markers");
    }

    let modified = {
        let mp4_file = <Mp4File as AudioFile>::read_from(file, parse_options())?;
        let old_ilst = mp4_file.ilst();
        Some(&*ilst) != old_ilst
    };
    if modified {
        mp4_file.save_to(file)?;
    }
    Ok(modified)
}

#[cfg(feature = "serato-markers")]
#[must_use]
fn import_serato_markers(
    importer: &mut Importer,
    ilst: &lofty::mp4::Ilst,
) -> Option<triseratops::tag::TagContainer> {
    let mut parsed = false;

    let mut serato_tags = triseratops::tag::TagContainer::new();

    if let Some(data) = ilst
        .atom(&SERATO_MARKERS_IDENT)
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
        .atom(&SERATO_MARKERS2_IDENT)
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
