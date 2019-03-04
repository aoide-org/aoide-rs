// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core as core;

use env_logger::Builder as LoggerBuilder;

use log::LevelFilter as LogLevelFilter;

use std::{env, fs::File, io, path::Path};

fn init_env_logger(log_level_filter: LogLevelFilter) {
    let mut logger_builder = LoggerBuilder::new();

    println!("Setting log level filter to {}", log_level_filter);
    logger_builder.filter(None, log_level_filter);

    if env::var("RUST_LOG").is_ok() {
        let rust_log_var = &env::var("RUST_LOG").unwrap();
        println!("Parsing RUST_LOG={}", rust_log_var);
        logger_builder.parse(rust_log_var);
    }

    logger_builder.init();
}

fn init_env_logger_verbosity(verbosity: u8) {
    let log_level_filter = match verbosity {
        0 => LogLevelFilter::Error,
        1 => LogLevelFilter::Warn,
        2 => LogLevelFilter::Info,
        3 => LogLevelFilter::Debug,
        _ => LogLevelFilter::Trace,
    };
    init_env_logger(log_level_filter);
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: {} <JSON_FILE>", args[0]);
        return;
    }

    // TODO: Parse verbosity from args
    init_env_logger_verbosity(3);

    let path = Path::new(&args[1]);
    try_main(path).unwrap();
}

pub fn try_main(path: &Path) -> io::Result<()> {
    log::info!("Opening file {:?}", path.as_os_str());
    let file = File::open(path)?;

    log::info!("Reading track metadata from JSON");
    let buf_reader = io::BufReader::new(file);
    let tracks: Vec<core::track::TrackEntity> = serde_json::from_reader(buf_reader).unwrap();
    log::info!("Deserialized tracks: {:#?}", tracks);
    Ok(())
}
