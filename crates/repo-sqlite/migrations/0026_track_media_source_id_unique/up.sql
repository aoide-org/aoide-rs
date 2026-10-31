-- SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Each media source belongs to at most one track.
DROP INDEX "idx_track_media_source_id";
CREATE UNIQUE INDEX "udx_track_media_source_id" ON "track" (
    "media_source_id"
);
