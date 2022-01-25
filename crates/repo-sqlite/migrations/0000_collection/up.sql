-- aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

CREATE TABLE IF NOT EXISTS collection (
    -- row header (immutable)
    row_id                 INTEGER PRIMARY KEY,
    row_created_ms         INTEGER NOT NULL,
    -- row header (mutable)
    row_updated_ms         INTEGER NOT NULL,
    -- entity header (immutable)
    entity_uid             BINARY(24) NOT NULL,
    -- entity header (mutable)
    entity_rev             INTEGER NOT NULL, -- RevisionNumber
    -- properties
    title                  TEXT NOT NULL,
    kind                   TEXT,
    notes                  TEXT,
    color_rgb              INTEGER, -- 0xRRGGBB (hex)
    color_idx              INTEGER, -- palette index
    media_source_path_kind TINYINT NOT NULL,
    media_source_root_url  TEXT,
    --
    UNIQUE (entity_uid), -- only the last revision is stored
    UNIQUE (kind, title)
);

CREATE INDEX IF NOT EXISTS idx_collection_row_created_ms_desc ON collection (
    row_created_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_collection_row_updated_ms_desc ON collection (
    row_updated_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_collection_title ON collection (
    title
);
