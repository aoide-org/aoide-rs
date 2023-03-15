// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Collection, EntityHeader, EntityUid};

use crate::util::roundtrip::PendingToken;

use super::FetchFilteredEntities;

#[derive(Debug, Clone)]
pub enum Task {
    Pending {
        token: PendingToken,
        task: PendingTask,
    },
    CreateEntity {
        new_collection: Collection,
    },
    UpdateEntity {
        entity_header: EntityHeader,
        modified_collection: Collection,
    },
    PurgeEntity {
        entity_uid: EntityUid,
    },
}

#[derive(Debug, Clone)]
pub enum PendingTask {
    FetchAllKinds,
    FetchFilteredEntities(FetchFilteredEntities),
}

#[cfg(feature = "webapi-backend")]
mod webapi;
