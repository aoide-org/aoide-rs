// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use id3::{
    self,
    frame::{Comment, ExtendedText, PictureType},
    TagLike as _,
};
use num_traits::FromPrimitive as _;
use semval::IsValid as _;
use time::{Date, PrimitiveDateTime, Time};

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::{
        artwork::{ApicType, Artwork},
        content::ContentMetadata,
    },
    music::key::KeySignature,
    tag::{FacetId, FacetedTags, PlainTag, TagsMap},
    track::{
        actor::ActorRole,
        album::Kind as AlbumKind,
        metric::MetricsFlags,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_LANGUAGE, FACET_ID_MOOD,
        },
        title::{TitleKind, Titles},
        Track,
    },
    util::{
        canonical::Canonical,
        clock::{DateOrDateTime, DateYYYYMMDD, MonthType, YearType, YEAR_MAX, YEAR_MIN},
        string::trimmed_non_empty_from,
    },
};
use triseratops::tag::format::id3::ID3Tag;

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
        import::{ImportTrackConfig, ImportTrackFlags, Importer, TrackScope},
    },
    util::{
        format_valid_replay_gain, format_validated_tempo_bpm, ingest_title_from,
        push_next_actor_role_name_from,
        tag::{FacetedTagMappingConfig, TagMappingConfig},
        trim_readable, try_ingest_embedded_artwork_image,
    },
    Error, Result,
};

pub(crate) fn map_id3_err(err: id3::Error) -> Error {
    let id3::Error {
        kind,
        description,
        partial_tag,
    } = err;
    match kind {
        id3::ErrorKind::Io(err) => Error::Io(err),
        kind => Error::Other(anyhow::Error::from(id3::Error {
            kind,
            description,
            partial_tag,
        })),
    }
}

fn parse_timestamp(timestamp: id3::Timestamp) -> anyhow::Result<DateOrDateTime> {
    let year = timestamp.year;
    if year < i32::from(YEAR_MIN) || year > i32::from(YEAR_MAX) {
        anyhow::bail!("Year out of range in {timestamp:?}");
    }
    match (timestamp.month, timestamp.day) {
        (Some(month), Some(day)) => {
            if (1..=12).contains(&month) {
                let date =
                    Date::from_calendar_date(year, month.try_into().expect("valid month"), day);
                if let Ok(date) = date {
                    if let (Some(hour), Some(min), Some(sec)) =
                        (timestamp.hour, timestamp.minute, timestamp.second)
                    {
                        let time = Time::from_hms(hour, min, sec);
                        if let Ok(time) = time {
                            let date_time = DateOrDateTime::DateTime(
                                PrimitiveDateTime::new(date, time).assume_utc().into(),
                            );
                            debug_assert!(date_time.is_valid());
                            return Ok(date_time);
                        }
                    }
                    let dt = DateYYYYMMDD::from(date);
                    debug_assert!(dt.is_valid());
                    return Ok(dt.into());
                } else {
                    let dt = DateYYYYMMDD::from_year_month(year as YearType, month as MonthType);
                    debug_assert!(dt.is_valid());
                    return Ok(dt.into());
                }
            }
            let dt = DateYYYYMMDD::from_year(year as YearType);
            debug_assert!(dt.is_valid());
            Ok(dt.into())
        }
        (Some(month), None) => {
            let dt = if month > 0 && month <= 12 {
                DateYYYYMMDD::from_year_month(year as YearType, month as MonthType)
            } else {
                DateYYYYMMDD::from_year(year as YearType)
            };
            debug_assert!(dt.is_valid());
            Ok(dt.into())
        }
        _ => {
            let dt = DateYYYYMMDD::from_year(year as YearType);
            debug_assert!(dt.is_valid());
            Ok(dt.into())
        }
    }
}

fn text_frames<'a>(tag: &'a id3::Tag, frame_id: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    tag.frames()
        .filter(move |frame| frame.id() == frame_id)
        .filter_map(|frame| {
            if let id3::Content::Text(txt) = frame.content() {
                Some(txt.as_str())
            } else {
                None
            }
        })
        // All "T..."" text frames (except "TXXX") may contain multiple
        // values separated by a NULL character
        .flat_map(|txt| txt.split('\0'))
}

