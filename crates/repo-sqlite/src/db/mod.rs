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

pub(crate) mod collection;
pub(crate) mod media_source;
pub(crate) mod media_tracker;
pub(crate) mod playlist;
pub(crate) mod playlist_entry;
pub(crate) mod track;
pub(crate) mod track_actor;
pub(crate) mod track_cue;
pub(crate) mod track_tag;
pub(crate) mod track_title;

mod join {
    use crate::db::{
        collection::schema::*, media_source::schema::*, media_tracker::schema::*,
        playlist::schema::*, playlist_entry::schema::*, track::schema::*,
    };

    allow_tables_to_appear_in_same_query!(
        collection,
        media_source,
        track,
        playlist,
        playlist_entry,
        media_tracker_directory,
        media_tracker_source,
    );
}
