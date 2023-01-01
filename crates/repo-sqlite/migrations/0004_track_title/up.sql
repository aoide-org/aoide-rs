-- SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS track_title (
    row_id                   INTEGER PRIMARY KEY,
    -- relations (immutable)
    track_id                 INTEGER NOT NULL,
    -- properties
    scope                    INTEGER NOT NULL, -- 0: track, 1: album
    kind                     INTEGER NOT NULL,
    name                     TEXT NOT NULL,
    --
    FOREIGN KEY(track_id) REFERENCES track(row_id) ON DELETE CASCADE
) STRICT;

DROP INDEX IF EXISTS idx_track_title_track_id;
CREATE INDEX idx_track_title_track_id ON track_title (
    track_id
);

-- Covering index (= contains all columns) for loading
-- Ordering of columns matches the canonical ordering on load
DROP INDEX IF EXISTS idx_track_title_scope_kind_name;
CREATE INDEX idx_track_title_scope_kind_name ON track_title (
    scope,
    kind,
    name
);

-- Searching
DROP INDEX IF EXISTS idx_track_title_name;
CREATE INDEX idx_track_title_name ON track_title (
    name
);
