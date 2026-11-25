//! The traffic record measured (docs/rules/traffic.md): one island played
//! under the game's rules with the rule on and with it off, side by side on two
//! threads, to say whether the rule changes anything played and what it costs;
//! and, from the run with it on, the busiest links of the years asked for, what
//! runs there, and how well they agree with where the roads are most worn.
//!
//!     cargo run --release -p isostate-core --example traffic_probe -- [--seed 7] [--end-year 2050] [--years 1900,2050] [--top 10]
#![allow(non_snake_case)]
use isostate_core::*;
use std::time::Instant;

struct Run {
    sim: Sim,
    seconds: f64,
    /// what the watcher's walk took a year, in milliseconds
    walk_ms: Vec<f64>,
    /// (year, top links, agreement lines), for the years asked
    seen: Vec<String>,
    most: f32,
    worst_through: f64,
}

fn town_near(st: &State, c: usize) -> String {
    let t = st.catch[c];
    if t >= 0 { st.town_name[t as usize].clone() } else { "-".into() }
}

fn ways_at(st: &State, c: usize) -> String {
    let mut s = ["none", "track", "made road", "turnpike"][st.road[c].min(3) as usize].to_string();
    if st.rail.get(c).copied().unwrap_or(0) > 0 {
        s.push_str("+rail");
    }
    if st.canal.get(c).copied().unwrap_or(0) > 0 {
        s.push_str("+canal");
    }
    if st.world.navigable[c] {
        s.push_str("+river");
    }
    if !st.world.land[c] {
        s = "sea".into();
    }
    s
}

