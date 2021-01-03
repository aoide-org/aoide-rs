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

pub mod collection;
pub mod media_source;
pub mod playlist;
pub mod playlist_entry;
pub mod track;
pub mod track_actor;
pub mod track_cue;
pub mod track_tag;
pub mod track_title;

mod join {
    use crate::db::{
        collection::schema::*, media_source::schema::*, playlist::schema::*,
        playlist_entry::schema::*, track::schema::*,
    };

    allow_tables_to_appear_in_same_query!(
        collection,
        media_source,
        track,
        playlist,
        playlist_entry
    );
}
