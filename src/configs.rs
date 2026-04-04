// crates.io
use config::Config;
use serde::Deserialize;

// This library
use crate::error::ErrorCode;
use crate::logger::*; // debug, error, info, trace, warn

#[derive(Debug, Deserialize)]
pub struct MainConfig {
    pub logger: LoggerConfig,
}

#[derive(Debug, Deserialize)]
pub struct LoggerConfig {
    pub console: AppendersConfig,
    pub file: AppendersConfig,
}

#[derive(Debug, Deserialize)]
pub struct AppendersConfig {
    pub enable: bool,
    pub level: String,
}

fn log_config_err(config_error: config::ConfigError) -> ErrorCode {
    let error_code = ErrorCode::ConfigLoadFail(config_error);
    error!("{}", error_code);
    error_code
}

pub fn init() -> Result<MainConfig, ErrorCode> {
    Config::builder()
        .add_source(config::File::with_name("configs/main.toml"))
        .build()
        .map_err(log_config_err)?
        .try_deserialize()
        .map_err(log_config_err)
}
