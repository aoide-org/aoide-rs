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

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive as _, ToPrimitive as _};

use tokio::{join, sync::mpsc, time::sleep};
use warp::{http::StatusCode, Filter};

use aoide_storage_sqlite::connection::{
    create_connection_pool,
    gatekeeper::{DatabaseConnectionGatekeeper, DatabaseConnectionGatekeeperConfig},
    get_pooled_connection,
};

use aoide_repo_sqlite::initialize_database;

use aoide_usecases_sqlite as uc;

use aoide_websrv_api::handle_rejection;

use super::{
    config::{Config, DatabaseConnection},
    routing,
};

const WEB_SERVER_LISTENING_DELAY: Duration = Duration::from_millis(250);

static OPENAPI_YAML: &str = include_str!("../res/openapi.yaml");

#[cfg(not(feature = "with-webapp"))]
static INDEX_HTML: &str = include_str!("../res/index.html");

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum State {
    Launching = 0,
    Starting = 1,
    Listening = 2,
    Stopping = 3,
    Terminating = 4,
}

#[derive(Debug)]
pub struct CurrentState {
    inner: AtomicU8,
}

impl CurrentState {
    pub fn new(initial_state: State) -> Self {
        let initial_state = initial_state.to_u8().unwrap();
        Self {
            inner: AtomicU8::new(initial_state),
        }
    }

    #[must_use]
    pub fn load(&self) -> State {
        State::from_u8(self.inner.load(Ordering::Relaxed)).unwrap()
    }

    fn store(&self, state: State) {
        self.inner.store(state.to_u8().unwrap(), Ordering::Relaxed);
    }

    fn switch(&self, from_state: State, to_state: State) -> Result<State, State> {
        let old = from_state.to_u8().unwrap();
        let new = to_state.to_u8().unwrap();
        self.inner
            .compare_exchange_weak(old, new, Ordering::Relaxed, Ordering::Relaxed)
            .map(|val| State::from_u8(val).unwrap())
            .map_err(|val| State::from_u8(val).unwrap())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Terminate { abort_pending_tasks: bool },
}

pub async fn run(
    config: Config,
    command_rx: mpsc::UnboundedReceiver<Command>,
    current_state: Arc<CurrentState>,
) -> anyhow::Result<()> {
    let launched_at = chrono::Utc::now();

    log::info!("Launching");
    current_state.store(State::Launching);

    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    log::info!(
        "Creating SQLite connection pool of size {}",
        config.database.connection_pool_size
    );
    let sqlite_database_connection = match &config.database.connection {
        DatabaseConnection::Sqlite(sqlite_connection) => sqlite_connection.as_ref(),
    };
    let connection_pool = create_connection_pool(
        sqlite_database_connection,
        config.database.connection_pool_size.into(),
    )
    .expect("Failed to create database connection pool");

    log::info!("Initializing database");
    initialize_database(&*get_pooled_connection(&connection_pool)?)
        .expect("Failed to initialize database");
    if config.database.migrate_schema_on_startup {
        log::info!("Migrating database schema");
        uc::database::migrate_schema(&*get_pooled_connection(&connection_pool)?)
            .expect("Failed to migrate database schema");
    }

    let shared_connection_pool = Arc::new(DatabaseConnectionGatekeeper::new(
        connection_pool,
        DatabaseConnectionGatekeeperConfig {
            acquire_read_timeout: Duration::from_millis(
                config.database.connection_timeout.acquire_read_millis.get(),
            ),
            acquire_write_timeout: Duration::from_millis(
                config
                    .database
                    .connection_timeout
                    .acquire_write_millis
                    .get(),
            ),
        },
    ));

    let about_json = serde_json::json!({
    "name": env!("CARGO_PKG_NAME"),
    "description": env!("CARGO_PKG_DESCRIPTION"),
    "version": env!("CARGO_PKG_VERSION"),
    "instance": {
        "launched_at": launched_at,
        "config": config,
        "environment": {
            "current_dir": std::env::current_dir().unwrap_or_default(),
            "vars": std::env::vars().fold(HashMap::new(), |mut vars, (key, val)| {
                debug_assert!(!vars.contains_key(&key));
                vars.insert(key, val);
                vars}),
        },
    }
    });

    log::info!("Creating service routes");

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
                        log::warn!("Failed to forward shutdown request");
                        Ok(StatusCode::BAD_GATEWAY)
                    })
            })
    };

    // GET /about
    let about_filter = warp::get()
        .and(warp::path("about"))
        .and(warp::path::end())
        .map(move || warp::reply::json(&about_json));

    let api_filters = warp::path("api").and(self::routing::api::create_filters(Arc::clone(
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
    let all_filters = all_filters.or(routing::app::get_index().or(routing::app::get_assets()));

    log::info!("Initializing server");

    let server = warp::serve(
        all_filters
            .with(warp::cors().allow_any_origin())
            .recover(handle_rejection),
    );

    log::info!("Starting");
    current_state.store(State::Starting);

    let (socket_addr, server_listener) = {
        let current_state = Arc::clone(&current_state);
        let mut command_rx = command_rx;
        server.bind_with_graceful_shutdown(config.endpoint.socket_addr(), async move {
            let mut abort_pending_tasks_on_termination = false;
            loop {
                tokio::select! {
                    Some(()) = server_shutdown_rx.recv() => break,
                    Some(command) = command_rx.recv() => {
                        match command {
                            Command::Terminate {
                                abort_pending_tasks,
                            } => {
                                abort_pending_tasks_on_termination = abort_pending_tasks;
                                break;
                            }
                        }
                    }
                    else => break,
                }
            }

            log::info!("Stopping");
            if let Err(state) = current_state.switch(State::Listening, State::Stopping) {
                log::error!("Ignoring unexpected state {:?} while stopping", state);
            }

            shared_connection_pool.decommission();
            // Abort the current task after decommissioning to prevent
            // that any new tasks are spawned after aborting the current
            // task!
            if abort_pending_tasks_on_termination {
                shared_connection_pool.abort_current_task();
            }
        })
    };

    let server_listening = {
        let current_state = Arc::clone(&current_state);
        async move {
            // Give the server some time to become ready and start listening
            // before announcing the actual endpoint address, i.e. when using
            // an ephemeral port. The delay might need to be tuned depending
            // on how long the startup actually takes. Unfortunately warp does
            // not provide any signal when the server has started listening.
            sleep(WEB_SERVER_LISTENING_DELAY).await;

            match current_state.switch(State::Starting, State::Listening) {
                Ok(_) => {
                    // -> stderr
                    log::info!("Listening on {}", socket_addr);
                    // -> stdout
                    println!("{}", socket_addr);
                }
                Err(state) => {
                    log::error!("Unexpected state {:?} while starting", state);
                    // Do not advertise the endpoint address on stdout
                }
            }
        }
    };

    join!(server_listener, server_listening);
    log::info!("Stopped");

    log::info!("Terminating");
    current_state.store(State::Terminating);

    Ok(())
}