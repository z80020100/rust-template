// crates.io
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinError;

// This library
use crate::threads::ThreadCommand;

#[derive(Error, Debug)]
#[repr(u8)]
pub enum ErrorCode {
    #[error("Success")]
    Success = 0,
    #[error("Undefined")]
    Undefined,
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
}
