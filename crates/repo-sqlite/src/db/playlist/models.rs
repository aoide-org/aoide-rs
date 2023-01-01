// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::schema::*;

use crate::prelude::*;

use aoide_core::{
    entity::{EntityHeaderTyped, EntityRevision},
    playlist::*,
    util::{clock::*, color::*},
};

use aoide_repo::{collection::RecordId as CollectionId, playlist::RecordHeader};

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = playlist)]
pub struct QueryableRecord {
    pub id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: String,
    pub entity_rev: i64,
    pub collection_id: RowId,
    pub collected_at: String,
    pub collected_ms: TimestampMillis,
    pub title: String,
    pub kind: Option<String>,
    pub notes: Option<String>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub flags: i16,
}

impl From<QueryableRecord> for (RecordHeader, CollectionId, Entity) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            id,
            row_created_ms,
            row_updated_ms,
            entity_uid,
            entity_rev,
            collection_id,
            collected_at,
            collected_ms,
            title,
            kind,
            notes,
            color_rgb,
            color_idx,
            flags,
        } = from;
        let header = RecordHeader {
            id: id.into(),
            created_at: DateTime::new_timestamp_millis(row_created_ms),
            updated_at: DateTime::new_timestamp_millis(row_updated_ms),
        };
        let collection_id = collection_id.into();
        let entity_hdr = entity_header_from_sql(&entity_uid, entity_rev);
        let entity_body = Playlist {
            collected_at: parse_datetime(&collected_at, collected_ms),
            title,
            kind,
            notes,
            color: if let Some(color_rgb) = color_rgb {
                debug_assert!(color_idx.is_none());
                let rgb_color = RgbColor(color_rgb as RgbColorCode);
                debug_assert!(rgb_color.is_valid());
                Some(Color::Rgb(rgb_color))
            } else {
                color_idx.map(|idx| Color::Index(idx as ColorIndex))
            },
            flags: Flags::from_bits_truncate(flags as u8),
        };
        (
            header,
            collection_id,
            Entity::new(EntityHeaderTyped::from_untyped(entity_hdr), entity_body),
        )
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = playlist)]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: String,
    pub entity_rev: i64,
    pub collection_id: RowId,
    pub collected_at: String,
    pub collected_ms: TimestampMillis,
    pub title: &'a str,
    pub kind: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub flags: i16,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(collection_id: CollectionId, created_at: DateTime, entity: &'a Entity) -> Self {
        let row_created_updated_ms = created_at.timestamp_millis();
        let (hdr, body) = entity.into();
        let EntityHeaderTyped { uid, rev } = hdr;
        let Playlist {
            collected_at,
            title,
            kind,
            notes,
            color,
            flags,
        } = body;
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            entity_uid: entity_uid_to_sql(uid),
            entity_rev: entity_revision_to_sql(*rev),
            collection_id: collection_id.into(),
            collected_at: collected_at.to_string(),
            collected_ms: collected_at.timestamp_millis(),
            title,
            kind: kind.as_deref(),
            notes: notes.as_deref(),
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
            flags: flags.bits() as i16,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = playlist, treat_none_as_null = true)]
pub struct TouchableRecord {
    pub row_updated_ms: TimestampMillis,
    pub entity_rev: i64,
}

impl TouchableRecord {
    pub fn bind(updated_at: DateTime, next_rev: EntityRevision) -> Self {
        let entity_rev = entity_revision_to_sql(next_rev);
        Self {
            row_updated_ms: updated_at.timestamp_millis(),
            entity_rev,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = playlist, treat_none_as_null = true)]
pub struct UpdatableRecord<'a> {
    pub row_updated_ms: TimestampMillis,
    pub entity_rev: i64,
    pub collected_at: String,
    pub collected_ms: TimestampMillis,
    pub title: &'a str,
    pub kind: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub flags: i16,
}

impl<'a> UpdatableRecord<'a> {
    pub fn bind(updated_at: DateTime, next_rev: EntityRevision, playlist: &'a Playlist) -> Self {
        let entity_rev = entity_revision_to_sql(next_rev);
        let Playlist {
            collected_at,
            title,
            kind,
            notes,
            color,
            flags,
        } = playlist;
        Self {
            row_updated_ms: updated_at.timestamp_millis(),
            entity_rev,
            collected_at: collected_at.to_string(),
            collected_ms: collected_at.timestamp_millis(),
            title,
            kind: kind.as_deref(),
            notes: notes.as_deref(),
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
            flags: flags.bits() as i16,
        }
    }
}
