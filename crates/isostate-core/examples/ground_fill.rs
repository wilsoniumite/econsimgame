//! How full a town's ground is (docs/rules/ground.md, "A town fills its
//! ground"): for every town of the islands given, in the years asked, of the
//! ground inside its outline how much is built, how much is town ground left
//! open, how much is farmed, and how many of its squares are under half built.
//! Plays the islands under the game's rules, one thread each.
//!
//! ```text
//! cargo run --release -p isostate-core --example ground_fill -- --seeds 1-8 --years 1950,2000,2050 --out fill.csv [--set ground.fill=false]
//! ```
//!
//! A square's built share is what the viewer draws on it: its residents over
//! its `dens` (the ground its houses take, in people as a square of 1900 holds
//! them) over the ceiling of 1900, plus the share of it under works, at most 1;
//! less the part kept as a park, which is counted apart. The outline is the
//! town's squares and the land no town holds that they close round: the
//! squares of a hole in the town, and those with five of their eight
//! neighbours in the town or the outline. Of those the farmed part is the
//! land rule's shares of farm, pasture and wood, and the built part the share
//! under works. Nothing here draws a random number of the run's or changes its
//! state.
#![allow(non_snake_case)]

use std::collections::VecDeque;
use std::io::Write;

use isostate_core::*;

#[global_allocator]
static HEAPS: isostate_core::heaps::ThreadHeaps = isostate_core::heaps::ThreadHeaps;

/// One town in one year.
#[derive(Clone, Debug, Default)]
struct Row {
    seed: u64,
    year: i64,
    town: usize,
    name: String,
    townsfolk: f64,
    squares: usize,
    outline: usize,
    built: f64,
    drawn: f64,
    park: f64,
    open: f64,
    hole_farmed: f64,
    hole_built: f64,
    hole_other: f64,
    under_half: usize,
    filled: usize,
    groups: usize,
    largest_group: usize,
    value: f64,
    dens_mean: f64,
    dens_middle: f64,
    dens_edge: f64,
    young: f64,
    works: f64,
    per_room: f64,
    outbid: f64,
    node: (usize, usize),
}

fn parse_seeds(s: &str) -> Vec<u64> {
    let mut v = Vec::new();
    for part in s.split(',') {
        if let Some((a, b)) = part.split_once('-') {
            v.extend(a.parse::<u64>().unwrap()..=b.parse::<u64>().unwrap());
        } else {
            v.push(part.parse().unwrap());
        }
    }
    v
}

