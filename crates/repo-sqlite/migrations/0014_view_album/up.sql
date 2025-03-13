-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Re-create the views.
DROP VIEW IF EXISTS view_album_artist_initial;
DROP VIEW IF EXISTS view_album;

CREATE VIEW view_album AS
SELECT
MIN(track.row_id) AS phantom_id,
album_artist.name AS artist,
album_title.name AS title,
COUNT(track.row_id) AS track_count,
GROUP_CONCAT(track.row_id) AS track_id_concat,
track.album_kind AS kind,
track.publisher AS publisher,
MIN(recorded_at_yyyymmdd) AS min_recorded_at_yyyymmdd,
MAX(recorded_at_yyyymmdd) AS max_recorded_at_yyyymmdd,
MIN(released_at_yyyymmdd) AS min_released_at_yyyymmdd,
MAX(released_at_yyyymmdd) AS max_released_at_yyyymmdd,
MIN(released_orig_at_yyyymmdd) AS min_released_orig_at_yyyymmdd,
MAX(released_orig_at_yyyymmdd) AS max_released_orig_at_yyyymmdd
FROM track
JOIN track_actor AS album_artist ON track.row_id=album_artist.track_id AND album_artist.scope=1 AND album_artist.kind=0 AND album_artist.role=0
JOIN track_title AS album_title ON track.row_id=album_title.track_id AND album_title.scope=1 AND album_title.kind=0
GROUP BY track.album_kind,track.publisher,album_artist.name,album_title.name;

CREATE VIEW view_album_artist_initial AS
SELECT
upper(substr(artist,1,1)) AS artist_initial,
count(*) AS album_count
FROM view_album
GROUP BY artist_initial;
