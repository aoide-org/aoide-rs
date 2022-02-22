// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::{
    entity::EntityUid,
    track::tag::{FACET_GENRE, FACET_MOOD},
};

use aoide_media::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    util::tag::{FacetedTagMappingConfigInner, TagMappingConfig},
};

use super::*;

mod uc {
    pub use aoide_usecases::media::tracker::import_files::ProgressEvent;
    pub use aoide_usecases_sqlite::media::tracker::import_files::*;
}

pub type RequestBody = aoide_core_api_json::media::tracker::import_files::Params;

pub type ResponseBody = aoide_core_api_json::media::tracker::import_files::Outcome;

#[tracing::instrument(
    name = "Importing media sources",
    skip(
        connection,
        report_progress_fn,
        abort_flag,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request<ReportProgressFn: FnMut(uc::ProgressEvent)>(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let params = request_body
        .try_into()
        .map_err(Into::into)
        .map_err(Error::BadRequest)?;
    let mut faceted_tag_mapping_config = FacetedTagMappingConfigInner::default();
    faceted_tag_mapping_config.insert(
        FACET_GENRE.to_owned().into(),
        TagMappingConfig {
            label_separator: ";".into(),
            split_score_attenuation: 0.75,
        },
    );
    faceted_tag_mapping_config.insert(
        FACET_MOOD.to_owned().into(),
        TagMappingConfig {
            label_separator: ";".into(),
            split_score_attenuation: 0.75,
        },
    );
    // FIXME: Replace hard-coded import flags
    let import_flags = ImportTrackFlags::all();
    let import_config = ImportTrackConfig {
        faceted_tag_mapping: faceted_tag_mapping_config.into(),
        flags: import_flags,
    };
    connection
        .transaction::<_, Error, _>(|| {
            uc::import_files(
                connection,
                collection_uid,
                &params,
                import_config,
                report_progress_fn,
                abort_flag,
            )
            .map_err(Into::into)
        })
        .map(Into::into)
}
