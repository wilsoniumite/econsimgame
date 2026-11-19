//! Where the roads cross water, with the crossings rule off and on
//! (docs/rules/crossings.md). Runs islands on the new land under the game's
//! rules, each twice, side by side: once with `crossings.enabled` off and
//! once on, and prints for 1750, 1800, 1850 and 1900, pooled over the islands:
//!
//! - how many times roads cross water, for every way (tracks too) and for the
//!   ways the viewer draws (made roads and turnpikes), and how many of those
//!   crossings are bridges, fords, ferries or nothing at all (with the rule off
//!   every crossing is nothing: it costs nothing and nobody built it);
//! - the measure Stella would recognise, on the ways between towns as the
//!   carts drive them (the cost graph's cheapest paths, which the roads are
//!   worn along): the crossing points such a way crosses again, the same river
//!   within ten km of road, the places where that happens, and the share of
//!   the ways between towns that do it anywhere;
//! - with the rule on, the bridges built by decade, what they cost, who paid.
//!
//!     cargo run --release -p isostate-core --example probe_crossings -- [--seeds 1-8] [--families N]
//!         [--set NAME=VALUE]... [--places]
//!
//! `--set` is laid over both runs (the switch itself is set by the probe).
//! `--places` lists, for each island in 1900, the places where a way between
//! towns crosses a river again, with the rule off and on (for pointing a camera).
#![allow(non_snake_case)]
use isostate_core::crossings::{self, NOT_BUILT, OnLand, Table};
use isostate_core::state::State;
use isostate_core::*;

/// How the crossings of one island stood in one year, for one kind of way.
#[derive(Clone, Debug, Default)]
struct Count {
    /// crossings of water: bridged, forded, ferried, and with nothing (the rule off)
    bridge: f64,
    ford: f64,
    ferry: f64,
    none: f64,
    /// links of way
    links: f64,
    /// one road (a chain of links between junctions) crossing the same river
    /// again within ten km along it: the crossings that do, and the places
    /// (a road and a river)
    repeat: f64,
    places: f64,
    /// where those places are: the middle of their crossings, in cells, and how many
    at: Vec<(f32, f32, usize)>,
}

impl Count {
    fn all(&self) -> f64 {
        self.bridge + self.ford + self.ferry + self.none
    }
}

#[derive(Clone, Debug, Default)]
struct Snap {
    /// [tracks and better, made roads and better]
    ways: [Count; 2],
}

#[derive(Clone, Debug, Default)]
struct Run {
    snaps: Vec<Snap>,
    /// bridges built (not inherited): (year, cost, span metres, paid by landlords)
    built: Vec<(i64, f64, f64, bool)>,
    inherited: usize,
    /// output by year, money
    gdp: Vec<(i64, f64)>,
}

/// The links the viewer joins between cells with road of `level` or better
/// (`network::road_segments`): every two neighbouring such cells, but a
/// diagonal only where no pair of straight links already joins its two cells.
fn segments(w: usize, h: usize, road: &[u8], land: &[bool], level: u8) -> Vec<(usize, usize)> {
    let on = |x: isize, y: isize| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && land[y as usize * w + x as usize] && road[y as usize * w + x as usize] >= level;
    let mut out = Vec::new();
    for c in 0..w * h {
        let (x, y) = ((c % w) as isize, (c / w) as isize);
        if !on(x, y) {
            continue;
        }
        for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !on(nx, ny) {
                continue;
            }
            let diag = dx != 0 && dy != 0;
            if diag && (on(x + dx, y) || on(x, y + dy)) {
                continue;
            }
            out.push((c, ny as usize * w + nx as usize));
        }
    }
    out
}

fn kind(tab: &Table, built: &[i32], year: i64, li: usize, rule: bool) -> usize {
    if !rule {
        3
    } else if built[li] != NOT_BUILT && built[li] as i64 <= year {
        0
    } else if tab.links[li].ferry {
        2
    } else {
        1
    }
}

