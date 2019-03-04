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
    rev_ordinal              INTEGER NOT NULL,
    rev_timestamp            DATETIME NOT NULL, -- with implicit time zone (UTC)
    name                     TEXT NOT NULL,     -- display name
    description              TEXT,
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
    rev_ordinal              INTEGER NOT NULL,
    rev_timestamp            DATETIME NOT NULL, -- with implicit time zone (UTC)
    ser_fmt                  INTEGER NOT NULL,  -- serialization format: 1 = JSON, 2 = BSON, 3 = CBOR, 4 = Bincode, ...
    ser_ver_major            INTEGER NOT NULL,  -- serialization version for data migration - breaking changes
    ser_ver_minor            INTEGER NOT NULL,  -- serialization version for data migration - backward-compatible changes
    ser_blob                 BLOB NOT NULL,     -- serialized track entity
    UNIQUE (uid)
);

CREATE TABLE aux_track_overview (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    track_title              TEXT,
    track_subtitle           TEXT,
    track_work               TEXT,
    track_movement           TEXT,
    album_title              TEXT,
    album_subtitle           TEXT,
    released_at              DATE, -- naive date, i.e. without any time zone
    released_by              TEXT, -- record label
    release_copyright        TEXT,
    track_index              INTEGER, -- > 0
    track_count              INTEGER, -- > 0
    disc_index               INTEGER, -- > 0
    disc_count               INTEGER, -- > 0
    movement_index           INTEGER, -- > 0
    movement_count           INTEGER, -- > 0
    lyrics_explicit          TINYINT, -- {0, 1}
    album_compilation        TINYINT, -- {0, 1}
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id)
);

CREATE TABLE aux_track_summary (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    track_artist             TEXT,
    track_composer           TEXT,
    track_conductor          TEXT,
    track_performer          TEXT,
    track_producer           TEXT,
    track_remixer            TEXT,
    album_artist             TEXT,
    album_composer           TEXT,
    album_conductor          TEXT,
    album_performer          TEXT,
    album_producer           TEXT,
    ratings_min              REAL, -- [0.0, 1.0]
    ratings_max              REAL, -- [0.0, 1.0]
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id)
);

CREATE TABLE aux_track_collection (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    collection_uid           BINARY(24) NOT NULL,
    since                    DATETIME NOT NULL,
    color_code               INTEGER,           -- 0xAARRGGBB (hex)
    play_count               INTEGER,
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id, collection_uid)
);

-- Index with a mutation of the unique constraint to optimize
-- the performance of subselects, joins, and filtering. See also:
-- https://gitlab.com/uklotzde/aoide-rs/issues/12
-- https://www.sqlite.org/queryplanner.html
CREATE INDEX idx_track_collection_collection_uid_track ON aux_track_collection (
    collection_uid, track_id
);

CREATE TABLE aux_track_source (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    content_uri              TEXT NOT NULL,     -- RFC 3986
    content_uri_decoded      TEXT NOT NULL,     -- percent-decoded URI
    content_type             TEXT NOT NULL,     -- RFC 6838
    audio_channels_count     INTEGER,           -- number of channels
    audio_duration_ms        REAL,              -- milliseconds
    audio_samplerate_hz      INTEGER,           -- Hz
    audio_bitrate_bps        INTEGER,           -- bits per second (bps)
    audio_enc_name           TEXT,              -- encoded by
    audio_enc_settings       TEXT,              -- encoder settings
    metadata_sync_when       DATETIME,          -- most recent metadata synchronization
    metadata_sync_rev_ordinal INTEGER,          -- most recent metadata synchronization
    metadata_sync_rev_timestamp DATETIME,       -- most recent metadata synchronization
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id, content_uri),
    UNIQUE (track_id, content_type)             -- at most one URI per content type
);

-- Index with a mutation of the unique constraint to optimize
-- the performance of subselects, joins, and filtering. See also:
-- https://gitlab.com/uklotzde/aoide-rs/issues/12
-- https://www.sqlite.org/queryplanner.html
CREATE INDEX idx_track_source_content_uri_track ON aux_track_source (
    content_uri, track_id
);

