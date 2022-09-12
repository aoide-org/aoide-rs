// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, collections::HashMap};

use metaflac::block::{Picture, PictureType};
use num_traits::FromPrimitive as _;

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::{
        artwork::{ApicType, Artwork},
        content::ContentMetadata,
    },
    music::{key::KeySignature, tempo::TempoBpm},
    tag::{FacetId, FacetedTags, PlainTag, TagsMap},
    track::{
        actor::Role as ActorRole,
        album::Kind as AlbumKind,
        index::Index,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_MOOD,
        },
        title::{Kind as TitleKind, Title, Titles},
        Track,
    },
    util::{
        canonical::Canonical,
        clock::{DateOrDateTime, DateYYYYMMDD},
        string::trimmed_non_empty_from,
    },
};

use crate::{
    io::{
        export::{ExportTrackConfig, FilteredActorNames},
        import::{ImportTrackConfig, ImportTrackFlags, Importer, TrackScope},
    },
    util::{
        format_valid_replay_gain, format_validated_tempo_bpm, ingest_title_from,
        key_signature_as_str, push_next_actor_role_name_from,
        tag::{FacetedTagMappingConfig, TagMappingConfig},
        trim_readable, try_ingest_embedded_artwork_image,
    },
    Result,
};

pub const ARTIST_KEY: &str = "ARTIST";
pub const ARRANGER_KEY: &str = "ARRANGER";
pub const COMPOSER_KEY: &str = "COMPOSER";
pub const CONDUCTOR_KEY: &str = "CONDUCTOR";
pub const PRODUCER_KEY: &str = "PRODUCER";
pub const REMIXER_KEY: &str = "REMIXER";
// MIXARTIST: Fallback for compatibility with Rekordbox, Engine DJ, and Traktor
pub const REMIXER_KEY2: &str = "MIXARTIST";
pub const MIXER_KEY: &str = "MIXER";
pub const DJMIXER_KEY: &str = "DJMIXER";
pub const ENGINEER_KEY: &str = "ENGINEER";
pub const DIRECTOR_KEY: &str = "DIRECTOR";
pub const LYRICIST_KEY: &str = "LYRICIST";
pub const WRITER_KEY: &str = "WRITER";

pub const ALBUM_ARTIST_KEY: &str = "ALBUMARTIST";
pub const ALBUM_ARTIST_KEY2: &str = "ALBUM_ARTIST";
pub const ALBUM_ARTIST_KEY3: &str = "ALBUM ARTIST";
pub const ALBUM_ARTIST_KEY4: &str = "ENSEMBLE";

pub const COMMENT_KEY: &str = "COMMENT";
pub const COMMENT_KEY2: &str = "DESCRIPTION";

pub const GENRE_KEY: &str = "GENRE";
pub const GROUPING_KEY: &str = "GROUPING";
pub const MOOD_KEY: &str = "MOOD";

pub const ISRC_KEY: &str = "ISRC";

pub const MUSICBRAINZ_RECORDING_ID_KEY: &str = "MUSICBRAINZ_TRACKID";
pub const MUSICBRAINZ_RELEASE_ID_KEY: &str = "MUSICBRAINZ_ALBUMID";
pub const MUSICBRAINZ_RELEASEGROUP_ID_KEY: &str = "MUSICBRAINZ_RELEASEGROUPID";

fn cmp_eq_comment_key(key1: &str, key2: &str) -> bool {
    key1.eq_ignore_ascii_case(key2)
}

pub fn filter_comment_values<'s, 'k>(
    comments: impl IntoIterator<Item = (&'s str, &'s str)>,
    key: &'k str,
) -> impl Iterator<Item = &'s str>
where
    'k: 's,
{
    comments
        .into_iter()
        .filter_map(|(k, v)| cmp_eq_comment_key(k, key).then(|| v))
}

pub fn read_first_comment_value<'s, 'k>(
    comments: impl IntoIterator<Item = (&'s str, &'s str)>,
    key: &'k str,
) -> Option<&'s str>
where
    'k: 's,
{
    filter_comment_values(comments, key).next()
}

pub trait CommentReader {
    fn read_first_value(&self, key: &str) -> Option<&str> {
        self.filter_values(key)
            .unwrap_or_default()
            .into_iter()
            .next()
    }

    // TODO: Prevent allocation of temporary vector
    fn filter_values(&self, key: &str) -> Option<Vec<&str>>;
}