fn count(tab: &Table, built: &[i32], year: i64, st: &State, level: u8, rule: bool) -> Count {
    let (w, h) = (st.world.W, st.world.H);
    let segs = segments(w, h, &st.road, &st.world.land, level);
    let mut out = Count { links: segs.len() as f64, ..Count::default() };
    let mut deg = vec![0usize; w * h];
    let mut at: Vec<Vec<usize>> = vec![Vec::new(); w * h];
    for (s, &(a, b)) in segs.iter().enumerate() {
        deg[a] += 1;
        deg[b] += 1;
        at[a].push(s);
        at[b].push(s);
        let Some(li) = tab.between(w, a, b) else { continue };
        let n = tab.links[li].n as f64;
        match kind(tab, built, year, li, rule) {
            0 => out.bridge += n,
            1 => out.ford += n,
            2 => out.ferry += n,
            _ => out.none += n,
        }
    }
    // the roads one at a time: chains of links through cells where two links
    // meet, from one junction or end to the next. On each, the crossings in
    // order with how far along they lie, in km.
    let mid = &tab.middles;
    let len = |a: usize, b: usize| 2.0 * ((mid[a].0 - mid[b].0).powi(2) + (mid[a].1 - mid[b].1).powi(2)).sqrt();
    let mut seen = vec![false; segs.len()];
    for s0 in 0..segs.len() {
        if seen[s0] {
            continue;
        }
        // walk back to an end of the chain, then forward along it
        let (mut s, mut c) = (s0, segs[s0].0);
        let mut guard = 0;
        while deg[c] == 2 && guard < segs.len() {
            let next = at[c].iter().copied().find(|&t| t != s).unwrap();
            if next == s0 {
                break; // a ring
            }
            s = next;
            c = if segs[s].0 == c { segs[s].1 } else { segs[s].0 };
            guard += 1;
        }
        // c is the chain's end and s its first link
        let mut on: Vec<(u32, f32, f32, f32)> = Vec::new();
        let mut along = 0.0f32;
        loop {
            seen[s] = true;
            let (a, b) = segs[s];
            let next_c = if a == c { b } else { a };
            if let Some(li) = tab.between(w, a, b) {
                for x in tab.of(li) {
                    let into = if tab.links[li].a as usize == c { x.t } else { 1.0 - x.t };
                    on.push((x.river, along + into * len(a, b), x.x, x.y));
                }
            }
            along += len(a, b);
            c = next_c;
            if deg[c] != 2 {
                break;
            }
            match at[c].iter().copied().find(|&t| t != s && !seen[t]) {
                Some(t) => s = t,
                None => break,
            }
        }
        let mut rivers_again: Vec<u32> = Vec::new();
        let mut flagged = vec![false; on.len()];
        for i in 0..on.len() {
            for j in i + 1..on.len() {
                if on[i].0 == on[j].0 && (on[i].1 - on[j].1).abs() <= 10.0 && ((on[i].2 - on[j].2).abs() + (on[i].3 - on[j].3).abs()) > 1e-5 {
                    flagged[i] = true;
                    flagged[j] = true;
                    if !rivers_again.contains(&on[i].0) {
                        rivers_again.push(on[i].0);
                    }
                }
            }
        }
        out.repeat += flagged.iter().filter(|&&f| f).count() as f64;
        for r in &rivers_again {
            let of: Vec<&(u32, f32, f32, f32)> = on.iter().zip(&flagged).filter(|(o, f)| **f && o.0 == *r).map(|(o, _)| o).collect();
            let n = of.len() as f32;
            out.at.push((of.iter().map(|o| o.2).sum::<f32>() / n, of.iter().map(|o| o.3).sum::<f32>() / n, of.len()));
        }
        out.places += rivers_again.len() as f64;
    }
    out
}

