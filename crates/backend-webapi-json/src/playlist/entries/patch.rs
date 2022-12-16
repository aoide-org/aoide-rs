// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::entity::EntityUidTyped;
use aoide_core_api_json::playlist::{export_entity_with_entries_summary, EntityWithEntriesSummary};

use aoide_core_json::entity::EntityUid as SerdeEntityUid;

use super::*;

mod uc {
    pub(super) use aoide_usecases::playlist::entries::PatchOperation;
    pub(super) use aoide_usecases_sqlite::playlist::entries::patch;
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlaylistRef {
    uid: SerdeEntityUid,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum PatchOperation {
    Append {
        entries: Vec<Entry>,
    },
    Prepend {
        entries: Vec<Entry>,
    },
    Insert {
        before: usize,
        entries: Vec<Entry>,
    },
    CopyAll {
        source_playlist: PlaylistRef,
    },
    Move {
        start: usize,
        end: usize,
        delta: isize,
    },
    Remove {
        start: usize,
        end: usize,
    },
    RemoveAll,
    ReverseAll,
    ShuffleAll,
}

impl From<PatchOperation> for uc::PatchOperation {
    fn from(from: PatchOperation) -> Self {
        use PatchOperation::*;
        match from {
            Append { entries } => Self::Append {
                entries: entries.into_iter().map(Into::into).collect(),
            },
            Prepend { entries } => Self::Prepend {
                entries: entries.into_iter().map(Into::into).collect(),
            },
            Insert { before, entries } => Self::Insert {
                before,
                entries: entries.into_iter().map(Into::into).collect(),
            },
            CopyAll { source_playlist } => {
                let PlaylistRef { uid } = source_playlist;
                Self::CopyAll {
                    source_playlist_uid: EntityUidTyped::from_untyped(uid),
                }
            }
            Move { start, end, delta } => Self::Move {
                range: start..end,
                delta,
            },
            Remove { start, end } => Self::Remove { range: start..end },
            RemoveAll => Self::RemoveAll,
            ReverseAll => Self::ReverseAll,
            ShuffleAll => Self::ShuffleAll,
        }
    }
}

pub type RequestBody = Vec<PatchOperation>;

pub type ResponseBody = EntityWithEntriesSummary;

pub fn handle_request(
    connection: &mut DbConnection,
    uid: EntityUid,
    query_params: EntityRevQueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let EntityRevQueryParams { rev } = query_params;
    let entity_header = _core::EntityHeader { uid, rev };
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::patch(
                connection,
                &entity_header,
                request_body.into_iter().map(Into::into),
            )
            .map_err(Into::into)
        })
        .map(|(_, entity_with_entries_summary)| {
            export_entity_with_entries_summary(entity_with_entries_summary)
        })
}
