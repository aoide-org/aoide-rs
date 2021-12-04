// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{borrow::Cow, time::Duration};

use chrono::{Datelike as _, NaiveDate, NaiveDateTime, NaiveTime, Timelike as _, Utc};
use id3::{
    self,
    frame::{Comment, PictureType},
};
use mime::Mime;
use num_traits::FromPrimitive as _;
use semval::IsValid as _;
use triseratops::tag::{
    format::id3::ID3Tag, Markers as SeratoMarkers, Markers2 as SeratoMarkers2,
    TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat,
};

use aoide_core::{
    audio::AudioContent,
    media::{concat_encoder_properties, ApicType, Artwork, Content, ContentMetadataFlags},
    tag::{FacetId, FacetedTags, PlainTag, Tags, TagsMap},
    track::{
        actor::ActorRole,
        album::AlbumKind,
        metric::MetricsFlags,
        release::DateOrDateTime,
        tag::{FACET_COMMENT, FACET_GENRE, FACET_GROUPING, FACET_ISRC, FACET_LANGUAGE, FACET_MOOD},
        title::{Title, TitleKind, Titles},
        Track,
    },
    util::{
        clock::{DateTime, DateYYYYMMDD, MonthType, YearType},
        Canonical, CanonicalizeInto as _,
    },
};

use aoide_core_serde::tag::Tags as SerdeTags;

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
        import::{ImportTrackConfig, ImportTrackFlags},
    },
    util::{
        digest::MediaDigest,
        format_valid_replay_gain, format_validated_tempo_bpm, parse_index_numbers,
        parse_key_signature, parse_replay_gain, parse_tempo_bpm, push_next_actor_role_name, serato,
        tag::{
            import_faceted_tags_from_label_value_iter, FacetedTagMappingConfig, TagMappingConfig,
        },
        try_load_embedded_artwork,
    },
    Error, Result,
};

