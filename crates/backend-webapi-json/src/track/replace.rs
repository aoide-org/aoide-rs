// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_json::track::{Entity, Track};
use aoide_usecases::InputError;

use super::*;

mod uc {
    pub(super) use aoide_core_api::track::replace::Summary;
    pub(super) use aoide_repo::track::ReplaceMode;
    pub(super) use aoide_usecases::track::{replace::Params, validate_input};
    pub(super) use aoide_usecases_sqlite::track::replace::*;
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ReplaceMode {
    CreateOnly,
    UpdateOnly,
    UpdateOrCreate,
}

impl From<ReplaceMode> for uc::ReplaceMode {
    fn from(from: ReplaceMode) -> Self {
        use ReplaceMode::*;
        match from {
            CreateOnly => Self::CreateOnly,
            UpdateOnly => Self::UpdateOnly,
            UpdateOrCreate => Self::UpdateOrCreate,
        }
    }
}

#[derive(Debug, Default, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<String>,
    pub not_created: Vec<Track>,
    pub not_updated: Vec<Track>,
}

impl From<uc::Summary> for Summary {
    fn from(from: uc::Summary) -> Self {
        let uc::Summary {
            created,
            updated,
            unchanged,
            skipped,
            failed,
            not_imported,
            not_created,
            not_updated,
        } = from;
        debug_assert!(skipped.is_empty());
        debug_assert!(failed.is_empty());
        debug_assert!(not_imported.is_empty());
        Self {
            created: created.into_iter().map(Into::into).collect(),
            updated: updated.into_iter().map(Into::into).collect(),
            unchanged: unchanged.into_iter().map(Into::into).collect(),
            not_created: not_created.into_iter().map(Into::into).collect(),
            not_updated: not_updated.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ReplaceMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_path_from_url: Option<bool>,
}

pub type RequestBody = Vec<Track>;

pub type ResponseBody = Summary;

pub fn handle_request(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    query_params: QueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let QueryParams {
        mode,
        resolve_path_from_url,
    } = query_params;
    let replace_mode = mode.unwrap_or(ReplaceMode::UpdateOrCreate);
    let resolve_path_from_url = resolve_path_from_url.unwrap_or(false);
    let params = uc::Params {
        mode: replace_mode.into(),
        resolve_path_from_url,
        preserve_collected_at: true,
        update_last_synchronized_rev: false,
    };
    let (tracks, errors): (Vec<_>, _) = request_body
        .into_iter()
        .map(|t| t.try_into().map_err(Error::BadRequest))
        .map(|res| {
            res.and_then(|t| {
                uc::validate_input(t)
                    .map(|(track, invalidities)| {
                        if !invalidities.is_empty() {
                            log::warn!("Replacing track {track:?} invalidities: {invalidities:?}",);
                        }
                        track
                    })
                    .map_err(|InputError(err)| err)
                    .map_err(Error::BadRequest)
            })
        })
        .partition(Result::is_ok);
    if let Some(err) = errors.into_iter().map(Result::unwrap_err).next() {
        return Err(err);
    }
    let tracks = tracks.into_iter().map(Result::unwrap);
    connection
        .transaction::<_, Error, _>(|| {
            uc::replace_many_by_media_source_content_path(
                connection,
                collection_uid,
                &params,
                tracks,
            )
            .map_err(Into::into)
        })
        .map(Into::into)
}
