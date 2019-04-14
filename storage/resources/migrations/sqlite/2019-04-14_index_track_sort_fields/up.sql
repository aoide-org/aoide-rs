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

CREATE INDEX IF NOT EXISTS idx_track_rev_ts ON tbl_track (
    rev_ts
);

CREATE INDEX IF NOT EXISTS idx_track_collection_since ON aux_track_collection (
    since
);

CREATE INDEX IF NOT EXISTS idx_track_brief_track_title ON aux_track_brief (
    track_title
);

CREATE INDEX IF NOT EXISTS idx_track_brief_track_artist ON aux_track_brief (
    track_artist
);

CREATE INDEX IF NOT EXISTS idx_track_brief_album_title ON aux_track_brief (
    album_title
);

CREATE INDEX IF NOT EXISTS idx_track_brief_album_artist ON aux_track_brief (
    album_artist
);

CREATE INDEX IF NOT EXISTS idx_track_brief_release_year ON aux_track_brief (
    release_year
);

CREATE INDEX IF NOT EXISTS idx_track_brief_music_tempo ON aux_track_brief (
    music_tempo
);
