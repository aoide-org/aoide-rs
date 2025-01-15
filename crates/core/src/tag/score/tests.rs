// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn validate() {
    assert!(Score::MIN.validate().is_ok());
    assert!(Score::MAX.validate().is_ok());
    assert!(Score::new_unchecked(Score::MIN.0 + Score::MAX.0)
        .validate()
        .is_ok());
    assert!(Score::new_unchecked(Score::MIN.0 - Score::MAX.0)
        .validate()
        .is_err());
    assert!(Score::new_unchecked(Score::MAX.0 + Score::MAX.0)
        .validate()
        .is_err());
}

#[test]
fn display() {
    assert_eq!("0.0%", Score::MIN.to_string());
    assert_eq!("100.0%", Score::MAX.to_string());
    assert_eq!("90.1%", Score(0.901_234_5).to_string());
    assert_eq!("90.2%", Score(0.901_5).to_string());
}
