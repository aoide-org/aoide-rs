// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::{borrow::Cow, collections::HashMap};

use metaflac::block::{Picture, PictureType};
use num_traits::FromPrimitive as _;
use triseratops::tag::{
    format::flac::FLACTag, format::ogg::OggTag, Markers2 as SeratoMarkers2,
    TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat,
};

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::{
        artwork::{ApicType, Artwork},
        content::ContentMetadata,
    },
    music::{key::KeySignature, tempo::TempoBpm},
    tag::{FacetId, FacetedTags, PlainTag, Tags, TagsMap},
    track::{
        actor::ActorRole,
        album::AlbumKind,
        index::Index,
        tag::{
            FACET_COMMENT, FACET_DESCRIPTION, FACET_GENRE, FACET_GROUPING, FACET_ISRC, FACET_MOOD,
        },
        title::{Title, TitleKind, Titles},
        Track,
    },
    util::{
        canonical::Canonical,
        clock::{DateOrDateTime, DateYYYYMMDD},
        string::trimmed_non_empty_from,
    },
};

use aoide_core_json::tag::Tags as SerdeTags;

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
        import::{ImportTrackConfig, ImportTrackFlags, Importer, TrackScope},
    },
    util::{
        format_valid_replay_gain, format_validated_tempo_bpm, ingest_title_from,
        push_next_actor_role_name_from, serato,
        tag::{FacetedTagMappingConfig, TagMappingConfig},
        trim_readable, try_ingest_embedded_artwork_image,
    },
    Result,
};

pub const MIXXX_CUSTOM_TAGS_KEY: &str = "MIXXX_CUSTOM_TAGS";

pub const AOIDE_TAGS_KEY: &str = "AOIDE_TAGS";

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
    fn write_single_value(&mut self, key: String, value: String) {
        self.write_multiple_values(key, vec![value]);
    }
    fn write_single_value_opt(&mut self, key: String, value: Option<String>) {
        if let Some(value) = value {
            self.write_single_value(key, value);
        } else {
            self.remove_all_values(&key);
        }
    }
    fn write_multiple_values(&mut self, key: String, values: Vec<String>);
    fn write_multiple_values_opt(&mut self, key: String, values: Option<Vec<String>>) {
        if let Some(values) = values {
            self.write_multiple_values(key, values);
        } else {
            self.remove_all_values(&key);
        }
    }
    fn remove_all_values(&mut self, key: &str);
}