fn extended_text_values<'a>(
    tag: &'a id3::Tag,
    txxx_description: &'a str,
) -> impl Iterator<Item = &'a str> + 'a {
    tag.extended_texts().filter_map(move |txxx| {
        if txxx.description == txxx_description {
            Some(txxx.value.as_str())
        } else {
            None
        }
    })
}

fn first_text_frame<'a>(tag: &'a id3::Tag, frame_id: &'a str) -> Option<&'a str> {
    text_frames(tag, frame_id).next()
}

fn extended_texts<'a>(
    tag: &'a id3::Tag,
    description: &'a str,
) -> impl Iterator<Item = &'a str> + 'a {
    tag.extended_texts()
        .filter(move |txxx| txxx.description == description)
        .map(|txxx| txxx.value.as_str())
}

fn first_extended_text<'a>(tag: &'a id3::Tag, description: &'a str) -> Option<&'a str> {
    extended_texts(tag, description).next()
}

pub fn import_loudness(importer: &mut Importer, tag: &id3::Tag) -> Option<LoudnessLufs> {
    first_extended_text(tag, "REPLAYGAIN_TRACK_GAIN")
        .and_then(|text| importer.import_replay_gain(text))
}

#[must_use]
pub fn import_encoder(tag: &id3::Tag) -> Option<Cow<'_, str>> {
    first_text_frame(tag, "TENC").map(Into::into)
}

fn import_faceted_tags_from_text_frames(
    importer: &mut Importer,
    tags_map: &mut TagsMap,
    faceted_tag_mapping_config: &FacetedTagMappingConfig,
    facet_id: &FacetId,
    tag: &id3::Tag,
    frame_id: &str,
) -> usize {
    importer.import_faceted_tags_from_label_values(
        tags_map,
        faceted_tag_mapping_config,
        facet_id,
        text_frames(tag, frame_id).map(Into::into),
    )
}

