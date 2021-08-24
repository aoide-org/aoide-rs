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
};

use anyhow::Error;
use dotenv::dotenv;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn init_environment() {
    if let Ok(path) = dotenv() {
        // Print to stderr because logging has not been initialized yet
        eprintln!("Loaded environment from dotenv file {:?}", path);
    }
}

const DEFAULT_TRACING_SUBSCRIBER_ENV_FILTER: &str = "info";

fn create_tracing_subscriber() -> anyhow::Result<impl Subscriber> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|err| {
        let rust_log_from_env = env::var("RUST_LOG").ok();
        if let Some(rust_log_from_env) = rust_log_from_env {
            if !rust_log_from_env.is_empty() {
                eprintln!(
                    "Failed to parse RUST_LOG environment variable '{}': {}",
                    rust_log_from_env, err
                );
            }
        }
        EnvFilter::new(DEFAULT_TRACING_SUBSCRIBER_ENV_FILTER.to_owned())
    });
    let formatting_layer = BunyanFormattingLayer::new(
        env!("CARGO_PKG_NAME").to_owned(),
        // Output the formatted spans to stderr
        std::io::stderr,
    );
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    Ok(subscriber)
}

pub fn init_tracing_subscriber() -> anyhow::Result<()> {
    // Capture and redirect all log messages as tracing events
    LogTracer::init()?;

    let subscriber = create_tracing_subscriber()?;
    set_global_default(subscriber)?;

    Ok(())
}

const ENDPOINT_IP_ENV: &str = "ENDPOINT_IP";
const DEFAULT_ENDPOINT_IP: IpAddr = IpAddr::V6(Ipv6Addr::UNSPECIFIED);

const ENDPOINT_PORT_ENV: &str = "ENDPOINT_PORT";
const EPHEMERAL_PORT: u16 = 0;
const DEFAULT_ENDPOINT_PORT: u16 = EPHEMERAL_PORT;

pub fn parse_endpoint_addr() -> SocketAddr {
    let endpoint_ip = env::var(ENDPOINT_IP_ENV)
        .map_err(Into::into)
        .and_then(|var| {
            tracing::debug!("{} = {}", ENDPOINT_IP_ENV, var);
            var.parse().map_err(|err| {
                tracing::warn!("Failed to parse {}: {}", ENDPOINT_IP_ENV, err);
                Error::from(err)
            })
        })
        .unwrap_or(DEFAULT_ENDPOINT_IP);
    let endpoint_port = env::var(ENDPOINT_PORT_ENV)
        .map_err(Into::into)
        .and_then(|var| {
            tracing::debug!("{} = {}", ENDPOINT_PORT_ENV, var);
            if var.trim().is_empty() {
                Ok(DEFAULT_ENDPOINT_PORT)
            } else {
                var.parse().map_err(|err| {
                    tracing::warn!("Failed to parse {}: {}", ENDPOINT_PORT_ENV, err);
                    Error::from(err)
                })
            }
        })
        .unwrap_or(DEFAULT_ENDPOINT_PORT);
    (endpoint_ip, endpoint_port).into()
}

const DATABASE_URL_ENV: &str = "DATABASE_URL";
const DEFAULT_DATABASE_URL: &str = ":memory:";

pub fn parse_database_url() -> String {
    env::var(DATABASE_URL_ENV)
        .map_err(Error::from)
        .map(|database_url| {
            tracing::debug!("{} = {}", DATABASE_URL_ENV, database_url);
            database_url
        })
        .unwrap_or_else(|_| DEFAULT_DATABASE_URL.into())
}

const DATABASE_CONNECTION_POOL_SIZE_ENV: &str = "DATABASE_CONNECTION_POOL_SIZE";
const MIN_DATABASE_CONNECTION_POOL_SIZE: u32 = 1;
const DEFAULT_DATABASE_CONNECTION_POOL_SIZE: u32 = MIN_DATABASE_CONNECTION_POOL_SIZE;

pub fn parse_database_connection_pool_size() -> u32 {
    env::var(DATABASE_CONNECTION_POOL_SIZE_ENV)
        .map_err(Into::into)
        .and_then(|var| {
            tracing::debug!("{} = {}", DATABASE_CONNECTION_POOL_SIZE_ENV, var);
            if var.trim().is_empty() {
                Ok(MIN_DATABASE_CONNECTION_POOL_SIZE)
            } else {
                var.parse()
                    .map(|val| {
                        if val < MIN_DATABASE_CONNECTION_POOL_SIZE {
                            tracing::warn!(
                                "Invalid {} = {} < {}",
                                DATABASE_CONNECTION_POOL_SIZE_ENV,
                                val,
                                MIN_DATABASE_CONNECTION_POOL_SIZE
                            );
                            MIN_DATABASE_CONNECTION_POOL_SIZE
                        } else {
                            val
                        }
                    })
                    .map_err(|err| {
                        tracing::warn!(
                            "Failed to parse {}: {}",
                            DATABASE_CONNECTION_POOL_SIZE_ENV,
                            err
                        );
                        Error::from(err)
                    })
            }
        })
        .unwrap_or(DEFAULT_DATABASE_CONNECTION_POOL_SIZE)
}