impl CommentWriter for Vec<(String, String)> {
    fn write_multiple_values(&mut self, key: String, values: Vec<String>) {
        // TODO: Optimize or use a different data structure for writing
        self.remove_all_values(&key);
        self.reserve(self.len() + values.len());
        for value in values {
            self.push((key.clone(), value));
        }
    }
    fn remove_all_values(&mut self, key: &str) {
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
                            "Failed to decode base64 encoded picture block: {}",
                            err
                        ));
                    })
                    .map(|decoded| (decoded, issues))
                    .ok()
            })
            .filter_map(|(decoded, mut issues)| {
                metaflac::block::Picture::from_bytes(&decoded[..])
                    .map_err(|err| {
                        issues.push(format!("Failed to decode FLAC picture block: {}", err));
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
        label_values.into_iter().map(ToOwned::to_owned),
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
        writer.write_single_value("REPLAYGAIN_TRACK_GAIN".to_owned(), formatted_track_gain);
    } else {
        writer.remove_all_values("REPLAYGAIN_TRACK_GAIN");
    }
}

pub fn import_encoder(reader: &'_ impl CommentReader) -> Option<Cow<'_, str>> {
    reader.read_first_value("ENCODEDBY").map(Into::into)
}

fn export_encoder(writer: &mut impl CommentWriter, encoder: Option<impl Into<String>>) {
    if let Some(encoder) = encoder.map(Into::into) {
        writer.write_single_value("ENCODEDBY".to_owned(), encoder);
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
        writer.write_single_value("BPM".to_owned(), formatted_bpm);
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
        .read_first_value("INITIALKEY")
        .and_then(|value| importer.import_key_signature(value))
        .or_else(|| {
            reader
                .read_first_value("KEY")
                .and_then(|value| importer.import_key_signature(value))
        })
}

fn export_key_signature(writer: &mut impl CommentWriter, key_signature: KeySignature) {
    if key_signature.is_unknown() {
        writer.remove_all_values("KEY");
    } else {
        // TODO: Write a custom key code string according to config
        writer.write_single_value("KEY".to_owned(), key_signature.to_string());
    }
}

pub fn import_album_kind(
    importer: &mut Importer,
    reader: &impl CommentReader,
) -> Option<AlbumKind> {
    let value = reader.read_first_value("COMPILATION");
    value
        .and_then(|compilation| trim_readable(compilation).parse::<u8>().ok())
        .map(|compilation| match compilation {
            0 => AlbumKind::Unknown, // either Album or Single
            1 => AlbumKind::Compilation,
            _ => {
                importer.add_issue(format!(
                    "Unexpected tag value: COMPILATION = '{}'",
                    value.expect("unreachable")
                ));
                AlbumKind::Unknown
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
            reader
                .read_first_value("PUBLISHER") // primary fallback
                .and_then(trimmed_non_empty_from)
        })
        .or_else(|| {
            reader
                .read_first_value("ORGANIZATION") // secondary fallback
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
                    "Overwriting total number of tracks {} parsed from field '{}' with {}",
                    index_total, "TRACKNUMBER", total,
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
                    "Overwriting total number of discs {} parsed from field '{}' with {}",
                    index_total, "DISCNUMBER", total,
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
                    "Overwriting total number of movements {} parsed from field '{}' with {}",
                    index_total, "MOVEMENT", total,
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

pub fn import_aoide_tags(importer: &mut Importer, reader: &impl CommentReader) -> Option<Tags> {
    let key = AOIDE_TAGS_KEY;
    reader
        .read_first_value(key)
        .and_then(|json| {
            serde_json::from_str::<SerdeTags>(json)
                .map_err(|err| {
                    importer.add_issue(format!("Failed to parse {}: {}", key, err));
                })
                .ok()
        })
        .map(Into::into)
}

pub fn import_serato_markers2(
    importer: &mut Importer,
    reader: &impl CommentReader,
    serato_tags: &mut SeratoTagContainer,
    format: SeratoTagFormat,
) {
    let vorbis_comment = match format {
        SeratoTagFormat::FLAC => SeratoMarkers2::FLAC_COMMENT,
        SeratoTagFormat::Ogg => SeratoMarkers2::OGG_COMMENT,
        _ => {
            return;
        }
    };

    reader.read_first_value(vorbis_comment).and_then(|data| {
        serato_tags
            .parse_markers2(data.as_bytes(), format)
            .map_err(|err| {
                importer.add_issue(format!("Failed to import Serato Markers2: {}", err));
            })
            .ok()
    });
}

pub fn import_into_track(
    importer: &mut Importer,
    reader: &impl CommentReader,
    config: &ImportTrackConfig,
    track: &mut Track,
) -> Result<()> {
    if let Some(tempo_bpm) = import_tempo_bpm(importer, reader) {
        track.metrics.tempo_bpm = Some(tempo_bpm);
    }

    if let Some(key_signature) = import_key_signature(importer, reader) {
        track.metrics.key_signature = key_signature;
    }

    if let Some(recorded_at) = import_recorded_at(importer, reader) {
        track.recorded_at = Some(recorded_at);
    }
    if let Some(released_at) = import_released_at(importer, reader) {
        track.released_at = Some(released_at);
    }
    if let Some(released_orig_at) = import_released_orig_at(importer, reader) {
        track.released_orig_at = Some(released_orig_at);
    }

    if let Some(publisher) = import_publisher(reader) {
        track.publisher = Some(publisher);
    }
    if let Some(copyright) = import_copyright(reader) {
        track.copyright = Some(copyright);
    }

    // Track titles
    let track_titles = import_track_titles(importer, reader);
    if !track_titles.is_empty() {
        track.titles = track_titles;
    }

    // Track actors
    let mut track_actors = Vec::with_capacity(8);
    for name in reader.filter_values("ARTIST").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Artist, name);
    }
    for name in reader.filter_values("ARRANGER").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Arranger, name);
    }
    for name in reader.filter_values("COMPOSER").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Composer, name);
    }
    for name in reader.filter_values("CONDUCTOR").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Conductor, name);
    }
    for name in reader.filter_values("PRODUCER").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Producer, name);
    }
    for name in reader.filter_values("REMIXER").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Remixer, name);
    }
    for name in reader.filter_values("MIXER").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Mixer, name);
    }
    for name in reader.filter_values("DJMIXER").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::DjMixer, name);
    }
    for name in reader.filter_values("ENGINEER").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Engineer, name);
    }
    for name in reader.filter_values("DIRECTOR").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Director, name);
    }
    for name in reader.filter_values("LYRICIST").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Lyricist, name);
    }
    for name in reader.filter_values("WRITER").unwrap_or_default() {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Writer, name);
    }
    let track_actors = importer.finish_import_of_actors(TrackScope::Track, track_actors);
    if !track_actors.is_empty() {
        track.actors = track_actors;
    }

    let mut album = track.album.untie_replace(Default::default());

    // Album titles
    let album_titles = import_album_titles(importer, reader);
    if !album_titles.is_empty() {
        album.titles = album_titles;
    }

    // Album actors
    let mut album_actors = Vec::with_capacity(4);
    for name in reader
        .filter_values("ALBUMARTIST")
        .unwrap_or_default()
        .into_iter()
        .chain(
            reader
                .filter_values("ALBUM_ARTIST")
                .unwrap_or_default()
                .into_iter(),
        )
        .chain(
            reader
                .filter_values("ALBUM ARTIST")
                .unwrap_or_default()
                .into_iter(),
        )
        .chain(
            reader
                .filter_values("ENSEMBLE")
                .unwrap_or_default()
                .into_iter(),
        )
    {
        push_next_actor_role_name_from(&mut album_actors, ActorRole::Artist, name);
    }
    let album_actors = importer.finish_import_of_actors(TrackScope::Album, album_actors);
    if !album_actors.is_empty() {
        album.actors = album_actors;
    }

    // Album properties
    if let Some(album_kind) = import_album_kind(importer, reader) {
        album.kind = album_kind;
    }

    track.album = Canonical::tie(album);

    let mut tags_map = TagsMap::default();
    if config.flags.contains(ImportTrackFlags::CUSTOM_AOIDE_TAGS) {
        // Pre-populate tags
        if let Some(tags) = import_aoide_tags(importer, reader) {
            debug_assert_eq!(0, tags_map.total_count());
            tags_map = tags.into();
        }
    }

    // Comment tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_COMMENT,
        reader
            .filter_values("COMMENT")
            .unwrap_or_default()
            .into_iter(),
    );

    // Description tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_DESCRIPTION,
        reader
            .filter_values("DESCRIPTION")
            .unwrap_or_default()
            .into_iter(),
    );

    // Genre tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_GENRE,
        reader
            .filter_values("GENRE")
            .unwrap_or_default()
            .into_iter(),
    );

    // Mood tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_MOOD,
        reader.filter_values("MOOD").unwrap_or_default().into_iter(),
    );

    // Grouping tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_GROUPING,
        reader
            .filter_values("GROUPING")
            .unwrap_or_default()
            .into_iter(),
    );

    // ISRC tags
    import_faceted_text_tags(
        importer,
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ISRC,
        reader.filter_values("ISRC").unwrap_or_default().into_iter(),
    );

    if let Some(index) = import_track_index(importer, reader) {
        track.indexes.track = index;
    }
    if let Some(index) = import_disc_index(importer, reader) {
        track.indexes.disc = index;
    }
    if let Some(index) = import_movement_index(importer, reader) {
        track.indexes.movement = index;
    }

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

    // Serato Tags
    if config
        .flags
        .contains(ImportTrackFlags::CUSTOM_SERATO_MARKERS)
    {
        let mut serato_tags = SeratoTagContainer::new();
        import_serato_markers2(importer, reader, &mut serato_tags, SeratoTagFormat::Ogg);

        let track_cues = serato::import_cues(&serato_tags);
        if !track_cues.is_empty() {
            track.cues = Canonical::tie(track_cues);
        }

        track.color = serato::import_track_color(&serato_tags);
    }

    Ok(())
}

