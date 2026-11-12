//! The land mechanism's islands in numbers, seed by seed, beside the port's:
//! how long one takes to make, its lakes, its navigable water, its largest
//! basin, its relief, and how much land, corn land, coal and ore it has; then
//! its marsh and flood plain, its river, navigable and tideway cells, and the
//! woodland potential of its cells 760 m up or more.
//!
//!     cargo run --release -p isostate-core --example land_stats -- [SEEDS, e.g. 1-12] [--set NAME=VALUE]...
#![allow(non_snake_case)]
use std::time::Instant;

use isostate_core::landcells::make_world;
use isostate_core::landform::{LandParams, build_relief, draw_design};
use isostate_core::landwater::{WET_MARSH, WET_PLAIN, make_water};
use isostate_core::rng::{land_stream, make_rngs};
use isostate_core::world::{World, generate_world};

/// What both generators' worlds can be asked.
struct Tally {
    land: f64,
    hills: f64,
    mountains: f64,
    fert: f64,
    fertile: f64,
    pasture: f64,
    forest: f64,
    rock: f64,
    clay: f64,
    coal: f64,
    coal_cells: f64,
    ore: f64,
    river: f64,
    nav: f64,
    coast: f64,
    dh: f64,
    h: f64,
    /// Of the first towns: on the coast, and on navigable water.
    town_ports: f64,
    town_nav: f64,
}

