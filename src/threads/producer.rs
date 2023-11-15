// crates.io
use tokio::sync::{broadcast, mpsc};
use tokio::time::{self, Duration};

// This library
use super::ThreadCommand;
use crate::logger::*; // debug, error, info, trace, warn

pub async fn start(
    mut cmd_receiver: broadcast::Receiver<ThreadCommand>,
    data_sender: mpsc::UnboundedSender<i32>,
) {
    let mut counter = 0;
    loop {
        tokio::select! {
            _ = time::sleep(Duration::from_secs(1)) => {
                counter += 1;
                info!("Produce: {}", counter);
                if let Err(err) = data_sender.send(counter) {
                    error!("Send error: {}", err);
                }
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
