// crates.io
use strum::EnumCount;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinError;

// This library
use super::ThreadCommand;

#[derive(Error, Debug, EnumCount)]
pub enum ThreadErrorCode {
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
    #[error("Graceful shutdown timed out")]
    ShutdownTimeout,
}

impl ThreadErrorCode {
    // Each variant maps to a unique non-zero exit code
    // Starting from 4 to avoid overlap with shared ErrorCode (1-3)
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::MpscUnboundChanI32SendFail(_) => 4,
            Self::MpscUnboundChanRecvFail => 5,
            Self::MpmcChanThrCmdSendFail(_) => 6,
            Self::MpmcChanRecvFail(_) => 7,
            Self::ThreadJoinFail(_) => 8,
            Self::ShutdownTimeout => 9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[tokio::test]
    async fn exit_codes_are_nonzero_unique_and_in_range() {
        let join_err = {
            let handle = tokio::spawn(std::future::pending::<()>());
            handle.abort();
            handle.await.unwrap_err()
        };

        let variants: Vec<ThreadErrorCode> = vec![
            ThreadErrorCode::MpscUnboundChanI32SendFail(mpsc::error::SendError(0)),
            ThreadErrorCode::MpscUnboundChanRecvFail,
            ThreadErrorCode::MpmcChanThrCmdSendFail(broadcast::error::SendError(
                ThreadCommand::Stop,
            )),
            ThreadErrorCode::MpmcChanRecvFail(broadcast::error::RecvError::Closed),
            ThreadErrorCode::ThreadJoinFail(join_err),
            ThreadErrorCode::ShutdownTimeout,
        ];

        let codes: Vec<u8> = variants.iter().map(|e| e.as_u8()).collect();

        for (variant, &code) in variants.iter().zip(&codes) {
            assert_ne!(code, 0, "{variant} must have non-zero exit code");
            assert!(code <= 125, "{variant} exit code {code} must be <= 125");
        }

        assert_eq!(
            variants.len(),
            ThreadErrorCode::COUNT,
            "Test must cover all ThreadErrorCode variants"
        );

        let unique: HashSet<u8> = codes.iter().copied().collect();
        assert_eq!(
            codes.len(),
            unique.len(),
            "Exit codes must be unique: {codes:?}"
        );
    }
}
