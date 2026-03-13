// crates.io
use tokio::sync::{
    broadcast::{self, error::RecvError},
    mpsc,
};

// This library
use super::ThreadCommand;
use crate::error::ErrorCode;
use crate::logger::*; // debug, error, info, trace, warn

pub async fn start(
    mut cmd_receiver: broadcast::Receiver<ThreadCommand>,
    mut data_receiver: mpsc::UnboundedReceiver<i32>,
) -> Result<(), ErrorCode> {
    let mut loop_running = true;
    while loop_running {
        tokio::select! {
            counter = data_receiver.recv() => {
                match counter {
                    Some(counter) => {
                        info!("Consume: {}", counter);
                    }
                    None => {
                        let error_code = ErrorCode::MpscUnboundChanRecvFail;
                        error!("{}", error_code);
                        return Err(error_code);
                    }
                }
            }
            cmd = cmd_receiver.recv() => {
                loop_running = cmd_handler(cmd)?;
            }
        }
    }
    Ok(())
}

// Intentionally kept per-module for independent customization in template usage
fn cmd_handler(cmd: Result<ThreadCommand, RecvError>) -> Result<bool, ErrorCode> {
    match cmd {
        Ok(cmd) => {
            info!("Receive command: {}", cmd);
            let loop_running = match cmd {
                ThreadCommand::Stop => false,
            };
            Ok(loop_running)
        }
        Err(err) => {
            let error_code = ErrorCode::MpmcChanRecvFail(err);
            error!("{}", error_code);
            Err(error_code)
        }
    }
}
