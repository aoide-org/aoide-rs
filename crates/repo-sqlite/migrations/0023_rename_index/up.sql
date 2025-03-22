-- SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

DROP INDEX "udx_track_actor_track_id_scope_role_where_kind_main";
CREATE UNIQUE INDEX "udx_track_actor_track_id_scope_role_where_kind_summary" ON "track_actor" (
    "track_id",
    "scope",
    "role"
) WHERE "kind"=0;
