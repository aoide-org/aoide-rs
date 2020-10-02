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

DROP TABLE IF EXISTS aux_track_collection;

CREATE TABLE tbl_collection_track (
    id                       INTEGER PRIMARY KEY,
    collection_id            INTEGER NOT NULL,
    track_id                 INTEGER NOT NULL,
    added_ts                 INTEGER NOT NULL,
    color_rgb                INTEGER, -- 0xRRGGBB (hex)
    color_idx                INTEGER, -- palette index
    play_count               INTEGER,
    last_played_ts           INTEGER,
    FOREIGN KEY(collection_id) REFERENCES tbl_collection(id),
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (collection_id, track_id)
);

CREATE INDEX IF NOT EXISTS idx_collection_tracks_track_id ON tbl_collection_track (
    track_id
);
