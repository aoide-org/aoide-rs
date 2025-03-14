-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Rename and re-create the table as proposed here: https://www.sqlite.org/lang_altertable.html

-- !!!This pragma is a no-op within a transaction!!!
-- Migrations are usually run within a transaction.
PRAGMA foreign_keys = OFF;

CREATE TABLE "track_tag_migrate" (
    "row_id"        INTEGER PRIMARY KEY,
    -- relations (immutable)
    "track_id"      INTEGER NOT NULL,
    -- properties
    "facet"         TEXT,          -- symbolic identifier
    "label"         TEXT,          -- arbitrary text without leading/trailing whitespace
    "score"         REAL NOT NULL, -- [0.0, 1.0]
    --
    FOREIGN KEY("track_id") REFERENCES "track"("row_id") ON DELETE CASCADE
) STRICT;
INSERT INTO "track_tag_migrate" SELECT * FROM "track_tag";
DROP TABLE "track_tag";
ALTER TABLE "track_tag_migrate" RENAME TO "track_tag";

-- Verify that all foreign key constraints are still valid.
PRAGMA foreign_key_check;

-- !!!This pragma is a no-op within a transaction!!!
-- Migrations are usually run within a transaction.
PRAGMA foreign_keys = ON;

-- NULL values are considered as distinct for UNIQUE indexes.
--
-- The columns facet and label are never both NULL.
--
-- See also:
--  - https://www.sqlite.org/nulls.html
--  - https://www.sqlite.org/partialindex.html
CREATE UNIQUE INDEX "udx_track_tag_track_id_facet" ON "track_tag" (
    "track_id",
    "facet"
) WHERE "label" IS NULL;
CREATE UNIQUE INDEX "udx_track_tag_track_id_label" ON "track_tag" (
    "track_id",
    "label"
) WHERE "facet" IS NULL;
CREATE UNIQUE INDEX "udx_track_tag_track_id_facet_label" ON "track_tag" (
    "track_id",
    "facet",
    "label"
) WHERE "facet" IS NOT NULL AND "label" IS NOT NULL;

-- Canonical ordering on load
CREATE INDEX "idx_track_tag_facet_label_score_desc" ON "track_tag" (
    "facet",
    "label",
    "score" DESC
) WHERE "facet" IS NOT NULL;

CREATE INDEX "idx_track_tag_label_score_desc" ON "track_tag" (
    "label",
    "score" DESC
);
