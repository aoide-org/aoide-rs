// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_json::track::Entity;

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::track::load::*;
}

pub type ResponseBody = Entity;

pub fn handle_request(connection: &mut SqliteConnection, uid: &EntityUid) -> Result<ResponseBody> {
    //FIXME: Add transactions after upgrading to diesel v2.0
    //connection.transaction::<_, Error, _>(|connection|
    uc::load_one(connection, uid).map_err(Into::into)
    //)
        .map(Into::into)
}
