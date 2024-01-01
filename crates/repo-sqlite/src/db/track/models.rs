// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    music::{
        key::KeySignature,
        tempo::{TempoBpm, TempoBpmValue},
    },
    track::{album::Album, index::*, metric::*},
    util::{clock::*, color::*},
    EntityRevision, Track, TrackBody, TrackEntity, TrackHeader,
};
use aoide_repo::media::source::RecordId as MediaSourceId;

use super::{encode_advisory_rating, encode_album_kind, encode_music_key_code, schema::*};
use crate::prelude::*;

#[derive(Debug, Insertable)]
#[diesel(table_name = track)]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: String,
    pub entity_rev: i64,
    pub media_source_id: RowId,
    pub last_synchronized_rev: Option<i64>,
    pub recorded_at: Option<String>,
    pub recorded_ms: Option<TimestampMillis>,
    pub recorded_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub released_at: Option<String>,
    pub released_ms: Option<TimestampMillis>,
    pub released_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub released_orig_at: Option<String>,
    pub released_orig_ms: Option<TimestampMillis>,
    pub released_orig_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub publisher: Option<&'a str>,
    pub copyright: Option<&'a str>,
    pub advisory_rating: Option<i16>,
    pub album_kind: Option<i16>,
    pub track_number: Option<i16>,
    pub track_total: Option<i16>,
    pub disc_number: Option<i16>,
    pub disc_total: Option<i16>,
    pub movement_number: Option<i16>,
    pub movement_total: Option<i16>,
    pub music_tempo_bpm: Option<TempoBpmValue>,
    pub music_key_code: Option<i16>,
    pub music_beats_per_measure: Option<i16>,
    pub music_beat_unit: Option<i16>,
    pub music_flags: i16,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
}

impl<'a> InsertableRecord<'a> {
    #[allow(clippy::too_many_lines)] // TODO
    pub fn bind(media_source_id: MediaSourceId, entity: &'a TrackEntity) -> Self {
        let TrackHeader { uid, rev } = &entity.hdr;
        let TrackBody {
            track,
            updated_at,
            last_synchronized_rev,
            content_url: _,
        } = &entity.body;
        let row_created_updated_ms = updated_at.timestamp_millis();
        let Track {
            media_source: _,
            recorded_at,
            released_at,
            released_orig_at,
            publisher,
            copyright,
            advisory_rating,
            album,
            actors: _,
            titles: _,
            indexes,
            metrics,
            color,
            cues: _,
            tags: _,
        } = track;
        let (recorded_at_yyyymmdd, recorded_at) =
            recorded_at.map_or((None, None), |recorded_at| match recorded_at {
                DateOrDateTime::Date(date) => (Some(date), None),
                DateOrDateTime::DateTime(dt) => (Some(dt.into()), Some(dt)),
            });
        let (released_at_yyyymmdd, released_at) =
            released_at.map_or((None, None), |released_at| match released_at {
                DateOrDateTime::Date(date) => (Some(date), None),
                DateOrDateTime::DateTime(dt) => (Some(dt.into()), Some(dt)),
            });
        let (released_orig_at_yyyymmdd, released_orig_at) =
            released_orig_at.map_or((None, None), |released_orig_at| match released_orig_at {
                DateOrDateTime::Date(date) => (Some(date), None),
                DateOrDateTime::DateTime(dt) => (Some(dt.into()), Some(dt)),
            });
        let Album {
            actors: _,
            titles: _,
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
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            entity_uid: encode_entity_uid(uid),
            entity_rev: encode_entity_revision(*rev),
            media_source_id: media_source_id.into(),
            last_synchronized_rev: last_synchronized_rev.map(encode_entity_revision),
            recorded_at: recorded_at.as_ref().map(ToString::to_string),
            recorded_ms: recorded_at.map(OffsetDateTimeMs::timestamp_millis),
            recorded_at_yyyymmdd: recorded_at_yyyymmdd.map(YyyyMmDdDate::value),
            released_at: released_at.as_ref().map(ToString::to_string),
            released_ms: released_at.map(OffsetDateTimeMs::timestamp_millis),
            released_at_yyyymmdd: released_at_yyyymmdd.map(YyyyMmDdDate::value),
            released_orig_at: released_orig_at.as_ref().map(ToString::to_string),
            released_orig_ms: released_orig_at.map(OffsetDateTimeMs::timestamp_millis),
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
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = track, treat_none_as_null = true)]
pub struct UpdatableRecord<'a> {
    pub row_updated_ms: TimestampMillis,
    pub entity_rev: i64,
    pub media_source_id: RowId,
    pub last_synchronized_rev: Option<i64>,
    pub recorded_at: Option<String>,
    pub recorded_ms: Option<TimestampMillis>,
    pub recorded_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub released_at: Option<String>,
    pub released_ms: Option<TimestampMillis>,
    pub released_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub released_orig_at: Option<String>,
    pub released_orig_ms: Option<TimestampMillis>,
    pub released_orig_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub publisher: Option<&'a str>,
    pub copyright: Option<&'a str>,
    pub advisory_rating: Option<i16>,
    pub album_kind: Option<i16>,
    pub track_number: Option<i16>,
    pub track_total: Option<i16>,
    pub disc_number: Option<i16>,
    pub disc_total: Option<i16>,
    pub movement_number: Option<i16>,
    pub movement_total: Option<i16>,
    pub music_tempo_bpm: Option<TempoBpmValue>,
    pub music_key_code: Option<i16>,
    pub music_beats_per_measure: Option<i16>,
    pub music_beat_unit: Option<i16>,
    pub music_flags: i16,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
}

