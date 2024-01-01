-- SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS media_source (
    -- row header (immutable)
    row_id                 INTEGER PRIMARY KEY,
    row_created_ms         INTEGER NOT NULL,
    -- row header (mutable)
    row_updated_ms         INTEGER NOT NULL,
    -- relations (immutable)
    collection_id          INTEGER NOT NULL,
    -- properties: collection
    collected_at           TEXT NOT NULL,
    collected_ms           INTEGER NOT NULL,
    -- properties: link
    content_link_path      TEXT NOT NULL,
    content_link_rev       INTEGER,
    -- properties: content metadata & digest
    content_type           TEXT NOT NULL,    -- RFC 6838 media type
    content_digest         BLOB,             -- cryptographic hash
    content_metadata_flags INTEGER NOT NULL, -- 0x01 = reliable, 0x02 = locked, 0x04 = stale
    -- properties: audio content metadata
    audio_duration_ms      REAL,             -- milliseconds
    audio_channel_count    INTEGER,          -- number of channels
    audio_channel_mask     INTEGER,          -- channel bit mask
    audio_samplerate_hz    REAL,             -- Hz
    audio_bitrate_bps      REAL,             -- bits per second (bps)
    audio_loudness_lufs    REAL,             -- LUFS (dB)
    audio_encoder          TEXT,             -- both name and settings, often referred to as encoded_by
    -- properties: artwork
    artwork_source         INTEGER,          -- 0 = Missing, 1 = Embedded, 2 = Linked (URI)
    artwork_uri            TEXT,             -- RFC 3986, absolute or relative to the media_source URI
    artwork_apic_type      INTEGER,          -- APIC picture type
    artwork_media_type     TEXT,             -- RFC 6838 media type
    artwork_digest         BLOB,             -- cryptographic artwork content hash
    artwork_size_width     INTEGER,
    artwork_size_height    INTEGER,
    artwork_color          INTEGER,          -- 0xRRGGBB (hex)
    artwork_thumbnail      BLOB,
    --
    FOREIGN KEY(collection_id) REFERENCES collection(row_id) ON DELETE CASCADE,
    UNIQUE (collection_id, content_link_path)
) STRICT;

DROP INDEX IF EXISTS idx_media_source_row_created_ms_desc;
CREATE INDEX idx_media_source_row_created_ms_desc ON media_source (
    row_created_ms DESC
);

DROP INDEX IF EXISTS idx_media_source_row_updated_ms_desc;
CREATE INDEX idx_media_source_row_updated_ms_desc ON media_source (
    row_updated_ms DESC
);

DROP INDEX IF EXISTS idx_media_source_collected_ms_desc;
CREATE INDEX idx_media_source_collected_ms_desc ON media_source (
    collected_ms DESC
);

DROP INDEX IF EXISTS idx_media_source_content_type;
CREATE INDEX idx_media_source_content_type ON media_source (
    content_type
);

DROP INDEX IF EXISTS idx_media_source_content_digest;
CREATE INDEX idx_media_source_content_digest ON media_source (
    content_digest
) WHERE content_digest IS NOT NULL;

DROP INDEX IF EXISTS idx_media_source_audio_duration_ms;
CREATE INDEX idx_media_source_audio_duration_ms ON media_source (
    audio_duration_ms
);
