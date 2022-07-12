// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use crate::{prelude::*, track::EntityUid as TrackUid};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// A reference to the track.
    pub uid: TrackUid,
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
