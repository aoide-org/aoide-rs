// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////
// GeoCoord
///////////////////////////////////////////////////////////////////////

pub type GeoCoord = f64;

///////////////////////////////////////////////////////////////////////
// GeoPoint
///////////////////////////////////////////////////////////////////////

/// A flat WGS 84 point without height/elevation on the surface of the
/// earth.
///
/// Both latitude and longitude are measured and stored in degrees to
/// prevent rounding errors by the conversion from/to radians.
#[derive(Clone, Debug, PartialEq)]
pub struct GeoPoint {
    /// Latitude in degrees
    pub lat: GeoCoord,

    /// Longitude in degrees
    pub lon: GeoCoord,
}

impl GeoPoint {
    pub const fn lat_min() -> GeoCoord {
        -90.0
    }

    pub const fn lat_max() -> GeoCoord {
        90.0
    }

    pub fn lon_min() -> GeoCoord {
        -180.0
    }

    pub const fn lon_max() -> GeoCoord {
        180.0
    }

    pub fn from_lat_lon(lat: GeoCoord, lon: GeoCoord) -> Self {
        Self { lat, lon }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GeoPointInvalidity {
    LatitudeOutOfRange,
    LongitudeOutOfRange,
}

impl Validate for GeoPoint {
    type Invalidity = GeoPointInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.lat < Self::lat_min(),
                GeoPointInvalidity::LatitudeOutOfRange,
            )
            .invalidate_if(
                self.lat > Self::lat_max(),
                GeoPointInvalidity::LatitudeOutOfRange,
            )
            .invalidate_if(
                self.lon < Self::lon_min(),
                GeoPointInvalidity::LongitudeOutOfRange,
            )
            .invalidate_if(
                self.lon > Self::lon_max(),
                GeoPointInvalidity::LongitudeOutOfRange,
            )
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
