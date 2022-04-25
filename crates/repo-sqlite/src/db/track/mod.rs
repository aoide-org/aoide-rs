// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub(crate) mod models;
pub(crate) mod schema;

use aoide_core::{
    media::Source,
    tag::Tags,
    track::{actor::Actor, cue::Cue, title::Title},
    util::canonical::Canonical,
};

use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive, ToPrimitive)]
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
