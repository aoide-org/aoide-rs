// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

use crate::{entity::EntityUid, util::color::*};

use chrono::{DateTime, Utc};

///////////////////////////////////////////////////////////////////////
// Collection
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Collection {
    pub uid: EntityUid,

    pub since: DateTime<Utc>,

    pub color: Option<ColorArgb>,

    pub play_count: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectionValidation {
    Uid,
}

impl Validate<CollectionValidation> for Collection {
    fn validate(&self) -> ValidationResult<CollectionValidation> {
        let mut errors = ValidationErrors::default();
        errors.map_and_merge_result(self.uid.validate(), |()| CollectionValidation::Uid);
        errors.into_result()
    }
}

#[derive(Debug)]
pub struct Collections;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectionsValidation {
    Collection(CollectionValidation),
}

impl Collections {
    pub fn validate<'a, I>(collections: I) -> ValidationResult<CollectionsValidation>
    where
        I: IntoIterator<Item = &'a Collection> + Copy,
    {
        let mut errors = ValidationErrors::default();
        for collection in collections.into_iter() {
            errors.map_and_merge_result(collection.validate(), CollectionsValidation::Collection);
        }
        errors.into_result()
    }
}
