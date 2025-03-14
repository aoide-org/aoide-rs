-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later
CREATE TABLE IF NOT EXISTS "track" (
    "row_id"                   INTEGER PRIMARY KEY,
    "row_created_ms"           INTEGER NOT NULL,
    "row_updated_ms"           INTEGER NOT NULL,
    "entity_uid"               TEXT NOT NULL,
    "entity_rev"               INTEGER NOT NULL,
    "media_source_id"          INTEGER NOT NULL,
    "last_synchronized_rev"    INTEGER,
    "recorded_at"              TEXT,
    "recorded_ms"              INTEGER,
    "recorded_at_yyyymmdd"     INTEGER,
    "released_at"              TEXT,
    "released_ms"              INTEGER,
    "released_at_yyyymmdd"     INTEGER,
    "released_orig_at"         TEXT,
    "released_orig_ms"         INTEGER,
    "released_orig_at_yyyymmdd" INTEGER,
    "publisher"                TEXT,
    "copyright"                TEXT,
    "advisory_rating"          INTEGER,
    "album_kind"               INTEGER,
    "track_number"             INTEGER,
    "track_total"              INTEGER,
    "disc_number"              INTEGER,
    "disc_total"               INTEGER,
    "movement_number"          INTEGER,
    "movement_total"           INTEGER,
    "music_tempo_bpm"          REAL,
    "music_key_code"           INTEGER,
    "music_beats_per_measure"  INTEGER,
    "music_beat_unit"          INTEGER,
    "music_flags"              INTEGER NOT NULL,
    "color_rgb"                INTEGER,
    "color_idx"                INTEGER,
    FOREIGN KEY("media_source_id") REFERENCES "media_source"("row_id") ON DELETE CASCADE,
    UNIQUE ("entity_uid")
) STRICT;

DROP INDEX IF EXISTS "idx_track_row_created_ms_desc";
CREATE INDEX "idx_track_row_created_ms_desc" ON "track" (
    "row_created_ms" DESC
);

DROP INDEX IF EXISTS "idx_track_row_updated_ms_desc";
CREATE INDEX "idx_track_row_updated_ms_desc" ON "track" (
    "row_updated_ms" DESC
);

DROP INDEX IF EXISTS "idx_track_media_source_id";
CREATE INDEX "idx_track_media_source_id" ON "track" (
    "media_source_id"
);

DROP INDEX IF EXISTS "idx_track_recorded_at_yyyymmdd";
CREATE INDEX "idx_track_recorded_at_yyyymmdd" ON "track" (
    "recorded_at_yyyymmdd"
) WHERE "recorded_at_yyyymmdd" IS NOT NULL;

DROP INDEX IF EXISTS "idx_track_recorded_at_yyyymmdd_desc";
CREATE INDEX "idx_track_recorded_at_yyyymmdd_desc" ON "track" (
    "recorded_at_yyyymmdd" DESC
) WHERE "recorded_at_yyyymmdd" IS NOT NULL;

DROP INDEX IF EXISTS "idx_track_released_at_yyyymmdd";
CREATE INDEX "idx_track_released_at_yyyymmdd" ON "track" (
    "released_at_yyyymmdd"
) WHERE "released_at_yyyymmdd" IS NOT NULL;

DROP INDEX IF EXISTS "idx_track_released_at_yyyymmdd_desc";
CREATE INDEX "idx_track_released_at_yyyymmdd_desc" ON "track" (
    "released_at_yyyymmdd" DESC
) WHERE "released_at_yyyymmdd" IS NOT NULL;

DROP INDEX IF EXISTS "idx_track_music_tempo_bpm";
CREATE INDEX "idx_track_music_tempo_bpm" ON "track" (
    "music_tempo_bpm"
) WHERE "music_tempo_bpm" IS NOT NULL;

DROP INDEX IF EXISTS "idx_track_music_tempo_bpm_desc";
CREATE INDEX "idx_track_music_tempo_bpm_desc" ON "track" (
    "music_tempo_bpm" DESC
) WHERE "music_tempo_bpm" IS NOT NULL;

DROP INDEX IF EXISTS "idx_track_music_key_code";
CREATE INDEX "idx_track_music_key_code" ON "track" (
    "music_key_code"
) WHERE "music_key_code" IS NOT NULL;
