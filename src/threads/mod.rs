// Standard
use std::future::Future;
use std::time::Duration;

// crates.io
use parse_display::Display;
use tokio::sync::{broadcast, mpsc};
use tokio::task::{JoinError, JoinSet};

// This library
use crate::configs::ThreadsConfig;
use crate::error::ErrorCode;
use crate::logger::*; // debug, error, info, trace, warn

// Threads
mod consumer;
mod producer;
mod signal_handler;

#[derive(Clone, Display)]
#[display("{}_thread", style = "snake_case")]
pub enum ThreadName {
    Consumer,
    Producer,
    SignalHandler,
}

#[derive(Clone, Debug, Display)]
pub enum ThreadCommand {
    Stop,
}

fn spawn_thread<F, R>(
    join_set: &mut JoinSet<R>,
    name: ThreadName, // TODO: set thread name if JoinSet::build_task becomes stable feature
    future: F,
) where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static, // thread return type
{
    join_set.spawn(async move {
        info!("Start {}", name);
        let ret = future.await;
        info!("End of {}", name);
        ret
    });
}

fn handle_join_result(join_ret: Result<Result<(), ErrorCode>, JoinError>) -> Option<ErrorCode> {
    match join_ret {
        Ok(Ok(())) => None,
        Ok(Err(error_code)) => {
            error!(
                "One of threads returned error: {} {{ {:#?} }}",
                error_code.as_u8(),
                error_code
            );
            Some(error_code)
        }
        Err(err) => {
            let error_code = ErrorCode::ThreadJoinFail(err);
            error!("{}", error_code);
            Some(error_code)
        }
    }
}

pub async fn start_threads(
    cmd_sender: broadcast::Sender<ThreadCommand>,
    threads_config: &ThreadsConfig,
) -> Result<(), ErrorCode> {
    let mut join_set = JoinSet::new();
    let (data_sender, data_receiver) = mpsc::unbounded_channel::<i32>();
    spawn_thread(
        &mut join_set,
        ThreadName::Consumer,
        consumer::start(cmd_sender.subscribe(), data_receiver),
    );
    spawn_thread(
        &mut join_set,
        ThreadName::Producer,
        producer::start(cmd_sender.subscribe(), data_sender),
    );
    spawn_thread(
        &mut join_set,
        ThreadName::SignalHandler,
        signal_handler::start(cmd_sender.subscribe(), cmd_sender.clone()),
    );
    let mut first_error = None;
    let timeout_duration = Duration::from_secs(threads_config.shutdown_timeout_secs);

    if let Some(join_ret) = join_set.join_next().await {
        if let Some(error_code) = handle_join_result(join_ret) {
            first_error = Some(error_code);
        }
        _ = stop_threads(&cmd_sender).await;
    }

    if !join_set.is_empty()
        && tokio::time::timeout(timeout_duration, async {
            while let Some(join_ret) = join_set.join_next().await {
                if let Some(error_code) = handle_join_result(join_ret)
                    && first_error.is_none()
                {
                    first_error = Some(error_code);
                }
            }
        })
        .await
        .is_err()
    {
        error!(
            "Graceful shutdown timed out after {}s, forcing exit",
            timeout_duration.as_secs()
        );
        join_set.abort_all();
        while join_set.join_next().await.is_some() {}
        if first_error.is_none() {
            first_error = Some(ErrorCode::ShutdownTimeout);
        }
    }

    if let Some(error_code) = first_error {
        Err(error_code)
    } else {
        Ok(())
    }
}

pub async fn stop_threads(cmd_sender: &broadcast::Sender<ThreadCommand>) -> Result<(), ErrorCode> {
    match cmd_sender.send(ThreadCommand::Stop) {
        Ok(_) => {
            info!("Send command: {}", ThreadCommand::Stop);
            Ok(())
        }
        Err(err) => {
            let error_code = ErrorCode::MpmcChanThrCmdSendFail(err);
            error!("{}", error_code);
            Err(error_code)
        }
    }
}
