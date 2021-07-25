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

use crate::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    util::{
        digest::MediaDigest,
        parse_artwork_from_embedded_image, parse_index_numbers, parse_key_signature,
        parse_replay_gain, parse_tempo_bpm, push_next_actor_role_name, serato,
        tag::{import_faceted_tags, FacetedTagMappingConfig},
    },
    Result,
};

use aoide_core::{
    audio::AudioContent,
    media::{concat_encoder_properties, Content, ContentMetadataFlags},
    tag::{Facet, Score as TagScore, Tags, TagsMap},
    track::{
        actor::ActorRole,
        album::AlbumKind,
        release::DateOrDateTime,
        tag::{FACET_CGROUP, FACET_COMMENT, FACET_GENRE, FACET_MOOD},
        title::{Title, TitleKind},
        Track,
    },
    util::{
        clock::{DateTime, DateYYYYMMDD, MonthType, YearType},
        Canonical, CanonicalizeInto as _,
    },
};

use aoide_core_serde::tag::Tags as SerdeTags;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use id3::{self, frame::PictureType};
use mime::Mime;
use semval::IsValid as _;
use std::{borrow::Cow, time::Duration};
use triseratops::tag::{
    format::id3::ID3Tag, Markers as SeratoMarkers, Markers2 as SeratoMarkers2,
    TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat,
};

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

fn import_faceted_text_tags(
    tags_map: &mut TagsMap,
    config: &FacetedTagMappingConfig,
    facet: &Facet,
    tag: &id3::Tag,
    frame_id: &str,
) {
    let removed_tags = tags_map.remove_faceted_tags(&facet);
    if removed_tags > 0 {
        log::debug!("Replacing {} custom '{}' tags", removed_tags, facet.value());
    }
    let tag_mapping_config = config.get(facet.value());
    let mut next_score_value = TagScore::max_value();
    for label in text_frames(tag, frame_id) {
        import_faceted_tags(
            tags_map,
            &mut next_score_value,
            &facet,
            tag_mapping_config,
            label,
        );
    }
}

