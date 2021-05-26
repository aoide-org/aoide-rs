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

use aoide_core::{collection::Entity as CollectionEntity, entity::EntityUid};
use reqwest::Client;
use url::Url;

#[derive(Debug, Clone, Default)]
pub struct State {
    available: Option<Vec<CollectionEntity>>,
    active_uid: Option<EntityUid>,
}

impl State {
    pub const fn available(&self) -> Option<&Vec<CollectionEntity>> {
        self.available.as_ref()
    }

    fn count_available_by_uid(&self, uid: &EntityUid) -> Option<usize> {
        self.available
            .as_ref()
            .map(|v| v.iter().filter(|x| &x.hdr.uid == uid).count())
    }

    pub fn find_available_by_uid(&self, uid: &EntityUid) -> Option<&CollectionEntity> {
        debug_assert!(self.count_available_by_uid(uid).unwrap_or_default() <= 1);
        self.available
            .as_ref()
            .and_then(|v| v.iter().find(|x| &x.hdr.uid == uid))
    }

    pub const fn active_uid(&self) -> Option<&EntityUid> {
        self.active_uid.as_ref()
    }

    pub fn active(&self) -> Option<&CollectionEntity> {
        if let (Some(available), Some(active_uid)) = (&self.available, &self.active_uid) {
            available.iter().find(|x| &x.hdr.uid == active_uid)
        } else {
            None
        }
    }

    pub fn replace_available(
        &mut self,
        new_available: impl Into<Option<Vec<CollectionEntity>>>,
    ) -> Option<Vec<CollectionEntity>> {
        let old_available = self.available.take();
        self.available = new_available.into();
        let active_uid = self.active_uid.take();
        self.replace_active_uid(active_uid);
        old_available
    }

    pub fn replace_active_uid(
        &mut self,
        new_active_uid: impl Into<Option<EntityUid>>,
    ) -> Option<EntityUid> {
        let old_active_uid = self.active_uid.take();
        self.active_uid = if let (Some(available), Some(new_active_uid)) =
            (&self.available, new_active_uid.into())
        {
            if available.iter().any(|x| x.hdr.uid == new_active_uid) {
                Some(new_active_uid)
            } else {
                None
            }
        } else {
            None
        };
        old_active_uid
    }

    pub fn reset_active_uid(&mut self) -> Option<EntityUid> {
        self.replace_active_uid(None)
    }
}

pub async fn load_available_collections(
    client: &Client,
    base_url: &Url,
) -> anyhow::Result<Vec<CollectionEntity>> {
    let url = base_url.join("c")?;
    let response =
        client.get(url).send().await.map_err(|err| {
            anyhow::Error::from(err).context("Failed to load available collections")
        })?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to load available collections: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload when loading available collections")
    })?;
    let available_collections: Vec<_> = serde_json::from_slice::<
        Vec<aoide_core_serde::collection::Entity>,
    >(&bytes)
    .map(|collections| {
        collections
            .into_iter()
            .map(CollectionEntity::from)
            .collect()
    })
    .map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to deserialize response payload when loading available collections")
    })?;
    log::debug!(
        "Loaded {} available collection(s)",
        available_collections.len()
    );
    Ok(available_collections)
}
