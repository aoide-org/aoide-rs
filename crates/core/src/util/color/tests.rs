// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn rgb_color_parse() {
    assert_eq!(RgbColor::BLACK, "#000000".parse().unwrap());
    assert_eq!(RgbColor::RED, "#FF0000".parse().unwrap());
    assert_eq!(RgbColor::GREEN, "#00FF00".parse().unwrap());
    assert_eq!(RgbColor::BLUE, "#0000FF".parse().unwrap());
}

#[test]
fn rgb_color_format() {
    assert_eq!("#000000", RgbColor::BLACK.to_string());
    assert_eq!("#FF0000", RgbColor::RED.to_string());
    assert_eq!("#00FF00", RgbColor::GREEN.to_string());
    assert_eq!("#0000FF", RgbColor::BLUE.to_string());
}

#[test]
fn rgb_color_channels() {
    assert_eq!(0x00, RgbColor::BLACK.red());
    assert_eq!(0x00, RgbColor::BLACK.green());
    assert_eq!(0x00, RgbColor::BLACK.blue());

    assert_eq!(0xff, RgbColor::RED.red());
    assert_eq!(0x00, RgbColor::RED.green());
    assert_eq!(0x00, RgbColor::RED.blue());

    assert_eq!(0x00, RgbColor::GREEN.red());
    assert_eq!(0xff, RgbColor::GREEN.green());
    assert_eq!(0x00, RgbColor::GREEN.blue());

    assert_eq!(0x00, RgbColor::BLUE.red());
    assert_eq!(0x00, RgbColor::BLUE.green());
    assert_eq!(0xff, RgbColor::BLUE.blue());

    assert_eq!(0x00, RgbColor::CYAN.red());
    assert_eq!(0xff, RgbColor::CYAN.green());
    assert_eq!(0xff, RgbColor::CYAN.blue());

    assert_eq!(0xff, RgbColor::MAGENTA.red());
    assert_eq!(0x00, RgbColor::MAGENTA.green());
    assert_eq!(0xff, RgbColor::MAGENTA.blue());

    assert_eq!(0xff, RgbColor::YELLOW.red());
    assert_eq!(0xff, RgbColor::YELLOW.green());
    assert_eq!(0x00, RgbColor::YELLOW.blue());

    assert_eq!(0xff, RgbColor::WHITE.red());
    assert_eq!(0xff, RgbColor::WHITE.green());
    assert_eq!(0xff, RgbColor::WHITE.blue());
}
