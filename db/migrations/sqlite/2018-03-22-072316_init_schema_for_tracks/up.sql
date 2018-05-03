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
    ser_blob                  BLOB NOT NULL      -- serialized track entity
);

CREATE TABLE tracks_media (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    uri                      TEXT NOT NULL,     -- RFC 3986
    content_type             TEXT NOT NULL,     -- RFC 6838
    sync_rev_ordinal         INTEGER,           -- most recent metadata synchronization
    sync_rev_timestamp       DATETIME,          -- most recent metadata synchronization
    audio_duration           INTEGER,           -- milliseconds
    audio_channels           INTEGER,           -- number of channels
    audio_samplerate         INTEGER,           -- Hz
    audio_bitrate            INTEGER,           -- bits per second (bps)
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (uri)
);

CREATE TABLE tracks_media_collection (
    id                       INTEGER PRIMARY KEY,
    media_id                 INTEGER NOT NULL,
    collection_uid           TEXT NOT NULL,
    FOREIGN KEY(media_id) REFERENCES tracks_media(id),
    UNIQUE(media_id, collection_uid)
);

CREATE TABLE tracks_overview (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    track_title              TEXT NOT NULL,
    track_subtitle           TEXT,
    track_artists            TEXT,
    track_composers          TEXT,
    track_conductors         TEXT,
    track_performers         TEXT,
    track_producers          TEXT,
    track_remixers           TEXT,
    track_number             INTEGER, -- > 0
    track_total              INTEGER, -- > 0
    disc_number              INTEGER, -- > 0
    disc_total               INTEGER, -- > 0
    album_title              TEXT,
    album_subtitle           TEXT,
    album_artists            TEXT,
    album_composers          TEXT,
    album_conductors         TEXT,
    album_performers         TEXT,
    album_producers          TEXT,
    album_grouping           TEXT,
    album_compilation        TINYINT, -- {0, 1}
    release_date             DATE, -- naive date, i.e. without any time zone
    release_label            TEXT,
    music_loudness           REAL, -- LUFS dB
    music_tempo              REAL, -- beats per minute (bpm)
    music_time_signature     TEXT, -- "numerator/denominator"
    music_key_signature      TINYINT, -- {1, ..., 24}
    music_acousticness       REAL, -- [0.0, 1.0]
    music_danceability       REAL, -- [0.0, 1.0]
    music_energy             REAL, -- [0.0, 1.0]
    music_instrumentalness   REAL, -- [0.0, 1.0]
    music_liveness           REAL, -- [0.0, 1.0]
    music_popularity         REAL, -- [0.0, 1.0]
    music_positivity         REAL, -- [0.0, 1.0]
    music_speechiness        REAL, -- [0.0, 1.0]
    ratings_min              REAL, -- [0.0, 1.0]
    ratings_max              REAL, -- [0.0, 1.0]
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id)
);

CREATE TABLE tracks_fulltext (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    fulltext                 CLOB NOT NULL,
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id)
);

CREATE TABLE tracks_tag (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    facet                    TEXT,
    term                     TEXT NOT NULL,
    confidence               REAL NOT NULL,
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id, facet, term)
);

CREATE TABLE tracks_comment (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    owner                    TEXT,
    comment                  CLOB NOT NULL,
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id, owner)
);

CREATE TABLE tracks_rating (
    id                       INTEGER PRIMARY KEY,
    track_id                 INTEGER NOT NULL,
    owner                    TEXT,
    rating                   REAL NOT NULL,
    FOREIGN KEY(track_id) REFERENCES tracks_entity(id),
    UNIQUE (track_id, owner)
);
