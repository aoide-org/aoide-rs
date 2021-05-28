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

use crate::{
    media::{is_valid_file_path_base_url, SourcePathKind},
    prelude::*,
};

use std::fmt::Debug;
use url::Url;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaSourceConfig {
    pub path_kind: SourcePathKind,

    pub base_url: Option<Url>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MediaSourceConfigInvalidity {
    BaseUrl,
}

impl Validate for MediaSourceConfig {
    type Invalidity = MediaSourceConfigInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self {
            path_kind,
            base_url,
        } = self;
        ValidationContext::new()
            .invalidate_if(
                match path_kind {
                    SourcePathKind::Uri | SourcePathKind::Url | SourcePathKind::FileUrl => {
                        base_url.is_some()
                    }
                    SourcePathKind::VirtualFilePath => {
                        if let Some(base_url) = base_url {
                            !is_valid_file_path_base_url(base_url)
                        } else {
                            false
                        }
                    }
                },
                Self::Invalidity::BaseUrl,
            )
            .into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Collection {
    pub title: String,

    pub kind: Option<String>,

    pub notes: Option<String>,

    pub color: Option<Color>,

    pub media_source_config: MediaSourceConfig,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CollectionInvalidity {
    TitleEmpty,
    KindEmpty,
    Color(ColorInvalidity),
    MediaSourceConfig(MediaSourceConfigInvalidity),
}

impl Validate for Collection {
    type Invalidity = CollectionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self {
            title,
            kind,
            color,
            media_source_config,
            notes: _,
        } = self;
        ValidationContext::new()
            .invalidate_if(title.trim().is_empty(), Self::Invalidity::TitleEmpty)
            .invalidate_if(
                kind.as_ref()
                    .map(|kind| kind.trim().is_empty())
                    .unwrap_or(false),
                Self::Invalidity::KindEmpty,
            )
            .validate_with(color, Self::Invalidity::Color)
            .validate_with(media_source_config, Self::Invalidity::MediaSourceConfig)
            .into()
    }
}

pub type Entity = crate::entity::Entity<CollectionInvalidity, Collection>;
