// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////

use super::*;

use crate::{
    entity::{EntityUid, EntityUidInvalidity},
    util::{clock::TickInstant, color::Color},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// References the track
    pub uid: EntityUid,

    pub color: Option<Color>,

    pub play_count: Option<usize>,

    pub last_played_at: Option<TickInstant>,
}

#[derive(Copy, Clone, Debug)]
pub enum ItemInvalidity {
    Uid(EntityUidInvalidity),
}

impl Validate for Item {
    type Invalidity = ItemInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.uid, Self::Invalidity::Uid)
            .into()
    }
}
