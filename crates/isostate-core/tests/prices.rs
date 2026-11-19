//! `prices`: merchants who mark prices with their stock and stop buying into a
//! glut (docs/rules/prices.md). That the port does not move while it is off is
//! `tests/golden.rs`; here, that it does what it says while it is on.
#![allow(non_snake_case)]

use isostate_core::econ::{money_total, step_month};
use isostate_core::prices::{self, PriceParams};
use isostate_core::sim::{self, RunResult, Sim};
use isostate_core::{Params, default_content, default_params};

fn params(seed: u64, end_year: i64, prices: PriceParams) -> Params {
    let mut P = default_params();
    P.seed = seed;
    P.end_year = Some(end_year);
    P.prices = prices;
    P
}

fn run(P: &Params) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(P, default_content(), false, true).expect("money conserved")
}

fn run_to(P: &Params, year: i64) -> Sim {
    let mut s = sim::start(P, default_content());
    while s.year <= year {
        sim::run_year(&mut s).expect("money conserved");
    }
    s
}

fn same_series(a: &RunResult, b: &RunResult) -> bool {
    a.state.series.data.iter().zip(&b.state.series.data).all(|(x, y)| x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()))
}

#[test]
fn forty_years_conserve_money_and_come_out_the_same_twice() {
    let P = params(3, 1790, PriceParams::game());
    let (a, b) = (run(&P), run(&P));
    assert!(same_series(&a, &b), "two runs with prices on differ");
    assert_eq!(a.events.iter().map(|e| &e.title).collect::<Vec<_>>(), b.events.iter().map(|e| &e.title).collect::<Vec<_>>());
    // and it is a different history from the port's
    let port = run(&params(3, 1790, PriceParams::default()));
    assert!(!same_series(&a, &port), "prices on changed nothing");
    assert_eq!(port.state.price_anchor.rows, 0, "the port keeps no anchors");
    assert_eq!(a.state.price_anchor.rows, a.state.price.rows);
}

#[test]
fn twenty_years_with_families_conserve_money() {
    let mut P = params(5, 1770, PriceParams::game());
    P.n_families = 6;
    run(&P);
}

#[test]
fn the_markup_steadies_iron_and_the_farm_share() {
    // one seed and fifty years: a smoke test of the direction, not the measurement
    let swing = |r: &RunResult, name: &str| -> f64 {
        let y0 = r.content.start_year;
        let v: Vec<f64> = (0..r.rec.n_years).map(|k| isostate_core::observer::annual(r.state.series.get(name), y0, y0 + k as i64)).collect();
        let d: Vec<f64> = v[10..].windows(2).map(|w| (w[1] / w[0]).ln()).collect();
        let m = d.iter().sum::<f64>() / d.len() as f64;
        (d.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / d.len() as f64).sqrt()
    };
    let port = run(&params(7, 1800, PriceParams::default()));
    let on = run(&params(7, 1800, PriceParams::game()));
    for name in ["price_iron", "out_iron", "ag_share"] {
        let (a, b) = (swing(&port, name), swing(&on, name));
        assert!(b < 0.8 * a, "{name}: yearly swing {b:.3} with prices on, {a:.3} under the port");
    }
}

