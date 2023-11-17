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
) -> ErrorCode {
    let mut error_code = ErrorCode::Undefined;
    let mut loop_running = true;
    while loop_running {
        tokio::select! {
            counter = data_receiver.recv() => {
                match counter {
                    Some(counter) => {
                        error_code = ErrorCode::Success;
                        info!("Consume: {}", counter);
                    }
                    None => {
                        error_code = ErrorCode::MpscUnboundChanRecvFail;
                        error!("{}", error_code);
                        loop_running = false;
                    }
                }
            }
            cmd = cmd_receiver.recv() => {
                match cmd_handler(cmd) {
                    Ok(running) => {
                        loop_running = running;
                    }
                    Err(err) => {
                        error_code = err;
                        loop_running = false;
                    }
                }
            }
        }
    }
    error_code
}

fn cmd_handler(cmd: Result<ThreadCommand, RecvError>) -> Result<bool, ErrorCode> {
    match cmd {
        Ok(cmd) => {
            info!("Receive command: {}", cmd);
            let loop_runing = match cmd {
                ThreadCommand::Stop => false,
            };
            Ok(loop_runing)
        }
        Err(err) => {
            let error_code = ErrorCode::MpmcChanRecvFail(err);
            error!("{}", error_code);
            Err(error_code)
        }
    }
}
