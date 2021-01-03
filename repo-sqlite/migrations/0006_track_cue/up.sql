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

CREATE TABLE IF NOT EXISTS track_cue (
    row_id                   INTEGER PRIMARY KEY,
    -- relations (immutable)
    track_id                 INTEGER NOT NULL,
    -- properties
    bank_idx                 SMALLINT NOT NULL, -- index for separating cues into banks (hot cues, loops, samples, ...)
    slot_idx                 SMALLINT,          -- optional index if the bank supports multiple slots
    -- either in or out position must be NOT NULL
    in_position_ms           REAL,     -- offset from start of media source in milliseconds
    out_position_ms          REAL,     -- offset from start of media source in milliseconds
    -- out_mode:
    -- NULL = continue playing at out position (default)
    --    0 = stop playing at out position
    --    1 = continue playing at in position of current slot (loop/repeat, i.e. rewind)
    --    2 = continue playing at in position of next slot
    out_mode                 TINYINT,
    label                    TEXT,
    color_rgb                INTEGER, -- 0xRRGGBB (hex)
    color_idx                INTEGER, -- palette index
    --
    FOREIGN KEY(track_id) REFERENCES track(row_id),
    UNIQUE (track_id, bank_idx, slot_idx)
);
