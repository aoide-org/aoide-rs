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
// TrackCollection
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Validate)]
pub struct TrackCollection {
    #[validate]
    pub uid: EntityUid,

    pub since: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorArgb>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<usize>,
}

impl TrackCollection {
    pub fn filter_slice_by_uid<'a>(
        collections: &'a [TrackCollection],
        collection_uid: &EntityUid,
    ) -> Option<&'a TrackCollection> {
        debug_assert!(
            collections
                .iter()
                .filter(|collection| &collection.uid == collection_uid)
                .count()
                <= 1
        );
        collections
            .iter()
            .filter(|collection| &collection.uid == collection_uid)
            .nth(0)
    }
}
