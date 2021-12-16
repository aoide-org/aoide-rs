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

use super::*;

#[test]
fn validate_time_sig() {
    assert!(TimeSignature::new(0, Some(0)).validate().is_err());
    assert!(TimeSignature::new(0, Some(1)).validate().is_err());
    assert!(TimeSignature::new(1, Some(0)).validate().is_err());
    assert!(TimeSignature::new(1, None).validate().is_ok());
    assert!(TimeSignature::new(1, Some(1)).validate().is_ok());
    assert!(TimeSignature::new(3, Some(4)).validate().is_ok());
    assert!(TimeSignature::new(4, Some(4)).validate().is_ok());
    assert!(TimeSignature::new(4, Some(3)).validate().is_ok());
}

#[test]
fn measure_position_is_valid_in_measure() {
    assert!(MeasurePosition::from_measure_number_and_beat_offset(2, 0.99).is_valid_in_measure(1));
    assert!(!MeasurePosition::from_measure_number_and_beat_offset(2, 1.0).is_valid_in_measure(1));
    assert!(MeasurePosition::from_measure_number_and_beat_offset(2, 3.99).is_valid_in_measure(4));
    assert!(!MeasurePosition::from_measure_number_and_beat_offset(2, 4.001).is_valid_in_measure(4));
}

#[test]
#[allow(clippy::float_cmp)]
fn measure_position_total_beat_offset() {
    assert_eq!(
        3.0,
        MeasurePosition::from_measure_number_and_beat_offset(2, 0.0).total_beat_offset(3)
    );
    assert_eq!(
        1.0,
        MeasurePosition::from_measure_number_and_beat_offset(1, 1.0).total_beat_offset(3)
    );
    assert_eq!(
        4.0,
        MeasurePosition::from_measure_number_and_beat_offset(2, 1.0).total_beat_offset(3)
    );
    assert_eq!(
        4.75,
        MeasurePosition::from_measure_number_and_beat_offset(2, 1.75).total_beat_offset(3)
    );
    assert_eq!(
        5.125,
        MeasurePosition::from_measure_number_and_beat_offset(2, 2.125).total_beat_offset(3)
    );
    assert_eq!(
        -5.0,
        MeasurePosition::from_measure_number_and_beat_offset(-2, 1.0).total_beat_offset(3)
    );
    assert_eq!(
        -0.5,
        MeasurePosition::from_measure_number_and_beat_offset(-1, 2.5).total_beat_offset(3)
    );
}

#[test]
fn measure_position_from_total_beat_offset() {
    assert_eq!(
        MeasurePosition::from_total_beat_offset(3.0, 4),
        MeasurePosition::from_measure_number_and_beat_offset(1, 3.0)
    );
    assert_eq!(
        MeasurePosition::from_total_beat_offset(4.0, 4),
        MeasurePosition::from_measure_number_and_beat_offset(2, 0.0)
    );
    assert_eq!(
        MeasurePosition::from_total_beat_offset(5.25, 4),
        MeasurePosition::from_measure_number_and_beat_offset(2, 1.25)
    );
    assert_eq!(
        MeasurePosition::from_total_beat_offset(-0.25, 3),
        MeasurePosition::from_measure_number_and_beat_offset(-1, 2.75)
    );
    assert_eq!(
        MeasurePosition::from_total_beat_offset(-4.25, 4),
        MeasurePosition::from_measure_number_and_beat_offset(-2, 3.75)
    );
}

#[test]
fn measure_position_measure_number() {
    assert_eq!(
        2,
        MeasurePosition::from_measure_number_and_beat_offset(2, 0.0).measure_number()
    );
    assert_eq!(
        1,
        MeasurePosition::from_measure_number_and_beat_offset(1, 1.0).measure_number()
    );
    assert_eq!(
        2,
        MeasurePosition::from_measure_number_and_beat_offset(2, 1.0).measure_number()
    );
    assert_eq!(
        2,
        MeasurePosition::from_measure_number_and_beat_offset(2, 1.75).measure_number()
    );
    assert_eq!(
        2,
        MeasurePosition::from_measure_number_and_beat_offset(2, 2.125).measure_number()
    );
    assert_eq!(
        -2,
        MeasurePosition::from_measure_number_and_beat_offset(-2, 1.0).measure_number()
    );
    assert_eq!(
        -1,
        MeasurePosition::from_measure_number_and_beat_offset(-1, 2.5).measure_number()
    );
}

#[test]
fn measure_position_move_by_beats() {
    let beats_per_measure = 3;
    let origin = MeasurePosition::from_measure_number_and_beat_offset(11, 0.0);
    assert_eq!(
        MeasurePosition::from_measure_number_and_beat_offset(
            11,
            origin.beat_offset_in_measure + 1.0
        ),
        origin.move_by_beats(beats_per_measure, 1.0)
    );
    assert_eq!(
        MeasurePosition::from_measure_number_and_beat_offset(
            10,
            BeatOffsetInMeasure::from(beats_per_measure - 1)
        ),
        origin.move_by_beats(beats_per_measure, -1.0)
    );
}
