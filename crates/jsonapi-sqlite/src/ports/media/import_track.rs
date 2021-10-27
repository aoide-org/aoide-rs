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

use url::Url;

use aoide_core::{
    track::tag::{FACET_GENRE, FACET_MOOD},
    util::clock::DateTime,
};

use aoide_media::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    resolver::{ResolveFromUrlError, SourcePathResolver, UrlResolver},
    util::tag::{FacetedTagMappingConfigInner, TagMappingConfig},
};

use aoide_core_serde::track::Track;

use super::*;

mod uc {
    pub use crate::usecases::media::*;
    pub use aoide_usecases::media::*;
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    pub url: Url,
}

pub type ResponseBody = Option<Track>;

pub fn handle_request(query_params: QueryParams) -> Result<ResponseBody> {
    let QueryParams { url } = query_params;
    // FIXME: Replace hard-coded tag mapping config
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
    let config = ImportTrackConfig {
        faceted_tag_mapping: faceted_tag_mapping_config.into(),
    };
    let source_path = match VirtualFilePathResolver::new().resolve_path_from_url(&url) {
        Ok(path) => path,
        Err(ResolveFromUrlError::InvalidUrl) => {
            let path = match UrlResolver.resolve_path_from_url(&url) {
                Ok(path) => path,
                Err(ResolveFromUrlError::InvalidUrl) => url.to_string().into(),
                Err(ResolveFromUrlError::Other(err)) => {
                    return Err(Error::Other(err));
                }
            };
            tracing::warn!("Trying to import from {}", path);
            path
        }
        Err(ResolveFromUrlError::Other(err)) => {
            return Err(Error::Other(err));
        }
    };
    let track = match uc::import_track_from_local_file_path(
        &VirtualFilePathResolver::new(),
        source_path,
        uc::SynchronizedImportMode::Always,
        &config,
        ImportTrackFlags::all(),
        DateTime::now_local(),
    )? {
        uc::ImportTrackFromFileOutcome::Imported(track) => Some(track),
        uc::ImportTrackFromFileOutcome::SkippedSynchronized(_) => unreachable!(),
        uc::ImportTrackFromFileOutcome::SkippedDirectory => None,
    };
    Ok(track.map(Into::into))
}
