// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    entity::{EntityHeaderTyped, EntityRevision},
    music::tempo::{Bpm, TempoBpm},
    track::{album::Album, index::*, metric::*, Entity, EntityBody, Track},
    util::{clock::*, color::*},
};

use aoide_repo::media::source::RecordId as MediaSourceId;

use crate::prelude::*;

use super::schema::*;

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
    pub recorded_at_yyyymmdd: Option<YYYYMMDD>,
    pub released_at: Option<String>,
    pub released_ms: Option<TimestampMillis>,
    pub released_at_yyyymmdd: Option<YYYYMMDD>,
    pub released_orig_at: Option<String>,
    pub released_orig_ms: Option<TimestampMillis>,
    pub released_orig_at_yyyymmdd: Option<YYYYMMDD>,
    pub publisher: Option<&'a str>,
    pub copyright: Option<&'a str>,
    pub album_kind: Option<i16>,
    pub track_number: Option<i16>,
    pub track_total: Option<i16>,
    pub disc_number: Option<i16>,
    pub disc_total: Option<i16>,
    pub movement_number: Option<i16>,
    pub movement_total: Option<i16>,
    pub music_tempo_bpm: Option<Bpm>,
    pub music_key_code: Option<i16>,
    pub music_beats_per_measure: Option<i16>,
    pub music_beat_unit: Option<i16>,
    pub music_flags: i16,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(media_source_id: MediaSourceId, entity: &'a Entity) -> Self {
        let EntityHeaderTyped { uid, rev } = &entity.hdr;
        let EntityBody {
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
            entity_uid: entity_uid_to_sql(uid),
            entity_rev: entity_revision_to_sql(*rev),
            media_source_id: media_source_id.into(),
            last_synchronized_rev: last_synchronized_rev.map(entity_revision_to_sql),
            recorded_at: recorded_at.as_ref().map(ToString::to_string),
            recorded_ms: recorded_at.map(DateTime::timestamp_millis),
            recorded_at_yyyymmdd: recorded_at_yyyymmdd.map(Into::into),
            released_at: released_at.as_ref().map(ToString::to_string),
            released_ms: released_at.map(DateTime::timestamp_millis),
            released_at_yyyymmdd: released_at_yyyymmdd.map(Into::into),
            released_orig_at: released_orig_at.as_ref().map(ToString::to_string),
            released_orig_ms: released_orig_at.map(DateTime::timestamp_millis),
            released_orig_at_yyyymmdd: released_orig_at_yyyymmdd.map(Into::into),
            publisher: publisher.as_ref().map(String::as_str),
            copyright: copyright.as_ref().map(String::as_str),
            album_kind: album_kind.map(|kind| kind as i16),
            track_number: track_index.number.map(|idx| idx as i16),
            track_total: track_index.total.map(|idx| idx as i16),
            disc_number: disc_index.number.map(|idx| idx as i16),
            disc_total: disc_index.total.map(|idx| idx as i16),
            movement_number: movement_index.number.map(|idx| idx as i16),
            movement_total: movement_index.total.map(|idx| idx as i16),
            music_tempo_bpm: tempo_bpm.map(TempoBpm::to_inner),
            music_key_code: key_signature.map(|s| s.code().to_value() as i16),
            music_beats_per_measure: time_signature
                .map(|time_sig| time_sig.beats_per_measure as i16),
            music_beat_unit: time_signature
                .and_then(|time_sig| time_sig.beat_unit)
                .map(|beat_unit| beat_unit as i16),
            music_flags: music_flags.bits() as i16,
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
    pub recorded_at_yyyymmdd: Option<YYYYMMDD>,
    pub released_at: Option<String>,
    pub released_ms: Option<TimestampMillis>,
    pub released_at_yyyymmdd: Option<YYYYMMDD>,
    pub released_orig_at: Option<String>,
    pub released_orig_ms: Option<TimestampMillis>,
    pub released_orig_at_yyyymmdd: Option<YYYYMMDD>,
    pub publisher: Option<&'a str>,
    pub copyright: Option<&'a str>,
    pub album_kind: Option<i16>,
    pub track_number: Option<i16>,
    pub track_total: Option<i16>,
    pub disc_number: Option<i16>,
    pub disc_total: Option<i16>,
    pub movement_number: Option<i16>,
    pub movement_total: Option<i16>,
    pub music_tempo_bpm: Option<Bpm>,
    pub music_key_code: Option<i16>,
    pub music_beats_per_measure: Option<i16>,
    pub music_beat_unit: Option<i16>,
    pub music_flags: i16,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
}

impl<'a> UpdatableRecord<'a> {
    pub fn bind(
        next_rev: EntityRevision,
        media_source_id: MediaSourceId,
        entity_body: &'a EntityBody,
    ) -> Self {
        let entity_rev = entity_revision_to_sql(next_rev);
        let EntityBody {
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
            last_synchronized_rev: last_synchronized_rev.map(entity_revision_to_sql),
            recorded_at: recorded_at.as_ref().map(ToString::to_string),
            recorded_ms: recorded_at.map(DateTime::timestamp_millis),
            recorded_at_yyyymmdd: recorded_at_yyyymmdd.map(Into::into),
            released_at: released_at.as_ref().map(ToString::to_string),
            released_ms: released_at.map(DateTime::timestamp_millis),
            released_at_yyyymmdd: released_at_yyyymmdd.map(Into::into),
            released_orig_at: released_orig_at.as_ref().map(ToString::to_string),
            released_orig_ms: released_orig_at.map(DateTime::timestamp_millis),
            released_orig_at_yyyymmdd: released_orig_at_yyyymmdd.map(Into::into),
            publisher: publisher.as_ref().map(String::as_str),
            copyright: copyright.as_ref().map(String::as_str),
            album_kind: album_kind.map(|kind| kind as i16),
            track_number: track_index.number.map(|number| number as i16),
            track_total: track_index.total.map(|total| total as i16),
            disc_number: disc_index.number.map(|number| number as i16),
            disc_total: disc_index.total.map(|total| total as i16),
            movement_number: movement_index.number.map(|number| number as i16),
            movement_total: movement_index.total.map(|total| total as i16),
            music_tempo_bpm: tempo_bpm.map(TempoBpm::to_inner),
            music_key_code: key_signature.map(|s| s.code().to_value() as i16),
            music_beats_per_measure: time_signature
                .map(|time_sig| time_sig.beats_per_measure as i16),
            music_beat_unit: time_signature
                .and_then(|time_sig| time_sig.beat_unit)
                .map(|beat_unit| beat_unit as i16),
            music_flags: music_flags.bits() as i16,
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
