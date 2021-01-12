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

use anyhow::Error;
use dotenv::dotenv;
use log::LevelFilter as LogLevelFilter;
use std::{
    env,
    net::{IpAddr, Ipv6Addr, SocketAddr},
};

pub fn init_environment() {
    if let Ok(path) = dotenv() {
        // Print to stderr because logging has not been initialized yet
        eprintln!("Loaded environment from file {:?}", path);
    }
}

const LOG_LEVEL_ENV: &str = "LOG_LEVEL";
const LOG_LEVEL_FILTER_DEFAULT: LogLevelFilter = LogLevelFilter::Info;

fn parse_log_level_filter(log_level: &str) -> Option<LogLevelFilter> {
    match log_level.to_lowercase().trim() {
        "error" => Some(LogLevelFilter::Error),
        "warn" => Some(LogLevelFilter::Warn),
        "info" => Some(LogLevelFilter::Info),
        "debug" => Some(LogLevelFilter::Debug),
        "trace" => Some(LogLevelFilter::Trace),
        _ => {
            if !log_level.is_empty() {
                eprintln!("Invalid log level: '{}'", log_level);
            }
            None
        }
    }
}

pub fn init_logging() -> LogLevelFilter {
    let mut builder = env_logger::Builder::from_default_env();
    let log_level_filter = env::var(LOG_LEVEL_ENV)
        .map_err(Error::from)
        .map(|log_level| {
            log::debug!("{} = {}", LOG_LEVEL_ENV, log_level);
            parse_log_level_filter(&log_level)
        })
        .unwrap_or_default()
        .unwrap_or(LOG_LEVEL_FILTER_DEFAULT);
    builder.filter(None, log_level_filter);
    builder.init();

    let log_level = log::max_level();

    // Print this message unconditionally to stderr to bypass the
    // actual logger for diagnostic purposes
    eprintln!("Log level: {}", log_level);

    log_level
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
            log::debug!("{} = {}", ENDPOINT_IP_ENV, var);
            var.parse().map_err(|err| {
                log::warn!("Failed to parse {}: {}", ENDPOINT_IP_ENV, err);
                Error::from(err)
            })
        })
        .unwrap_or(DEFAULT_ENDPOINT_IP);
    let endpoint_port = env::var(ENDPOINT_PORT_ENV)
        .map_err(Into::into)
        .and_then(|var| {
            log::debug!("{} = {}", ENDPOINT_PORT_ENV, var);
            if var.trim().is_empty() {
                Ok(DEFAULT_ENDPOINT_PORT)
            } else {
                var.parse().map_err(|err| {
                    log::warn!("Failed to parse {}: {}", ENDPOINT_PORT_ENV, err);
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
            log::debug!("{} = {}", DATABASE_URL_ENV, database_url);
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
            log::debug!("{} = {}", DATABASE_CONNECTION_POOL_SIZE_ENV, var);
            if var.trim().is_empty() {
                Ok(MIN_DATABASE_CONNECTION_POOL_SIZE)
            } else {
                var.parse()
                    .map(|val| {
                        if val < MIN_DATABASE_CONNECTION_POOL_SIZE {
                            log::warn!(
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
                        log::warn!(
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
