-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Rename and re-create the table as proposed here: https://www.sqlite.org/lang_altertable.html

-- !!!This pragma is a no-op within a transaction!!!
-- Migrations are usually run within a transaction.
PRAGMA foreign_keys = OFF;

CREATE TABLE track_cue_migrate (
    row_id                   INTEGER PRIMARY KEY,
    -- relations (immutable)
    track_id                 INTEGER NOT NULL,
    -- properties
    bank_idx                 INTEGER NOT NULL, -- index for separating cues into banks (hot cues, loops, samples, ...)
    slot_idx                 INTEGER,          -- optional index if the bank supports multiple slots
    -- either in or out position must be NOT NULL
    in_position_ms           REAL,     -- offset from start of media source in milliseconds
    out_position_ms          REAL,     -- offset from start of media source in milliseconds
    -- out_mode:
    -- NULL = continue playing at out position (default)
    --    0 = stop playing at out position
    --    1 = continue playing at in position of current slot (loop/repeat, i.e. rewind)
    --    2 = continue playing at in position of next slot
    out_mode                 INTEGER,
    kind                     TEXT,
    label                    TEXT,
    color_rgb                INTEGER, -- 0xRRGGBB (hex)
    color_idx                INTEGER, -- palette index
    flags                    INTEGER NOT NULL, -- bitmask of flags, e.g. locking to prevent unintended modifications
    --
    FOREIGN KEY(track_id) REFERENCES track(row_id) ON DELETE CASCADE
) STRICT;
INSERT INTO track_cue_migrate SELECT * FROM track_cue;
DROP TABLE track_cue;
ALTER TABLE track_cue_migrate RENAME TO track_cue;

-- Verify that all foreign key constraints are still valid.
PRAGMA foreign_key_check;

-- !!!This pragma is a no-op within a transaction!!!
-- Migrations are usually run within a transaction.
PRAGMA foreign_keys = ON;

-- NULL values are considered as distinct for UNIQUE indexes.
--
-- See also:
--  - https://www.sqlite.org/nulls.html
--  - https://www.sqlite.org/partialindex.html
CREATE UNIQUE INDEX udx_track_cue_track_id_bank_idx_where_slot_idx_null ON track_cue (
    track_id,
    bank_idx
) WHERE slot_idx IS NULL;
CREATE UNIQUE INDEX udx_track_cue_track_id_bank_idx_slot_idx ON track_cue (
    track_id,
    bank_idx,
    slot_idx
) WHERE slot_idx IS NOT NULL;
