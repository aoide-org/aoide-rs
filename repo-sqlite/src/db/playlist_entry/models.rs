// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::schema::*;

use crate::prelude::*;

use aoide_core::{
    entity::EntityUid,
    playlist::{track::Item as TrackItem, Entry, Item},
    util::clock::{DateTime, TimestampMillis},
};

use aoide_repo::{playlist::RecordId as PlaylistId, track::RecordId as TrackId};

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable)]
pub struct QueryableRecord {
    pub playlist_id: RowId,
    pub ordering: i64,
    pub track_id: Option<RowId>,
    pub track_uid: Option<Vec<u8>>,
    pub added_at: String,
    pub added_ms: TimestampMillis,
    pub title: Option<String>,
    pub notes: Option<String>,
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
        } = from;
        let item = if let Some(track_uid) = track_uid {
            Item::Track(TrackItem {
                uid: EntityUid::from_slice(&track_uid),
            })
        } else {
            Item::Separator
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
#[table_name = "playlist_entry"]
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
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(
        playlist_id: PlaylistId,
        track_id: Option<TrackId>,
        ordering: i64,
        created_at: DateTime,
        created_entry: &'a Entry,
    ) -> Self {
        let row_created_updated_ms = created_at.timestamp_millis();
        let Entry {
            added_at,
            title,
            notes,
            item: _,
        } = created_entry;
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
        }
    }
}

#[derive(Debug, AsChangeset)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "playlist_entry"]
pub struct UpdatableRecord<'a> {
    pub row_updated_ms: TimestampMillis,
    pub added_at: &'a str,
    pub added_ms: TimestampMillis,
    pub title: Option<&'a str>,
    pub notes: Option<&'a str>,
}