fn measure(s: &sim::Sim, seed: u64, year: i64) -> Vec<Row> {
    let (st, C, P) = (&s.st, &s.content, &s.params);
    let (N, W, H) = (st.N, st.world.W, st.world.H);
    let cy = ground::ceiling_year(st, P, year);
    let full = towns::ceiling(C, P, cy.min(P.ground.from));
    let plough = towns::plough_uses(C);
    let value = ground::land_value(st, P);
    let anchor = longrun::anchor(st, P);
    let mut owner = vec![usize::MAX; N];
    for t in 0..st.T_max {
        for &c in &st.town_ground[t] {
            owner[c] = t;
        }
    }
    let nb8 = |c: usize| {
        let (y, x) = ((c / W) as i64, (c % W) as i64);
        (-1i64..=1).flat_map(move |dy| (-1i64..=1).map(move |dx| (y + dy, x + dx))).filter(move |&(ny, nx)| ny >= 0 && nx >= 0 && (ny as usize) < H && (nx as usize) < W && !(ny == y && nx == x)).map(move |(ny, nx)| ny as usize * W + nx as usize)
    };
    let mut out = Vec::new();
    for t in 0..st.T_max {
        let list = &st.town_ground[t];
        if !st.town_active[t] || list.is_empty() {
            continue;
        }
        let mine = |c: usize| owner[c] == t;
        // the outline: the town's squares, the land they close round
        let (mut x0, mut x1, mut y0, mut y1) = (usize::MAX, 0, usize::MAX, 0);
        for &c in list {
            x0 = x0.min(c % W);
            x1 = x1.max(c % W);
            y0 = y0.min(c / W);
            y1 = y1.max(c / W);
        }
        let (x0, y0, x1, y1) = (x0.saturating_sub(1), y0.saturating_sub(1), (x1 + 1).min(W - 1), (y1 + 1).min(H - 1));
        let inside = |c: usize| (x0..=x1).contains(&(c % W)) && (y0..=y1).contains(&(c / W));
        let mut inner = vec![false; N];
        // bays: land no town holds with five of eight neighbours in the outline
        loop {
            let mut more = false;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let c = y * W + x;
                    if mine(c) || inner[c] || owner[c] != usize::MAX || !st.world.land[c] {
                        continue;
                    }
                    if nb8(c).filter(|&n| mine(n) || inner[n]).count() >= 5 {
                        inner[c] = true;
                        more = true;
                    }
                }
            }
            if !more {
                break;
            }
        }
        // holes: what cannot reach the box's edge without crossing the town (4-steps)
        let mut outside = vec![false; N];
        let mut q = VecDeque::new();
        for y in y0..=y1 {
            for x in x0..=x1 {
                let c = y * W + x;
                if (y == y0 || y == y1 || x == x0 || x == x1) && !mine(c) && !inner[c] {
                    outside[c] = true;
                    q.push_back(c);
                }
            }
        }
        while let Some(c) = q.pop_front() {
            let (y, x) = (c / W, c % W);
            for (dy, dx) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                let (ny, nx) = (y as i64 + dy, x as i64 + dx);
                if ny < 0 || nx < 0 || ny as usize >= H || nx as usize >= W {
                    continue;
                }
                let n = ny as usize * W + nx as usize;
                if inside(n) && !outside[n] && !mine(n) && !inner[n] {
                    outside[n] = true;
                    q.push_back(n);
                }
            }
        }
        for y in y0..=y1 {
            for x in x0..=x1 {
                let c = y * W + x;
                if !outside[c] && !mine(c) && owner[c] == usize::MAX && st.world.land[c] {
                    inner[c] = true;
                }
            }
        }
        let mut r = Row { seed, year, town: t, name: st.town_name[t].clone(), townsfolk: st.urban_pop[t], squares: list.len(), node: (st.town_node[t] % W, st.town_node[t] / W), ..Default::default() };
        let mut built_of = vec![0.0f64; N];
        let (mut young, mut works, mut per_room) = (0.0, 0.0, 0.0);
        for &c in list {
            let dens = st.ground.dens.get(c).copied().unwrap_or(1.0);
            let ks = st.cell_K_share[c].clamp(0.0, 1.0);
            let share = ground::built_share(st, C, P, year, c);
            let park = (1.0 - ks) * ground::park_of(st, P, c);
            let b = ks + (1.0 - ks) * share;
            // what the viewer draws today: residents over dens, as houses
            let homes = st.urban_res[c] / (dens * full).max(1e-9);
            r.drawn += (homes + ks).min(1.0);
            built_of[c] = b;
            r.built += b;
            r.park += park;
            r.open += (1.0 - b - park).max(0.0);
            r.under_half += (b + park < 0.5) as usize;
            r.filled += (b + park >= 0.5) as usize;
            young += share;
            works += towns::room_beside_works(P, ks);
            per_room += st.urban_res[c] / st.town_room[c].max(1e-9);
            r.dens_mean += dens;
        }
        let n = list.len() as f64;
        r.young = young / n;
        r.works = works / n;
        r.per_room = per_room / n;
        r.dens_mean /= n;
        for c in 0..N {
            if inner[c] {
                r.outline += 1;
                let ks = st.cell_K_share[c].clamp(0.0, 1.0);
                let farmed: f64 = plough.iter().map(|&s| st.lam[(s, c)]).sum();
                r.hole_built += ks;
                r.hole_farmed += farmed;
                r.hole_other += (1.0 - ks - farmed).max(0.0);
            }
        }
        r.outline += list.len();
        // the squares at least half built (or park), in groups that touch
        let mut seen = vec![false; N];
        for &c in list {
            if seen[c] || built_of[c] + (1.0 - st.cell_K_share[c].clamp(0.0, 1.0)) * ground::park_of(st, P, c) < 0.5 {
                continue;
            }
            r.groups += 1;
            let mut size = 0;
            let mut stack = vec![c];
            seen[c] = true;
            while let Some(a) = stack.pop() {
                size += 1;
                for b in nb8(a) {
                    if !seen[b] && mine(b) && built_of[b] + (1.0 - st.cell_K_share[b].clamp(0.0, 1.0)) * ground::park_of(st, P, b) >= 0.5 {
                        seen[b] = true;
                        stack.push(b);
                    }
                }
            }
            r.largest_group = r.largest_group.max(size);
        }
        // dens in the middle (the centre square) and at the edge (within a square and a half of the farthest)
        let far = list.iter().map(|&c| ground::from_centre(st, t, c)).fold(0.0, f64::max);
        r.dens_middle = st.ground.dens.get(st.town_node[t]).copied().unwrap_or(1.0);
        let edge: Vec<f64> = list.iter().filter(|&&c| ground::from_centre(st, t, c) >= far - 1.5).map(|&c| st.ground.dens.get(c).copied().unwrap_or(1.0)).collect();
        r.dens_edge = edge.iter().sum::<f64>() / edge.len().max(1) as f64;
        r.value = if st.ground.rent.len() == st.T_max && st.ground.rent0[t] > 0.0 { st.ground.rent[t] / anchor / (st.ground.rent0[t] / st.ground.anchor0[t]) } else { f64::NAN };
        let _ = &value;
        // the town's bid for a whole square (its rent a unit of room times the room
        // of a full square of 1900) over what the plough would pay for its squares
        if st.ground.rent.len() == st.T_max && st.ground.rent[t].is_finite() && st.catch.len() == N {
            let plough_rent: f64 = list.iter().map(|&c| towns::plough_rent(P, &st.bids, P.land_fixed_cost * st.wbar[t].max(1e-6), c, &plough)).sum::<f64>() / n;
            r.outbid = st.ground.rent[t] * full / plough_rent.max(1e-12);
        } else {
            r.outbid = f64::NAN;
        }
        out.push(r);
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds = vec![7u64];
    let mut years: Vec<i64> = vec![1950, 2000, 2050];
    let mut out = String::from("ground_fill.csv");
    let mut sets: Vec<(String, serde_json::Value)> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seeds" => {
                seeds = parse_seeds(&args[i + 1]);
                i += 1;
            }
            "--years" => {
                years = args[i + 1].split(',').map(|y| y.parse().expect("--years Y,Y,...")).collect();
                i += 1;
            }
            "--out" => {
                out = args[i + 1].clone();
                i += 1;
            }
            "--set" => {
                let (k, v) = args[i + 1].split_once('=').expect("--set key=value");
                let v = serde_json::from_str(v).unwrap_or(serde_json::Value::String(v.to_string()));
                sets.push((k.to_string(), v));
                i += 1;
            }
            a => panic!("unknown argument {a}"),
        }
        i += 1;
    }
    let end = *years.iter().max().expect("a year");
    let mut base = game_params();
    base.n_families = 6;
    base.end_year = Some(end);
    for (k, v) in &sets {
        base = sweep::set_field(&base, k, v).unwrap_or_else(|e| panic!("{e}"));
    }
    let handles: Vec<_> = seeds
        .iter()
        .map(|&seed| {
            let mut P = base.clone();
            P.seed = seed;
            let years = years.clone();
            std::thread::spawn(move || {
                let t0 = std::time::Instant::now();
                let mut s = sim::start(&P, game_content());
                let mut rows = Vec::new();
                while s.year <= end {
                    let year = s.year;
                    sim::run_year(&mut s).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
                    if years.contains(&year) {
                        rows.extend(measure(&s, seed, year));
                    }
                }
                eprintln!("seed {seed}: {:.0} s", t0.elapsed().as_secs_f64());
                rows
            })
        })
        .collect();
    let rows: Vec<Row> = handles.into_iter().flat_map(|h| h.join().expect("a run")).collect();
    let mut f = std::fs::File::create(&out).expect("the file can be written");
    writeln!(f, "seed,year,town,name,townsfolk,squares,outline,built,drawn,park,open,hole_farmed,hole_built,hole_other,under_half,filled,groups,largest_group,value,dens_mean,dens_middle,dens_edge,young,works,per_room,outbid,x,y").unwrap();
    for r in &rows {
        writeln!(
            f,
            "{},{},{},{},{:.0},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{}",
            r.seed, r.year, r.town, r.name, r.townsfolk, r.squares, r.outline, r.built, r.drawn, r.park, r.open, r.hole_farmed, r.hole_built, r.hole_other, r.under_half, r.filled, r.groups, r.largest_group, r.value, r.dens_mean, r.dens_middle, r.dens_edge, r.young, r.works, r.per_room, r.outbid, r.node.0, r.node.1
        )
        .unwrap();
    }
    // the summary: the first city (most townsfolk), the three towns on the dearest
    // ground (of four squares and more), and every town, on the mean of the islands
    println!("{:<34} {:>5} {:>7} {:>6} {:>7} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>7}", "", "year", "squares", "outl", "built", "park", "open", "farmed", "<half", "groups", "big", "dens", "dmid", "dedge", "young", "perrm", "drawn", "outbid");
    for &y in &years {
        let groups: [(&str, Box<dyn Fn(&[&Row]) -> Vec<Row>>); 3] = [
            ("the first city", Box::new(|v: &[&Row]| v.iter().max_by(|a, b| a.townsfolk.total_cmp(&b.townsfolk)).map(|r| vec![(*r).clone()]).unwrap_or_default())),
            ("three on the dearest ground", Box::new(|v: &[&Row]| {
                let mut w: Vec<&Row> = v.iter().copied().filter(|r| r.squares >= 4 && r.value.is_finite()).collect();
                w.sort_by(|a, b| b.value.total_cmp(&a.value));
                w.into_iter().take(3).cloned().collect()
            })),
            ("every town", Box::new(|v: &[&Row]| v.iter().map(|r| (*r).clone()).collect())),
        ];
        for (label, pick) in &groups {
            let mut acc = [0.0f64; 17];
            let mut n = 0.0f64;
            for &seed in &seeds {
                let v: Vec<&Row> = rows.iter().filter(|r| r.seed == seed && r.year == y).collect();
                let p = pick(&v);
                if p.is_empty() {
                    continue;
                }
                // shares of the outline, summed over the towns picked on the island
                let outl: f64 = p.iter().map(|r| r.outline as f64).sum();
                let sq: f64 = p.iter().map(|r| r.squares as f64).sum();
                let k = p.len() as f64;
                let vals = [
                    sq / k,
                    outl / k,
                    p.iter().map(|r| r.built + r.hole_built).sum::<f64>() / outl,
                    p.iter().map(|r| r.park).sum::<f64>() / outl,
                    p.iter().map(|r| r.open).sum::<f64>() / outl,
                    p.iter().map(|r| r.hole_farmed).sum::<f64>() / outl,
                    p.iter().map(|r| r.under_half as f64).sum::<f64>() / sq,
                    p.iter().map(|r| r.groups as f64).sum::<f64>() / k,
                    p.iter().map(|r| r.largest_group as f64 / r.filled.max(1) as f64).sum::<f64>() / k,
                    p.iter().map(|r| r.dens_mean * r.squares as f64).sum::<f64>() / sq,
                    p.iter().map(|r| r.dens_middle).sum::<f64>() / k,
                    p.iter().map(|r| r.dens_edge).sum::<f64>() / k,
                    p.iter().map(|r| r.young * r.squares as f64).sum::<f64>() / sq,
                    p.iter().map(|r| r.per_room * r.squares as f64).sum::<f64>() / sq,
                    p.iter().map(|r| r.townsfolk).sum::<f64>() / k,
                    p.iter().map(|r| r.drawn + r.hole_built).sum::<f64>() / outl,
                    (p.iter().filter(|r| r.outbid > 0.0).map(|r| r.outbid.ln()).sum::<f64>() / p.iter().filter(|r| r.outbid > 0.0).count().max(1) as f64).exp(),
                ];
                for (a, b) in acc.iter_mut().zip(vals) {
                    *a += b;
                }
                n += 1.0;
            }
            let m: Vec<f64> = acc.iter().map(|a| a / n.max(1.0)).collect();
            println!(
                "{:<34} {:>5} {:>7.1} {:>6.1} {:>7.3} {:>6.3} {:>6.3} {:>6.3} {:>6.3} {:>6.2} {:>6.2} {:>6.2} {:>6.2} {:>6.2} {:>6.3} {:>6.3} {:>6.3} {:>7.1}   townsfolk {:.0}",
                label, y, m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8], m[9], m[10], m[11], m[12], m[13], m[15], m[16], m[14]
            );
        }
    }
    println!("built, park, open (town ground unbuilt) and farmed (land no town holds inside the outline) are shares of the outline;");
    println!("<half is the share of the town's squares under half built; groups the groups of squares half built or more that touch,");
    println!("big the largest group's share of them; young the mean square's share built on (its room over a whole square's);");
    println!("perrm its people over its room; drawn the outline's share the viewer draws built today (residents over dens);");
    println!("outbid the town's rent for a whole square over the plough's for its squares (geometric mean of the towns).");
}
