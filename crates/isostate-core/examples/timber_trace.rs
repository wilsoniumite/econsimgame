//! The timber-and-brick cycle, month by month on one island
//! (docs/probes/tr-timber.md): who buys timber and bricks, who stocks them, who
//! cuts and fires them, and what each sees when it acts.
//!
//!     cargo run --release -p isostate-core --example timber_trace -- --seed 7 --out DIR
//!         [--rules port|game] [--families 6] [--end-year 2050] [--from 1850]
//!         [--set key=value]... [--set-from YEAR]
//!
//! `--set` lays a setting over the rules by the sweep's names; with
//! `--set-from YEAR` the settings are laid over the running island before
//! January of that year instead (an arm, as the lab's `--rule` is), so the two
//! islands are one to the last digit until then.
//!
//! It writes two files into DIR, named by the seed:
//!
//! - `months-SEED.csv`, a row a month from `--from`: for timber, sawn timber,
//!   bricks and building work the island's price (towns weighted by people, as
//!   the observer weighs them), the merchants' anchor and the price expected,
//!   what was made, asked for and sold, the merchants' stock, the plant on
//!   order still to come in each good, and the capacity standing and on order
//!   of the works that make it; the woods' hands, the cover and the wage.
//! - `years-SEED.csv`, a row a year and good from the books: what was made,
//!   and who took it (households, landlords and families, works as materials
//!   and as fuel, the land, works being built, railways and canals), the stock
//!   at the year's end and what was asked for; then every method's use of it.
//!
//! The loop is `sim::run_year` with a sample after each month; if that changes,
//! change this with it.
#![allow(non_snake_case)]

use std::fmt::Write as _;
use std::path::PathBuf;

use isostate_core::accounts::{GOODS_FIELDS, KG};
use isostate_core::sweep::{parse_value, set_field};
use isostate_core::*;

