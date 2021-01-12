-- aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
--
-- This program is free software: you can redistribute it and/or modify
-- it under the terms of the GNU Affero General Public License as
-- published by the Free Software Foundation, either version 3 of the
-- License, or (at your option) any later version.
--
-- This program is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY; without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU Affero General Public License for more details.
--
-- You should have received a copy of the GNU Affero General Public License
-- along with this program.  If not, see <https://www.gnu.org/licenses/>.

CREATE TABLE IF NOT EXISTS track_tag (
    row_id        INTEGER PRIMARY KEY,
    -- relations (immutable)
    track_id      INTEGER NOT NULL,
    -- properties
    facet         TEXT,
    label         TEXT,
    score         REAL NOT NULL, -- [0.0, 1.0]
    --
    FOREIGN KEY(track_id) REFERENCES track(row_id),
    UNIQUE (track_id, facet, label)
);

-- Canonical ordering on load
CREATE INDEX IF NOT EXISTS idx_track_tag_facet_label_score_desc ON track_tag (
    facet,
    label,
    score DESC
);

CREATE INDEX IF NOT EXISTS idx_track_tag_label_score_desc ON track_tag (
    label,
    score DESC
);
