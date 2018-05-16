-- Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

CREATE TABLE collections_entity (
    id                       INTEGER PRIMARY KEY,
    uid                      TEXT NOT NULL,     -- globally unique identifier
    rev_ordinal              INTEGER NOT NULL,
    rev_timestamp            DATETIME NOT NULL, -- with implicit time zone (UTC)
    name                     TEXT NOT NULL,     -- display name
    description              TEXT,
    UNIQUE (uid)
);

CREATE TABLE tracks_entity (
    id                       INTEGER PRIMARY KEY,
    uid                      TEXT NOT NULL,     -- globally unique identifier
    rev_ordinal              INTEGER NOT NULL,
    rev_timestamp            DATETIME NOT NULL, -- with implicit time zone (UTC)
    ser_fmt                  INTEGER NOT NULL,  -- serialization format: 1 = JSON, 2 = BSON, 3 = CBOR, 4 = Bincode, ...
    ser_ver_major            INTEGER NOT NULL,  -- serialization version for data migration - breaking changes
    ser_ver_minor            INTEGER NOT NULL,  -- serialization version for data migration - backward-compatible changes
    ser_blob                 BLOB NOT NULL      -- serialized track entity
);

CREATE TABLE aux_tracks_resource (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    collection_uid           TEXT NOT NULL,
    collection_since         DATETIME NOT NULL,
    source_uri               TEXT NOT NULL,     -- RFC 3986
    source_sync_when         DATETIME,          -- most recent metadata synchronization
    source_sync_rev_ordinal  INTEGER,           -- most recent metadata synchronization
    source_sync_rev_timestamp DATETIME,         -- most recent metadata synchronization
    content_type             TEXT NOT NULL,     -- RFC 6838
    audio_duration_ms        REAL,              -- milliseconds
    audio_channels           INTEGER,           -- number of channels
    audio_samplerate_hz      INTEGER,           -- Hz
    audio_bitrate_bps        INTEGER,           -- bits per second (bps)
    audio_enc_name           TEXT,              -- encoded by
    audio_enc_settings       TEXT,              -- encoder settings
    color_code               INTEGER,           -- 0xAARRGGBB (hex)
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (collection_uid, track_id),
    UNIQUE (collection_uid, source_uri)
);

CREATE TABLE aux_tracks_overview (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    track_title              TEXT NOT NULL,
    track_number             INTEGER, -- > 0
    track_total              INTEGER, -- > 0
    disc_number              INTEGER, -- > 0
    disc_total               INTEGER, -- > 0
    album_title              TEXT,
    album_grouping           TEXT,
    album_compilation        TINYINT, -- {0, 1}
    release_date             DATE, -- naive date, i.e. without any time zone
    release_label            TEXT,
    lyrics_explicit          TINYINT, -- {0, 1}
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id)
);

CREATE TABLE aux_tracks_summary (
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
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id)
);

CREATE TABLE aux_tracks_music (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    music_loudness_db        REAL NOT NULL, -- LUFS dB
    music_tempo_bpm          REAL NOT NULL, -- beats per minute (bpm)
    music_time_sig_num       TINYINT NOT NULL, -- >= 0
    music_time_sig_denom     TINYINT NOT NULL, -- >= 0
    music_key_sig_code       TINYINT NOT NULL, -- {(0), 1, ..., 24}
    music_acousticness       REAL, -- [0.0, 1.0]
    music_danceability       REAL, -- [0.0, 1.0]
    music_energy             REAL, -- [0.0, 1.0]
    music_instrumentalness   REAL, -- [0.0, 1.0]
    music_liveness           REAL, -- [0.0, 1.0]
    music_popularity         REAL, -- [0.0, 1.0]
    music_valence            REAL, -- [0.0, 1.0]
    music_speechiness        REAL, -- [0.0, 1.0]
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id)
);

CREATE TABLE aux_tracks_ref (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    type                     TINYINT, -- 0=track, actor=1, album=2, release=3
    value                    TEXT NOT NULL,
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id, type, value)
);

CREATE TABLE aux_tracks_tag (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    facet                    TEXT,
    term                     TEXT NOT NULL,
    confidence               REAL NOT NULL,
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id, facet, term)
);

CREATE TABLE aux_tracks_comment (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    owner                    TEXT,
    comment                  CLOB NOT NULL,
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id, owner)
);

CREATE TABLE aux_tracks_rating (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    owner                    TEXT,
    rating                   REAL NOT NULL,
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id, owner)
);
