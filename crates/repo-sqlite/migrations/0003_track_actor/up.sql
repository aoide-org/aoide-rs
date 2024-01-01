-- SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS track_actor (
    row_id                   INTEGER PRIMARY KEY,
    -- relations (immutable)
    track_id                 INTEGER NOT NULL,
    -- properties
    scope                    INTEGER NOT NULL, -- 0: track, 1: album
    kind                     INTEGER NOT NULL,
    role                     INTEGER NOT NULL,
    name                     TEXT NOT NULL,
    role_notes               TEXT,
    --
    FOREIGN KEY(track_id) REFERENCES track(row_id) ON DELETE CASCADE
) STRICT;

DROP INDEX IF EXISTS idx_track_actor_track_id;
CREATE INDEX idx_track_actor_track_id ON track_actor (
    track_id
);

-- Covering index (= contains all columns) for loading
-- Ordering of columns matches the canonical ordering on load
DROP INDEX IF EXISTS idx_track_actor_scope_role_kind_name;
CREATE INDEX idx_track_actor_scope_role_kind_name ON track_actor (
    scope,
    role,
    kind,
    name
);

-- Searching
DROP INDEX IF EXISTS idx_track_actor_name_role;
CREATE INDEX idx_track_actor_name_role ON track_actor (
    name,
    role
);
