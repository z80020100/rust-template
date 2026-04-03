use std::sync::Mutex;

use tauri::{AppHandle, Emitter, State};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{self, Duration};

pub struct SimulationState {
    stop_sender: Mutex<Option<broadcast::Sender<()>>>,
}

impl SimulationState {
    pub fn new() -> Self {
        Self {
            stop_sender: Mutex::new(None),
        }
    }
}

#[tauri::command]
pub async fn start(app: AppHandle, state: State<'_, SimulationState>) -> Result<(), String> {
    let (stop_tx, _) = broadcast::channel::<()>(1);
    let mut stop_rx_p = stop_tx.subscribe();
    let mut stop_rx_c = stop_tx.subscribe();
    let (data_tx, mut data_rx) = mpsc::unbounded_channel::<i32>();

    {
        let mut guard = state.stop_sender.lock().unwrap();
        if guard.is_some() {
            return Err("Already running".into());
        }
        *guard = Some(stop_tx);
    }

    // Producer
    let app_p = app.clone();
    tokio::spawn(async move {
        let mut counter = 0;
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    counter += 1;
                    let _ = app_p.emit("produce", counter);
                    let _ = data_tx.send(counter);
                }
                _ = stop_rx_p.recv() => break,
            }
        }
    });

    // Consumer
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(counter) = data_rx.recv() => {
                    let _ = app.emit("consume", counter);
                }
                _ = stop_rx_c.recv() => break,
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop(state: State<'_, SimulationState>) -> Result<(), String> {
    let mut guard = state.stop_sender.lock().unwrap();
    if let Some(sender) = guard.take() {
        let _ = sender.send(());
    }
    Ok(())
}