#[must_use]
pub fn find_embedded_artwork_image(tag: &id3::Tag) -> Option<(ApicType, &str, &[u8])> {
    tag.pictures()
        .filter_map(|p| {
            if p.picture_type == PictureType::CoverFront {
                Some((ApicType::CoverFront, p))
            } else {
                None
            }
        })
        .chain(tag.pictures().filter_map(|p| {
            if p.picture_type == PictureType::Media {
                Some((ApicType::Media, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().filter_map(|p| {
            if p.picture_type == PictureType::Leaflet {
                Some((ApicType::Leaflet, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().filter_map(|p| {
            if p.picture_type == PictureType::Other {
                Some((ApicType::Other, p))
            } else {
                None
            }
        }))
        // otherwise take the first picture that could be parsed
        .chain(tag.pictures().map(|p| {
            (
                ApicType::from_u8(p.picture_type.into()).unwrap_or(ApicType::Other),
                p,
            )
        }))
        .map(|(apic_type, p)| (apic_type, p.mime_type.as_str(), p.data.as_slice()))
        .next()
}

pub fn import_timestamp_from_first_text_frame(
    importer: &mut Importer,
    tag: &id3::Tag,
    frame_id: &str,
) -> Option<DateOrDateTime> {
    first_text_frame(tag, frame_id).and_then(|text| {
        text.parse()
            .map_err(anyhow::Error::from)
            .and_then(parse_timestamp)
            .map_err(|err| {
                importer.add_issue(format!(
                    "Failed to parse ID3 time stamp from input '{text}' in text frame '{frame_id}': {err}",
                ));
            })
            .ok()
    })
}

pub fn import_album_kind(importer: &mut Importer, tag: &id3::Tag) -> Option<AlbumKind> {
    let value = first_text_frame(tag, "TCMP");
    value
        .and_then(|compilation| trim_readable(compilation).parse::<u8>().ok())
        .map(|compilation| match compilation {
            0 => AlbumKind::Unknown, // either Album or Single
            1 => AlbumKind::Compilation,
            _ => {
                importer.add_issue(format!(
                    "Unexpected tag value: TCMP = '{}'",
                    value.expect("unreachable")
                ));
                AlbumKind::Unknown
            }
        })
}

pub fn import_metadata_into_track(
    importer: &mut Importer,
    tag: &id3::Tag,
    config: &ImportTrackConfig,
    track: &mut Track,
) -> Result<()> {
    let mut tempo_bpm_non_fractional = false;
    if let Some(tempo_bpm) = first_extended_text(tag, "BPM")
        .and_then(|input| importer.import_tempo_bpm(input))
        // Alternative: Try "TEMPO" if "BPM" is missing or invalid
        .or_else(|| {
            first_extended_text(tag, "TEMPO").and_then(|input| importer.import_tempo_bpm(input))
        })
        // Fallback: Parse integer BPM
        .or_else(|| {
            tempo_bpm_non_fractional = true;
            first_text_frame(tag, "TBPM").and_then(|input| importer.import_tempo_bpm(input))
        })
    {
        track.metrics.tempo_bpm = Some(tempo_bpm);
        track.metrics.flags.set(
            MetricsFlags::TEMPO_BPM_NON_FRACTIONAL,
            tempo_bpm_non_fractional,
        );
    } else {
        // Reset
        track.metrics.tempo_bpm = None;
    }

    if let Some(key_signature) =
        first_text_frame(tag, "TKEY").and_then(|text| importer.import_key_signature(text))
    {
        track.metrics.key_signature = key_signature;
    } else {
        track.metrics.key_signature = KeySignature::unknown();
    }

    // Track titles
    let mut track_titles = Vec::with_capacity(4);
    if let Some(title) = tag
        .title()
        .and_then(|name| ingest_title_from(name, TitleKind::Main))
    {
        track_titles.push(title);
    }
    if let Some(title) =
        first_text_frame(tag, "TSST").and_then(|name| ingest_title_from(name, TitleKind::Sub))
    {
        track_titles.push(title);
    }
    if let Some(title) =
        first_text_frame(tag, "MVNM").and_then(|name| ingest_title_from(name, TitleKind::Movement))
    {
        track_titles.push(title);
    }
    let itunes_work_title = if config
        .flags
        .contains(ImportTrackFlags::COMPATIBILITY_ID3V2_ITUNES_GROUPING_MOVEMENT_WORK)
    {
        // Starting with iTunes 12.5.4 the "TIT1" text frame is used
        // for storing the work instead of the grouping. It is only
        // imported as a fallback if the legacy text frame WORK was empty
        // to prevent inconsistencies and for performing the migration to
        // iTunes tags.
        first_text_frame(tag, "TIT1").and_then(|name| ingest_title_from(name, TitleKind::Work))
    } else {
        None
    };
    let imported_work_from_itunes_tit1 = itunes_work_title.is_some();
    if let Some(title) = itunes_work_title.or_else(|| {
        first_extended_text(tag, "WORK")
            .and_then(|name| ingest_title_from(name, TitleKind::Work))
            .map(|title| {
                if config
                    .flags
                    .contains(ImportTrackFlags::COMPATIBILITY_ID3V2_ITUNES_GROUPING_MOVEMENT_WORK)
                {
                    importer.add_issue(format!(
                        "Imported work title '{}' from legacy ID3 text frame TXXX:WORK",
                        title.name
                    ));
                }
                title
            })
    }) {
        track_titles.push(title);
    }
    track.titles = importer.finish_import_of_titles(TrackScope::Track, track_titles);

    // Track actors
    let mut track_actors = Vec::with_capacity(8);
    if let Some(name) = tag.artist() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Artist, name);
    }
    for name in text_frames(tag, "TCOM") {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Composer, name);
    }
    for name in text_frames(tag, "TPE3") {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Conductor, name);
    }
    for name in extended_text_values(tag, "DIRECTOR") {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Director, name);
    }
    for name in text_frames(tag, "TPE4") {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Remixer, name);
    }
    for name in text_frames(tag, "TEXT") {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Lyricist, name);
    }
    for name in extended_text_values(tag, "Writer") {
        // "Writer", not "WRITER" in all caps
        // See also: https://tickets.metabrainz.org/browse/PICARD-1101
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Writer, name);
    }
    // TODO: Import TIPL frames
    track.actors = importer.finish_import_of_actors(TrackScope::Track, track_actors);

    let mut album = track.album.untie_replace(Default::default());

    // Album titles
    let mut album_titles = Vec::with_capacity(1);
    if let Some(title) = tag
        .album()
        .and_then(|name| ingest_title_from(name, TitleKind::Main))
    {
        album_titles.push(title);
    }
    album.titles = importer.finish_import_of_titles(TrackScope::Album, album_titles);

    // Album actors
    let mut album_actors = Vec::with_capacity(4);
    if let Some(name) = tag.album_artist() {
        push_next_actor_role_name_from(&mut album_actors, ActorRole::Artist, name);
    }
    album.actors = importer.finish_import_of_actors(TrackScope::Album, album_actors);

    // Album properties
    if let Some(album_kind) = import_album_kind(importer, tag) {
        album.kind = album_kind;
    } else {
        album.kind = AlbumKind::Unknown;
    }

    track.album = Canonical::tie(album);

    track.recorded_at = import_timestamp_from_first_text_frame(importer, tag, "TDRC");
    track.released_at = import_timestamp_from_first_text_frame(importer, tag, "TDRL");
    track.released_orig_at = import_timestamp_from_first_text_frame(importer, tag, "TDOR");

    track.publisher = first_text_frame(tag, "TPUB")
        .and_then(trimmed_non_empty_from)
        .map(Into::into);
    track.copyright = first_text_frame(tag, "TCOP")
        .and_then(trimmed_non_empty_from)
        .map(Into::into);

    let mut tags_map = TagsMap::default();

    // Grouping tags
    // Apple decided to store the Work in the traditional ID3v2 Content Group
    // frame (TIT1) and introduced new Grouping (GRP1) and Movement Name (MVNM)
    // frames.
    // https://discussions.apple.com/thread/7900430
    // http://blog.jthink.net/2016/11/the-reason-why-is-grouping-field-no.html
    if import_faceted_tags_from_text_frames(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_GROUPING,
        tag,
        "GRP1",
    ) > 0
    {
        if !imported_work_from_itunes_tit1 {
            importer.add_issue("Imported grouping tag(s) from ID3 text frame GRP1 instead of TIT1");
        }
    } else {
        // Use the legacy/classical text frame only as a fallback
        if import_faceted_tags_from_text_frames(
            importer,
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_GROUPING,
            tag,
            "TIT1",
        ) > 0
            && config
                .flags
                .contains(ImportTrackFlags::COMPATIBILITY_ID3V2_ITUNES_GROUPING_MOVEMENT_WORK)
        {
            importer.add_issue("Imported grouping tag(s) from ID3 text frame TIT1 instead of GRP1");
        }
    }

    // Import gigtags from raw grouping tags before any other tags.
    #[cfg(feature = "gigtags")]
    if config.flags.contains(ImportTrackFlags::GIGTAGS) {
        if let Some(faceted_tags) = tags_map.take_faceted_tags(&FACET_ID_GROUPING) {
            tags_map = crate::util::gigtags::import_from_faceted_tags(faceted_tags);
        }
    }

    // Comment tag
    let comments = tag
        .comments()
        .filter(|comm| comm.description.trim().is_empty())
        .map(|comm| comm.text.to_owned());
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_COMMENT,
        comments.map(Into::into),
    );

    // Description tag
    let descriptions = tag
        .comments()
        .filter(|comm| comm.description.trim().to_lowercase() == "description")
        .map(|comm| comm.text.to_owned());
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_DESCRIPTION,
        descriptions.map(Into::into),
    );

    // Genre tags
    import_faceted_tags_from_text_frames(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_GENRE,
        tag,
        "TCON",
    );

    // Mood tags
    import_faceted_tags_from_text_frames(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_MOOD,
        tag,
        "TMOO",
    );

    // ISRC tag
    import_faceted_tags_from_text_frames(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_ISRC,
        tag,
        "TSRC",
    );

    // Language tag
    import_faceted_tags_from_text_frames(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_LANGUAGE,
        tag,
        "TLAN",
    );

    debug_assert!(track.tags.is_empty());
    track.tags = Canonical::tie(tags_map.into());

    // Indexes (in pairs)
    if tag.track().is_some() || tag.total_tracks().is_some() {
        track.indexes.track.number = tag.track().map(|i| (i & 0xFFFF) as u16);
        track.indexes.track.total = tag.total_tracks().map(|i| (i & 0xFFFF) as u16);
    } else {
        track.indexes.track = Default::default();
    }
    if tag.disc().is_some() || tag.total_discs().is_some() {
        track.indexes.disc.number = tag.disc().map(|i| (i & 0xFFFF) as u16);
        track.indexes.disc.total = tag.total_discs().map(|i| (i & 0xFFFF) as u16);
    } else {
        track.indexes.disc = Default::default();
    }
    if let Some(movement) = first_text_frame(tag, "MVIN")
        .and_then(|text| importer.import_index_numbers_from_field("MVIN", text))
    {
        track.indexes.movement = movement;
    } else {
        // Reset
        track.indexes.movement = Default::default();
    }

    // Artwork
    if config
        .flags
        .contains(ImportTrackFlags::METADATA_EMBEDDED_ARTWORK)
    {
        let artwork =
            if let Some((apic_type, media_type, image_data)) = find_embedded_artwork_image(tag) {
                let (artwork, _, issues) = try_ingest_embedded_artwork_image(
                    apic_type,
                    image_data,
                    None,
                    Some(media_type.to_owned()),
                    &mut config.flags.new_artwork_digest(),
                );
                issues
                    .into_iter()
                    .for_each(|message| importer.add_issue(message));
                artwork
            } else {
                Artwork::Missing
            };
        track.media_source.artwork = Some(artwork);
    }

    #[cfg(feature = "serato-markers")]
    #[allow(clippy::blocks_in_if_conditions)]
    if config.flags.contains(ImportTrackFlags::SERATO_MARKERS) {
        let mut serato_tags = triseratops::tag::TagContainer::new();
        let mut parsed = false;
        for geob in tag.encapsulated_objects() {
            if match geob.description.as_str() {
                triseratops::tag::Markers::ID3_TAG => serato_tags
                    .parse_markers(&geob.data, triseratops::tag::TagFormat::ID3)
                    .map_err(|err| {
                        importer.add_issue(format!("Failed to parse Serato Markers: {err}"));
                    })
                    .is_ok(),
                triseratops::tag::Markers2::ID3_TAG => serato_tags
                    .parse_markers2(&geob.data, triseratops::tag::TagFormat::ID3)
                    .map_err(|err| {
                        importer.add_issue(format!("Failed to parse Serato Markers2: {err}"));
                    })
                    .is_ok(),
                _ => false,
            } {
                parsed = true;
            }
        }
        if parsed {
            track.cues = Canonical::tie(crate::util::serato::import_cues(&serato_tags));
            track.color = crate::util::serato::import_track_color(&serato_tags);
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum ExportError {
    UnsupportedLegacyVersion(id3::Version),
}

pub fn export_track(
    config: &ExportTrackConfig,
    track: &mut Track,
    tag: &mut id3::Tag,
) -> std::result::Result<(), ExportError> {
    if tag.version() != id3::Version::Id3v24 {
        return Err(ExportError::UnsupportedLegacyVersion(tag.version()));
    }

    // Audio properties
    match &track.media_source.content_metadata {
        ContentMetadata::Audio(audio) => {
            if let Some(formatted_track_gain) = audio.loudness.and_then(format_valid_replay_gain) {
                tag.add_frame(ExtendedText {
                    description: "REPLAYGAIN_TRACK_GAIN".to_owned(),
                    value: formatted_track_gain,
                });
            } else {
                tag.remove_extended_text(Some("REPLAYGAIN_TRACK_GAIN"), None);
            }
            if let Some(encoder) = &audio.encoder {
                tag.set_text("TENC", encoder)
            } else {
                tag.remove("TENC");
            }
        }
    }

    // Music: Tempo/BPM
    tag.remove_extended_text(Some("TEMPO"), None);
    if let Some(formatted_bpm) = format_validated_tempo_bpm(&mut track.metrics.tempo_bpm) {
        tag.add_frame(ExtendedText {
            description: "BPM".to_owned(),
            value: formatted_bpm,
        });
        tag.set_text(
            "TBPM",
            track
                .metrics
                .tempo_bpm
                .expect("valid bpm")
                .to_inner()
                .round()
                .to_string(),
        );
    } else {
        tag.remove_extended_text(Some("BPM"), None);
        tag.remove("TBPM");
    }

    // Music: Key
    if track.metrics.key_signature.is_unknown() {
        tag.remove("TKEY");
    } else {
        // TODO: Write a custom key code string according to config
        tag.set_text("TKEY", track.metrics.key_signature.to_string());
    }

    // Track titles
    if let Some(title) = Titles::main_title(track.titles.iter()) {
        tag.set_title(title.name.to_owned());
    } else {
        tag.remove_title();
    }
    tag.set_text_values(
        "TIT3",
        Titles::filter_kind(track.titles.iter(), TitleKind::Sub).map(|title| &title.name),
    );
    tag.set_text_values(
        "MVNM",
        Titles::filter_kind(track.titles.iter(), TitleKind::Movement).map(|title| &title.name),
    );
    tag.remove_extended_text(Some("WORK"), None);
    if config
        .flags
        .contains(ExportTrackFlags::COMPATIBILITY_ID3V2_ITUNES_GROUPING_MOVEMENT_WORK)
    {
        tag.set_text_values(
            "TIT1",
            Titles::filter_kind(track.titles.iter(), TitleKind::Work).map(|title| &title.name),
        );
    } else if let Some(joined_titles) = TagMappingConfig::join_labels_with_separator(
        Titles::filter_kind(track.titles.iter(), TitleKind::Work).map(|title| title.name.as_str()),
        ID3V24_MULTI_FIELD_SEPARATOR,
    ) {
        tag.add_frame(ExtendedText {
            description: "WORK".to_owned(),
            value: joined_titles.into_owned(),
        });
    }

    // Track actors
    export_filtered_actor_names(
        tag,
        "TPE1",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Artist),
    );
    export_filtered_actor_names(
        tag,
        "TCOM",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Composer),
    );
    export_filtered_actor_names(
        tag,
        "TPE3",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Conductor),
    );
    export_filtered_actor_names_txxx(
        tag,
        "DIRECTOR",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Director),
    );
    export_filtered_actor_names(
        tag,
        "TPE4",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Remixer),
    );
    export_filtered_actor_names(
        tag,
        "TEXT",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Lyricist),
    );
    // "Writer", not "WRITER" in all caps
    // See also: https://tickets.metabrainz.org/browse/PICARD-1101
    export_filtered_actor_names_txxx(
        tag,
        "Writer",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Writer),
    );
    // TODO: Export TIPL frames

    // Album
    if let Some(title) = Titles::main_title(track.album.titles.iter()) {
        tag.set_album(title.name.to_owned());
    } else {
        tag.remove_album();
    }
    export_filtered_actor_names(
        tag,
        "TPE2",
        FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
    );
    match track.album.kind {
        AlbumKind::Unknown => {
            tag.remove("TCMP");
        }
        AlbumKind::Compilation => {
            tag.set_text("TCMP", "1");
        }
        AlbumKind::Album | AlbumKind::Single => {
            tag.set_text("TCMP", "0");
        }
    }

    if let Some(recorded_at) = &track.recorded_at {
        let timestamp = export_date_or_date_time(*recorded_at);
        tag.set_text("TDRC", timestamp.to_string());
    } else {
        tag.remove("TDRC");
    }
    if let Some(released_at) = &track.released_at {
        let timestamp = export_date_or_date_time(*released_at);
        tag.set_text("TDRL", timestamp.to_string());
    } else {
        tag.remove("TDRL");
    }
    if let Some(released_orig_at) = &track.released_orig_at {
        let timestamp = export_date_or_date_time(*released_orig_at);
        tag.set_text("TDOR", timestamp.to_string());
    } else {
        tag.remove("TDOR");
    }

    // Publishing info
    if let Some(publisher) = &track.publisher {
        tag.set_text("TPUB", publisher);
    } else {
        tag.remove("TPUB");
    }
    if let Some(copyright) = &track.copyright {
        tag.set_text("TCOP", copyright);
    } else {
        tag.remove("TCOP");
    }

    // Numbers
    if let Some(track_number) = track.indexes.track.number {
        tag.set_track(track_number.into());
    } else {
        tag.remove_track();
    }
    if let Some(track_total) = track.indexes.track.total {
        tag.set_total_tracks(track_total.into());
    } else {
        tag.remove_total_tracks();
    }
    if let Some(disc_number) = track.indexes.disc.number {
        tag.set_disc(disc_number.into());
    } else {
        tag.remove_disc();
    }
    if let Some(disc_total) = track.indexes.disc.total {
        tag.set_total_discs(disc_total.into());
    } else {
        tag.remove_total_discs();
    }
    if let Some(movement_number) = track.indexes.movement.number {
        if let Some(movement_total) = track.indexes.movement.total {
            tag.set_text("MVIN", format!("{movement_number}/{movement_total}"));
        } else {
            tag.set_text("MVIN", movement_number.to_string());
        }
    } else if let Some(movement_total) = track.indexes.movement.total {
        tag.set_text("MVIN", format!("/{movement_total}"));
    } else {
        tag.remove("MVIN");
    }

    // Export selected tags into dedicated fields
    let mut tags_map = TagsMap::from(track.tags.clone().untie());

    // Comment(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_COMMENT) {
        export_faceted_tags_comment(
            tag,
            String::new(),
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags_comment(tag, String::new(), None, &[]);
    }

    // Description(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_DESCRIPTION)
    {
        export_faceted_tags_comment(
            tag,
            "description",
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags_comment(tag, "description", None, &[]);
    }

    // Genre(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_GENRE) {
        export_faceted_tags(
            tag,
            "TCON",
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags(tag, "TCON", None, &[]);
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_MOOD) {
        export_faceted_tags(
            tag,
            "TMOO",
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags(tag, "TMOO", None, &[]);
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_ISRC) {
        export_faceted_tags(
            tag,
            "TSRC",
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags(tag, "TSRC", None, &[]);
    }

    // Language(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_LANGUAGE) {
        export_faceted_tags(
            tag,
            "TLAN",
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags(tag, "TLAN", None, &[]);
    }

    // Grouping(s)
    {
        let facet_id = &FACET_ID_GROUPING;
        let mut tags = tags_map
            .take_faceted_tags(facet_id)
            .map(|FacetedTags { facet_id: _, tags }| tags)
            .unwrap_or_default();
        #[cfg(feature = "gigtags")]
        if config.flags.contains(ExportTrackFlags::GIGTAGS) {
            if let Err(err) = crate::util::gigtags::export_and_encode_remaining_tags_into(
                tags_map.into(),
                &mut tags,
            ) {
                log::error!("Failed to export gigitags: {err}");
            }
        }
        let grouping_frame_id = if config
            .flags
            .contains(ExportTrackFlags::COMPATIBILITY_ID3V2_ITUNES_GROUPING_MOVEMENT_WORK)
        {
            "GRP1"
        } else {
            tag.remove("GRP1");
            "TIT1"
        };
        if tags.is_empty() {
            export_faceted_tags(tag, grouping_frame_id, None, &[]);
        } else {
            export_faceted_tags(
                tag,
                grouping_frame_id,
                config.faceted_tag_mapping.get(facet_id.value()),
                &tags,
            );
        }
    }

    Ok(())
}

fn export_date_or_date_time(dt: DateOrDateTime) -> id3::Timestamp {
    match dt {
        DateOrDateTime::Date(date) => {
            if date.is_year() {
                id3::Timestamp {
                    year: date.year() as _,
                    month: None,
                    day: None,
                    hour: None,
                    minute: None,
                    second: None,
                }
            } else {
                id3::Timestamp {
                    year: date.year() as _,
                    month: Some(date.month() as _),
                    day: Some(date.day_of_month() as _),
                    hour: None,
                    minute: None,
                    second: None,
                }
            }
        }
        DateOrDateTime::DateTime(date_time) => {
            let date_time = date_time.to_inner();
            id3::Timestamp {
                year: date_time.date().year(),
                month: Some(date_time.date().month() as _),
                day: Some(date_time.date().day() as _),
                hour: Some(date_time.time().hour() as _),
                minute: Some(date_time.time().minute() as _),
                second: Some(date_time.time().second() as _),
            }
        }
    }
}

fn export_filtered_actor_names(
    tag: &mut id3::Tag,
    text_frame_id: impl AsRef<str>,
    actor_names: FilteredActorNames<'_>,
) {
    match actor_names {
        FilteredActorNames::Summary(name) => {
            tag.set_text(text_frame_id, name);
        }
        FilteredActorNames::Primary(names) => {
            tag.set_text_values(text_frame_id, names);
        }
    }
}

fn export_filtered_actor_names_txxx(
    tag: &mut id3::Tag,
    txxx_description: impl AsRef<str>,
    actor_names: FilteredActorNames<'_>,
) {
    tag.remove_extended_text(Some(txxx_description.as_ref()), None);
    match actor_names {
        FilteredActorNames::Summary(name) => {
            tag.add_frame(ExtendedText {
                description: txxx_description.as_ref().to_owned(),
                value: name.to_owned(),
            });
        }
        FilteredActorNames::Primary(names) => {
            if let Some(joined_names) = TagMappingConfig::join_labels_with_separator(
                names.iter().copied(),
                ID3V24_MULTI_FIELD_SEPARATOR,
            ) {
                tag.add_frame(ExtendedText {
                    description: txxx_description.as_ref().to_owned(),
                    value: joined_names.into_owned(),
                });
            }
        }
    }
}

const ID3V24_MULTI_FIELD_SEPARATOR: &str = "\0";

fn export_faceted_tags(
    tag: &mut id3::Tag,
    text_frame_id: impl AsRef<str>,
    config: Option<&TagMappingConfig>,
    tags: &[PlainTag],
) {
    let joined_labels = if let Some(config) = config {
        config.join_labels(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
        )
    } else {
        TagMappingConfig::join_labels_with_separator(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
            ID3V24_MULTI_FIELD_SEPARATOR,
        )
    };
    if let Some(joined_labels) = joined_labels {
        tag.set_text(text_frame_id, joined_labels);
    } else {
        tag.remove(text_frame_id);
    }
}

fn export_faceted_tags_comment(
    tag: &mut id3::Tag,
    description: impl Into<String>,
    config: Option<&TagMappingConfig>,
    tags: &[PlainTag],
) {
    let joined_labels = if let Some(config) = config {
        config.join_labels(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
        )
    } else {
        TagMappingConfig::join_labels_with_separator(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
            ID3V24_MULTI_FIELD_SEPARATOR,
        )
    };
    if let Some(joined_labels) = joined_labels {
        let description = description.into();
        tag.remove_comment(Some(&description), None);
        let comment = Comment {
            lang: String::new(),
            description,
            text: joined_labels.into(),
        };
        tag.add_frame(comment);
    }
}
