//! Cursor + screen sampling.
//!
//! Platform backends live in `sampler/macos.rs` and `sampler/windows.rs`. Both return the
//! N×N physical pixels centred on the cursor, as sRGB, list the displays and move the cursor.
//! Everything else here (window math, cropping, the sampling loop with freeze/resume and its
//! measurements) is platform independent.

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
    pub fn displays() -> Vec<Display> {
        Vec::new()
    }
    pub fn set_excluded_window(_display: u32, _window: Option<u32>) {}
}

pub use platform::{displays, has_permission, init, request_permission, set_excluded_window};

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

/// A display in platform-native global coordinates (macOS: points; Windows: physical pixels).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Display {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// CSS pixels of a webview on this display per native unit (macOS 1; Windows 1/scale).
    pub css_per_native: f64,
    /// Physical pixels per native unit (macOS: the backing scale, e.g. 2; Windows 1).
    pub physical_per_native: f64,
}

impl Display {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }

    /// A global point in CSS pixels relative to this display's top-left corner.
    pub fn to_local_css(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.x) * self.css_per_native,
            (y - self.y) * self.css_per_native,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Capture {
    /// Where the sample was taken, in platform-native global coordinates.
    pub cursor: (f64, f64),
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

/// `cursor` moved by whole physical pixels on the display under it. Used for arrow-key nudges:
/// the OS only places the cursor on whole points (2 px on a 2× display), so the sampler keeps
/// its own pixel offset instead of moving the cursor.
pub(crate) fn offset_point(
    cursor: (f64, f64),
    offset: (i32, i32),
    displays: &[Display],
) -> (f64, f64) {
    let per = displays
        .iter()
        .find(|d| d.contains(cursor.0, cursor.1))
        .map(|d| d.physical_per_native)
        .unwrap_or(1.0);
    (
        cursor.0 + offset.0 as f64 / per,
        cursor.1 + offset.1 as f64 / per,
    )
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

/// Payload of the `color-hovered` event (see the Tauri command contract in SPEC.md).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoveredEvent {
    /// Cursor in platform-native global coordinates.
    pub x: f64,
    pub y: f64,
    /// Display under the cursor, and the cursor relative to it in CSS pixels.
    pub display: u32,
    pub local_x: f64,
    pub local_y: f64,
    pub info: color_core::ColorInfo,
    /// The N×N sample, row by row; the centre is the pixel under the cursor.
    pub loupe: Vec<Vec<Rgb>>,
}

impl HoveredEvent {
    fn from_capture(cap: &Capture, displays: &[Display]) -> Self {
        let (x, y) = cap.cursor;
        let (local_x, local_y) = displays
            .iter()
            .find(|d| d.id == cap.display.id)
            .map(|d| d.to_local_css(x, y))
            .unwrap_or((0.0, 0.0));
        HoveredEvent {
            x,
            y,
            display: cap.display.id,
            local_x,
            local_y,
            info: color_core::color_info(cap.center()),
            loupe: to_grid(&cap.pixels, cap.size),
        }
    }
}

/// Row-major pixels to rows.
pub(crate) fn to_grid(pixels: &[Rgb], size: u32) -> Vec<Vec<Rgb>> {
    pixels.chunks(size as usize).map(|r| r.to_vec()).collect()
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
    /// While frozen the loop keeps running but takes no samples.
    frozen: AtomicBool,
    generation: AtomicU64,
    size: AtomicU32,
    status: Mutex<SamplerStatus>,
    last: Mutex<Option<HoveredEvent>>,
    displays: Mutex<Vec<Display>>,
    nudge: Mutex<Nudge>,
}

/// Arrow-key offset from the cursor, in physical pixels. Cleared when the mouse moves away from
/// `anchor`, the cursor position when the offset was set.
#[derive(Debug, Default)]
struct Nudge {
    offset: (i32, i32),
    anchor: Option<(f64, f64)>,
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

    pub fn is_frozen(&self) -> bool {
        self.shared.frozen.load(Ordering::SeqCst)
    }

    /// Stop updating and return the colour under the cursor at that moment.
    pub fn freeze(&self) -> Option<HoveredEvent> {
        self.shared.frozen.store(true, Ordering::SeqCst);
        self.shared.last.lock().unwrap().clone()
    }

    /// Shift the sampled point by whole physical pixels, relative to the current cursor.
    pub fn nudge(&self, dx: i32, dy: i32) -> Result<(), SampleError> {
        let cursor = cursor()?;
        let mut n = self.shared.nudge.lock().unwrap();
        if n.anchor != Some(cursor) {
            n.offset = (0, 0);
        }
        n.offset = (n.offset.0 + dx, n.offset.1 + dy);
        n.anchor = Some(cursor);
        Ok(())
    }

    /// The point to sample: the cursor plus any arrow-key offset (cleared once the mouse moves).
    fn sample_point(&self, displays: &[Display]) -> Result<(f64, f64), SampleError> {
        let cursor = cursor()?;
        let mut n = self.shared.nudge.lock().unwrap();
        if n.anchor.is_some() && n.anchor != Some(cursor) {
            *n = Nudge::default();
        }
        Ok(offset_point(cursor, n.offset, displays))
    }

    pub fn resume(&self) {
        self.shared.frozen.store(false, Ordering::SeqCst);
    }

    /// Displays as of the last start.
    pub fn displays(&self) -> Vec<Display> {
        self.shared.displays.lock().unwrap().clone()
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
        *self.shared.last.lock().unwrap() = None;
        *self.shared.displays.lock().unwrap() = displays();
        *self.shared.nudge.lock().unwrap() = Nudge::default();
        self.shared.frozen.store(false, Ordering::SeqCst);

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
        let loop_started = Instant::now();
        let displays = self.displays();

        while self.shared.running.load(Ordering::SeqCst)
            && self.shared.generation.load(Ordering::SeqCst) == generation
        {
            if self.is_frozen() {
                std::thread::sleep(interval);
                next = Instant::now();
                continue;
            }
            let size = self.shared.size.load(Ordering::Relaxed);
            let started = Instant::now();
            let sample = self
                .sample_point(&displays)
                .and_then(|(x, y)| capture_at(x, y, size));
            match sample {
                Ok(cap) => {
                    let hover = HoveredEvent::from_capture(&cap, &displays);
                    let now = Instant::now();
                    capture_total += now - started;
                    capture_count += 1;
                    meter.tick(now);
                    let _ = app.emit("color-hovered", &hover);
                    *self.shared.last.lock().unwrap() = Some(hover);

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

        let secs = loop_started.elapsed().as_secs_f64();
        log::info!(
            "sampling stopped: {capture_count} samples in {secs:.1} s ({:.1}/s), avg capture {:.2} ms",
            capture_count as f64 / secs,
            capture_total.as_secs_f64() * 1000.0 / capture_count.max(1) as f64
        );
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
            cursor: (0.0, 0.0),
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
    fn grid_rows() {
        let pixels: Vec<Rgb> = (0..9).map(|i| Rgb::new(i, 0, 0)).collect();
        let grid = to_grid(&pixels, 3);
        assert_eq!(grid.len(), 3);
        assert_eq!(grid[1][2], Rgb::new(5, 0, 0));
    }

    #[test]
    fn local_css_position() {
        // A 150% Windows monitor right of the primary: physical origin 1920, 1 css px = 1.5 px.
        let d = Display {
            id: 2,
            x: 1920.0,
            y: 0.0,
            width: 2880.0,
            height: 1620.0,
            css_per_native: 1.0 / 1.5,
            physical_per_native: 1.0,
        };
        assert!(d.contains(1920.0, 0.0) && !d.contains(1919.0, 0.0));
        assert_eq!(d.to_local_css(2220.0, 300.0), (200.0, 200.0));
    }

    #[test]
    fn offsets_are_whole_physical_pixels() {
        // A 2× Mac display: one physical pixel is half a point.
        let retina = Display {
            id: 1,
            x: 0.0,
            y: 0.0,
            width: 1728.0,
            height: 1117.0,
            css_per_native: 1.0,
            physical_per_native: 2.0,
        };
        assert_eq!(
            offset_point((100.0, 50.0), (1, 1), &[retina]),
            (100.5, 50.5)
        );
        assert_eq!(
            offset_point((100.0, 50.0), (-3, 0), &[retina]),
            (98.5, 50.0)
        );
        // Windows works in physical pixels already; off-display falls back to 1:1.
        assert_eq!(
            offset_point((5000.0, 5000.0), (1, -1), &[retina]),
            (5001.0, 4999.0)
        );
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
