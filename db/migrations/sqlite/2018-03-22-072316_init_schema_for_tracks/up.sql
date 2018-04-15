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

CREATE TABLE collection_entity (
    id                      INTEGER PRIMARY KEY,
    uid                     TEXT NOT NULL,     -- globally unique identifier
    revno                   INTEGER NOT NULL,
    revdt                   DATETIME NOT NULL, -- implicit time zone (UTC)
    name                    TEXT NOT NULL,     -- display name
    UNIQUE (uid)
);

-- Activates a single collection for the track database. All tracks
-- become dirty when switching the collection.
CREATE TABLE active_collection (
    id                      INTEGER PRIMARY KEY DEFAULT 1, -- only a single row is stored in this table
    collection_id           INTEGER NOT NULL,
    FOREIGN KEY(collection_id) REFERENCES collection(id)
);

CREATE TABLE track_entity (
    id                      INTEGER PRIMARY KEY,
    uid                     TEXT NOT NULL,     -- globally unique identifier
    revno                   INTEGER NOT NULL,
    revdt                   DATETIME NOT NULL, -- implicit time zone (UTC)
    collection_id           INTEGER,
    -- The media/audio columns are set according to the collected resource of the collection.
    -- All media/audio columns are NULL if the track is not related to a collection.
    media_uri               TEXT,              -- RFC 3986
    media_content_type      TEXT,              -- RFC 6838
    media_sync_revno        INTEGER,           -- most recent metadata synchronization
    media_sync_revdt        DATETIME,          -- most recent metadata synchronization
    audio_duration          INTEGER,           -- milliseconds
    audio_channels          INTEGER,           -- number of channels
    audio_samplerate        INTEGER,           -- Hz
    audio_bitrate           INTEGER,           -- bits per second (bps)
    entity_fmt              INTEGER NOT NULL,  -- serialization format: 1 = JSON, 2 = BSON, 3 = CBOR, 4 = Bincode, ...
    entity_vermaj           INTEGER NOT NULL,  -- for data migration - breaking changes
    entity_vermin           INTEGER NOT NULL,  -- for data migration - backward-compatible changes
    entity_blob             BLOB NOT NULL,     -- serialized track entity
    UNIQUE (media_uri)                         -- each track can only be stored once in a library
    FOREIGN KEY(collection_id) REFERENCES active_collection(collection_id)
);

-- Keeps track of required and not yet finished track_* updates
-- after changes in the main track table. The track database is
-- consistent if this table is empty.
CREATE TABLE dirty_tracks (
    id                      INTEGER PRIMARY KEY,
    track_id                INTEGER NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track(id),
    UNIQUE (track_id)
);

CREATE TABLE track_overview (
    id                      INTEGER PRIMARY KEY,
    track_id                INTEGER NOT NULL,
    track_title             TEXT NOT NULL,
    track_subtitle          TEXT,
    track_artists           TEXT,
    track_composers         TEXT,
    track_conductors        TEXT,
    track_performers        TEXT,
    track_producers         TEXT,
    track_remixers          TEXT,
    track_number            INTEGER, -- > 0
    track_total             INTEGER, -- > 0
    disc_number             INTEGER, -- > 0
    disc_total              INTEGER, -- > 0
    album_title             TEXT,
    album_subtitle          TEXT,
    album_artists           TEXT,
    album_composers         TEXT,
    album_conductors        TEXT,
    album_performers        TEXT,
    album_producers         TEXT,
    album_grouping          TEXT,
    album_compilation       TINYINT, -- {0, 1}
    release_date            DATE, -- naive date, i.e. without any time zone
    release_label           TEXT,
    music_loudness          REAL, -- LUFS dB
    music_tempo             REAL, -- beats per minute (bpm)
    music_time_signature    TEXT, -- "numerator/denominator"
    music_key_signature     TINYINT, -- {1, ..., 24}
    music_acousticness      REAL, -- [0.0, 1.0]
    music_danceability      REAL, -- [0.0, 1.0]
    music_energy            REAL, -- [0.0, 1.0]
    music_instrumentalness  REAL, -- [0.0, 1.0]
    music_liveness          REAL, -- [0.0, 1.0]
    music_popularity        REAL, -- [0.0, 1.0]
    music_positivity        REAL, -- [0.0, 1.0]
    music_speechiness       REAL, -- [0.0, 1.0]
    ratings_min             REAL, -- [0.0, 1.0]
    ratings_max             REAL, -- [0.0, 1.0]
    FOREIGN KEY(track_id) REFERENCES track(id),
    UNIQUE (track_id)
);

CREATE TABLE track_fulltext (
    id                      INTEGER PRIMARY KEY,
    track_id                INTEGER NOT NULL,
    fulltext                CLOB NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track(id),
    UNIQUE (track_id)
);

CREATE TABLE track_collections (
    id                      INTEGER PRIMARY KEY,
    track_id                INTEGER NOT NULL,
    collection_uid          TEXT NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track(id),
    UNIQUE (track_id, collection_uid) -- each track is contained in any collection at most once
);

CREATE TABLE track_tags (
    id                      INTEGER PRIMARY KEY,
    track_id                INTEGER NOT NULL,
    facet                   TEXT,
    term                    TEXT NOT NULL,
    confidence              REAL NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track(id),
    UNIQUE (track_id, facet, term)
);

CREATE TABLE track_comments (
    id                      INTEGER PRIMARY KEY,
    track_id                INTEGER NOT NULL,
    owner                   TEXT,
    comment                 CLOB NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track(id),
    UNIQUE (track_id, owner)
);

CREATE TABLE track_ratings (
    id                      INTEGER PRIMARY KEY,
    track_id                INTEGER NOT NULL,
    owner                   TEXT,
    rating                  REAL NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track(id),
    UNIQUE (track_id, owner)
);
