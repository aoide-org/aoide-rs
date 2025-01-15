-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

CREATE TABLE IF NOT EXISTS collection_vfs (
    row_id                 INTEGER PRIMARY KEY,
    collection_id          INTEGER NOT NULL,
    excluded_content_path  TEXT NOT NULL,
    --
    FOREIGN KEY(collection_id) REFERENCES collection(row_id) ON DELETE CASCADE,
    UNIQUE (collection_id, excluded_content_path)
) STRICT;
