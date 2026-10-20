//! Windows backend: GDI region capture via `xcap`, in physical pixels.
//!
//! Requires per-monitor DPI awareness (Tauri sets PerMonitorV2 at startup; `init` does it for
//! standalone tools) so cursor and monitor coordinates are physical on every monitor.
//! Windows does not colour-manage the desktop, so captured values are the sRGB values apps
//! rendered. We only detect and log a non-sRGB display profile (no correction yet).

use std::collections::HashMap;
use std::sync::Mutex;

use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::Graphics::Gdi::{
    CreateDCW, DeleteDC, GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MONITORINFOEXW,
    MonitorFromPoint,
};
use windows_sys::Win32::UI::ColorSystem::GetICMProfileW;
use windows_sys::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

use super::{Capture, Display, DisplayInfo, SampleError, crop_rgba, window_start};

/// Picker overlays are excluded from capture with `SetWindowDisplayAffinity` instead.
pub fn set_excluded_window(_display: u32, _window: Option<u32>) {}

/// Monitors in physical virtual-screen pixels.
pub fn displays() -> Vec<Display> {
    xcap::Monitor::all()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|m| {
            Some(Display {
                id: m.id().ok()?,
                x: m.x().ok()? as f64,
                y: m.y().ok()? as f64,
                width: m.width().ok()? as f64,
                height: m.height().ok()? as f64,
                css_per_native: 1.0 / m.scale_factor().ok()? as f64,
                physical_per_native: 1.0,
            })
        })
        .collect()
}

pub fn init() {
    // Fails harmlessly if awareness was already set (e.g. by Tauri).
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

pub fn has_permission() -> bool {
    true
}

pub fn request_permission() -> bool {
    true
}

/// Global cursor position in physical pixels (virtual-screen coordinates).
pub fn cursor() -> Result<(f64, f64), SampleError> {
    let mut pt = POINT { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut pt) } == 0 {
        return Err(SampleError::Capture("GetCursorPos failed".into()));
    }
    Ok((pt.x as f64, pt.y as f64))
}

pub fn capture_at(x: f64, y: f64, size: u32) -> Result<Capture, SampleError> {
    let (x, y) = (x.floor() as i32, y.floor() as i32);
    let err = |e: xcap::XCapError| SampleError::Capture(e.to_string());
    let monitor = xcap::Monitor::from_point(x, y).map_err(|_| SampleError::NoDisplay)?;
    let (mx, my) = (monitor.x().map_err(err)?, monitor.y().map_err(err)?);
    let (mw, mh) = (
        monitor.width().map_err(err)?,
        monitor.height().map_err(err)?,
    );

    let sx = window_start((x - mx) as i64, size, mw);
    let sy = window_start((y - my) as i64, size, mh);
    let image = monitor.capture_region(sx, sy, size, size).map_err(err)?;

    let id = monitor.id().map_err(err)?;
    let scale = monitor.scale_factor().map_err(err)? as f64;
    let (profile, is_srgb) = display_profile(x, y);
    Ok(Capture {
        cursor: (x as f64, y as f64),
        size,
        pixels: crop_rgba(image.as_raw(), image.width(), 0, 0, size),
        display: DisplayInfo {
            id,
            scale,
            profile,
            is_srgb,
            converted_to_srgb: false,
        },
    })
}

/// ICC profile file name of the monitor containing `(x, y)`, cached per monitor device.
fn display_profile(x: i32, y: i32) -> (String, bool) {
    static CACHE: Mutex<Option<HashMap<String, (String, bool)>>> = Mutex::new(None);

    let hmonitor = unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST) };
    let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    if unsafe { GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut MONITORINFO) } == 0 {
        return ("unknown".into(), false);
    }
    let device = String::from_utf16_lossy(&info.szDevice)
        .trim_end_matches('\0')
        .to_string();

    let mut cache = CACHE.lock().unwrap();
    let cache = cache.get_or_insert_with(HashMap::new);
    if let Some(hit) = cache.get(&device) {
        return hit.clone();
    }

    let profile = unsafe {
        let hdc = CreateDCW(
            info.szDevice.as_ptr(),
            info.szDevice.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
        );
        let mut buf = [0u16; 260];
        let mut len = buf.len() as u32;
        let ok = !hdc.is_null() && GetICMProfileW(hdc, &mut len, buf.as_mut_ptr()) != 0;
        if !hdc.is_null() {
            DeleteDC(hdc);
        }
        ok.then(|| {
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            let path = path.trim_end_matches('\0');
            path.rsplit(['\\', '/']).next().unwrap_or(path).to_string()
        })
    };
    let entry = match profile {
        Some(name) => {
            let is_srgb = name.to_ascii_lowercase().contains("srgb");
            (name, is_srgb)
        }
        None => ("unknown".into(), false),
    };
    cache.insert(device, entry.clone());
    entry
}
