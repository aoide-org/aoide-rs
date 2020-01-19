-- aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

CREATE TABLE tbl_collection (
    id                       INTEGER PRIMARY KEY,
    uid                      BINARY(24) NOT NULL,
    rev_no                   INTEGER NOT NULL,
    rev_ts                   INTEGER NOT NULL,
    name                     TEXT NOT NULL,
    desc                     TEXT,
    UNIQUE (uid)
);

CREATE INDEX idx_collection_name ON tbl_collection (
    name
);
