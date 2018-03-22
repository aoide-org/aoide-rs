extern crate aoide;

extern crate env_logger;

#[macro_use] extern crate log;

use env_logger::Builder as LoggerBuilder;

use log::LevelFilter as LogLevelFilter;

use std::env;

pub fn init_env_logger_verbosity(verbosity: u8) {
    let mut logger_builder = LoggerBuilder::new();

    let log_level_filter = match verbosity {
        0 => LogLevelFilter::Error,
        1 => LogLevelFilter::Warn,
        2 => LogLevelFilter::Info,
        3 => LogLevelFilter::Debug,
        _ => LogLevelFilter::Trace,
    };
    println!("Setting log level filter to {}", log_level_filter);
    logger_builder.filter(None, log_level_filter);

    if env::var("RUST_LOG").is_ok() {
        let rust_log_var = &env::var("RUST_LOG").unwrap();
        println!("Parsing RUST_LOG={}", rust_log_var);
        logger_builder.parse(rust_log_var);
    }

    logger_builder.init();
}

pub fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 1 {
        println!("usage: {}", args[0]);
        return;
    }

    // TODO: Parse verbosity from args
    init_env_logger_verbosity(2);

    info!("TODO");
}
