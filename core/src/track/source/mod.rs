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

use crate::audio::{AudioContent, AudioContentValidation};

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Source {
    pub uri: String,

    // The content_type uniquely identifies a Source of
    // a Track, i.e. no duplicate content types are allowed
    // among the track sources of each track.
    pub content_type: String,

    pub audio_content: Option<AudioContent>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SourceValidation {
    UriMinLen(usize),
    ContentTypeMinLen(usize),
    AudioContent(AudioContentValidation),
}

const URI_MIN_LEN: usize = 1;

const CONTENT_TYPE_MIN_LEN: usize = 1;

impl Validate for Source {
    type Validation = SourceValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(
            self.uri.len() < URI_MIN_LEN,
            SourceValidation::UriMinLen(URI_MIN_LEN),
        );
        context.add_violation_if(
            self.content_type.len() < CONTENT_TYPE_MIN_LEN,
            SourceValidation::ContentTypeMinLen(CONTENT_TYPE_MIN_LEN),
        );
        // TODO: Validate MIME type
        if let Some(ref audio_content) = self.audio_content {
            context.map_and_merge_result(audio_content.validate(), SourceValidation::AudioContent);
        }
        context.into_result()
    }
}

#[derive(Debug)]
pub struct Sources;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SourcesValidation {
    Source(SourceValidation),
    ContentTypeAmbiguous,
}

impl Sources {
    pub fn validate<'a, I>(sources: I) -> ValidationResult<SourcesValidation>
    where
        I: IntoIterator<Item = &'a Source> + Copy,
    {
        let mut context = ValidationContext::default();
        for source in sources.into_iter() {
            context.map_and_merge_result(source.validate(), SourcesValidation::Source);
        }
        if !context.has_violations() {
            let mut content_types: Vec<_> = sources
                .into_iter()
                .map(|source| &source.content_type)
                .collect();
            content_types.sort_unstable();
            content_types.dedup();
            context.add_violation_if(
                content_types.len() < sources.into_iter().count(),
                SourcesValidation::ContentTypeAmbiguous,
            );
        }
        context.into_result()
    }

    pub fn filter_content_type<'a, 'b, I>(
        sources: I,
        content_type: &'b str,
    ) -> impl Iterator<Item = &'a Source>
    where
        I: IntoIterator<Item = &'a Source>,
        'b: 'a,
    {
        sources
            .into_iter()
            .filter(move |source| source.content_type == content_type)
    }
}