/// Spearman's rank correlation.
fn spearman(a: &[f64], b: &[f64]) -> f64 {
    let rank = |v: &[f64]| -> Vec<f64> {
        let mut ix: Vec<usize> = (0..v.len()).collect();
        ix.sort_by(|&i, &j| v[i].total_cmp(&v[j]));
        let mut r = vec![0.0; v.len()];
        let mut i = 0;
        while i < ix.len() {
            let mut j = i;
            while j + 1 < ix.len() && v[ix[j + 1]] == v[ix[i]] {
                j += 1;
            }
            for k in i..=j {
                r[ix[k]] = (i + j) as f64 / 2.0;
            }
            i = j + 1;
        }
        r
    };
    let (ra, rb) = (rank(a), rank(b));
    let n = a.len() as f64;
    let (ma, mb) = (ra.iter().sum::<f64>() / n, rb.iter().sum::<f64>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for i in 0..a.len() {
        sab += (ra[i] - ma) * (rb[i] - mb);
        saa += (ra[i] - ma).powi(2);
        sbb += (rb[i] - mb).powi(2);
    }
    sab / (saa * sbb).sqrt()
}

fn look(st: &State, year: i64, top: usize) -> String {
    let (W, H, N) = (st.world.W, st.world.H, st.N);
    let tf = &st.tf;
    let mut links: Vec<(usize, usize, f32)> = (0..N * 4).filter(|&l| tf.goods[l] > 0.0).map(|l| (l / 4, l % 4, tf.goods[l])).collect();
    links.sort_by(|a, b| b.2.total_cmp(&a.2));
    let mut s = format!("\n### {year}: the {top} busiest links\n\nrank | from (x, y) | to (x, y) | weight a year | mode | ways at the two ends | town\n---|---|---|---|---|---|---\n");
    for (i, &(c, k, w)) in links.iter().take(top).enumerate() {
        let b = traffic::other_end(W, H, c, k).unwrap();
        s.push_str(&format!(
            "{} | {}, {} | {}, {} | {:.0} | {} | {} / {} | {}\n",
            i + 1,
            c % W,
            c / W,
            b % W,
            b / W,
            w,
            traffic::MODES[tf.mode[c * 4 + k] as usize],
            ways_at(st, c),
            ways_at(st, b),
            town_near(st, c)
        ));
    }
    // agreement: land links with traffic, the link's weight against the worn squares' own
    // traffic (the mean of its two ends) and against their road level (the lesser)
    let (mut lw, mut cw, mut lv) = (Vec::new(), Vec::new(), Vec::new());
    for &(c, k, w) in &links {
        let b = traffic::other_end(W, H, c, k).unwrap();
        if st.world.land[c] && st.world.land[b] {
            lw.push(w as f64);
            cw.push(0.5 * (st.traffic[c] + st.traffic[b]));
            lv.push(st.road[c].min(st.road[b]) as f64);
        }
    }
    let land_links = lw.len();
    s.push_str(&format!("\nland links with traffic: {land_links}; Spearman's rank correlation of a link's weight with its ends' square traffic {:.3}, with the lesser road level of its ends {:.3}\n", spearman(&lw, &cw), spearman(&lw, &lv)));
    // of the busiest fifty land links by cart, how many run between two turnpikes; of the fifty
    // busiest squares by the roads' own count, how many touch one of the hundred busiest links
    let cart50: Vec<&(usize, usize, f32)> = links.iter().filter(|l| tf.mode[l.0 * 4 + l.1] == traffic::CART).take(50).collect();
    let both3 = cart50.iter().filter(|l| { let b = traffic::other_end(W, H, l.0, l.1).unwrap(); st.road[l.0] == 3 && st.road[b] == 3 }).count();
    let mut cells: Vec<usize> = (0..N).filter(|&c| st.world.land[c]).collect();
    cells.sort_by(|&a, &b| st.traffic[b].total_cmp(&st.traffic[a]));
    let top100: std::collections::HashSet<usize> = links.iter().take(100).flat_map(|l| [l.0, traffic::other_end(W, H, l.0, l.1).unwrap()]).collect();
    let touch = cells.iter().take(50).filter(|c| top100.contains(c)).count();
    s.push_str(&format!("of the 50 busiest links by cart, {both3} run between two turnpike squares; of the 50 squares the roads count busiest, {touch} are an end of one of the 100 busiest links\n"));
    let by_mode: Vec<String> = (0..4).map(|m| format!("{} {}", traffic::MODES[m], links.iter().take(top).filter(|l| tf.mode[l.0 * 4 + l.1] == m as u8).count())).collect();
    s.push_str(&format!("modes among the top {top}: {}\n", by_mode.join(", ")));
    // the busiest by cart alone
    s.push_str(&format!("\nthe {top} busiest links by cart\n\nrank | from (x, y) | to (x, y) | weight a year | ways at the two ends | town\n---|---|---|---|---|---\n"));
    for (i, l) in links.iter().filter(|l| tf.mode[l.0 * 4 + l.1] == traffic::CART).take(top).enumerate() {
        let b = traffic::other_end(W, H, l.0, l.1).unwrap();
        s.push_str(&format!("{} | {}, {} | {}, {} | {:.0} | {} / {} | {}\n", i + 1, l.0 % W, l.0 / W, b % W, b / W, l.2, ways_at(st, l.0), ways_at(st, b), town_near(st, l.0)));
    }
    // what wears the turnpikes: each turnpike square's links' weight by cart and by everything
    // else; one worn by the railway's freight is one whose links by cart alone carry less than
    // the turnpike's threshold (the weight `update_roads` wears a square with counts every mode)
    let (mut pikes, mut by_rail, mut beside_rail) = (0usize, 0usize, 0usize);
    let t3 = 40_000.0f32;
    for c in 0..N {
        if !st.world.land[c] || st.road[c] < 3 {
            continue;
        }
        pikes += 1;
        let mut cart = 0.0f32;
        for k in 0..4 {
            if tf.mode[c * 4 + k] == traffic::CART {
                cart += tf.goods[c * 4 + k];
            }
            let (dx, dy) = traffic::SLOTS[k];
            let (x, y) = ((c % W) as isize - dx, (c / W) as isize - dy);
            if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
                let o = y as usize * W + x as usize;
                if tf.mode[o * 4 + k] == traffic::CART {
                    cart += tf.goods[o * 4 + k];
                }
            }
        }
        // a square's own traffic is about half the weight on its links (in and out)
        if 0.5 * cart < t3 {
            by_rail += 1;
            beside_rail += (st.rail.get(c).copied().unwrap_or(0) > 0) as usize;
        }
    }
    s.push_str(&format!("\nturnpike squares {pikes}: {by_rail} carry less than the turnpike's 40,000 a year by cart ({beside_rail} of them have a railway), so their wear is the other modes' weight\n"));
    s
}

