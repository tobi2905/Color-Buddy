mod commands;
pub mod sampler;

use std::time::Instant;

use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

/// Ctrl+Shift+C on Windows, Cmd+Shift+C on macOS.
fn picker_shortcut() -> Shortcut {
    #[cfg(target_os = "macos")]
    let mods = Modifiers::SUPER | Modifiers::SHIFT;
    #[cfg(not(target_os = "macos"))]
    let mods = Modifiers::CONTROL | Modifiers::SHIFT;
    Shortcut::new(Some(mods), Code::KeyC)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    sampler::init();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    let pressed = Instant::now();
                    if event.state() == ShortcutState::Pressed && *shortcut == picker_shortcut() {
                        app.state::<sampler::Sampler>().toggle(app.clone(), pressed);
                    }
                })
                .build(),
        )
        .manage(sampler::Sampler::default())
        .setup(|app| {
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            if let Err(err) = app.global_shortcut().register(picker_shortcut()) {
                log::error!("could not register picker hotkey: {err}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_picker,
            commands::stop_picker,
            commands::set_sample_size,
            commands::get_sampler_status,
            commands::open_screen_recording_settings,
            commands::self_test,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
