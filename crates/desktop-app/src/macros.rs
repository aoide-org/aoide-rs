// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

macro_rules! upgrade_or_break {
    ($var:ident) => {
        if let Some($var) = $var.upgrade() {
            $var
        } else {
            break;
        }
    };
}

macro_rules! upgrade_or_return {
    ($var:ident) => {
        if let Some($var) = $var.upgrade() {
            $var
        } else {
            return;
        }
    };
}

macro_rules! upgrade_or_abort {
    ($var:ident) => {
        if let Some($var) = $var.upgrade() {
            $var
        } else {
            return discro::tasklet::OnChanged::Abort;
        }
    };
}
