// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Collection, EntityHeader, EntityUid};

use crate::util::roundtrip::PendingToken;

#[derive(Debug)]
pub enum Task {
    FetchAllKinds {
        token: PendingToken,
    },
    FetchFilteredEntities {
        token: PendingToken,
        filter_by_kind: Option<String>,
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

#[cfg(feature = "webapi-backend")]
mod webapi;
