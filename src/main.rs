// crates.io
use tokio::runtime;
use tokio::sync::broadcast;

// Custom library
use rust_template::constant;
use rust_template::logger::{self, *}; // debug, error, info, trace, warn
use rust_template::threads::{self, *};

async fn main_async() {
    trace!("Hello, world!");
    debug!("Hello, world!");
    info!("Hello, world!");
    warn!("Hello, world!");
    error!("Hello, world!");
    let (cmd_sender, _) = broadcast::channel::<ThreadCommand>(constant::BROADCAST_CHANNEL_CAPACITY);
    threads::start_threads(cmd_sender).await;
}

fn main() {
    /*
     * https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html
     * WorkerGuard should be assigned in the main function or whatever the entrypoint of the program is
     * This will ensure that the guard will be dropped during an unwinding or when main exits successfully
     */
    let _ground = logger::init();
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main_async());
}
