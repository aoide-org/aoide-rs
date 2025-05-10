// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::prelude::*;

use aoide_core::{
    EntityRevision, Track, TrackBody, TrackEntity, TrackHeader,
    music::{
        key::KeySignature,
        tempo::{TempoBpm, TempoBpmValue},
    },
    track::{Actors, Titles, actor, album::Album, index::*, metric::*},
    util::{clock::*, color::*},
};
use aoide_repo::media::source::RecordId as MediaSourceId;

use crate::{
    RowId,
    util::entity::{encode_entity_revision, encode_entity_uid},
};

use super::{encode_advisory_rating, encode_album_kind, encode_music_key_code, schema::*};

#[derive(Debug, Insertable)]
#[diesel(table_name = track)]
pub(crate) struct InsertableRecord<'a> {
    pub(crate) row_created_ms: TimestampMillis,
    pub(crate) row_updated_ms: TimestampMillis,
    pub(crate) entity_uid: String,
    pub(crate) entity_rev: i64,
    pub(crate) media_source_id: RowId,
    pub(crate) last_synchronized_rev: Option<i64>,
    pub(crate) recorded_at: Option<String>,
    pub(crate) recorded_ms: Option<TimestampMillis>,
    pub(crate) recorded_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub(crate) released_at: Option<String>,
    pub(crate) released_ms: Option<TimestampMillis>,
    pub(crate) released_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub(crate) released_orig_at: Option<String>,
    pub(crate) released_orig_ms: Option<TimestampMillis>,
    pub(crate) released_orig_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub(crate) publisher: Option<&'a str>,
    pub(crate) copyright: Option<&'a str>,
    pub(crate) advisory_rating: Option<i16>,
    pub(crate) album_kind: Option<i16>,
    pub(crate) track_number: Option<i16>,
    pub(crate) track_total: Option<i16>,
    pub(crate) disc_number: Option<i16>,
    pub(crate) disc_total: Option<i16>,
    pub(crate) movement_number: Option<i16>,
    pub(crate) movement_total: Option<i16>,
    pub(crate) music_tempo_bpm: Option<TempoBpmValue>,
    pub(crate) music_key_code: Option<i16>,
    pub(crate) music_beats_per_measure: Option<i16>,
    pub(crate) music_beat_unit: Option<i16>,
    pub(crate) music_flags: i16,
    pub(crate) color_rgb: Option<i32>,
    pub(crate) color_idx: Option<i16>,
    pub(crate) sort_track_artist: Option<&'a str>,
    pub(crate) sort_album_artist: Option<&'a str>,
    pub(crate) sort_track_title: Option<&'a str>,
    pub(crate) sort_album_title: Option<&'a str>,
}

