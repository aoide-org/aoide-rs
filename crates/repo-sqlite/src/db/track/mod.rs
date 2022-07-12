// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use aoide_core::{
    media::Source,
    tag::Tags,
    track::{actor::Actor, cue::Cue, title::Title},
    util::canonical::Canonical,
};

use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum Scope {
    Track = 0,
    Album = 1,
}

use aoide_repo::track::RecordHeader;

#[derive(Debug)]
pub(crate) struct EntityPreload {
    pub(crate) media_source: Source,
    pub(crate) track_titles: Canonical<Vec<Title>>,
    pub(crate) track_actors: Canonical<Vec<Actor>>,
    pub(crate) album_titles: Canonical<Vec<Title>>,
    pub(crate) album_actors: Canonical<Vec<Actor>>,
    pub(crate) tags: Canonical<Tags>,
    pub(crate) cues: Canonical<Vec<Cue>>,
}
