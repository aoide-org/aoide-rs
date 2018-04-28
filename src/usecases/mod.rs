// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use std::error;
use std::fmt;

use aoide_core::domain::entity::*;
use aoide_core::domain::collection::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionsError {
    NotFound,
    Unexpected,
}

impl fmt::Display for CollectionsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for CollectionsError {
    fn description(&self) -> &str {
        "TODO: describe CollectionsError"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

pub type CollectionsResult<T> = Result<T, CollectionsError>;

pub trait Collections {
    fn create_entity<S: Into<String>>(&self, name: S) -> CollectionsResult<CollectionEntity>;

    fn update_entity(&self, entity: &mut CollectionEntity) -> CollectionsResult<EntityRevision>;

    fn find_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<CollectionEntity>>;

    fn remove_entity(&self, uid: &EntityUid) -> CollectionsResult<()>;

    fn find_entities_by_name(&self, name: &str) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_starting_with(
        &self,
        name: &str,
    ) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_containing(
        &self,
        name: &str,
    ) -> CollectionsResult<Vec<CollectionEntity>>;

    fn activate_collection(&self, uid: &EntityUid) -> CollectionsResult<()>;
}