impl<'a> InsertableRecord<'a> {
    #[expect(clippy::too_many_lines)] // TODO
    pub(crate) fn bind(media_source_id: MediaSourceId, entity: &'a TrackEntity) -> Self {
        let TrackHeader { uid, rev } = &entity.hdr;
        let TrackBody {
            track,
            updated_at,
            last_synchronized_rev,
            content_url: _,
        } = &entity.body;
        let row_created_updated_ms = updated_at.unix_timestamp_millis();
        let Track {
            media_source: _,
            recorded_at,
            released_at,
            released_orig_at,
            publisher,
            copyright,
            advisory_rating,
            album,
            actors: track_actors,
            titles: track_titles,
            indexes,
            metrics,
            color,
            cues: _,
            tags: _,
        } = track;
        let (recorded_at_yyyymmdd, recorded_at) =
            recorded_at
                .as_ref()
                .map_or((None, None), |recorded_at| match recorded_at {
                    DateOrDateTime::Date(date) => (Some(*date), None),
                    DateOrDateTime::DateTime(dt) => {
                        (Some(YyyyMmDdDate::from_date(dt.date())), Some(dt))
                    }
                });
        let (released_at_yyyymmdd, released_at) =
            released_at
                .as_ref()
                .map_or((None, None), |released_at| match released_at {
                    DateOrDateTime::Date(date) => (Some(*date), None),
                    DateOrDateTime::DateTime(dt) => {
                        (Some(YyyyMmDdDate::from_date(dt.date())), Some(dt))
                    }
                });
        let (released_orig_at_yyyymmdd, released_orig_at) = released_orig_at.as_ref().map_or(
            (None, None),
            |released_orig_at| match released_orig_at {
                DateOrDateTime::Date(date) => (Some(*date), None),
                DateOrDateTime::DateTime(dt) => {
                    (Some(YyyyMmDdDate::from_date(dt.date())), Some(dt))
                }
            },
        );
        let Album {
            actors: album_actors,
            titles: album_titles,
            kind: album_kind,
        } = album.as_ref();
        let Indexes {
            track: track_index,
            disc: disc_index,
            movement: movement_index,
        } = indexes;
        let Metrics {
            tempo_bpm,
            key_signature,
            time_signature,
            flags: music_flags,
        } = metrics;
        let sort_track_artist =
            Actors::sort_or_summary_actor(track_actors.iter(), actor::Role::Artist);
        let sort_album_artist =
            Actors::sort_or_summary_actor(album_actors.iter(), actor::Role::Artist);
        let sort_track_title = Titles::sort_or_main_title(track_titles.iter());
        let sort_album_title = Titles::sort_or_main_title(album_titles.iter());
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            entity_uid: encode_entity_uid(uid),
            entity_rev: encode_entity_revision(*rev),
            media_source_id: media_source_id.into(),
            last_synchronized_rev: last_synchronized_rev.map(encode_entity_revision),
            recorded_at: recorded_at.as_ref().map(ToString::to_string),
            recorded_ms: recorded_at.map(OffsetDateTimeMs::unix_timestamp_millis),
            recorded_at_yyyymmdd: recorded_at_yyyymmdd.map(YyyyMmDdDate::value),
            released_at: released_at.as_ref().map(ToString::to_string),
            released_ms: released_at.map(OffsetDateTimeMs::unix_timestamp_millis),
            released_at_yyyymmdd: released_at_yyyymmdd.map(YyyyMmDdDate::value),
            released_orig_at: released_orig_at.as_ref().map(ToString::to_string),
            released_orig_ms: released_orig_at.map(OffsetDateTimeMs::unix_timestamp_millis),
            released_orig_at_yyyymmdd: released_orig_at_yyyymmdd.map(YyyyMmDdDate::value),
            publisher: publisher.as_ref().map(String::as_str),
            copyright: copyright.as_ref().map(String::as_str),
            advisory_rating: advisory_rating.map(encode_advisory_rating),
            album_kind: album_kind.map(encode_album_kind),
            track_number: track_index.number.map(|idx| idx as i16),
            track_total: track_index.total.map(|idx| idx as i16),
            disc_number: disc_index.number.map(|idx| idx as i16),
            disc_total: disc_index.total.map(|idx| idx as i16),
            movement_number: movement_index.number.map(|idx| idx as i16),
            movement_total: movement_index.total.map(|idx| idx as i16),
            music_tempo_bpm: tempo_bpm.map(TempoBpm::value),
            music_key_code: key_signature
                .map(KeySignature::code)
                .map(encode_music_key_code),
            music_beats_per_measure: time_signature
                .map(|time_sig| time_sig.beats_per_measure as i16),
            music_beat_unit: time_signature
                .and_then(|time_sig| time_sig.beat_unit)
                .map(|beat_unit| beat_unit as i16),
            music_flags: i16::from(music_flags.bits()),
            color_rgb: if let Some(Color::Rgb(color)) = color {
                Some(color.code() as i32)
            } else {
                None
            },
            color_idx: if let Some(Color::Index(index)) = color {
                Some(*index)
            } else {
                None
            },
            sort_track_artist: sort_track_artist.map(|actor| actor.name.as_str()),
            sort_album_artist: sort_album_artist.map(|actor| actor.name.as_str()),
            sort_track_title: sort_track_title.map(|title| title.name.as_str()),
            sort_album_title: sort_album_title.map(|title| title.name.as_str()),
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = track, treat_none_as_null = true)]
pub(crate) struct UpdatableRecord<'a> {
    pub(crate) row_updated_ms: TimestampMillis,
    pub(crate) entity_rev: i64,
    pub(crate) media_source_id: RowId,
    pub(crate) last_synchronized_rev: Option<i64>,
    pub(crate) recorded_at: Option<String>,
    pub(crate) recorded_ms: Option<TimestampMillis>,
    pub(crate) recorded_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub(crate) released_at: Option<String>,
    pub(crate) released_ms: Option<TimestampMillis>,
    pub(crate) released_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub(crate) released_orig_at: Option<String>,
    pub(crate) released_orig_ms: Option<TimestampMillis>,
    pub(crate) released_orig_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub(crate) publisher: Option<&'a str>,
    pub(crate) copyright: Option<&'a str>,
    pub(crate) advisory_rating: Option<i16>,
    pub(crate) album_kind: Option<i16>,
    pub(crate) track_number: Option<i16>,
    pub(crate) track_total: Option<i16>,
    pub(crate) disc_number: Option<i16>,
    pub(crate) disc_total: Option<i16>,
    pub(crate) movement_number: Option<i16>,
    pub(crate) movement_total: Option<i16>,
    pub(crate) music_tempo_bpm: Option<TempoBpmValue>,
    pub(crate) music_key_code: Option<i16>,
    pub(crate) music_beats_per_measure: Option<i16>,
    pub(crate) music_beat_unit: Option<i16>,
    pub(crate) music_flags: i16,
    pub(crate) color_rgb: Option<i32>,
    pub(crate) color_idx: Option<i16>,
    pub(crate) sort_track_artist: Option<&'a str>,
    pub(crate) sort_album_artist: Option<&'a str>,
    pub(crate) sort_track_title: Option<&'a str>,
    pub(crate) sort_album_title: Option<&'a str>,
}

