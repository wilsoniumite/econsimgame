//! The shape of the new land's rivers, island by island: how much they wind
//! (sinuosity, length along the channel over length along the valley), how
//! long their bends are (the wavelength) and how tight (the radius of
//! curvature at a bend's apex), each in widths of the river as it is drawn
//! (`river_width`, about two and a half times the true width; divide a
//! multiple by 2.44 for the true one). The plains are where the water falls
//! less than 2 in 1,000 over a kilometre, the middling reaches 2 to 4, the
//! steep ones more, and the plains are split into small streams (1 to 5 cubic
//! metres a second) and big rivers (5 and more, a cell's river); streams under
//! 1 cubic metre a second, braided reaches and tidewater are left out. docs/rules/land.md, "The bends", has the numbers.
//!
//!     cargo run --release -p isostate-core --example river_shape -- [SEEDS, e.g. 1-8] [--set NAME=VALUE]...
#![allow(non_snake_case)]
use isostate_core::landform::{CELL_M, LandParams, build_relief, draw_design};
use isostate_core::landwater::{Shape, make_water, river_shape};
use isostate_core::rng::land_stream;

fn pct(v: &[f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    s[((s.len() - 1) as f64 * q).round() as usize]
}

fn line(name: &str, s: &Shape) -> String {
    let tight = s.radius.iter().filter(|&&r| r < 2.5).count();
    format!(
        "{name:>6}  {:>7.0}  {:>5.2}  {:>6.1}  {:>5.1} {:>5.1} {:>5.1}  {:>5} {:>5.1}%  {:>5}",
        s.channel,
        s.sinuosity(),
        pct(&s.wavelength, 0.5),
        pct(&s.radius, 0.0),
        pct(&s.radius, 0.05),
        pct(&s.radius, 0.5),
        s.radius.len(),
        100.0 * tight as f64 / s.radius.len().max(1) as f64,
        s.necks
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = (1..=8).collect();
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
    println!("lengths in widths as drawn; radius: the tightest bend, the 5th percentile, the median; tight: bends under 2.5 widths");
    println!("  seed  reach  channel  sinu.  wavel.  r.min  r.p5 r.med  bends tight  necks");
    let mut all = [Shape::default(), Shape::default(), Shape::default(), Shape::default()];
    for &seed in &seeds {
        let mut rng = land_stream(seed);
        let d = draw_design(&mut rng, 120, 84, &P);
        let relief = build_relief(&d, &P);
        let water = make_water(&d, &P, &relief, &mut rng);
        let mut by = [Shape::default(), Shape::default(), Shape::default(), Shape::default()];
        for r in &water.rivers {
            let n = r.x.len();
            let pts: Vec<(f64, f64)> = (0..n).map(|k| (r.x[k] as f64 * CELL_M, r.y[k] as f64 * CELL_M)).collect();
            let width: Vec<f64> = r.width.iter().map(|&v| v as f64).collect();
            let mut along = vec![0.0f64; n];
            for k in 1..n {
                along[k] = along[k - 1] + (pts[k].0 - pts[k - 1].0).hypot(pts[k].1 - pts[k - 1].1);
            }
            // the fall over a kilometre either side
            let class: Vec<u8> = (0..n)
                .map(|k| {
                    if r.flow[k] < 1.0 || r.braided[k] || r.level[k] < 0.5 {
                        return 9;
                    }
                    let a = (0..=k).rev().find(|&j| along[k] - along[j] >= 1000.0).unwrap_or(0);
                    let b = (k..n).find(|&j| along[j] - along[k] >= 1000.0).unwrap_or(n - 1);
                    let fall = if b > a { (r.level[a] - r.level[b]) as f64 / (along[b] - along[a]) } else { 0.0 };
                    if fall >= 0.004 {
                        3
                    } else if fall >= 0.002 {
                        2
                    } else if r.flow[k] >= 5.0 {
                        1
                    } else {
                        0
                    }
                })
                .collect();
            for (c, s) in by.iter_mut().enumerate() {
                let keep: Vec<bool> = class.iter().map(|&k| k as usize == c).collect();
                s.add(&river_shape(&pts, &width, &keep));
            }
        }
        for (c, name) in ["small", "big", "middle", "steep"].iter().enumerate() {
            println!("  {seed:>4} {}", line(name, &by[c]));
            all[c].add(&by[c]);
        }
    }
    println!("all seeds");
    for (c, name) in ["small", "big", "middle", "steep"].iter().enumerate() {
        println!("  {:>4} {}", "", line(name, &all[c]));
    }
}
