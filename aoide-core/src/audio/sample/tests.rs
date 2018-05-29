// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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
fn sample_rate_default() {
    assert_eq!(SampleRate::default().hz, 0);
}

#[test]
fn sample_rate_is_valid() {
    assert!(!SampleRate::default().is_valid());
    assert!(SampleRate::MAX.is_valid());
    assert!(SampleRate::COMPACT_DISC.is_valid());
    assert!(SampleRate::STUDIO_48KHZ.is_valid());
    assert!(SampleRate::STUDIO_96KHZ.is_valid());
    assert!(SampleRate::STUDIO_192KHZ.is_valid());
}

#[test]
fn sample_rate_to_string() {
    assert_eq!("44100 Hz", SampleRate::COMPACT_DISC.to_string());
    assert_eq!("48000 Hz", SampleRate::STUDIO_48KHZ.to_string());
    assert_eq!("96000 Hz", SampleRate::STUDIO_96KHZ.to_string());
    assert_eq!("192000 Hz", SampleRate::STUDIO_192KHZ.to_string());
}
