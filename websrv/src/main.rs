// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#![deny(clippy::clone_on_ref_ptr)]
#![warn(rust_2018_idioms)]

use std::{collections::HashMap, env::current_exe, sync::Arc, time::Duration};

use tokio::{join, signal, sync::mpsc, time::sleep};
use warp::{http::StatusCode, Filter};

use aoide_storage_sqlite::{
    create_database_connection_pool, get_pooled_database_connection, initialize_database,
    tokio::{DatabaseConnectionGatekeeper, DatabaseConnectionGatekeeperConfig},
};

use aoide_usecases_sqlite as uc;

use aoide_websrv_api::{handle_rejection, Error};

mod env;
mod routes;

const WEB_SERVER_LISTENING_DELAY: Duration = Duration::from_millis(250);

static OPENAPI_YAML: &str = include_str!("../res/openapi.yaml");

#[cfg(not(feature = "with-webapp"))]
static INDEX_HTML: &str = include_str!("../res/index.html");

const DB_CONNECTION_ACQUIRE_READ_TIMEOUT: Duration = Duration::from_secs(10);

const DB_CONNECTION_ACQUIRE_WRITE_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let started_at = chrono::Utc::now();

    env::init_environment();

    env::init_tracing_and_logging()?;

    if let Ok(exe_path) = current_exe() {
        tracing::info!("Executable: {}", exe_path.display());
    }
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    let endpoint_addr = env::parse_endpoint_addr();
    tracing::info!("Endpoint address: {}", endpoint_addr);

    let database_url = env::parse_database_url();
    tracing::info!("Database URL: {}", database_url);

    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    let database_connection_pool_size = env::parse_database_connection_pool_size();
    let connection_pool =
        create_database_connection_pool(&database_url, database_connection_pool_size)
            .expect("Failed to create database connection pool");

    initialize_database(&*get_pooled_database_connection(&connection_pool)?)
        .expect("Failed to initialize database");
    if env::parse_database_migrate_schema_on_startup() {
        uc::database::migrate_schema(&*get_pooled_database_connection(&connection_pool)?)
            .expect("Failed to migrate database schema");
    }

    let shared_connection_pool = Arc::new(DatabaseConnectionGatekeeper::new(
        connection_pool,
        DatabaseConnectionGatekeeperConfig {
            acquire_read_timeout: DB_CONNECTION_ACQUIRE_READ_TIMEOUT,
            acquire_write_timeout: DB_CONNECTION_ACQUIRE_WRITE_TIMEOUT,
        },
    ));

    tracing::info!("Creating service routes");

    // POST /shutdown
    let (server_shutdown_tx, mut server_shutdown_rx) = mpsc::unbounded_channel::<()>();
    let shutdown_filter = {
        let server_shutdown_tx = server_shutdown_tx.clone();
        warp::post()
            .and(warp::path("shutdown"))
            .and(warp::path::end())
            .map(move || {
                server_shutdown_tx
                    .send(())
                    .map(|()| StatusCode::ACCEPTED)
                    .or_else(|_| {
                        tracing::warn!("Failed to forward shutdown request");
                        Ok(StatusCode::BAD_GATEWAY)
                    })
            })
    };

    // GET /about
    let about_filter = warp::get()
        .and(warp::path("about"))
        .and(warp::path::end())
        .map(move || {
            warp::reply::json(&serde_json::json!({
            "name": env!("CARGO_PKG_NAME"),
            "description": env!("CARGO_PKG_DESCRIPTION"),
            "version": env!("CARGO_PKG_VERSION"),
            "instance": {
                "startedAt": started_at,
                "environment": {
                    "vars": std::env::vars().fold(HashMap::new(), |mut vars, (key, val)| {
                        debug_assert!(!vars.contains_key(&key));
                        vars.insert(key, val);
                        vars}),
                    "currentWorkingDirectory": std::env::current_dir().unwrap_or_default(),
                },
                "networking": {
                    "endpointAddress": endpoint_addr,
                },
                "database": {
                    "url": database_url,
                    "connectionPoolSize": database_connection_pool_size,
                }
            }
            }))
        });

    let api_filters = warp::path("api").and(self::routes::api::create_filters(Arc::clone(
        &shared_connection_pool,
    )));

    // Static content
    let openapi_yaml = warp::path("openapi.yaml").map(|| {
        warp::reply::with_header(
            OPENAPI_YAML,
            "Content-Type",
            "application/x-yaml;charset=utf-8",
        )
    });
    let static_filters = openapi_yaml;

    #[cfg(not(feature = "with-webapp"))]
    let static_filters = static_filters.or(INDEX_HTML);

    let all_filters = api_filters
        .or(static_filters)
        .or(shutdown_filter)
        .or(about_filter);

    #[cfg(feature = "with-webapp")]
    let all_filters = all_filters.or(routes::app::get_index().or(routes::app::get_assets()));

    tracing::info!("Initializing server");

    let server = warp::serve(
        all_filters
            .with(warp::cors().allow_any_origin())
            .recover(handle_rejection),
    );

    tracing::info!("Starting");

    let (socket_addr, server_listener) =
        server.bind_with_graceful_shutdown(endpoint_addr, async move {
            tokio::select! {
                _ = server_shutdown_rx.recv() => {}
                _ = signal::ctrl_c() => {}
            }
            tracing::info!("Stopping");
            shared_connection_pool.decommission();
        });

    let server_listening = async move {
        // Give the server some time to become ready and start listening
        // before announcing the actual endpoint address, i.e. when using
        // an ephemeral port. The delay might need to be tuned depending
        // on how long the startup actually takes. Unfortunately warp does
        // not provide any signal when the server has started listening.
        sleep(WEB_SERVER_LISTENING_DELAY).await;

        // -> stderr
        tracing::info!("Listening on {}", socket_addr);
        // -> stdout
        println!("{}", socket_addr);
    };

    join!(server_listener, server_listening);
    tracing::info!("Stopped");

    Ok(())
}