// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_usecases_sqlite::playlist::purge as uc;

use super::*;

pub type ResponseBody = ();

pub fn handle_request(connection: &mut SqliteConnection, uid: &EntityUid) -> Result<ResponseBody> {
    //FIXME: Add transactions after upgrading to diesel v2.0
    //connection.transaction::<_, Error, _>(|connection|
    uc::purge(connection, uid).map_err(Into::into)
    //)
}
