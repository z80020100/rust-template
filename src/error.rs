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

// https://doc.rust-lang.org/std/mem/fn.discriminant.html
impl ErrorCode {
    fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
    pub fn as_u8(&self) -> u8 {
        self.discriminant()
    }
}

impl PartialEq for ErrorCode {
    fn eq(&self, other: &Self) -> bool {
        self.discriminant() == other.discriminant()
    }
}

impl Eq for ErrorCode {}