#[test]
fn a_price_stands_off_its_anchor_with_the_stock() {
    let P = params(3, 1752, PriceParams::game());
    let s = run_to(&P, 1751);
    let C = &s.content;
    let (t, g) = (0, C.gi("iron"));
    let (lo, hi) = (P.price_floor * C.base_price[g], P.price_ceiling * C.base_price[g]);
    let mark = |P: &Params, gap: f64, imbalance: f64| -> (f64, f64) {
        let mut st = s.st.clone();
        st.price_anchor[(t, g)] = 6.0;
        st.price[(t, g)] = 6.0;
        prices::mark(&mut st, P, t, g, gap, imbalance, lo, hi, P.decay[g]);
        (st.price_anchor[(t, g)], st.price[(t, g)])
    };
    // the anchor moves as the port's price does
    let port_step = |gap: f64, imb: f64| 6.0 * (P.price_speed * gap + P.price_flow * imb).clamp(-0.12, 0.12).exp();
    for (gap, imb) in [(0.4, 0.2), (-0.7, -1.0), (1.0, 1.0), (0.0, 0.0)] {
        let (a, _) = mark(&P, gap, imb);
        assert_eq!(a.to_bits(), port_step(gap, imb).to_bits());
    }
    // thin shelves: the price goes above the anchor; full ones: below; at the target stock: on it
    let (a, p) = mark(&P, 0.2, 0.0);
    assert!((p / a - (0.5f64 * 0.2).exp()).abs() < 1e-12);
    let (a, p) = mark(&P, -0.2, 0.0);
    assert!((p / a - (-0.5f64 * 0.2).exp()).abs() < 1e-12);
    let (a, p) = mark(&P, 0.0, 0.0);
    assert!((p / a - 1.0).abs() < 1e-12);
    // never more than 12 per cent in a month
    let (_, p) = mark(&P, 1.0, 1.0);
    assert!((p / 6.0 - 0.12f64.exp()).abs() < 1e-12);
    // with no markup the price is the anchor, which is the port's rule
    let mut flat = P.clone();
    flat.prices.markup = 0.0;
    let (a, p) = mark(&flat, 0.4, 0.2);
    assert!((p - a).abs() < 1e-12 && a.to_bits() == port_step(0.4, 0.2).to_bits());
}

/// Under `prices.no_spoil_markup` a good that spoils (services, half a month)
/// is priced at its anchor whatever the shelves hold, while a good that keeps
/// (iron) is marked as before; with the switch off services are marked too.
#[test]
fn a_good_that_spoils_carries_no_markup() {
    let P = params(3, 1752, PriceParams::game());
    assert!(P.prices.no_spoil_markup, "the game's merchants");
    let s = run_to(&P, 1751);
    let C = &s.content;
    let t = 0;
    let mark = |P: &Params, g: usize, gap: f64| -> (f64, f64) {
        let mut st = s.st.clone();
        st.price_anchor[(t, g)] = 6.0;
        st.price[(t, g)] = 6.0;
        let (lo, hi) = (P.price_floor * C.base_price[g], P.price_ceiling * C.base_price[g]);
        prices::mark(&mut st, P, t, g, gap, 0.0, lo, hi, P.decay[g]);
        (st.price_anchor[(t, g)], st.price[(t, g)])
    };
    // which goods spoil, under the game's content: services alone
    let spoil: Vec<&str> = (0..C.G).filter(|&g| P.decay[g] > prices::SPOILS).map(|g| C.good_name[g].as_str()).collect();
    assert_eq!(spoil, ["services"]);
    let (services, iron) = (C.gi("services"), C.gi("iron"));
    // (gaps small enough that the month's step of at most 12 per cent does not bind)
    for gap in [0.2, -0.2] {
        let (a, p) = mark(&P, services, gap);
        assert_eq!(p.to_bits(), a.to_bits(), "services at gap {gap}: priced at the anchor");
        let (a, p) = mark(&P, iron, gap);
        assert!((p / a - (0.5 * gap).exp()).abs() < 1e-12, "iron at gap {gap}: marked as before");
    }
    // the switch off: services are marked with the rest
    let mut off = P.clone();
    off.prices.no_spoil_markup = false;
    let (a, p) = mark(&off, services, 0.2);
    assert!((p / a - 0.1f64.exp()).abs() < 1e-12);
    // and the anchor moves the same way either way
    assert_eq!(mark(&off, services, 0.2).0.to_bits(), mark(&P, services, 0.2).0.to_bits());
}

