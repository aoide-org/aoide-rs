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
fn score_position_is_valid_in_measure() {
    assert!(ScorePosition {
        measure_offset: 2,
        beat_offset: 0.99
    }.is_valid_in_measure(1));
    assert!(!ScorePosition {
        measure_offset: 2,
        beat_offset: 1.0
    }.is_valid_in_measure(1));
    assert!(ScorePosition {
        measure_offset: 2,
        beat_offset: 3.999
    }.is_valid_in_measure(4));
    assert!(!ScorePosition {
        measure_offset: 2,
        beat_offset: 4.001
    }.is_valid_in_measure(4));
}

#[test]
fn score_position_total_beat_offset() {
    let beats_per_measure = 3;
    assert_eq!(
        6.0,
        ScorePosition {
            measure_offset: 2,
            beat_offset: 0.0
        }
        .total_beat_offset(beats_per_measure)
    );
    assert_eq!(
        6.0,
        ScorePosition {
            measure_offset: 2,
            beat_offset: 0.0
        }
        .total_beat_offset_with_incomplete_first_measure(beats_per_measure, 0.0)
    );
    assert_eq!(
        5.0,
        ScorePosition {
            measure_offset: 2,
            beat_offset: 0.0
        }
        .total_beat_offset_with_incomplete_first_measure(beats_per_measure, 1.0)
    );
    assert_eq!(
        4.0,
        ScorePosition {
            measure_offset: 2,
            beat_offset: 0.0
        }
        .total_beat_offset_with_incomplete_first_measure(beats_per_measure, 2.0)
    );
    assert_eq!(
        8.0,
        ScorePosition {
            measure_offset: 2,
            beat_offset: 2.0
        }
        .total_beat_offset(beats_per_measure)
    );
    assert_eq!(
        8.0,
        ScorePosition {
            measure_offset: 2,
            beat_offset: 2.0
        }
        .total_beat_offset_with_incomplete_first_measure(beats_per_measure, 0.0)
    );
    assert_eq!(
        7.0,
        ScorePosition {
            measure_offset: 2,
            beat_offset: 2.0
        }
        .total_beat_offset_with_incomplete_first_measure(beats_per_measure, 1.0)
    );
    assert_eq!(
        6.0,
        ScorePosition {
            measure_offset: 2,
            beat_offset: 2.0
        }
        .total_beat_offset_with_incomplete_first_measure(beats_per_measure, 2.0)
    );
}

#[test]
fn score_position_move_by_beats() {
    let beats_per_measure = 3;
    let origin = ScorePosition {
        measure_offset: 11,
        beat_offset: 0.0,
    };
    assert_eq!(
        ScorePosition {
            measure_offset: 11,
            beat_offset: origin.beat_offset + 1.0
        },
        origin.move_by_beats(beats_per_measure, 1.0)
    );
    assert_eq!(
        ScorePosition {
            measure_offset: 10,
            beat_offset: BeatDelta::from(beats_per_measure) - 1.0
        },
        origin.move_by_beats(beats_per_measure, -1.0)
    );
}
