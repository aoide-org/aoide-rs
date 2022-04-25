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

use num_traits::FromPrimitive as _;

use aoide_core::{
    entity::{EntityHeader, EntityRevision},
    music::{
        beat::{BeatUnit, BeatsPerMeasure, TimeSignature},
        key::{KeyCode, KeyCodeValue, KeySignature},
        tempo::{Bpm, TempoBpm},
    },
    track::{actor::*, album::*, index::*, metric::*, title::*, *},
    util::{clock::*, color::*},
};

use aoide_repo::media::source::RecordId as MediaSourceId;

use crate::prelude::*;

use super::{schema::*, *};

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "track"]
#[allow(dead_code)] // aux_ members are required for Diesel but never read
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
    pub last_played_at: Option<String>,
    pub last_played_ms: Option<TimestampMillis>,
    pub times_played: Option<i64>,
    // TODO: Remove these unused members if no longer required by Diesel
    aux_track_title: Option<String>,
    aux_track_artist: Option<String>,
    aux_track_composer: Option<String>,
    aux_album_title: Option<String>,
    aux_album_artist: Option<String>,
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
        (media_source_id.into(), record_header, entity_header)
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
        last_played_at,
        last_played_ms,
        times_played,
        aux_track_title: _,
        aux_track_artist: _,
        aux_track_composer: _,
        aux_album_title: _,
        aux_album_artist: _,
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
        .ok_or_else(|| anyhow::anyhow!("Invalid album kind value: {}", album_kind))?;
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
        tempo_bpm: music_tempo_bpm.map(TempoBpm::from_raw),
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
    let play_counter = PlayCounter {
        last_played_at: parse_datetime_opt(last_played_at.as_deref(), last_played_ms),
        times_played: times_played.map(|val| val as PlayCount),
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
        play_counter,
    };
    let entity_body = EntityBody {
        track,
        updated_at: header.updated_at,
        last_synchronized_rev,
    };
    let entity = Entity::new(entity_hdr, entity_body);
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
    pub last_played_at: Option<String>,
    pub last_played_ms: Option<TimestampMillis>,
    pub times_played: Option<i64>,
    pub aux_track_title: Option<&'a str>,
    pub aux_track_artist: Option<&'a str>,
    pub aux_track_composer: Option<&'a str>,
    pub aux_album_title: Option<&'a str>,
    pub aux_album_artist: Option<&'a str>,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(media_source_id: MediaSourceId, entity: &'a Entity) -> Self {
        let EntityHeader { uid, rev } = &entity.hdr;
        let EntityBody {
            track,
            updated_at,
            last_synchronized_rev,
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
            play_counter:
                PlayCounter {
                    last_played_at,
                    times_played,
                },
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
            music_tempo_bpm: tempo_bpm.map(TempoBpm::to_raw),
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
            last_played_at: last_played_at.as_ref().map(ToString::to_string),
            last_played_ms: last_played_at.map(DateTime::timestamp_millis),
            times_played: times_played.map(|count| count as i64),
            aux_track_title: track.track_title(),
            aux_track_artist: track.track_artist(),
            aux_track_composer: track.track_composer(),
            aux_album_title: track.album_title(),
            aux_album_artist: track.album_artist(),
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
    pub last_played_at: Option<String>,
    pub last_played_ms: Option<TimestampMillis>,
    pub times_played: Option<i64>,
    pub aux_track_title: Option<&'a str>,
    pub aux_track_artist: Option<&'a str>,
    pub aux_track_composer: Option<&'a str>,
    pub aux_album_title: Option<&'a str>,
    pub aux_album_artist: Option<&'a str>,
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
        } = entity_body;
        let Track {
            media_source: _,
            recorded_at,
            released_at,
            released_orig_at,
            publisher,
            copyright,
            album,
            actors: track_actors,
            titles: track_titles,
            indexes,
            metrics,
            color,
            play_counter:
                PlayCounter {
                    last_played_at,
                    times_played,
                },
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
            music_tempo_bpm: tempo_bpm.map(TempoBpm::to_raw),
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
            last_played_at: last_played_at.as_ref().map(ToString::to_string),
            last_played_ms: last_played_at.map(DateTime::timestamp_millis),
            times_played: times_played.map(|count| count as i64),
            aux_track_title: Titles::main_title(track_titles.as_ref())
                .map(|title| title.name.as_str()),
            aux_track_artist: Actors::main_actor(track_actors.iter(), ActorRole::Artist)
                .map(|actor| actor.name.as_str()),
            aux_track_composer: Actors::main_actor(track_actors.iter(), ActorRole::Composer)
                .map(|actor| actor.name.as_str()),
            aux_album_title: Titles::main_title(album_titles.as_ref())
                .map(|title| title.name.as_str()),
            aux_album_artist: Actors::main_actor(album_actors.iter(), ActorRole::Artist)
                .map(|actor| actor.name.as_str()),
        }
    }
}
