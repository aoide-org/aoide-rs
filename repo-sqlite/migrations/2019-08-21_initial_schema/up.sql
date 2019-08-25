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

-----------------------------------------------------------------------
-- Collections
-----------------------------------------------------------------------

CREATE TABLE tbl_collection (
    id                       INTEGER PRIMARY KEY,
    uid                      BINARY(24) NOT NULL,
    rev_ver                   INTEGER NOT NULL,
    rev_ts                   INTEGER NOT NULL,
    name                     TEXT NOT NULL,
    desc                     TEXT,
    UNIQUE (uid)
);

CREATE INDEX idx_collection_name ON tbl_collection (
    name
);

-----------------------------------------------------------------------
-- Tracks
-----------------------------------------------------------------------

CREATE TABLE tbl_track (
    id                       INTEGER PRIMARY KEY,
    uid                      BINARY(24) NOT NULL,
    rev_ver                  INTEGER NOT NULL,
    rev_ts                   INTEGER NOT NULL,
    data_fmt                 INTEGER NOT NULL,  -- serialization format
    data_vmaj                INTEGER NOT NULL,  -- serialization version for data migration - breaking changes
    data_vmin                INTEGER NOT NULL,  -- serialization version for data migration - backward-compatible changes
    data_blob                BLOB NOT NULL,     -- serialized track entity
    UNIQUE (uid)
);

CREATE INDEX IF NOT EXISTS idx_track_rev_ts ON tbl_track (
    rev_ts
);

CREATE TABLE aux_track_collection (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    collection_uid           BINARY(24) NOT NULL,
    since                    DATETIME NOT NULL, -- UTC
    color_code               INTEGER,           -- 0xAARRGGBB (hex)
    play_count               INTEGER,
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id, collection_uid)
);

-- Index with a permutation of the unique constraint to optimize
-- the performance of subselects, joins, and filtering. See also:
-- https://gitlab.com/uklotzde/aoide-rs/issues/12
-- https://www.sqlite.org/queryplanner.html
CREATE INDEX idx_track_collection_collection_uid_track ON aux_track_collection (
    collection_uid, track_id
);

CREATE INDEX IF NOT EXISTS idx_track_collection_since ON aux_track_collection (
    since
);

CREATE TABLE aux_track_source (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    uri                      TEXT NOT NULL,     -- RFC 3986
    uri_decoded              TEXT NOT NULL,     -- percent-decoded URI
    media_type               TEXT NOT NULL,     -- RFC 6838
    audio_channel_count      INTEGER,           -- number of channels
    audio_duration           REAL,              -- milliseconds
    audio_samplerate         INTEGER,           -- Hz
    audio_bitrate            INTEGER,           -- bits per second (bps)
    audio_loudness           REAL,              -- LUFS (dB)
    audio_enc_name           TEXT,              -- encoded by
    audio_enc_settings       TEXT,              -- encoder settings
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id, uri),                     -- at most one source per URI for each track
    UNIQUE (track_id, media_type)             -- at most one source per content type for each track
);

-- Index with a permutation of the unique constraint to optimize
-- the performance of subselects, joins, and filtering. See also:
-- https://gitlab.com/uklotzde/aoide-rs/issues/12
-- https://www.sqlite.org/queryplanner.html
CREATE INDEX idx_track_source_uri_track ON aux_track_source (
    uri, track_id
);

-- Index with a permutation of the unique constraint to optimize
-- the performance of subselects, joins, and filtering. See also:
-- https://gitlab.com/uklotzde/aoide-rs/issues/12
-- https://www.sqlite.org/queryplanner.html
CREATE INDEX idx_track_source_media_type_track ON aux_track_source (
    media_type, track_id
);

CREATE TABLE aux_track_brief (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    track_title              TEXT,
    track_artist             TEXT,
    track_composer           TEXT,
    album_title              TEXT,
    album_artist             TEXT,
    release_year             INTEGER,
    track_number              INTEGER, -- > 0
    track_total              INTEGER, -- > 0
    disc_number               INTEGER, -- > 0
    disc_total               INTEGER, -- > 0
    music_tempo              REAL, -- beats per minute (bpm)
    music_key                TINYINT, -- {(0), 1, ..., 24}
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id)
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

CREATE TABLE aux_tag_facet (
    id                       INTEGER PRIMARY KEY,
    facet                    TEXT NOT NULL COLLATE NOCASE,
    UNIQUE (facet)
);

CREATE TABLE aux_tag_label (
    id                       INTEGER PRIMARY KEY,
    label                    TEXT NOT NULL,
    UNIQUE (label)
);

CREATE TABLE aux_track_tag (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    facet_id                 INTEGER,
    label_id                 INTEGER,
    score                    REAL NOT NULL, -- [0.0, 1.0]
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    FOREIGN KEY(facet_id) REFERENCES aux_tag_facet(id),
    FOREIGN KEY(label_id) REFERENCES aux_tag_label(id),
    UNIQUE (track_id, facet_id, label_id)
);

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
