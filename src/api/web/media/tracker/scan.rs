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
    pub use crate::usecases::media::tracker::scan::*;
    pub use aoide_usecases::media::tracker::scan::ProgressEvent;
}

use aoide_core::entity::EntityUid;

use aoide_core_serde::usecases::media::tracker::scan::Outcome;

use tokio::sync::watch;
use url::Url;

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    pub root_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
}

pub type RequestBody = Params;
pub type ResponseBody = Outcome;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
    progress_event_tx: Option<&watch::Sender<Option<uc::ProgressEvent>>>,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let RequestBody {
        root_url,
        max_depth,
    } = request_body;
    uc::scan_directories_recursively(
        &pooled_connection,
        collection_uid,
        &root_url,
        max_depth,
        &mut |progress_event| {
            if let Some(progress_event_tx) = progress_event_tx {
                if progress_event_tx.send(Some(progress_event)).is_err() {
                    log::error!("Failed to send progress event");
                }
            }
        },
        abort_flag,
    )
    .map(Into::into)
    .map_err(Into::into)
}
