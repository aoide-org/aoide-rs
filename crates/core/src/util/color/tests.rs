// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn parse() {
    assert_eq!(RgbColor::BLACK, "#000000".parse().unwrap());
    assert_eq!(RgbColor::RED, "#FF0000".parse().unwrap());
    assert_eq!(RgbColor::GREEN, "#00FF00".parse().unwrap());
    assert_eq!(RgbColor::BLUE, "#0000FF".parse().unwrap());
}

#[test]
fn format() {
    assert_eq!("#000000", RgbColor::BLACK.to_string());
    assert_eq!("#FF0000", RgbColor::RED.to_string());
    assert_eq!("#00FF00", RgbColor::GREEN.to_string());
    assert_eq!("#0000FF", RgbColor::BLUE.to_string());
}