pub(crate) fn map_err(err: id3::Error) -> Error {
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

fn parse_timestamp(timestamp: id3::Timestamp) -> DateOrDateTime {
    match (timestamp.month, timestamp.day) {
        (Some(month), Some(day)) => {
            let date = NaiveDate::from_ymd_opt(timestamp.year, month.into(), day.into());
            if let Some(date) = date {
                if let (Some(hour), Some(min), Some(sec)) =
                    (timestamp.hour, timestamp.minute, timestamp.second)
                {
                    let time = NaiveTime::from_hms_opt(hour.into(), min.into(), sec.into());
                    if let Some(time) = time {
                        return DateTime::from(chrono::DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(date, time),
                            Utc,
                        ))
                        .into();
                    }
                }
                DateYYYYMMDD::from(date).into()
            } else if month > 0 && month <= 12 {
                DateYYYYMMDD::from_year_month(timestamp.year as YearType, month as MonthType).into()
            } else {
                DateYYYYMMDD::from_year(timestamp.year as YearType).into()
            }
        }
        (Some(month), None) => {
            if month > 0 && month <= 12 {
                DateYYYYMMDD::from_year_month(timestamp.year as YearType, month as MonthType).into()
            } else {
                DateYYYYMMDD::from_year(timestamp.year as YearType).into()
            }
        }
        _ => DateYYYYMMDD::from_year(timestamp.year as YearType).into(),
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
    id3_tag: &'a id3::Tag,
    txxx_description: &'a str,
) -> impl Iterator<Item = &'a str> + 'a {
    id3_tag.extended_texts().filter_map(move |txxx| {
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

fn import_faceted_tags_from_text_frames(
    tags_map: &mut TagsMap,
    faceted_tag_mapping_config: &FacetedTagMappingConfig,
    facet_id: &FacetId,
    tag: &id3::Tag,
    frame_id: &str,
) -> usize {
    import_faceted_tags_from_label_value_iter(
        tags_map,
        faceted_tag_mapping_config,
        facet_id,
        text_frames(tag, frame_id).map(ToOwned::to_owned),
    )
}

pub fn import_track(
    tag: &id3::Tag,
    mut audio_content: AudioContent,
    config: &ImportTrackConfig,
    track: &mut Track,
) -> Result<()> {
    let metadata_flags = if audio_content.duration.is_some() {
        // Accurate duration
        ContentMetadataFlags::RELIABLE
    } else {
        audio_content.duration = tag
            .duration()
            .map(|secs| Duration::from_secs(u64::from(secs)).into());
        ContentMetadataFlags::UNRELIABLE
    };
    if track
        .media_source
        .content_metadata_flags
        .update(metadata_flags)
    {
        let loudness =
            first_extended_text(tag, "REPLAYGAIN_TRACK_GAIN").and_then(parse_replay_gain);
        let encoder =
            concat_encoder_properties(first_text_frame(tag, "TENC"), first_text_frame(tag, "TSSE"))
                .map(Cow::into_owned);
        audio_content = AudioContent {
            loudness,
            encoder,
            ..audio_content
        };
        track.media_source.content = Content::Audio(audio_content);
    }

    let mut tempo_bpm_non_fractional = false;
    if let Some(tempo_bpm) = first_extended_text(tag, "BPM")
        .and_then(parse_tempo_bpm)
        // Alternative: Try "TEMPO" if "BPM" is missing or invalid
        .or_else(|| first_extended_text(tag, "TEMPO").and_then(parse_tempo_bpm))
        // Fallback: Parse integer BPM
        .or_else(|| {
            tempo_bpm_non_fractional = true;
            first_text_frame(tag, "TBPM").and_then(parse_tempo_bpm)
        })
    {
        debug_assert!(tempo_bpm.is_valid());
        track.metrics.tempo_bpm = Some(tempo_bpm);
        track.metrics.flags.set(
            MetricsFlags::TEMPO_BPM_NON_FRACTIONAL,
            tempo_bpm_non_fractional,
        );
    }

    if let Some(key_signature) = first_text_frame(tag, "TKEY").and_then(parse_key_signature) {
        track.metrics.key_signature = key_signature;
    }

    // Track titles
    let mut track_titles = Vec::with_capacity(4);
    if let Some(name) = tag.title() {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Main,
        };
        track_titles.push(title);
    }
    if let Some(name) = first_text_frame(tag, "TSST") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Sub,
        };
        track_titles.push(title);
    }
    if let Some(name) = first_text_frame(tag, "MVNM") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Movement,
        };
        track_titles.push(title);
    }
    let mut work_name = if let Some(name) = first_extended_text(tag, "WORK") {
        if config
            .flags
            .contains(ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK)
        {
            if name.trim().is_empty() {
                None
            } else {
                tracing::warn!(
                    "Imported work title '{}' from legacy ID3 text frame TXXX:WORK",
                    name
                );
                Some(name)
            }
        } else {
            Some(name)
        }
    } else {
        None
    };
    let mut imported_work_from_itunes_tit1 = false;
    work_name = work_name.or_else(|| {
        if config
            .flags
            .contains(ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK)
        {
            // Starting with iTunes 12.5.4 the "TIT1" text frame is used
            // for storing the work instead of the grouping. It is only
            // imported as a fallback if the legacy text frame WORK was empty
            // to prevent inconsistencies and for performing the migration to
            // iTunes tags.
            imported_work_from_itunes_tit1 = true;
            first_text_frame(tag, "TIT1")
        } else {
            None
        }
    });
    if let Some(name) = work_name {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Work,
        };
        track_titles.push(title);
    }
    let track_titles = track_titles.canonicalize_into();
    if !track_titles.is_empty() {
        track.titles = Canonical::tie(track_titles);
    }

    // Track actors
    let mut track_actors = Vec::with_capacity(8);
    if let Some(name) = tag.artist() {
        push_next_actor_role_name(&mut track_actors, ActorRole::Artist, name.to_owned());
    }
    for name in text_frames(tag, "TCOM") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Composer, name.to_owned());
    }
    for name in text_frames(tag, "TPE3") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name.to_owned());
    }
    for name in extended_text_values(tag, "DIRECTOR") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Director, name.to_owned());
    }
    for name in text_frames(tag, "TPE4") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Remixer, name.to_owned());
    }
    for name in text_frames(tag, "TEXT") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Lyricist, name.to_owned());
    }
    for name in extended_text_values(tag, "Writer") {
        // "Writer", not "WRITER" in all caps
        // See also: https://tickets.metabrainz.org/browse/PICARD-1101
        push_next_actor_role_name(&mut track_actors, ActorRole::Writer, name.to_owned());
    }
    // TODO: Import TIPL frames
    let track_actors = track_actors.canonicalize_into();
    if !track_actors.is_empty() {
        track.actors = Canonical::tie(track_actors);
    }

    let mut album = track.album.untie_replace(Default::default());

    // Album titles
    let mut album_titles = Vec::with_capacity(1);
    if let Some(name) = tag.album() {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Main,
        };
        album_titles.push(title);
    }
    let album_titles = album_titles.canonicalize_into();
    if !album_titles.is_empty() {
        album.titles = Canonical::tie(album_titles);
    }

    // Album actors
    let mut album_actors = Vec::with_capacity(4);
    if let Some(name) = tag.album_artist() {
        push_next_actor_role_name(&mut album_actors, ActorRole::Artist, name.to_owned());
    }
    let album_actors = album_actors.canonicalize_into();
    if !album_actors.is_empty() {
        album.actors = Canonical::tie(album_actors);
    }

    // Album properties
    album.kind = first_text_frame(tag, "TCMP")
        .map(|tcmp| tcmp.parse::<u8>())
        .transpose()
        .map_err(anyhow::Error::from)?
        .map(|tcmp| match tcmp {
            0 => AlbumKind::Unknown, // either album or single
            1 => AlbumKind::Compilation,
            _ => {
                tracing::warn!("Unexpected iTunes compilation tag: TCMP = {}", tcmp);
                AlbumKind::Unknown
            }
        })
        .unwrap_or_default();

    track.album = Canonical::tie(album);

    // Release properties
    // Instead of the release date "TDRL" most applications use the recording date "TDRC".
    // See also https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html
    if let Some(released_at) = tag
        .date_released()
        .or_else(|| tag.date_recorded())
        .map(parse_timestamp)
    {
        track.release.released_at = Some(released_at);
    }
    if let Some(label) = first_text_frame(tag, "TPUB") {
        track.release.released_by = Some(label.to_owned());
    }
    if let Some(copyright) = first_text_frame(tag, "TCOP") {
        track.release.copyright = Some(copyright.to_owned());
    }

    let mut tags_map = TagsMap::default();
    if config.flags.contains(ImportTrackFlags::MIXXX_CUSTOM_TAGS) {
        for geob in tag
            .encapsulated_objects()
            .filter(|geob| geob.description == "Mixxx CustomTags")
        {
            if geob
                .mime_type
                .parse::<Mime>()
                .ok()
                .as_ref()
                .map(Mime::type_)
                != Some(mime::APPLICATION_JSON.type_())
            {
                tracing::warn!(
                    "Unexpected MIME type for GEOB '{}': {}",
                    geob.description,
                    geob.mime_type
                );
                continue;
            }
            if let Some(custom_tags) = serde_json::from_slice::<SerdeTags>(&geob.data)
                .map_err(|err| {
                    tracing::warn!("Failed to parse Mixxx custom tags: {}", err);
                    err
                })
                .ok()
                .map(Tags::from)
            {
                // Initialize map with all existing custom tags as starting point
                debug_assert_eq!(0, tags_map.total_count());
                tags_map = custom_tags.into();
            }
        }
    }

    // Comment tag
    let comments = tag
        .comments()
        .filter(|comm| comm.description.is_empty())
        .map(|comm| comm.text.to_owned());
    import_faceted_tags_from_label_value_iter(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_COMMENT,
        comments,
    );

    // Genre tags
    import_faceted_tags_from_text_frames(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_GENRE,
        tag,
        "TCON",
    );

    // Mood tags
    import_faceted_tags_from_text_frames(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_MOOD,
        tag,
        "TMOO",
    );

    // Grouping tags
    // Apple decided to store the Work in the traditional ID3v2 Content Group
    // frame (TIT1) and introduced new Grouping (GRP1) and Movement Name (MVNM)
    // frames.
    // https://discussions.apple.com/thread/7900430
    // http://blog.jthink.net/2016/11/the-reason-why-is-grouping-field-no.html
    if import_faceted_tags_from_text_frames(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_GROUPING,
        tag,
        "GRP1",
    ) > 0
    {
        if !imported_work_from_itunes_tit1 {
            tracing::warn!("Imported grouping tag(s) from ID3 text frame GRP1 instead of TIT1");
        }
    } else {
        // Use the legacy/classical text frame only as a fallback
        if import_faceted_tags_from_text_frames(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_GROUPING,
            tag,
            "TIT1",
        ) > 0
            && config
                .flags
                .contains(ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK)
        {
            tracing::warn!("Imported grouping tag(s) from ID3 text frame TIT1 instead of GRP1");
        }
    }

    // ISRC tag
    import_faceted_tags_from_text_frames(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ISRC,
        tag,
        "TSRC",
    );

    // Language tag
    import_faceted_tags_from_text_frames(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_LANGUAGE,
        tag,
        "TLAN",
    );

    debug_assert!(track.tags.is_empty());
    track.tags = Canonical::tie(tags_map.into());

    // Indexes (in pairs)
    if tag.track().is_some() || tag.total_tracks().is_some() {
        track.indexes.track.number = tag.track().map(|i| (i & 0xFFFF) as u16);
        track.indexes.track.total = tag.total_tracks().map(|i| (i & 0xFFFF) as u16);
    }
    if tag.disc().is_some() || tag.total_discs().is_some() {
        track.indexes.disc.number = tag.disc().map(|i| (i & 0xFFFF) as u16);
        track.indexes.disc.total = tag.total_discs().map(|i| (i & 0xFFFF) as u16);
    }
    if let Some(movement) = first_text_frame(tag, "MVIN").and_then(parse_index_numbers) {
        track.indexes.movement = movement;
    }

    // Artwork
    if config.flags.contains(ImportTrackFlags::EMBEDDED_ARTWORK) {
        let mut image_digest = if config.flags.contains(ImportTrackFlags::ARTWORK_DIGEST) {
            if config
                .flags
                .contains(ImportTrackFlags::ARTWORK_DIGEST_SHA256)
            {
                // Compatibility
                MediaDigest::sha256()
            } else {
                // Default
                MediaDigest::new()
            }
        } else {
            Default::default()
        };
        let artwork = tag
            .pictures()
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
            .filter_map(|(t, p)| {
                try_load_embedded_artwork(
                    &track.media_source.path,
                    t,
                    &p.data,
                    None,
                    &mut image_digest,
                )
                .map(Artwork::Embedded)
            })
            .next();
        if artwork.is_some() {
            track.media_source.artwork = artwork;
        } else {
            track.media_source.artwork = Some(Artwork::Missing);
        }
    }

    // Serato Tags
    if config.flags.contains(ImportTrackFlags::SERATO_TAGS) {
        let mut serato_tags = SeratoTagContainer::new();

        for geob in tag.encapsulated_objects() {
            match geob.description.as_str() {
                SeratoMarkers::ID3_TAG => {
                    serato_tags
                        .parse_markers(&geob.data, SeratoTagFormat::ID3)
                        .map_err(|err| {
                            tracing::warn!("Failed to parse Serato Markers: {}", err);
                        })
                        .ok();
                }
                SeratoMarkers2::ID3_TAG => {
                    serato_tags
                        .parse_markers2(&geob.data, SeratoTagFormat::ID3)
                        .map_err(|err| {
                            tracing::warn!("Failed to parse Serato Markers2: {}", err);
                        })
                        .ok();
                }
                _ => (),
            }
        }

        let track_cues = serato::read_cues(&serato_tags)?;
        if !track_cues.is_empty() {
            track.cues = Canonical::tie(track_cues);
        }

        track.color = serato::read_track_color(&serato_tags);
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
    id3_tag: &mut id3::Tag,
) -> std::result::Result<(), ExportError> {
    if id3_tag.version() != id3::Version::Id3v24 {
        return Err(ExportError::UnsupportedLegacyVersion(id3_tag.version()));
    }

    // Audio properties
    match &track.media_source.content {
        Content::Audio(audio) => {
            if let Some(formatted_track_gain) =
                audio.loudness.map(format_valid_replay_gain).flatten()
            {
                id3_tag.add_extended_text("REPLAYGAIN_TRACK_GAIN", formatted_track_gain);
            } else {
                id3_tag.remove_extended_text(Some("REPLAYGAIN_TRACK_GAIN"), None);
            }
            if let Some(encoder) = &audio.encoder {
                id3_tag.set_text("TENC", encoder)
            } else {
                id3_tag.remove("TENC");
            }
            // TENC and TSSE have been joined during import
            id3_tag.remove("TSSE");
        }
    }

    // Music: Tempo/BPM
    id3_tag.remove_extended_text(Some("TEMPO"), None);
    if let Some(formatted_bpm) = format_validated_tempo_bpm(&mut track.metrics.tempo_bpm) {
        id3_tag.add_extended_text("BPM", formatted_bpm);
        id3_tag.set_text(
            "TBPM",
            track
                .metrics
                .tempo_bpm
                .expect("valid bpm")
                .0
                .round()
                .to_string(),
        );
    } else {
        id3_tag.remove_extended_text(Some("BPM"), None);
        id3_tag.remove("TBPM");
    }

    // Music: Key
    if track.metrics.key_signature.is_unknown() {
        id3_tag.remove("TKEY");
    } else {
        // TODO: Write a custom key code string according to config
        id3_tag.set_text("TKEY", track.metrics.key_signature.to_string());
    }

    // Track titles
    if let Some(title) = Titles::main_title(track.titles.iter()) {
        id3_tag.set_title(title.name.to_owned());
    } else {
        id3_tag.remove_title();
    }
    id3_tag.set_text_values(
        "TIT3",
        Titles::filter_kind(track.titles.iter(), TitleKind::Sub).map(|title| &title.name),
    );
    id3_tag.set_text_values(
        "MVNM",
        Titles::filter_kind(track.titles.iter(), TitleKind::Movement).map(|title| &title.name),
    );
    id3_tag.remove_extended_text(Some("WORK"), None);
    if config
        .flags
        .contains(ExportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK)
    {
        id3_tag.set_text_values(
            "TIT1",
            Titles::filter_kind(track.titles.iter(), TitleKind::Work).map(|title| &title.name),
        );
    } else if let Some(joined_titles) = TagMappingConfig::join_labels_str_iter_with_separator(
        Titles::filter_kind(track.titles.iter(), TitleKind::Work).map(|title| title.name.as_str()),
        ID3V24_MULTI_FIELD_SEPARATOR,
    ) {
        id3_tag.add_extended_text("WORK", joined_titles.to_owned());
    }

    // Track actors
    export_filtered_actor_names(
        id3_tag,
        "TPE1",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Artist),
    );
    export_filtered_actor_names(
        id3_tag,
        "TCOM",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Composer),
    );
    export_filtered_actor_names(
        id3_tag,
        "TPE3",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Conductor),
    );
    export_filtered_actor_names_txxx(
        id3_tag,
        "DIRECTOR",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Director),
    );
    export_filtered_actor_names(
        id3_tag,
        "TPE4",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Remixer),
    );
    export_filtered_actor_names(
        id3_tag,
        "TEXT",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Lyricist),
    );
    // "Writer", not "WRITER" in all caps
    // See also: https://tickets.metabrainz.org/browse/PICARD-1101
    export_filtered_actor_names_txxx(
        id3_tag,
        "Writer",
        FilteredActorNames::new(track.actors.iter(), ActorRole::Writer),
    );
    // TODO: Export TIPL frames

    // Album
    if let Some(title) = Titles::main_title(track.album.titles.iter()) {
        id3_tag.set_album(title.name.to_owned());
    } else {
        id3_tag.remove_album();
    }
    export_filtered_actor_names(
        id3_tag,
        "TPE2",
        FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
    );
    match track.album.kind {
        AlbumKind::Unknown => {
            id3_tag.remove("TCMP");
        }
        AlbumKind::Compilation => {
            id3_tag.set_text("TCMP", "1");
        }
        AlbumKind::Album | AlbumKind::Single => {
            id3_tag.set_text("TCMP", "0");
        }
    }

    // Release
    if let Some(copyright) = &track.release.copyright {
        id3_tag.set_text("TCOP", copyright);
    } else {
        id3_tag.remove("TCOP");
    }
    if let Some(released_by) = &track.release.released_by {
        id3_tag.set_text("TPUB", released_by);
    } else {
        id3_tag.remove("TPUB");
    }
    if let Some(released_at) = &track.release.released_at {
        let timestamp = export_date_or_date_time(*released_at);
        id3_tag.set_date_released(timestamp);
    } else {
        id3_tag.remove_date_released();
    }

    // Numbers
    if let Some(track_number) = track.indexes.track.number {
        id3_tag.set_track(track_number.into());
    } else {
        id3_tag.remove_track();
    }
    if let Some(track_total) = track.indexes.track.total {
        id3_tag.set_total_tracks(track_total.into());
    } else {
        id3_tag.remove_total_tracks();
    }
    if let Some(disc_number) = track.indexes.disc.number {
        id3_tag.set_disc(disc_number.into());
    } else {
        id3_tag.remove_disc();
    }
    if let Some(disc_total) = track.indexes.disc.total {
        id3_tag.set_total_discs(disc_total.into());
    } else {
        id3_tag.remove_total_discs();
    }
    if let Some(movement_number) = track.indexes.movement.number {
        if let Some(movement_total) = track.indexes.movement.total {
            id3_tag.set_text("MVIN", format!("{}/{}", movement_number, movement_total));
        } else {
            id3_tag.set_text("MVIN", movement_number.to_string());
        }
    } else if let Some(movement_total) = track.indexes.movement.total {
        id3_tag.set_text("MVIN", format!("/{}", movement_total));
    } else {
        id3_tag.remove("MVIN");
    }

    let mut tags_map = TagsMap::from(track.tags.clone().untie());

    // Comment(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_COMMENT) {
        export_faceted_tags_comment(
            id3_tag,
            String::default(),
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags_comment(id3_tag, String::default(), None, &[]);
    }

    // Genre(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_GENRE) {
        export_faceted_tags(
            id3_tag,
            "TCON",
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags(id3_tag, "TCON", None, &[]);
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_MOOD) {
        export_faceted_tags(
            id3_tag,
            "TMOO",
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags(id3_tag, "TMOO", None, &[]);
    }

    // Grouping(s)
    let grouping_frame_id = if config
        .flags
        .contains(ExportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK)
    {
        "GRP1"
    } else {
        id3_tag.remove("GRP1");
        "TIT1"
    };
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_GROUPING) {
        export_faceted_tags(
            id3_tag,
            grouping_frame_id,
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags(id3_tag, grouping_frame_id, None, &[]);
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ISRC) {
        export_faceted_tags(
            id3_tag,
            "TSRC",
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags(id3_tag, "TSRC", None, &[]);
    }

    // Language(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_LANGUAGE) {
        export_faceted_tags(
            id3_tag,
            "TLAN",
            config.faceted_tag_mapping.get(facet_id.value()),
            &tags,
        );
    } else {
        export_faceted_tags(id3_tag, "TLAN", None, &[]);
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
            let date_time = chrono::DateTime::<Utc>::from(date_time);
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
    id3_tag: &mut id3::Tag,
    text_frame_id: impl AsRef<str>,
    actor_names: FilteredActorNames<'_>,
) {
    match actor_names {
        FilteredActorNames::Summary(name) => {
            id3_tag.set_text(text_frame_id, name);
        }
        FilteredActorNames::Primary(names) => {
            id3_tag.set_text_values(text_frame_id, names);
        }
    }
}

fn export_filtered_actor_names_txxx(
    id3_tag: &mut id3::Tag,
    txxx_description: impl AsRef<str>,
    actor_names: FilteredActorNames<'_>,
) {
    id3_tag.remove_extended_text(Some(txxx_description.as_ref()), None);
    match actor_names {
        FilteredActorNames::Summary(name) => {
            id3_tag.add_extended_text(txxx_description.as_ref().to_owned(), name);
        }
        FilteredActorNames::Primary(names) => {
            if let Some(joined_names) = TagMappingConfig::join_labels_str_iter_with_separator(
                names.iter().copied(),
                ID3V24_MULTI_FIELD_SEPARATOR,
            ) {
                id3_tag.add_extended_text(
                    txxx_description.as_ref().to_owned(),
                    joined_names.to_owned(),
                );
            }
        }
    }
}

const ID3V24_MULTI_FIELD_SEPARATOR: &str = "\0";

fn export_faceted_tags(
    id3_tag: &mut id3::Tag,
    text_frame_id: impl AsRef<str>,
    config: Option<&TagMappingConfig>,
    tags: &[PlainTag],
) {
    let joined_labels = if let Some(config) = config {
        config.join_labels_str_iter(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
        )
    } else {
        TagMappingConfig::join_labels_str_iter_with_separator(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
            ID3V24_MULTI_FIELD_SEPARATOR,
        )
    };
    if let Some(joined_labels) = joined_labels {
        id3_tag.set_text(text_frame_id, joined_labels);
    } else {
        id3_tag.remove(text_frame_id);
    }
}

fn export_faceted_tags_comment(
    id3_tag: &mut id3::Tag,
    description: impl Into<String>,
    config: Option<&TagMappingConfig>,
    tags: &[PlainTag],
) {
    let joined_labels = if let Some(config) = config {
        config.join_labels_str_iter(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
        )
    } else {
        TagMappingConfig::join_labels_str_iter_with_separator(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
            ID3V24_MULTI_FIELD_SEPARATOR,
        )
    };
    if let Some(joined_labels) = joined_labels {
        let comment = Comment {
            lang: String::default(),
            description: description.into(),
            text: joined_labels.into(),
        };
        id3_tag.add_comment(comment);
    } else {
        id3_tag.remove_comment(Some(&description.into()), None);
    }
}
