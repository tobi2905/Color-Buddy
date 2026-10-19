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
            // Dev aid for Phase 1 measurements: keep the debug window (and its test patches) visible.
            if std::env::var("COLOR_BUDDY_ON_TOP").as_deref() == Ok("1")
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.set_always_on_top(true);
            }
            // Dev aid: `COLOR_BUDDY_AUTOSTART=<seconds>` samples for that long after launch.
            if let Some(secs) = std::env::var("COLOR_BUDDY_AUTOSTART")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
            {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    let sampler = handle.state::<sampler::Sampler>();
                    sampler.start(handle.clone(), Instant::now());
                    std::thread::sleep(std::time::Duration::from_secs_f64(secs));
                    sampler.stop(&handle);
                });
            }
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