impl CommentReader for Vec<(String, String)> {
    fn read_first_value(&self, key: &str) -> Option<&str> {
        // TODO: Use read_first_comment_value()
        self.iter()
            .find_map(|(k, v)| cmp_eq_comment_key(k, key).then(|| v.as_str()))
    }

    fn filter_values(&self, key: &str) -> Option<Vec<&str>> {
        // TODO: Use filter_comment_values()
        let values: Vec<_> = self
            .iter()
            .filter_map(|(k, v)| cmp_eq_comment_key(k, key).then(|| v.as_str()))
            .collect();
        (!values.is_empty()).then(|| values)
    }
}

impl CommentReader for HashMap<String, String> {
    fn read_first_value(&self, key: &str) -> Option<&str> {
        // TODO: Use read_first_comment_value()
        self.iter()
            .find_map(|(k, v)| cmp_eq_comment_key(k, key).then(|| v.as_str()))
    }

    fn filter_values(&self, key: &str) -> Option<Vec<&str>> {
        // TODO: Use filter_comment_values()
        let values: Vec<_> = self
            .iter()
            .filter_map(|(k, v)| cmp_eq_comment_key(k, key).then(|| v.as_str()))
            .collect();
        values.is_empty().then(|| values)
    }
}

pub trait CommentWriter {
    fn write_single_value(&mut self, key: Cow<'_, str>, value: String) {
        self.write_multiple_values(key, vec![value]);
    }
    fn overwrite_single_value(&mut self, key: Cow<'_, str>, value: &'_ str);
    fn write_single_value_opt(&mut self, key: Cow<'_, str>, value: Option<String>) {
        if let Some(value) = value {
            self.write_single_value(key, value);
        } else {
            self.remove_all_values(&key);
        }
    }
    fn overwrite_single_value_opt(&mut self, key: Cow<'_, str>, value: Option<&'_ str>) {
        if let Some(value) = value {
            self.overwrite_single_value(key, value);
        } else {
            self.remove_all_values(&key);
        }
    }
    fn write_multiple_values(&mut self, key: Cow<'_, str>, values: Vec<String>);
    fn write_multiple_values_opt(&mut self, key: Cow<'_, str>, values: Option<Vec<String>>) {
        if let Some(values) = values {
            self.write_multiple_values(key, values);
        } else {
            self.remove_all_values(&key);
        }
    }
    fn remove_all_values(&mut self, key: &'_ str);
}

impl CommentWriter for Vec<(String, String)> {
    fn overwrite_single_value(&mut self, key: Cow<'_, str>, value: &'_ str) {
        // Not optimized, but good enough and safe
        if self.iter().any(|(any_key, _)| any_key == &key) {
            self.write_single_value(key, value.into());
        }
    }
    fn write_multiple_values(&mut self, key: Cow<'_, str>, values: Vec<String>) {
        // TODO: Optimize or use a different data structure for writing
        self.remove_all_values(&key);
        self.reserve(self.len() + values.len());
        let key = key.into_owned();
        for value in values {
            self.push((key.clone(), value));
        }
    }
    fn remove_all_values(&mut self, key: &'_ str) {
        self.retain(|(cmp_key, _)| cmp_key != key)
    }
}

pub fn find_embedded_artwork_image(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Option<(ApicType, String, Vec<u8>)> {
    // https://wiki.xiph.org/index.php/VorbisComment#Cover_art
    // The unofficial COVERART field in a VorbisComment tag is deprecated:
    // https://wiki.xiph.org/VorbisComment#Unofficial_COVERART_field_.28deprecated.29
    let picture_iter_by_type = |picture_type: Option<PictureType>| {
        reader
            .filter_values("METADATA_BLOCK_PICTURE")
            .unwrap_or_default()
            .into_iter()
            .chain(reader.filter_values("COVERART").unwrap_or_default())
            .map(|elem| (elem, Vec::new()))
            .filter_map(|(base64_data, mut issues)| {
                base64::decode(base64_data)
                    .map_err(|err| {
                        issues.push(format!(
                            "Failed to decode base64 encoded picture block: {err}"
                        ));
                    })
                    .map(|decoded| (decoded, issues))
                    .ok()
            })
            .filter_map(|(decoded, mut issues)| {
                metaflac::block::Picture::from_bytes(&decoded[..])
                    .map_err(|err| {
                        issues.push(format!("Failed to decode FLAC picture block: {err}"));
                    })
                    .map(|picture| (picture, issues))
                    .ok()
            })
            .filter(move |(picture, _)| {
                if let Some(picture_type) = picture_type {
                    picture.picture_type == picture_type
                } else {
                    true
                }
            })
    };
    // Decoding and discarding the blocks multiple times is inefficient
    // but expected to occur only infrequently. Most files will include
    // just a front cover and nothing else.
    picture_iter_by_type(Some(PictureType::CoverFront))
        .chain(picture_iter_by_type(Some(PictureType::Media)))
        .chain(picture_iter_by_type(Some(PictureType::Leaflet)))
        .chain(picture_iter_by_type(Some(PictureType::Other)))
        // otherwise take the first picture that could be parsed
        .chain(picture_iter_by_type(None))
        .map(|(picture, issues)| {
            let Picture {
                picture_type,
                mime_type,
                data,
                ..
            } = picture;
            let apic_type = ApicType::from_u8(picture_type as u8).unwrap_or(ApicType::Other);
            issues
                .into_iter()
                .for_each(|message| importer.add_issue(message));
            (apic_type, mime_type, data)
        })
        .next()
}

pub fn import_faceted_text_tags<'a>(
    importer: &mut Importer,
    tags_map: &mut TagsMap,
    faceted_tag_mapping_config: &FacetedTagMappingConfig,
    facet_id: &FacetId,
    label_values: impl IntoIterator<Item = &'a str>,
) {
    importer.import_faceted_tags_from_label_values(
        tags_map,
        faceted_tag_mapping_config,
        facet_id,
        label_values.into_iter().map(Into::into),
    );
}

pub fn import_loudness(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Option<LoudnessLufs> {
    reader
        .read_first_value("REPLAYGAIN_TRACK_GAIN")
        .and_then(|value| importer.import_replay_gain(value))
}

fn export_loudness(writer: &mut impl CommentWriter, loudness: Option<LoudnessLufs>) {
    if let Some(formatted_track_gain) = loudness.and_then(format_valid_replay_gain) {
        writer.write_single_value("REPLAYGAIN_TRACK_GAIN".into(), formatted_track_gain);
    } else {
        writer.remove_all_values("REPLAYGAIN_TRACK_GAIN");
    }
}

pub fn import_encoder(reader: &'_ impl CommentReader) -> Option<Cow<'_, str>> {
    reader.read_first_value("ENCODEDBY").map(Into::into)
}

fn export_encoder(writer: &mut impl CommentWriter, encoder: Option<impl Into<String>>) {
    if let Some(encoder) = encoder.map(Into::into) {
        writer.write_single_value("ENCODEDBY".into(), encoder);
    } else {
        writer.remove_all_values("ENCODEDBY");
    }
}

pub fn import_tempo_bpm(importer: &mut Importer, reader: &impl CommentReader) -> Option<TempoBpm> {
    reader
        .read_first_value("BPM")
        .and_then(|input| importer.import_tempo_bpm(input))
        // Alternative: Try "TEMPO" if "BPM" is missing or invalid
        .or_else(|| {
            reader
                .read_first_value("TEMPO")
                .and_then(|input| importer.import_tempo_bpm(input))
        })
}

fn export_tempo_bpm(writer: &mut impl CommentWriter, tempo_bpm: &mut Option<TempoBpm>) {
    if let Some(formatted_bpm) = format_validated_tempo_bpm(tempo_bpm) {
        writer.write_single_value("BPM".into(), formatted_bpm);
    } else {
        writer.remove_all_values("BPM");
    }
    writer.remove_all_values("TEMPO");
}

pub fn import_key_signature(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Option<KeySignature> {
    reader
        .read_first_value("KEY")
        .and_then(|value| importer.import_key_signature(value))
        .or_else(|| {
            // Fallback for Rekordbox/Serato/Traktor
            // https://docs.google.com/spreadsheets/d/1zhIJPOtYIueV72Gd81aVnbSa6dIA-azq9fnGC2rHUzo
            reader
                .read_first_value("INITIALKEY")
                .and_then(|value| importer.import_key_signature(value))
        })
}

fn export_key_signature(writer: &mut impl CommentWriter, key_signature: Option<KeySignature>) {
    if let Some(key_signature) = key_signature {
        let value = key_signature_as_str(key_signature);
        writer.write_single_value("KEY".into(), value.into());
        writer.overwrite_single_value("INITIALKEY".into(), value);
    } else {
        writer.remove_all_values("KEY");
        writer.remove_all_values("INITIALKEY");
    }
}

pub fn import_album_kind(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Option<AlbumKind> {
    let value = reader.read_first_value("COMPILATION");
    value
        .and_then(|compilation| trim_readable(compilation).parse::<u8>().ok())
        .and_then(|compilation| match compilation {
            0 => Some(AlbumKind::NoCompilation),
            1 => Some(AlbumKind::Compilation),
            _ => {
                importer.add_issue(format!(
                    "Unexpected tag value: COMPILATION = '{}'",
                    value.expect("unreachable")
                ));
                None
            }
        })
}

pub fn import_recorded_at(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Option<DateOrDateTime> {
    reader
        .read_first_value("DATE")
        .and_then(|value| importer.import_year_tag_from_field("DATE", value))
        .or_else(|| {
            reader
                .read_first_value("YEAR")
                .and_then(|value| importer.import_year_tag_from_field("YEAR", value))
        })
}

pub fn import_released_at(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Option<DateOrDateTime> {
    reader
        .read_first_value("RELEASEDATE")
        .and_then(|value| importer.import_year_tag_from_field("RELEASEDATE", value))
        .or_else(|| {
            reader
                .read_first_value("RELEASEYEAR")
                .and_then(|value| importer.import_year_tag_from_field("RELEASEYEAR", value))
        })
}

pub fn import_released_orig_at(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Option<DateOrDateTime> {
    reader
        .read_first_value("ORIGINALDATE")
        .and_then(|value| importer.import_year_tag_from_field("ORIGINALDATE", value))
        .or_else(|| {
            reader
                .read_first_value("ORIGINALYEAR")
                .and_then(|value| importer.import_year_tag_from_field("ORIGINALYEAR", value))
        })
}

pub fn import_publisher(reader: &impl CommentReader) -> Option<String> {
    reader
        .read_first_value("LABEL")
        .and_then(trimmed_non_empty_from)
        .or_else(|| {
            // Primary fallback
            reader
                .read_first_value("PUBLISHER")
                .and_then(trimmed_non_empty_from)
        })
        .or_else(|| {
            // Secondary fallback
            reader
                .read_first_value("ORGANIZATION")
                .and_then(trimmed_non_empty_from)
        })
        .map(Into::into)
}

pub fn import_copyright(reader: &impl CommentReader) -> Option<String> {
    reader
        .read_first_value("COPYRIGHT")
        .and_then(trimmed_non_empty_from)
        .map(Into::into)
}

pub fn import_track_index(importer: &mut Importer, reader: &impl CommentReader) -> Option<Index> {
    if let Some(mut index) = reader
        .read_first_value("TRACKNUMBER")
        .and_then(|value| importer.import_index_numbers_from_field("TRACKNUMBER", value))
    {
        if let Some(total) =
            // According to https://wiki.xiph.org/Field_names "TRACKTOTAL" is
            // the proposed field name, but some applications use "TOTALTRACKS".
            reader
                .read_first_value("TRACKTOTAL")
                .and_then(|value| value.parse().ok())
                .or_else(|| {
                    reader
                        .read_first_value("TOTALTRACKS")
                        .and_then(|value| value.parse().ok())
                })
        {
            if let Some(index_total) = index.total {
                importer.add_issue(format!(
                    "Overwriting total number of tracks {index_total} parsed from field 'TRACKNUMBER' with {total}"
                ));
            }
            index.total = Some(total);
        }
        Some(index)
    } else {
        None
    }
}

pub fn import_disc_index(importer: &mut Importer, reader: &impl CommentReader) -> Option<Index> {
    if let Some(mut index) = reader
        .read_first_value("DISCNUMBER")
        .and_then(|value| importer.import_index_numbers_from_field("DISCNUMBER", value))
    {
        if let Some(total) = reader
            .read_first_value("DISCTOTAL")
            .and_then(|value| value.parse().ok())
            .or_else(|| {
                reader
                    .read_first_value("TOTALDISCS")
                    .and_then(|value| value.parse().ok())
            })
        {
            if let Some(index_total) = index.total {
                importer.add_issue(format!(
                    "Overwriting total number of discs {index_total} parsed from field 'DISCNUMBER' with {total}"
                ));
            }
            index.total = Some(total);
        }
        Some(index)
    } else {
        None
    }
}

pub fn import_movement_index(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Option<Index> {
    if let Some(mut index) = reader
        .read_first_value("MOVEMENT")
        .and_then(|value| importer.import_index_numbers_from_field("MOVEMENT", value))
    {
        if let Some(total) = reader
            .read_first_value("MOVEMENTTOTAL")
            .and_then(|value| value.parse().ok())
        {
            if let Some(index_total) = index.total {
                importer.add_issue(format!(
                    "Overwriting total number of movements {index_total} parsed from field 'MOVEMENT' with {total}"
                ));
            }
            index.total = Some(total);
        }
        Some(index)
    } else {
        None
    }
}

pub fn import_track_titles(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Canonical<Vec<Title>> {
    let mut track_titles = Vec::with_capacity(4);
    if let Some(title) = reader
        .read_first_value("TITLE")
        .and_then(|name| ingest_title_from(name, TitleKind::Main))
    {
        track_titles.push(title);
    }

    if let Some(title) = reader
        .read_first_value("SUBTITLE")
        .and_then(|name| ingest_title_from(name, TitleKind::Sub))
    {
        track_titles.push(title);
    }
    if let Some(title) = reader
        .read_first_value("WORK")
        .and_then(|name| ingest_title_from(name, TitleKind::Work))
    {
        track_titles.push(title);
    }
    if let Some(title) = reader
        .read_first_value("MOVEMENTNAME")
        .and_then(|name| ingest_title_from(name, TitleKind::Movement))
    {
        track_titles.push(title);
    }
    importer.finish_import_of_titles(TrackScope::Track, track_titles)
}

pub fn import_album_titles(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Canonical<Vec<Title>> {
    let mut album_titles = Vec::with_capacity(1);
    if let Some(title) = reader
        .read_first_value("ALBUM")
        .and_then(|name| ingest_title_from(name, TitleKind::Main))
    {
        album_titles.push(title);
    }
    importer.finish_import_of_titles(TrackScope::Album, album_titles)
}

#[cfg(feature = "serato-markers")]
#[must_use]
pub fn import_serato_markers2(
    importer: &mut Importer,
    reader: &impl CommentReader,
    serato_tags: &mut triseratops::tag::TagContainer,
    format: triseratops::tag::TagFormat,
) -> bool {
    let vorbis_comment = match format {
        triseratops::tag::TagFormat::FLAC => {
            <triseratops::tag::Markers2 as triseratops::tag::format::flac::FLACTag>::FLAC_COMMENT
        }
        triseratops::tag::TagFormat::Ogg => {
            <triseratops::tag::Markers2 as triseratops::tag::format::ogg::OggTag>::OGG_COMMENT
        }
        _ => {
            return false;
        }
    };

    reader
        .read_first_value(vorbis_comment)
        .and_then(|data| {
            serato_tags
                .parse_markers2(data.as_bytes(), format)
                .map_err(|err| {
                    importer.add_issue(format!("Failed to import Serato Markers2: {err}"));
                })
                .ok()
        })
        .is_some()
}

pub fn import_into_track(
    importer: &mut Importer,
    reader: &impl CommentReader,
    config: &ImportTrackConfig,
    track: &mut Track,
) -> Result<()> {
    track.metrics.tempo_bpm = import_tempo_bpm(importer, reader);

    track.metrics.key_signature = import_key_signature(importer, reader);

    track.recorded_at = import_recorded_at(importer, reader);
    track.released_at = import_released_at(importer, reader);
    track.released_orig_at = import_released_orig_at(importer, reader);

    track.publisher = import_publisher(reader);
    track.copyright = import_copyright(reader);

    // Track titles
    track.titles = import_track_titles(importer, reader);

    // Track actors
    let mut track_actors = Vec::with_capacity(8);
    for name in reader.filter_values(ARTIST_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Artist, name);
    }
    for name in reader.filter_values(ARRANGER_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Arranger, name);
    }
    for name in reader.filter_values(COMPOSER_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Composer, name);
    }
    for name in reader.filter_values(CONDUCTOR_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Conductor, name);
    }
    for name in reader.filter_values(CONDUCTOR_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Producer, name);
    }
    if reader.read_first_value(REMIXER_KEY).is_some() {
        for name in reader.filter_values(REMIXER_KEY).unwrap_or_default() {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Remixer, name);
        }
    } else {
        for name in reader.filter_values(REMIXER_KEY2).unwrap_or_default() {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Remixer, name);
        }
    }
    for name in reader.filter_values(MIXER_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Mixer, name);
    }
    for name in reader.filter_values(DJMIXER_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::DjMixer, name);
    }
    for name in reader.filter_values(ENGINEER_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Engineer, name);
    }
    for name in reader.filter_values(DIRECTOR_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Director, name);
    }
    for name in reader.filter_values(LYRICIST_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Lyricist, name);
    }
    for name in reader.filter_values(WRITER_KEY).unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Writer, name);
    }
    track.actors = importer.finish_import_of_actors(TrackScope::Track, track_actors);

    let mut album = track.album.untie_replace(Default::default());

    // Album titles
    album.titles = import_album_titles(importer, reader);

    // Album actors
    let mut album_actors = Vec::with_capacity(4);
    for name in reader
        .filter_values(ALBUM_ARTIST_KEY)
        .unwrap_or_default()
        .into_iter()
        .chain(
            reader
                .filter_values(ALBUM_ARTIST_KEY2)
                .unwrap_or_default()
                .into_iter(),
        )
        .chain(
            reader
                .filter_values(ALBUM_ARTIST_KEY3)
                .unwrap_or_default()
                .into_iter(),
        )
        .chain(
            reader
                .filter_values(ALBUM_ARTIST_KEY4)
                .unwrap_or_default()
                .into_iter(),
        )
    {
        push_next_actor_role_name_from(&mut album_actors, ActorRole::Artist, name);
    }
    album.actors = importer.finish_import_of_actors(TrackScope::Album, album_actors);

    // Album properties
    album.kind = import_album_kind(importer, reader);

    track.album = Canonical::tie(album);

    let mut tags_map = TagsMap::default();

    // Grouping tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_GROUPING,
        reader
            .filter_values(GROUPING_KEY)
            .unwrap_or_default()
            .into_iter(),
    );

    // Import gigtags from raw grouping tags before any other tags.
    #[cfg(feature = "gigtag")]
    if config.flags.contains(ImportTrackFlags::GIGTAGS) {
        if let Some(faceted_tags) = tags_map.take_faceted_tags(&FACET_ID_GROUPING) {
            tags_map = crate::util::gigtag::import_from_faceted_tags(faceted_tags);
        }
    }

    // Comment tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_COMMENT,
        reader
            .filter_values(COMMENT_KEY)
            .unwrap_or_default()
            .into_iter(),
    );

    // Description tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_DESCRIPTION,
        reader
            .filter_values(COMMENT_KEY2)
            .unwrap_or_default()
            .into_iter(),
    );

    // Genre tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_GENRE,
        reader
            .filter_values(GENRE_KEY)
            .unwrap_or_default()
            .into_iter(),
    );

    // Mood tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_MOOD,
        reader
            .filter_values(MOOD_KEY)
            .unwrap_or_default()
            .into_iter(),
    );

    // ISRC tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_ISRC,
        reader
            .filter_values(ISRC_KEY)
            .unwrap_or_default()
            .into_iter(),
    );

    track.indexes.track = import_track_index(importer, reader).unwrap_or_default();
    track.indexes.disc = import_disc_index(importer, reader).unwrap_or_default();
    track.indexes.movement = import_movement_index(importer, reader).unwrap_or_default();

    if config
        .flags
        .contains(ImportTrackFlags::METADATA_EMBEDDED_ARTWORK)
    {
        let artwork = if let Some((apic_type, media_type, image_data)) =
            find_embedded_artwork_image(importer, reader)
        {
            let (artwork, _, issues) = try_ingest_embedded_artwork_image(
                apic_type,
                &image_data,
                None,
                Some(media_type),
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
    if config.flags.contains(ImportTrackFlags::SERATO_MARKERS) {
        let mut serato_tags = triseratops::tag::TagContainer::new();
        if import_serato_markers2(
            importer,
            reader,
            &mut serato_tags,
            triseratops::tag::TagFormat::Ogg,
        ) {
            track.cues = Canonical::tie(crate::util::serato::import_cues(&serato_tags));
            track.color = crate::util::serato::import_track_color(&serato_tags);
        }
    }

    Ok(())
}

pub fn export_track(
    config: &ExportTrackConfig,
    track: &mut Track,
    writer: &mut impl CommentWriter,
) {
    // Audio properties
    match &track.media_source.content.metadata {
        ContentMetadata::Audio(audio) => {
            export_loudness(writer, audio.loudness);
            export_encoder(writer, audio.encoder.to_owned());
        }
    }

    export_tempo_bpm(writer, &mut track.metrics.tempo_bpm);
    export_key_signature(writer, track.metrics.key_signature);

    // Track titles
    writer.write_single_value_opt(
        "TITLE".into(),
        Titles::main_title(track.titles.iter()).map(|title| title.name.to_owned()),
    );
    writer.write_multiple_values(
        "SUBTITLE".into(),
        Titles::filter_kind(track.titles.iter(), TitleKind::Sub)
            .map(|title| title.name.clone())
            .collect(),
    );
    writer.write_multiple_values(
        "WORK".into(),
        Titles::filter_kind(track.titles.iter(), TitleKind::Work)
            .map(|title| title.name.clone())
            .collect(),
    );
    writer.write_multiple_values(
        "MOVEMENTNAME".into(),
        Titles::filter_kind(track.titles.iter(), TitleKind::Movement)
            .map(|title| title.name.clone())
            .collect(),
    );

    // Track actors
    export_filtered_actor_names(
        writer,
        ARTIST_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Artist),
    );
    export_filtered_actor_names(
        writer,
        ARRANGER_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Arranger),
    );
    export_filtered_actor_names(
        writer,
        COMPOSER_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Composer),
    );
    export_filtered_actor_names(
        writer,
        CONDUCTOR_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Conductor),
    );
    export_filtered_actor_names(
        writer,
        CONDUCTOR_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Producer),
    );
    export_filtered_actor_names(
        writer,
        REMIXER_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Remixer),
    );
    export_filtered_actor_names(
        writer,
        MIXER_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Mixer),
    );
    export_filtered_actor_names(
        writer,
        DJMIXER_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::DjMixer),
    );
    export_filtered_actor_names(
        writer,
        ENGINEER_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Engineer),
    );
    export_filtered_actor_names(
        writer,
        DIRECTOR_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Director),
    );
    export_filtered_actor_names(
        writer,
        LYRICIST_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Lyricist),
    );
    export_filtered_actor_names(
        writer,
        WRITER_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Writer),
    );

    // Album
    writer.write_single_value_opt(
        "ALBUM".into(),
        Titles::main_title(track.album.titles.iter()).map(|title| title.name.to_owned()),
    );
    export_filtered_actor_names(
        writer,
        "ALBUMARTIST".into(),
        FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
    );
    if let Some(kind) = track.album.kind {
        match kind {
            AlbumKind::NoCompilation | AlbumKind::Album | AlbumKind::Single => {
                writer.write_single_value("COMPILATION".into(), "0".to_owned());
            }
            AlbumKind::Compilation => {
                writer.write_single_value("COMPILATION".into(), "1".to_owned());
            }
        }
    } else {
        writer.remove_all_values("COMPILATION");
    }

    writer.write_single_value_opt("COPYRIGHT".into(), track.copyright.clone());
    writer.write_single_value_opt("LABEL".into(), track.publisher.clone());
    writer.overwrite_single_value_opt("PUBLISHER".into(), track.publisher.as_deref()); // alternative
    writer.overwrite_single_value_opt("ORGANIZATION".into(), track.publisher.as_deref()); // alternative
    writer.write_single_value_opt(
        "DATE".into(),
        track.recorded_at.as_ref().map(ToString::to_string),
    );
    let recorded_year = track
        .recorded_at
        .map(DateYYYYMMDD::from)
        .map(DateYYYYMMDD::year);
    writer.write_single_value_opt(
        "YEAR".into(),
        recorded_year.as_ref().map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "RELEASEDATE".into(),
        track.released_at.as_ref().map(ToString::to_string),
    );
    let released_year = track
        .released_at
        .map(DateYYYYMMDD::from)
        .map(DateYYYYMMDD::year);
    writer.write_single_value_opt(
        "RELEASEYEAR".into(),
        released_year.as_ref().map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "ORIGINALDATE".into(),
        track.released_orig_at.as_ref().map(ToString::to_string),
    );
    let released_orig_year = track
        .released_orig_at
        .map(DateYYYYMMDD::from)
        .map(DateYYYYMMDD::year);
    writer.write_single_value_opt(
        "ORIGINALYEAR".into(),
        released_orig_year.as_ref().map(ToString::to_string),
    );

    // Numbers
    writer.write_single_value_opt(
        "TRACKNUMBER".into(),
        track.indexes.track.number.as_ref().map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "TRACKTOTAL".into(),
        track.indexes.track.total.as_ref().map(ToString::to_string),
    );
    // According to https://wiki.xiph.org/Field_names "TRACKTOTAL" is
    // the proposed field name, but some applications use(d) "TOTALTRACKS".
    writer.remove_all_values("TOTALTRACKS");
    writer.write_single_value_opt(
        "DISCNUMBER".into(),
        track.indexes.disc.number.as_ref().map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "DISCTOTAL".into(),
        track.indexes.disc.total.as_ref().map(ToString::to_string),
    );
    // According to https://wiki.xiph.org/Field_names "DISCTOTAL" is
    // the proposed field name, but some applications use(d) "TOTALDISCS".
    writer.remove_all_values("TOTALDISCS");
    writer.write_single_value_opt(
        "MOVEMENT".into(),
        track
            .indexes
            .movement
            .number
            .as_ref()
            .map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "MOVEMENTTOTAL".into(),
        track
            .indexes
            .movement
            .total
            .as_ref()
            .map(ToString::to_string),
    );

    // Export selected tags into dedicated fields
    let mut tags_map = TagsMap::from(track.tags.clone().untie());

    // Comment(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_COMMENT) {
        export_faceted_tags(
            writer,
            COMMENT_KEY.into(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values(COMMENT_KEY);
    }

    // Description(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_DESCRIPTION)
    {
        export_faceted_tags(
            writer,
            COMMENT_KEY2.into(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values(COMMENT_KEY2);
    }

    // Genre(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_GENRE) {
        export_faceted_tags(
            writer,
            GENRE_KEY.into(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values(GENRE_KEY);
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_MOOD) {
        export_faceted_tags(
            writer,
            MOOD_KEY.into(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values(MOOD_KEY);
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_ISRC) {
        export_faceted_tags(
            writer,
            ISRC_KEY.into(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values(ISRC_KEY);
    }

    // Grouping(s)
    {
        let facet_id = &FACET_ID_GROUPING;
        let mut tags = tags_map
            .take_faceted_tags(facet_id)
            .map(|FacetedTags { facet_id: _, tags }| tags)
            .unwrap_or_default();
        #[cfg(feature = "gigtag")]
        if config
            .flags
            .contains(crate::io::export::ExportTrackFlags::GIGTAGS)
        {
            if let Err(err) = crate::util::gigtag::export_and_encode_remaining_tags_into(
                tags_map.into(),
                &mut tags,
            ) {
                log::error!("Failed to export gigitags: {err}");
            }
        }
        if tags.is_empty() {
            writer.remove_all_values(GROUPING_KEY);
        } else {
            export_faceted_tags(
                writer,
                GROUPING_KEY.into(),
                config.faceted_tag_mapping.get(facet_id.value()),
                tags,
            );
        }
    }
}

fn export_filtered_actor_names<'a>(
    writer: &mut impl CommentWriter,
    key: Cow<'a, str>,
    actor_names: FilteredActorNames<'_>,
) {
    match actor_names {
        FilteredActorNames::Summary(name) => {
            writer.write_single_value(key, name.to_owned());
        }
        FilteredActorNames::Primary(names) => {
            writer.write_multiple_values(key, names.into_iter().map(ToOwned::to_owned).collect());
        }
    }
}

fn export_faceted_tags<'a>(
    writer: &mut impl CommentWriter,
    key: Cow<'a, str>,
    config: Option<&TagMappingConfig>,
    tags: Vec<PlainTag>,
) {
    if let Some(config) = config {
        let joined_labels = config.join_labels(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
        );
        writer.write_single_value_opt(key, joined_labels.map(Into::into));
    } else {
        let tag_labels = tags
            .into_iter()
            .map(|tag| tag.label.unwrap_or_default().into_value())
            .collect();
        writer.write_multiple_values(key, tag_labels);
    }
}
