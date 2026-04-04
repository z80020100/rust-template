// Standard
use std::process::ExitCode;

// crates.io
use tokio::runtime;
use tokio::sync::broadcast;

// Custom library
use rust_template::configs;
use rust_template::constant;
use rust_template::logger::*; // debug, error, info, trace, warn
use rust_template::threads::{self, ThreadCommand};

async fn main_async(threads_config: &threads::ThreadsConfig) -> Result<(), u8> {
    trace!("Hello, world!");
    debug!("Hello, world!");
    info!("Hello, world!");
    warn!("Hello, world!");
    error!("Hello, world!");
    let (cmd_sender, _) = broadcast::channel::<ThreadCommand>(constant::BROADCAST_CHANNEL_CAPACITY);
    threads::start_threads(cmd_sender, threads_config)
        .await
        .map_err(|e| e.as_u8())
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
    let result: Result<(), u8> = configs::init()
        .map_err(|e| e.as_u8())
        .and_then(|main_config| {
            debug!("Loaded config: \n{:#?}", main_config);
            logger.reconfig(main_config.logger).map_err(|e| e.as_u8())?;
            let threads_config = threads::ThreadsConfig::load();
            runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(main_async(&threads_config))
        });
    info!("Exit {}", app_info);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code),
    }
}
