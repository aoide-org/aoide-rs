// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
pub(crate) mod view_album;
pub(crate) mod view_track_search;

mod join {
    use crate::db::{
        collection::schema::*, media_source::schema::*, media_tracker::schema::*,
        playlist::schema::*, playlist_entry::schema::*, track::schema::*, track_actor::schema::*,
        view_track_search::schema::*,
    };

    diesel::allow_tables_to_appear_in_same_query!(
        collection,
        collection_vfs,
        media_source,
        track,
        track_actor,
        playlist,
        playlist_entry,
        media_tracker_directory,
        media_tracker_source,
        view_track_search,
    );
}
