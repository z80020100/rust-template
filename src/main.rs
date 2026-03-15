// Standard
use std::process::ExitCode;

// crates.io
use tokio::runtime;
use tokio::sync::broadcast;

// Custom library
use rust_template::configs;
use rust_template::constant;
use rust_template::error::ErrorCode;
use rust_template::logger::*; // debug, error, info, trace, warn
use rust_template::threads::{self, *};

async fn main_async(threads_config: &configs::ThreadsConfig) -> Result<(), ErrorCode> {
    trace!("Hello, world!");
    debug!("Hello, world!");
    info!("Hello, world!");
    warn!("Hello, world!");
    error!("Hello, world!");
    let (cmd_sender, _) = broadcast::channel::<ThreadCommand>(constant::BROADCAST_CHANNEL_CAPACITY);
    threads::start_threads(cmd_sender, threads_config).await
}

fn main() -> ExitCode {
    /*
     * https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html
     * WorkerGuard should be assigned in the main function or whatever the entrypoint of the program is
     * This will ensure that the guard will be dropped during an unwinding or when main exits successfully
     */
    let mut logger = Logger::default();
    let _ground = logger.get_guard();
    let app_info = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    info!("Start {}", app_info);
    let result = configs::init().and_then(|main_config| {
        debug!("Loaded config: \n{:#?}", main_config);
        let threads_config = main_config.threads;
        logger.reconfig(main_config.logger)?;
        runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(main_async(&threads_config))
    });
    info!("Exit {}", app_info);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => ExitCode::from(e.as_u8()),
    }
}
