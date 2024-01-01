-- SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS collection (
    -- row header (immutable)
    row_id                 INTEGER PRIMARY KEY,
    row_created_ms         INTEGER NOT NULL,
    -- row header (mutable)
    row_updated_ms         INTEGER NOT NULL,
    -- entity header (immutable)
    entity_uid             TEXT NOT NULL, -- ULID
    -- entity header (mutable)
    entity_rev             INTEGER NOT NULL, -- RevisionNumber
    -- properties
    title                  TEXT NOT NULL,
    kind                   TEXT,
    notes                  TEXT,
    color_rgb              INTEGER, -- 0xRRGGBB (hex)
    color_idx              INTEGER, -- palette index
    media_source_path_kind INTEGER NOT NULL,
    media_source_root_url  TEXT,
    --
    UNIQUE (entity_uid), -- only the last revision is stored
    UNIQUE (kind, title)
) STRICT;

DROP INDEX IF EXISTS idx_collection_row_created_ms_desc;
CREATE INDEX idx_collection_row_created_ms_desc ON collection (
    row_created_ms DESC
);

DROP INDEX IF EXISTS idx_collection_row_updated_ms_desc;
CREATE INDEX idx_collection_row_updated_ms_desc ON collection (
    row_updated_ms DESC
);

DROP INDEX IF EXISTS idx_collection_title;
CREATE INDEX idx_collection_title ON collection (
    title
);
