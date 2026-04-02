// crates.io
use strum::EnumCount;
use thiserror::Error;
use tracing::level_filters::ParseLevelFilterError;
use tracing_subscriber::reload::Error as TracingSubscriberReloadError;

#[derive(Error, Debug, EnumCount)]
pub enum ErrorCode {
    #[error("Failed to load config: {0}")]
    ConfigLoadFail(#[from] config::ConfigError),
    #[error("Failed to configure logger: {0}")]
    LoggerLevelParseFail(#[from] ParseLevelFilterError),
    #[error("Failed to configure logger: {0}")]
    LoggerLevelReloadFail(#[from] TracingSubscriberReloadError),
}

impl ErrorCode {
    // Each variant maps to a unique non-zero exit code (1..=3)
    // ThreadErrorCode uses 4..=9 — ranges must not overlap
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::ConfigLoadFail(_) => 1,
            Self::LoggerLevelParseFail(_) => 2,
            Self::LoggerLevelReloadFail(_) => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{Registry, reload};

    #[test]
    fn exit_codes_are_nonzero_unique_and_in_range() {
        let reload_err = {
            let (layer, handle) = reload::Layer::<LevelFilter, Registry>::new(LevelFilter::INFO);
            drop(layer);
            handle.reload(LevelFilter::DEBUG).unwrap_err()
        };

        let variants: Vec<ErrorCode> = vec![
            ErrorCode::ConfigLoadFail(config::ConfigError::Message("test".into())),
            ErrorCode::LoggerLevelParseFail("invalid".parse::<LevelFilter>().unwrap_err()),
            ErrorCode::LoggerLevelReloadFail(reload_err),
        ];

        let codes: Vec<u8> = variants.iter().map(|e| e.as_u8()).collect();

        for (variant, &code) in variants.iter().zip(&codes) {
            assert_ne!(code, 0, "{variant} must have non-zero exit code");
            assert!(code <= 125, "{variant} exit code {code} must be <= 125");
        }

        assert_eq!(
            variants.len(),
            ErrorCode::COUNT,
            "Test must cover all ErrorCode variants"
        );

        let unique: HashSet<u8> = codes.iter().copied().collect();
        assert_eq!(
            codes.len(),
            unique.len(),
            "Exit codes must be unique: {codes:?}"
        );
    }
}
