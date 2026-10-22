-- SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Database options that are persistent and only need to be applied once
PRAGMA encoding = 'UTF-8';

-- Schema options (implicit default schema: main)
PRAGMA journal_mode = WAL;        -- better write-concurrency
PRAGMA wal_checkpoint(TRUNCATE);  -- free some space by truncating possibly massive WAL files from the last run
PRAGMA auto_vacuum = INCREMENTAL; -- allows to use `PRAGMA incremental_vacuum` instead of `VACUUM`
