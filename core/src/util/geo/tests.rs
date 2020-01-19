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

#[test]
fn validate_geo_point() {
    assert!(GeoPoint::from_lat_lon(0.0, 0.0).is_valid());
    assert!(GeoPoint::from_lat_lon(-90.0, -180.0).is_valid());
    assert!(GeoPoint::from_lat_lon(90.0, 180.0).is_valid());
    assert!(GeoPoint::from_lat_lon(-90.0, 180.0).is_valid());
    assert!(!GeoPoint::from_lat_lon(-90.1, 180.0).is_valid());
    assert!(GeoPoint::from_lat_lon(90.0, -180.0).is_valid());
    assert!(!GeoPoint::from_lat_lon(90.1, -180.0).is_valid());
    assert!(GeoPoint::from_lat_lon(90.0, -180.0).is_valid());
    assert!(!GeoPoint::from_lat_lon(90.0, -180.1).is_valid());
    assert!(GeoPoint::from_lat_lon(-90.0, 180.0).is_valid());
    assert!(!GeoPoint::from_lat_lon(-90.0, 180.1).is_valid());
}
