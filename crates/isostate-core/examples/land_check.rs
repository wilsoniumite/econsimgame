//! Many islands of the land mechanism, each held to what the economy needs of
//! it: six first towns, water that runs to the sea without a ring, flows that
//! grow downstream, river levels that never rise, every field a number in 0 to
//! 1; one island; every river ending where it says; every lake's water held
//! behind its lip; no wood above the tree line. `tests/land.rs` asks the same
//! of a few seeds; this asks it of as many as wanted, and of odd settings.
//!
//!     cargo run --release -p isostate-core --example land_check -- [SEEDS, e.g. 1-40] [--set NAME=VALUE]...
//!
//! Prints a line an island and ends with an error if any island broke a rule.
#![allow(non_snake_case)]
use isostate_core::landcells::generate_land;
use isostate_core::landform::LandParams;
use isostate_core::landwater::RiverEnd;
use isostate_core::world::NEIGH8;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = (1..=40).collect();
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
    let (W, H) = (120usize, 84usize);
    let N = W * H;
    let mut bad = 0;
    println!("seed  towns  ports  land  lakes  lake cells  by the sea  no way down  gaps  est heads  confluences  outlets  tideways    nav  nav to sea  faults");
    for &seed in &seeds {
        let made = std::panic::catch_unwind(|| generate_land(seed, W, H, 6, &P));
        let w = match made {
            Ok(w) => w,
            Err(_) => {
                println!("{seed:4}  the generator panicked");
                bad += 1;
                continue;
            }
        };
        let lf = w.landform.as_ref().unwrap();
        let mut faults: Vec<String> = Vec::new();
        if w.town_sites.len() != 6 {
            faults.push(format!("{} first towns", w.town_sites.len()));
        }
        // rings, and flow that falls downstream
        let mut ring = false;
        let mut falls = 0;
        for start in 0..N {
            if !w.land[start] {
                continue;
            }
            let (mut c, mut steps) = (start, 0);
            while w.down[c] >= 0 && w.land[w.down[c] as usize] {
                c = w.down[c] as usize;
                steps += 1;
                if steps > N {
                    ring = true;
                    break;
                }
            }
            let d = w.down[start];
            if d >= 0 && w.land[d as usize] && lf.discharge[d as usize] < lf.discharge[start] {
                falls += 1;
            }
        }
        if ring {
            faults.push("a ring in down".into());
        }
        if falls > 0 {
            faults.push(format!("flow falls downstream at {falls} cells"));
        }
        let mut rises = 0;
        for r in &lf.rivers {
            for i in 1..r.level.len() {
                if r.level[i] > r.level[i - 1] + 1e-3 {
                    rises += 1;
                }
            }
            if r.x.iter().chain(&r.y).chain(&r.width).chain(&r.level).chain(&r.flow).any(|v| !v.is_finite()) {
                faults.push("a river point that is not a number".into());
                break;
            }
        }
        if rises > 0 {
            faults.push(format!("river levels rise at {rises} points"));
        }
        for (name, f) in [("fertility", &w.fertility), ("pasture", &w.pasture), ("forest", &w.forest_pot), ("rock", &w.rock), ("clay", &w.clay), ("ore", &w.ore), ("coal", &w.coal), ("h", &w.h), ("forest0", &w.forest0)] {
            if f.iter().any(|v| !(0.0..=1.0).contains(v)) {
                faults.push(format!("{name} out of 0 to 1"));
            }
        }
        if lf.height.iter().any(|v| !v.is_finite()) || lf.flow.iter().any(|v| !v.is_finite()) {
            faults.push("a fine height or flow that is not a number".into());
        }
        let near = |c: usize, f: &dyn Fn(usize) -> bool| {
            NEIGH8.iter().any(|&(dy, dx)| {
                let (y, x) = ((c / W) as isize + dy, (c % W) as isize + dx);
                y >= 0 && y < H as isize && x >= 0 && x < W as isize && f(y as usize * W + x as usize)
            })
        };
        let by_sea = (0..N).filter(|&c| lf.lake[c] && near(c, &|n| !w.land[n] && !lf.lake[n])).count();
        let no_down = (0..N).filter(|&c| w.land[c] && w.down[c] < 0).count();
        let kinds = |k: u8| (0..N).filter(|&c| lf.site_kind[c] == k).count();
        // navigable water from which a boat reaches the open sea without leaving it
        let nav = w.navigable.iter().filter(|&&n| n).count();
        let mut seen = vec![false; N];
        let mut stack: Vec<usize> = (0..N).filter(|&c| w.navigable[c] && near(c, &|n| !w.land[n] && !lf.lake[n])).collect();
        for &c in &stack {
            seen[c] = true;
        }
        while let Some(c) = stack.pop() {
            for (dy, dx) in NEIGH8 {
                let (y, x) = ((c / W) as isize + dy, (c % W) as isize + dx);
                if y >= 0 && y < H as isize && x >= 0 && x < W as isize {
                    let nc = y as usize * W + x as usize;
                    if w.navigable[nc] && !seen[nc] {
                        seen[nc] = true;
                        stack.push(nc);
                    }
                }
            }
        }
        let nav_sea = seen.iter().filter(|&&s| s).count();
        if lf.lakes.iter().any(|l| !(l.outflow > 0.0)) {
            faults.push("a lake with nothing running out of it".into());
        }
        // one island
        let (_, islands) = isostate_core::ndimage::label(&w.land, H, W);
        if islands != 1 {
            faults.push(format!("{islands} islands"));
        }
        // every river ends where it says, and rivers joining rivers end in the sea or a lake
        let (k, fw, fh) = (lf.fine as f32, lf.fw, lf.fh);
        let node = |x: f32, y: f32| ((y * k) as usize).min(fh - 1) * fw + ((x * k) as usize).min(fw - 1);
        let mut astray = 0;
        for (r, river) in lf.rivers.iter().enumerate() {
            let (x, y) = (*river.x.last().unwrap(), *river.y.last().unwrap());
            let ok = match river.end {
                RiverEnd::Sea => lf.height[node(x, y)] <= 0.0 && lf.lake_id[node(x, y)] == 0,
                RiverEnd::Lake(l) => lf.lake_id[node(x, y)] as usize == l + 1,
                RiverEnd::River(m) => lf.rivers[m].x.iter().zip(&lf.rivers[m].y).any(|(&a, &b)| (a - x).abs() < 1e-4 && (b - y).abs() < 1e-4),
            };
            let (mut cur, mut hops) = (r, 0);
            while let RiverEnd::River(m) = lf.rivers[cur].end {
                cur = m;
                hops += 1;
                if hops > lf.rivers.len() {
                    break;
                }
            }
            if !ok || hops > lf.rivers.len() {
                astray += 1;
            }
        }
        if astray > 0 {
            faults.push(format!("{astray} rivers that do not end where they say"));
        }
        // a lake's bed lies under its water, and its shore is no lower than the
        // water (but a hand's breadth of shallows) except where it spills out
        let mut leaks = 0;
        for (l, lake) in lf.lakes.iter().enumerate() {
            let id = l as u16 + 1;
            let outlet = node(lake.outlet.0, lake.outlet.1);
            for n in 0..fw * fh {
                if lf.lake_id[n] != id {
                    continue;
                }
                let (i, j) = ((n % fw) as isize, (n / fw) as isize);
                let low_shore = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)].iter().any(|&(di, dj)| {
                    let nb = (j + dj) as usize * fw + (i + di) as usize;
                    lf.lake_id[nb] != id && nb != outlet && lf.height[nb] < lake.level - 0.26
                });
                if lf.height[n] >= lake.level || low_shore {
                    leaks += 1;
                    break;
                }
            }
        }
        if leaks > 0 {
            faults.push(format!("{leaks} lakes not held behind their lips"));
        }
        // no wood above the tree line
        let wooded_tops = (0..N).filter(|&c| w.land[c] && w.h[c] * lf.h_ref >= 760.0 && w.forest_pot[c] > 0.0).count();
        if wooded_tops > 0 {
            faults.push(format!("wood above 760 m in {wooded_tops} cells"));
        }
        let ports = w.town_sites.iter().filter(|&&c| w.coast[c]).count();
        println!(
            "{seed:4}  {:5}  {ports:5}  {:4}  {:5}  {:10}  {:10}  {:11}  {:4}  {:9}  {:11}  {:7}  {:9}  {nav:5}  {nav_sea:9}  {}",
            w.town_sites.len(),
            w.land.iter().filter(|&&l| l).count(),
            lf.lakes.len(),
            lf.lake.iter().filter(|&&l| l).count(),
            by_sea,
            no_down,
            kinds(4),
            kinds(1),
            kinds(2),
            kinds(3),
            lf.estuary.iter().filter(|&&e| e).count(),
            if faults.is_empty() { "none".to_string() } else { faults.join("; ") }
        );
        if !faults.is_empty() {
            bad += 1;
        }
    }
    if bad > 0 {
        eprintln!("{bad} of {} islands broke a rule", seeds.len());
        std::process::exit(1);
    }
}
