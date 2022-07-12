-- SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS track_title (
    row_id                   INTEGER PRIMARY KEY,
    -- relations (immutable)
    track_id                 INTEGER NOT NULL,
    -- properties
    scope                    TINYINT NOT NULL, -- 0: track, 1: album
    kind                     TINYINT NOT NULL,
    name                     TEXT NOT NULL,
    --
    FOREIGN KEY(track_id) REFERENCES track(row_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_track_title_track_id ON track_title (
    track_id
);

-- Canonical ordering on load
CREATE INDEX IF NOT EXISTS idx_track_title_scope_kind_name ON track_title (
    scope,
    kind,
    name
);
