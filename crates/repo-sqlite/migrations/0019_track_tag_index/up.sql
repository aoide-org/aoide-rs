-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Required for efficient track searches.
CREATE INDEX "idx_track_tag_track_id_facet_label" ON "track_tag" (
    "track_id",
    "facet",
    "label"
);
