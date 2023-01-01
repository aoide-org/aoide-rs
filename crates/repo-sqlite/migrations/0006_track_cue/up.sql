-- SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS track_cue (
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
    FOREIGN KEY(track_id) REFERENCES track(row_id) ON DELETE CASCADE,
    UNIQUE (track_id, bank_idx, slot_idx)
) STRICT;
