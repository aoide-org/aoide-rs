-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Remove column added_at from playlist entry and instead store
-- the time_zone in playlists.

ALTER TABLE "playlist" ADD COLUMN "iana_tz" TEXT;

ALTER TABLE "playlist_entry" DROP COLUMN "added_at";
