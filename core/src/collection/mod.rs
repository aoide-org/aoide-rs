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

use ::url::Url;

use crate::{media::SourcePathKind, prelude::*, util::url::is_valid_base_url};

use std::fmt::Debug;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaSourceConfig {
    pub path_kind: SourcePathKind,

    pub root_url: Option<Url>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MediaSourceConfigInvalidity {
    MissingRootUrl,
    InvalidRootUrl,
}

impl Validate for MediaSourceConfig {
    type Invalidity = MediaSourceConfigInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self {
            path_kind,
            root_url,
        } = self;
        let mut context = ValidationContext::new();
        context = if let Some(root_url) = root_url {
            context.invalidate_if(
                !is_valid_base_url(root_url),
                Self::Invalidity::InvalidRootUrl,
            )
        } else {
            let root_url_mandatory = match *path_kind {
                SourcePathKind::VirtualFilePath => true,
                SourcePathKind::Uri | SourcePathKind::Url | SourcePathKind::FileUrl => false,
            };
            context.invalidate_if(root_url_mandatory, Self::Invalidity::MissingRootUrl)
        };
        context.into()
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
