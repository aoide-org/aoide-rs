-- SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS media_tracker_directory (
    -- row header (immutable)
    row_id                 INTEGER PRIMARY KEY,
    row_created_ms         INTEGER NOT NULL,
    -- row header (mutable)
    row_updated_ms         INTEGER NOT NULL,
    -- relations (immutable)
    collection_id          INTEGER NOT NULL,
    -- properties
    content_path           TEXT NOT NULL,
    status                 INTEGER NOT NULL, -- 0 = current, 1 = outdated, 3 = added, 3 = modified, 4 = orphaned
    digest                 BLOB,             -- cryptographic hash of the directory's contents (file metadata)
    --
    FOREIGN KEY(collection_id) REFERENCES collection(row_id) ON DELETE CASCADE,
    UNIQUE (collection_id, content_path)
) STRICT;

CREATE TABLE IF NOT EXISTS media_tracker_source (
    -- relations (immutable)
    directory_id           INTEGER NOT NULL,
    source_id              INTEGER NOT NULL,
    --
    FOREIGN KEY(directory_id) REFERENCES media_tracker_directory(row_id) ON DELETE CASCADE,
    FOREIGN KEY(source_id) REFERENCES media_source(row_id) ON DELETE CASCADE,
    UNIQUE (source_id)
) STRICT;

DROP INDEX IF EXISTS idx_media_tracker_source_directory_id;
CREATE INDEX idx_media_tracker_source_directory_id ON media_tracker_source (
    directory_id
);
