// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use clap;
use clap::{App, Arg};

use log::LevelFilter as LogLevelFilter;

const VERBOSITY_ARG: &str = "VERBOSITY";

const DATABASE_URL_ARG: &str = "DATABASE_URL";
const DATABASE_URL_DEFAULT: &str = ":memory:";

const SKIP_DATABASE_MAINTENANCE_ARG: &str = "SKIP_DATABASE_MAINTENANCE";

const LISTEN_ADDR_ARG: &str = "LISTEN_ADDR";
const LISTEN_ADDR_DEFAULT: &str = "localhost:8080";

pub struct ArgMatches<'a>(clap::ArgMatches<'a>);

impl<'a> ArgMatches<'a> {
    pub fn new<'b>(app: App<'a, 'b>) -> Self {
        ArgMatches(
            app.arg(
                Arg::with_name(DATABASE_URL_ARG)
                    .help("Sets the database URL")
                    .default_value(DATABASE_URL_DEFAULT)
                    .index(1),
            ).arg(
                    Arg::with_name(LISTEN_ADDR_ARG)
                        .short("l")
                        .long("listen")
                        .default_value(LISTEN_ADDR_DEFAULT)
                        .help("Sets the network listen address"),
                )
                .arg(
                    Arg::with_name(SKIP_DATABASE_MAINTENANCE_ARG)
                        .long("skipDatabaseMaintenance")
                        .help("Skips database schema migration and maintenance tasks on startup"),
                )
                .arg(
                    Arg::with_name(VERBOSITY_ARG)
                        .short("v")
                        .long("verbose")
                        .multiple(true)
                        .help("Sets the level of verbosity (= number of occurrences)"),
                )
                .get_matches(),
        )
    }

    pub fn log_level_filter(&self) -> LogLevelFilter {
        match self.0.occurrences_of(VERBOSITY_ARG) {
            0 => LogLevelFilter::Error,
            1 => LogLevelFilter::Warn,
            2 => LogLevelFilter::Info,
            3 => LogLevelFilter::Debug,
            _ => LogLevelFilter::Trace,
        }
    }

    pub fn database_url(&self) -> &str {
        self.0.value_of(DATABASE_URL_ARG).unwrap()
    }

    pub fn skip_database_maintenance(&self) -> bool {
        self.0.is_present(SKIP_DATABASE_MAINTENANCE_ARG)
    }

    pub fn listen_addr(&self) -> &str {
        self.0.value_of(LISTEN_ADDR_ARG).unwrap()
    }
}
