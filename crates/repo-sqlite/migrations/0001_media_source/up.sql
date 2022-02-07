-- aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    collected_ms           INTEGER,
    -- properties: content
    external_rev           INTEGER,
    path                   TEXT NOT NULL,
    content_type           TEXT NOT NULL,    -- RFC 6838 media type
    content_digest         BINARY,           -- cryptographic (audio) content hash
    content_metadata_flags TINYINT NOT NULL, -- 0x01 = reliable, 0x02 = locked, 0x04 = stale
    advisory_rating        TINYINT,          -- 0 = unrated, 1 = explicit, 2 = clean
    -- properties: audio content
    audio_duration_ms      REAL,             -- milliseconds
    audio_channel_count    INTEGER,          -- number of channels
    audio_samplerate_hz    REAL,             -- Hz
    audio_bitrate_bps      REAL,             -- bits per second (bps)
    audio_loudness_lufs    REAL,             -- LUFS (dB)
    audio_encoder          TEXT,             -- both name and settings, often referred to as encoded_by
    -- properties: artwork
    artwork_source         TINYINT,          -- 0 = Missing, 1 = Embedded, 2 = Linked (URI)
    artwork_uri            TEXT,             -- RFC 3986, absolute or relative to the media_source URI
    artwork_apic_type      TINYINT,          -- APIC picture type
    artwork_media_type     TEXT,             -- RFC 6838 media type
    artwork_digest         BINARY,           -- cryptographic artwork content hash
    artwork_size_width     INTEGER,
    artwork_size_height    INTEGER,
    artwork_thumbnail      BINARY,
    --
    FOREIGN KEY(collection_id) REFERENCES collection(row_id) ON DELETE CASCADE,
    UNIQUE (collection_id, path)
);

CREATE INDEX idx_media_source_row_created_ms_desc ON media_source (
    row_created_ms DESC
);

CREATE INDEX idx_media_source_row_updated_ms_desc ON media_source (
    row_updated_ms DESC
);

CREATE INDEX IF NOT EXISTS idx_media_source_collected_ms_desc ON media_source (
    collected_ms DESC
);

CREATE INDEX idx_media_source_content_type ON media_source (
    content_type
);

CREATE INDEX idx_media_source_content_digest ON media_source (
    content_digest
) WHERE content_digest IS NOT NULL;

CREATE INDEX idx_media_source_audio_duration_ms ON media_source (
    audio_duration_ms
);
