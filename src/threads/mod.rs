// Standard
use std::future::Future;

// crates.io
use parse_display::Display;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

// This library
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

pub async fn start_threads(cmd_sender: broadcast::Sender<ThreadCommand>) -> Result<(), ErrorCode> {
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
    while let Some(join_ret) = join_set.join_next().await {
        match join_ret {
            Ok(res) => {
                if let Err(err_code) = res {
                    error!(
                        "One of threads returned error: {} {{ {:#?} }}",
                        err_code.as_u8(),
                        err_code
                    );
                    if first_error.is_none() {
                        _ = stop_threads(&cmd_sender).await;
                        first_error = Some(err_code);
                    }
                }
            }
            Err(err) => {
                let error_code = ErrorCode::ThreadJoinFail(err);
                error!("{}", error_code);
                if first_error.is_none() {
                    _ = stop_threads(&cmd_sender).await;
                    first_error = Some(error_code);
                }
                break;
            }
        }
    }
    if let Some(err_code) = first_error {
        Err(err_code)
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
            let err_code = ErrorCode::MpmcChanThrCmdSendFail(err);
            error!("{}", err_code);
            Err(err_code)
        }
    }
}
