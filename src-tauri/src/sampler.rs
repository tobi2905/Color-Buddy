//! Cursor + screen sampling (Phase 1 spike).
//!
//! Platform backends live in `sampler/macos.rs` and `sampler/windows.rs`. Both return the
//! N×N physical pixels centred on the cursor, as sRGB. Everything else here (window math,
//! cropping, the sampling loop and its measurements) is platform independent.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use color_core::Rgb;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "macos")]
#[path = "sampler/macos.rs"]
mod platform;
#[cfg(windows)]
#[path = "sampler/windows.rs"]
mod platform;
#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
    use super::*;
    pub fn init() {}
    pub fn has_permission() -> bool {
        true
    }
    pub fn request_permission() -> bool {
        true
    }
    pub fn cursor() -> Result<(f64, f64), SampleError> {
        Err(SampleError::Unsupported)
    }
    pub fn capture_at(_x: f64, _y: f64, _size: u32) -> Result<Capture, SampleError> {
        Err(SampleError::Unsupported)
    }
}

pub use platform::{has_permission, init, request_permission};

/// Allowed sample sizes (N in N×N).
pub const SIZES: [u32; 3] = [1, 3, 5];
pub const DEFAULT_SIZE: u32 = 5;
/// Target rate of `color-hovered` events.
pub const TARGET_HZ: f64 = 30.0;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub id: u32,
    /// Physical pixels per logical point/DIP.
    pub scale: f64,
    /// Human-readable name of the display color profile, if known.
    pub profile: String,
    pub is_srgb: bool,
    /// Whether captured pixels were converted from the display profile to sRGB.
    pub converted_to_srgb: bool,
}

#[derive(Debug, Clone)]
pub struct Capture {
    pub size: u32,
    /// Row-major, `size * size` pixels; the centre is the pixel under the cursor.
    pub pixels: Vec<Rgb>,
    pub display: DisplayInfo,
}

impl Capture {
    pub fn center(&self) -> Rgb {
        self.pixels[(self.size * self.size / 2) as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SampleError {
    /// macOS Screen Recording permission is missing.
    PermissionDenied,
    NoDisplay,
    Capture(String),
    #[allow(dead_code)]
    Unsupported,
}

impl std::fmt::Display for SampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PermissionDenied => write!(
                f,
                "Screen Recording permission is missing. Allow Color-Buddy in System Settings → \
                 Privacy & Security → Screen & System Audio Recording, then restart the app."
            ),
            Self::NoDisplay => write!(f, "No display found under the cursor."),
            Self::Capture(msg) => write!(f, "Screen capture failed: {msg}"),
            Self::Unsupported => write!(f, "Screen sampling is not supported on this platform."),
        }
    }
}

impl SampleError {
    fn kind(&self) -> &'static str {
        match self {
            Self::PermissionDenied => "permission",
            _ => "capture",
        }
    }
}

/// Current cursor position in platform-native global coordinates
/// (macOS: points, origin top-left of the main display; Windows: physical pixels).
pub fn cursor() -> Result<(f64, f64), SampleError> {
    platform::cursor()
}

/// Sample the N×N area under the cursor.
pub fn capture(size: u32) -> Result<Capture, SampleError> {
    let (x, y) = platform::cursor()?;
    platform::capture_at(x, y, size)
}

/// Sample the N×N area centred on a global point (platform-native coordinates, see [`cursor`]).
pub fn capture_at(x: f64, y: f64, size: u32) -> Result<Capture, SampleError> {
    platform::capture_at(x, y, size)
}

// ---------------------------------------------------------------------------------------------
// Pure helpers (shared by the backends, unit tested below)
// ---------------------------------------------------------------------------------------------

/// First pixel of a `size`-wide window centred on `center`, clamped to `[0, extent)`.
pub(crate) fn window_start(center: i64, size: u32, extent: u32) -> u32 {
    let size = size.min(extent) as i64;
    let start = center - size / 2;
    start.clamp(0, extent as i64 - size) as u32
}

/// The whole-point span covering physical pixels `[start_px, start_px + size)` at `scale`.
/// Returns `(first_point, point_count)`.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) fn covering_points(start_px: u32, size: u32, scale: f64) -> (u32, u32) {
    let first = (start_px as f64 / scale).floor() as u32;
    let end = ((start_px + size) as f64 / scale).ceil() as u32;
    (first, (end - first).max(1))
}

/// Crop a `size`×`size` block at `(x0, y0)` out of a tightly packed RGBA buffer `width` pixels wide.
pub(crate) fn crop_rgba(rgba: &[u8], width: u32, x0: u32, y0: u32, size: u32) -> Vec<Rgb> {
    let mut out = Vec::with_capacity((size * size) as usize);
    for y in y0..y0 + size {
        for x in x0..x0 + size {
            let i = ((y * width + x) * 4) as usize;
            out.push(Rgb::new(rgba[i], rgba[i + 1], rgba[i + 2]));
        }
    }
    out
}

