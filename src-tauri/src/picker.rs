//! The picker overlay: one transparent, always-on-top window per display that draws the loupe
//! next to the cursor and catches the freezing click and keys (a "full-screen catcher", decided
//! at the start of Phase 5). The windows are created hidden at startup so opening is instant.
//!
//! The overlays never appear in samples: on macOS captures only include windows below the
//! overlay, on Windows the overlay is excluded from screen capture.
//!
//! States: closed → live (sampling) → frozen (colour copied) → closed.

use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::history::History;
use crate::sampler::{self, Display, HoveredEvent, Sampler};

pub const LABEL_PREFIX: &str = "picker-";

fn label(display: &Display) -> String {
    format!("{LABEL_PREFIX}{}", display.id)
}

fn picker_windows(app: &AppHandle) -> Vec<WebviewWindow> {
    app.webview_windows()
        .into_iter()
        .filter(|(label, _)| label.starts_with(LABEL_PREFIX))
        .map(|(_, w)| w)
        .collect()
}

/// Create a hidden overlay for every display that doesn't have one, and close overlays of
/// displays that are gone (or have moved or changed size).
pub fn ensure_windows(app: &AppHandle) {
    let displays = sampler::displays();
    for w in picker_windows(app) {
        let still_there = displays
            .iter()
            .any(|d| label(d) == w.label() && matches_geometry(&w, d));
        if !still_there {
            let _ = w.destroy();
        }
    }
    for d in &displays {
        if app.get_webview_window(&label(d)).is_none()
            && let Err(err) = create_window(app, d)
        {
            log::error!(
                "could not create picker overlay for display {}: {err}",
                d.id
            );
        }
    }
}

fn matches_geometry(w: &WebviewWindow, d: &Display) -> bool {
    let (Ok(pos), Ok(size), Ok(scale)) = (w.outer_position(), w.inner_size(), w.scale_factor())
    else {
        return false;
    };
    // Compare in the platform's native units (points on macOS, physical pixels on Windows).
    let native = |v: f64| {
        if cfg!(target_os = "macos") {
            v / scale
        } else {
            v
        }
    };
    (native(pos.x as f64) - d.x).abs() < 2.0
        && (native(pos.y as f64) - d.y).abs() < 2.0
        && (native(size.width as f64) - d.width).abs() < 2.0
        && (native(size.height as f64) - d.height).abs() < 2.0
}

fn create_window(app: &AppHandle, d: &Display) -> tauri::Result<WebviewWindow> {
    let url = WebviewUrl::App(format!("picker.html?display={}", d.id).into());
    let builder = WebviewWindowBuilder::new(app, label(d), url)
        .title("Color-Buddy picker")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .focused(false)
        .visible(false)
        .visible_on_all_workspaces(true);

    // macOS positions windows in global points, matching the display list.
    #[cfg(target_os = "macos")]
    let builder = builder.position(d.x, d.y).inner_size(d.width, d.height);

    let window = builder.build()?;

    #[cfg(windows)]
    {
        use tauri::{PhysicalPosition, PhysicalSize};
        window.set_position(PhysicalPosition::new(d.x as i32, d.y as i32))?;
        window.set_size(PhysicalSize::new(d.width as u32, d.height as u32))?;
        exclude_from_capture(&window);
    }
    Ok(window)
}

#[cfg(windows)]
fn exclude_from_capture(window: &WebviewWindow) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE,
    };
    match window.hwnd() {
        Ok(hwnd) => {
            if unsafe { SetWindowDisplayAffinity(hwnd.0 as _, WDA_EXCLUDEFROMCAPTURE) } == 0 {
                log::error!("SetWindowDisplayAffinity failed; the overlay may appear in samples");
            }
        }
        Err(err) => log::error!("no HWND for picker overlay: {err}"),
    }
}

