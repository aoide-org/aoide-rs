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

use crate::prelude::*;

use aoide_core::{collection::Entity as CollectionEntity, entity::EntityUid};

#[derive(Debug, Clone, Default)]
pub struct RemoteState {
    pub(super) available_collections: RemoteData<Vec<CollectionEntity>>,
}

impl RemoteState {
    pub const fn available_collections(&self) -> &RemoteData<Vec<CollectionEntity>> {
        &self.available_collections
    }

    fn count_available_collections_by_uid(&self, uid: &EntityUid) -> Option<usize> {
        self.available_collections
            .get()
            .map(|v| v.value.iter().filter(|x| &x.hdr.uid == uid).count())
    }

    pub fn find_available_collections_by_uid(&self, uid: &EntityUid) -> Option<&CollectionEntity> {
        debug_assert!(
            self.count_available_collections_by_uid(uid)
                .unwrap_or_default()
                <= 1
        );
        self.available_collections
            .get()
            .and_then(|v| v.value.iter().find(|x| &x.hdr.uid == uid))
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub(super) remote: RemoteState,
    pub(super) active_collection_uid: Option<EntityUid>,
}

impl State {
    pub const fn remote(&self) -> &RemoteState {
        &self.remote
    }

    pub const fn active_collection_uid(&self) -> Option<&EntityUid> {
        self.active_collection_uid.as_ref()
    }

    pub fn active_collection(&self) -> Option<&CollectionEntity> {
        if let (Some(available), Some(active_collection_uid)) = (
            self.remote.available_collections.get(),
            &self.active_collection_uid,
        ) {
            available
                .value
                .iter()
                .find(|x| &x.hdr.uid == active_collection_uid)
        } else {
            None
        }
    }

    pub(super) fn set_available_collections(
        &mut self,
        new_available_collections: Vec<CollectionEntity>,
    ) {
        self.remote.available_collections = RemoteData::ready_now(new_available_collections);
        let active_uid = self.active_collection_uid.take();
        self.set_active_collection_uid(active_uid);
    }

    pub(super) fn set_active_collection_uid(
        &mut self,
        new_active_uid: impl Into<Option<EntityUid>>,
    ) {
        self.active_collection_uid = if let (Some(available), Some(new_active_uid)) = (
            self.remote.available_collections.get(),
            new_active_uid.into(),
        ) {
            if available.value.iter().any(|x| x.hdr.uid == new_active_uid) {
                Some(new_active_uid)
            } else {
                None
            }
        } else {
            None
        };
    }
}
