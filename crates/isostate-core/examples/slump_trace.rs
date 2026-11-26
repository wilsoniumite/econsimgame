//! The slumps when machines tend (docs/rules/tending.md, "The slumps"): one
//! island under the game's rules with six families to 2050, with any setting by
//! name, and a row a month from a year on for each good named: what its works
//! could make, made, their hands and the share of their capacity they ran, the
//! merchants' stock and sales, the price, and the share of its works' cost that
//! is hands; and for the island the wage and what is made, linked (the series
//! the news's slump clock reads).
//!
//!     cargo run --profile work -p isostate-core --example slump_trace -- SEED FROM OUT.csv services,machinery [key=value ...]
//!
//! Each row is `seed,year,month,good,field,value`; the island's rows have the
//! good `island`.
#![allow(non_snake_case)]
use std::io::Write as _;

use isostate_core::params::Rules;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args[1].parse().unwrap();
    let from: i64 = args[2].parse().unwrap();
    let out = std::path::PathBuf::from(&args[3]);
    let names: Vec<String> = args[4].split(',').map(|s| s.to_string()).collect();
    let mut P = Rules::Game.params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(2050);
    for kv in &args[5..] {
        let (k, v) = kv.split_once('=').expect("key=value");
        P = isostate_core::sweep::set_field(&P, k, &isostate_core::sweep::parse_value(v)).unwrap();
    }
    let C = Rules::Game.content();
    let mut rows: Vec<String> = Vec::new();
    let res = isostate_core::edits::play_noting(&P, C.clone(), &[], |sim, month| {
        let Some(month) = month else { return };
        let st = &sim.st;
        let Cn = &sim.content;
        let y = sim.year;
        if y < from {
            return;
        }
        let T = st.T_max;
        let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
        let mut r = |g: &str, name: &str, v: f64| rows.push(format!("{seed},{y},{},{g},{name},{v}", month + 1));
        let pop: f64 = act.iter().map(|&t| st.pop[t]).sum();
        let wage: f64 = act.iter().map(|&t| st.wage[t] * st.pop[t]).sum::<f64>() / pop.max(1.0);
        r("island", "pop", pop);
        r("island", "wage", wage);
        r("island", "hh_cash", act.iter().map(|&t| st.hh_cash[t]).sum());
        r("island", "hh_income", act.iter().map(|&t| st.hh_income[t]).sum());
        let tm = st.series.index.get("made_linked").copied();
        if let Some(i) = tm {
            let s = &st.series.data[i];
            let t = s.iter().rposition(|x| x.is_finite() && *x != 0.0).unwrap_or(0);
            r("island", "made_linked", s[t]);
        }
        // the goods as the content has them now: the tree adds goods as it goes
        for gname in &names {
            let Some(g) = Cn.good_name.iter().position(|x| x == gname) else { continue };
            let (mut K, mut made, mut uK, mut L, mut order, mut n, mut cost, mut wages, mut fK) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
            for f in 0..st.F_count {
                if !st.f_alive[f] || Cn.method_out[st.f_method[f]] != g {
                    continue;
                }
                K += st.f_K[f];
                made += st.f_prod[f];
                uK += st.f_u[f] * st.f_K[f];
                fK += st.f_fill[f] * st.f_K[f];
                L += st.f_labor[f];
                order += st.f_order_K[f];
                n += 1.0;
                cost += st.f_cost_m[f];
                wages += st.f_labor[f] * st.wage[st.f_town[f]];
            }
            let (mut inv, mut outflow, mut pq, mut fq) = (0.0, 0.0, 0.0, 0.0);
            for &t in &act {
                inv += st.inv[(t, g)];
                outflow += st.outflow[(t, g)];
                pq += st.price[(t, g)] * st.outflow[(t, g)];
                fq += st.fill[(t, g)] * st.outflow[(t, g)];
            }
            r(gname, "month_made", st.month_stats.prod.get(g).copied().unwrap_or(f64::NAN));
            r(gname, "market_fill", fq / outflow.max(1e-9));
            r(gname, "works", n);
            r(gname, "capacity", K);
            r(gname, "on_order", order);
            r(gname, "made", made);
            r(gname, "u", uK / K.max(1e-9));
            r(gname, "fill", fK / K.max(1e-9));
            r(gname, "hands", L);
            r(gname, "pay_over_cost", wages / cost.max(1e-9));
            r(gname, "stock", inv);
            r(gname, "outflow", outflow);
            r(gname, "price", pq / outflow.max(1e-9));
        }
    });
    if let Err(e) = res {
        eprintln!("seed {seed}: FAILED {e}");
        std::process::exit(1);
    }
    let mut f = std::io::BufWriter::new(std::fs::File::create(&out).unwrap());
    writeln!(f, "seed,year,month,good,field,value").unwrap();
    for r in rows {
        writeln!(f, "{r}").unwrap();
    }
}