pub fn import_track(
    config: &ImportTrackConfig,
    flags: ImportTrackFlags,
    mut audio_content: AudioContent,
    mut track: Track,
    tag: &id3::Tag,
) -> Result<Track> {
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
            first_extended_text(&tag, "REPLAYGAIN_TRACK_GAIN").and_then(parse_replay_gain);
        let encoder = concat_encoder_properties(
            first_text_frame(&tag, "TENC"),
            first_text_frame(&tag, "TSSE"),
        )
        .map(Cow::into_owned);
        audio_content = AudioContent {
            loudness,
            encoder,
            ..audio_content
        };
        track.media_source.content = Content::Audio(audio_content);
    }

    if let Some(tempo_bpm) = first_extended_text(&tag, "BPM")
        .and_then(parse_tempo_bpm)
        // Alternative: Try "TEMPO" if "BPM" is missing or invalid
        .or_else(|| first_extended_text(&tag, "TEMPO").and_then(parse_tempo_bpm))
        // Fallback: Parse integer BPM
        .or_else(|| first_text_frame(&tag, "TBPM").and_then(parse_tempo_bpm))
    {
        debug_assert!(tempo_bpm.is_valid());
        track.metrics.tempo_bpm = Some(tempo_bpm);
    }

    if let Some(key_signature) = first_text_frame(&tag, "TKEY").and_then(parse_key_signature) {
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
    if let Some(name) = first_text_frame(&tag, "TSST") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Sub,
        };
        track_titles.push(title);
    }
    if let Some(name) = first_text_frame(&tag, "MVNM") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Movement,
        };
        track_titles.push(title);
    }
    if flags.contains(ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK) {
        // Starting with iTunes 12.5.4 the "TIT1" text frame is used
        // for storing the work instead of the grouping.
        if let Some(name) = first_text_frame(&tag, "TIT1") {
            let title = Title {
                name: name.to_owned(),
                kind: TitleKind::Work,
            };
            track_titles.push(title);
        }
    } else if let Some(name) = first_extended_text(&tag, "WORK") {
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
    for name in text_frames(&tag, "TCOM") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Composer, name.to_owned());
    }
    for name in text_frames(&tag, "TPE3") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name.to_owned());
    }
    let track_actors = track_actors.canonicalize_into();
    if !track_actors.is_empty() {
        track.actors = Canonical::tie(track_actors);
    }

    let mut album = track.album.untie();

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
    if first_text_frame(&tag, "TCMP")
        .and_then(|tcmp| tcmp.parse::<u8>().ok())
        .unwrap_or_default()
        == 1
    {
        album.kind = AlbumKind::Compilation;
    }

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
    if let Some(label) = first_text_frame(&tag, "TPUB") {
        track.release.released_by = Some(label.to_owned());
    }
    if let Some(copyright) = first_text_frame(&tag, "TCOP") {
        track.release.copyright = Some(copyright.to_owned());
    }

    let mut tags_map = TagsMap::default();
    if flags.contains(ImportTrackFlags::MIXXX_CUSTOM_TAGS) {
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
                log::warn!(
                    "Unexpected MIME type for GEOB '{}': {}",
                    geob.description,
                    geob.mime_type
                );
                continue;
            }
            if let Some(custom_tags) = serde_json::from_slice::<SerdeTags>(&geob.data)
                .map_err(|err| {
                    log::warn!("Failed to parse Mixxx custom tags: {}", err);
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
    for comment in tag
        .comments()
        .filter(|comm| comm.description.is_empty())
        .map(|comm| comm.text.as_str())
    {
        let removed_comments = tags_map.remove_faceted_tags(&FACET_COMMENT);
        if removed_comments > 0 {
            log::debug!(
                "Replacing {} custom '{}' tags",
                removed_comments,
                FACET_COMMENT.value()
            );
        }
        let mut next_score_value = TagScore::default_value();
        import_faceted_tags(
            &mut tags_map,
            &mut next_score_value,
            &FACET_COMMENT,
            None,
            comment.to_owned(),
        );
    }

    // Genre tags
    import_faceted_text_tags(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_GENRE,
        &tag,
        "TCON",
    );

    // Mood tags
    import_faceted_text_tags(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_MOOD,
        &tag,
        "TMOO",
    );

    // Grouping tags
    // Apple decided to store the Work in the traditional ID3v2 Content Group
    // frame (TIT1) and introduced new Grouping (GRP1) and Movement Name (MVNM)
    // frames.
    // https://discussions.apple.com/thread/7900430
    // http://blog.jthink.net/2016/11/the-reason-why-is-grouping-field-no.html
    if flags.contains(ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK) {
        import_faceted_text_tags(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_CGROUP,
            &tag,
            "GRP1",
        );
    } else {
        import_faceted_text_tags(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_CGROUP,
            &tag,
            "TIT1",
        );
    }

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
    if let Some(movement) = first_text_frame(&tag, "MVIN").and_then(parse_index_numbers) {
        track.indexes.movement = movement;
    }

    // Artwork
    if flags.contains(ImportTrackFlags::ARTWORK) {
        let mut image_digest = if flags.contains(ImportTrackFlags::ARTWORK_DIGEST) {
            if flags.contains(ImportTrackFlags::ARTWORK_DIGEST_SHA256) {
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
            .filter(|p| p.picture_type == PictureType::CoverFront)
            .chain(
                tag.pictures()
                    .filter(|p| p.picture_type == PictureType::Media),
            )
            .chain(
                tag.pictures()
                    .filter(|p| p.picture_type == PictureType::Leaflet),
            )
            .chain(
                tag.pictures()
                    .filter(|p| p.picture_type == PictureType::Other),
            )
            // otherwise take the first picture that could be parsed
            .chain(tag.pictures())
            .filter_map(|p| parse_artwork_from_embedded_image(&p.data, None, &mut image_digest))
            .next();
        if let Some(artwork) = artwork {
            track.media_source.artwork = artwork;
        }
    }

    // Serato Tags
    if flags.contains(ImportTrackFlags::SERATO_TAGS) {
        let mut serato_tags = SeratoTagContainer::new();

        for geob in tag.encapsulated_objects() {
            match geob.description.as_str() {
                SeratoMarkers::ID3_TAG => {
                    serato_tags
                        .parse_markers(&geob.data, SeratoTagFormat::ID3)
                        .map_err(|err| {
                            log::warn!("Failed to parse Serato Markers: {}", err);
                        })
                        .ok();
                }
                SeratoMarkers2::ID3_TAG => {
                    serato_tags
                        .parse_markers2(&geob.data, SeratoTagFormat::ID3)
                        .map_err(|err| {
                            log::warn!("Failed to parse Serato Markers2: {}", err);
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

    Ok(track)
}
