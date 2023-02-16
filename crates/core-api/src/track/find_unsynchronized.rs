// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    entity::{Entity, EntityRevision},
    media::content::ContentLink,
    track,
    util::url::BaseUrl,
};

use crate::filtering::StringPredicate;

#[derive(Debug, Clone, Default)]
pub struct Params {
    pub vfs_content_path_root_url: Option<BaseUrl>,
    pub content_path_predicate: Option<StringPredicate<'static>>,
}

#[derive(Debug, Clone)]
pub struct UnsynchronizedTrack {
    pub content_link: ContentLink,
    pub last_synchronized_rev: Option<EntityRevision>,
}

pub type UnsynchronizedTrackEntity = Entity<track::EntityType, UnsynchronizedTrack, ()>;