const GOODS: [&str; 4] = ["timber", "lumber", "bricks", "building"];

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut families, mut end_year, mut from) = (7u64, 6usize, 2050i64, 1850i64);
    let mut rules = Rules::Game;
    let mut out = PathBuf::from(".");
    let mut sets: Vec<(String, String)> = Vec::new();
    let mut set_from: Option<i64> = None;
    for pair in args.chunks(2) {
        let value = pair.get(1).ok_or("a value after each option")?.clone();
        match pair[0].as_str() {
            "--seed" => seed = value.parse().map_err(|e| format!("--seed: {e}"))?,
            "--families" => families = value.parse().map_err(|e| format!("--families: {e}"))?,
            "--end-year" => end_year = value.parse().map_err(|e| format!("--end-year: {e}"))?,
            "--from" => from = value.parse().map_err(|e| format!("--from: {e}"))?,
            "--out" => out = PathBuf::from(value),
            "--rules" => rules = if value == "port" { Rules::Port } else { Rules::Game },
            "--set" => {
                let (k, v) = value.split_once('=').ok_or("--set key=value")?;
                sets.push((k.to_string(), v.to_string()));
            }
            "--set-from" => set_from = Some(value.parse().map_err(|e| format!("--set-from: {e}"))?),
            other => return Err(format!("unknown option {other}")),
        }
    }
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut P = save::params(rules, seed, families);
    P.end_year = Some(end_year);
    if set_from.is_none() {
        for (k, v) in &sets {
            P = set_field(&P, k, &parse_value(v))?;
        }
    }
    let mut s = sim::start(&P, rules.content());
    s.st.watch_markets = true;
    let C = s.content.clone();
    let goods: Vec<usize> = GOODS.iter().map(|n| C.gi(n)).collect();
    let mut months = String::new();
    write!(months, "year,month,wage,woods_hands,woods_out,cover,timber_cut_cells,worked_cover").unwrap();
    for n in GOODS {
        for col in ["price", "anchor", "pexp", "made", "asked", "sold", "stock", "on_order", "cap", "cap_order", "util"] {
            write!(months, ",{n}_{col}").unwrap();
        }
    }
    months.push('\n');
    let mut years = String::from("year,good,kind,method,value\n");
    let fi = s.st.forestry_sector;
    while s.year <= s.end_year {
        if Some(s.year) == set_from {
            for (k, v) in &sets {
                s.params = set_field(&s.params, k, &parse_value(v))?;
            }
        }
        let year = s.year;
        for month in 0..12 {
            sim::run_month(&mut s, month)?;
            if year < from {
                continue;
            }
            let st = &s.st;
            let T = st.T_max;
            let w: Vec<f64> = (0..T).map(|t| if st.town_active[t] { st.pop[t] } else { 0.0 }).collect();
            let wsum: f64 = w.iter().sum::<f64>().max(1.0);
            let nat = |m: &mat::Mat, g: usize| (0..T).map(|t| m[(t, g)] * w[t]).sum::<f64>() / wsum;
            let tot = |m: &mat::Mat, g: usize| (0..T).filter(|&t| st.town_active[t]).map(|t| m[(t, g)]).sum::<f64>();
            let wage = (0..T).map(|t| st.wage[t] * w[t]).sum::<f64>() / wsum;
            let (mut wh, mut wo, mut cells, mut worked, mut wl) = (0.0, 0.0, 0usize, 0.0, 0.0);
            for u in 0..st.ex_cell.len() {
                if st.ex_sector[u] == fi {
                    wh += st.ex_labor[u];
                    wo += st.ex_out[u];
                    if st.ex_out[u] > 0.0 {
                        cells += 1;
                    }
                    // the cover of the woods being worked, weighted by the hands on them
                    worked += st.ex_labor[u] * st.cover[st.ex_cell[u]] / st.world.forest_pot[st.ex_cell[u]].max(1e-6);
                    wl += st.ex_labor[u];
                }
            }
            let worked = if wl > 0.0 { worked / wl } else { f64::NAN };
            let cover = if s.params.woods.record { st.series.get("woods_cover")[s.t - 1] } else { f64::NAN };
            write!(months, "{year},{},{wage:.6},{wh:.3},{wo:.3},{cover:.5},{cells},{worked:.5}", month + 1).unwrap();
            for &g in &goods {
                let anchor = if st.price_anchor.rows > 0 { nat(&st.price_anchor, g) } else { f64::NAN };
                let on_order: f64 = (0..st.F_count).filter(|&f| st.f_alive[f]).map(|f| st.f_order[(f, g)]).sum();
                let (mut cap, mut cap_order, mut made_cap) = (0.0, 0.0, 0.0);
                for f in 0..st.F_count {
                    if st.f_alive[f] && C.method_out[st.f_method[f]] == g {
                        cap += st.f_K[f];
                        cap_order += st.f_order_K[f];
                        made_cap += st.f_K[f] * st.f_u[f];
                    }
                }
                let util = if cap > 0.0 { made_cap / cap } else { f64::NAN };
                write!(
                    months,
                    ",{:.6},{:.6},{:.6},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.4}",
                    nat(&st.price, g),
                    anchor,
                    nat(&st.pexp, g),
                    st.month_stats.prod[g],
                    tot(&st.month_requests, g),
                    tot(&st.month_sold, g),
                    tot(&st.inv, g),
                    on_order,
                    cap,
                    cap_order,
                    util
                )
                .unwrap();
            }
            months.push('\n');
        }
        // the plant ordered at the year's end, by the method of the works that ordered it
        let before: Vec<Vec<f64>> = goods.iter().map(|&g| (0..s.st.F_count).map(|f| s.st.f_order[(f, g)]).collect()).collect();
        sim::end_year(&mut s);
        if year < from {
            continue;
        }
        for (k, &g) in goods.iter().enumerate() {
            let mut by_method = vec![0.0f64; C.method_name.len()];
            for f in 0..s.st.F_count {
                let was = before[k].get(f).copied().unwrap_or(0.0);
                let now = if s.st.f_alive[f] { s.st.f_order[(f, g)] } else { 0.0 };
                if now > was {
                    by_method[s.st.f_method[f]] += now - was;
                }
            }
            for (m, v) in by_method.iter().enumerate() {
                if *v > 0.0 {
                    writeln!(years, "{year},{},ordered,{},{v:.1}", C.good_name[g], C.method_name[m]).unwrap();
                }
            }
        }
        let books = &s.st.acct;
        if let Some(rec) = books.hist.n.checked_sub(1).and_then(|k| books.year(k)) {
            let G = C.G;
            let T = rec.goods.len() / (G * KG);
            for &g in &goods {
                for (k, name) in GOODS_FIELDS.iter().enumerate() {
                    if name.starts_with("price") || *name == "anchor" {
                        continue;
                    }
                    let v: f64 = (0..T).map(|t| rec.goods[(t * G + g) * KG + k] as f64).sum();
                    writeln!(years, "{year},{},{name},,{v:.1}", C.good_name[g]).unwrap();
                }
                let mut by_method = vec![0.0f64; C.method_name.len()];
                for u in rec.uses.iter().filter(|u| u.good as usize == g) {
                    by_method[u.method as usize] += u.used as f64 + u.burnt as f64;
                }
                for (m, v) in by_method.iter().enumerate() {
                    if *v > 0.0 {
                        writeln!(years, "{year},{},use,{},{v:.1}", C.good_name[g], C.method_name[m]).unwrap();
                    }
                }
            }
        }
    }
    std::fs::write(out.join(format!("months-{seed}.csv")), months).map_err(|e| e.to_string())?;
    std::fs::write(out.join(format!("years-{seed}.csv")), years).map_err(|e| e.to_string())?;
    println!("island {seed}: {}", save::check(&s));
    Ok(())
}
