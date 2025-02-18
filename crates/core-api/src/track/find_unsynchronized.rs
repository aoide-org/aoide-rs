// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{Entity, EntityRevision, media::content::ContentLink, track};

use crate::{filtering::StringPredicate, media::source::ResolveUrlFromContentPath};

#[derive(Debug, Clone, Default)]
pub struct Params {
    pub resolve_url_from_content_path: Option<ResolveUrlFromContentPath>,
    pub content_path_predicate: Option<StringPredicate<'static>>,
}

#[derive(Debug, Clone)]
pub struct UnsynchronizedTrack {
    pub content_link: ContentLink,
    pub last_synchronized_rev: Option<EntityRevision>,
}

pub type UnsynchronizedTrackEntity = Entity<track::EntityType, UnsynchronizedTrack, ()>;
