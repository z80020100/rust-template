// crates.io
use tracing::{debug, error, info, trace, warn};

// Custom library
use rust_template::logger;

fn main() {
    logger::init();
    trace!("Hello, world!");
    debug!("Hello, world!");
    info!("Hello, world!");
    warn!("Hello, world!");
    error!("Hello, world!");
}