-- Index with a mutation of the unique constraint to optimize
-- the performance of subselects, joins, and filtering. See also:
-- https://gitlab.com/uklotzde/aoide-rs/issues/12
-- https://www.sqlite.org/queryplanner.html
CREATE INDEX idx_track_source_content_type_track ON aux_track_source (
    content_type, track_id
);

CREATE TABLE aux_track_profile (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    tempo_bpm                REAL NOT NULL, -- beats per minute (bpm)
    time_sig_top             TINYINT NOT NULL, -- >= 0
    time_sig_bottom          TINYINT NOT NULL, -- >= 0
    key_sig_code             TINYINT NOT NULL, -- {(0), 1, ..., 24}
    acousticness_score       REAL, -- [0.0, 1.0]
    danceability_score       REAL, -- [0.0, 1.0]
    energy_score             REAL, -- [0.0, 1.0]
    instrumentalness_score   REAL, -- [0.0, 1.0]
    liveness_score           REAL, -- [0.0, 1.0]
    popularity_score         REAL, -- [0.0, 1.0]
    valence_score            REAL, -- [0.0, 1.0]
    speechiness_score        REAL, -- [0.0, 1.0]
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id)
);

CREATE TABLE aux_track_tag_term (
    id                       INTEGER PRIMARY KEY,
    term                     TEXT NOT NULL,
    UNIQUE (term)
);

CREATE TABLE aux_track_tag_facet (
    id                       INTEGER PRIMARY KEY,
    facet                    TEXT NOT NULL COLLATE NOCASE,
    UNIQUE (facet)
);

CREATE TABLE aux_track_tag (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    score                    REAL NOT NULL, -- [0.0, 1.0]
    term_id                  INTEGER NOT NULL,
    facet_id                 INTEGER,
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    FOREIGN KEY(term_id) REFERENCES aux_track_tag_term(id),
    FOREIGN KEY(facet_id) REFERENCES aux_track_tag_facet(id),
    UNIQUE (track_id, term_id, facet_id)
);

-- TODO: Create additional indexes on aux_track_tag when actually needed!

CREATE TABLE aux_track_comment (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    text                     CLOB NOT NULL,
    owner                    TEXT,
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id, owner)
);

-- TODO: Create additional indexes on aux_track_comment when actually needed!

CREATE TABLE aux_track_rating (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    score                    REAL NOT NULL, -- [0.0, 1.0]
    owner                    TEXT,
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id, owner)
);

-- TODO: Create additional indexes on aux_track_rating when actually needed!

CREATE TABLE aux_track_xref (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    origin                   TINYINT NOT NULL,
    reference                TEXT NOT NULL,
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE (track_id, origin, reference)
);

-- TODO: Create additional indexes on aux_track_xref when actually needed!

-----------------------------------------------------------------------
-- Tasks
-----------------------------------------------------------------------

CREATE TABLE tbl_pending_task (
    -- AUTOINCREMENT: Required for ordered execution of pending tasks
    id                       INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_uid           BLOB, -- 24-byte globally unique identifier
    job_type                 INTEGER NOT NULL,
    job_params               BLOB NOT NULL
);

CREATE TABLE tbl_pending_task_track (
    id                       INTEGER PRIMARY KEY,
    task_id                  INTEGER NOT NULL,
    track_id                 INTEGER NOT NULL,
    FOREIGN KEY(task_id) REFERENCES tbl_pending_task(id),
    FOREIGN KEY(track_id) REFERENCES tbl_track(id),
    UNIQUE(task_id, track_id)
);

-- Index with a mutation of the unique constraint to optimize
-- the performance of subselects, joins, and filtering. See also:
-- https://gitlab.com/uklotzde/aoide-rs/issues/12
-- https://www.sqlite.org/queryplanner.html
CREATE INDEX idx_pending_task_track ON tbl_pending_task_track (
    track_id, task_id
);
