// crates.io
use cfg_if::cfg_if;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
#[cfg(windows)]
use tokio::signal::windows::{ctrl_break, ctrl_c, ctrl_close};
use tokio::sync::broadcast::{self, error::RecvError};

// This library
use super::ThreadCommand;
use super::error::ThreadErrorCode;
use crate::logger::*; // debug, error, info, trace, warn

pub async fn start(
    mut cmd_receiver: broadcast::Receiver<ThreadCommand>,
    cmd_sender: broadcast::Sender<ThreadCommand>,
) -> Result<(), ThreadErrorCode> {
    let mut loop_running = true;
    while loop_running {
        tokio::select! {
            is_supported = signal_handler() => {
                match is_supported {
                    Some(_) => {
                        if let Err(err) = super::stop_threads(&cmd_sender).await {
                            error!("Failed to stop threads from signal handler: {}", err);
                        }
                        loop_running = false;
                    }
                    None => {
                        loop_running = false;
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
fn cmd_handler(cmd: Result<ThreadCommand, RecvError>) -> Result<bool, ThreadErrorCode> {
    match cmd {
        Ok(cmd) => {
            info!("Receive command: {}", cmd);
            let loop_running = match cmd {
                ThreadCommand::Stop => false,
            };
            Ok(loop_running)
        }
        Err(err) => {
            let error_code = ThreadErrorCode::MpmcChanRecvFail(err);
            error!("{}", error_code);
            Err(error_code)
        }
    }
}

async fn signal_handler() -> Option<()> {
    cfg_if! {
        if #[cfg(unix)] {
            signal_handler_unix().await;
            Some(())
        } else if #[cfg(windows)] {
            signal_handler_windows().await;
            Some(())
        }
        else {
            warn!("Signal handler is not supported on \"{}\"", std::env::consts::OS);
            None
        }
    }
}

#[cfg(unix)]
async fn signal_handler_unix() {
    let mut sighup_stream = signal(SignalKind::hangup()).unwrap();
    let mut sigint_stream = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm_stream = signal(SignalKind::terminate()).unwrap();
    tokio::select! {
        Some(_) = sighup_stream.recv() => {
            warn!("Receive SIGHUP");
        }
        Some(_) = sigint_stream.recv() => {
            warn!("Receive SIGINT");
        }
        Some(_) = sigterm_stream.recv() => {
            warn!("Receive SIGTERM");
        }
    }
}

#[cfg(windows)]
async fn signal_handler_windows() {
    let mut signal_ctrl_c = ctrl_c().unwrap();
    let mut signal_ctrl_break = ctrl_break().unwrap();
    let mut signal_ctrl_close = ctrl_close().unwrap();
    tokio::select! {
        Some(_) = signal_ctrl_c.recv() => {
            warn!("Receive CTRL+C");
        }
        Some(_) = signal_ctrl_break.recv() => {
            warn!("Receive CTRL+BREAK");
        }
        Some(_) = signal_ctrl_close.recv() => {
            warn!("Receive CTRL+CLOSE");
        }
    }
}