/// Counts events over a sliding one-second window.
#[derive(Debug, Default)]
pub(crate) struct RateMeter {
    stamps: std::collections::VecDeque<Instant>,
}

impl RateMeter {
    pub fn tick(&mut self, now: Instant) {
        self.stamps.push_back(now);
        while let Some(&front) = self.stamps.front() {
            if now.duration_since(front) > Duration::from_secs(1) {
                self.stamps.pop_front();
            } else {
                break;
            }
        }
    }

    /// Events in the last second.
    pub fn rate(&self) -> f64 {
        self.stamps.len() as f64
    }
}

// ---------------------------------------------------------------------------------------------
// Sampling loop
// ---------------------------------------------------------------------------------------------

/// Payload of the temporary Phase 1 `color-hovered` event.
#[derive(Debug, Clone, Serialize)]
pub struct HoveredEvent {
    pub hex: String,
}

/// Payload of the `sampler-status` debug event.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplerStatus {
    pub running: bool,
    pub size: u32,
    /// Samples emitted in the last second.
    pub samples_per_second: f64,
    /// Average time of one capture call, in ms (shows headroom above the target rate).
    pub avg_capture_ms: f64,
    /// Time from the start trigger (hotkey/button) to the first sample, in ms.
    pub first_sample_ms: Option<f64>,
    pub display: Option<DisplayInfo>,
    pub error: Option<String>,
    pub error_kind: Option<String>,
}

#[derive(Default)]
struct Shared {
    running: AtomicBool,
    generation: AtomicU64,
    size: AtomicU32,
    status: Mutex<SamplerStatus>,
}

/// Starts/stops the background sampling thread. Cheap to clone.
#[derive(Clone)]
pub struct Sampler {
    shared: Arc<Shared>,
}

impl Default for Sampler {
    fn default() -> Self {
        let shared = Shared::default();
        shared.size.store(DEFAULT_SIZE, Ordering::Relaxed);
        Self {
            shared: Arc::new(shared),
        }
    }
}

impl Sampler {
    pub fn is_running(&self) -> bool {
        self.shared.running.load(Ordering::SeqCst)
    }

    pub fn status(&self) -> SamplerStatus {
        let mut status = self.shared.status.lock().unwrap().clone();
        status.running = self.is_running();
        status.size = self.shared.size.load(Ordering::Relaxed);
        status
    }

    pub fn set_size(&self, size: u32) -> Result<(), String> {
        if !SIZES.contains(&size) {
            return Err(format!("sample size must be one of {SIZES:?}"));
        }
        self.shared.size.store(size, Ordering::Relaxed);
        Ok(())
    }

    /// `trigger` is when the user asked to start (hotkey press), for latency measurement.
    pub fn start(&self, app: AppHandle, trigger: Instant) {
        if self.shared.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let generation = self.shared.generation.fetch_add(1, Ordering::SeqCst) + 1;
        *self.shared.status.lock().unwrap() = SamplerStatus::default();

        if !has_permission() {
            // Registers the app in System Settings and shows the system prompt the first time.
            request_permission();
            self.fail(&app, SampleError::PermissionDenied);
            return;
        }

        let sampler = self.clone();
        std::thread::Builder::new()
            .name("sampler".into())
            .spawn(move || sampler.run(app, generation, trigger))
            .expect("failed to spawn sampler thread");
    }

    pub fn stop(&self, app: &AppHandle) {
        self.shared.running.store(false, Ordering::SeqCst);
        self.emit_status(app);
    }

    pub fn toggle(&self, app: AppHandle, trigger: Instant) {
        if self.is_running() {
            self.stop(&app);
        } else {
            self.start(app, trigger);
        }
    }

    fn fail(&self, app: &AppHandle, err: SampleError) {
        log::error!("sampler stopped: {err}");
        self.shared.running.store(false, Ordering::SeqCst);
        {
            let mut status = self.shared.status.lock().unwrap();
            status.error = Some(err.to_string());
            status.error_kind = Some(err.kind().to_string());
        }
        self.emit_status(app);
    }

    fn emit_status(&self, app: &AppHandle) {
        let _ = app.emit("sampler-status", self.status());
    }

