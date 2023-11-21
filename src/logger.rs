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

pub struct Logger {
    pub console_level: Level,
    pub file_level: Level,
    pub file_path: String,
}

impl Logger {
    pub fn new(console_level: Level, file_level: Level, file_path: String) -> Self {
        Self {
            console_level,
            file_level,
            file_path,
        }
    }

    pub fn init(&self) -> non_blocking::WorkerGuard {
        let format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]";
        let timer = OffsetTime::new(
            UtcOffset::current_local_offset().unwrap(),
            format_description::parse(format).unwrap(),
        );

        let directory = Path::new(&self.file_path).parent().unwrap();
        let file_name_prefix = Path::new(&self.file_path).file_name().unwrap();
        let file_appender = rolling::daily(directory, file_name_prefix);
        let (non_blocking_appender, _guard) = non_blocking(file_appender);
        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_timer(timer.clone())
            .with_thread_names(true)
            .with_writer(non_blocking_appender)
            .with_filter(LevelFilter::from_level(self.file_level));

        let stdeer_layer = fmt::layer()
            .with_ansi(true)
            .with_file(true)
            .with_line_number(true)
            .with_timer(timer.clone())
            .with_thread_names(true)
            .with_writer(io::stderr)
            .with_filter(LevelFilter::from_level(self.console_level));

        tracing_subscriber::Registry::default()
            .with(file_layer)
            .with(stdeer_layer)
            .init();

        // https://docs.rs/tracing-panic/0.1.1/tracing_panic/fn.panic_hook.html
        panic::set_hook(Box::new(panic_hook));

        info!(
            console_level = self.console_level.to_string(),
            file_level = self.file_level.to_string(),
            "Logger initialized:"
        );

        // https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html
        _guard
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(
            Level::INFO,
            Level::WARN,
            "log/rust_template.log".to_string(),
        )
    }
}
