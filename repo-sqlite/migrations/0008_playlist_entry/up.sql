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

CREATE TABLE IF NOT EXISTS playlist_entry (
    -- row header (immutable)
    row_id                   INTEGER PRIMARY KEY,
    row_created_ms           INTEGER NOT NULL,
    -- row header (mutable)
    row_updated_ms           INTEGER NOT NULL,
    -- relations (immutable)
    playlist_id              INTEGER NOT NULL,
    track_id                 INTEGER, -- NULL for separators
    -- private properties
    ordering                 INTEGER NOT NULL, -- does not affect row_created_ms/row_updated_ms
    -- properties
    added_at                 TEXT NOT NULL,
    added_ms                 INTEGER NOT NULL,
    title                    TEXT,
    notes                    TEXT,
    --
    UNIQUE(playlist_id, ordering),
    FOREIGN KEY(playlist_id) REFERENCES playlist(row_id),
    FOREIGN KEY(track_id) REFERENCES track(row_id)
);

CREATE INDEX IF NOT EXISTS idx_playlist_entry_row_created_ms_desc ON playlist (
    row_created_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_playlist_entry_row_updated_ms_desc ON playlist (
    row_updated_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_playlist_entry_added_ms_desc ON playlist_entry (
    added_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_playlist_entry_playlist_id_track_id ON playlist_entry (
    playlist_id,
    track_id
);

CREATE INDEX IF NOT EXISTS idx_playlist_entry_track_id ON playlist_entry (
    track_id
);
