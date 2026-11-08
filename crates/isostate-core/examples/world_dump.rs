//! Dump the noise fields and spine control points of the Rust world generator
//! for one seed, for `parity/replay_world.py` (Python's downstream code must
//! reproduce the Rust terrain from them).
//!
//!     cargo run -p isostate-core --release --example world_dump -- 1
#![allow(non_snake_case)]
use isostate_core::rng::{make_rngs, uniform};
use isostate_core::world::{generate_world, smooth_noise};
use std::io::Write;
fn dump(name: &str, v: &[f64]) {
    let mut f = std::fs::File::create(format!("{}/rs_{name}.f64", out_dir())).unwrap();
    for x in v { f.write_all(&x.to_le_bytes()).unwrap(); }
}
fn main() {
    let seed: u64 = std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or(1);
    let (H, W) = (84usize, 120usize);
    let mut r = make_rngs(seed);
    let rng = &mut r.world;
    // the same draws in the same order as generate_world
    let base = smooth_noise(rng, H, W, 26.0, 5, 0.55);
    let detail = smooth_noise(rng, H, W, 8.0, 3, 0.5);
    let wx = smooth_noise(rng, H, W, 22.0, 3, 0.5);
    let wy = smooth_noise(rng, H, W, 22.0, 3, 0.5);
    let (Hf, Wf) = (H as f64, W as f64);
    let p = [Hf * uniform(rng, 0.12, 0.25), Wf * uniform(rng, 0.25, 0.4), Hf * uniform(rng, 0.4, 0.6), Wf * uniform(rng, 0.45, 0.7), Hf * uniform(rng, 0.75, 0.9), Wf * uniform(rng, 0.35, 0.6)];
    dump("base", &base); dump("detail", &detail); dump("wx", &wx); dump("wy", &wy); dump("p", &p);
    let mut r = make_rngs(seed);
    let w = generate_world(&mut r.world, W, H, 6, 0.58);
    let terr: Vec<f64> = w.terrain.iter().map(|&t| t as f64).collect();
    dump("terrain", &terr); dump("h", &w.h);
}

/// Where the dumps go: `$ISOSTATE_WORLD_DIR` or /tmp/worldcmp.
fn out_dir() -> String {
    let d = std::env::var("ISOSTATE_WORLD_DIR").unwrap_or_else(|_| "/tmp/worldcmp".to_string());
    std::fs::create_dir_all(&d).unwrap();
    d
}
