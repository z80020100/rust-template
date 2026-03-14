// Standard
use std::io;
use std::panic;
use std::path::Path;
use std::str::FromStr;

// crates.io
use time::{UtcOffset, format_description};
use tracing::Level;
pub use tracing::{debug, error, info, trace, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_panic::panic_hook;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::{
    Layer, filter::LevelFilter, fmt, layer::SubscriberExt, reload,
    util::SubscriberInitExt,
};

// This library
use crate::configs::{AppendersConfig, LoggerConfig};
use crate::error::ErrorCode;

// Type-erased reload handle to avoid deeply nested generic types
trait LevelFilterReloader: Send + Sync {
    fn reload(&self, new_filter: LevelFilter) -> Result<(), reload::Error>;
}

impl<S> LevelFilterReloader for reload::Handle<LevelFilter, S>
where
    LevelFilter: Layer<S> + 'static,
    S: tracing::Subscriber + 'static,
{
    fn reload(&self, new_filter: LevelFilter) -> Result<(), reload::Error> {
        self.reload(new_filter)
    }
}

pub struct Logger {
    pub guard: non_blocking::WorkerGuard,
    pub console_enable: bool,
    pub console_level: Level,
    console_level_reload_handle: Box<dyn LevelFilterReloader>,
    pub file_enable: bool,
    pub file_level: Level,
    pub file_path_prefix: String,
    file_level_reload_handle: Box<dyn LevelFilterReloader>,
    pub utc_offset: UtcOffset,
}

impl Logger {
    pub fn new(console_level: Level, file_level: Level, file_path_prefix: String) -> Self {
        let format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]";
        // UtcOffset::current_local_offset() may fail on Unix/Linux in multi-threaded
        // processes. The `time` crate (since 0.3.9) returns Err(IndeterminateOffset)
        // instead of calling libc::localtime_r, because localtime_r internally uses
        // getenv("TZ") which is not thread-safe — a concurrent setenv/putenv call
        // from another thread would cause a data race (undefined behavior).
        // macOS is exempt as its localtime_r implementation is considered thread-safe.
        //
        // Currently Logger is constructed in main() before the Tokio runtime starts
        // (single-threaded), so this typically succeeds. However, to avoid a panic if
        // the initialization order changes, we fall back to UTC.
        //
        // References:
        // - https://docs.rs/time/latest/time/struct.UtcOffset.html#method.current_local_offset
        // - https://github.com/time-rs/time/issues/293
        // - https://github.com/time-rs/time/blob/main/time/src/utc_offset.rs
        let utc_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
        let timer = OffsetTime::new(utc_offset, format_description::parse(format).unwrap());

        let (file_level_filter, file_level_reload_handle) =
            reload::Layer::new(LevelFilter::from_level(file_level));
        let directory = Path::new(&file_path_prefix).parent().unwrap();
        let file_name_prefix = Path::new(&file_path_prefix).file_name().unwrap();
        let file_appender = rolling::daily(directory, file_name_prefix);
        let (non_blocking_appender, guard) = non_blocking(file_appender);
        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_timer(timer.clone())
            .with_thread_names(true)
            .with_writer(non_blocking_appender)
            .with_filter(file_level_filter);

        let (console_level_filter, console_level_reload_handle) =
            reload::Layer::new(LevelFilter::from_level(console_level));
        let stdeer_layer = fmt::layer()
            .with_ansi(true)
            .with_file(true)
            .with_line_number(true)
            .with_timer(timer.clone())
            .with_thread_names(true)
            .with_writer(io::stderr)
            .with_filter(console_level_filter);

        tracing_subscriber::Registry::default()
            .with(file_layer)
            .with(stdeer_layer)
            .init();

        // https://docs.rs/tracing-panic/0.1.1/tracing_panic/fn.panic_hook.html
        panic::set_hook(Box::new(panic_hook));

        let logger = Self {
            guard,
            console_enable: true,
            console_level,
            console_level_reload_handle: Box::new(console_level_reload_handle),
            file_enable: true,
            file_level,
            file_path_prefix,
            file_level_reload_handle: Box::new(file_level_reload_handle),
            utc_offset,
        };
        logger.log_status("Logger initialized:");
        logger
    }

    pub fn get_guard(&self) -> &non_blocking::WorkerGuard {
        &self.guard
    }

    fn utc_offset_str(&self) -> String {
        let (h, m, _) = self.utc_offset.as_hms();
        format!("{:+03}:{:02}", h, m.unsigned_abs())
    }

    fn log_status(&self, message: &str) {
        info!(
            console_enable = self.console_enable,
            console_level = self.console_level.to_string(),
            file_enable = self.file_enable,
            file_level = self.file_level.to_string(),
            file_path_prefix = self.file_path_prefix.as_str(),
            utc_offset = self.utc_offset_str(),
            "{}",
            message
        );
    }

    fn parse_level_filter(config: &AppendersConfig) -> Result<LevelFilter, ErrorCode> {
        if config.enable {
            LevelFilter::from_str(&config.level).map_err(|err| {
                let error_code = ErrorCode::LoggerLevelParseFail(err);
                error!("{}", error_code);
                error_code
            })
        } else {
            Ok(LevelFilter::OFF)
        }
    }

    pub fn reconfig(&mut self, logger_config: LoggerConfig) -> ErrorCode {
        // Parse and validate new levels before making any changes
        let console_filter = match Self::parse_level_filter(&logger_config.console) {
            Ok(filter) => filter,
            Err(error_code) => return error_code,
        };
        let file_filter = match Self::parse_level_filter(&logger_config.file) {
            Ok(filter) => filter,
            Err(error_code) => return error_code,
        };

        let new_console_level = console_filter.into_level().unwrap_or(self.console_level);
        let new_file_level = file_filter.into_level().unwrap_or(self.file_level);

        // Log before applying changes to ensure visibility under old filter levels
        warn!(
            console_enable = logger_config.console.enable,
            console_level = new_console_level.to_string(),
            file_enable = logger_config.file.enable,
            file_level = new_file_level.to_string(),
            file_path_prefix = self.file_path_prefix.as_str(),
            utc_offset = self.utc_offset_str(),
            "Logger reconfiguring:"
        );

        // Save old console filter for potential rollback
        let old_console_filter = if self.console_enable {
            LevelFilter::from_level(self.console_level)
        } else {
            LevelFilter::OFF
        };

        // Apply filter changes
        if let Err(err) = self.console_level_reload_handle.reload(console_filter) {
            let error_code = ErrorCode::LoggerLevelReloadFail(err);
            error!("{}", error_code);
            return error_code;
        }
        if let Err(err) = self.file_level_reload_handle.reload(file_filter) {
            if let Err(rollback_err) = self.console_level_reload_handle.reload(old_console_filter) {
                warn!("Failed to rollback console filter: {}", rollback_err);
            }
            let error_code = ErrorCode::LoggerLevelReloadFail(err);
            error!("{}", error_code);
            return error_code;
        }

        // Update state only after successful reload
        self.console_enable = logger_config.console.enable;
        self.console_level = new_console_level;
        self.file_enable = logger_config.file.enable;
        self.file_level = new_file_level;

        ErrorCode::Success
    }
}

impl Default for Logger {
    fn default() -> Self {
        let file_path_prefix = format!("log/{}.log", env!("CARGO_PKG_NAME")).replace('-', "_");
        Self::new(Level::DEBUG, Level::DEBUG, file_path_prefix)
    }
}
