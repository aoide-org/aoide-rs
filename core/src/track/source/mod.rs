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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSource {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub uri: String,

    // The content_type uniquely identifies a TrackSource of
    // a Track, i.e. no duplicate content types are allowed
    // among the track sources of each track.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub content_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
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

impl IsValid for TrackSource {
    fn is_valid(&self) -> bool {
        // TODO: Validate the URI
        // Currently (2018-05-28) there is no crate that is able to do this.
        // Crate http/hyper: Fails to recognize absolute file paths with the
        // scheme "file" and without an authority, e.g. parsing fails for
        // "file:///path/to/local/file.txt"
        // Crate url: Doesn't care about reserved characters, e.g. parses
        // "file:///path to local/file.txt" successfully
        !self.uri.is_empty()
            && !self.content_type.is_empty()
            && self.audio_content.iter().all(IsValid::is_valid)
    }
}
