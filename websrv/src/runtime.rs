// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use axum::{
    Json, Router,
    extract::{self, DefaultBodyLimit, FromRef},
    http::StatusCode,
    response::Html,
};
use jiff::Timestamp;
use serde_json::Value;
use tokio::{net::TcpListener, sync::mpsc};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi as _;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use aoide_repo_sqlite::{initialize_connection, initialize_database};
use aoide_storage_sqlite::connection::pool::{
    create_connection_pool, gatekeeper::Gatekeeper as DatabaseConnectionGatekeeper,
    get_pooled_connection,
};
use aoide_usecases_sqlite as uc;
use aoide_websrv_axum_sqlite::{handle_method_not_allowed, handle_not_found};

use super::{config::Config, routing};
use crate::{config::DatabaseConfig, openapi as doc};

static INDEX_HTML: &str = include_str!("../res/index.html");

const OPENAPI_JSON_PATH: &str = "/openapi.json";

const SWAGGER_UI_PATH: &str = "/swagger-ui";

#[derive(Clone, FromRef)]
struct ServiceState {
    about_json: Arc<Value>,
    shutdown_tx: mpsc::UnboundedSender<()>,
}

/// Web interface
///
/// A minimal HTML page for exploring the API.
#[utoipa::path(
    get,
    path = "/",
    operation_id = "getIndexHtml",
    tag = doc::ADMINISTRATION_TAG,
    responses(
        (status = 200, description = "The HTML page.", body = String, content_type = "text/html"),
    ),
)]
async fn index_html() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// Report service properties
///
/// Inspect the service environment.
#[utoipa::path(
    get,
    path = "/about",
    operation_id = "about",
    tag = doc::ADMINISTRATION_TAG,
    responses(
        (status = 200, description = "The properties of the service.", body = Object),
    ),
)]
async fn about(extract::State(about_json): extract::State<Arc<Value>>) -> Json<Value> {
    Json(Value::clone(&about_json))
}

/// Shut down gracefully
///
/// Shut down the server after finishing all pending requests.
#[utoipa::path(
    post,
    path = "/shutdown",
    operation_id = "shutdown",
    tag = doc::ADMINISTRATION_TAG,
    responses(
        (status = 202, description = "The request has been accepted."),
        (status = 502, description = "The server failed to forward the request."),
    ),
)]
async fn shutdown(
    extract::State(shutdown_tx): extract::State<mpsc::UnboundedSender<()>>,
) -> Result<StatusCode, StatusCode> {
    shutdown_tx
        .send(())
        .map(|()| StatusCode::ACCEPTED)
        .map_err(|_| {
            log::warn!("Failed to forward shutdown request");
            StatusCode::BAD_GATEWAY
        })
}

/// All routes of the service, documented by the OpenAPI document.
fn openapi_router(gatekeeper: Arc<DatabaseConnectionGatekeeper>) -> OpenApiRouter<ServiceState> {
    OpenApiRouter::with_openapi(doc::ApiDoc::openapi())
        .nest("/api", routing::api::create_router(gatekeeper))
        .routes(routes!(index_html))
        .routes(routes!(about))
        .routes(routes!(shutdown))
}

#[cfg(test)]
pub(crate) fn collect_openapi(
    gatekeeper: Arc<DatabaseConnectionGatekeeper>,
) -> utoipa::openapi::OpenApi {
    openapi_router(gatekeeper).into_openapi()
}

fn create_router(gatekeeper: Arc<DatabaseConnectionGatekeeper>, state: ServiceState) -> Router {
    let (router, openapi) = openapi_router(gatekeeper).split_for_parts();
    let swagger_ui = SwaggerUi::new(SWAGGER_UI_PATH)
        .external_url_unchecked(OPENAPI_JSON_PATH, doc::finish_document(openapi));
    router
        .merge(swagger_ui)
        .method_not_allowed_fallback(handle_method_not_allowed)
        .fallback(handle_not_found)
        // Request bodies are unlimited, e.g. for importing or replacing many tracks at once.
        .layer(DefaultBodyLimit::disable())
        .layer(CorsLayer::new().allow_origin(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

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
        "Commissioning SQLite database: {storage}",
        storage = config.connection.storage,
    );

    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    let pool_max_size = config.connection.pool.max_size;
    log::info!("Creating connection pool of max. size {pool_max_size}");
    let connection_pool = create_connection_pool(
        &config.connection.storage,
        pool_max_size,
        initialize_connection,
    )?;

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
    let launched_at = Timestamp::now();

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

    let (server_shutdown_tx, mut server_shutdown_rx) = mpsc::unbounded_channel::<()>();
    let state = ServiceState {
        about_json: Arc::new(about_json),
        shutdown_tx: server_shutdown_tx,
    };
    let router = create_router(Arc::clone(&shared_connection_pool), state);

    log::info!("Initializing server");

    log::info!("Starting");
    current_state_tx.write(Some(State::Starting));

    let tcp_listener = TcpListener::bind(config.network.endpoint.socket_addr()).await?;
    let socket_addr = tcp_listener.local_addr()?;

    log::info!("Listening on {socket_addr}");
    current_state_tx.write(Some(State::Listening { socket_addr }));

    let abort_pending_tasks_on_termination = Arc::new(AtomicBool::new(false));
    let shutdown_signal = {
        let mut command_rx = command_rx;
        let abort_pending_tasks_on_termination = Arc::clone(&abort_pending_tasks_on_termination);
        async move {
            tokio::select! {
                Some(()) = server_shutdown_rx.recv() => (),
                Some(command) = command_rx.recv() => {
                    match command {
                        Command::Terminate {
                            abort_pending_tasks,
                        } => {
                            abort_pending_tasks_on_termination.store(abort_pending_tasks, Ordering::Release);
                        }
                    }
                }
                else => (),
            }
        }
    };

    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    log::info!("Stopping");
    current_state_tx.write(Some(State::Stopping));

    shared_connection_pool.decommission();

    log::info!("Terminating");
    current_state_tx.write(Some(State::Terminating));

    Ok(())
}
