-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS "playlist_entry" (
    -- row header (immutable)
    "row_id"                   INTEGER PRIMARY KEY,
    "row_created_ms"           INTEGER NOT NULL,
    -- row header (mutable)
    "row_updated_ms"           INTEGER NOT NULL,
    -- relations (immutable)
    "playlist_id"              INTEGER NOT NULL,
    "track_id"                 INTEGER, -- NULL for separators
    -- private properties
    "ordering"                 INTEGER NOT NULL, -- does not affect row_created_ms/row_updated_ms
    -- properties
    "added_at"                 TEXT NOT NULL,
    "added_ms"                 INTEGER NOT NULL,
    "title"                    TEXT,
    "notes"                    TEXT,
    "item_data"                TEXT,
    --
    UNIQUE("playlist_id", "ordering"),
    FOREIGN KEY("playlist_id") REFERENCES "playlist"("row_id") ON DELETE CASCADE,
    FOREIGN KEY("track_id") REFERENCES "track"("row_id") ON DELETE CASCADE
) STRICT;

DROP INDEX IF EXISTS "idx_playlist_entry_row_created_ms_desc";
CREATE INDEX "idx_playlist_entry_row_created_ms_desc" ON "playlist" (
    "row_created_ms" DESC
);

DROP INDEX IF EXISTS "idx_playlist_entry_row_updated_ms_desc";
CREATE INDEX "idx_playlist_entry_row_updated_ms_desc" ON "playlist" (
    "row_updated_ms" DESC
);

DROP INDEX IF EXISTS "idx_playlist_entry_added_ms_desc";
CREATE INDEX "idx_playlist_entry_added_ms_desc" ON "playlist_entry" (
    "added_ms" DESC
);

DROP INDEX IF EXISTS "idx_playlist_entry_playlist_id_track_id";
CREATE INDEX "idx_playlist_entry_playlist_id_track_id" ON "playlist_entry" (
    "playlist_id",
    "track_id"
);

DROP INDEX IF EXISTS "idx_playlist_entry_track_id";
CREATE INDEX "idx_playlist_entry_track_id" ON "playlist_entry" (
    "track_id"
);
