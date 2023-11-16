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

pub async fn start_threads(cmd_sender: broadcast::Sender<ThreadCommand>) -> ErrorCode {
    let mut error_code = ErrorCode::Undefined;
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
    while let Some(join_ret) = join_set.join_next().await {
        match join_ret {
            Ok(ret) => {
                error_code = ret;
            }
            Err(err) => {
                error_code = ErrorCode::ThreadJoinFail(err);
                error!("{}", error_code);
                break;
            }
        }
        if error_code != ErrorCode::Success {
            error!(
                "One of threads returned error: {} {{ {:#?} }}",
                error_code.as_u8(),
                error_code
            );
            _ = stop_threads(cmd_sender.clone()).await;
        }
    }
    error_code
}

pub async fn stop_threads(cmd_sender: broadcast::Sender<ThreadCommand>) -> ErrorCode {
    let error_code = match cmd_sender.send(ThreadCommand::Stop) {
        Ok(_) => {
            info!("Send command: {}", ThreadCommand::Stop);
            ErrorCode::Success
        }
        Err(err) => {
            let err_code = ErrorCode::MpmcChanThrCmdSendFail(err);
            error!("{}", err_code);
            err_code
        }
    };
    error_code
}
