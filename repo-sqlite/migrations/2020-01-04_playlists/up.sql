-- aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

-----------------------------------------------------------------------
-- Playlists
-----------------------------------------------------------------------

CREATE TABLE tbl_playlist (
    id                       INTEGER PRIMARY KEY,
    uid                      BINARY(24) NOT NULL,
    rev_ver                  INTEGER NOT NULL,
    rev_ts                   INTEGER NOT NULL,
    data_fmt                 INTEGER NOT NULL,  -- serialization format
    data_vmaj                INTEGER NOT NULL,  -- serialization version for data migration - breaking changes
    data_vmin                INTEGER NOT NULL,  -- serialization version for data migration - backward-compatible changes
    data_blob                BLOB NOT NULL,     -- serialized playlist entity
    UNIQUE (uid)
);

CREATE INDEX IF NOT EXISTS idx_playlist_rev_ts ON tbl_playlist (
    rev_ts
);

CREATE TABLE aux_playlist_brief (
    id                       INTEGER PRIMARY KEY,
    playlist_id              INTEGER NOT NULL,
    name                     TEXT NOT NULL,
    desc                     TEXT,
    rtype                    TEXT,
    color_code               INTEGER, -- 0xRRGGBB (hex)
    entries_count            INTEGER NOT NULL,
    entries_since_min        DATETIME, -- UTC
    entries_since_max        DATETIME, -- UTC
    FOREIGN KEY(playlist_id) REFERENCES tbl_playlist(id)
);

CREATE INDEX IF NOT EXISTS idx_playlist_brief_name ON aux_playlist_brief (
    name
);

CREATE INDEX IF NOT EXISTS idx_playlist_brief_rtype ON aux_playlist_brief (
    rtype
);

CREATE TABLE aux_playlist_track (
    id                       INTEGER PRIMARY KEY,
    playlist_id              INTEGER NOT NULL,
    track_uid                BINARY(24) NOT NULL,
    track_ref_count          INTEGER NOT NULL, -- number of occurrences in playlist
    FOREIGN KEY(playlist_id) REFERENCES tbl_playlist(id)
);

CREATE INDEX IF NOT EXISTS idx_playlist_track_track_uid ON aux_playlist_track (
    track_uid
);
