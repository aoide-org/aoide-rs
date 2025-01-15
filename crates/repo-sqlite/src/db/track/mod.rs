// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;
use nonicle::Canonical;

use aoide_core::{
    media::Source,
    music::key::KeyCode,
    tag::Tags,
    track::{actor::Actor, album::Kind, cue::Cue, title::Title, AdvisoryRating},
};
use aoide_core_api::track::search::Scope;
use aoide_repo::{RepoError, RepoResult};

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
    u8::try_from(value)
        .ok()
        .and_then(Kind::from_repr)
        .ok_or_else(|| RepoError::Other(anyhow!("invalid track album Kind value: {value}")))
}

pub(crate) const fn encode_advisory_rating(value: AdvisoryRating) -> i16 {
    value as _
}

pub(crate) fn decode_advisory_rating(value: i16) -> RepoResult<AdvisoryRating> {
    u8::try_from(value)
        .ok()
        .and_then(AdvisoryRating::from_repr)
        .ok_or_else(|| RepoError::Other(anyhow!("invalid track AdvisoryRating value: {value}")))
}

pub(crate) const fn encode_search_scope(value: Scope) -> i16 {
    value as _
}

pub(crate) fn decode_search_scope(value: i16) -> RepoResult<Scope> {
    u8::try_from(value)
        .ok()
        .and_then(Scope::from_repr)
        .ok_or_else(|| RepoError::Other(anyhow!("invalid track search Scope value: {value}")))
}

pub(crate) fn encode_music_key_code(value: KeyCode) -> i16 {
    i16::from(value.to_value())
}

pub(crate) fn decode_music_key_code(value: i16) -> RepoResult<KeyCode> {
    value
        .try_into()
        .ok()
        .and_then(KeyCode::try_from_value)
        .ok_or_else(|| RepoError::Other(anyhow!("invalid musical KeyCode value: {value}")))
}
