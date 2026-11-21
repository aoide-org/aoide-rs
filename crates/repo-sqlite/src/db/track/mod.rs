// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use nonicle::Canonical;

use aoide_core::{
    media::Source,
    music::key::KeyCode,
    tag::Tags,
    track::{AdvisoryRating, actor::Actor, album::Kind, cue::Cue, title::Title},
};
use aoide_core_api::track::search::Scope;
use aoide_repo::RepoResult;

use crate::util::decode_repr;

pub(crate) mod models;
pub(crate) mod schema;

#[derive(Debug)]
pub(crate) struct EntityPreload {
    pub(crate) media_source: Source,
    pub(crate) track_titles: Canonical<Vec<Title>>,
    pub(crate) track_actors: Canonical<Vec<Actor>>,
    pub(crate) album_titles: Canonical<Vec<Title>>,
    pub(crate) album_actors: Canonical<Vec<Actor>>,
    pub(crate) tags: Canonical<Tags<'static>>,
    pub(crate) cues: Canonical<Vec<Cue>>,
}

pub(crate) const fn encode_album_kind(value: Kind) -> i16 {
    value as _
}

pub(crate) fn decode_album_kind(value: i16) -> RepoResult<Kind> {
    decode_repr(value, Kind::from_repr, "track album Kind")
}

pub(crate) const fn encode_advisory_rating(value: AdvisoryRating) -> i16 {
    value as _
}

pub(crate) fn decode_advisory_rating(value: i16) -> RepoResult<AdvisoryRating> {
    decode_repr(value, AdvisoryRating::from_repr, "track AdvisoryRating")
}

pub(crate) const fn encode_search_scope(value: Scope) -> i16 {
    value as _
}

pub(crate) fn decode_search_scope(value: i16) -> RepoResult<Scope> {
    decode_repr(value, Scope::from_repr, "track search Scope")
}

pub(crate) fn encode_music_key_code(value: KeyCode) -> i16 {
    i16::from(value.to_value())
}

pub(crate) fn decode_music_key_code(value: i16) -> RepoResult<KeyCode> {
    decode_repr(value, KeyCode::try_from_value, "musical KeyCode")
}
