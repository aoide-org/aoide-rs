// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use num_traits::FromPrimitive as _;

use aoide_core::{
    entity::{EntityHeaderTyped, EntityRevision},
    music::{
        beat::{BeatUnit, BeatsPerMeasure, TimeSignature},
        key::{KeyCode, KeyCodeValue, KeySignature},
        tempo::{Bpm, TempoBpm},
    },
    track::{
        album::{Album, Kind as AlbumKind},
        index::*,
        metric::*,
        Entity, EntityBody, EntityHeader, Track,
    },
    util::{clock::*, color::*},
};

use aoide_repo::media::source::RecordId as MediaSourceId;

use crate::prelude::*;

use super::{schema::*, *};

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "track"]
pub struct QueryableRecord {
    pub id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: Vec<u8>,
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
    pub album_kind: i16,
    pub track_number: Option<i16>,
    pub track_total: Option<i16>,
    pub disc_number: Option<i16>,
    pub disc_total: Option<i16>,
    pub movement_number: Option<i16>,
    pub movement_total: Option<i16>,
    pub music_tempo_bpm: Option<Bpm>,
    pub music_key_code: i16,
    pub music_beats_per_measure: Option<i16>,
    pub music_beat_unit: Option<i16>,
    pub music_flags: i16,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
}

impl From<QueryableRecord> for (MediaSourceId, RecordHeader, EntityHeader) {
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
        let entity_header = entity_header_from_sql(&entity_uid, entity_rev);
        (
            media_source_id.into(),
            record_header,
            EntityHeaderTyped::from_untyped(entity_header),
        )
    }
}

pub(crate) fn load_repo_entity(
    preload: EntityPreload,
    queryable: QueryableRecord,
) -> RepoResult<(RecordHeader, Entity)> {
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
    } = queryable;
    let header = RecordHeader {
        id: id.into(),
        created_at: DateTime::new_timestamp_millis(row_created_ms),
        updated_at: DateTime::new_timestamp_millis(row_updated_ms),
    };
    let entity_hdr = entity_header_from_sql(&entity_uid, entity_rev);
    let last_synchronized_rev = last_synchronized_rev.map(entity_revision_from_sql);
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
    let album_kind = AlbumKind::from_i16(album_kind)
        .ok_or_else(|| anyhow::anyhow!("Invalid album kind value: {album_kind}"))?;
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
    let metrics = Metrics {
        tempo_bpm: music_tempo_bpm.map(TempoBpm::from_inner),
        key_signature: KeySignature::new(KeyCode::from_value(music_key_code as KeyCodeValue)),
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
        album,
        actors: track_actors,
        titles: track_titles,
        indexes,
        tags,
        color,
        metrics,
        cues,
    };
    let entity_body = EntityBody {
        track,
        updated_at: header.updated_at,
        last_synchronized_rev,
        content_url: None,
    };
    let entity = Entity::new(EntityHeaderTyped::from_untyped(entity_hdr), entity_body);
    Ok((header, entity))
}

#[derive(Debug, Insertable)]
#[table_name = "track"]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: &'a [u8],
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
    pub album_kind: i16,
    pub track_number: Option<i16>,
    pub track_total: Option<i16>,
    pub disc_number: Option<i16>,
    pub disc_total: Option<i16>,
    pub movement_number: Option<i16>,
    pub movement_total: Option<i16>,
    pub music_tempo_bpm: Option<Bpm>,
    pub music_key_code: i16,
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
            entity_uid: uid.as_ref(),
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
            album_kind: *album_kind as i16,
            track_number: track_index.number.map(|idx| idx as i16),
            track_total: track_index.total.map(|idx| idx as i16),
            disc_number: disc_index.number.map(|idx| idx as i16),
            disc_total: disc_index.total.map(|idx| idx as i16),
            movement_number: movement_index.number.map(|idx| idx as i16),
            movement_total: movement_index.total.map(|idx| idx as i16),
            music_tempo_bpm: tempo_bpm.map(TempoBpm::to_inner),
            music_key_code: key_signature.code().to_value() as i16,
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
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "track"]
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
    pub album_kind: i16,
    pub track_number: Option<i16>,
    pub track_total: Option<i16>,
    pub disc_number: Option<i16>,
    pub disc_total: Option<i16>,
    pub movement_number: Option<i16>,
    pub movement_total: Option<i16>,
    pub music_tempo_bpm: Option<Bpm>,
    pub music_key_code: i16,
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
            album_kind: *album_kind as i16,
            track_number: track_index.number.map(|number| number as i16),
            track_total: track_index.total.map(|total| total as i16),
            disc_number: disc_index.number.map(|number| number as i16),
            disc_total: disc_index.total.map(|total| total as i16),
            movement_number: movement_index.number.map(|number| number as i16),
            movement_total: movement_index.total.map(|total| total as i16),
            music_tempo_bpm: tempo_bpm.map(TempoBpm::to_inner),
            music_key_code: key_signature.code().to_value() as i16,
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