#[test]
fn a_new_town_takes_its_mother_towns_anchors() {
    let P = params(3, 1752, PriceParams::game());
    let s = run_to(&P, 1750);
    let (C, mut st) = (s.content.clone(), s.st);
    let n = (0..st.T_max).find(|&t| !st.town_active[t]).expect("room for a town");
    let G = st.price.cols;
    assert!((0..G).any(|g| st.price_anchor[(n, g)] != st.price_anchor[(0, g)]), "the empty slot already matches");
    assert!((0..G).any(|g| st.pexp[(n, g)] != st.pexp[(0, g)]), "the empty slot already expects her prices");
    // a site on her doorstep: her anchors, and exactly what her people expect
    let home = st.town_node[0];
    assert_eq!(st.dcatch[home], 0.0);
    prices::found_town(&mut st, &C, &P, n, 0, home);
    assert!((0..G).all(|g| st.price_anchor[(n, g)] == st.price_anchor[(0, g)]));
    assert!((0..G).all(|g| st.pexp[(n, g)] == st.pexp[(0, g)]));
    // a site at the far edge of her country: what a load from there fetched,
    // her expected price less the carriage, and never under the floor
    let site = (0..st.N).filter(|&c| st.catch[c] == 0 && st.dcatch[c].is_finite()).max_by(|&a, &b| st.dcatch[a].total_cmp(&st.dcatch[b])).unwrap();
    assert!(st.dcatch[site] > 5.0, "her country is too small to tell");
    prices::found_town(&mut st, &C, &P, n, 0, site);
    assert!((0..G).all(|g| st.price_anchor[(n, g)] == st.price_anchor[(0, g)]));
    for g in 0..G {
        let net = st.pexp[(0, g)] - P.freight * C.weight[g] * st.dcatch[site];
        assert_eq!(st.pexp[(n, g)], net.max(P.price_floor * C.base_price[g]), "{}", C.good_name[g]);
        assert!(st.pexp[(n, g)] > 0.0);
    }
    let coal = C.gi("coal");
    assert!(st.pexp[(n, coal)] < st.pexp[(0, coal)], "coal is heavy: it fetched less out there");
    // and nothing at all while the mechanism is off
    let off = params(3, 1752, PriceParams::default());
    let mut st = run_to(&off, 1750).st;
    let stale = st.pexp.clone();
    prices::found_town(&mut st, &C, &off, n, 0, site);
    assert_eq!(st.price_anchor.rows, 0);
    assert_eq!(st.pexp, stale);
}

/// The same through a run's own year end, which is what holds the one line in
/// `people::found_towns`: without it the new town's row would still be the
/// goods' base prices, as every empty slot's is from the first month.
#[test]
fn a_town_founded_in_a_run_starts_from_another_towns_anchors() {
    let P = params(3, 1770, PriceParams::game());
    let mut s = sim::start(&P, default_content());
    let G = s.st.price.cols;
    loop {
        assert!(s.year <= 1765, "no town was founded by 1765 on this seed");
        let year = s.year;
        sim::run_year(&mut s).expect("money conserved");
        let st = &s.st;
        let row = |t: usize| -> Vec<u64> { (0..G).map(|g| st.price_anchor[(t, g)].to_bits()).collect() };
        let new: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t] && st.town_founded[t] == year).collect();
        for &n in &new {
            let mother = (0..st.T_max).find(|&t| t != n && st.town_active[t] && row(t) == row(n));
            assert!(mother.is_some(), "the town founded in {year} has nobody's anchors");
            assert!((0..G).any(|g| st.price_anchor[(n, g)] != s.content.base_price[g]), "its anchors are still the base prices");
            // its people expect what the mother town's do, less the carriage: no more
            // than she does for any good, and less for something (it was founded far off)
            let m = mother.unwrap();
            assert!((0..G).all(|g| st.pexp[(n, g)] <= st.pexp[(m, g)]), "the town founded in {year} expects more than its mother town for something");
            assert!((0..G).any(|g| st.pexp[(n, g)] < st.pexp[(m, g)]), "the town founded in {year} expects the carriage to cost nothing");
        }
        if !new.is_empty() {
            break;
        }
    }
}

/// Towns and goods where some works is making the good.
fn made_somewhere(s: &Sim) -> Vec<(usize, usize)> {
    let (st, C) = (&s.st, &s.content);
    let mut v: Vec<(usize, usize)> = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_K[f] > 0.0).map(|f| (st.f_town[f], C.method_out[st.f_method[f]])).collect();
    v.sort();
    v.dedup();
    v
}

