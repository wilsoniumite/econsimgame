//! Where the hands go when machines tend (docs/rules/tending.md, "Where the
//! hands go"): one island under the game's rules with six families to 2050,
//! with any setting by name, and a long table of its books from 1900, a year
//! at a time, for the trace.
//!
//!     cargo run --profile work -p isostate-core --example tending_trace -- SEED OUT.csv [key=value ...]
//!
//! Each row is `seed,year,table,name,field,value`:
//!
//! - `island`: the island's income account (`accounts::IslandYear`), every
//!   number in it by its name, the needs' spending and the rents by sector
//!   numbered after their names;
//! - `good`: each good summed over the towns (made, bought by households,
//!   landlords and families, used by works and the land, building, spoiled,
//!   asked, the change in stock), its price weighted by what was bought, and
//!   the value of what was made and of what households bought;
//! - `sector`: each land sector summed over the towns (out, value, hands,
//!   wages, carting, inputs, rent);
//! - `method`: each method summed over the towns (out, value, hands, wages,
//!   inputs, fuel, carting, rent, managers, profit, dividends);
//! - `towns`: every column of the towns' table summed over the towns, and the
//!   wage weighted by people;
//! - `state`: what is read off the island at the year's turn: people, the
//!   townsfolk, the works' hands and the hands there are, town ground and its
//!   townsfolk a cell, and the tasks' year (hands by class, the land's hands,
//!   minders and carters).
#![allow(non_snake_case)]
use std::io::Write as _;

use isostate_core::accounts::{GOODS_FIELDS, KG, KL, KM, KT, LAND_FIELDS, METHOD_FIELDS, TOWN_FIELDS};
use isostate_core::params::Rules;
use isostate_core::tasks::{CLASSES, K};

