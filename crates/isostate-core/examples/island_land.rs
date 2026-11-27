//! The land at the island's settings (docs/rules/land.md, "The island's
//! settings"): how long it takes to make, the whole and the sketch, and what
//! comes of it, for one island at the defaults and at the ends of each setting
//! of the land.
//!
//!     cargo run --profile work -p isostate-core --example island_land -- SEED [NAME=VALUE[,NAME=VALUE]]...
//!
//! With no settings named it tries the defaults and both ends of hills, rain,
//! wander and lakes. Each line gives the time to make the land alone
//! (`landcells::land_alone`) and its sketch (`landcells::sketch_land`), by the
//! clock and by the processor (this thread's time, which other work on the
//! machine moves less), then the land: land cells, mountain cells, river and
//! navigable cells, lakes, cells with coal, fertile cells (over 0.5), the first
//! towns found of six, and how near the sketch comes to the whole (the share of
//! cells it calls land or sea as the whole does, and of the whole's river cells
//! that are rivers in it).
#![allow(non_snake_case)]
use isostate_core::island::{Island, SETTINGS};
use isostate_core::landcells::{generate_land, land_alone, sketch_land};
use isostate_core::params::game_params;
use isostate_core::world::MOUNTAINS;

fn timed<T>(f: impl FnOnce() -> T) -> (T, f64, f64) {
    let cpu0 = isostate_core::sweep::thread_cpu_seconds().unwrap_or(f64::NAN);
    let t0 = std::time::Instant::now();
    let out = f();
    (out, t0.elapsed().as_secs_f64(), isostate_core::sweep::thread_cpu_seconds().unwrap_or(f64::NAN) - cpu0)
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seed: u64 = a.first().and_then(|s| s.parse().ok()).expect("SEED");
    let mut asked: Vec<String> = a[1..].to_vec();
    if asked.is_empty() {
        asked.push(String::new());
        for s in SETTINGS.iter().filter(|s| s.part == isostate_core::island::Part::Land) {
            asked.push(format!("{}={}", s.name, s.least));
            asked.push(format!("{}={}", s.name, s.most));
        }
    }
    println!("island {seed}: settings | alone s (cpu) | sketch s (cpu) | land mountains rivers navigable lakes coal fertile towns | sketch: same land/sea, rivers kept");
    for words in &asked {
        let mut P = game_params();
        P.seed = seed;
        Island::parse(words).and_then(|i| i.set(&mut P)).unwrap();
        let L = P.land.clone();
        let (whole, s_whole, c_whole) = timed(|| land_alone(seed, P.W, P.H, &L));
        let (sketch, s_sketch, c_sketch) = timed(|| sketch_land(seed, P.W, P.H, &L));
        let towns = generate_land(seed, P.W, P.H, 6, &L).town_sites.len();
        let n = whole.land.len();
        let count = |v: &dyn Fn(usize) -> bool| (0..n).filter(|&c| v(c)).count();
        let lakes = whole.landform.as_ref().map_or(0, |lf| lf.lakes.len());
        let same = count(&|c| whole.land[c] == sketch.land[c]) as f64 / n as f64;
        let rivers = count(&|c| whole.river[c]);
        let kept = count(&|c| whole.river[c] && sketch.river[c]) as f64 / rivers.max(1) as f64;
        println!(
            "{:<12} | {:5.2} ({:5.2}) | {:5.2} ({:5.2}) | {} {} {} {} {} {} {} {} | {:.3} {:.3}",
            if words.is_empty() { "as it comes" } else { words },
            s_whole,
            c_whole,
            s_sketch,
            c_sketch,
            count(&|c| whole.land[c]),
            count(&|c| whole.land[c] && whole.terrain[c] == MOUNTAINS),
            rivers,
            count(&|c| whole.navigable[c]),
            lakes,
            count(&|c| whole.coal[c] > 0.0),
            count(&|c| whole.fertility[c] > 0.5),
            towns,
            same,
            kept
        );
    }
}
