use std::sync::Mutex;

#[cfg(debug_assertions)]
use tauri::WebviewWindow;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};

struct SimulationInner {
    stop_sender: broadcast::Sender<()>,
    handles: Vec<JoinHandle<()>>,
}

pub struct SimulationState {
    inner: Mutex<Option<SimulationInner>>,
}

impl SimulationState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn open_devtools(window: WebviewWindow) {
    window.open_devtools();
}

#[tauri::command]
pub async fn start(app: AppHandle, state: State<'_, SimulationState>) -> Result<(), String> {
    let (stop_tx, _) = broadcast::channel::<()>(1);
    let mut stop_rx_p = stop_tx.subscribe();
    let mut stop_rx_c = stop_tx.subscribe();
    let (data_tx, mut data_rx) = mpsc::unbounded_channel::<i32>();

    let mut guard = state.inner.lock().unwrap();
    if guard.is_some() {
        return Err("Already running".into());
    }

    // Producer
    let app_p = app.clone();
    let producer = tokio::spawn(async move {
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
    let consumer = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(counter) = data_rx.recv() => {
                    let _ = app.emit("consume", counter);
                }
                _ = stop_rx_c.recv() => break,
            }
        }
    });

    *guard = Some(SimulationInner {
        stop_sender: stop_tx,
        handles: vec![producer, consumer],
    });

    Ok(())
}

#[tauri::command]
pub async fn stop(state: State<'_, SimulationState>) -> Result<(), String> {
    let inner = {
        let mut guard = state.inner.lock().unwrap();
        guard.take()
    };

    if let Some(inner) = inner {
        let _ = inner.stop_sender.send(());
        for handle in inner.handles {
            let _ = handle.await;
        }
    }

    Ok(())
}
