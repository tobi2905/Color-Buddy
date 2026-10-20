//! `#[tauri::command]` functions.

use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

use crate::export;
use crate::sampler::{self, Sampler, SamplerStatus};
use crate::storage::{self, Storage};

#[tauri::command]
pub fn get_color_info(hex: String) -> Result<color_core::ColorInfo, String> {
    Ok(color_core::color_info(parse(&hex)?))
}

#[tauri::command]
pub fn get_color_details(
    hex: String,
    profile: color_core::simulate::VisionProfile,
) -> Result<color_core::ColorDetails, String> {
    Ok(color_core::color_details(parse(&hex)?, profile))
}

/// One palette per scheme.
#[tauri::command]
pub fn generate_palettes(base_hex: String) -> Result<Vec<color_core::palette::Palette>, String> {
    Ok(color_core::palette::generate_palettes(parse(&base_hex)?))
}

/// `roles` (optional, one per colour) marks background/surface pairs as subtle.
#[tauri::command]
pub fn check_palette(
    hexes: Vec<String>,
    roles: Option<Vec<color_core::palette::Role>>,
) -> Result<Vec<color_core::check::PairCheck>, String> {
    let colors = hexes
        .iter()
        .map(|h| parse(h))
        .collect::<Result<Vec<_>, _>>()?;
    if roles.as_ref().is_some_and(|r| r.len() != colors.len()) {
        return Err("roles must have one entry per colour".into());
    }
    Ok(color_core::check::check_palette(&colors, roles.as_deref()))
}

fn parse(hex: &str) -> Result<color_core::Rgb, String> {
    color_core::Rgb::from_hex(hex).ok_or_else(|| format!("not a hex colour: {hex}"))
}

/// Best matches first, at most 10.
#[tauri::command]
pub fn search_color_by_name(query: String) -> Vec<color_core::ColorName> {
    color_core::search_color_by_name(&query, 10)
}

/// Opened from the main window ("Pick from screen"): focus returns there afterwards.
#[tauri::command]
pub fn start_picker(app: AppHandle) {
    crate::picker::open_from_main(&app, Instant::now());
}

#[tauri::command]
pub fn stop_picker(app: AppHandle) {
    crate::picker::close(&app);
}

/// Freeze on the colour under the cursor: copies its hex and adds it to history.
#[tauri::command]
pub fn freeze_pick(app: AppHandle) -> Result<color_core::ColorInfo, String> {
    crate::picker::freeze(&app).map(|hover| hover.info)
}

/// Shift the sampled point by whole physical pixels (arrow keys in the picker).
#[tauri::command]
pub fn nudge_cursor(app: AppHandle, dx: i32, dy: i32) -> Result<(), String> {
    crate::picker::nudge(&app, dx.clamp(-10, 10), dy.clamp(-10, 10))
}

/// Newest first.
#[tauri::command]
pub fn list_history(
    limit: Option<i64>,
    storage: State<'_, Storage>,
) -> Result<Vec<storage::HistoryEntry>, String> {
    storage.list_history(limit.unwrap_or(storage::HISTORY_CAP))
}

#[tauri::command]
pub fn delete_history_entry(id: i64, storage: State<'_, Storage>) -> Result<(), String> {
    storage.delete_history_entry(id)
}

#[tauri::command]
pub fn clear_history(storage: State<'_, Storage>) -> Result<(), String> {
    storage.clear_history()
}

// --- settings ---------------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings(storage: State<'_, Storage>) -> Result<storage::StoredSettings, String> {
    storage.settings()
}

/// Save all settings; other windows (the picker) get `settings-changed`.
#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    settings: storage::Settings,
    storage: State<'_, Storage>,
) -> Result<(), String> {
    storage.save_settings(&settings)?;
    let _ = app.emit("settings-changed", &settings);
    Ok(())
}

/// Defaults for everything except "welcome steps done".
#[tauri::command]
pub fn reset_settings(
    app: AppHandle,
    storage: State<'_, Storage>,
) -> Result<storage::Settings, String> {
    let settings = storage.reset_settings()?;
    let _ = app.emit("settings-changed", &settings);
    Ok(settings)
}

// --- palettes and export ------------------------------------------------------------------------

#[tauri::command]
pub fn save_palette(
    palette: storage::PaletteInput,
    storage: State<'_, Storage>,
) -> Result<storage::SavedPalette, String> {
    storage.save_palette(&palette)
}

#[tauri::command]
pub fn list_palettes(storage: State<'_, Storage>) -> Result<Vec<storage::SavedPalette>, String> {
    storage.list_palettes()
}

#[tauri::command]
pub fn rename_palette(id: i64, name: String, storage: State<'_, Storage>) -> Result<(), String> {
    storage.rename_palette(id, &name)
}

#[tauri::command]
pub fn delete_palette(id: i64, storage: State<'_, Storage>) -> Result<(), String> {
    storage.delete_palette(id)
}

