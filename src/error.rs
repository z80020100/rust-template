// crates.io
use strum::EnumCount;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinError;
use tracing::level_filters::ParseLevelFilterError;
use tracing_subscriber::reload::Error as TracingSubscriberReloadError;

// This library
use crate::threads::ThreadCommand;

#[derive(Error, Debug, EnumCount)]
pub enum ErrorCode {
    #[error("Failed to send data (MPSC): {0}")]
    MpscUnboundChanI32SendFail(#[from] mpsc::error::SendError<i32>),
    #[error("Failed to receive data (MPSC): channel closed")]
    MpscUnboundChanRecvFail,
    #[error("Failed to send thread command (broadcast): {0}")]
    MpmcChanThrCmdSendFail(#[from] broadcast::error::SendError<ThreadCommand>),
    #[error("Failed to receive thread command (broadcast): {0}")]
    MpmcChanRecvFail(#[from] broadcast::error::RecvError),
    #[error("Failed to join thread: {0}")]
    ThreadJoinFail(#[from] JoinError),
    #[error("Failed to load config: {0}")]
    ConfigLoadFail(#[from] config::ConfigError),
    #[error("Failed to configure logger: {0}")]
    LoggerLevelParseFail(#[from] ParseLevelFilterError),
    #[error("Failed to configure logger: {0}")]
    LoggerLevelReloadFail(#[from] TracingSubscriberReloadError),
    #[error("Graceful shutdown timed out")]
    ShutdownTimeout,
}

impl ErrorCode {
    // Each variant maps to a unique non-zero exit code
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::MpscUnboundChanI32SendFail(_) => 1,
            Self::MpscUnboundChanRecvFail => 2,
            Self::MpmcChanThrCmdSendFail(_) => 3,
            Self::MpmcChanRecvFail(_) => 4,
            Self::ThreadJoinFail(_) => 5,
            Self::ConfigLoadFail(_) => 6,
            Self::LoggerLevelParseFail(_) => 7,
            Self::LoggerLevelReloadFail(_) => 8,
            Self::ShutdownTimeout => 9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{Registry, reload};

    #[tokio::test]
    async fn exit_codes_are_nonzero_unique_and_in_range() {
        let join_err = {
            let handle = tokio::spawn(std::future::pending::<()>());
            handle.abort();
            handle.await.unwrap_err()
        };

        let reload_err = {
            let (layer, handle) = reload::Layer::<LevelFilter, Registry>::new(LevelFilter::INFO);
            drop(layer);
            handle.reload(LevelFilter::DEBUG).unwrap_err()
        };

        let variants: Vec<ErrorCode> = vec![
            ErrorCode::MpscUnboundChanI32SendFail(mpsc::error::SendError(0)),
            ErrorCode::MpscUnboundChanRecvFail,
            ErrorCode::MpmcChanThrCmdSendFail(broadcast::error::SendError(ThreadCommand::Stop)),
            ErrorCode::MpmcChanRecvFail(broadcast::error::RecvError::Closed),
            ErrorCode::ThreadJoinFail(join_err),
            ErrorCode::ConfigLoadFail(config::ConfigError::Message("test".into())),
            ErrorCode::LoggerLevelParseFail("invalid".parse::<LevelFilter>().unwrap_err()),
            ErrorCode::LoggerLevelReloadFail(reload_err),
            ErrorCode::ShutdownTimeout,
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
