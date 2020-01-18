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

use super::*;

use crate::{
    entity::{EntityUid, EntityUidInvalidity},
    util::{color::*, clock::TickInstant},
};

///////////////////////////////////////////////////////////////////////
// Collection
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Collection {
    pub uid: EntityUid,

    pub since: TickInstant,

    pub comment: Option<String>,

    pub color: Option<ColorRgb>,

    pub play_count: Option<usize>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CollectionInvalidity {
    Uid(EntityUidInvalidity),
}

impl Validate for Collection {
    type Invalidity = CollectionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.uid, CollectionInvalidity::Uid)
            .into()
    }
}

#[derive(Debug)]
pub struct Collections;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CollectionsInvalidity {
    Collection(CollectionInvalidity),
    NonUniqueUid,
}

impl Collections {
    pub fn validate<'a, I>(collections: I) -> ValidationResult<CollectionsInvalidity>
    where
        I: Iterator<Item = &'a Collection> + Clone,
    {
        let mut uids: Vec<_> = collections.clone().map(|c| &c.uid).collect();
        uids.sort_unstable();
        uids.dedup();
        collections
            .clone()
            .fold(ValidationContext::new(), |context, collection| {
                context.validate_with(collection, CollectionsInvalidity::Collection)
            })
            .invalidate_if(
                uids.len() < collections.count(),
                CollectionsInvalidity::NonUniqueUid,
            )
            .into()
    }

    pub fn find_by_uid<'a, I>(collections: I, uid: &EntityUid) -> Option<&'a Collection>
    where
        I: Iterator<Item = &'a Collection> + Clone,
    {
        debug_assert!(Self::validate(collections.clone()).is_ok());
        collections.filter(|c| &c.uid == uid).nth(0)
    }
}
