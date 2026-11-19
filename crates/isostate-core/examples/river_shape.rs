//! The shape of the new land's rivers, island by island: how much they wind
//! (sinuosity, length along the channel over length along the valley), how
//! long their bends are (the wavelength) and how tight (the radius of
//! curvature at a bend's apex), each in widths of the river as it is drawn
//! (`river_width`, about two and a half times the true width; divide a
//! multiple by 2.44 for the true one). The plains are where the water falls
//! less than 2 in 1,000 down its valley (between the points a kilometre up and
//! down the channel, over the straight line between them), the middling
//! reaches 2 to 4, the steep ones more, and the plains are split into small
//! streams (1 to 5 cubic metres a second) and big rivers (5 and more, a cell's
//! river); streams under 1 cubic metre a second, braided reaches, tidewater
//! and the last 600 m of a river that joins another (the confluence's turn,
//! not a bend) are left out. docs/rules/land.md, "The bends", has the numbers.
//!
//!     cargo run --release -p isostate-core --example river_shape -- [SEEDS, e.g. 1-8] [--set NAME=VALUE]... [--tight] [--against]
//!
//! `--tight` lists every bend on the plains under 2.5 widths, in cells, for
//! pointing a camera. `--against` makes each island a second time with the
//! meanders as they were before 19 November (230 steps, no least radius, a
//! neck of 1.4 widths) and says how far the rivers now lie from where they
//! lay then: of the points of the big rivers on the plains, the 95th
//! percentile and the most of their distance from the old course of the same
//! river, in metres and in widths, and how many rivers end in anything else
//! or reach the sea or a lake anywhere else (a tributary meets its river
//! wherever that river now runs, so its last point may move with it).
#![allow(non_snake_case)]
use std::collections::HashMap;

use isostate_core::landform::{CELL_M, LandParams, build_relief, draw_design};
use isostate_core::landwater::{River, RiverEnd, Shape, make_water, river_shape};
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

/// A river's points in metres.
fn metres(r: &River) -> Vec<(f64, f64)> {
    (0..r.x.len()).map(|k| (r.x[k] as f64 * CELL_M, r.y[k] as f64 * CELL_M)).collect()
}

