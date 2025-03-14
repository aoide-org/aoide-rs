-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

-- Rename the predefined "genre" facet.
UPDATE "track_tag" SET "facet"='gnre' WHERE "facet"='genre';
