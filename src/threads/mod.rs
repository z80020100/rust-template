// Standard
use std::future::Future;

// crates.io
use parse_display::Display;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

// This library
use crate::logger::*; // debug, error, info, trace, warn

// Threads
mod consumer;
mod producer;

#[derive(Clone, Display)]
#[display("{}_thread", style = "snake_case")]
pub enum ThreadName {
    Consumer,
    Producer,
}

#[derive(Clone, Display)]
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

pub async fn start_threads(cmd_sender: broadcast::Sender<ThreadCommand>) {
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
    while let Some(join_ret) = join_set.join_next().await {
        match join_ret {
            Ok(ret) => info!("Thread return value: {:?}", ret),
            Err(err) => info!("Thread join error: {:?}", err),
        }
    }
}

pub async fn stop_threads(cmd_sender: broadcast::Sender<ThreadCommand>) {
    if let Err(err) = cmd_sender.send(ThreadCommand::Stop) {
        error!("Send error: {}", err);
    }
}
