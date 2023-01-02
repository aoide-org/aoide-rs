// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use lofty::mp4::{AtomData, AtomIdent, Mp4File};

use aoide_core::{
    media::AdvisoryRating,
    track::{actor::Role as ActorRole, album::Kind as AlbumKind, tag::FACET_ID_XID, Track},
    util::canonical::Canonical,
};

use crate::{
    io::import::{ImportTrackConfig, ImportTrackFlags, Importer},
    util::push_next_actor_role_name,
    Result,
};

const COMPILATION_IDENT: AtomIdent<'_> = AtomIdent::Fourcc(*b"cpil");

// FIXME: Remove after <https://github.com/Serial-ATA/lofty-rs/pull/100>
const DIRECTOR_IDENT: AtomIdent<'_> = AtomIdent::Fourcc(*b"\xA9dir");

// FIXME: Remove after <https://github.com/Serial-ATA/lofty-rs/pull/98> has been merged.
const XID_IDENT: AtomIdent<'_> = AtomIdent::Fourcc(*b"xid ");

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

pub fn import_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    mp4_file: Mp4File,
    track: &mut Track,
) -> Result<()> {
    // Pre-processing

    // TODO: Handle in generic import
    // See also: <https://github.com/Serial-ATA/lofty-rs/issues/99>
    let advisory_rating = mp4_file
        .ilst()
        .and_then(|ilst| ilst.advisory_rating().map(import_advisory_rating));

    // TODO: How to use ItemKey::FlagCompilation for reading this boolean?
    // See also: <https://github.com/Serial-ATA/lofty-rs/issues/99>
    let album_kind = mp4_file.ilst().and_then(|ilst| {
        ilst.atom(&COMPILATION_IDENT).and_then(|atom| {
            atom.data()
                .filter_map(|data| match data {
                    AtomData::Bool(value) => Some(if *value {
                        AlbumKind::Compilation
                    } else {
                        AlbumKind::NoCompilation
                    }),
                    _ => None,
                })
                .next()
        })
    });

    let director_names = mp4_file.ilst().and_then(|ilst| {
        ilst.atom(&DIRECTOR_IDENT).map(|atom| {
            atom.data()
                .filter_map(|data| match data {
                    AtomData::UTF8(value) => Some(value.to_owned()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
    });

    let xid_tags = mp4_file.ilst().and_then(|ilst| {
        ilst.atom(&XID_IDENT).map(|atom| {
            atom.data()
                .filter_map(|data| match data {
                    AtomData::UTF8(value) => Some(value.to_owned()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
    });

    #[cfg(feature = "serato-markers")]
    let serato_tags = if config.flags.contains(ImportTrackFlags::SERATO_MARKERS) {
        mp4_file
            .ilst()
            .and_then(|ilst| import_serato_markers(importer, ilst))
    } else {
        None
    };

    // Generic import
    let tagged_file = mp4_file.into();
    super::import_tagged_file_into_track(importer, config, tagged_file, track)?;

    // Post-processing

    debug_assert!(track.media_source.advisory_rating.is_none());
    track.media_source.advisory_rating = advisory_rating;

    if let Some(album_kind) = album_kind {
        let mut album = track.album.untie_replace(Default::default());
        debug_assert!(album.kind.is_none());
        album.kind = Some(album_kind);
        track.album = Canonical::tie(album);
    }

    if let Some(director_names) = director_names {
        let mut track_actors = track.actors.untie_replace(Default::default());
        for name in director_names {
            push_next_actor_role_name(&mut track_actors, ActorRole::Director, name);
        }
        track.actors = Canonical::tie(track_actors);
    }

    if let Some(xid_tags) = xid_tags {
        let mut tags_map = track.tags.untie_replace(Default::default()).into();
        importer.import_faceted_tags_from_label_values(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_XID,
            xid_tags.into_iter().map(Into::into),
        );
        track.tags = Canonical::tie(tags_map.into());
    }

    #[cfg(feature = "serato-markers")]
    if let Some(serato_tags) = serato_tags {
        track.cues = Canonical::tie(crate::util::serato::import_cues(&serato_tags));
        track.color = crate::util::serato::import_track_color(&serato_tags);
    }

    Ok(())
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