#[test]
fn merchants_with_long_stocks_take_less_and_the_works_hire_less() {
    let P = params(3, 1760, PriceParams::game());
    let s = run_to(&P, 1755);
    let C = &s.content;
    let iron = C.gi("iron");
    let (t, _) = *made_somewhere(&s).iter().find(|&&(_, g)| g == iron).expect("an iron town");
    let iron_works: Vec<usize> = (0..s.st.F_count).filter(|&f| s.st.f_alive[f] && s.st.f_town[f] == t && C.method_out[s.st.f_method[f]] == iron).collect();
    let step = |P: &Params, months: f64| -> (f64, f64, f64) {
        let mut st = s.st.clone();
        st.inv[(t, iron)] = months * st.outflow[(t, iron)];
        let (before, issued) = (money_total(&st), st.issued);
        step_month(&mut st, C, P);
        let drift = money_total(&st) - before - (st.issued - issued);
        assert!(drift.abs() < 1e-6 * before, "money drifted by {drift}");
        (iron_works.iter().map(|&f| st.f_labor[f]).sum(), st.town_prod[(t, iron)] - s.st.town_prod[(t, iron)], st.wage[t])
    };
    let (hands_short, made_short, wage_short) = step(&P, 3.0);
    let (hands_long, made_long, wage_long) = step(&P, 7.0);
    let (hands_glut, made_glut, _) = step(&P, 30.0);
    assert!(hands_short > 0.0 && made_short > 0.0);
    assert!(hands_long < hands_short && made_long < made_short, "seven months of stock should mean short time");
    assert!(hands_glut < hands_long && made_glut < made_long);
    assert!(wage_long <= wage_short, "works on short time do not bid the wage up");
    // with the brake off the stock makes no difference to this month's work
    let mut free = P.clone();
    free.prices.glut_months = 0.0;
    let (a, b) = (step(&free, 3.0), step(&free, 30.0));
    assert_eq!((a.0.to_bits(), a.1.to_bits()), (b.0.to_bits(), b.1.to_bits()));
    // labour moves a third of the way a month (`labor_adjust_firm`), so one month of the
    // worst glut cuts the hands by about a third, not to nothing
    assert!(hands_glut > 0.5 * hands_short && hands_glut < 0.8 * hands_short, "{hands_glut} against {hands_short}");
}

/// The wage root-finder (`econ::excess_demand_root`) takes Newton steps on excess
/// demand and tells the bisection which way to go at midpoints far from the root.
/// That is only right if excess demand falls as the wage rises. The merchants'
/// take multiplies each works' utilisation by a number fixed before the wage is
/// sought, so it should be; here the shortcut is held to the plain bisection,
/// every midpoint evaluated, month by month, in years when the brake is biting.
#[test]
fn the_wage_shortcut_agrees_with_the_plain_bisection() {
    // a brake that bites often, to give the check something to chew on, and the game's
    // (the harsh one had a floor of a tenth of capacity until the floor became a
    // constant, nothing, in stage 5 of the pass over the rules)
    let harsh = PriceParams { glut_months: 1.0, glut_stop_months: 2.5, ..PriceParams::game() };
    for (pp, families) in [(harsh, 0), (PriceParams::game(), 0), (PriceParams::game(), 6)] {
        let mut P = params(11, 1760, pp);
        P.n_families = families;
        let mut s = run_to(&P, 1753);
        let C = s.content.clone();
        let (mut biting, mut months) = (0, 0);
        for _ in 0..3 {
            for month in 0..12 {
                let fi: Vec<usize> = (0..s.st.F_count).filter(|&f| s.st.f_alive[f] && s.st.f_K[f] > 1e-9).collect();
                let ft: Vec<usize> = fi.iter().map(|&f| s.st.f_town[f]).collect();
                let out: Vec<usize> = fi.iter().map(|&f| C.method_out[s.st.f_method[f]]).collect();
                let take = prices::merchants_take(&s.st, &P, &ft, &out);
                biting += take.iter().filter(|&&v| v < 1.0).count();
                let mut plain = s.st.clone();
                plain.plain_bisection = true;
                step_month(&mut plain, &C, &P);
                sim::run_month(&mut s, month).expect("money conserved");
                for t in 0..s.st.T_max {
                    assert_eq!(s.st.wage[t].to_bits(), plain.wage[t].to_bits(), "{}-{month}, town {t}: {} against {}", s.year, s.st.wage[t], plain.wage[t]);
                }
                months += 1;
            }
            // the year end, as sim::run_year has it
            let year = s.year;
            let Sim { st, content, params, rngs, events, .. } = &mut s;
            sim::yearly(st, content, params, rngs, year, events, false);
            s.year += 1;
        }
        assert_eq!(months, 36);
        if families == 0 && P.prices.glut_months < 2.0 {
            assert!(biting > 100, "the harsh brake bit only {biting} works-months: the check saw too little");
        }
    }
}

