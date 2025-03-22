-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

DROP INDEX "idx_track_title_track_id";
CREATE UNIQUE INDEX "udx_track_title_track_id_scope_kind" ON "track_title" (
    "track_id",
    "scope",
    "kind"
);

-- Add indexed "sort_track_title" column to "track".
ALTER TABLE "track" ADD COLUMN "sort_track_title" TEXT DEFAULT NULL;
CREATE INDEX "idx_track_sort_track_title" ON "track" (
    "sort_track_title"
);

-- Populate "sort_track_title" column.
UPDATE "track" SET "sort_track_title" = (
    SELECT "name" FROM "track_title" "x"
    WHERE "track"."row_id"="x"."track_id"
    AND "x"."scope"=0 AND "x"."kind"=2
);
UPDATE "track" SET "sort_track_title" = (
    SELECT "name" FROM "track_title" "x"
    WHERE "track"."row_id"="x"."track_id"
    AND "sort_track_title" IS NULL
    AND "x"."scope"=0 AND "x"."kind"=0
);

-- Add indexed "sort_album_title" column to "track".
ALTER TABLE "track" ADD COLUMN "sort_album_title" TEXT DEFAULT NULL;
CREATE INDEX "idx_track_sort_album_title" ON "track" (
    "sort_album_title"
);

-- Populate "sort_album_title" column.
UPDATE "track" SET "sort_album_title" = (
    SELECT "name" FROM "track_title" "x"
    WHERE "track"."row_id"="x"."track_id"
    AND "x"."scope"=1 AND "x"."kind"=2
);
UPDATE "track" SET "sort_album_title" = (
    SELECT "name" FROM "track_title" "x"
    WHERE "track"."row_id"="x"."track_id"
    AND "sort_album_title" IS NULL
    AND "x"."scope"=1 AND "x"."kind"=0
);
