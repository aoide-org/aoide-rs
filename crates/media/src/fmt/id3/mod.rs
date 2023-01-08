// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use id3::{
    self,
    frame::{Comment, ExtendedText},
    TagLike as _,
};

use aoide_core::{
    media::content::ContentMetadata,
    tag::{FacetKey, FacetedTags, Label, PlainTag, TagsMap},
    track::{
        actor::Role as ActorRole,
        album::Kind as AlbumKind,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_LANGUAGE, FACET_ID_MOOD,
        },
        title::{Kind as TitleKind, Titles},
        Track,
    },
    util::clock::DateOrDateTime,
};

use crate::{
    io::export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
    util::{
        format_valid_replay_gain, format_validated_tempo_bpm, key_signature_as_str,
        tag::TagMappingConfig,
    },
    Error,
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

#[derive(Debug)]
pub(crate) enum ExportError {
    UnsupportedLegacyVersion(id3::Version),
}

#[allow(clippy::too_many_lines)] // TODO
pub(crate) fn export_track(
    config: &ExportTrackConfig,
    track: &mut Track,
    tag: &mut id3::Tag,
) -> std::result::Result<(), ExportError> {
    if tag.version() != id3::Version::Id3v24 {
        return Err(ExportError::UnsupportedLegacyVersion(tag.version()));
    }

    // Audio properties
    match &track.media_source.content.metadata {
        ContentMetadata::Audio(audio) => {
            if let Some(formatted_track_gain) = audio.loudness.and_then(format_valid_replay_gain) {
                tag.add_frame(ExtendedText {
                    description: "REPLAYGAIN_TRACK_GAIN".to_owned(),
                    value: formatted_track_gain,
                });
            } else {
                tag.remove_extended_text(Some("REPLAYGAIN_TRACK_GAIN"), None);
            }
            // The encoder is a read-only property.
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

    // Musical key
    if let Some(key_signature) = track.metrics.key_signature {
        tag.set_text("TKEY", key_signature_as_str(key_signature));
    } else {
        tag.remove("TKEY");
    }

    // Track titles
    if let Some(title) = Titles::main_title(track.titles.iter()) {
        tag.set_title(title.name.clone());
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
        tag.set_album(title.name.clone());
    } else {
        tag.remove_album();
    }
    export_filtered_actor_names(
        tag,
        "TPE2",
        FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
    );
    if let Some(kind) = track.album.kind {
        match kind {
            AlbumKind::NoCompilation | AlbumKind::Album | AlbumKind::Single => {
                tag.set_text("TCMP", "0");
            }
            AlbumKind::Compilation => {
                tag.set_text("TCMP", "1");
            }
        }
    } else {
        tag.remove("TCMP");
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
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_COMMENT) {
        export_faceted_tags_comment(
            tag,
            String::new(),
            config.faceted_tag_mapping.get(facet_id.as_str()),
            &tags,
        );
    } else {
        export_faceted_tags_comment(tag, String::new(), None, &[]);
    }

    // Description(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_DESCRIPTION) {
        export_faceted_tags_comment(
            tag,
            "description",
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            &tags,
        );
    } else {
        export_faceted_tags_comment(tag, "description", None, &[]);
    }

    // Genre(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_GENRE) {
        export_faceted_tags(
            tag,
            "TCON",
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            &tags,
        );
    } else {
        export_faceted_tags(tag, "TCON", None, &[]);
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_MOOD) {
        export_faceted_tags(
            tag,
            "TMOO",
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            &tags,
        );
    } else {
        export_faceted_tags(tag, "TMOO", None, &[]);
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_ISRC) {
        export_faceted_tags(
            tag,
            "TSRC",
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            &tags,
        );
    } else {
        export_faceted_tags(tag, "TSRC", None, &[]);
    }

    // Language(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_LANGUAGE) {
        export_faceted_tags(
            tag,
            "TLAN",
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            &tags,
        );
    } else {
        export_faceted_tags(tag, "TLAN", None, &[]);
    }

    // Grouping(s)
    {
        let facet_id = FACET_ID_GROUPING;
        let mut tags = tags_map
            .take_faceted_tags(facet_id)
            .map(|FacetedTags { facet_id: _, tags }| tags)
            .unwrap_or_default();
        #[cfg(feature = "gigtag")]
        if config.flags.contains(ExportTrackFlags::GIGTAGS) {
            if let Err(err) = crate::util::gigtag::export_and_encode_remaining_tags_into(
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
                config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
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
                    year: i32::from(date.year()),
                    month: None,
                    day: None,
                    hour: None,
                    minute: None,
                    second: None,
                }
            } else {
                id3::Timestamp {
                    year: i32::from(date.year()),
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
    tags: &[PlainTag<'_>],
) {
    let joined_labels = if let Some(config) = config {
        config.join_labels(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(Label::as_str)),
        )
    } else {
        TagMappingConfig::join_labels_with_separator(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(Label::as_str)),
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
    tags: &[PlainTag<'_>],
) {
    let joined_labels = if let Some(config) = config {
        config.join_labels(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(Label::as_str)),
        )
    } else {
        TagMappingConfig::join_labels_with_separator(
            tags.iter()
                .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(Label::as_str)),
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
