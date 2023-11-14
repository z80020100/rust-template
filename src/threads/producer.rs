// crates.io
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

// This library
use crate::logger::*; // debug, error, info, trace, warn

pub async fn start(sender: mpsc::UnboundedSender<i32>) {
    let mut counter = 0;
    loop {
        time::sleep(Duration::from_secs(1)).await;
        counter += 1;
        info!("Produce: {}", counter);
        if let Err(err) = sender.send(counter) {
            error!("Send error: {}", err);
        }
    }
}
