// crates.io
use tokio::sync::broadcast::{self, error::RecvError};
// TODO: support Windows
use tokio::signal::unix::{signal, SignalKind};

// This library
use super::ThreadCommand;
use crate::error::ErrorCode;
use crate::logger::*; // debug, error, info, trace, warn

pub async fn start(
    mut cmd_receiver: broadcast::Receiver<ThreadCommand>,
    cmd_sender: broadcast::Sender<ThreadCommand>,
) -> ErrorCode {
    let mut error_code = ErrorCode::Success;
    let mut sighup_stream = signal(SignalKind::hangup()).unwrap();
    let mut sigint_stream = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm_stream = signal(SignalKind::terminate()).unwrap();
    let mut loop_running = true;
    while loop_running {
        tokio::select! {
            Some(_) = sighup_stream.recv() => {
                warn!("Receive SIGHUP");
                error_code = super::stop_threads(cmd_sender.clone()).await;
                loop_running = false;
            }
            Some(_) = sigint_stream.recv() => {
                warn!("Receive SIGINT");
                error_code = super::stop_threads(cmd_sender.clone()).await;
                loop_running = false;
            }
            Some(_) = sigterm_stream.recv() => {
                warn!("Receive SIGTERM");
                error_code = super::stop_threads(cmd_sender.clone()).await;
                loop_running = false;
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
