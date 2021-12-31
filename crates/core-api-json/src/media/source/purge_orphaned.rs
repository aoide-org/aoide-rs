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

use url::Url;

#[cfg(feature = "backend")]
use aoide_core::util::url::{BaseUrl, BaseUrlError};

use crate::prelude::*;

mod _inner {
    pub use aoide_core_api::media::source::purge_orphaned::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,
}

#[cfg(feature = "frontend")]
impl From<_inner::Params> for Params {
    fn from(from: _inner::Params) -> Self {
        let _inner::Params { root_url } = from;
        Self {
            root_url: root_url.map(Into::into),
        }
    }
}

#[cfg(feature = "backend")]
impl TryFrom<Params> for _inner::Params {
    type Error = BaseUrlError;

    fn try_from(from: Params) -> Result<Self, Self::Error> {
        let Params { root_url } = from;
        let root_url = root_url.map(BaseUrl::try_autocomplete_from).transpose()?;
        Ok(Self { root_url })
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub root_url: Option<Url>,
    pub purged: u64,
}

#[cfg(feature = "frontend")]
impl TryFrom<Outcome> for _inner::Outcome {
    type Error = aoide_core::util::url::BaseUrlError;

    fn try_from(from: Outcome) -> Result<Self, Self::Error> {
        let Outcome { root_url, purged } = from;
        let root_url = root_url.map(TryInto::try_into).transpose()?;
        Ok(Self {
            root_url,
            purged: purged as usize,
        })
    }
}

#[cfg(feature = "backend")]
impl From<_inner::Outcome> for Outcome {
    fn from(from: _inner::Outcome) -> Self {
        let _inner::Outcome { root_url, purged } = from;
        Self {
            root_url: root_url.map(Into::into),
            purged: purged as u64,
        }
    }
}
