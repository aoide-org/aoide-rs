CREATE TABLE track_vault (
    id                     INTEGER PRIMARY KEY,
    added                  DATETIME NOT NULL, -- implicit time zone UTC
    updated                DATETIME, -- implicit time zone UTC
    media_url              TEXT NOT NULL,
    media_type             TEXT NOT NULL, -- RFC 6838
    media_imported         DATETIME, -- most recent metadata import
    media_exported         DATETIME, -- most recent metadata export
    audio_duration         INTEGER NOT NULL, -- milliseconds
    audio_channels         INTEGER NOT NULL, -- number of channels
    audio_samplerate       INTEGER NOT NULL, -- Hz
    audio_bitrate          INTEGER NOT NULL, -- bits per second (bps)
    metadata_format        INTEGER NOT NULL, -- serialization format: 1 = JSON, 2 = BSON, 3 = CBOR, 4 = Bincode, ...
    metadata_version_major INTEGER NOT NULL, -- for metadata migration - breaking changes
    metadata_version_minor INTEGER NOT NULL, -- for metadata migration - backward-compatible changes
    metadata_blob          BLOB NOT NULL, -- serialized track metadata
    metadata_sha256        BLOB NOT NULL, -- serialized track metadata hash
    UNIQUE (media_url),
    UNIQUE (metadata_sha256)
);

CREATE TABLE track_overview (
    id                     INTEGER PRIMARY KEY,
    track_id               INTEGER,
    track_title            TEXT,
    track_subtitle         TEXT,
    track_artists          TEXT,
    track_composers        TEXT,
    track_conductors       TEXT,
    track_performers       TEXT,
    track_producers        TEXT,
    track_remixers         TEXT,
    track_number           INTEGER, -- > 0
    track_total            INTEGER, -- > 0
    disc_number            INTEGER, -- > 0
    disc_total             INTEGER, -- > 0
    album_title            TEXT,
    album_subtitle         TEXT,
    album_artists          TEXT,
    album_composers        TEXT,
    album_conductors       TEXT,
    album_performers       TEXT,
    album_producers        TEXT,
    album_grouping         TEXT,
    album_compilation      TINYINT, -- {0, 1}
    release_date           DATE, -- naive date, i.e. without any time zone
    release_label          TEXT,
    music_loudness         REAL, -- LUFS dB
    music_tempo            REAL, -- beats per minute (bpm)
    music_time_signature   TEXT, -- "numerator/denominator"
    music_key_signature    TINYINT, -- {1, ..., 24}
    music_acousticness     REAL, -- [0.0, 1.0]
    music_danceability     REAL, -- [0.0, 1.0]
    music_energy           REAL, -- [0.0, 1.0]
    music_instrumentalness REAL, -- [0.0, 1.0]
    music_liveness         REAL, -- [0.0, 1.0]
    music_popularity       REAL, -- [0.0, 1.0]
    music_positivity       REAL, -- [0.0, 1.0]
    music_speechiness      REAL, -- [0.0, 1.0]
    ratings_min            REAL, -- [0.0, 1.0]
    ratings_max            REAL, -- [0.0, 1.0]
    FOREIGN KEY(track_id) REFERENCES track_vault(id),
    UNIQUE (track_id)
);

CREATE TABLE track_fulltext (
    id                     INTEGER PRIMARY KEY,
    track_id               INTEGER,
    fulltext               CLOB NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track_vault(id),
    UNIQUE (track_id)
);

CREATE TABLE track_tags (
    id                     INTEGER PRIMARY KEY,
    track_id               INTEGER,
    facet                  TEXT,
    term                   TEXT NOT NULL,
    confidence             REAL NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track_vault(id),
    UNIQUE (track_id, facet, term)
);

CREATE TABLE track_comments (
    id                     INTEGER PRIMARY KEY,
    track_id               INTEGER,
    owner                  TEXT,
    comment                CLOB NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track_vault(id),
    UNIQUE (track_id, owner)
);

CREATE TABLE track_ratings (
    id                     INTEGER PRIMARY KEY,
    track_id               INTEGER,
    owner                  TEXT,
    rating                 REAL NOT NULL,
    FOREIGN KEY(track_id) REFERENCES track_vault(id),
    UNIQUE (track_id, owner)
);
