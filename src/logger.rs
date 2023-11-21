// Standard
use std::io;
use std::panic;
use std::path::Path;

// crates.io
use time::{format_description, UtcOffset};
use tracing::Level;
pub use tracing::{debug, error, info, trace, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_panic::panic_hook;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer,
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

    let stdeer_layer = fmt::layer()
        .with_ansi(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(timer.clone())
        .with_thread_names(true)
        .with_writer(io::stderr)
        .with_filter(LevelFilter::from_level(console_level));

    tracing_subscriber::Registry::default()
        .with(file_layer)
        .with(stdeer_layer)
        .init();

    // https://docs.rs/tracing-panic/0.1.1/tracing_panic/fn.panic_hook.html
    panic::set_hook(Box::new(panic_hook));

    info!(
        console_level = console_level.to_string(),
        file_level = file_level.to_string(),
        "Logger initialized:"
    );

    // https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html
    _guard
}
