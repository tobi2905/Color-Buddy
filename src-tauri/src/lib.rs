mod commands;
mod export;
mod hotkey;
mod picker;
pub mod sampler;
mod storage;

use std::time::Instant;

use tauri::Manager;
use tauri_plugin_global_shortcut::ShortcutState;

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
                    if event.state() == ShortcutState::Pressed
                        && app.state::<hotkey::Hotkey>().matches(shortcut)
                    {
                        picker::on_hotkey(app, pressed);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(sampler::Sampler::default())
        .manage(hotkey::Hotkey::default())
        .setup(|app| {
            let db_path = app.path().app_data_dir()?.join("app.db");
            let storage = storage::Storage::open(&db_path).map_err(|e| {
                log::error!("opening {}: {e}", db_path.display());
                e
            })?;
            app.manage(storage);
            // Dev aid for Phase 1 measurements: keep the debug window (and its test patches) visible.
            if std::env::var("COLOR_BUDDY_ON_TOP").as_deref() == Ok("1")
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.set_always_on_top(true);
            }
            picker::ensure_windows(app.handle());
            // Dev aid: `COLOR_BUDDY_AUTOSTART=<seconds>` opens the picker for that long after launch.
            if let Some(secs) = std::env::var("COLOR_BUDDY_AUTOSTART")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
            {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    let h = handle.clone();
                    let _ = handle.run_on_main_thread(move || picker::open(&h, Instant::now()));
                    std::thread::sleep(std::time::Duration::from_secs_f64(secs));
                    let h = handle.clone();
                    let _ = handle.run_on_main_thread(move || picker::close(&h));
                });
            }
            // The frontend applies the user's saved shortcut via `set_hotkey` on start.
            hotkey::register_default(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_color_info,
            commands::color_values,
            commands::search_color_by_name,
            commands::get_color_details,
            commands::generate_palettes,
            commands::check_palette,
            commands::start_picker,
            commands::stop_picker,
            commands::freeze_pick,
            commands::nudge_cursor,
            commands::list_history,
            commands::delete_history_entry,
            commands::clear_history,
            commands::get_settings,
            commands::update_settings,
            commands::reset_settings,
            commands::save_palette,
            commands::list_palettes,
            commands::rename_palette,
            commands::delete_palette,
            commands::export_palette,
            commands::export_palette_to_file,
            commands::copy_to_clipboard,
            commands::set_sample_size,
            commands::get_sampler_status,
            commands::open_screen_recording_settings,
            commands::self_test,
            commands::set_hotkey,
            commands::get_screen_permission,
            commands::request_screen_permission,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
