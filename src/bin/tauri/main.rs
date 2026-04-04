// Prevents an extra console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Standard
use std::process::ExitCode;

// crates.io
use tauri::Manager;

// Custom library
use rust_template::configs;
use rust_template::logger::*; // debug, error, info, trace, warn

mod commands;

fn main() -> ExitCode {
    let mut logger = Logger::default();
    let _guard = logger.get_guard();
    let app_info = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    info!("Start {}", app_info);
    let result: Result<(), u8> = configs::init()
        .map_err(|e| e.as_u8())
        .and_then(|main_config| {
            debug!("Loaded config: \n{:#?}", main_config);
            logger.reconfig(main_config.logger).map_err(|e| e.as_u8())?;
            tauri::Builder::default()
                .manage(commands::SimulationState::new())
                .invoke_handler(tauri::generate_handler![commands::start, commands::stop])
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
                .map_err(|e| {
                    error!("Tauri runtime error: {}", e);
                    10u8 // ErrorCode uses 1-3 and ThreadErrorCode uses 4-9
                })
        });
    info!("Exit {}", app_info);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code),
    }
}
