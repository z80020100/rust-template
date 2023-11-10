// crates.io
use tracing::{debug, error, info, trace, warn};

// Custom library
use rust_template::logger;

fn main() {
    /*
     * https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html
     * WorkerGuard should be assigned in the main function or whatever the entrypoint of the program is
     * This will ensure that the guard will be dropped during an unwinding or when main exits successfully
     */
    let _ground = logger::init();
    trace!("Hello, world!");
    debug!("Hello, world!");
    info!("Hello, world!");
    warn!("Hello, world!");
    error!("Hello, world!");
}
