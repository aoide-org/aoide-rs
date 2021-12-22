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

use std::{
    env,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    str::ParseBoolError,
};

use anyhow::Error;
use dotenv::dotenv;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_log::LogTracer;
use tracing_subscriber::EnvFilter;

pub fn init_environment() {
    if let Ok(path) = dotenv() {
        // Print to stderr because logging has not been initialized yet
        eprintln!("Loaded environment from dotenv file {:?}", path);
    }
}

const TRACING_SUBSCRIBER_ENV_FILTER_DEFAULT: &str = "info";

fn create_env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|err| {
        let rust_log_from_env = env::var("RUST_LOG").ok();
        if let Some(rust_log_from_env) = rust_log_from_env {
            if !rust_log_from_env.is_empty() {
                eprintln!(
                    "Failed to parse RUST_LOG environment variable '{}': {}",
                    rust_log_from_env, err
                );
            }
        }
        EnvFilter::new(TRACING_SUBSCRIBER_ENV_FILTER_DEFAULT.to_owned())
    })
}

fn create_tracing_subscriber() -> anyhow::Result<impl Subscriber> {
    let env_filter = create_env_filter();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .finish();
    Ok(subscriber)
}

pub fn init_tracing_and_logging() -> anyhow::Result<()> {
    // Capture and redirect all log messages as tracing events
    LogTracer::init()?;

    let subscriber = create_tracing_subscriber()?;
    set_global_default(subscriber)?;

    Ok(())
}

const ENDPOINT_IP_ENV: &str = "ENDPOINT_IP";
const ENDPOINT_IP_DEFAULT: IpAddr = IpAddr::V6(Ipv6Addr::UNSPECIFIED);

const ENDPOINT_PORT_ENV: &str = "ENDPOINT_PORT";
const ENDPOINT_PORT_EPHEMERAL: u16 = 0;
const ENDPOINT_PORT_DEFAULT: u16 = ENDPOINT_PORT_EPHEMERAL;

pub fn parse_endpoint_addr() -> SocketAddr {
    let endpoint_ip = env::var(ENDPOINT_IP_ENV)
        .map_err(Into::into)
        .and_then(|var| {
            tracing::debug!("{} = {}", ENDPOINT_IP_ENV, var);
            var.parse().map_err(|err| {
                tracing::warn!("Failed to parse {} = {}: {}", ENDPOINT_IP_ENV, var, err);
                Error::from(err)
            })
        })
        .unwrap_or(ENDPOINT_IP_DEFAULT);
    let endpoint_port = env::var(ENDPOINT_PORT_ENV)
        .map_err(Into::into)
        .and_then(|var| {
            tracing::debug!("{} = {}", ENDPOINT_PORT_ENV, var);
            if var.trim().is_empty() {
                Ok(ENDPOINT_PORT_DEFAULT)
            } else {
                var.parse().map_err(|err| {
                    tracing::warn!("Failed to parse {} = {}: {}", ENDPOINT_PORT_ENV, var, err);
                    Error::from(err)
                })
            }
        })
        .unwrap_or(ENDPOINT_PORT_DEFAULT);
    (endpoint_ip, endpoint_port).into()
}

const DATABASE_URL_ENV: &str = "DATABASE_URL";
const DATABASE_URL_DEFAULT: &str = ":memory:";

pub fn parse_database_url() -> String {
    env::var(DATABASE_URL_ENV)
        .map_err(Error::from)
        .map(|var| {
            tracing::debug!("{} = {}", DATABASE_URL_ENV, var);
            var
        })
        .unwrap_or_else(|_| DATABASE_URL_DEFAULT.into())
}

const DATABASE_MIGRATE_SCHEMA_ON_STARTUP_ENV: &str = "DATABASE_MIGRATE_SCHEMA_ON_STARTUP";
const DATABASE_MIGRATE_SCHEMA_ON_STARTUP_DEFAULT: bool = true;

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

pub fn parse_database_migrate_schema_on_startup() -> bool {
    env::var(DATABASE_MIGRATE_SCHEMA_ON_STARTUP_ENV)
        .map_err(Into::into)
        .and_then(|var| {
            tracing::debug!("{} = {}", DATABASE_MIGRATE_SCHEMA_ON_STARTUP_ENV, var);
            parse_bool_var(&var).map_err(|err| {
                tracing::warn!(
                    "Failed to parse {} = {}: {}",
                    DATABASE_MIGRATE_SCHEMA_ON_STARTUP_ENV,
                    var,
                    err,
                );
                Error::from(err)
            })
        })
        .unwrap_or(DATABASE_MIGRATE_SCHEMA_ON_STARTUP_DEFAULT)
}

const DATABASE_CONNECTION_POOL_SIZE_ENV: &str = "DATABASE_CONNECTION_POOL_SIZE";
const DATABASE_CONNECTION_POOL_SIZE_MIN: u32 = 1;
const DATABASE_CONNECTION_POOL_SIZE_DEFAULT: u32 = DATABASE_CONNECTION_POOL_SIZE_MIN;

pub fn parse_database_connection_pool_size() -> u32 {
    env::var(DATABASE_CONNECTION_POOL_SIZE_ENV)
        .map_err(Into::into)
        .and_then(|var| {
            tracing::debug!("{} = {}", DATABASE_CONNECTION_POOL_SIZE_ENV, var);
            if var.trim().is_empty() {
                Ok(DATABASE_CONNECTION_POOL_SIZE_MIN)
            } else {
                var.parse()
                    .map(|val| {
                        if val < DATABASE_CONNECTION_POOL_SIZE_MIN {
                            tracing::warn!(
                                "Invalid {} = {} < {}",
                                DATABASE_CONNECTION_POOL_SIZE_ENV,
                                val,
                                DATABASE_CONNECTION_POOL_SIZE_MIN
                            );
                            DATABASE_CONNECTION_POOL_SIZE_MIN
                        } else {
                            val
                        }
                    })
                    .map_err(|err| {
                        tracing::warn!(
                            "Failed to parse {} = {}: {}",
                            DATABASE_CONNECTION_POOL_SIZE_ENV,
                            var,
                            err
                        );
                        Error::from(err)
                    })
            }
        })
        .unwrap_or(DATABASE_CONNECTION_POOL_SIZE_DEFAULT)
}
