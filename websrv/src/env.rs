// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    env::{self, VarError},
    net::IpAddr,
    num::NonZeroU32,
    str::ParseBoolError,
};

use aoide_storage_sqlite::connection::{IN_MEMORY_STORAGE, Storage as SqliteDatabaseStorage};
use dotenvy::dotenv;
use tracing::{Subscriber, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::EnvFilter;
use url::Url;

use crate::config::Config;

pub(crate) fn init_environment() {
    if let Ok(path) = dotenv() {
        // Print to stderr because logging has not been initialized yet
        eprintln!(
            "Loaded environment from dotenv file {path}",
            path = path.display()
        );
    }
}

// Prevents warning messages when reading environment variables that are not present
fn read_optional_var(key: &str) -> Result<Option<String>, VarError> {
    match env::var(key) {
        Ok(var) => Ok(Some(var)),
        Err(VarError::NotPresent) => Ok(None),
        Err(err) => Err(err),
    }
}

const TRACING_SUBSCRIBER_ENV_FILTER_DEFAULT: &str = "info";

fn create_env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|err| {
        let rust_log_from_env = env::var("RUST_LOG").ok();
        if let Some(rust_log_from_env) = rust_log_from_env
            && !rust_log_from_env.is_empty()
        {
            eprintln!("Failed to parse RUST_LOG environment variable '{rust_log_from_env}': {err}");
        }
        EnvFilter::new(TRACING_SUBSCRIBER_ENV_FILTER_DEFAULT)
    })
}

fn create_tracing_subscriber() -> impl Subscriber {
    let env_filter = create_env_filter();
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .finish()
}

pub(crate) fn init_tracing_and_logging() -> anyhow::Result<()> {
    // Capture and redirect all log messages as tracing events
    LogTracer::init()?;

    let subscriber = create_tracing_subscriber();
    set_global_default(subscriber)?;

    Ok(())
}

const ENDPOINT_IP_ENV: &str = "ENDPOINT_IP";

fn parse_endpoint_ip_addr() -> Option<IpAddr> {
    read_optional_var(ENDPOINT_IP_ENV)
        .map_err(|err| err.to_string())
        .and_then(|var| {
            var.map(|var| {
                log::debug!("{ENDPOINT_IP_ENV} = {var}");
                var.parse()
                    .map_err(|err| format!("Failed to parse '{ENDPOINT_IP_ENV}' = '{var}': {err}"))
            })
            .transpose()
        })
        .map_err(|err| {
            log::warn!("{err}");
        })
        .ok()
        .flatten()
}

const ENDPOINT_PORT_ENV: &str = "ENDPOINT_PORT";

fn parse_endpoint_port() -> Option<u16> {
    read_optional_var(ENDPOINT_PORT_ENV)
        .map_err(|err| err.to_string())
        .and_then(|var| {
            var.map(|var| {
                log::debug!("{ENDPOINT_PORT_ENV} = {var}");
                if var.trim().is_empty() {
                    Ok(None)
                } else {
                    var.parse()
                        .map_err(|err| {
                            format!("Failed to parse '{ENDPOINT_PORT_ENV}' = '{var}': {err}")
                        })
                        .map(Some)
                }
            })
            .transpose()
            .map(Option::flatten)
        })
        .map_err(|err| {
            log::warn!("{err}");
        })
        .ok()
        .flatten()
}

const DATABASE_URL_ENV: &str = "DATABASE_URL";

fn parse_sqlite_database_storage() -> Option<SqliteDatabaseStorage> {
    read_optional_var(DATABASE_URL_ENV)
        .map_err(|err| err.to_string())
        .and_then(|var| {
            var.map(|var| {
                log::debug!("{DATABASE_URL_ENV} = {var}");
                match var.trim() {
                    "" => Ok(None),
                    IN_MEMORY_STORAGE => Ok(Some(SqliteDatabaseStorage::InMemory)),
                    trimmed => trimmed
                        .parse::<Url>()
                        .map_err(|err| err.to_string())
                        .and_then(|url| {
                            url.to_file_path()
                                .map_err(|()| "not a file path".to_owned())
                        })
                        .map_err(|err| {
                            format!("Failed to parse '{DATABASE_URL_ENV}' = '{var}': {err}")
                        })
                        .map(|path| Some(SqliteDatabaseStorage::File { path })),
                }
            })
            .transpose()
            .map(Option::flatten)
        })
        .map_err(|err| {
            log::warn!("{err}");
        })
        .ok()
        .flatten()
}

