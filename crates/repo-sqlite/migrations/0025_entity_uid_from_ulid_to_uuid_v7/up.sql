-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Custom migration without any schema changes.
--
-- Replaces all existing ULID values with random UUID v7 values.
-- All external entity references (collections, playlists, tracks) become invalid.
--
-- The Tantivy search index has to be deleted manually to enforce a rebuild.
