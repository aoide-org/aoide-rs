// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::content::ContentMetadata,
    music::{key::KeySignature, tempo::TempoBpm},
    tag::{FacetKey, FacetedTags, Label, PlainTag, TagsMap},
    track::{
        actor::Role as ActorRole,
        album::Kind as AlbumKind,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_MOOD,
        },
        title::{Kind as TitleKind, Titles},
        Track,
    },
    util::clock::DateYYYYMMDD,
};

use crate::{
    io::export::{ExportTrackConfig, FilteredActorNames},
    util::{
        format_valid_replay_gain, format_validated_tempo_bpm, key_signature_as_str,
        tag::TagMappingConfig, TempoBpmFormat,
    },
};

pub(super) const ARTIST_KEY: &str = "ARTIST";
pub(super) const ARRANGER_KEY: &str = "ARRANGER";
pub(super) const COMPOSER_KEY: &str = "COMPOSER";
pub(super) const CONDUCTOR_KEY: &str = "CONDUCTOR";
pub(super) const REMIXER_KEY: &str = "REMIXER";
pub(super) const MIXER_KEY: &str = "MIXER";
pub(super) const DJMIXER_KEY: &str = "DJMIXER";
pub(super) const ENGINEER_KEY: &str = "ENGINEER";
pub(super) const DIRECTOR_KEY: &str = "DIRECTOR";
pub(super) const LYRICIST_KEY: &str = "LYRICIST";
pub(super) const WRITER_KEY: &str = "WRITER";

pub(super) const COMMENT_KEY: &str = "COMMENT";
pub(super) const COMMENT_KEY2: &str = "DESCRIPTION";

pub(super) const GENRE_KEY: &str = "GENRE";
pub(super) const GROUPING_KEY: &str = "GROUPING";
pub(super) const MOOD_KEY: &str = "MOOD";

pub(super) const ISRC_KEY: &str = "ISRC";

pub(crate) trait CommentWriter {
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
        self.retain(|(cmp_key, _)| cmp_key != key);
    }
}

fn export_loudness(writer: &mut impl CommentWriter, loudness: Option<LoudnessLufs>) {
    if let Some(formatted_track_gain) = loudness.and_then(format_valid_replay_gain) {
        writer.write_single_value("REPLAYGAIN_TRACK_GAIN".into(), formatted_track_gain);
    } else {
        writer.remove_all_values("REPLAYGAIN_TRACK_GAIN");
    }
}

fn export_tempo_bpm(writer: &mut impl CommentWriter, tempo_bpm: &mut Option<TempoBpm>) {
    if let Some(formatted_bpm) = format_validated_tempo_bpm(tempo_bpm, TempoBpmFormat::Float) {
        writer.write_single_value("BPM".into(), formatted_bpm);
    } else {
        writer.remove_all_values("BPM");
    }
    writer.remove_all_values("TEMPO");
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

#[allow(clippy::too_many_lines)] // TODO
pub(crate) fn export_track(
    config: &ExportTrackConfig,
    track: &mut Track,
    writer: &mut impl CommentWriter,
) {
    // Audio properties
    match &track.media_source.content.metadata {
        ContentMetadata::Audio(audio) => {
            export_loudness(writer, audio.loudness);
            // The encoder is a read-only property.
        }
    }

    export_tempo_bpm(writer, &mut track.metrics.tempo_bpm);
    export_key_signature(writer, track.metrics.key_signature);

    // Track titles
    writer.write_single_value_opt(
        "TITLE".into(),
        Titles::main_title(track.titles.iter()).map(|title| title.name.clone()),
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
        FilteredActorNames::new(track.actors.iter(), ActorRole::MixEngineer),
    );
    export_filtered_actor_names(
        writer,
        DJMIXER_KEY.into(),
        FilteredActorNames::new(track.actors.iter(), ActorRole::MixDj),
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
        Titles::main_title(track.album.titles.iter()).map(|title| title.name.clone()),
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
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_COMMENT) {
        export_faceted_tags(
            writer,
            COMMENT_KEY.into(),
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        writer.remove_all_values(COMMENT_KEY);
    }

    // Description(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_DESCRIPTION) {
        export_faceted_tags(
            writer,
            COMMENT_KEY2.into(),
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        writer.remove_all_values(COMMENT_KEY2);
    }

    // Genre(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_GENRE) {
        export_faceted_tags(
            writer,
            GENRE_KEY.into(),
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        writer.remove_all_values(GENRE_KEY);
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_MOOD) {
        export_faceted_tags(
            writer,
            MOOD_KEY.into(),
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        writer.remove_all_values(MOOD_KEY);
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_ISRC) {
        export_faceted_tags(
            writer,
            ISRC_KEY.into(),
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        writer.remove_all_values(ISRC_KEY);
    }

    // Grouping(s)
    {
        let facet_id = FACET_ID_GROUPING;
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
                config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
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
        FilteredActorNames::Individual(names) => {
            writer.write_multiple_values(key, names.into_iter().map(ToOwned::to_owned).collect());
        }
    }
}

fn export_faceted_tags<'a>(
    writer: &mut impl CommentWriter,
    key: Cow<'a, str>,
    config: Option<&TagMappingConfig>,
    tags: Vec<PlainTag<'_>>,
) {
    if let Some(config) = config {
        let joined_labels = config.join_labels(
            tags.into_iter()
                .filter_map(|PlainTag { label, score: _ }| label.map(Label::into_inner)),
        );
        writer.write_single_value_opt(key, joined_labels.map(Into::into));
    } else {
        let tag_labels = tags
            .into_iter()
            .map(|tag| tag.label.unwrap_or_default().into_inner().into_owned())
            .collect();
        writer.write_multiple_values(key, tag_labels);
    }
}
