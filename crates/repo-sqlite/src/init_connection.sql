-- SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Connection options that need to be applied to each connection
PRAGMA busy_timeout = 5000;       -- wait for locks held by concurrently initialized connections instead of failing with "database is locked" (must be the first statement)
PRAGMA wal_autocheckpoint = 1000; -- write WAL changes back every 1000 pages (default), for an in average 1MB WAL file
PRAGMA automatic_index = 1;       -- detect and log missing indexes
PRAGMA foreign_keys = 1;          -- check foreign key constraints
PRAGMA defer_foreign_keys = 1;    -- delay enforcement of foreign key constraints until commit
PRAGMA recursive_triggers = 1;    -- for recursive ON CASCADE DELETE actions

-- Schema options (implicit default schema: main)
PRAGMA synchronous = NORMAL;      -- fsync only in critical moments, safe for `journal_mode = WAL`
PRAGMA secure_delete = 0;         -- avoid some disk I/O