/// Each point's reach: 0 small and 1 big on the plains, 2 middling, 3 steep, 9 left out.
fn classes(r: &River, pts: &[(f64, f64)]) -> Vec<u8> {
    let n = pts.len();
    let mut along = vec![0.0f64; n];
    for k in 1..n {
        along[k] = along[k - 1] + (pts[k].0 - pts[k - 1].0).hypot(pts[k].1 - pts[k - 1].1);
    }
    (0..n)
        .map(|k| {
            if r.flow[k] < 1.0 || r.braided[k] || r.level[k] < 0.5 {
                return 9;
            }
            // nor the turn a tributary makes into the river it joins, its last 600 m
            if matches!(r.end, RiverEnd::River(_)) && along[n - 1] - along[k] < 600.0 {
                return 9;
            }
            let a = (0..=k).rev().find(|&j| along[k] - along[j] >= 1000.0).unwrap_or(0);
            let b = (k..n).find(|&j| along[j] - along[k] >= 1000.0).unwrap_or(n - 1);
            // over the valley, not along the channel, so that a reach made straighter is not made steeper
            let valley = (pts[b].0 - pts[a].0).hypot(pts[b].1 - pts[a].1);
            let fall = if b > a && valley > 0.0 { (r.level[a] - r.level[b]) as f64 / valley } else { 0.0 };
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
        .collect()
}

/// How far each point of `new` lies from the line `old`, in metres.
fn apart(new: &[(f64, f64)], old: &[(f64, f64)]) -> Vec<f64> {
    let cell = 250.0;
    let key = |p: (f64, f64)| ((p.0 / cell).floor() as i64, (p.1 / cell).floor() as i64);
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for k in 0..old.len().saturating_sub(1) {
        let (a, b) = (key(old[k]), key(old[k + 1]));
        for x in a.0.min(b.0)..=a.0.max(b.0) {
            for y in a.1.min(b.1)..=a.1.max(b.1) {
                grid.entry((x, y)).or_default().push(k);
            }
        }
    }
    new.iter()
        .map(|&p| {
            let (cx, cy) = key(p);
            let mut best = f64::INFINITY;
            for ring in 0..40i64 {
                for x in cx - ring..=cx + ring {
                    for y in cy - ring..=cy + ring {
                        if (x - cx).abs() != ring && (y - cy).abs() != ring {
                            continue;
                        }
                        for &k in grid.get(&(x, y)).map(|v| v.as_slice()).unwrap_or(&[]) {
                            let (a, b) = (old[k], old[k + 1]);
                            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                            let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / (dx * dx + dy * dy).max(1e-9)).clamp(0.0, 1.0);
                            best = best.min((p.0 - a.0 - dx * t).hypot(p.1 - a.1 - dy * t));
                        }
                    }
                }
                if best < ring as f64 * cell {
                    break;
                }
            }
            best
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = (1..=8).collect();
    let mut over = serde_json::to_value(LandParams::game()).unwrap();
    let (mut tight, mut against) = (false, false);
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--set" {
            let (k, v) = args[i + 1].split_once('=').expect("--set NAME=VALUE");
            over[k] = serde_json::from_str(v).unwrap();
            i += 2;
        } else if args[i] == "--tight" {
            tight = true;
            i += 1;
        } else if args[i] == "--against" {
            against = true;
            i += 1;
        } else {
            seeds = match args[i].split_once('-') {
                Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                None => args[i].split(',').map(|s| s.parse().unwrap()).collect(),
            };
            i += 1;
        }
    }
    let P: LandParams = serde_json::from_value(over).unwrap();
    let before = LandParams { meander_steps: 230, bend_radius: 0.0, neck: 1.4, ..P.clone() };
    println!("lengths in widths as drawn; radius: the tightest bend, the 5th percentile, the median; tight: bends under 2.5 widths");
    println!("  seed  reach  channel  sinu.  wavel.  r.min  r.p5 r.med  bends tight  necks");
    let mut all = [Shape::default(), Shape::default(), Shape::default(), Shape::default()];
    let mut moved_rows: Vec<String> = Vec::new();
    for &seed in &seeds {
        let make = |P: &LandParams| {
            let mut rng = land_stream(seed);
            let d = draw_design(&mut rng, 120, 84, P);
            let relief = build_relief(&d, P);
            make_water(&d, P, &relief, &mut rng)
        };
        let water = make(&P);
        let mut by = [Shape::default(), Shape::default(), Shape::default(), Shape::default()];
        for r in &water.rivers {
            let pts = metres(r);
            let width: Vec<f64> = r.width.iter().map(|&v| v as f64).collect();
            let class = classes(r, &pts);
            for (c, s) in by.iter_mut().enumerate() {
                let keep: Vec<bool> = class.iter().map(|&k| k as usize == c).collect();
                let one = river_shape(&pts, &width, &keep);
                if tight && c <= 1 {
                    for (rr, at) in one.radius.iter().zip(one.apex.iter()) {
                        if *rr < 2.5 {
                            println!("  {seed:>4}  a tight bend: {rr:.2} widths at {:.2},{:.2}", at.0 / CELL_M, at.1 / CELL_M);
                        }
                    }
                }
                s.add(&one);
            }
        }
        for (c, name) in ["small", "big", "middle", "steep"].iter().enumerate() {
            println!("  {seed:>4} {}", line(name, &by[c]));
            all[c].add(&by[c]);
        }
        if against {
            let old = make(&before);
            let (mut m, mut wd, mut ends) = (Vec::new(), Vec::new(), 0);
            for (r, o) in water.rivers.iter().zip(old.rivers.iter()) {
                let (pn, po) = (metres(r), metres(o));
                // a tributary meets its river where that river now runs; a mouth into the sea or a lake must not move
                let mouth_moved = !matches!(r.end, RiverEnd::River(_)) && pn.last().zip(po.last()).is_none_or(|(a, b)| (a.0 - b.0).hypot(a.1 - b.1) > 1.0);
                if r.end != o.end || mouth_moved {
                    ends += 1;
                }
                let class = classes(r, &pn);
                let d = apart(&pn, &po);
                for k in 0..pn.len() {
                    if class[k] == 1 {
                        m.push(d[k]);
                        wd.push(d[k] / r.width[k] as f64);
                    }
                }
            }
            moved_rows.push(format!(
                "  {seed:>4}  {:>6} rivers, {ends} ending elsewhere; big rivers on the plains lie from their old course: 95% within {:.0} m ({:.1} widths), the most {:.0} m ({:.1} widths)",
                water.rivers.len(),
                pct(&m, 0.95),
                pct(&wd, 0.95),
                pct(&m, 1.0),
                pct(&wd, 1.0)
            ));
        }
    }
    println!("all seeds");
    for (c, name) in ["small", "big", "middle", "steep"].iter().enumerate() {
        println!("  {:>4} {}", "", line(name, &all[c]));
    }
    if against {
        println!("against the meanders before 19 November (230 steps, no least radius, a neck of 1.4 widths)");
        for r in &moved_rows {
            println!("{r}");
        }
    }
}
