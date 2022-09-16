-- SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS track (
    -- row header (immutable)
    row_id                   INTEGER PRIMARY KEY,
    row_created_ms           INTEGER NOT NULL,
    -- row header (mutable)
    row_updated_ms           INTEGER NOT NULL,
    -- entity header (immutable)
    entity_uid               TEXT NOT NULL, -- ULID
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
    publisher                TEXT,    -- publisher or record label
    copyright                TEXT,
    album_kind               INTEGER,
    -- properties: indexing
    track_number             INTEGER, -- > 0
    track_total              INTEGER, -- > 0
    disc_number              INTEGER, -- > 0
    disc_total               INTEGER, -- > 0
    movement_number          INTEGER, -- > 0
    movement_total           INTEGER, -- > 0
    -- properties: musical signature
    music_tempo_bpm          REAL,    -- beats per minute
    music_key_code           INTEGER, -- musical key signature code {(0), 1, ..., 24}
    music_beats_per_measure  INTEGER, -- musical time signature, top value
    music_beat_unit          INTEGER, -- musical time signature, bottom value
    music_flags              INTEGER NOT NULL, -- bitmask of flags, e.g. for locking individual properties to prevent unintended modifications
    -- properties: custom
    color_rgb                INTEGER, -- 0xRRGGBB (hex)
    color_idx                INTEGER, -- palette index
    --
    FOREIGN KEY(media_source_id) REFERENCES media_source(row_id) ON DELETE CASCADE,
    UNIQUE (entity_uid) -- only the last revision is stored
) STRICT;

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
