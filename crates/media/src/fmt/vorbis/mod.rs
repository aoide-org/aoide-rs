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

use std::borrow::Cow;

use semval::IsValid as _;
use triseratops::tag::{
    format::flac::FLACTag, format::ogg::OggTag, Markers2 as SeratoMarkers2,
    TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat,
};

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::{concat_encoder_properties, Content},
    music::{key::KeySignature, time::TempoBpm},
    tag::{FacetId, FacetedTags, PlainTag, Tags, TagsMap},
    track::{
        actor::ActorRole,
        album::AlbumKind,
        index::Index,
        release::DateOrDateTime,
        tag::{FACET_COMMENT, FACET_GENRE, FACET_GROUPING, FACET_ISRC, FACET_MOOD},
        title::{Title, TitleKind, Titles},
        Track,
    },
    util::CanonicalizeInto as _,
};

use aoide_core_serde::tag::Tags as SerdeTags;

use crate::{
    io::export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
    util::{
        format_valid_replay_gain, format_validated_tempo_bpm, parse_index_numbers,
        parse_key_signature, parse_replay_gain, parse_tempo_bpm, parse_year_tag,
        tag::{
            import_faceted_tags_from_label_value_iter, FacetedTagMappingConfig, TagMappingConfig,
        },
    },
};

pub const MIXXX_CUSTOM_TAGS_KEY: &str = "MIXXX_CUSTOM_TAGS";

pub const AOIDE_TAGS_KEY: &str = "AOIDE_TAGS";

pub trait CommentReader {
    fn read_first_value(&self, key: &str) -> Option<&str>;
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

pub fn import_faceted_text_tags<'a>(
    tags_map: &mut TagsMap,
    faceted_tag_mapping_config: &FacetedTagMappingConfig,
    facet_id: &FacetId,
    label_values: impl Iterator<Item = &'a str>,
) {
    import_faceted_tags_from_label_value_iter(
        tags_map,
        faceted_tag_mapping_config,
        facet_id,
        label_values.map(ToOwned::to_owned),
    );
}

pub fn import_loudness(reader: &impl CommentReader) -> Option<LoudnessLufs> {
    reader
        .read_first_value("REPLAYGAIN_TRACK_GAIN")
        .and_then(parse_replay_gain)
}

fn export_loudness(writer: &mut impl CommentWriter, loudness: Option<LoudnessLufs>) {
    if let Some(formatted_track_gain) = loudness.map(format_valid_replay_gain).flatten() {
        writer.write_single_value("REPLAYGAIN_TRACK_GAIN".to_owned(), formatted_track_gain);
    } else {
        writer.remove_all_values("REPLAYGAIN_TRACK_GAIN");
    }
}

