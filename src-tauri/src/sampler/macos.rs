//! macOS backend: CoreGraphics capture of a small rect, colour-matched into sRGB.
//!
//! The window server returns pixels in the display's colour space (e.g. Display P3 on
//! MacBook displays), so we redraw the captured image into an sRGB bitmap context, which makes
//! ColorSync convert it. Without this, sRGB #FF0000 on a P3 panel reads back as ~#EB3323.

use objc2_core_foundation::{CFRetained, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorRenderingIntent, CGColorSpace, CGContext, CGDirectDisplayID,
    CGDisplayBounds, CGDisplayCopyColorSpace, CGDisplayCopyDisplayMode, CGDisplayMode, CGError,
    CGEvent, CGGetDisplaysWithPoint, CGImage, CGImageAlphaInfo, CGInterpolationQuality,
    CGPreflightScreenCaptureAccess, CGRequestScreenCaptureAccess, CGWindowImageOption,
    CGWindowListOption, kCGColorSpaceSRGB, kCGNullWindowID,
};

use super::{Capture, DisplayInfo, SampleError, covering_points, crop_rgba, window_start};

pub fn init() {}

pub fn has_permission() -> bool {
    CGPreflightScreenCaptureAccess()
}

pub fn request_permission() -> bool {
    CGRequestScreenCaptureAccess()
}

/// Global cursor location in points (Quartz coordinates: origin top-left of the main display).
pub fn cursor() -> Result<(f64, f64), SampleError> {
    let event = CGEvent::new(None).ok_or_else(|| SampleError::Capture("CGEventCreate".into()))?;
    let p = CGEvent::location(Some(&event));
    Ok((p.x, p.y))
}

fn display_at(x: f64, y: f64) -> Option<CGDirectDisplayID> {
    let mut ids = [0 as CGDirectDisplayID; 8];
    let mut count = 0u32;
    let err = unsafe {
        CGGetDisplaysWithPoint(
            CGPoint { x, y },
            ids.len() as u32,
            ids.as_mut_ptr(),
            &mut count,
        )
    };
    (err == CGError::Success && count > 0).then_some(ids[0])
}

fn display_info(id: CGDirectDisplayID, scale: f64) -> DisplayInfo {
    let space = CGDisplayCopyColorSpace(id);
    let name = CGColorSpace::name(Some(&space)).map(|n| n.to_string());
    let is_srgb = name.as_deref() == Some(&unsafe { kCGColorSpaceSRGB }.to_string());
    DisplayInfo {
        id,
        scale,
        profile: name.unwrap_or_else(|| "display ICC profile (not sRGB)".into()),
        is_srgb,
        converted_to_srgb: true,
    }
}

pub fn capture_at(x: f64, y: f64, size: u32) -> Result<Capture, SampleError> {
    let id = display_at(x, y).ok_or(SampleError::NoDisplay)?;
    let bounds = CGDisplayBounds(id);
    let mode = CGDisplayCopyDisplayMode(id)
        .ok_or_else(|| SampleError::Capture("CGDisplayCopyDisplayMode".into()))?;
    let px_w = CGDisplayMode::pixel_width(Some(&mode)) as u32;
    let px_h = CGDisplayMode::pixel_height(Some(&mode)) as u32;
    let scale = px_w as f64 / bounds.size.width;

    // Physical pixel under the cursor, then the N×N window around it (in display pixels).
    let px = ((x - bounds.origin.x) * scale).floor() as i64;
    let py = ((y - bounds.origin.y) * scale).floor() as i64;
    let sx = window_start(px, size, px_w);
    let sy = window_start(py, size, px_h);

    // CoreGraphics captures whole points; grab the points covering the window and crop.
    let (pt_x, pt_w) = covering_points(sx, size, scale);
    let (pt_y, pt_h) = covering_points(sy, size, scale);
    let rect = CGRect {
        origin: CGPoint {
            x: bounds.origin.x + pt_x as f64,
            y: bounds.origin.y + pt_y as f64,
        },
        size: CGSize {
            width: pt_w as f64,
            height: pt_h as f64,
        },
    };

    #[allow(deprecated)] // ScreenCaptureKit is the replacement; see SPEC.md decision log.
    let image = objc2_core_graphics::CGWindowListCreateImage(
        rect,
        CGWindowListOption::OptionOnScreenOnly,
        kCGNullWindowID,
        CGWindowImageOption::BestResolution,
    )
    .ok_or_else(|| SampleError::Capture("CGWindowListCreateImage returned nothing".into()))?;

    let (rgba, w, h) = to_srgb_rgba(&image)?;

    // The image may be at a different resolution than the display mode reports; map via its size.
    let img_scale_x = w as f64 / pt_w as f64;
    let img_scale_y = h as f64 / pt_h as f64;
    let ox = ((sx as f64 - pt_x as f64 * scale) * img_scale_x / scale).round() as u32;
    let oy = ((sy as f64 - pt_y as f64 * scale) * img_scale_y / scale).round() as u32;
    if ox + size > w || oy + size > h {
        return Err(SampleError::Capture(format!(
            "captured {w}x{h}, need {size}x{size} at ({ox},{oy})"
        )));
    }

    Ok(Capture {
        size,
        pixels: crop_rgba(&rgba, w, ox, oy, size),
        display: display_info(id, scale),
    })
}

/// Draw `image` into an 8-bit sRGB RGBA bitmap (colour-matched by ColorSync).
fn to_srgb_rgba(image: &CGImage) -> Result<(Vec<u8>, u32, u32), SampleError> {
    let w = CGImage::width(Some(image));
    let h = CGImage::height(Some(image));
    let srgb: CFRetained<CGColorSpace> =
        CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }))
            .ok_or_else(|| SampleError::Capture("sRGB colour space".into()))?;

    let mut buf = vec![0u8; w * h * 4];
    // RGBA, 8 bits per channel, byte order as laid out in memory.
    let info = CGImageAlphaInfo::PremultipliedLast.0;
    let ctx = unsafe {
        CGBitmapContextCreate(buf.as_mut_ptr().cast(), w, h, 8, w * 4, Some(&srgb), info)
    }
    .ok_or_else(|| SampleError::Capture("CGBitmapContextCreate".into()))?;

    CGContext::set_interpolation_quality(Some(&ctx), CGInterpolationQuality::None);
    CGContext::set_rendering_intent(
        Some(&ctx),
        CGColorRenderingIntent::RenderingIntentRelativeColorimetric,
    );
    CGContext::draw_image(
        Some(&ctx),
        CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: w as f64,
                height: h as f64,
            },
        },
        Some(image),
    );
    drop(ctx);
    Ok((buf, w as u32, h as u32))
}
