// Prevents an extra console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let title: String = env!("CARGO_BIN_NAME")
                .split('-')
                .map(|w| {
                    let mut c = w.chars();
                    c.next()
                        .unwrap()
                        .to_uppercase()
                        .chain(c)
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join(" ");
            window.set_title(&title)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