    fn run(&self, app: AppHandle, generation: u64, trigger: Instant) {
        let interval = Duration::from_secs_f64(1.0 / TARGET_HZ);
        let mut meter = RateMeter::default();
        let mut capture_total = Duration::ZERO;
        let mut capture_count = 0u32;
        let mut last_status = Instant::now();
        let mut last_display: Option<DisplayInfo> = None;
        let mut next = Instant::now();

        while self.shared.running.load(Ordering::SeqCst)
            && self.shared.generation.load(Ordering::SeqCst) == generation
        {
            let size = self.shared.size.load(Ordering::Relaxed);
            let started = Instant::now();
            match capture(size) {
                Ok(cap) => {
                    let now = Instant::now();
                    capture_total += now - started;
                    capture_count += 1;
                    meter.tick(now);
                    let _ = app.emit(
                        "color-hovered",
                        HoveredEvent {
                            hex: cap.center().to_hex(),
                        },
                    );

                    let mut status = self.shared.status.lock().unwrap();
                    if status.first_sample_ms.is_none() {
                        let ms = (now - trigger).as_secs_f64() * 1000.0;
                        log::info!("first sample {ms:.1} ms after trigger");
                        status.first_sample_ms = Some(ms);
                    }
                    if last_display.as_ref() != Some(&cap.display) {
                        if !cap.display.is_srgb {
                            log::warn!(
                                "display {} profile is '{}', not sRGB ({})",
                                cap.display.id,
                                cap.display.profile,
                                if cap.display.converted_to_srgb {
                                    "converting captured pixels to sRGB"
                                } else {
                                    "no correction applied"
                                }
                            );
                        }
                        last_display = Some(cap.display.clone());
                    }
                    status.display = Some(cap.display);
                    status.error = None;
                    status.error_kind = None;
                }
                Err(SampleError::PermissionDenied) => {
                    self.fail(&app, SampleError::PermissionDenied);
                    return;
                }
                Err(err) => {
                    // Transient (e.g. cursor between displays); keep going but surface it.
                    log::warn!("{err}");
                    let mut status = self.shared.status.lock().unwrap();
                    status.error = Some(err.to_string());
                    status.error_kind = Some(err.kind().to_string());
                }
            }

            if last_status.elapsed() >= Duration::from_millis(500) {
                {
                    let mut status = self.shared.status.lock().unwrap();
                    status.samples_per_second = meter.rate();
                    if capture_count > 0 {
                        status.avg_capture_ms =
                            capture_total.as_secs_f64() * 1000.0 / capture_count as f64;
                    }
                }
                self.emit_status(&app);
                last_status = Instant::now();
            }

            // Fixed-rate schedule; if we fell behind, restart the schedule instead of bursting.
            next += interval;
            let now = Instant::now();
            if next > now {
                std::thread::sleep(next - now);
            } else {
                next = now;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_is_centred_on_the_cursor() {
        assert_eq!(window_start(100, 5, 1920), 98);
        assert_eq!(window_start(100, 3, 1920), 99);
        assert_eq!(window_start(100, 1, 1920), 100);
    }

    #[test]
    fn window_is_clamped_at_screen_edges() {
        assert_eq!(window_start(0, 5, 1920), 0);
        assert_eq!(window_start(1, 5, 1920), 0);
        assert_eq!(window_start(1919, 5, 1920), 1915);
        assert_eq!(window_start(-3, 5, 1920), 0);
        assert_eq!(window_start(5000, 5, 1920), 1915);
    }

    #[test]
    fn window_never_exceeds_a_tiny_extent() {
        assert_eq!(window_start(1, 5, 3), 0);
    }

    #[test]
    fn covering_points_at_2x() {
        // Pixels 98..103 at 2x are points 49..51.5 -> whole points 49, 50, 51.
        assert_eq!(covering_points(98, 5, 2.0), (49, 3));
        // Pixels 99..104 -> points 49.5..52 -> 49, 50, 51.
        assert_eq!(covering_points(99, 5, 2.0), (49, 3));
        assert_eq!(covering_points(100, 1, 2.0), (50, 1));
        assert_eq!(covering_points(101, 1, 2.0), (50, 1));
    }

    #[test]
    fn covering_points_at_1x_is_identity() {
        assert_eq!(covering_points(98, 5, 1.0), (98, 5));
    }

    #[test]
    fn crop_reads_rows_in_order() {
        // 3x2 image: pixel value = (x, y, 7)
        let mut rgba = Vec::new();
        for y in 0..2u8 {
            for x in 0..3u8 {
                rgba.extend_from_slice(&[x, y, 7, 255]);
            }
        }
        let px = crop_rgba(&rgba, 3, 1, 0, 2);
        assert_eq!(
            px,
            vec![
                Rgb::new(1, 0, 7),
                Rgb::new(2, 0, 7),
                Rgb::new(1, 1, 7),
                Rgb::new(2, 1, 7)
            ]
        );
    }

    #[test]
    fn capture_center_is_middle_pixel() {
        let pixels = (0..25).map(|i| Rgb::new(i, 0, 0)).collect();
        let cap = Capture {
            size: 5,
            pixels,
            display: DisplayInfo {
                id: 0,
                scale: 1.0,
                profile: "sRGB".into(),
                is_srgb: true,
                converted_to_srgb: false,
            },
        };
        assert_eq!(cap.center(), Rgb::new(12, 0, 0));
    }

    #[test]
    fn rate_meter_counts_last_second_only() {
        let t0 = Instant::now();
        let mut meter = RateMeter::default();
        for i in 0..30 {
            meter.tick(t0 + Duration::from_millis(i * 33));
        }
        assert_eq!(meter.rate(), 30.0);
        meter.tick(t0 + Duration::from_millis(2500));
        assert_eq!(meter.rate(), 1.0);
    }
}
