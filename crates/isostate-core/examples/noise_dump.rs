//! Dump the raw random draws of one `smooth_noise` call and its output, for
//! `parity/replay_noise.py` (Python's smooth_noise fed the same draws must match).
//!
//!     cargo run -p isostate-core --release --example noise_dump -- 26 5 0.55
#![allow(non_snake_case)]
use isostate_core::rng::{integers, make_rngs, random_n};
use isostate_core::world::smooth_noise;
use std::io::Write;
fn main() {
    let scale: f64 = std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or(26.0);
    let octaves: usize = std::env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(5);
    let pers: f64 = std::env::args().nth(3).map(|s| s.parse().unwrap()).unwrap_or(0.55);
    let (H, W) = (84usize, 120usize);
    let mut r = make_rngs(1);
    let mut r2 = r.world.clone();
    let out = smooth_noise(&mut r.world, H, W, scale, octaves, pers);
    // the same draws again, recorded
    let mut draws: Vec<f64> = Vec::new();
    for o in 0..octaves {
        let step = 2usize.max((scale / 2f64.powi(o as i32)).round_ties_even() as usize);
        let (gh, gw) = (H / step + 4, W / step + 4);
        draws.extend(random_n(&mut r2, gh * gw));
        draws.push(integers(&mut r2, step) as f64);
        draws.push(integers(&mut r2, step) as f64);
    }
    let mut f = std::fs::File::create(format!("{}/noise_draws.f64", out_dir())).unwrap();
    for x in &draws { f.write_all(&x.to_le_bytes()).unwrap(); }
    let mut f = std::fs::File::create(format!("{}/noise_out.f64", out_dir())).unwrap();
    for x in &out { f.write_all(&x.to_le_bytes()).unwrap(); }
}

/// Where the dumps go: `$ISOSTATE_WORLD_DIR` or /tmp/worldcmp.
fn out_dir() -> String {
    let d = std::env::var("ISOSTATE_WORLD_DIR").unwrap_or_else(|_| "/tmp/worldcmp".to_string());
    std::fs::create_dir_all(&d).unwrap();
    d
}
