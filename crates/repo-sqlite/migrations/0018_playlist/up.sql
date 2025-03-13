-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Rename and re-create the table as proposed here: https://www.sqlite.org/lang_altertable.html

-- !!!This pragma is a no-op within a transaction!!!
-- Migrations are usually run within a transaction.
PRAGMA foreign_keys = OFF;

CREATE TABLE IF NOT EXISTS playlist_migrate (
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
    collection_id            INTEGER,
    -- properties (mutable)
    title                    TEXT NOT NULL,
    kind                     TEXT,
    notes                    TEXT,
    color_rgb                INTEGER, -- 0xRRGGBB (hex)
    color_idx                INTEGER, -- palette index
    flags                    INTEGER NOT NULL, -- bitmask of flags, e.g. locking to prevent unintended modifications
    --
    FOREIGN KEY(collection_id) REFERENCES collection(row_id) ON DELETE CASCADE
) STRICT;
INSERT INTO playlist_migrate SELECT * FROM playlist;
DROP TABLE playlist;
ALTER TABLE playlist_migrate RENAME TO playlist;

-- Verify that all foreign key constraints are still valid.
PRAGMA foreign_key_check;

-- !!!This pragma is a no-op within a transaction!!!
-- Migrations are usually run within a transaction.
PRAGMA foreign_keys = ON;

-- Only the last revision is stored.
CREATE UNIQUE INDEX udx_playlist_entity_uid ON collection (
    entity_uid
);

-- NULL values are considered as distinct for UNIQUE indexes.
--
-- See also:
--  - https://www.sqlite.org/nulls.html
--  - https://www.sqlite.org/partialindex.html
CREATE UNIQUE INDEX udx_playlist_collection_id_title_where_kind_null ON playlist (
    collection_id,
    title
) WHERE kind IS NULL;
CREATE UNIQUE INDEX udx_playlist_collection_id_title_kind ON playlist (
    collection_id,
    title,
    kind
) WHERE kind IS NOT NULL;

CREATE INDEX idx_playlist_row_created_ms_desc ON playlist (
    row_created_ms DESC
);

CREATE INDEX idx_playlist_row_updated_ms_desc ON playlist (
    row_updated_ms DESC
);

CREATE INDEX idx_playlist_kind_title ON playlist (
    kind,
    title
) WHERE kind IS NOT NULL;

CREATE INDEX idx_playlist_title ON playlist (
    title
);
