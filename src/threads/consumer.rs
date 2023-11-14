// crates.io
use tokio::sync::mpsc;

// This library
use crate::logger::*; // debug, error, info, trace, warn

pub async fn start(mut reveiver: mpsc::UnboundedReceiver<i32>) {
    loop {
        if let Some(counter) = reveiver.recv().await {
            info!("Consume: {}", counter);
        }
    }
}
