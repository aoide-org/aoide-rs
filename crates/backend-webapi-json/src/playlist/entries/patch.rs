// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::EntityUidTyped;
use aoide_core_api_json::playlist::{EntityWithEntriesSummary, export_entity_with_entries_summary};
use aoide_core_json::entity::EntityUid as SerdeEntityUid;

use super::*;

mod uc {
    pub(super) use aoide_usecases::playlist::entries::PatchOperation;
    pub(super) use aoide_usecases_sqlite::playlist::patch_entries;
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PlaylistRef {
    uid: SerdeEntityUid,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
        use PatchOperation as From;
        match from {
            From::Append { entries } => Self::Append {
                entries: entries.into_iter().map(Into::into).collect(),
            },
            From::Prepend { entries } => Self::Prepend {
                entries: entries.into_iter().map(Into::into).collect(),
            },
            From::Insert { before, entries } => Self::Insert {
                before,
                entries: entries.into_iter().map(Into::into).collect(),
            },
            From::CopyAll { source_playlist } => {
                let PlaylistRef { uid } = source_playlist;
                Self::CopyAll {
                    source_playlist_uid: EntityUidTyped::from_untyped(uid),
                }
            }
            From::Move { start, end, delta } => Self::Move {
                range: start..end,
                delta,
            },
            From::Remove { start, end } => Self::Remove { range: start..end },
            From::RemoveAll => Self::RemoveAll,
            From::ReverseAll => Self::ReverseAll,
            From::ShuffleAll => Self::ShuffleAll,
        }
    }
}

pub type RequestBody = Vec<PatchOperation>;

pub type ResponseBody = EntityWithEntriesSummary;

#[expect(clippy::needless_pass_by_value)] // consume arguments
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
            uc::patch_entries(
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