const DATABASE_CONNECTION_POOL_SIZE_ENV: &str = "DATABASE_CONNECTION_POOL_SIZE";

fn parse_database_connection_pool_size() -> Option<NonZeroU32> {
    read_optional_var(DATABASE_CONNECTION_POOL_SIZE_ENV)
        .map_err(|err| err.to_string())
        .and_then(|var| {
            var.map(|var| {
                log::debug!("{DATABASE_CONNECTION_POOL_SIZE_ENV} = {var}");
                if var.trim().is_empty() {
                    // Silently ignore whitespace
                    Ok(None)
                } else {
                    var.parse().map(Some).map_err(|err| {
                        format!(
                            "Failed to parse '{DATABASE_CONNECTION_POOL_SIZE_ENV}' = '{var}': \
                             {err}"
                        )
                    })
                }
            })
            .transpose()
            .map(Option::flatten)
        })
        .map_err(|err| {
            log::warn!("{err}");
        })
        .ok()
        .flatten()
}

fn parse_bool_var(var: &str) -> Result<bool, ParseBoolError> {
    var.to_lowercase().parse::<bool>().or_else(|err| {
        if let Ok(val) = var.parse::<u8>() {
            match val {
                0 => return Ok(false),
                1 => return Ok(true),
                _ => (),
            }
        }
        Err(err)
    })
}

fn parse_option_bool_var_with_key(key: &str) -> Option<bool> {
    read_optional_var(key)
        .map_err(|err| err.to_string())
        .and_then(|var| {
            var.map(|var| {
                log::debug!("{key} = {var}");
                parse_bool_var(&var)
                    .map_err(|err| format!("Failed to parse '{key}' = '{var}': {err}"))
            })
            .transpose()
        })
        .map_err(|err| {
            log::warn!("{err}");
        })
        .ok()
        .flatten()
}

const DATABASE_MIGRATE_SCHEMA_ON_STARTUP_ENV: &str = "DATABASE_MIGRATE_SCHEMA_ON_STARTUP";

fn parse_database_migrate_schema_on_startup() -> Option<bool> {
    parse_option_bool_var_with_key(DATABASE_MIGRATE_SCHEMA_ON_STARTUP_ENV)
}

pub(crate) fn parse_config_into(config: &mut Config) {
    if let Some(ip_addr) = parse_endpoint_ip_addr() {
        config.network.endpoint.ip_addr = ip_addr;
    }
    if let Some(port) = parse_endpoint_port() {
        config.network.endpoint.port = port;
    }
    if let Some(storage) = parse_sqlite_database_storage() {
        config.database.connection.storage = storage;
    }
    if let Some(connection_pool_size) = parse_database_connection_pool_size() {
        config.database.connection.pool.max_size = connection_pool_size;
    }
    if let Some(migrate_schema_on_startup) = parse_database_migrate_schema_on_startup() {
        config.database.migrate_schema_on_startup = migrate_schema_on_startup;
    }
}

#[cfg(feature = "launcher-ui")]
const LAUNCH_HEADLESS_ENV: &str = "LAUNCH_HEADLESS";

#[cfg(feature = "launcher-ui")]
pub(crate) fn parse_launch_headless() -> Option<bool> {
    parse_option_bool_var_with_key(LAUNCH_HEADLESS_ENV)
}

const DEFAULT_CONFIG_ENV: &str = "DEFAULT_CONFIG";

pub(crate) fn parse_default_config() -> Option<bool> {
    parse_option_bool_var_with_key(DEFAULT_CONFIG_ENV)
}
