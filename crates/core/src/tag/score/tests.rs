// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn validate() {
    assert!(Score::min().validate().is_ok());
    assert!(Score::max().validate().is_ok());
    assert!(Score::new(Score::min().0 + Score::max().0)
        .validate()
        .is_ok());
    assert!(Score::new(Score::min().0 - Score::max().0)
        .validate()
        .is_err());
    assert!(Score::new(Score::max().0 + Score::max().0)
        .validate()
        .is_err());
}

#[test]
fn display() {
    assert_eq!("0.0%", format!("{}", Score::min()));
    assert_eq!("100.0%", format!("{}", Score::max()));
    assert_eq!("90.1%", format!("{}", Score(0.901_234_5)));
    assert_eq!("90.2%", format!("{}", Score(0.901_5)));
}
