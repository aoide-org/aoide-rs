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

CREATE TABLE IF NOT EXISTS media_dir_cache (
    -- row header (immutable)
    row_id                 INTEGER PRIMARY KEY,
    row_created_ms         INTEGER NOT NULL,
    -- row header (mutable)
    row_updated_ms         INTEGER NOT NULL,
    -- relations (immutable)
    collection_id          INTEGER NOT NULL,
    -- properties
    uri                    TEXT NOT NULL,
    status                 TINYINT NOT NULL, -- 0 = current, 1 = outdated (scheduled for updating), 2 = updated, 3 = lost (not found during updating)
    digest                 BINARY,           -- cryptographic hash of the directory's contents (file metadata)
    --
    FOREIGN KEY(collection_id) REFERENCES collection(row_id),
    UNIQUE (collection_id, uri)
);
