// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

use aoide_core::util::{clock::*, color::*};

use aoide_repo::{entity::*, RepoId};

use chrono::{naive::NaiveDateTime, DateTime, Utc};

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable)]
#[table_name = "tbl_playlist"]
pub struct InsertableEntity<'a> {
    pub uid: &'a [u8],
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub data_fmt: i16,
    pub data_vmaj: i16,
    pub data_vmin: i16,
    pub data_blob: &'a [u8],
}

impl<'a> InsertableEntity<'a> {
    pub fn bind(
        hdr: &'a EntityHeader,
        data_fmt: EntityDataFormat,
        data_ver: EntityDataVersion,
        data_blob: &'a [u8],
    ) -> Self {
        Self {
            uid: hdr.uid.as_ref(),
            rev_no: hdr.rev.no as i64,
            rev_ts: (hdr.rev.ts.0).0,
            data_fmt: data_fmt as i16,
            data_vmaj: data_ver.major as i16,
            data_vmin: data_ver.minor as i16,
            data_blob,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "tbl_playlist"]
pub struct UpdatableEntity<'a> {
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub data_fmt: i16,
    pub data_vmaj: i16,
    pub data_vmin: i16,
    pub data_blob: &'a [u8],
}

impl<'a> UpdatableEntity<'a> {
    pub fn bind(
        next_rev: &'a EntityRevision,
        data_fmt: EntityDataFormat,
        data_ver: EntityDataVersion,
        data_blob: &'a [u8],
    ) -> Self {
        Self {
            rev_no: next_rev.no as i64,
            rev_ts: (next_rev.ts.0).0,
            data_fmt: data_fmt as i16,
            data_vmaj: data_ver.major as i16,
            data_vmin: data_ver.minor as i16,
            data_blob,
        }
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "tbl_playlist"]
pub struct QueryableEntityData {
    pub id: RepoId,
    pub uid: Vec<u8>,
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub data_fmt: i16,
    pub data_vmaj: i16,
    pub data_vmin: i16,
    pub data_blob: Vec<u8>,
}

impl From<QueryableEntityData> for EntityData {
    fn from(from: QueryableEntityData) -> Self {
        let rev = EntityRevision {
            no: from.rev_no as u64,
            ts: TickInstant(Ticks(from.rev_ts)),
        };
        let hdr = EntityHeader {
            uid: EntityUid::from_slice(&from.uid),
            rev,
        };
        let fmt = if from.data_fmt == EntityDataFormat::JSON as i16 {
            EntityDataFormat::JSON
        } else {
            // TODO: How to handle unexpected/invalid values?
            unreachable!()
        };
        let ver = EntityDataVersion {
            major: from.data_vmaj as EntityDataVersionNumber,
            minor: from.data_vmin as EntityDataVersionNumber,
        };
        (hdr, (fmt, ver, from.data_blob))
    }
}

#[derive(Debug, Queryable)]
pub struct QueryableBrief {
    pub id: RepoId,
    pub uid: Vec<u8>,
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub name: String,
    pub desc: Option<String>,
    pub playlist_type: Option<String>,
    pub color_code: Option<i32>,
    pub tracks_count: i64,
    pub entries_count: i64,
    pub entries_since_min: Option<NaiveDateTime>,
    pub entries_since_max: Option<NaiveDateTime>,
}

impl From<QueryableBrief> for (RepoId, EntityHeader, PlaylistBrief) {
    fn from(from: QueryableBrief) -> Self {
        let QueryableBrief {
            id,
            uid,
            rev_no,
            rev_ts,
            name,
            desc,
            playlist_type,
            color_code,
            tracks_count,
            entries_count,
            entries_since_min,
            entries_since_max,
        } = from;
        let hdr = EntityHeader {
            uid: EntityUid::from_slice(&uid),
            rev: EntityRevision {
                no: rev_no as u64,
                ts: TickInstant(Ticks(rev_ts)),
            },
        };
        let entries_since_min = entries_since_min.map(|min| DateTime::from_utc(min, Utc).into());
        let entries_since_max = entries_since_max.map(|max| DateTime::from_utc(max, Utc).into());
        debug_assert_eq!(entries_since_min.is_some(), entries_since_max.is_some());
        let entries_since_minmax = match (entries_since_min, entries_since_max) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        };
        let entries = PlaylistBriefEntries {
            tracks_count: tracks_count as usize,
            entries_count: entries_count as usize,
            entries_since_minmax,
        };
        let brief = PlaylistBrief {
            name,
            description: desc,
            r#type: playlist_type,
            color: color_code.map(|c| ColorRgb(c as ColorCode)),
            entries,
        };
        (id, hdr, brief)
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_playlist_brief"]
pub struct InsertableBrief<'a> {
    pub playlist_id: RepoId,
    pub name: &'a str,
    pub desc: Option<&'a str>,
    pub playlist_type: Option<&'a str>,
    pub color_code: Option<i32>,
    pub tracks_count: i64,
    pub entries_count: i64,
    pub entries_since_min: Option<NaiveDateTime>,
    pub entries_since_max: Option<NaiveDateTime>,
}

impl<'a> InsertableBrief<'a> {
    pub fn bind(playlist_id: RepoId, brief_ref: &'a PlaylistBriefRef<'a>) -> Self {
        let &PlaylistBriefRef {
            name,
            description,
            r#type,
            color,
            entries,
        } = brief_ref;
        let PlaylistBriefEntries {
            tracks_count,
            entries_count,
            entries_since_minmax,
        } = entries;
        Self {
            playlist_id,
            name,
            desc: description,
            playlist_type: r#type,
            color_code: color.map(|color| color.code() as i32),
            tracks_count: tracks_count as i64,
            entries_count: entries_count as i64,
            entries_since_min: entries_since_minmax.map(|minmax| DateTime::from(minmax.0).naive_utc()),
            entries_since_max: entries_since_minmax.map(|minmax| DateTime::from(minmax.1).naive_utc()),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_playlist_track"]
pub struct InsertableTrack<'a> {
    pub playlist_id: RepoId,
    pub track_uid: &'a [u8],
    pub track_ref_count: i64,
}

impl<'a> InsertableTrack<'a> {
    pub fn bind(playlist_id: RepoId, track_uid: &'a EntityUid, track_ref_count: usize) -> Self {
        Self {
            playlist_id,
            track_uid: track_uid.as_ref(),
            track_ref_count: track_ref_count as i64,
        }
    }
}
