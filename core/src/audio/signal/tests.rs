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

///////////////////////////////////////////////////////////////////////

#[test]
fn sample_rate_default() {
    assert!(!SampleRateHz::default().is_valid());
}

#[test]
fn sample_rate_is_valid() {
    assert!(SampleRateHz::min().is_valid());
    assert!(SampleRateHz::max().is_valid());
    assert!(SampleRateHz::of_compact_disc().is_valid());
    assert!(SampleRateHz::of_studio_48k().is_valid());
    assert!(SampleRateHz::of_studio_96k().is_valid());
    assert!(SampleRateHz::of_studio_192k().is_valid());
}

#[test]
fn sample_rate_to_string() {
    assert!(SampleRateHz::default()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
    assert!(SampleRateHz::of_compact_disc()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
    assert!(SampleRateHz::of_studio_48k()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
    assert!(SampleRateHz::of_studio_96k()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
    assert!(SampleRateHz::of_studio_192k()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
}

#[test]
fn loudness_to_string() {
    assert!(LoudnessLufs(-1.234)
        .to_string()
        .ends_with(LoudnessLufs::unit_of_measure()));
}