pub fn export_track(
    config: &ExportTrackConfig,
    track: &mut Track,
    writer: &mut impl CommentWriter,
) {
    // Audio properties
    match &track.media_source.content_metadata {
        ContentMetadata::Audio(audio) => {
            export_loudness(writer, audio.loudness);
            export_encoder(writer, audio.encoder.to_owned());
        }
    }

    export_tempo_bpm(writer, &mut track.metrics.tempo_bpm);
    export_key_signature(writer, track.metrics.key_signature);

    // Track titles
    writer.write_single_value_opt(
        "TITLE".to_owned(),
        Titles::main_title(track.titles.iter()).map(|title| title.name.to_owned()),
    );
    writer.write_multiple_values(
        "SUBTITLE".to_owned(),
        Titles::filter_kind(track.titles.iter(), TitleKind::Sub)
            .map(|title| title.name.to_owned())
            .collect(),
    );
    writer.write_multiple_values(
        "WORK".to_owned(),
        Titles::filter_kind(track.titles.iter(), TitleKind::Work)
            .map(|title| title.name.to_owned())
            .collect(),
    );
    writer.write_multiple_values(
        "MOVEMENTNAME".to_owned(),
        Titles::filter_kind(track.titles.iter(), TitleKind::Movement)
            .map(|title| title.name.to_owned())
            .collect(),
    );

    // Track actors
    export_filtered_actor_names(
        writer,
        "ARTIST".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Artist),
    );
    export_filtered_actor_names(
        writer,
        "ARRANGER".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Arranger),
    );
    export_filtered_actor_names(
        writer,
        "COMPOSER".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Composer),
    );
    export_filtered_actor_names(
        writer,
        "CONDUCTOR".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Conductor),
    );
    export_filtered_actor_names(
        writer,
        "PRODUCER".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Producer),
    );
    export_filtered_actor_names(
        writer,
        "REMIXER".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Remixer),
    );
    export_filtered_actor_names(
        writer,
        "MIXER".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Mixer),
    );
    export_filtered_actor_names(
        writer,
        "DJMIXER".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::DjMixer),
    );
    export_filtered_actor_names(
        writer,
        "ENGINEER".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Engineer),
    );
    export_filtered_actor_names(
        writer,
        "DIRECTOR".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Director),
    );
    export_filtered_actor_names(
        writer,
        "LYRICIST".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Lyricist),
    );
    export_filtered_actor_names(
        writer,
        "WRITER".to_owned(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::Writer),
    );

    // Album
    writer.write_single_value_opt(
        "ALBUM".to_owned(),
        Titles::main_title(track.album.titles.iter()).map(|title| title.name.to_owned()),
    );
    export_filtered_actor_names(
        writer,
        "ALBUMARTIST".to_owned(),
        FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
    );
    match track.album.kind {
        AlbumKind::Unknown => {
            writer.remove_all_values("COMPILATION");
        }
        AlbumKind::Compilation => {
            writer.write_single_value("COMPILATION".to_owned(), "1".to_owned());
        }
        AlbumKind::Album | AlbumKind::Single => {
            writer.write_single_value("COMPILATION".to_owned(), "0".to_owned());
        }
    }

    writer.write_single_value_opt("COPYRIGHT".to_owned(), track.copyright.to_owned());
    writer.write_single_value_opt("LABEL".to_owned(), track.publisher.to_owned());
    writer.write_single_value_opt(
        "DATE".to_owned(),
        track.recorded_at.as_ref().map(ToString::to_string),
    );
    let recorded_year = track
        .recorded_at
        .map(DateYYYYMMDD::from)
        .map(DateYYYYMMDD::year);
    writer.write_single_value_opt(
        "YEAR".to_owned(),
        recorded_year.as_ref().map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "RELEASEDATE".to_owned(),
        track.released_at.as_ref().map(ToString::to_string),
    );
    let released_year = track
        .released_at
        .map(DateYYYYMMDD::from)
        .map(DateYYYYMMDD::year);
    writer.write_single_value_opt(
        "RELEASEYEAR".to_owned(),
        released_year.as_ref().map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "ORIGINALDATE".to_owned(),
        track.released_orig_at.as_ref().map(ToString::to_string),
    );
    let released_orig_year = track
        .released_orig_at
        .map(DateYYYYMMDD::from)
        .map(DateYYYYMMDD::year);
    writer.write_single_value_opt(
        "ORIGINALYEAR".to_owned(),
        released_orig_year.as_ref().map(ToString::to_string),
    );

    // Numbers
    writer.write_single_value_opt(
        "TRACKNUMBER".to_owned(),
        track.indexes.track.number.as_ref().map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "TRACKTOTAL".to_owned(),
        track.indexes.track.total.as_ref().map(ToString::to_string),
    );
    // According to https://wiki.xiph.org/Field_names "TRACKTOTAL" is
    // the proposed field name, but some applications use(d) "TOTALTRACKS".
    writer.remove_all_values("TOTALTRACKS");
    writer.write_single_value_opt(
        "DISCNUMBER".to_owned(),
        track.indexes.disc.number.as_ref().map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "DISCTOTAL".to_owned(),
        track.indexes.disc.total.as_ref().map(ToString::to_string),
    );
    // According to https://wiki.xiph.org/Field_names "DISCTOTAL" is
    // the proposed field name, but some applications use(d) "TOTALDISCS".
    writer.remove_all_values("TOTALDISCS");
    writer.write_single_value_opt(
        "MOVEMENT".to_owned(),
        track
            .indexes
            .movement
            .number
            .as_ref()
            .map(ToString::to_string),
    );
    writer.write_single_value_opt(
        "MOVEMENTTOTAL".to_owned(),
        track
            .indexes
            .movement
            .total
            .as_ref()
            .map(ToString::to_string),
    );

    // Export all tags
    writer.remove_all_values(MIXXX_CUSTOM_TAGS_KEY); // drop legacy key
    if config.flags.contains(ExportTrackFlags::CUSTOM_AOIDE_TAGS) && !track.tags.is_empty() {
        match serde_json::to_string(&aoide_core_json::tag::Tags::from(
            track.tags.clone().untie(),
        )) {
            Ok(value) => {
                writer.write_single_value(AOIDE_TAGS_KEY.to_owned(), value);
            }
            Err(err) => {
                log::warn!("Failed to write {}: {}", AOIDE_TAGS_KEY, err);
            }
        }
    } else {
        writer.remove_all_values(AOIDE_TAGS_KEY);
    }

    // Export selected tags into dedicated fields
    let mut tags_map = TagsMap::from(track.tags.clone().untie());

    // Comment(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_COMMENT) {
        export_faceted_tags(
            writer,
            "COMMENT".to_owned(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values("COMMENT");
    }

    // Description(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_DESCRIPTION) {
        export_faceted_tags(
            writer,
            "DESCRIPTION".to_owned(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values("DESCRIPTION");
    }

    // Genre(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_GENRE) {
        export_faceted_tags(
            writer,
            "GENRE".to_owned(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values("GENRE");
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_MOOD) {
        export_faceted_tags(
            writer,
            "MOOD".to_owned(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values("MOOD");
    }

    // Grouping(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_GROUPING) {
        export_faceted_tags(
            writer,
            "GROUPING".to_owned(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values("GROUPING");
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ISRC) {
        export_faceted_tags(
            writer,
            "ISRC".to_owned(),
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        writer.remove_all_values("ISRC");
    }
}

fn export_filtered_actor_names(
    writer: &mut impl CommentWriter,
    key: String,
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

fn export_faceted_tags(
    writer: &mut impl CommentWriter,
    key: String,
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