/// Above the menu bar and Dock, on every Space and over full-screen apps; returns the window
/// number so captures can exclude it.
#[cfg(target_os = "macos")]
fn raise_overlay(window: &WebviewWindow) -> Option<u32> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    const SCREEN_SAVER_LEVEL: isize = 1000;
    // canJoinAllSpaces | stationary | fullScreenAuxiliary
    const BEHAVIOR: usize = 1 | 16 | 256;
    let ptr = window.ns_window().ok()? as *mut AnyObject;
    let ns_window = unsafe { ptr.as_ref() }?;
    unsafe {
        let _: () = msg_send![ns_window, setLevel: SCREEN_SAVER_LEVEL];
        let _: () = msg_send![ns_window, setCollectionBehavior: BEHAVIOR];
        let number: isize = msg_send![ns_window, windowNumber];
        u32::try_from(number).ok()
    }
}

/// Show the overlays and start sampling. `trigger` is when the user asked (for latency).
pub fn open(app: &AppHandle, trigger: Instant) {
    let sampler = app.state::<Sampler>().inner().clone();
    if sampler.is_running() {
        return;
    }
    ensure_windows(app);
    let cursor = sampler::cursor().ok();
    for w in picker_windows(app) {
        let _ = w.show();
        #[cfg(target_os = "macos")]
        {
            let id = w.label()[LABEL_PREFIX.len()..].parse::<u32>().ok();
            if let (Some(id), Some(number)) = (id, raise_overlay(&w)) {
                sampler::set_excluded_window(id, Some(number));
            }
        }
    }
    // Keys go to the overlay under the cursor.
    let under_cursor = cursor.and_then(|(x, y)| {
        sampler::displays()
            .into_iter()
            .find(|d| d.contains(x, y))
            .and_then(|d| app.get_webview_window(&label(&d)))
    });
    if let Some(w) = under_cursor.or_else(|| picker_windows(app).into_iter().next()) {
        let _ = w.set_focus();
    }
    for w in picker_windows(app) {
        log::info!(
            "picker overlay {} visible={:?} position={:?} size={:?}",
            w.label(),
            w.is_visible(),
            w.outer_position(),
            w.inner_size()
        );
    }
    let _ = app.emit("picker-opened", ());
    sampler.start(app.clone(), trigger);
}

/// Freeze on the current colour: copy its hex, add it to history and tell the overlay.
pub fn freeze(app: &AppHandle) -> Result<HoveredEvent, String> {
    let sampler = app.state::<Sampler>();
    if !sampler.is_running() {
        return Err("the picker isn't open".into());
    }
    let hover = sampler
        .freeze()
        .ok_or("no colour sampled yet; move the cursor and try again")?;
    if let Err(err) = app.clipboard().write_text(hover.info.hex.clone()) {
        log::error!("copying {} failed: {err}", hover.info.hex);
    }
    app.state::<History>().add(&hover.info.hex, "picker");
    log::info!("picker frozen on {}", hover.info.hex);
    let _ = app.emit("picker-frozen", &hover);
    Ok(hover)
}

/// Move the cursor by whole physical pixels; resumes live sampling if frozen.
pub fn nudge(app: &AppHandle, dx: i32, dy: i32) -> Result<(), String> {
    let sampler = app.state::<Sampler>();
    if sampler.is_frozen() {
        sampler.resume();
        let _ = app.emit("picker-resumed", ());
    }
    sampler::nudge(dx, dy).map_err(|e| e.to_string())
}

pub fn close(app: &AppHandle) {
    log::info!("picker closed");
    app.state::<Sampler>().stop(app);
    for w in picker_windows(app) {
        let _ = w.hide();
    }
    let _ = app.emit("picker-closed", ());
}

/// The picker hotkey: open when closed, freeze when live, close when frozen.
pub fn on_hotkey(app: &AppHandle, pressed: Instant) {
    let sampler = app.state::<Sampler>();
    match (sampler.is_running(), sampler.is_frozen()) {
        (false, _) => open(app, pressed),
        (true, false) => {
            if let Err(err) = freeze(app) {
                log::warn!("{err}");
            }
        }
        (true, true) => close(app),
    }
}
