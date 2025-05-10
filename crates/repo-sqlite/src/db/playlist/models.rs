// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::prelude::*;
use semval::prelude::*;

use aoide_core::{
    EntityRevision, Playlist, PlaylistEntity, PlaylistHeader,
    playlist::Flags,
    util::{clock::*, color::*},
};
use aoide_repo::{CollectionId, playlist::RecordHeader};

use crate::{
    RowId,
    util::entity::{decode_entity_header, encode_entity_revision, encode_entity_uid},
};

use super::schema::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = playlist, primary_key(row_id))]
pub struct QueryableRecord {
    pub row_id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: String,
    pub entity_rev: i64,
    pub collection_id: Option<RowId>,
    pub title: String,
    pub kind: Option<String>,
    pub notes: Option<String>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub flags: i16,
}

impl From<QueryableRecord> for (RecordHeader, Option<CollectionId>, PlaylistEntity) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            row_id,
            row_created_ms,
            row_updated_ms,
            entity_uid,
            entity_rev,
            collection_id,
            title,
            kind,
            notes,
            color_rgb,
            color_idx,
            flags,
        } = from;
        let header = RecordHeader {
            id: row_id.into(),
            created_at: UtcDateTimeMs::from_unix_timestamp_millis(row_created_ms),
            updated_at: UtcDateTimeMs::from_unix_timestamp_millis(row_updated_ms),
        };
        let collection_id = collection_id.map(Into::into);
        let entity_hdr = decode_entity_header(&entity_uid, entity_rev);
        let entity_body = Playlist {
            title,
            kind,
            notes,
            color: if let Some(color_rgb) = color_rgb {
                debug_assert!(color_idx.is_none());
                let rgb_color = RgbColor::new(color_rgb as RgbColorCode);
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
            PlaylistEntity::new(PlaylistHeader::from_untyped(entity_hdr), entity_body),
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
    pub collection_id: Option<RowId>,
    pub title: &'a str,
    pub kind: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub flags: i16,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(
        collection_id: Option<CollectionId>,
        created_at: UtcDateTimeMs,
        entity: &'a PlaylistEntity,
    ) -> Self {
        let row_created_updated_ms = created_at.unix_timestamp_millis();
        let (hdr, body) = entity.into();
        let PlaylistHeader { uid, rev } = hdr;
        let Playlist {
            title,
            kind,
            notes,
            color,
            flags,
        } = body;
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            entity_uid: encode_entity_uid(uid),
            entity_rev: encode_entity_revision(*rev),
            collection_id: collection_id.map(Into::into),
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
            flags: i16::from(flags.bits()),
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
    pub fn bind(updated_at: UtcDateTimeMs, next_rev: EntityRevision) -> Self {
        let entity_rev = encode_entity_revision(next_rev);
        Self {
            row_updated_ms: updated_at.unix_timestamp_millis(),
            entity_rev,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = playlist, treat_none_as_null = true)]
pub struct UpdatableRecord<'a> {
    pub row_updated_ms: TimestampMillis,
    pub entity_rev: i64,
    pub title: &'a str,
    pub kind: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub flags: i16,
}

impl<'a> UpdatableRecord<'a> {
    pub fn bind(
        updated_at: UtcDateTimeMs,
        next_rev: EntityRevision,
        playlist: &'a Playlist,
    ) -> Self {
        let entity_rev = encode_entity_revision(next_rev);
        let Playlist {
            title,
            kind,
            notes,
            color,
            flags,
        } = playlist;
        Self {
            row_updated_ms: updated_at.unix_timestamp_millis(),
            entity_rev,
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
            flags: i16::from(flags.bits()),
        }
    }
}
