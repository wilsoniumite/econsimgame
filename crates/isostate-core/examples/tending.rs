//! The works by kind and the hands by class, a row a year from 1900, for the
//! tending machines (docs/rules/tending.md): one island under the game's rules
//! with six families to 2050, with any setting by name.
//!
//!     cargo run --release -p isostate-core --example tending -- SEED OUT.csv [key=value ...]
//!
//! The machines that tend stand off under the game's rules: give
//! `tasks.tending=true` to play them.
//!
//! Each row is one year and one kind of works (`data/tasks.toml`): the works'
//! hands (the year's mean of the months), what they made (each method's output
//! at the island's mean price of its good in that year, and at the prices of
//! 1950, which the reader may take as fixed), and the number of works; then,
//! once for the year (on the kind "all"), the hands by class in the works and
//! what machines did there, the land's hands and the carters, the linked real
//! wage, and each tending use's find year.
#![allow(non_snake_case)]
use std::collections::BTreeMap;
use std::io::Write as _;

use isostate_core::params::Rules;
use isostate_core::tasks::{CLASSES, K};

const KINDS: &[(&str, &[&str])] = &[
    ("furnaces", &["bloomery", "charcoal_furnace", "coke_furnace", "hot_blast", "crucible_steel", "bessemer", "forge_rails", "rolling_mill", "steel_rolling"]),
    ("mills", &["textile_mill", "brickworks", "continuous_kiln", "steam_sawmill", "lime_kiln", "cement_works", "brewhouse", "steam_brewery", "potbank", "pottery_works", "gasworks", "edge_tool_works", "tool_factory", "soap_boilers", "alkali_works", "ammonia_soda", "glasshouse", "sheet_glass", "tank_furnace", "paper_mill", "machine_paper"]),
    ("engines", &["millwright", "engine_works", "machine_shop", "engineering_works"]),
    ("hand_trades", &["smithy", "hand_loom", "saw_pit", "charcoal_burning"]),
    ("builders", &["masons", "carpenters", "bricklayers", "contract_builders", "steel_frame"]),
    ("power", &["coal_power_station", "electrical_works", "oil_from_coal", "nitrogen_works"]),
    ("shops", &["services", "emporium"]),
    ("offices", &["offices"]),
];

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
    let kind_of = |name: &str| KINDS.iter().position(|(_, ms)| ms.contains(&name));
    let mut hands = vec![0.0f64; 0];
    let mut months = 0.0f64;
    let mut rows: Vec<String> = Vec::new();
    // the 1950 prices, filled in that year and read back for every row at the end
    let mut years: Vec<(i64, Vec<f64>, Vec<f64>, Vec<f64>, Vec<usize>, String)> = Vec::new();
    let mut p1950: Vec<f64> = Vec::new();
    // the run's own content (the tree's overlay laid), kept for the rows
    let mut run_content: Option<isostate_core::content::Content> = None;
    let res = isostate_core::edits::play_noting(&P, C.clone(), &[], |sim, month| {
        let st = &sim.st;
        let Cn = &sim.content;
        if hands.len() != Cn.M {
            hands = vec![0.0; Cn.M];
            run_content = Some(Cn.clone());
        }
        if month.is_some() {
            if sim.year >= 1900 {
                for f in 0..st.f_alive.len().min(st.f_labor.len()) {
                    if st.f_alive[f] {
                        hands[st.f_method[f]] += st.f_labor[f];
                    }
                }
                months += 1.0;
            }
            return;
        }
        let y = sim.year - 1;
        if y < 1900 {
            return;
        }
        let k = (y - Cn.start_year) as usize;
        let mean = |name: &str| -> f64 {
            let Some(i) = st.series.names.iter().position(|n| n == name) else { return f64::NAN };
            let v = &st.series.data[i];
            if v.len() >= k * 12 + 12 { v[k * 12..k * 12 + 12].iter().sum::<f64>() / 12.0 } else { f64::NAN }
        };
        let price: Vec<f64> = (0..Cn.G).map(|g| mean(&format!("price_{}", Cn.good_name[g]))).collect();
        let outm: Vec<f64> = (0..Cn.M).map(|m| mean(&format!("mout_{}", Cn.method_name[m]))).collect();
        let n_works: Vec<usize> = (0..Cn.M).map(|m| (0..st.f_alive.len()).filter(|&f| st.f_alive[f] && st.f_method[f] == m).count()).collect();
        if y == 1950 {
            p1950 = price.clone();
        }
        let h: Vec<f64> = hands.iter().map(|v| v / months.max(1.0)).collect();
        // the year's classes, machines and the rest
        let mut extra = String::new();
        if let Some(ty) = st.tasks.years.last() {
            for c in 0..K {
                extra += &format!(",{},{},{}", ty.work[c], ty.held[c], ty.hands[c]);
            }
            extra += &format!(",{},{},{},{}", ty.minders, ty.land_hands.iter().sum::<f64>(), ty.land_minders, ty.carters);
            extra += &format!(",{}", ty.land_hands[1]);
        } else {
            extra += &",NaN".repeat(3 * K + 5);
        }
        extra += &format!(",{},{}", mean("real_wage_linked"), mean("real_wage_chained"));
        let found: BTreeMap<&str, i64> = Cn.tasks.uses.iter().enumerate().map(|(u, us)| (us.name.as_str(), st.tasks.found.get(u).copied().unwrap_or(-1))).collect();
        for n in ["moving_line", "automatic_machine", "robot", "lights_out_line"] {
            extra += &format!(",{}", found.get(n).copied().unwrap_or(-1));
        }
        years.push((y, h, outm, price, n_works, extra));
        hands.iter_mut().for_each(|v| *v = 0.0);
        months = 0.0;
    });
    if let Err(e) = res {
        eprintln!("seed {seed}: FAILED {e}");
    }
    let Cn = run_content.as_ref().unwrap_or(&C);
    let g_of = |m: usize| Cn.method_out[m];
    let mut head = String::from("seed,year,kind,hands,made,made_1950,works");
    for c in CLASSES {
        head += &format!(",work_{c},held_{c},hands_{c}");
    }
    head += ",minders,land_hands,land_minders,carters,land_tend,real_wage_linked,real_wage_chained,found_moving_line,found_automatic_machine,found_robot,found_lights_out_line";
    for (y, h, outm, price, nw, extra) in &years {
        let mut tot = [0.0f64; 4];
        for (ki, (kn, _)) in KINDS.iter().enumerate() {
            let mut s = [0.0f64; 4];
            for m in 0..Cn.M.min(h.len()) {
                if kind_of(&Cn.method_name[m]) != Some(ki) {
                    continue;
                }
                let o = if outm[m].is_finite() { outm[m] } else { 0.0 };
                s[0] += h[m];
                s[1] += o * price[g_of(m)];
                s[2] += o * p1950.get(g_of(m)).copied().unwrap_or(f64::NAN);
                s[3] += nw[m] as f64;
            }
            for i in 0..4 {
                tot[i] += s[i];
            }
            rows.push(format!("{seed},{y},{kn},{},{},{},{}{}", s[0], s[1], s[2], s[3], ",".repeat(3 * K + 11)));
        }
        rows.push(format!("{seed},{y},all,{},{},{},{}{extra}", tot[0], tot[1], tot[2], tot[3]));
    }
    let mut f = std::fs::File::create(&out).unwrap();
    writeln!(f, "{head}").unwrap();
    for r in rows {
        writeln!(f, "{r}").unwrap();
    }
}
