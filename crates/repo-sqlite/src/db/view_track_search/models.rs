// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    audio::{BitrateBpsValue, DurationMsValue, LoudnessLufsValue, SampleRateHzValue},
    music::{
        beat::{BeatUnit, BeatsPerMeasure, TimeSignature},
        key::KeySignature,
        tempo::{TempoBpm, TempoBpmValue},
    },
    prelude::*,
    track::{album::Album, index::*, metric::*},
    util::{clock::*, color::*},
    Track, TrackBody, TrackEntity, TrackHeader,
};
use aoide_repo::{media::source::RecordId as MediaSourceId, track::RecordHeader};

use super::schema::*;
use crate::{
    db::track::{decode_advisory_rating, decode_album_kind, decode_music_key_code, EntityPreload},
    prelude::*,
};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = view_track_search)]
pub struct QueryableRecord {
    pub id: RowId,
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
    pub publisher: Option<String>,
    pub copyright: Option<String>,
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
    pub collection_id: RowId,
    pub collected_ms: TimestampMillis,
    pub content_link_path: String,
    pub content_type: String,
    pub audio_duration_ms: Option<DurationMsValue>,
    pub audio_channel_count: Option<i16>,
    pub audio_channel_mask: Option<i32>,
    pub audio_samplerate_hz: Option<SampleRateHzValue>,
    pub audio_bitrate_bps: Option<BitrateBpsValue>,
    pub audio_loudness_lufs: Option<LoudnessLufsValue>,
}

impl From<QueryableRecord> for (MediaSourceId, RecordHeader, TrackHeader) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            id,
            row_created_ms,
            row_updated_ms,
            entity_uid,
            entity_rev,
            media_source_id,
            ..
        } = from;
        let record_header = RecordHeader {
            id: id.into(),
            created_at: DateTime::new_timestamp_millis(row_created_ms),
            updated_at: DateTime::new_timestamp_millis(row_updated_ms),
        };
        let entity_header = decode_entity_header(&entity_uid, entity_rev);
        (
            media_source_id.into(),
            record_header,
            TrackHeader::from_untyped(entity_header),
        )
    }
}

#[allow(clippy::too_many_lines)] // TODO
pub(crate) fn load_repo_entity(
    preload: EntityPreload,
    queryable: QueryableRecord,
) -> RepoResult<(RecordHeader, TrackEntity)> {
    let EntityPreload {
        media_source,
        track_titles,
        track_actors,
        album_titles,
        album_actors,
        tags,
        cues,
    } = preload;
    let QueryableRecord {
        id,
        row_created_ms,
        row_updated_ms,
        entity_uid,
        entity_rev,
        media_source_id: _,
        last_synchronized_rev,
        recorded_at,
        recorded_ms,
        recorded_at_yyyymmdd,
        released_at,
        released_ms,
        released_at_yyyymmdd,
        released_orig_at,
        released_orig_ms,
        released_orig_at_yyyymmdd,
        publisher,
        copyright,
        advisory_rating,
        album_kind,
        track_number,
        track_total,
        disc_number,
        disc_total,
        movement_number,
        movement_total,
        music_tempo_bpm,
        music_key_code,
        music_beats_per_measure,
        music_beat_unit,
        music_flags,
        color_rgb,
        color_idx,
        ..
    } = queryable;
    let header = RecordHeader {
        id: id.into(),
        created_at: DateTime::new_timestamp_millis(row_created_ms),
        updated_at: DateTime::new_timestamp_millis(row_updated_ms),
    };
    let entity_hdr = decode_entity_header(&entity_uid, entity_rev);
    let last_synchronized_rev = last_synchronized_rev.map(decode_entity_revision);
    let recorded_at = if let Some(recorded_at) = recorded_at {
        let recorded_at = parse_datetime_opt(Some(recorded_at.as_str()), recorded_ms);
        debug_assert_eq!(
            recorded_at.map(Into::into),
            recorded_at_yyyymmdd.map(DateYYYYMMDD::new),
        );
        recorded_at.map(Into::into)
    } else {
        recorded_at_yyyymmdd.map(DateYYYYMMDD::new).map(Into::into)
    };
    let released_at = if let Some(released_at) = released_at {
        let released_at = parse_datetime_opt(Some(released_at.as_str()), released_ms);
        debug_assert_eq!(
            released_at.map(Into::into),
            released_at_yyyymmdd.map(DateYYYYMMDD::new),
        );
        released_at.map(Into::into)
    } else {
        released_at_yyyymmdd.map(DateYYYYMMDD::new).map(Into::into)
    };
    let released_orig_at = if let Some(released_orig_at) = released_orig_at {
        let released_orig_at =
            parse_datetime_opt(Some(released_orig_at.as_str()), released_orig_ms);
        debug_assert_eq!(
            released_orig_at.map(Into::into),
            released_orig_at_yyyymmdd.map(DateYYYYMMDD::new),
        );
        released_orig_at.map(Into::into)
    } else {
        released_orig_at_yyyymmdd
            .map(DateYYYYMMDD::new)
            .map(Into::into)
    };
    let advisory_rating = advisory_rating.map(decode_advisory_rating).transpose()?;
    let album_kind = album_kind.map(decode_album_kind).transpose()?;
    let album = Canonical::tie(Album {
        kind: album_kind,
        actors: album_actors,
        titles: album_titles,
    });
    let track_index = Index {
        number: track_number.map(|number| number as u16),
        total: track_total.map(|total| total as u16),
    };
    let disc_index = Index {
        number: disc_number.map(|number| number as u16),
        total: disc_total.map(|total| total as u16),
    };
    let movement_index = Index {
        number: movement_number.map(|number| number as u16),
        total: movement_total.map(|total| total as u16),
    };
    let indexes = Indexes {
        track: track_index,
        disc: disc_index,
        movement: movement_index,
    };
    let time_signature = {
        if let Some(beats_per_measure) = music_beats_per_measure {
            Some(TimeSignature {
                beats_per_measure: beats_per_measure as BeatsPerMeasure,
                beat_unit: music_beat_unit.map(|note_value| note_value as BeatUnit),
            })
        } else {
            debug_assert!(music_beat_unit.is_none());
            None
        }
    };
    let music_key_code = music_key_code.map(decode_music_key_code).transpose()?;
    let metrics = Metrics {
        tempo_bpm: music_tempo_bpm.map(TempoBpm::new),
        key_signature: music_key_code.map(KeySignature::new),
        time_signature,
        flags: MetricsFlags::from_bits_truncate(music_flags as u8),
    };
    let color = if let Some(color_rgb) = color_rgb {
        debug_assert!(color_idx.is_none());
        let rgb_color = RgbColor(color_rgb as RgbColorCode);
        debug_assert!(rgb_color.is_valid());
        Some(Color::Rgb(rgb_color))
    } else {
        color_idx.map(|idx| Color::Index(idx as ColorIndex))
    };
    let track = Track {
        media_source,
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
        tags,
        color,
        metrics,
        cues,
    };
    let entity_body = TrackBody {
        track,
        updated_at: header.updated_at,
        last_synchronized_rev,
        content_url: None,
    };
    let entity = TrackEntity::new(TrackHeader::from_untyped(entity_hdr), entity_body);
    Ok((header, entity))
}
