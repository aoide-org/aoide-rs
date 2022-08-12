// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_usecases_sqlite::collection::purge as uc;

use super::*;

pub type ResponseBody = ();

pub fn handle_request(connection: &mut DbConnection, uid: &EntityUid) -> Result<ResponseBody> {
    connection
        .transaction::<_, Error, _>(|connection| uc::purge(connection, uid).map_err(Into::into))
}
