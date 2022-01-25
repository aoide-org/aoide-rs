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

macro_rules! record_id_newtype {
    ($type_name:ident) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
        pub struct $type_name($crate::RecordId);

        impl $type_name {
            #[must_use]
            pub const fn new(inner: $crate::RecordId) -> Self {
                Self(inner)
            }
            #[must_use]
            pub const fn to_inner(self) -> $crate::RecordId {
                let Self(inner) = self;
                inner
            }
        }

        impl From<$crate::RecordId> for $type_name {
            fn from(from: $crate::RecordId) -> Self {
                Self::new(from)
            }
        }

        impl From<$type_name> for $crate::RecordId {
            fn from(from: $type_name) -> Self {
                from.to_inner()
            }
        }
    };
}

macro_rules! entity_repo_trait_common_functions {
    ($record_id_type:ident, $entity_type:ident, $entity_type_name:ident) => {
        paste::paste! {
            fn [<resolve_ $entity_type_name:lower _id>](&self, uid: &aoide_core::entity::EntityUid) -> $crate::prelude::RepoResult<$record_id_type> {
                self.[<resolve_ $entity_type_name:lower _entity_revision>](uid)
                    .map(|(hdr, _rev)| hdr.id)
            }

            fn [<resolve_ $entity_type_name:lower _entity_revision>](
                &self,
                uid: &aoide_core::entity::EntityUid,
            ) -> $crate::prelude::RepoResult<(crate::RecordHeader<$record_id_type>, aoide_core::entity::EntityRevision)>;

            fn [<touch_ $entity_type_name:lower _entity_revision>](
                &self,
                entity_header: &aoide_core::entity::EntityHeader,
                updated_at: aoide_core::util::clock::DateTime,
            ) -> $crate::prelude::RepoResult<(crate::RecordHeader<$record_id_type>, aoide_core::entity::EntityRevision)>;

            fn [<update_ $entity_type_name:lower _entity_revision>](
                &self,
                updated_at: aoide_core::util::clock::DateTime,
                updated_entity: &$entity_type,
            ) -> $crate::prelude::RepoResult<()> {
                let (id, rev) =
                    self.[<resolve_ $entity_type_name:lower _entity_revision>](&updated_entity.hdr.uid).map(|(hdr, rev)| (hdr.id, rev))?;
                if updated_entity.hdr.rev.prev() != Some(rev) {
                    return Err($crate::prelude::RepoError::Conflict);
                }
                self.[<update_ $entity_type_name:lower _entity>](id, updated_at, updated_entity)
            }

            fn [<update_ $entity_type_name:lower _entity>](
                &self,
                id: $record_id_type,
                updated_at: aoide_core::util::clock::DateTime,
                updated_entity: &$entity_type,
            ) -> $crate::prelude::RepoResult<()>;

            fn [<load_ $entity_type_name:lower _entity>](&self, id: $record_id_type) -> $crate::prelude::RepoResult<(crate::RecordHeader<$record_id_type>, $entity_type)>;

            /// Purge the entity
            ///
            /// Purging is supposed to be recursive and affects all relationships,
            /// i.e. all records that belong to this entity must be deleted. This
            /// could either be implemented implicitly using ON DELETE CASCADE
            /// constraints for foreign key (FK) relationships in an SQL database
            /// or programmatically.
            fn [<purge_ $entity_type_name:lower _entity>](&self, id: $record_id_type) -> $crate::prelude::RepoResult<()>;
        }
    }
}
