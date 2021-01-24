// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::atomic::AtomicBool;

use super::*;

mod uc {
    pub use crate::usecases::media::*;
}

use aoide_core::entity::EntityUid;

use url::Url;

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DirScanParams {
    pub root_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DirScanSummary {
    pub current: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
    pub skipped: usize,
}

impl From<uc::DirScanSummary> for DirScanSummary {
    fn from(from: uc::DirScanSummary) -> Self {
        let uc::DirScanSummary {
            current,
            added,
            modified,
            orphaned,
            skipped,
        } = from;
        Self {
            current,
            added,
            modified,
            orphaned,
            skipped,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum DirScanOutcome {
    Finished(DirScanSummary),
    Aborted,
}

impl From<uc::DirScanOutcome> for DirScanOutcome {
    fn from(from: uc::DirScanOutcome) -> Self {
        match from {
            uc::DirScanOutcome::Finished(summary) => Self::Finished(summary.into()),
            uc::DirScanOutcome::Aborted => Self::Aborted,
        }
    }
}

pub type RequestBody = DirScanParams;
pub type ResponseBody = DirScanOutcome;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let RequestBody {
        root_url,
        max_depth,
    } = request_body;
    Ok(uc::scan_directories_recursively(
        &pooled_connection,
        collection_uid,
        &root_url,
        max_depth,
        abort_flag,
    )
    .map(Into::into)?)
}
