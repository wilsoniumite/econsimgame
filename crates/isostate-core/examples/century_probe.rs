//! Where the merchants' living goes, and why the island is rich by 1800
//! (the century probe, 27 November 2026; docs/rules/prices.md, "What is carried
//! out is asked"). One island under the game's rules with six families, any
//! setting by name, a line a year:
//!
//!     cargo run --profile work -p isostate-core --example century_probe -- SEED OUT.jsonl [--start settled|landfall] [--set key=value]... [--end YEAR]
//!
//! **The merchants.** The probe sets `State::watch_merchants`, and `econ::step_month`
//! keeps each town's merchants' month (`econ::MERCHANT_TERMS`), summed here over
//! the towns and the year. Their income is what the books count, their takings
//! less what they paid out (`net`). It is the sum of
//!
//! - `spread`: the spread they keep on what the land and the works make
//!   (`merchant_spread` times its worth at the town's prices);
//! - `gap`: what they brought in, at the prices of the town it came to, less what
//!   they paid for it at the prices of the towns it came from;
//! - `carriage`: less the carting and handling they paid to bring it;
//! - `spoiled`: less what was lost of the stock;
//! - `stock_grew`: less what their stock gained in worth over the year, at the
//!   prices of its months (cash they put into the shelves);
//! - `held`: what the stock they carried from month to month gained as prices
//!   moved under it (and what came and went with districts at the year's turn);
//! - `under_price`: what buyers paid under the town's prices (a works that buys
//!   an input cheaper than the town's price, carted to it from a cheaper town);
//! - `rest`: what the stock's floor at nothing made up, which should be nothing.
//!
//! **The wage in grain.** The island's mean wage, by people, over its mean price of
//! grain, by people (as `observer::national_prices`), a year's means of months; and
//! the same split into seven terms that multiply to it exactly:
//!
//! - grain made a hand on the farms (a month), itself split unit by unit as the
//!   land's rule makes it, `A f lam^alpha L^(1 - alpha)`, into hand-weighted
//!   geometric means: the ideas found (the town's tools and husbandry), the
//!   machines and the rest of `A` (improvements of the ground, the families' land),
//!   the weather (`harvest_mult`), the field's quality `f`, the land a hand
//!   `(lam / L)^alpha`, and the spread between units;
//! - the farms' share of the land's hands; the land's share of the hands that
//!   make (all at work less the carters); the share of hands not carting;
//! - the island's income over the worth of its grain at the island's price of
//!   grain (what the land's other goods, the works and the rest add), split into
//!   grain's price where it is made over the island's, the worth of all the land
//!   makes over that of its grain, and income over the worth of all the land makes;
//! - labour's share of income (the books' wage bill over income);
//! - the mean wage over the wage bill a hand (whom the mean by people weighs).
#![allow(non_snake_case)]
use isostate_core::*;
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mut P = game_params();
    P.seed = a[0].parse().expect("SEED");
    P.n_families = 6;
    P.end_year = Some(2050);
    let out = a[1].clone();
    let mut i = 2;
    let mut sets: Vec<(String, String)> = Vec::new();
    while i < a.len() {
        match a[i].as_str() {
            "--start" => {
                landfall::set_start(&mut P, &a[i + 1]).unwrap();
                i += 1;
            }
            "--set" => {
                let (k, v) = a[i + 1].split_once('=').expect("--set key=value");
                sets.push((k.to_string(), v.to_string()));
                i += 1;
            }
            "--end" => {
                P.end_year = Some(a[i + 1].parse().unwrap());
                i += 1;
            }
            s => panic!("unknown argument {s}"),
        }
        i += 1;
    }
    for (k, v) in &sets {
        P = sweep::set_field(&P, k, &sweep::parse_value(v)).unwrap();
    }
    let mut fy = std::fs::File::create(&out).unwrap();
    let mut sim = sim::start(&P, game_content());
    sim.st.watch_merchants = true;
    let C = sim.content.clone();
    let T = sim.st.T_max;
    let grain = C.gi("grain");
    let farms: Vec<usize> = (0..C.NS).filter(|&s| C.sector_good[s] == grain).collect();
    let spread = sim.params.merchant_spread;
    let husbandry = sim.params.husbandry_boost;
    let K = econ::MERCHANT_TERMS.len();
    let col = |n: &str| econ::MERCHANT_TERMS.iter().position(|&c| c == n).unwrap();
    let mut close_was = f64::NAN;
    while sim.year <= sim.end_year {
        let year = sim.year;
        // the merchants' year, island sums of each column, and the stock's months
        let mut m = vec![0.0; K];
        let (mut open0, mut turn) = (f64::NAN, f64::NAN);
        // the wage's year
        let (mut wage, mut pgrain) = (0.0, 0.0);
        let (mut made, mut farm_hands, mut land_hands, mut at_work, mut carters) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let (mut v_grain, mut v_land) = (0.0, 0.0);
        // hand-weighted sums of the logs of each factor of the farms' output a hand
        let mut logs = [0.0f64; 6];
        for month in 0..12 {
            // the prices the month trades at
            let price = sim.st.price.clone();
            let pop: f64 = (0..T).filter(|&t| sim.st.town_active[t]).map(|t| sim.st.pop[t]).sum();
            pgrain += (0..T).filter(|&t| sim.st.town_active[t]).map(|t| sim.st.pop[t] * price[(t, grain)]).sum::<f64>() / pop.max(1.0);
            if let Err(e) = sim::run_month(&mut sim, month) {
                writeln!(fy, "{}", serde_json::json!({"year": year, "failed": e})).unwrap();
                return;
            }
            let st = &sim.st;
            let mm = &st.month_merchants;
            let (mut open, mut close) = (0.0, 0.0);
            for t in 0..mm.rows {
                for k in 0..K {
                    m[k] += mm[(t, k)];
                }
                open += mm[(t, col("stock_open"))];
                close += mm[(t, col("stock_close"))];
            }
            if month == 0 {
                open0 = open;
                // what the stock gained over the year's turn, outside any month's cash
                turn = open - close_was;
            }
            close_was = close;
            // the wage, by people, and the hands
            let pop: f64 = (0..T).filter(|&t| st.town_active[t]).map(|t| st.pop[t]).sum();
            wage += (0..T).filter(|&t| st.town_active[t]).map(|t| st.pop[t] * st.wage[t]).sum::<f64>() / pop.max(1.0);
            at_work += st.month_stats.employment.iter().sum::<f64>();
            carters += st.carters.iter().sum::<f64>();
            land_hands += st.month_stats.ex_labor_by_sector.iter().sum::<f64>();
            for u in 0..st.ex_out.len() {
                let (s, t) = (st.ex_sector[u], st.ex_town[u]);
                let y = st.ex_out[u];
                v_land += y * price[(t, C.sector_good[s])];
                if !farms.contains(&s) {
                    continue;
                }
                v_grain += y * price[(t, grain)];
                made += y;
                let l = st.ex_labor[u];
                farm_hands += l;
                if l <= 0.0 || y <= 0.0 {
                    continue;
                }
                let alpha = C.sector_alpha[s];
                let ideas = st.tools_mult[t] * (1.0 + husbandry * st.know[(t, st.t_husb)]);
                let weather = if st.harvest_mult.rows > s && st.harvest_mult.cols > st.ex_cell[u] { st.harvest_mult[(s, st.ex_cell[u])] } else { 1.0 };
                let field = st.ex_field[u];
                let land = (st.ex_lam[u] / l).powf(alpha);
                let rest = (y / l) / (C.sector_A[s] * ideas * weather * field * land);
                for (k, x) in [ideas, rest, weather, field, land, y / l].into_iter().enumerate() {
                    logs[k] += l * x.ln();
                }
            }
        }
        sim::end_year(&mut sim);
        let iy = accounts::island_year(&sim.st);
        let (income, wage_bill, merchants) = iy.as_ref().map_or((f64::NAN, f64::NAN, f64::NAN), |y| (y.income, y.wage_bill, y.merchants_div + y.merchants_kept));
        // the merchants' terms
        let net = m[col("net")];
        // the stock's months, open less close summed over the year: what the stock
        // lost in worth from January to December, and what it gained between months
        let stock_grew = open0 - close_was;
        let held = m[col("stock_open")] - m[col("stock_close")] - stock_grew;
        let mut terms = vec![
            ("spread", spread * m[col("made")]),
            ("gap", m[col("brought_value")] - m[col("brought_cost")]),
            ("carriage", -m[col("carriage")]),
            ("spoiled", -m[col("spoiled")]),
            ("stock_grew", stock_grew),
            ("held", held),
            ("under_price", m[col("sales")] - m[col("at_price")]),
        ];
        let sum: f64 = terms.iter().map(|t| t.1).sum();
        terms.push(("rest", net - sum));
        // the wage's terms
        let n = 12.0;
        let (w, pg) = (wage / n, pgrain / n);
        let (lf, ll, la, lc) = (farm_hands / n, land_hands / n, at_work / n, carters / n);
        let geo: Vec<f64> = logs.iter().map(|x| (x / farm_hands.max(1e-12)).exp()).collect();
        let a_hand = made / farm_hands.max(1e-12);
        let wage_terms = serde_json::json!({
            "grain_a_hand": a_hand,
            "ideas": geo[0], "machines_rest": geo[1] * C.sector_A[farms[0]], "weather": geo[2], "field": geo[3], "land_a_hand": geo[4],
            "spread_of_units": a_hand / geo[5],
            "farms_of_land": lf / ll.max(1e-12),
            "land_of_making": ll / (la - lc).max(1e-12),
            "not_carting": (la - lc) / la.max(1e-12),
            "grain_price_here": v_grain / (pg * made).max(1e-12),
            "land_over_grain": v_land / v_grain.max(1e-12),
            "income_over_land": income / v_land.max(1e-12),
            "labour_share": wage_bill / income,
            "wage_over_bill": n * w * la / wage_bill,
        });
        let row = serde_json::json!({
            "year": year, "seed": sim.params.seed,
            "income": income, "merchants": merchants, "merchants_net": net,
            "share_merchants": merchants / income,
            "terms": terms.iter().map(|(k, v)| serde_json::json!([k, v])).collect::<Vec<_>>(),
            "columns": econ::MERCHANT_TERMS.iter().zip(&m).map(|(k, v)| serde_json::json!([k, v])).collect::<Vec<_>>(),
            "stock_open_jan": open0, "stock_close_dec": close_was, "stock_turn": turn,
            "wage": w, "price_grain": pg, "wage_in_grain": w / pg,
            "made_grain": made, "farm_hands": lf, "land_hands": ll, "at_work": la, "carters": lc,
            "wage_terms": wage_terms,
        });
        writeln!(fy, "{row}").unwrap();
    }
}