impl<'a> UpdatableRecord<'a> {
    #[allow(clippy::too_many_lines)] // TODO
    pub fn bind(
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
            actors: _,
            titles: _,
            indexes,
            metrics,
            color,
            cues: _,
            tags: _,
        } = track;
        let (recorded_at_yyyymmdd, recorded_at) = recorded_at
            .map(|recorded_at| match recorded_at {
                DateOrDateTime::Date(date) => (Some(date), None),
                DateOrDateTime::DateTime(dt) => (Some(dt.into()), Some(dt)),
            })
            .unwrap_or((None, None));
        let (released_at_yyyymmdd, released_at) = released_at
            .map(|released_at| match released_at {
                DateOrDateTime::Date(date) => (Some(date), None),
                DateOrDateTime::DateTime(dt) => (Some(dt.into()), Some(dt)),
            })
            .unwrap_or((None, None));
        let (released_orig_at_yyyymmdd, released_orig_at) = released_orig_at
            .map(|released_orig_at| match released_orig_at {
                DateOrDateTime::Date(date) => (Some(date), None),
                DateOrDateTime::DateTime(dt) => (Some(dt.into()), Some(dt)),
            })
            .unwrap_or((None, None));
        let Album {
            actors: _,
            titles: _,
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
        Self {
            row_updated_ms: updated_at.timestamp_millis(),
            entity_rev,
            media_source_id: media_source_id.into(),
            last_synchronized_rev: last_synchronized_rev.map(encode_entity_revision),
            recorded_at: recorded_at.as_ref().map(ToString::to_string),
            recorded_ms: recorded_at.map(OffsetDateTimeMs::timestamp_millis),
            recorded_at_yyyymmdd: recorded_at_yyyymmdd.map(YyyyMmDdDate::value),
            released_at: released_at.as_ref().map(ToString::to_string),
            released_ms: released_at.map(OffsetDateTimeMs::timestamp_millis),
            released_at_yyyymmdd: released_at_yyyymmdd.map(YyyyMmDdDate::value),
            released_orig_at: released_orig_at.as_ref().map(ToString::to_string),
            released_orig_ms: released_orig_at.map(OffsetDateTimeMs::timestamp_millis),
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
        }
    }
}