/// The brake on a glut leans on two numbers it does not own: the stock
/// merchants mean to hold (`price_target_months`) and the most a town may ship
/// in a month (`trade_cap`), which sets the least stock a town that only ships
/// holds. A brake that is full on by the larger of the two is refused where the
/// settings are read, in a sentence that names them; one that starts below it
/// and is still easing off there is not, since such brakes measured as well as
/// the game's.
#[test]
fn the_brake_moves_with_the_two_numbers_it_leans_on() {
    let game = isostate_core::game_params();
    // the game's brake is full on well above both
    assert!((prices::brake_floor(&game) - 1.0 / 0.3).abs() < 1e-12, "the floor is 1 / trade_cap at the game's numbers");
    assert!(game.prices.glut_stop_months > prices::brake_floor(&game));
    assert_eq!(game.check(), Ok(()));
    // a cap on shipments of an eighth means a town that only ships holds eight months,
    // where the game's brake is full on: refused
    let err = game.with_json_overrides(r#"{"trade_cap": 0.125}"#).expect_err("a brake full on at 8 months, on a floor of 8");
    assert!(err.contains("prices.glut_stop_months is 8") && err.contains("above 8 months") && err.contains("trade_cap of 0.125 "), "{err}");
    // and so is a stock target where the brake is full on
    let err = game.with_toml_overrides("price_target_months = 8.0\n").expect_err("a brake full on at the target stock");
    assert!(err.contains("prices.glut_stop_months") && err.contains("price_target_months (8)"), "{err}");
    // and the brake the prices page saw choke every works, full on at 0.6 months
    let err = game.with_json_overrides(r#"{"prices": {"glut_months": 0.5, "glut_stop_months": 0.6}}"#).expect_err("a brake full on under the target");
    assert!(err.contains("prices.glut_stop_months is 0.6"), "{err}");
    // brakes that start below the floor and still ease off above it are read: the
    // page's 0.5 to 40, the sensitivity probe's 2 and 3, and the cliff at 4
    for over in [r#"{"prices": {"glut_months": 0.5, "glut_stop_months": 40}}"#, r#"{"prices": {"glut_months": 2}}"#, r#"{"prices": {"glut_months": 3}}"#, r#"{"prices": {"glut_months": 2, "glut_stop_months": 4.5}}"#, r#"{"prices": {"glut_stop_months": 4}}"#] {
        game.with_json_overrides(over).unwrap_or_else(|e| panic!("{over}: {e}"));
    }
    // moved together, it is read
    let moved = game.with_json_overrides(r#"{"trade_cap": 0.125, "prices": {"glut_months": 9, "glut_stop_months": 12}}"#).expect("the brake moved with the cap");
    assert!(moved.prices.glut_stop_months > prices::brake_floor(&moved));
    // with the brake off, or the merchants' prices off, there is nothing to hold
    game.with_json_overrides(r#"{"trade_cap": 0.125, "prices": {"glut_months": 0}}"#).expect("no brake");
    default_params().with_json_overrides(r#"{"trade_cap": 0.125}"#).expect("the port has no brake");
    // a file says which file it was
    let dir = std::env::temp_dir().join(format!("isostate-prices-brake-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("cap.toml");
    std::fs::write(&file, "trade_cap = 0.125\n").unwrap();
    let err = game.load_over(&file).expect_err("the file's cap is under the brake");
    assert!(err.contains("cap.toml") && err.contains("prices.glut_stop_months"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}
