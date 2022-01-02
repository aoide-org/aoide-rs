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

use crate::prelude::remote::RemoteData;

use aoide_core::{collection::Entity as CollectionEntity, entity::EntityUid};

#[derive(Debug, Default)]
pub struct RemoteView {
    pub available_collections: RemoteData<Vec<CollectionEntity>>,
}

impl RemoteView {
    pub fn is_pending(&self) -> bool {
        self.available_collections.is_pending()
    }

    fn count_available_collections_by_uid(&self, uid: &EntityUid) -> Option<usize> {
        self.available_collections
            .last_value()
            .map(|v| v.iter().filter(|x| &x.hdr.uid == uid).count())
    }

    pub fn find_available_collection_by_uid(&self, uid: &EntityUid) -> Option<&CollectionEntity> {
        debug_assert!(
            self.count_available_collections_by_uid(uid)
                .unwrap_or_default()
                <= 1
        );
        self.available_collections
            .last_value()
            .and_then(|v| v.iter().find(|x| &x.hdr.uid == uid))
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub(super) remote_view: RemoteView,
    pub(super) active_collection_uid: Option<EntityUid>,
}

impl State {
    pub const fn remote_view(&self) -> &RemoteView {
        &self.remote_view
    }

    pub const fn active_collection_uid(&self) -> Option<&EntityUid> {
        self.active_collection_uid.as_ref()
    }

    pub fn active_collection(&self) -> Option<&CollectionEntity> {
        if let (Some(available_collections), Some(active_collection_uid)) = (
            self.remote_view.available_collections.last_value(),
            &self.active_collection_uid,
        ) {
            available_collections
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
        self.remote_view
            .available_collections
            .finish_pending_round_with_value_now(
                self.remote_view.available_collections.round_counter(),
                new_available_collections,
            );
        let active_uid = self.active_collection_uid.take();
        self.set_active_collection_uid(active_uid);
    }

    pub(super) fn set_active_collection_uid(
        &mut self,
        new_active_uid: impl Into<Option<EntityUid>>,
    ) {
        self.active_collection_uid = if let (Some(available_collections), Some(new_active_uid)) = (
            self.remote_view.available_collections.last_value(),
            new_active_uid.into(),
        ) {
            if available_collections
                .iter()
                .any(|x| x.hdr.uid == new_active_uid)
            {
                Some(new_active_uid)
            } else {
                None
            }
        } else {
            None
        };
    }
}
