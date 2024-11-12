// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::prelude::*;

use aoide_core::{
    playlist::{Entry, Item, SeparatorItem, TrackItem},
    util::clock::{OffsetDateTimeMs, TimestampMillis},
};
use aoide_repo::{PlaylistId, TrackId};

use crate::{
    util::{clock::parse_datetime, entity::decode_entity_uid_typed},
    RowId,
};

use super::schema::*;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable)]
pub struct QueryableRecord {
    pub playlist_id: RowId,
    pub ordering: i64,
    pub track_id: Option<RowId>,
    pub track_uid: Option<String>,
    pub added_at: String,
    pub added_ms: TimestampMillis,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub item_data: Option<String>,
}

impl From<QueryableRecord> for (PlaylistId, i64, Option<TrackId>, Entry) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            playlist_id,
            ordering,
            track_id,
            track_uid,
            added_at,
            added_ms,
            title,
            notes,
            item_data,
        } = from;
        let item = if let Some(track_uid) = &track_uid {
            debug_assert!(item_data.is_none());
            Item::Track(TrackItem {
                uid: decode_entity_uid_typed(track_uid),
            })
        } else {
            Item::Separator(SeparatorItem { kind: item_data })
        };
        let entry = Entry {
            added_at: parse_datetime(&added_at, added_ms),
            title,
            notes,
            item,
        };
        (
            playlist_id.into(),
            ordering,
            track_id.map(Into::into),
            entry,
        )
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = playlist_entry)]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub playlist_id: RowId,
    pub track_id: Option<RowId>,
    pub ordering: i64,
    pub added_at: String,
    pub added_ms: TimestampMillis,
    pub title: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub item_data: Option<&'a str>,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(
        playlist_id: PlaylistId,
        track_id: Option<TrackId>,
        ordering: i64,
        created_at: &'a OffsetDateTimeMs,
        created_entry: &'a Entry,
    ) -> Self {
        let row_created_updated_ms = created_at.timestamp_millis();
        let Entry {
            added_at,
            title,
            notes,
            item,
        } = created_entry;
        let item_data = match item {
            Item::Separator(SeparatorItem { kind }) => {
                debug_assert!(track_id.is_none());
                kind.as_deref()
            }
            Item::Track(TrackItem { uid: _ }) => {
                debug_assert!(track_id.is_some());
                None
            }
        };
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            playlist_id: playlist_id.into(),
            track_id: track_id.map(Into::into),
            added_at: added_at.to_string(),
            added_ms: added_at.timestamp_millis(),
            ordering,
            title: title.as_deref(),
            notes: notes.as_deref(),
            item_data,
        }
    }
}
