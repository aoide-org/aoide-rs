-- aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

CREATE TABLE IF NOT EXISTS playlist (
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
);

CREATE INDEX IF NOT EXISTS idx_playlist_row_created_ms_desc ON playlist (
    row_created_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_playlist_row_updated_ms_desc ON playlist (
    row_updated_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_playlist_collection_id_collected_ms_desc ON playlist (
    collection_id,
    collected_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_playlist_kind_title ON playlist (
    kind,
    title
) WHERE kind IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_playlist_title ON playlist (
    title
);
