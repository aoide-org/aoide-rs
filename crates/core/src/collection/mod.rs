// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::Debug;

use semval::prelude::*;

use crate::{
    EntityHeaderTyped, EntityUidTyped,
    media::content::{ContentPathConfig, ContentPathConfigInvalidity},
    util::color::{Color, ColorInvalidity},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaSourceConfig {
    pub content_path: ContentPathConfig,
}

#[derive(Copy, Clone, Debug)]
pub enum MediaSourceConfigInvalidity {
    ContentPath(ContentPathConfigInvalidity),
}

impl Validate for MediaSourceConfig {
    type Invalidity = MediaSourceConfigInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self { content_path } = self;
        ValidationContext::new()
            .validate_with(content_path, Self::Invalidity::ContentPath)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collection {
    pub title: String,

    /// Semantic type of the collection
    ///
    /// A custom identifier that allows third-party applications
    /// to distinguish different kinds of collections.
    pub kind: Option<String>,

    pub notes: Option<String>,

    pub color: Option<Color>,

    pub media_source_config: MediaSourceConfig,
}

#[derive(Copy, Clone, Debug)]
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
                kind.as_ref().is_some_and(|kind| kind.trim().is_empty()),
                Self::Invalidity::KindEmpty,
            )
            .validate_with(color, Self::Invalidity::Color)
            .validate_with(media_source_config, Self::Invalidity::MediaSourceConfig)
            .into()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EntityType;

pub type EntityUid = EntityUidTyped<EntityType>;

pub type EntityHeader = EntityHeaderTyped<EntityType>;

pub type Entity = crate::entity::Entity<EntityType, Collection, CollectionInvalidity>;