fn play(P: Params, C: Content, years: Vec<i64>, top: usize, on: bool) -> Run {
    let t0 = Instant::now();
    let mut sim = sim::start(&P, C);
    let mut walk_ms = Vec::new();
    let mut seen = Vec::new();
    let mut most = 0.0f32;
    let mut worst_through = 0.0f64;
    let mut Pon = P.clone();
    Pon.traffic.enabled = true;
    while sim.year <= sim.end_year {
        sim::run_year(&mut sim).unwrap_or_else(|e| panic!("{e}"));
        let year = sim.year - 1;
        if on {
            let st = &sim.st;
            most = most.max(st.tf.goods.iter().copied().fold(0.0, f32::max));
            // the two walks are one walk: the weight leaving each land square is what wore its road
            for c in 0..st.N {
                if st.world.land[c] && st.tf.through.len() == st.N {
                    let (a, b) = (st.tf.through[c], st.traffic[c]);
                    worst_through = worst_through.max((a - b).abs() / b.abs().max(1e-9));
                }
            }
            if years.contains(&year) {
                seen.push(look(st, year, top));
            }
        } else if std::env::var_os("TRAFFIC_PROBE_UNTIMED").is_none() {
            // the walk timed on the run with the rule off: the same sizes, nothing it does is
            // read (with TRAFFIC_PROBE_UNTIMED set it is not, and the two runs' seconds compare)
            let t = Instant::now();
            traffic::year(&mut sim.st, &Pon, year, false);
            walk_ms.push(t.elapsed().as_secs_f64() * 1e3);
            sim.st.tf = Default::default();
        }
    }
    Run { seconds: t0.elapsed().as_secs_f64(), sim, walk_ms, seen, most, worst_through }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut end, mut years, mut top) = (7u64, 2050i64, vec![1900i64, 2050], 10usize);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => seed = args[i + 1].parse().unwrap(),
            "--end-year" => end = args[i + 1].parse().unwrap(),
            "--years" => years = args[i + 1].split(',').map(|y| y.parse().unwrap()).collect(),
            "--top" => top = args[i + 1].parse().unwrap(),
            s => panic!("what is {s}?"),
        }
        i += 2;
    }
    let rules = Rules::Game;
    let mut P = rules.params();
    P.n_families = 6;
    P.seed = seed;
    P.end_year = Some(end);
    let C = rules.content();
    let mut Poff = P.clone();
    Poff.traffic.enabled = false;
    println!("island {seed}, game rules, 6 families, to {end}: the rule on and off on two threads");
    let (on, off) = std::thread::scope(|s| {
        let (P1, C1, y1) = (P.clone(), C.clone(), years.clone());
        let a = s.spawn(move || play(P1, C1, y1, top, true));
        let (C2, y2) = (C.clone(), years.clone());
        let b = s.spawn(move || play(Poff, C2, y2, top, false));
        (a.join().unwrap(), b.join().unwrap())
    });
    let same_series = on.sim.st.series.data.len() == off.sim.st.series.data.len()
        && on.sim.st.series.data.iter().zip(&off.sim.st.series.data).all(|(a, b)| a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits()));
    let ev = |s: &Sim| serde_json::to_string(&s.events).unwrap();
    let same_news = ev(&on.sim) == ev(&off.sim);
    let same_roads = on.sim.st.road == off.sim.st.road && on.sim.st.traffic.iter().zip(&off.sim.st.traffic).all(|(a, b)| a.to_bits() == b.to_bits());
    println!("every monthly series the same to the bit: {same_series}; the news the same: {same_news}; the roads and their traffic the same: {same_roads}");
    println!("seconds: on {:.1}, off {:.1} (side by side; the off run also timed the walk itself)", on.seconds, off.seconds);
    let w = &off.walk_ms;
    let mut sorted = w.clone();
    sorted.sort_by(|a, b| a.total_cmp(b));
    println!(
        "the walk a year: mean {:.2} ms, median {:.2}, longest {:.2}, over {} years; all years {:.2} s",
        w.iter().sum::<f64>() / w.len().max(1) as f64,
        sorted.get(sorted.len() / 2).copied().unwrap_or(f64::NAN),
        sorted.last().copied().unwrap_or(f64::NAN),
        w.len(),
        w.iter().sum::<f64>() / 1e3
    );
    println!("the most any link carried in a year: {:.0} (the byte's top is 10^{})", on.most, traffic::BYTE_HI);
    println!("the weight leaving each land square, the watcher's against the roads': worst relative difference {:.2e}", on.worst_through);
    let rec = record::pack(&on.sim.st, &on.sim.content, &on.sim.params, &on.sim.events, &on.sim.rec, "probe", false);
    println!("run.bin with the rule on: {} bytes; the two layers {} bytes raw", rec.len(), on.sim.rec.way_goods.len() + on.sim.rec.way_mode.len());
    for y in &on.sim.st.tf.years {
        if y.year % 50 == 0 {
            let sent = y.sent_district + y.sent_trade;
            let all: f64 = y.weight_squares.iter().sum();
            println!(
                "{}: freight, millions of weight-km: cart {:.1}, boat {:.1}, rail {:.1}, sea {:.1}; mean haul {:.1} km, trade's {:.1} km; track {} km, made road {} km, turnpike {} km",
                y.year,
                y.weight_squares[0] * 2e-6,
                y.weight_squares[1] * 2e-6,
                y.weight_squares[2] * 2e-6,
                y.weight_squares[3] * 2e-6,
                all * 2.0 / sent.max(1e-9),
                y.trade_weight_squares * 2.0 / y.sent_trade.max(1e-9),
                y.road_squares[1] * 2,
                y.road_squares[2] * 2,
                y.road_squares[3] * 2
            );
        }
    }
    for s in &on.seen {
        print!("{s}");
    }
    if let Ok(dir) = std::env::var("TRAFFIC_PROBE_RUN") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/run.bin"), rec).unwrap();
    }
}