fn island(P: Params, C: Content) -> Run {
    let rule = P.crossings.enabled;
    let mut sim = sim::start(&P, C);
    let mut run = Run::default();
    // the table, the same whether the rule is on or off
    let tab = {
        let st = &sim.st;
        let lf = st.world.landform.as_deref().expect("the probe runs on the new land");
        crossings::table(&OnLand { lf, w: st.world.W, h: st.world.H }, &st.world.land, &st.world.terrain, &P.crossings)
    };
    if rule {
        assert_eq!(tab, sim.st.xs.table, "the rule's table is the probe's");
    } else {
        // how the table's links cross: once, or the same river two or more times (a meander cut)
        let mut by = [0usize; 4];
        let mut twice = 0usize;
        for li in 0..tab.links.len() {
            let xs = tab.of(li);
            let mut rivers: Vec<u32> = xs.iter().map(|x| x.river).collect();
            rivers.sort_unstable();
            let n = rivers.len();
            rivers.dedup();
            by[n.min(3)] += 1;
            if rivers.len() < n {
                twice += 1;
            }
        }
        println!("island {}: {} links over water, {} crossing once, {} twice, {} three times or more; {} cross the same river more than once", P.seed, tab.links.len(), by[1], by[2], by[3], twice);
    }
    while sim.year <= sim.end_year {
        sim::run_year(&mut sim).unwrap_or_else(|e| panic!("{e}"));
        let st = &sim.st;
        let year = sim.year - 1;
        if [1750, 1800, 1850, 1900].contains(&year) {
            let none = vec![NOT_BUILT; tab.links.len()];
            let built = if rule { &st.xs.built } else { &none };
            let ways = [1u8, 2].map(|lv| count(&tab, built, year, st, lv, rule));
            run.snaps.push(Snap { ways });
        }
    }
    let st = &sim.st;
    for b in &st.xs.bridges {
        if b.inherited {
            run.inherited += 1;
        } else {
            run.built.push((b.year, b.cost, b.span, b.lords));
        }
    }
    let y0 = sim.content.start_year;
    let series = |name: &str| st.series.index.get(name).map_or(&[][..], |&k| &st.series.data[k][..]);
    for y in y0..=sim.end_year {
        run.gdp.push((y, 12.0 * observer::annual(series("gdp_nom"), y0, y)));
    }
    run
}

