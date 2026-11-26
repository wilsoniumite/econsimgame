//! The carting, a row a year from 1890, for the lorry (docs/rules/lorry.md):
//! one island under the game's rules with six families to 2050, with any
//! setting by name.
//!
//!     cargo run --release -p isostate-core --example lorry -- SEED OUT.csv [key=value ...]
//!
//! Each row is one year: the carters (worker-months a month, the year's mean)
//! and all the hands at work (the works' hours and minders, the land's hands and
//! what else its units hired, and the carters); the carting bill over the year
//! (money) and the island's output (the same year's `gdp_nom`, twelve months);
//! the bill in its three parts as the books keep them (the works' materials and
//! output, the land's produce to market, the trade between towns); the weight
//! carried times the squares it went, by cart, boat, rail and sea (the traffic
//! record); the carting's work, the share of its hauling lorries did, the hands
//! an hour of its work took (1 by hand) and its cost over the wage; the squares
//! of land by road level; the towns' share of people, the people and the linked
//! real wage; and the motor vehicles made and the lorry's find year.
#![allow(non_snake_case)]
use std::io::Write as _;

use isostate_core::params::Rules;

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
        let Cn = &sim.content;
        let y = sim.year - 1;
        if y < 1890 {
            return;
        }
        let k = (y - Cn.start_year) as usize;
        let mean = |name: &str| -> f64 {
            let Some(i) = st.series.names.iter().position(|n| n == name) else { return f64::NAN };
            let v = &st.series.data[i];
            if v.len() >= k * 12 + 12 { v[k * 12..k * 12 + 12].iter().sum::<f64>() / 12.0 } else { f64::NAN }
        };
        let warm = sim.params.warmup_years.max(0) as usize;
        let (bill, carters) = st.cart_years.get(warm + k).map_or((f64::NAN, f64::NAN), |&(b, c)| (b, c / 12.0));
        let ty = st.tasks.years.last().filter(|t| t.year == y);
        let hands_all = ty.map_or(f64::NAN, |t| t.hands.iter().sum::<f64>() + t.minders + t.land_hands.iter().sum::<f64>() + t.land_minders + t.carters);
        let parts = st.acct.carting_parts(k).unwrap_or([f64::NAN; 3]);
        let tf = st.tf.years.iter().find(|t| t.year == y);
        let ws = tf.map_or([f64::NAN; 4], |t| t.weight_squares);
        let roads = tf.map_or([0usize; 4], |t| t.road_squares);
        let lorry = Cn.tasks.uses.iter().position(|u| u.name == "lorry");
        let found = lorry.and_then(|u| st.tasks.found.get(u).copied()).unwrap_or(-1);
        let (cw, cl, ch, ck) = ty.map_or((0.0, 0.0, 1.0, 1.0), |t| if t.cart_work > 0.0 { (t.cart_work, t.cart_held, t.cart_hands, t.cart_kappa) } else { (0.0, 0.0, 1.0, 1.0) });
        rows.push(format!(
            "{seed},{y},{carters},{hands_all},{bill},{},{},{},{},{},{},{},{},{cw},{cl},{ch},{ck},{},{},{},{},{},{},{},{},{found}",
            12.0 * mean("gdp_nom"),
            parts[0], parts[1], parts[2],
            ws[0], ws[1], ws[2], ws[3],
            roads[0], roads[1], roads[2], roads[3],
            mean("urban_share"), mean("pop"), mean("real_wage_linked"),
            12.0 * mean("out_motor_vehicles"),
        ));
    });
    if let Err(e) = res {
        eprintln!("seed {seed}: FAILED {e}");
    }
    let mut f = std::fs::File::create(&out).unwrap();
    writeln!(f, "seed,year,carters,hands_all,bill,gdp,cart_works,cart_land,cart_trade,ws_cart,ws_boat,ws_rail,ws_sea,cart_work,lorry_share,cart_hands,cart_kappa,road_none,road_track,road_made,road_turnpike,urban_share,pop,real_wage_linked,motor_vehicles_made,lorry_found").unwrap();
    for r in rows {
        writeln!(f, "{r}").unwrap();
    }
}