pub fn import_encoder(reader: &'_ impl CommentReader) -> Option<Cow<'_, str>> {
    concat_encoder_properties(
        reader.read_first_value("ENCODEDBY"),
        reader.read_first_value("ENCODERSETTINGS"),
    )
}

fn export_encoder(writer: &mut impl CommentWriter, encoder: Option<impl Into<String>>) {
    if let Some(encoder) = encoder.map(Into::into) {
        writer.write_single_value("ENCODEDBY".to_owned(), encoder);
    } else {
        writer.remove_all_values("ENCODEDBY");
    }
    // ENCODEDBY and ENCODERSETTINGS have been joined during import
    writer.remove_all_values("ENCODERSETTINGS");
}

pub fn import_tempo_bpm(reader: &impl CommentReader) -> Option<TempoBpm> {
    if let Some(tempo_bpm) = reader
        .read_first_value("BPM")
        .and_then(parse_tempo_bpm)
        // Alternative: Try "TEMPO" if "BPM" is missing or invalid
        .or_else(|| reader.read_first_value("TEMPO").and_then(parse_tempo_bpm))
    {
        debug_assert!(tempo_bpm.is_valid());
        Some(tempo_bpm)
    } else {
        None
    }
}

fn export_tempo_bpm(writer: &mut impl CommentWriter, tempo_bpm: &mut Option<TempoBpm>) {
    if let Some(formatted_bpm) = format_validated_tempo_bpm(tempo_bpm) {
        writer.write_single_value("BPM".to_owned(), formatted_bpm);
    } else {
        writer.remove_all_values("BPM");
    }
    writer.remove_all_values("TEMPO");
}

pub fn import_key_signature(reader: &impl CommentReader) -> Option<KeySignature> {
    reader
        .read_first_value("INITIALKEY")
        .and_then(parse_key_signature)
        .or_else(|| reader.read_first_value("KEY").and_then(parse_key_signature))
}

fn export_key_signature(writer: &mut impl CommentWriter, key_signature: KeySignature) {
    if key_signature.is_unknown() {
        writer.remove_all_values("KEY");
    } else {
        // TODO: Write a custom key code string according to config
        writer.write_single_value("KEY".to_owned(), key_signature.to_string());
    }
}

pub fn import_album_kind(reader: &impl CommentReader) -> Option<AlbumKind> {
    if reader
        .read_first_value("COMPILATION")
        .and_then(|compilation| compilation.parse::<u8>().ok())
        .unwrap_or_default()
        == 1
    {
        Some(AlbumKind::Compilation)
    } else {
        None
    }
}

pub fn import_released_at(reader: &impl CommentReader) -> Option<DateOrDateTime> {
    reader.read_first_value("DATE").and_then(parse_year_tag)
}

pub fn import_released_by(reader: &impl CommentReader) -> Option<String> {
    reader.read_first_value("LABEL").map(ToOwned::to_owned)
}

pub fn import_release_copyright(reader: &impl CommentReader) -> Option<String> {
    reader.read_first_value("COPYRIGHT").map(ToOwned::to_owned)
}

pub fn import_track_index(reader: &impl CommentReader) -> Option<Index> {
    if let Some(mut index) = reader
        .read_first_value("TRACKNUMBER")
        .and_then(parse_index_numbers)
    {
        if index.total.is_none() {
            // According to https://wiki.xiph.org/Field_names "TRACKTOTAL" is
            // the proposed field name, but some applications use "TOTALTRACKS".
            index.total = reader
                .read_first_value("TRACKTOTAL")
                .and_then(|input| input.parse().ok())
                .or_else(|| {
                    reader
                        .read_first_value("TOTALTRACKS")
                        .and_then(|input| input.parse().ok())
                });
        }
        Some(index)
    } else {
        None
    }
}

pub fn import_disc_index(reader: &impl CommentReader) -> Option<Index> {
    if let Some(mut index) = reader
        .read_first_value("DISCNUMBER")
        .and_then(parse_index_numbers)
    {
        if index.total.is_none() {
            // According to https://wiki.xiph.org/Field_names "DISCTOTAL" is
            // the proposed field name, but some applications use "TOTALDISCS".
            index.total = reader
                .read_first_value("DISCTOTAL")
                .and_then(|input| input.parse().ok())
                .or_else(|| {
                    reader
                        .read_first_value("TOTALDISCS")
                        .and_then(|input| input.parse().ok())
                });
        }
        Some(index)
    } else {
        None
    }
}

pub fn import_movement_index(reader: &impl CommentReader) -> Option<Index> {
    if let Some(mut index) = reader
        .read_first_value("MOVEMENT")
        .and_then(parse_index_numbers)
    {
        if index.total.is_none() {
            index.total = reader
                .read_first_value("MOVEMENTTOTAL")
                .and_then(|input| input.parse().ok());
        }
        Some(index)
    } else {
        None
    }
}

pub fn import_track_titles(reader: &impl CommentReader) -> Vec<Title> {
    let mut track_titles = Vec::with_capacity(4);
    if let Some(name) = reader.read_first_value("TITLE") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Main,
        };
        track_titles.push(title);
    }
    if let Some(name) = reader.read_first_value("SUBTITLE") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Sub,
        };
        track_titles.push(title);
    }
    if let Some(name) = reader.read_first_value("WORK") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Work,
        };
        track_titles.push(title);
    }
    if let Some(name) = reader.read_first_value("MOVEMENTNAME") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Movement,
        };
        track_titles.push(title);
    }
    track_titles.canonicalize_into()
}

pub fn import_album_titles(reader: &impl CommentReader) -> Vec<Title> {
    let mut album_titles = Vec::with_capacity(1);
    if let Some(name) = reader.read_first_value("ALBUM") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Main,
        };
        album_titles.push(title);
    }
    album_titles.canonicalize_into()
}

pub fn import_aoide_tags(reader: &impl CommentReader) -> Option<Tags> {
    reader
        .read_first_value(AOIDE_TAGS_KEY)
        .and_then(|json| {
            serde_json::from_str::<SerdeTags>(json)
                .map_err(|err| {
                    tracing::warn!("Failed to parse {}: {}", AOIDE_TAGS_KEY, err);
                    err
                })
                .ok()
        })
        .map(Into::into)
}

pub fn import_serato_markers2(
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

    reader
        .read_first_value(vorbis_comment)
        .and_then(|data| serato_tags.parse_markers2(data.as_bytes(), format).ok());
}

pub fn export_track(
    config: &ExportTrackConfig,
    track: &mut Track,
    writer: &mut impl CommentWriter,
) {
    // Audio properties
    match &track.media_source.content {
        Content::Audio(audio) => {
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

    // Release
    writer.write_single_value_opt("COPYRIGHT".to_owned(), track.release.copyright.to_owned());
    writer.write_single_value_opt("LABEL".to_owned(), track.release.released_by.to_owned());
    writer.write_single_value_opt(
        "DATE".to_owned(),
        track.release.released_at.as_ref().map(ToString::to_string),
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
    if config.flags.contains(ExportTrackFlags::AOIDE_TAGS) && !track.tags.is_empty() {
        match serde_json::to_string(&aoide_core_serde::tag::Tags::from(
            track.tags.clone().untie(),
        )) {
            Ok(value) => {
                writer.write_single_value(AOIDE_TAGS_KEY.to_owned(), value);
            }
            Err(err) => {
                tracing::warn!("Failed to write {}: {}", AOIDE_TAGS_KEY, err);
            }
        }
    } else {
        writer.remove_all_values(AOIDE_TAGS_KEY);
    }

    // Export selected tags into dedicated fields
    let mut tags_map = TagsMap::from(track.tags.clone().untie());

    // Comment(s)
    // The original specification only defines a "DESCRIPTION" field,
    // while MusicBrainz recommends to use "COMMENT".
    // http://www.xiph.org/vorbis/doc/v-comment.html
    // https://picard.musicbrainz.org/docs/mappings
    writer.remove_all_values("DESCRIPTION");
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
        let joined_labels = config.join_labels_str_iter(
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
