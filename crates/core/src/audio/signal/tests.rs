// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    assert!(SampleRateHz::default().validate().is_err());
}

#[test]
fn validate_sample_rate() {
    assert!(SampleRateHz::min().validate().is_ok());
    assert!(SampleRateHz::max().validate().is_ok());
    assert!(SampleRateHz::of_compact_disc().validate().is_ok());
    assert!(SampleRateHz::of_studio_48k().validate().is_ok());
    assert!(SampleRateHz::of_studio_96k().validate().is_ok());
    assert!(SampleRateHz::of_studio_192k().validate().is_ok());
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
