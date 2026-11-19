//! The land's twentieth century on some islands, a line a year (docs/rules/tasks.md,
//! "The tractor and the cutter"; docs/rules/machines.md, "Nitrogen and breeding").
//!
//!     cargo run --release -p isostate-core --example land -- [--seeds 7] [--end-year 1960]
//!         [--from 1900] [--json '{"tree": {"stage": 6}}'] [--every 1]
//!
//! Each island is played under the game's rules with six families, on a thread
//! of its own. For every year from `--from` it prints, for each machine for a
//! task that reaches the land (the hoist, the tractor, the cutter): the year
//! found, its quality, and over the towns, each weighted by the work of the
//! sectors it reaches, the month's wage, a machine's plant and run goods at the
//! town's prices and its minders' pay, each over the wage (so that their sum is
//! what a unit costs of a month's wage at quality 1 and `scale` 1), the unit's
//! cost over the wage as it stands, and the share of the class the towns would
//! give it; then the shares of farm output with nitrogen and breeding standing,
//! the farms' output a hand, the island's mean price of oil, coal, electricity
//! and chemicals in the towns that made or bought any, and the shares of hands
//! on the land and in the works. The sizes of the tractor and the cutter in
//! data/tasks.toml were read from this.
#![allow(non_snake_case)]
use isostate_core::observer::annual;
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut end_year, mut json, mut from, mut every) = (vec![7u64], 1960i64, None::<String>, 1900i64, 1i64);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => {
                let s = next();
                seeds = match s.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => s.split(',').map(|x| x.parse().unwrap()).collect(),
                }
            }
            "--end-year" => end_year = next().parse().unwrap(),
            "--json" => json = Some(next()),
            "--from" => from = next().parse().unwrap(),
            "--every" => every = next().parse().unwrap(),
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = Rules::Game.params();
    if let Some(j) = &json {
        P = P.with_json_overrides(j).expect("the settings are known");
    }
    P.n_families = 6;
    P.end_year = Some(end_year);
    let handles: Vec<_> = seeds
        .iter()
        .map(|&seed| {
            let mut P = P.clone();
            P.seed = seed;
            std::thread::spawn(move || island(&P, from, every))
        })
        .collect();
    for h in handles {
        print!("{}", h.join().unwrap());
    }
}

fn island(P: &Params, from: i64, every: i64) -> String {
    let C = tree::content(P, game_content()).expect("the game's tree lays");
    let mut s = sim::start(P, C);
    let mut out = format!("island {} (tree.stage {})\n", P.seed, P.tree.stage);
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved every month");
        if year < from || (year - from) % every != 0 {
            continue;
        }
        let (st, C) = (&s.st, &s.content);
        let NS = C.NS;
        out += &format!("{year}");
        for (u, us) in C.tasks.uses.iter().enumerate() {
            if !us.on_sector.iter().any(|&b| b) {
                continue;
            }
            let found = st.tasks.found.get(u).copied().unwrap_or(-1);
            let (mut wsum, mut acc) = (0.0, [0.0f64; 6]);
            for t in (0..st.T_max).filter(|&t| st.town_active[t] && st.wage[t] > 0.0) {
                let work: f64 = (0..NS).filter(|&k| us.on_sector[k]).map(|k| st.tasks.land_work.get(t * NS + k).copied().unwrap_or(0.0)).sum();
                let work = if work > 0.0 { work } else { 1e-9 };
                let w = st.wage[t];
                let p = |g: usize| st.price[(t, g)];
                let plant: f64 = us.plant.iter().map(|&(g, v)| v * p(g)).sum();
                let run: f64 = us.run.iter().map(|&(g, v)| v * p(g)).sum();
                let ret = (P.hurdle + 1.0 / us.life) / 12.0 * plant;
                let cost = st.tasks.cost.get(t).and_then(|c| c.get(u)).copied().unwrap_or(f64::NAN);
                let share = (0..NS).filter(|&k| us.on_sector[k]).map(|k| st.tasks.land_target.get(t).map_or(0.0, |r| r[k][us.class])).fold(0.0, f64::max);
                for (a, v) in acc.iter_mut().zip([w, ret / w, run / w, us.minders, cost / w, share]) {
                    if v.is_finite() {
                        *a += work * v;
                    }
                }
                wsum += work;
            }
            let m: Vec<f64> = acc.iter().map(|a| a / wsum).collect();
            let q = st.tasks.quality.get(u).copied().unwrap_or(f64::NAN);
            out += &format!(
                " | {} found {} q {:.2}: wage {:.2} return {:.3} run {:.3} minders {:.3} (sum {:.3}) cost/w {:.3} share {:.3}",
                us.name,
                found,
                q,
                m[0],
                m[1],
                m[2],
                m[3],
                m[1] + m[2] + m[3],
                m[4],
                m[5]
            );
        }
        if let Some(t) = st.tasks.years.last() {
            let r = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
            out += &format!(" | nitrogen {:.3} breeding {:.3} grain a hand {:.1}", r(t.nitrogen_out, t.farm_out), r(t.breeding_out, t.farm_out), r(t.farm_out, t.farm_hands));
        }
        // a good's mean price where it is sold: weighted by the towns' sales of it
        for name in ["oil", "coal", "electricity", "electricals", "chemicals", "machinery", "steel", "grain"] {
            let Some(&g) = C.good_index.get(name) else { continue };
            let (mut a, mut b) = (0.0, 0.0);
            for t in (0..st.T_max).filter(|&t| st.town_active[t]) {
                let x = st.outflow[(t, g)].max(0.0);
                a += x * st.price[(t, g)];
                b += x;
            }
            out += &format!(" {name} {:.3}", if b > 0.0 { a / b } else { f64::NAN });
        }
        // the works of the two new methods: how many stand, their capacity, and
        // how many stand in a town that sells electricity
        for name in ["oil_from_coal", "nitrogen_works"] {
            let Some(m) = C.method_name.iter().position(|n| n == name) else { continue };
            let ge = C.good_index.get("electricity").copied();
            let fs: Vec<usize> = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_method[f] == m).collect();
            let lit = fs.iter().filter(|&&f| ge.is_some_and(|g| st.outflow[(st.f_town[f], g)] > 0.0)).count();
            let k: f64 = fs.iter().map(|&f| st.f_K[f]).sum();
            let fill: f64 = fs.iter().map(|&f| st.f_fill[f]).sum::<f64>() / fs.len().max(1) as f64;
            let made: f64 = C.good_index.get(if name == "oil_from_coal" { "oil" } else { "chemicals" }).map_or(f64::NAN, |&g| st.cumprod[g]);
            out += &format!(" | {name} {} works (K {:.0}, {lit} where electricity is sold, fill {fill:.3}, made so far {made:.0})", fs.len(), k);
        }
        let at = |name: &str| st.series.index.get(name).map_or(f64::NAN, |&k| annual(&st.series.data[k], C.start_year, year));
        out += &format!(" | land {:.3} works {:.3} wage {:.2}\n", at("ag_share"), at("ind_share"), at("wage"));
    }
    out
}