fn tally(w: &World) -> Tally {
    let N = w.W * w.H;
    let land: Vec<usize> = (0..N).filter(|&c| w.land[c]).collect();
    let sum = |f: &[f64]| land.iter().map(|&c| f[c]).sum::<f64>();
    let count = |f: &dyn Fn(usize) -> bool| land.iter().filter(|&&c| f(c)).count() as f64;
    let (mut dh, mut nd) = (0.0, 0.0);
    for &c in &land {
        for nb in [c + 1, c + w.W] {
            if w.land[nb] {
                dh += (w.h[c] - w.h[nb]).abs();
                nd += 1.0;
            }
        }
    }
    Tally {
        land: land.len() as f64,
        hills: count(&|c| w.terrain[c] == 2),
        mountains: count(&|c| w.terrain[c] == 3),
        fert: sum(&w.fertility),
        fertile: count(&|c| w.fertility[c] > 0.5),
        pasture: sum(&w.pasture),
        forest: sum(&w.forest_pot),
        rock: sum(&w.rock),
        clay: sum(&w.clay),
        coal: sum(&w.coal),
        coal_cells: count(&|c| w.coal[c] > 0.0),
        ore: sum(&w.ore),
        river: count(&|c| w.river[c]),
        nav: count(&|c| w.navigable[c]),
        coast: count(&|c| w.coast[c]),
        dh: dh / nd,
        h: sum(&w.h) / land.len() as f64,
        town_ports: w.town_sites.iter().filter(|&&c| w.coast[c]).count() as f64,
        town_nav: w.town_sites.iter().filter(|&&c| w.navigable[c]).count() as f64,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = (1..=12).collect();
    let mut over = serde_json::to_value(LandParams::game()).unwrap();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--set" {
            let (k, v) = args[i + 1].split_once('=').expect("--set NAME=VALUE");
            over[k] = serde_json::from_str(v).unwrap();
            i += 2;
        } else {
            seeds = match args[i].split_once('-') {
                Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                None => args[i].split(',').map(|s| s.parse().unwrap()).collect(),
            };
            i += 1;
        }
    }
    let P: LandParams = serde_json::from_value(over).unwrap();
    let names =
        ["land", "hills", "mountains", "fert", "fertile", "pasture", "forest", "rock", "clay", "coal", "coal_cells", "ore", "river", "nav", "coast", "dh*1000", "h*1000", "town ports", "town nav"];
    let row = |t: &Tally| {
        [t.land, t.hills, t.mountains, t.fert, t.fertile, t.pasture, t.forest, t.rock, t.clay, t.coal, t.coal_cells, t.ore, t.river, t.nav, t.coast, 1000.0 * t.dh, 1000.0 * t.h, t.town_ports, t.town_nav]
    };
    let (mut old_sum, mut new_sum) = (vec![0.0; names.len()], vec![0.0; names.len()]);
    let (mut old_all, mut new_all): (Vec<Vec<f64>>, Vec<Vec<f64>>) = (Vec::new(), Vec::new());
    println!("seed  secs (relief water cells)  lakes  largest km2  upland  nav km  river km  basin%  max m  mean m  hills  mtns  land  lake cells  sites");
    let mut tsum = [0.0; 4];
    // the wet ground, the water boats use and the woods on the tops, printed after
    let mut wet_rows: Vec<String> = Vec::new();
    let mut wet_sum = [0.0f64; 8];
    for &seed in &seeds {
        let mut r = make_rngs(seed);
        let old = generate_world(&mut r.world, 120, 84, 6, 0.58);
        let t0 = Instant::now();
        let mut rng = land_stream(seed);
        let d = draw_design(&mut rng, 120, 84, &P);
        let relief = build_relief(&d, &P);
        let t1 = t0.elapsed().as_secs_f64();
        let water = make_water(&d, &P, &relief, &mut rng);
        let t2 = t0.elapsed().as_secs_f64();
        let w = make_world(&d, &P, &relief, &water, &mut rng, 6);
        let t3 = t0.elapsed().as_secs_f64();
        for (k, v) in [t3, t1, t2 - t1, t3 - t2].iter().enumerate() {
            tsum[k] += v;
        }
        let lf = w.landform.as_ref().unwrap();
        let (to, tn) = (tally(&old), tally(&w));
        for (k, (a, b)) in row(&to).iter().zip(row(&tn).iter()).enumerate() {
            old_sum[k] += a;
            new_sum[k] += b;
        }
        old_all.push(row(&to).to_vec());
        new_all.push(row(&tn).to_vec());
        let largest = lf.lakes.iter().map(|l| l.area_km2).fold(0.0, f32::max);
        let upland = lf.lakes.iter().filter(|l| l.level > 80.0).count();
        let zmax = lf.height.iter().cloned().fold(0.0, f32::max);
        let land_nodes: Vec<f32> = lf.height.iter().cloned().filter(|&z| z > 0.0).collect();
        let zmean = land_nodes.iter().sum::<f32>() / land_nodes.len() as f32;
        let river_km: f64 = lf
            .rivers
            .iter()
            .map(|r| (1..r.x.len()).map(|i| 2.0 * ((r.x[i] - r.x[i - 1]) as f64).hypot((r.y[i] - r.y[i - 1]) as f64)).sum::<f64>())
            .sum();
        let basin = lf.outlets.first().map_or(0.0, |o| o.area_km2 as f64) / (tn.land * 4.0);
        let sites: Vec<&str> = w.town_sites.iter().map(|&c| ["plain", "estuary", "confluence", "lake", "gap"][lf.site_kind[c] as usize]).collect();
        println!(
            "{seed:4}  {t3:.2} ({t1:.2} {:.2} {:.2})  {:5}  {:11.1}  {:6}  {:6.0}  {:8.0}  {:6.1}  {:5.0}  {:6.0}  {:5.0}  {:4.0}  {:4.0}  {:10}  {}",
            t2 - t1,
            t3 - t2,
            lf.lakes.len(),
            largest,
            upland,
            2.0 * tn.nav,
            river_km,
            100.0 * basin,
            zmax,
            zmean,
            tn.hills,
            tn.mountains,
            tn.land,
            lf.lake.iter().filter(|&&l| l).count(),
            sites.join(" "),
        );
        let node_km2 = (2.0 / lf.fine as f64).powi(2);
        let marsh = lf.wet.iter().filter(|&&v| v == WET_MARSH).count() as f64 * node_km2;
        let plain = lf.wet.iter().filter(|&&v| v == WET_PLAIN).count() as f64 * node_km2;
        let N = w.W * w.H;
        let tops: Vec<usize> = (0..N).filter(|&c| w.land[c] && w.h[c] * lf.h_ref >= 760.0).collect();
        let wood_tops = tops.iter().map(|&c| w.forest_pot[c]).sum::<f64>();
        let wood_top_most = tops.iter().map(|&c| w.forest_pot[c]).fold(0.0, f64::max);
        let wr = [marsh, plain, tn.river, tn.nav, lf.estuary.iter().filter(|&&e| e).count() as f64, tops.len() as f64, wood_tops, wood_top_most];
        for (s, v) in wet_sum.iter_mut().zip(wr) {
            *s += v;
        }
        wet_rows.push(format!(
            "{seed:4}  {:9.1}  {:9.1}  {:11.0}  {:9.0}  {:13.0}  {:13.0}  {:12.2}  {:12.3}",
            wr[0], wr[1], wr[2], wr[3], wr[4], wr[5], wr[6], wr[7]
        ));
    }
    let n = seeds.len() as f64;
    println!("mean seconds: {:.2} (relief {:.2}, water {:.2}, cells {:.2})", tsum[0] / n, tsum[1] / n, tsum[2] / n, tsum[3] / n);
    println!("\nseed  marsh km2  plain km2  river cells  nav cells  estuary cells  cells >=760 m  wood on them  most of it");
    for r in &wet_rows {
        println!("{r}");
    }
    let m: Vec<f64> = wet_sum.iter().map(|s| s / n).collect();
    println!("mean  {:9.1}  {:9.1}  {:11.1}  {:9.1}  {:13.1}  {:13.1}  {:12.2}  {:12.3}", m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7]);
    println!("\n{:12} {:>10} {:>22} {:>10} {:>22} {:>7}", "per island", "port", "[least .. most]", "land", "[least .. most]", "ratio");
    for (k, name) in names.iter().enumerate() {
        let lo = |a: &Vec<Vec<f64>>| a.iter().map(|r| r[k]).fold(f64::INFINITY, f64::min);
        let hi = |a: &Vec<Vec<f64>>| a.iter().map(|r| r[k]).fold(f64::NEG_INFINITY, f64::max);
        println!(
            "{:12} {:10.1} {:>22} {:10.1} {:>22} {:7.2}",
            name,
            old_sum[k] / n,
            format!("[{:.0} .. {:.0}]", lo(&old_all), hi(&old_all)),
            new_sum[k] / n,
            format!("[{:.0} .. {:.0}]", lo(&new_all), hi(&new_all)),
            new_sum[k] / old_sum[k].max(1e-9)
        );
    }
}
