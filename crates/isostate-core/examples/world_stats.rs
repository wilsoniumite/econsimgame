//! World generation statistics over seeds, to compare with Python's generator
//! (`parity/world_stats.py`): the two draw different random numbers, so only the
//! distributions can agree.
//!
//!     cargo run -p isostate-core --release --example world_stats -- 100
#![allow(non_snake_case)]
use isostate_core::rng::make_rngs;
use isostate_core::world::{generate_world, smooth_noise};
fn mean(v: &[f64]) -> f64 { v.iter().sum::<f64>() / v.len() as f64 }
fn std(v: &[f64]) -> f64 { let m = mean(v); (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt() }
fn main() {
    let n: u64 = std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or(30);
    let (mut hills, mut mounts, mut bm, mut bs, mut dm, mut ds, mut fert) = (vec![], vec![], vec![], vec![], vec![], vec![], vec![]);
    for seed in 1..=n {
        let mut r = make_rngs(seed);
        let base = smooth_noise(&mut r.world, 84, 120, 26.0, 5, 0.55);
        let detail = smooth_noise(&mut r.world, 84, 120, 8.0, 3, 0.5);
        bm.push(mean(&base)); bs.push(std(&base)); dm.push(mean(&detail)); ds.push(std(&detail));
        let mut r = make_rngs(seed);
        let w = generate_world(&mut r.world, 120, 84, 6, 0.58);
        let land = w.land.iter().filter(|&&l| l).count() as f64;
        hills.push(w.terrain.iter().filter(|&&t| t == 2).count() as f64 / land);
        mounts.push(w.terrain.iter().filter(|&&t| t == 3).count() as f64 / land);
        let f: Vec<f64> = (0..w.land.len()).filter(|&c| w.land[c]).map(|c| w.fertility[c]).collect();
        fert.push(mean(&f));
    }
    let mut sorted = hills.clone(); sorted.sort_by(|a, b| a.total_cmp(b));
    println!("rust {n} seeds: hills mean {:.3} median {:.3} | mountains {:.3} | fert {:.3} | base mean {:.3} std {:.3} | detail mean {:.3} std {:.3}",
        mean(&hills), sorted[sorted.len() / 2], mean(&mounts), mean(&fert), mean(&bm), mean(&bs), mean(&dm), mean(&ds));
}