/// A saved palette (`id`) or one given inline (`palette`), as text in `format`.
fn export_text(
    storage: &Storage,
    id: Option<i64>,
    palette: Option<export::ExportPalette>,
    format: export::Format,
    name_level: Option<export::NameLevel>,
) -> Result<(String, String), String> {
    let palette = match (id, palette) {
        (_, Some(p)) => p,
        (Some(id), None) => {
            let saved = storage.palette(id)?;
            export::ExportPalette {
                name: saved.name,
                scheme: saved.scheme,
                base_hex: saved.base_hex,
                colors: saved
                    .colors
                    .into_iter()
                    .map(|c| export::ExportColor {
                        hex: c.hex,
                        role: c.role,
                    })
                    .collect(),
            }
        }
        (None, None) => return Err("Nothing to export.".into()),
    };
    let text = export::export(&palette, format, name_level.unwrap_or_default())?;
    Ok((text, export::slug(&palette.name)))
}

#[tauri::command]
pub fn export_palette(
    id: Option<i64>,
    palette: Option<export::ExportPalette>,
    format: export::Format,
    name_level: Option<export::NameLevel>,
    storage: State<'_, Storage>,
) -> Result<String, String> {
    export_text(&storage, id, palette, format, name_level).map(|(text, _)| text)
}

/// Ask where to save, then write the export. Returns the path, or None if cancelled.
#[tauri::command]
pub async fn export_palette_to_file(
    app: AppHandle,
    id: Option<i64>,
    palette: Option<export::ExportPalette>,
    format: export::Format,
    name_level: Option<export::NameLevel>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (text, slug) = export_text(&app.state::<Storage>(), id, palette, format, name_level)?;
    let ext = export::extension(format);
    let file_name = match format {
        export::Format::Tailwind => format!("{slug}.tailwind.{ext}"),
        _ => format!("{slug}.{ext}"),
    };
    let Some(path) = app
        .dialog()
        .file()
        .set_file_name(&file_name)
        .add_filter(ext.to_uppercase(), &[ext])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = path.into_path().map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| format!("couldn't save the file: {e}"))?;
    Ok(Some(path.display().to_string()))
}

#[tauri::command]
pub fn copy_to_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

// --- Phase 1 debug commands (temporary) ------------------------------------------------------

#[tauri::command]
pub fn set_sample_size(size: u32, sampler: State<'_, Sampler>) -> Result<(), String> {
    sampler.set_size(size)
}

#[tauri::command]
pub fn get_sampler_status(sampler: State<'_, Sampler>) -> SamplerStatus {
    sampler.status()
}

#[tauri::command]
pub fn open_screen_recording_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// A test patch drawn in the debug window: expected colour and centre in CSS pixels
/// relative to the webview's top-left corner.
#[derive(Debug, Deserialize)]
pub struct Patch {
    pub hex: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchResult {
    pub expected: String,
    pub size: u32,
    pub center: String,
    /// Every pixel of the N×N sample equals the expected colour.
    pub all_match: bool,
    /// Largest per-channel difference over the N×N sample.
    pub max_channel_diff: u8,
}

/// Samples each patch at sizes 1, 3 and 5 at its exact screen position.
#[tauri::command]
pub fn self_test(window: WebviewWindow, patches: Vec<Patch>) -> Result<Vec<PatchResult>, String> {
    if !sampler::has_permission() {
        sampler::request_permission();
        return Err(sampler::SampleError::PermissionDenied.to_string());
    }
    let pos = window.inner_position().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for patch in patches {
        let expected = color_core::Rgb::from_hex(&patch.hex)
            .ok_or_else(|| format!("bad hex {}", patch.hex))?;
        // Convert to the sampler's platform-native global coordinates.
        #[cfg(target_os = "macos")]
        let (gx, gy) = (
            pos.x as f64 / scale + patch.x,
            pos.y as f64 / scale + patch.y,
        );
        #[cfg(not(target_os = "macos"))]
        let (gx, gy) = (
            pos.x as f64 + patch.x * scale,
            pos.y as f64 + patch.y * scale,
        );

        for size in sampler::SIZES {
            let cap = sampler::capture_at(gx, gy, size).map_err(|e| e.to_string())?;
            let max_channel_diff = cap
                .pixels
                .iter()
                .map(|p| {
                    p.r.abs_diff(expected.r)
                        .max(p.g.abs_diff(expected.g))
                        .max(p.b.abs_diff(expected.b))
                })
                .max()
                .unwrap_or(0);
            log::info!(
                "self-test {} at ({gx:.1}, {gy:.1}) N={size}: centre {} max diff {max_channel_diff}",
                expected.to_hex(),
                cap.center().to_hex()
            );
            results.push(PatchResult {
                expected: expected.to_hex(),
                size,
                center: cap.center().to_hex(),
                all_match: max_channel_diff == 0,
                max_channel_diff,
            });
        }
    }
    Ok(results)
}

/// Replace the picker hotkey, e.g. "CommandOrControl+Shift+C".
#[tauri::command]
pub fn set_hotkey(app: AppHandle, accelerator: String) -> Result<(), String> {
    crate::hotkey::set(&app, &accelerator)
}

/// Whether screen capture is allowed (always true outside macOS).
#[tauri::command]
pub fn get_screen_permission() -> bool {
    sampler::has_permission()
}

/// Ask macOS for Screen Recording permission (shows the system prompt the first time).
#[tauri::command]
pub fn request_screen_permission() -> bool {
    sampler::request_permission()
}
