-- aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
--
-- This program is free software: you can redistribute it and/or modify
-- it under the terms of the GNU Affero General Public License as
-- published by the Free Software Foundation, either version 3 of the
-- License, or (at your option) any later version.
--
-- This program is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY; without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU Affero General Public License for more details.
--
-- You should have received a copy of the GNU Affero General Public License
-- along with this program.  If not, see <https://www.gnu.org/licenses/>.

CREATE TABLE IF NOT EXISTS track (
    -- row header (immutable)
    row_id                   INTEGER PRIMARY KEY,
    row_created_ms           INTEGER NOT NULL,
    -- row header (mutable)
    row_updated_ms           INTEGER NOT NULL,
    -- entity header (immutable)
    entity_uid               BINARY(24) NOT NULL,
    -- entity header (mutable)
    entity_rev               INTEGER NOT NULL, -- RevisionNumber
    -- relations (immutable)
    media_source_id          INTEGER NOT NULL,
    -- properties: synchronization with external content (media source)
    last_synchronized_rev    INTEGER,
    -- properties: album/release
    recorded_at              TEXT,
    recorded_ms              INTEGER,
    recorded_at_yyyymmdd     INTEGER, -- naive, gregorian release date as YYYYMMDD (parsed from recorded_at)
    released_at              TEXT,
    released_ms              INTEGER,
    released_at_yyyymmdd     INTEGER, -- naive, gregorian release date as YYYYMMDD (parsed from released_at)
    released_orig_at          TEXT,
    released_orig_ms          INTEGER,
    released_orig_at_yyyymmdd INTEGER, -- naive, gregorian release date as YYYYMMDD (parsed from released_at)
    released_by              TEXT,    -- publisher or record label
    copyright                TEXT,
    album_kind               TINYINT NOT NULL,
    -- properties: indexing
    track_number             INTEGER, -- > 0
    track_total              INTEGER, -- > 0
    disc_number              INTEGER, -- > 0
    disc_total               INTEGER, -- > 0
    movement_number          INTEGER, -- > 0
    movement_total           INTEGER, -- > 0
    -- properties: musical signature
    music_tempo_bpm          REAL,    -- beats per minute
    music_key_code           TINYINT NOT NULL, -- musical key signature code {(0), 1, ..., 24}
    music_beats_per_measure  INTEGER, -- musical time signature, top value
    music_beat_unit          INTEGER, -- musical time signature, bottom value
    music_flags              INTEGER NOT NULL, -- bitmask of flags, e.g. for locking individual properties to prevent unintended modifications
    -- properties: custom
    color_rgb                INTEGER, -- 0xRRGGBB (hex)
    color_idx                INTEGER, -- palette index
    -- properties: play counter
    last_played_at           TEXT,
    last_played_ms           INTEGER,
    times_played             INTEGER,
    -- auxiliary properties for filtering and sorting
    aux_track_title          TEXT,    -- main title (derived from entries in track_title)
    aux_track_artist         TEXT,    -- summary/default artist (derived from entries in track_actor)
    aux_track_composer       TEXT,    -- summary composer (derived from entries in track_actor)
    aux_album_title          TEXT,    -- main album title (derived from entries in track_title)
    aux_album_artist         TEXT,    -- summary/default album artist (derived from track_actor)
    --
    FOREIGN KEY(media_source_id) REFERENCES media_source(row_id) ON DELETE CASCADE,
    UNIQUE (entity_uid) -- only the last revision is stored
);

CREATE INDEX IF NOT EXISTS idx_track_row_created_ms_desc ON track (
    row_created_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_track_row_updated_ms_desc ON track (
    row_updated_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_track_media_source_id ON track (
    media_source_id
);

CREATE INDEX IF NOT EXISTS idx_track_recorded_at_yyyymmdd ON track (
    recorded_at_yyyymmdd
) WHERE recorded_at_yyyymmdd IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_recorded_at_yyyymmdd_desc ON track (
    recorded_at_yyyymmdd DESC
) WHERE recorded_at_yyyymmdd IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_released_at_yyyymmdd ON track (
    released_at_yyyymmdd
) WHERE released_at_yyyymmdd IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_released_at_yyyymmdd_desc ON track (
    released_at_yyyymmdd DESC
) WHERE released_at_yyyymmdd IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_music_tempo_bpm ON track (
    music_tempo_bpm
) WHERE music_tempo_bpm IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_music_tempo_bpm_desc ON track (
    music_tempo_bpm DESC
) WHERE music_tempo_bpm IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_music_key_code ON track (
    music_key_code
) WHERE music_key_code IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_aux_track_title ON track (
    aux_track_title
);

CREATE INDEX IF NOT EXISTS idx_track_aux_track_artist ON track (
    aux_track_artist
);

CREATE INDEX IF NOT EXISTS idx_track_aux_track_composer ON track (
    aux_track_composer
) WHERE aux_track_composer IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_aux_album_title ON track (
    aux_album_title
) WHERE aux_album_title IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_aux_album_artist ON track (
    aux_album_artist
) WHERE aux_album_artist IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_track_last_played_ms_desc ON track (
    last_played_ms DESC
) WHERE last_played_ms IS NOT NULL;
