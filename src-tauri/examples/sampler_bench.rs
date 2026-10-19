//! Phase 1 measurement tool for the screen sampler.
//!
//!   cargo run --release --example sampler_bench           # timing per sample size
//!   cargo run --release --example sampler_bench -- watch  # print the colour under the cursor
//!
//! The terminal app running this needs Screen Recording permission on macOS.

use std::time::{Duration, Instant};

use color_buddy_lib::sampler;

fn main() {
    sampler::init();
    println!("screen capture permission: {}", sampler::has_permission());
    if !sampler::has_permission() {
        sampler::request_permission();
        eprintln!("grant Screen Recording to this terminal app, restart it, and run again");
        std::process::exit(1);
    }

    if std::env::args().any(|a| a == "watch") {
        watch();
    } else {
        bench();
    }
}

fn bench() {
    let cold = Instant::now();
    let first = sampler::capture(5).expect("capture failed");
    println!("cold first capture (N=5): {:.1} ms", ms(cold.elapsed()));
    println!("display: {:?}", first.display);

    for size in sampler::SIZES {
        let mut times: Vec<Duration> = (0..300)
            .map(|_| {
                let t = Instant::now();
                sampler::capture(size).expect("capture failed");
                t.elapsed()
            })
            .collect();
        times.sort();
        let avg = times.iter().sum::<Duration>() / times.len() as u32;
        let p95 = times[times.len() * 95 / 100];
        println!(
            "N={size}: avg {:.2} ms, p95 {:.2} ms, max {:.2} ms -> up to {:.0} samples/s",
            ms(avg),
            ms(p95),
            ms(*times.last().unwrap()),
            1000.0 / ms(avg)
        );
    }
}

fn watch() {
    println!("move the cursor; Ctrl+C to stop");
    loop {
        let (x, y) = sampler::cursor().unwrap();
        match sampler::capture(5) {
            Ok(cap) => {
                print!("({x:.1}, {y:.1}) {} ", cap.center().to_hex());
                #[cfg(target_os = "macos")]
                print!("| raw xcap (no colour conversion): {}", raw_xcap(x, y));
                println!();
            }
            Err(e) => println!("({x:.1}, {y:.1}) error: {e}"),
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

#[cfg(target_os = "macos")]
fn raw_xcap(x: f64, y: f64) -> String {
    let Ok(m) = xcap::Monitor::from_point(x as i32, y as i32) else {
        return "-".into();
    };
    let (lx, ly) = (
        (x as i32 - m.x().unwrap()) as u32,
        (y as i32 - m.y().unwrap()) as u32,
    );
    match m.capture_region(lx, ly, 1, 1) {
        Ok(img) => {
            let p = img.get_pixel(0, 0);
            format!("#{:02X}{:02X}{:02X}", p[0], p[1], p[2])
        }
        Err(e) => e.to_string(),
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}
