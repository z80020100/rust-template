// crates.io
use config::Config;
use serde::Deserialize;

// This library
use crate::error::ErrorCode;
use crate::logger::*; // debug, error, info, trace, warn

#[derive(Debug, Deserialize)]
pub struct MainConfig {
    #[serde(default)]
    pub threads: ThreadsConfig,
    pub logger: LoggerConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ThreadsConfig {
    pub shutdown_timeout_secs: u64,
}

impl Default for ThreadsConfig {
    fn default() -> Self {
        Self {
            shutdown_timeout_secs: 10,
        }
    }
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

pub fn init() -> Result<MainConfig, ErrorCode> {
    match Config::builder()
        .add_source(config::File::with_name("configs/main.toml"))
        .build()
    {
        Ok(config_builder) => match config_builder.try_deserialize() {
            Ok(main_config) => Ok(main_config),
            Err(config_error) => {
                let error_code = ErrorCode::ConfigLoadFail(config_error);
                error!("{}", error_code);
                Err(error_code)
            }
        },
        Err(config_error) => {
            let error_code = ErrorCode::ConfigLoadFail(config_error);
            error!("{}", error_code);
            Err(error_code)
        }
    }
}
