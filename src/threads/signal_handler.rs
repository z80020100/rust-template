// crates.io
use tokio::sync::broadcast;
// TODO: support Windows
use tokio::signal::unix::{signal, SignalKind};

// This library
use super::ThreadCommand;
use crate::logger::*; // debug, error, info, trace, warn

pub async fn start(
    mut cmd_receiver: broadcast::Receiver<ThreadCommand>,
    cmd_sender: broadcast::Sender<ThreadCommand>,
) {
    let mut sighup_stream = signal(SignalKind::hangup()).unwrap();
    let mut sigint_stream = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm_stream = signal(SignalKind::terminate()).unwrap();
    loop {
        tokio::select! {
            Some(_) = sighup_stream.recv() => {
                warn!("Receive SIGHUP");
                super::stop_threads(cmd_sender.clone()).await;

            }
            Some(_) = sigint_stream.recv() => {
                warn!("Receive SIGINT");
                super::stop_threads(cmd_sender.clone()).await;
            }
            Some(_) = sigterm_stream.recv() => {
                warn!("Receive SIGTERM");
                super::stop_threads(cmd_sender.clone()).await;
            }
            Ok(cmd) = cmd_receiver.recv() => {
                info!("Receive command: {}", cmd);
                match cmd {
                    ThreadCommand::Stop => {
                        break;
                    }
                }
            }
        }
    }
}
