// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

fn main() {
    // Update embedded migrations after the SQL files included by `embed_migrations!()` changed.
    println!("cargo:rerun-if-changed=migrations");
}
