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

pub mod models;
pub mod schema;

use aoide_core::{
    media::Source,
    tag::Tags,
    track::{actor::Actor, cue::Cue, title::Title},
    util::Canonical,
};

use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum Scope {
    Track = 0,
    Album = 1,
}

use aoide_repo::track::RecordHeader;

#[derive(Debug)]
pub struct EntityPreload {
    pub media_source: Source,
    pub track_titles: Canonical<Vec<Title>>,
    pub track_actors: Canonical<Vec<Actor>>,
    pub album_titles: Canonical<Vec<Title>>,
    pub album_actors: Canonical<Vec<Actor>>,
    pub tags: Canonical<Tags>,
    pub cues: Canonical<Vec<Cue>>,
}
