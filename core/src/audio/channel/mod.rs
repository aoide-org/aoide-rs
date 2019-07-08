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

use std::u16;

///////////////////////////////////////////////////////////////////////
// ChannelCount
///////////////////////////////////////////////////////////////////////

type ChannelCountValue = u16;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChannelCount(pub ChannelCountValue);

impl ChannelCount {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn min() -> Self {
        Self(1)
    }

    pub const fn max() -> Self {
        Self(u16::MAX)
    }
}

impl Validate for ChannelCount {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if *self < Self::min() || *self > Self::max() {
            errors.add("number of channels", ValidationError::new("invalid value"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl From<ChannelCountValue> for ChannelCount {
    fn from(from: ChannelCountValue) -> Self {
        Self(from)
    }
}

impl From<ChannelCount> for ChannelCountValue {
    fn from(from: ChannelCount) -> Self {
        from.0
    }
}

///////////////////////////////////////////////////////////////////////
// ChannelLayout
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ChannelLayout {
    Mono,

    DualMono,

    Stereo,
    // ...to be continued
}

impl ChannelLayout {
    pub fn channel_count(self) -> ChannelCount {
        match self {
            ChannelLayout::Mono => ChannelCount(1),
            ChannelLayout::DualMono => ChannelCount(2),
            ChannelLayout::Stereo => ChannelCount(2),
        }
    }

    pub fn channels(self) -> Channels {
        Channels {
            count: self.channel_count(),
            layout: Some(self),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Channels
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "post_validate_channels"))]
pub struct Channels {
    #[validate]
    pub count: ChannelCount,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<ChannelLayout>,
}

impl Channels {
    pub fn count(count: ChannelCount) -> Self {
        Self {
            count,
            layout: None,
        }
    }

    pub fn layout(layout: ChannelLayout) -> Self {
        Self {
            count: layout.channel_count(),
            layout: Some(layout),
        }
    }

    pub fn default_layout(count: ChannelCount) -> Option<ChannelLayout> {
        match count {
            ChannelCount(1) => Some(ChannelLayout::Mono),
            ChannelCount(2) => Some(ChannelLayout::Stereo),
            _ => None,
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn post_validate_channels(channels: &Channels) -> Result<(), ValidationError> {
    if !channels
        .layout
        .iter()
        .all(|layout| layout.channel_count() == channels.count)
    {
        return Err(ValidationError::new(
            "channel layout mismatches channel count",
        ));
    }
    Ok(())
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_channels() {
        let mut channels = Channels {
            count: ChannelCount(1),
            layout: Some(ChannelLayout::DualMono),
        };
        assert!(channels.validate().is_err());
        channels.count = channels.layout.unwrap().channel_count();
        assert!(channels.validate().is_ok());
        channels.layout = None;
        assert!(channels.validate().is_ok());
    }
}
