-- SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS track_tag (
    row_id        INTEGER PRIMARY KEY,
    -- relations (immutable)
    track_id      INTEGER NOT NULL,
    -- properties
    facet         TEXT,          -- symbolic identifier
    label         TEXT,          -- arbitrary text without leading/trailing whitespace
    score         REAL NOT NULL, -- [0.0, 1.0]
    --
    FOREIGN KEY(track_id) REFERENCES track(row_id) ON DELETE CASCADE,
    UNIQUE (track_id, facet, label)
) STRICT;

-- Canonical ordering on load
DROP INDEX IF EXISTS idx_track_tag_facet_label_score_desc;
CREATE INDEX idx_track_tag_facet_label_score_desc ON track_tag (
    facet,
    label,
    score DESC
) WHERE facet IS NOT NULL;

DROP INDEX IF EXISTS idx_track_tag_label_score_desc;
CREATE INDEX idx_track_tag_label_score_desc ON track_tag (
    label,
    score DESC
);
