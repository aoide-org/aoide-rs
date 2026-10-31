-- SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- The unique index for playlist UIDs was accidentally created on the
-- collection table, duplicating udx_collection_entity_uid. Playlist UIDs
-- have not been enforced to be unique and lookups by UID scanned the table.
DROP INDEX "udx_playlist_entity_uid";
CREATE UNIQUE INDEX "udx_playlist_entity_uid" ON "playlist" (
    "entity_uid"
);

-- Partial indexes cannot be used for loading or deleting all cues of a track,
-- neither by queries nor by cascading deletes of tracks.
CREATE INDEX "idx_track_cue_track_id" ON "track_cue" (
    "track_id"
);

-- Partial indexes cannot be used for loading or deleting all playlists of a
-- collection, neither by queries nor by cascading deletes of collections.
CREATE INDEX "idx_playlist_collection_id" ON "playlist" (
    "collection_id"
);

-- Unused: Names are never filtered or sorted without scope. LIKE cannot use
-- indexes, because it is replaced by a custom function.
DROP INDEX "idx_track_title_name";
DROP INDEX "idx_track_actor_name_role";

-- Redundant: Indexes can be traversed in both directions.
DROP INDEX "idx_track_recorded_at_yyyymmdd_desc";
DROP INDEX "idx_track_released_at_yyyymmdd_desc";
DROP INDEX "idx_track_music_tempo_bpm_desc";
