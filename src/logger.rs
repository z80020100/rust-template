// Standard
use std::io;
use std::path::Path;

// crates.io
use time::{format_description, UtcOffset};
use tracing_subscriber::fmt::time::OffsetTime;

use tracing::{info, Level};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter},
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

pub fn init() -> non_blocking::WorkerGuard {
    let format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]";
    let timer = OffsetTime::new(
        UtcOffset::current_local_offset().unwrap(),
        format_description::parse(format).unwrap(),
    );

    // TODO: parameterization
    let file_level = Level::WARN;
    let console_level = Level::INFO;
    let file_path = "log/rust_template.log";

    let directory = Path::new(file_path).parent().unwrap();
    let file_name_prefix = Path::new(file_path).file_name().unwrap();
    let file_appender = rolling::daily(directory, file_name_prefix);
    let (non_blocking_appender, _guard) = non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_timer(timer.clone())
        .with_thread_names(true)
        .with_writer(non_blocking_appender)
        .with_filter(LevelFilter::from_level(file_level));

    let stderr_level = if console_level > Level::WARN {
        Level::WARN
    } else {
        console_level
    };
    let stdeer_layer = fmt::layer()
        .with_ansi(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(timer.clone())
        .with_thread_names(true)
        .with_writer(io::stderr)
        .with_filter(LevelFilter::from_level(stderr_level));

    let stdout_level = console_level;
    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(timer.clone())
        .with_thread_names(true)
        .with_writer(io::stdout)
        .with_filter(filter_fn(move |metadata| {
            let level = *metadata.level();
            level > stderr_level && level <= stdout_level
        }));

    tracing_subscriber::Registry::default()
        .with(file_layer)
        .with(stdeer_layer)
        .with(stdout_layer)
        .init();

    info!(
        console_level = console_level.to_string(),
        file_level = file_level.to_string(),
        "Logger initialized:"
    );

    // https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html
    _guard
}
