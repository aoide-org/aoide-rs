// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use serde::{Deserialize, Serialize};

use aoide_storage_sqlite::connection::{
    pool::{create_connection_pool, gatekeeper::Gatekeeper, get_pooled_connection},
    Config as ConnectionConfig,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub connection: ConnectionConfig,
    pub migrate_schema: bool,
}

pub fn commission_database(config: &DatabaseConfig) -> anyhow::Result<Gatekeeper> {
    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    log::info!(
        "Creating SQLite connection pool of max. size {}",
        config.connection.pool.max_size
    );
    let connection_pool =
        create_connection_pool(&config.connection.storage, config.connection.pool.max_size)?;

    log::info!("Initializing database");
    aoide_repo_sqlite::initialize_database(&*get_pooled_connection(&connection_pool)?)?;

    if config.migrate_schema {
        log::info!("Migrating database schema");
        aoide_usecases_sqlite::database::migrate_schema(&*get_pooled_connection(
            &connection_pool,
        )?)?;
    }

    let gatekeeper = Gatekeeper::new(connection_pool, config.connection.pool.gatekeeper);

    Ok(gatekeeper)
}