/// mean [least .. greatest] over islands
fn spread(v: &[f64]) -> String {
    let ok: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if ok.is_empty() {
        return "-".into();
    }
    let m = ok.iter().sum::<f64>() / ok.len() as f64;
    let lo = ok.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = ok.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let f = |x: f64| if x.abs() >= 100.0 { format!("{x:.0}") } else if x.abs() >= 1.0 { format!("{x:.1}") } else { format!("{x:.3}") };
    format!("{} [{} .. {}]", f(m), f(lo), f(hi))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut families, mut places) = ((1..=8).collect::<Vec<u64>>(), 6usize, false);
    let mut sets: Vec<(String, String)> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--families" => {
                families = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--seeds" => {
                let s = &args[i + 1];
                seeds = match s.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => s.split(',').map(|x| x.parse().unwrap()).collect(),
                };
                i += 1;
            }
            "--set" => {
                let (k, v) = args[i + 1].split_once('=').expect("--set NAME=VALUE");
                sets.push((k.to_string(), v.to_string()));
                i += 1;
            }
            "--places" => places = true,
            s => panic!("what is {s}?"),
        }
        i += 1;
    }
    let mut P = Rules::Game.params();
    P.n_families = families;
    P.land.enabled = true;
    for (k, v) in &sets {
        let mut json = v.clone();
        for part in k.rsplit('.') {
            json = format!("{{\"{part}\": {json}}}");
        }
        P = P.with_json_overrides(&json).unwrap_or_else(|e| panic!("--set {k}={v}: {e}"));
    }
    let C = Rules::Game.content();
    println!("game rules on the new land, {families} families, seeds {:?}{}", seeds, sets.iter().map(|(k, v)| format!(", {k}={v}")).collect::<String>());
    let runs: Vec<(Run, Run)> = std::thread::scope(|s| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let go = |on: bool| {
                    let mut P = P.clone();
                    P.seed = seed;
                    P.crossings.enabled = on;
                    let C = C.clone();
                    s.spawn(move || island(P, C))
                };
                (go(false), go(true))
            })
            .collect();
        hs.into_iter().map(|(a, b)| (a.join().expect("a run panicked"), b.join().expect("a run panicked"))).collect()
    });
    let pick = |pair: &(Run, Run), on: bool| -> Run { if on { pair.1.clone() } else { pair.0.clone() } };
    for (k, name) in ["every way (tracks and better)", "the ways drawn (made roads and turnpikes)"].iter().enumerate() {
        println!("\n## Crossings of water by {name}, an island: mean [least .. greatest]\n");
        println!("| year | rule | links | crossings | bridges | fords | ferries | nothing | crossings of a river the same road crosses again within 10 km | such places (a road and a river) |");
        println!("|---|---|---|---|---|---|---|---|---|---|");
        for (si, year) in [1750, 1800, 1850, 1900].iter().enumerate() {
            for (rule, on) in [("off", false), ("on", true)] {
                let get = |f: &dyn Fn(&Count) -> f64| -> Vec<f64> { runs.iter().map(|p| pick(p, on).snaps.get(si).map_or(f64::NAN, |s| f(&s.ways[k]))).collect() };
                println!(
                    "| {year} | {rule} | {} | {} | {} | {} | {} | {} | {} | {} |",
                    spread(&get(&|c| c.links)),
                    spread(&get(&|c| c.all())),
                    spread(&get(&|c| c.bridge)),
                    spread(&get(&|c| c.ford)),
                    spread(&get(&|c| c.ferry)),
                    spread(&get(&|c| c.none)),
                    spread(&get(&|c| c.repeat)),
                    spread(&get(&|c| c.places)),
                );
            }
        }
    }
    println!("\n## Bridges built with the rule on, an island (inherited in 1750: {})\n", spread(&runs.iter().map(|(_, b)| b.inherited as f64).collect::<Vec<_>>()));
    println!("| decade | bridges | metres of river | cost, parts in 10,000 of the decade's output | paid by landlords | by households |");
    println!("|---|---|---|---|---|---|");
    for d in (1750..1900).step_by(10) {
        let of = |r: &Run| r.built.iter().filter(|b| b.0 >= d && b.0 < d + 10).cloned().collect::<Vec<_>>();
        let n: Vec<f64> = runs.iter().map(|(_, r)| of(r).len() as f64).collect();
        let m: Vec<f64> = runs.iter().map(|(_, r)| of(r).iter().map(|b| b.2).sum()).collect();
        let share: Vec<f64> = runs
            .iter()
            .map(|(_, r)| {
                let out: f64 = r.gdp.iter().filter(|g| g.0 >= d && g.0 < d + 10).map(|g| g.1).sum();
                1e4 * of(r).iter().map(|b| b.1).sum::<f64>() / out.max(1e-9)
            })
            .collect();
        let lords: Vec<f64> = runs.iter().map(|(_, r)| of(r).iter().filter(|b| b.3).count() as f64).collect();
        let hh: Vec<f64> = runs.iter().map(|(_, r)| of(r).iter().filter(|b| !b.3).count() as f64).collect();
        println!("| {d}s | {} | {} | {} | {} | {} |", spread(&n), spread(&m), spread(&share), spread(&lords), spread(&hh));
    }
    let total: Vec<f64> = runs.iter().map(|(_, r)| r.built.len() as f64).collect();
    let all_cost: Vec<f64> = runs
        .iter()
        .map(|(_, r)| 1e4 * r.built.iter().map(|b| b.1).sum::<f64>() / r.gdp.iter().filter(|g| g.0 >= 1750 && g.0 < 1900).map(|g| g.1).sum::<f64>().max(1e-9))
        .collect();
    println!("\nBuilt in all 1750-1899: {} an island; their cost {} parts in 10,000 of the period's output", spread(&total), spread(&all_cost));
    if places {
        for (s, (off, on)) in seeds.iter().zip(&runs) {
            for (rule, r) in [("off", off), ("on", on)] {
                let mut at = r.snaps.last().map(|s| s.ways[1].at.clone()).unwrap_or_default();
                at.sort_by(|a, b| b.2.cmp(&a.2));
                let list: Vec<String> = at.iter().take(12).map(|(x, y, n)| format!("{x:.1},{y:.1} ({n})")).collect();
                println!("island {s}, rule {rule}, 1900: {}", list.join("; "));
            }
        }
    }
}
