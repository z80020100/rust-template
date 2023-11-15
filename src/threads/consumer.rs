// crates.io
use tokio::sync::{broadcast, mpsc};

// This library
use super::ThreadCommand;
use crate::logger::*; // debug, error, info, trace, warn

pub async fn start(
    mut cmd_receiver: broadcast::Receiver<ThreadCommand>,
    mut data_receiver: mpsc::UnboundedReceiver<i32>,
) {
    loop {
        tokio::select! {
            Some(counter) = data_receiver.recv() => {
                info!("Consume: {}", counter);
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
