-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Rename and re-create the table as proposed here: https://www.sqlite.org/lang_altertable.html

-- !!!This pragma is a no-op within a transaction!!!
-- Migrations are usually run within a transaction.
PRAGMA foreign_keys = OFF;

CREATE TABLE "collection_migrate" (
    -- row header (immutable)
    "row_id"                 INTEGER PRIMARY KEY,
    "row_created_ms"         INTEGER NOT NULL,
    -- row header (mutable)
    "row_updated_ms"         INTEGER NOT NULL,
    -- entity header (immutable)
    "entity_uid"             TEXT NOT NULL, -- ULID
    -- entity header (mutable)
    "entity_rev"             INTEGER NOT NULL, -- RevisionNumber
    -- properties
    "title"                  TEXT NOT NULL,
    "kind"                   TEXT,
    "notes"                  TEXT,
    "color_rgb"              INTEGER, -- 0xRRGGBB (hex)
    "color_idx"              INTEGER, -- palette index
    "media_source_path_kind" INTEGER NOT NULL,
    "media_source_root_url"  TEXT
) STRICT;
INSERT INTO "collection_migrate" SELECT * FROM "collection";
DROP TABLE "collection";
ALTER TABLE "collection_migrate" RENAME TO "collection";

-- Verify that all foreign key constraints are still valid.
PRAGMA foreign_key_check;

-- !!!This pragma is a no-op within a transaction!!!
-- Migrations are usually run within a transaction.
PRAGMA foreign_keys = ON;

-- Only the last revision is stored.
CREATE UNIQUE INDEX "udx_collection_entity_uid" ON "collection" (
    "entity_uid"
);

-- NULL values are considered as distinct for UNIQUE indexes.
-- Therefore we need to maintain 2 separate partial UNIQUE indexes to enforce the desired constraints.
--
-- Creating the first partial index may fail if conflicting titles without a kind (= NULL) exist.
-- In this case the affected collections need to be deleted manually.
--
-- See also:
--  - https://www.sqlite.org/nulls.html
--  - https://www.sqlite.org/partialindex.html
CREATE UNIQUE INDEX "udx_collection_title_where_kind_null" ON "collection" (
    "title"
) WHERE "kind" IS NULL;
CREATE UNIQUE INDEX "udx_collection_kind_title" ON "collection" (
    "kind",
    "title"
) WHERE "kind" IS NOT NULL;

CREATE INDEX "idx_collection_row_created_ms_desc" ON "collection" (
    "row_created_ms" DESC
);

CREATE INDEX "idx_collection_row_updated_ms_desc" ON "collection" (
    "row_updated_ms" DESC
);

CREATE INDEX "idx_collection_title" ON "collection" (
    "title"
);