fn flatten(prefix: &str, v: &serde_json::Value, names: &dyn Fn(&str, usize) -> String, out: &mut Vec<(String, f64)>) {
    match v {
        serde_json::Value::Number(n) => out.push((prefix.to_string(), n.as_f64().unwrap_or(f64::NAN))),
        serde_json::Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                flatten(&names(prefix, i), x, names, out);
            }
        }
        serde_json::Value::Object(o) => {
            for (k, x) in o {
                let p = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                flatten(&p, x, names, out);
            }
        }
        _ => {}
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args[1].parse().unwrap();
    let out = std::path::PathBuf::from(&args[2]);
    let mut P = Rules::Game.params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(2050);
    for kv in &args[3..] {
        let (k, v) = kv.split_once('=').expect("key=value");
        P = isostate_core::sweep::set_field(&P, k, &isostate_core::sweep::parse_value(v)).unwrap();
    }
    let C = Rules::Game.content();
    let mut rows: Vec<String> = Vec::new();
    let res = isostate_core::edits::play_noting(&P, C.clone(), &[], |sim, month| {
        if month.is_some() {
            return;
        }
        let st = &sim.st;
        let y = sim.year - 1;
        if y < 1900 {
            return;
        }
        let T = st.T_max;
        let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
        let pop: f64 = act.iter().map(|&t| st.pop[t]).sum();
        let urban: f64 = act.iter().map(|&t| st.urban_pop[t]).sum();
        let ind: f64 = act.iter().map(|&t| st.ind_labor[t]).sum();
        let hands: f64 = act.iter().map(|&t| isostate_core::who_works::hands(st, &sim.params, t)).sum();
        let ground: f64 = act.iter().map(|&t| st.town_ground[t].len() as f64).sum();
        let mut r = |name: &str, v: f64| rows.push(format!("{seed},{y},state,all,{name},{v}"));
        r("pop", pop);
        r("urban", urban);
        r("ind_labor", ind);
        r("hands", hands);
        r("town_cells", ground);
        r("urban_per_cell", urban / ground.max(1.0));
        r("towns", act.len() as f64);
        if let Some(ty) = st.tasks.years.last() {
            for c in 0..K {
                r(&format!("work_{}", CLASSES[c]), ty.work[c]);
                r(&format!("held_{}", CLASSES[c]), ty.held[c]);
                r(&format!("hands_{}", CLASSES[c]), ty.hands[c]);
                r(&format!("land_hands_{}", CLASSES[c]), ty.land_hands[c]);
            }
            r("minders", ty.minders);
            r("land_minders", ty.land_minders);
            r("carters", ty.carters);
        }
    });
    let sim = match res {
        Ok(s) => s,
        Err(e) => {
            eprintln!("seed {seed}: FAILED {e}");
            std::process::exit(1);
        }
    };
    let st = &sim.st;
    let Cn = &sim.content;
    let b = &st.acct;
    let names = |p: &str, i: usize| -> String {
        match p {
            "need_households" | "need_landlords" => format!("{p}.{}", Cn.service_name.get(i).cloned().unwrap_or(i.to_string())),
            "rent_by_sector" => format!("{p}.{}", Cn.sector_name.get(i).cloned().unwrap_or(i.to_string())),
            _ => format!("{p}.{i}"),
        }
    };
    for iy in &b.years {
        if iy.year < 1900 {
            continue;
        }
        let mut flat = Vec::new();
        flatten("", &serde_json::to_value(iy).unwrap(), &names, &mut flat);
        for (k, v) in flat {
            if k != "year" {
                rows.push(format!("{seed},{},island,all,{k},{v}", iy.year));
            }
        }
    }
    for k in 0..b.hist.n {
        let y = b.start_year + k as i64;
        if y < 1900 {
            continue;
        }
        let Some(rec) = b.year(k) else { continue };
        let T = rec.towns.len() / KT;
        let G = rec.goods.len() / (T * KG);
        let NS = rec.land.len() / (T * KL);
        let M = rec.methods.len() / (T * KM);
        // goods
        for g in 0..G {
            let mut s = vec![0.0f64; KG];
            let (mut pq, mut q, mut made_v, mut hh_v, mut lost_v) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for t in 0..T {
                let row = &rec.goods[(t * G + g) * KG..][..KG];
                for (c, v) in row.iter().enumerate() {
                    if c < 18 {
                        s[c] += *v as f64;
                    }
                }
                let p = row[18] as f64;
                let bought = (row[2] + row[3] + row[4]) as f64;
                pq += p * bought;
                q += bought;
                made_v += p * row[0] as f64;
                hh_v += p * row[2] as f64;
                lost_v += p * row[12] as f64;
            }
            let gname = Cn.good_name.get(g).cloned().unwrap_or(g.to_string());
            for c in 0..18 {
                rows.push(format!("{seed},{y},good,{gname},{},{}", GOODS_FIELDS[c], s[c]));
            }
            rows.push(format!("{seed},{y},good,{gname},price_bought,{}", if q > 0.0 { pq / q } else { f64::NAN }));
            rows.push(format!("{seed},{y},good,{gname},made_value,{made_v}"));
            rows.push(format!("{seed},{y},good,{gname},households_value,{hh_v}"));
            rows.push(format!("{seed},{y},good,{gname},spoiled_value,{lost_v}"));
        }
        // land sectors
        for s in 0..NS {
            let mut a = vec![0.0f64; KL];
            for t in 0..T {
                for c in 0..KL {
                    a[c] += rec.land[(t * NS + s) * KL + c] as f64;
                }
            }
            let sname = Cn.sector_name.get(s).cloned().unwrap_or(s.to_string());
            for c in 0..KL {
                rows.push(format!("{seed},{y},sector,{sname},{},{}", LAND_FIELDS[c], a[c]));
            }
        }
        // methods
        for m in 0..M {
            let mut a = vec![0.0f64; KM];
            for t in 0..T {
                for c in 0..KM {
                    a[c] += rec.methods[(t * M + m) * KM + c] as f64;
                }
            }
            if a.iter().all(|v| *v == 0.0) {
                continue;
            }
            let mname = Cn.method_name.get(m).cloned().unwrap_or(m.to_string());
            for c in 0..KM {
                rows.push(format!("{seed},{y},method,{mname},{},{}", METHOD_FIELDS[c], a[c]));
            }
        }
        // towns
        let mut a = vec![0.0f64; KT];
        let (mut wp, mut pp) = (0.0, 0.0);
        for t in 0..T {
            let row = &rec.towns[t * KT..][..KT];
            for c in 0..KT {
                a[c] += row[c] as f64;
            }
            wp += row[1] as f64 * row[0] as f64;
            pp += row[0] as f64;
        }
        for c in 0..KT {
            rows.push(format!("{seed},{y},towns,all,{},{}", TOWN_FIELDS[c], a[c]));
        }
        rows.push(format!("{seed},{y},towns,all,wage_by_pop,{}", wp / pp.max(1.0)));
    }
    let mut f = std::io::BufWriter::new(std::fs::File::create(&out).unwrap());
    writeln!(f, "seed,year,table,name,field,value").unwrap();
    for r in rows {
        writeln!(f, "{r}").unwrap();
    }
}
