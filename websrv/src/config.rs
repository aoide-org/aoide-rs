// aoide.org - Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    num::{NonZeroU32, NonZeroU64},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use aoide_storage_sqlite::connection::{
    pool::{
        gatekeeper::Config as DatabaseConnectionGatekeeperConfig,
        Config as DatabaseConnectionPoolConfig,
    },
    Config as DatabaseConnectionConfig, Storage,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub endpoint: EndpointConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub ip_addr: IpAddr,
    pub port: u16,
}

const ENDPOINT_PORT_EPHEMERAL: u16 = 0;

impl EndpointConfig {
    pub const fn new_v6() -> Self {
        Self {
            ip_addr: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            port: ENDPOINT_PORT_EPHEMERAL,
        }
    }

    #[allow(unused)]
    pub const fn new_v4() -> Self {
        Self {
            ip_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: ENDPOINT_PORT_EPHEMERAL,
        }
    }

    pub fn socket_addr(self) -> SocketAddr {
        let Self { ip_addr, port } = self;
        SocketAddr::new(ip_addr, port)
    }
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self::new_v6()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub connection: DatabaseConnectionConfig,
    pub migrate_schema_on_startup: bool,
}

const DEFAULT_DATABASE_CONNECTION_POOL_SIZE: u32 = 8;

const DEFAULT_DATABASE_CONNECTION_TIMEOUT_ACQUIRE_READ: Duration = Duration::from_secs(10);

const DEFAULT_DATABASE_CONNECTION_TIMEOUT_ACQUIRE_WRITE: Duration = Duration::from_secs(30);

fn non_zero_duration_as_millis(duration: Duration) -> NonZeroU64 {
    let millis: u64 = duration.as_millis().try_into().unwrap();
    debug_assert!(millis > 0);
    millis.try_into().unwrap()
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection: DatabaseConnectionConfig {
                storage: Storage::InMemory,
                pool: DatabaseConnectionPoolConfig {
                    max_size: NonZeroU32::new(DEFAULT_DATABASE_CONNECTION_POOL_SIZE)
                        .expect("non-zero size"),
                    gatekeeper: DatabaseConnectionGatekeeperConfig {
                        acquire_read_timeout_millis: non_zero_duration_as_millis(
                            DEFAULT_DATABASE_CONNECTION_TIMEOUT_ACQUIRE_READ,
                        ),
                        acquire_write_timeout_millis: non_zero_duration_as_millis(
                            DEFAULT_DATABASE_CONNECTION_TIMEOUT_ACQUIRE_WRITE,
                        ),
                    },
                },
            },
            migrate_schema_on_startup: true,
        }
    }
}
