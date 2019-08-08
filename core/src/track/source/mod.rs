// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

use crate::audio::AudioContent;

///////////////////////////////////////////////////////////////////////
// TrackSource
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSource {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[validate(url)]
    pub uri: String,

    // The content_type uniquely identifies a TrackSource of
    // a Track, i.e. no duplicate content types are allowed
    // among the track sources of each track.
    #[serde(rename = "t", skip_serializing_if = "String::is_empty", default)]
    // TODO: Validate MIME type
    #[validate(length(min = 1))]
    pub content_type: String,

    #[serde(rename = "a", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub audio_content: Option<AudioContent>,
}

impl TrackSource {
    pub fn filter_slice_by_content_type<'a>(
        sources: &'a [TrackSource],
        content_type: &str,
    ) -> Option<&'a TrackSource> {
        debug_assert!(
            sources
                .iter()
                .filter(|source| source.content_type == content_type)
                .count()
                <= 1
        );
        sources
            .iter()
            .filter(|source| source.content_type == content_type)
            .nth(0)
    }
}
