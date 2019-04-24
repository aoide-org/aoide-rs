-- aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

CREATE TABLE aux_marker_label (
    id                       INTEGER PRIMARY KEY,
    label                    TEXT NOT NULL,
    UNIQUE (label)
);

CREATE TABLE aux_track_marker (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    label_id                 INTEGER,
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    FOREIGN KEY(label_id) REFERENCES aux_marker_label(id),
    UNIQUE (track_id, label_id)
);
