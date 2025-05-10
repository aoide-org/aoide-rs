// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

macro_rules! record_id_newtype {
    ($type_name:ident) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    ($record_id_type:ident, $entity_type:ident, $entity_uid_type:ident, $entity_header_type:ident, $entity_type_name:ident) => {
        paste::paste! {
            fn [<resolve_ $entity_type_name:lower _id>](&mut self, uid: &$entity_uid_type) -> $crate::RepoResult<$record_id_type> {
                self.[<resolve_ $entity_type_name:lower _entity_revision>](uid)
                    .map(|(hdr, _rev)| hdr.id)
            }

            fn [<resolve_ $entity_type_name:lower _entity_revision>](
                &mut self,
                uid: &$entity_uid_type,
            ) -> $crate::RepoResult<(crate::RecordHeader<$record_id_type>, aoide_core::EntityRevision)>;

            fn [<touch_ $entity_type_name:lower _entity_revision>](
                &mut self,
                entity_header: &$entity_header_type,
                updated_at: aoide_core::util::clock::UtcDateTimeMs,
            ) -> $crate::RepoResult<(crate::RecordHeader<$record_id_type>, aoide_core::EntityRevision)>;

            fn [<update_ $entity_type_name:lower _entity_revision>](
                &mut self,
                updated_at: aoide_core::util::clock::UtcDateTimeMs,
                updated_entity: &$entity_type,
            ) -> $crate::RepoResult<()> {
                let (id, rev) =
                    self.[<resolve_ $entity_type_name:lower _entity_revision>](&updated_entity.hdr.uid).map(|(hdr, rev)| (hdr.id, rev))?;
                if updated_entity.hdr.rev.prev() != Some(rev) {
                    return Err($crate::RepoError::Conflict);
                }
                self.[<update_ $entity_type_name:lower _entity>](id, updated_at, updated_entity)
            }

            fn [<update_ $entity_type_name:lower _entity>](
                &mut self,
                id: $record_id_type,
                updated_at: aoide_core::util::clock::UtcDateTimeMs,
                updated_entity: &$entity_type,
            ) -> $crate::RepoResult<()>;

            fn [<load_ $entity_type_name:lower _entity>](&mut self, id: $record_id_type) -> $crate::RepoResult<(crate::RecordHeader<$record_id_type>, $entity_type)>;

            /// Purge the entity
            ///
            /// Purging is supposed to be recursive and affects all relationships,
            /// i.e. all records that belong to this entity must be deleted. This
            /// could either be implemented implicitly using ON DELETE CASCADE
            /// constraints for foreign key (FK) relationships in an SQL database
            /// or programmatically.
            fn [<purge_ $entity_type_name:lower _entity>](&mut self, id: $record_id_type) -> $crate::RepoResult<()>;
        }
    }
}
