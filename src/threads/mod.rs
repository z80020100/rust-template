// Standard
use std::future::Future;

// crates.io
use parse_display::Display;
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

pub async fn start_threads() {
    let mut join_set = JoinSet::new();
    spawn_thread(&mut join_set, ThreadName::Consumer, consumer::start());
    spawn_thread(&mut join_set, ThreadName::Producer, producer::start());
    while let Some(join_ret) = join_set.join_next().await {
        match join_ret {
            Ok(ret) => info!("Thread return value: {:?}", ret),
            Err(err) => info!("Thread join error: {:?}", err),
        }
    }
}
