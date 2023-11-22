// Standard
use std::io;
use std::panic;
use std::path::Path;
use std::str::FromStr;

// crates.io
use time::{format_description, UtcOffset};
use tracing::Level;
pub use tracing::{debug, error, info, trace, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_panic::panic_hook;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, reload, util::SubscriberInitExt, Layer,
    Registry,
};

// This library
use crate::configs::LoggerConfig;
use crate::error::ErrorCode;

// Type definitions
// FIXME: clippy::type_complexity
type ConsoleLevelReloadHandle = reload::Handle<
    tracing::level_filters::LevelFilter,
    tracing_subscriber::layer::Layered<
        tracing_subscriber::filter::Filtered<
            tracing_subscriber::fmt::Layer<
                tracing_subscriber::Registry,
                tracing_subscriber::fmt::format::DefaultFields,
                tracing_subscriber::fmt::format::Format<
                    tracing_subscriber::fmt::format::Full,
                    tracing_subscriber::fmt::time::OffsetTime<
                        std::vec::Vec<time::format_description::FormatItem<'static>>,
                    >,
                >,
                tracing_appender::non_blocking::NonBlocking,
            >,
            tracing_subscriber::reload::Layer<
                tracing::level_filters::LevelFilter,
                tracing_subscriber::Registry,
            >,
            tracing_subscriber::Registry,
        >,
        tracing_subscriber::Registry,
    >,
>;
type FileLevelReloadHandle = reload::Handle<LevelFilter, Registry>;
pub struct Logger {
    pub guard: non_blocking::WorkerGuard,
    pub console_enable: bool,
    pub console_level: Level,
    pub console_level_reload_handle: ConsoleLevelReloadHandle,
    pub file_enable: bool,
    pub file_level: Level,
    pub file_path_prefix: String,
    pub file_level_reload_handle: FileLevelReloadHandle,
}

impl Logger {
    pub fn new(console_level: Level, file_level: Level, file_path_prefix: String) -> Self {
        let format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]";
        let timer = OffsetTime::new(
            UtcOffset::current_local_offset().unwrap(),
            format_description::parse(format).unwrap(),
        );

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

        info!(
            console_level = console_level.to_string(),
            file_level = file_level.to_string(),
            file_path_prefix = file_path_prefix.as_str(),
            "Logger initialized:"
        );

        Self {
            guard,
            console_enable: true,
            console_level,
            console_level_reload_handle,
            file_enable: true,
            file_level,
            file_path_prefix,
            file_level_reload_handle,
        }
    }

    pub fn get_guard(&self) -> &non_blocking::WorkerGuard {
        &self.guard
    }

    pub fn reconfig(&mut self, logger_config: LoggerConfig) -> ErrorCode {
        let mut error_code = ErrorCode::Success;

        if !logger_config.console.enable {
            debug!("Disable console logger");
            if let Err(err) = self.console_level_reload_handle.reload(LevelFilter::OFF) {
                error_code = ErrorCode::LoggerLevelReloadFail(err);
                error!("{}", error_code);
                return error_code;
            }
            self.console_enable = false;
        } else {
            match LevelFilter::from_str(&logger_config.console.level) {
                Ok(console_level_filter) => {
                    if let Err(err) = self
                        .console_level_reload_handle
                        .reload(console_level_filter)
                    {
                        error_code = ErrorCode::LoggerLevelReloadFail(err);
                        error!("{}", error_code);
                        return error_code;
                    }
                    self.console_level = Level::from_str(&logger_config.console.level).unwrap();
                }
                Err(err) => {
                    error_code = ErrorCode::LoggerLevelParseFail(err);
                    error!("{}", error_code);
                    return error_code;
                }
            }
        }

        if !logger_config.file.enable {
            debug!("Disable file logger");
            if let Err(err) = self.file_level_reload_handle.reload(LevelFilter::OFF) {
                error_code = ErrorCode::LoggerLevelReloadFail(err);
                error!("{}", error_code);
                return error_code;
            }
            self.file_enable = false;
        } else {
            match LevelFilter::from_str(&logger_config.file.level) {
                Ok(file_level_filter) => {
                    if let Err(err) = self.file_level_reload_handle.reload(file_level_filter) {
                        error_code = ErrorCode::LoggerLevelReloadFail(err);
                        error!("{}", error_code);
                        return error_code;
                    }
                    self.file_level = Level::from_str(&logger_config.file.level).unwrap();
                }
                Err(err) => {
                    let error_code = ErrorCode::LoggerLevelParseFail(err);
                    error!("{}", error_code);
                    return error_code;
                }
            }
        }

        if self.console_enable && self.file_enable {
            warn!(
                console_level = self.console_level.to_string(),
                file_level = self.file_level.to_string(),
                file_path_prefix = self.file_path_prefix.as_str(),
                "Logger reconfigured:"
            );
        } else if self.console_enable {
            warn!(
                console_level = self.console_level.to_string(),
                file_enable = self.file_enable,
                "Logger reconfigured:"
            );
        } else if self.file_enable {
            warn!(
                console_enable = self.console_enable,
                file_level = self.file_level.to_string(),
                file_path_prefix = self.file_path_prefix.as_str(),
                "Logger reconfigured:"
            );
        } else {
            warn!(
                console_enable = self.console_enable,
                file_enable = self.file_enable,
                "Logger reconfigured:"
            );
        }

        error_code
    }
}

impl Default for Logger {
    fn default() -> Self {
        let file_path_prefix = format!("log/{}.log", env!("CARGO_PKG_NAME")).replace('-', "_");
        Self::new(Level::DEBUG, Level::DEBUG, file_path_prefix)
    }
}
