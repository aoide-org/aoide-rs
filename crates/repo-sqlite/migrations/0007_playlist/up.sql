-- SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS playlist (
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
    collection_id            INTEGER NOT NULL,
    -- properties (mutable)
    collected_at             TEXT NOT NULL,
    collected_ms             INTEGER NOT NULL,
    title                    TEXT NOT NULL,
    kind                     TEXT,
    notes                    TEXT,
    color_rgb                INTEGER, -- 0xRRGGBB (hex)
    color_idx                INTEGER, -- palette index
    flags                    INTEGER NOT NULL, -- bitmask of flags, e.g. locking to prevent unintended modifications
    --
    UNIQUE (entity_uid), -- only the last revision is stored
    FOREIGN KEY(collection_id) REFERENCES collection(row_id) ON DELETE CASCADE
) STRICT;

DROP INDEX IF EXISTS idx_playlist_row_created_ms_desc;
CREATE INDEX idx_playlist_row_created_ms_desc ON playlist (
    row_created_ms DESC
);

DROP INDEX IF EXISTS idx_playlist_row_updated_ms_desc;
CREATE INDEX idx_playlist_row_updated_ms_desc ON playlist (
    row_updated_ms DESC
);

DROP INDEX IF EXISTS idx_playlist_collection_id_collected_ms_desc;
CREATE INDEX idx_playlist_collection_id_collected_ms_desc ON playlist (
    collection_id,
    collected_ms DESC
);

DROP INDEX IF EXISTS idx_playlist_kind_title;
CREATE INDEX idx_playlist_kind_title ON playlist (
    kind,
    title
) WHERE kind IS NOT NULL;

DROP INDEX IF EXISTS idx_playlist_title;
CREATE INDEX idx_playlist_title ON playlist (
    title
);
