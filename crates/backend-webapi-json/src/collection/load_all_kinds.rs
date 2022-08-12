// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_usecases_sqlite::collection::load as uc;

use super::*;

pub type ResponseBody = Vec<String>;

pub fn handle_request(connection: &mut DbConnection) -> Result<ResponseBody> {
    connection
        .transaction::<_, Error, _>(|connection| uc::load_all_kinds(connection).map_err(Into::into))
}