impl<'a> UpdatableRecord<'a> {
    #[expect(clippy::too_many_lines)] // TODO
    pub(crate) fn bind(
        next_rev: EntityRevision,
        media_source_id: MediaSourceId,
        entity_body: &'a TrackBody,
    ) -> Self {
        let entity_rev = encode_entity_revision(next_rev);
        let TrackBody {
            track,
            updated_at,
            last_synchronized_rev,
            content_url: _,
        } = entity_body;
        let Track {
            media_source: _,
            recorded_at,
            released_at,
            released_orig_at,
            publisher,
            copyright,
            advisory_rating,
            album,
            actors: track_actors,
            titles: track_titles,
            indexes,
            metrics,
            color,
            cues: _,
            tags: _,
        } = track;
        let (recorded_at_yyyymmdd, recorded_at) =
            recorded_at
                .as_ref()
                .map_or((None, None), |recorded_at| match recorded_at {
                    DateOrDateTime::Date(date) => (Some(*date), None),
                    DateOrDateTime::DateTime(dt) => {
                        (Some(YyyyMmDdDate::from_date(dt.date())), Some(dt))
                    }
                });
        let (released_at_yyyymmdd, released_at) =
            released_at
                .as_ref()
                .map_or((None, None), |released_at| match released_at {
                    DateOrDateTime::Date(date) => (Some(*date), None),
                    DateOrDateTime::DateTime(dt) => {
                        (Some(YyyyMmDdDate::from_date(dt.date())), Some(dt))
                    }
                });
        let (released_orig_at_yyyymmdd, released_orig_at) = released_orig_at.as_ref().map_or(
            (None, None),
            |released_orig_at| match released_orig_at {
                DateOrDateTime::Date(date) => (Some(*date), None),
                DateOrDateTime::DateTime(dt) => {
                    (Some(YyyyMmDdDate::from_date(dt.date())), Some(dt))
                }
            },
        );
        let Album {
            actors: album_actors,
            titles: album_titles,
            kind: album_kind,
        } = album.as_ref();
        let Indexes {
            track: track_index,
            disc: disc_index,
            movement: movement_index,
        } = indexes;
        let Metrics {
            tempo_bpm,
            key_signature,
            time_signature,
            flags: music_flags,
        } = metrics;
        let sort_track_artist =
            Actors::sort_or_summary_actor(track_actors.iter(), actor::Role::Artist);
        let sort_album_artist =
            Actors::sort_or_summary_actor(album_actors.iter(), actor::Role::Artist);
        let sort_track_title = Titles::sort_or_main_title(track_titles.iter());
        let sort_album_title = Titles::sort_or_main_title(album_titles.iter());
        Self {
            row_updated_ms: updated_at.unix_timestamp_millis(),
            entity_rev,
            media_source_id: media_source_id.into(),
            last_synchronized_rev: last_synchronized_rev.map(encode_entity_revision),
            recorded_at: recorded_at.as_ref().map(ToString::to_string),
            recorded_ms: recorded_at.map(OffsetDateTimeMs::unix_timestamp_millis),
            recorded_at_yyyymmdd: recorded_at_yyyymmdd.map(YyyyMmDdDate::value),
            released_at: released_at.as_ref().map(ToString::to_string),
            released_ms: released_at.map(OffsetDateTimeMs::unix_timestamp_millis),
            released_at_yyyymmdd: released_at_yyyymmdd.map(YyyyMmDdDate::value),
            released_orig_at: released_orig_at.as_ref().map(ToString::to_string),
            released_orig_ms: released_orig_at.map(OffsetDateTimeMs::unix_timestamp_millis),
            released_orig_at_yyyymmdd: released_orig_at_yyyymmdd.map(YyyyMmDdDate::value),
            publisher: publisher.as_ref().map(String::as_str),
            copyright: copyright.as_ref().map(String::as_str),
            advisory_rating: advisory_rating.map(encode_advisory_rating),
            album_kind: album_kind.map(encode_album_kind),
            track_number: track_index.number.map(|number| number as i16),
            track_total: track_index.total.map(|total| total as i16),
            disc_number: disc_index.number.map(|number| number as i16),
            disc_total: disc_index.total.map(|total| total as i16),
            movement_number: movement_index.number.map(|number| number as i16),
            movement_total: movement_index.total.map(|total| total as i16),
            music_tempo_bpm: tempo_bpm.map(TempoBpm::value),
            music_key_code: key_signature
                .map(KeySignature::code)
                .map(encode_music_key_code),
            music_beats_per_measure: time_signature
                .map(|time_sig| time_sig.beats_per_measure as i16),
            music_beat_unit: time_signature
                .and_then(|time_sig| time_sig.beat_unit)
                .map(|beat_unit| beat_unit as i16),
            music_flags: i16::from(music_flags.bits()),
            color_rgb: if let Some(Color::Rgb(color)) = color {
                Some(color.code() as i32)
            } else {
                None
            },
            color_idx: if let Some(Color::Index(index)) = color {
                Some(*index)
            } else {
                None
            },
            sort_track_artist: sort_track_artist.map(|actor| actor.name.as_str()),
            sort_album_artist: sort_album_artist.map(|actor| actor.name.as_str()),
            sort_track_title: sort_track_title.map(|title| title.name.as_str()),
            sort_album_title: sort_album_title.map(|title| title.name.as_str()),
        }
    }
}
