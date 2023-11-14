// crates.io
use tokio::time::{self, Duration};

// This library
use crate::logger::*; // debug, error, info, trace, warn

pub async fn start() {
    loop {
        time::sleep(Duration::from_secs(1)).await;
        info!("Produce");
    }
}
