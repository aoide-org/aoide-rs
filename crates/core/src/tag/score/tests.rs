// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn validate() {
    assert!(Score::MIN.validate().is_ok());
    assert!(Score::MAX.validate().is_ok());
    assert!(Score::new(Score::MIN.0 + Score::MAX.0).validate().is_ok());
    assert!(Score::new(Score::MIN.0 - Score::MAX.0).validate().is_err());
    assert!(Score::new(Score::MAX.0 + Score::MAX.0).validate().is_err());
}

#[test]
fn display() {
    assert_eq!("0.0%", format!("{}", Score::MIN));
    assert_eq!("100.0%", format!("{}", Score::MAX));
    assert_eq!("90.1%", format!("{}", Score(0.901_234_5)));
    assert_eq!("90.2%", format!("{}", Score(0.901_5)));
}
