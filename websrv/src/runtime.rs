// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use time::OffsetDateTime;
use tokio::{sync::mpsc, time::sleep};
use warp::{http::StatusCode, Filter};

use aoide_storage_sqlite::connection::pool::{
    create_connection_pool, gatekeeper::Gatekeeper as DatabaseConnectionGatekeeper,
    get_pooled_connection,
};

use aoide_repo_sqlite::initialize_database;

use aoide_usecases_sqlite as uc;

use aoide_websrv_warp_sqlite::handle_rejection;

use crate::config::DatabaseConfig;

use super::{config::Config, routing};

const WEB_SERVER_LISTENING_DELAY: Duration = Duration::from_millis(250);

static OPENAPI_YAML: &str = include_str!("../res/openapi.yaml");

#[cfg(not(feature = "webapp"))]
static INDEX_HTML: &str = include_str!("../res/index.html");

#[derive(Debug, Clone, Copy)]
pub(crate) enum State {
    Launching,
    Starting,
    Listening { socket_addr: SocketAddr },
    Stopping,
    Terminating,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Command {
    Terminate { abort_pending_tasks: bool },
}

fn provision_database(config: &DatabaseConfig) -> anyhow::Result<DatabaseConnectionGatekeeper> {
    log::info!(
        "Commissioning SQLite database: {}",
        config.connection.storage,
    );

    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    let pool_max_size = config.connection.pool.max_size;
    log::info!("Creating connection pool of max. size {pool_max_size}");
    let connection_pool = create_connection_pool(&config.connection.storage, pool_max_size)?;

    log::info!("Initializing database");
    initialize_database(&mut *get_pooled_connection(&connection_pool)?)?;

    if config.migrate_schema_on_startup {
        log::info!("Migrating database schema");
        uc::database::migrate_schema(&mut *get_pooled_connection(&connection_pool)?)?;
    }

    Ok(DatabaseConnectionGatekeeper::new(
        connection_pool,
        config.connection.pool.gatekeeper,
    ))
}

pub(crate) async fn run(
    config: Config,
    command_rx: mpsc::UnboundedReceiver<Command>,
    current_state_tx: discro::Publisher<Option<State>>,
) -> anyhow::Result<()> {
    let launched_at = OffsetDateTime::now_utc();

    log::info!("Launching");
    current_state_tx.write(Some(State::Launching));

    let shared_connection_pool = Arc::new(provision_database(&config.database)?);

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

    #[cfg(not(feature = "webapp"))]
    let static_filters = static_filters.or(warp::path::end().map(|| warp::reply::html(INDEX_HTML)));

    let all_filters = api_filters
        .or(static_filters)
        .or(shutdown_filter)
        .or(about_filter);

    #[cfg(feature = "webapp")]
    let all_filters = all_filters.or(routing::app::get_index().or(routing::app::get_assets()));

    log::info!("Initializing server");

    let server = warp::serve(
        all_filters
            .with(warp::cors().allow_any_origin())
            .recover(handle_rejection),
    );

    log::info!("Starting");
    current_state_tx.write(Some(State::Starting));

    let abort_pending_tasks_on_termination = Arc::new(AtomicBool::new(false));
    let (socket_addr, server_listener) = {
        let mut command_rx = command_rx;
        let abort_pending_tasks_on_termination = Arc::clone(&abort_pending_tasks_on_termination);
        server.bind_with_graceful_shutdown(config.network.endpoint.socket_addr(), async move {
            loop {
                tokio::select! {
                    Some(()) = server_shutdown_rx.recv() => break,
                    Some(command) = command_rx.recv() => {
                        match command {
                            Command::Terminate {
                                abort_pending_tasks,
                            } => {
                                abort_pending_tasks_on_termination.store(abort_pending_tasks, Ordering::Release);
                                break;
                            }
                        }
                    }
                    else => break,
                }
            }
        })
    };

    // Give the server some time to become ready and start listening
    // before announcing the actual endpoint address, i.e. when using
    // an ephemeral port. The delay might need to be tuned depending
    // on how long the startup actually takes. Unfortunately warp does
    // not provide any signal when the server has started listening.
    sleep(WEB_SERVER_LISTENING_DELAY).await;

    log::info!("Listening on {socket_addr}");
    current_state_tx.write(Some(State::Listening { socket_addr }));

    server_listener.await;

    log::info!("Stopping");
    current_state_tx.write(Some(State::Stopping));

    shared_connection_pool.decommission();
    // Abort the current task after decommissioning to prevent
    // that any new tasks are spawned after aborting the current
    // task!
    if abort_pending_tasks_on_termination.load(Ordering::Acquire) {
        shared_connection_pool.abort_current_task();
    }

    log::info!("Terminating");
    current_state_tx.write(Some(State::Terminating));

    Ok(())
}
