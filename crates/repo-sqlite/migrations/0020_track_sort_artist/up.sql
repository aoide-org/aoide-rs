-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE UNIQUE INDEX "udx_track_actor_track_id_scope_role_where_kind_main" ON "track_actor" (
    "track_id",
    "scope",
    "role"
) WHERE "kind"=0;
CREATE UNIQUE INDEX "udx_track_actor_track_id_scope_role_where_kind_sort" ON "track_actor" (
    "track_id",
    "scope",
    "role"
) WHERE "kind"=2;

-- Add indexed "sort_track_artist" column to "track".
ALTER TABLE "track" ADD COLUMN "sort_track_artist" TEXT DEFAULT NULL;
CREATE INDEX "idx_track_sort_track_artist" ON "track" (
    "sort_track_artist"
);

-- Populate "sort_track_artist" column.
UPDATE "track" SET "sort_track_artist" = (
    SELECT "name" FROM "track_actor" "x"
    WHERE "track"."row_id"="x"."track_id"
    AND "x"."scope"=0 AND "x"."role"=0 AND "x"."kind"=2
);
UPDATE "track" SET "sort_track_artist" = (
    SELECT "name" FROM "track_actor" "x"
    WHERE "track"."row_id"="x"."track_id"
    AND "sort_track_artist" IS NULL
    AND "x"."scope"=0 AND "x"."role"=0 AND "x"."kind"=0
);

-- Add indexed "sort_album_artist" column to "track".
ALTER TABLE "track" ADD COLUMN "sort_album_artist" TEXT DEFAULT NULL;
CREATE INDEX "idx_track_sort_album_artist" ON "track" (
    "sort_album_artist"
);

-- Populate "sort_album_artist" column.
UPDATE "track" SET "sort_album_artist" = (
    SELECT "name" FROM "track_actor" "x"
    WHERE "track"."row_id"="x"."track_id"
    AND "x"."scope"=1 AND "x"."role"=0 AND "x"."kind"=2
);
UPDATE "track" SET "sort_album_artist" = (
    SELECT "name" FROM "track_actor" "x"
    WHERE "track"."row_id"="x"."track_id"
    AND "sort_album_artist" IS NULL
    AND "x"."scope"=1 AND "x"."role"=0 AND "x"."kind"=0
);
