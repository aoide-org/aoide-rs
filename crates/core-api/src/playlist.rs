// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::playlist::{Entity, EntriesSummary};

#[derive(Debug, Clone)]
pub struct EntityWithEntriesSummary {
    pub entity: Entity,
    pub entries: EntriesSummary,
}
