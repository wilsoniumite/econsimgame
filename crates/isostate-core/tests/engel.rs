//! The Engel curve (not in the Python; docs/rules/engel.md): the budget is spent
//! exactly at every income, nothing changes below the reference income, every
//! further shilling is split somewhere between the two sets of shares, demand
//! still falls as a price rises, content can say the richer set, and a run with
//! the curve on conserves money, repeats itself and buys less bread. With the
//! curve off the golden test (tests/golden.rs) covers that nothing changed.
#![allow(non_snake_case)]

use isostate_core::content::{DEFAULT_CONTENT_TOML, content_from_str};
use isostate_core::econ::{self, goods_from_services, service_mix};
use isostate_core::engel::{self, EngelParams, RICH_SHARES};
use isostate_core::mat::Mat;
use isostate_core::observer::annual;
use isostate_core::sim::{self, RunResult};
use isostate_core::{Content, Params, default_content, default_params, game_params};

const STRENGTHS: [f64; 6] = [0.5, 1.0, 2.0, 3.0, 8.0, 60.0];

fn curve(strength: f64) -> EngelParams {
    EngelParams { strength, ..EngelParams::game() }
}

/// One town's prices: the content's base prices, each good scaled by `scale`.
fn prices(C: &Content, scale: impl Fn(usize) -> f64) -> Mat {
    Mat::from_vec(1, C.G, (0..C.G).map(|g| C.base_price[g] * scale(g)).collect())
}

/// A few price lists: the base prices, dear grain, cheap coal and cloth, and a jumble.
fn price_lists(C: &Content) -> Vec<Mat> {
    let (grain, coal, cloth) = (C.gi("grain"), C.gi("coal"), C.gi("cloth"));
    vec![
        prices(C, |_| 1.0),
        prices(C, |g| if g == grain { 2.5 } else { 1.0 }),
        prices(C, |g| if g == coal || g == cloth { 0.3 } else { 1.0 }),
        prices(C, |g| 0.4 + 0.37 * ((g * 7 + 3) % 11) as f64),
    ]
}

fn spending(q: &Mat, unit: &Mat) -> Vec<f64> {
    (0..unit.cols).map(|s| q[(0, s)] * unit[(0, s)]).collect()
}

fn engel_q(C: &Content, E: &EngelParams, budget: f64, unit: &Mat) -> Mat {
    engel::les(&[budget], unit, &C.subsistence, &C.shares, &C.shares_rich, E).0
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn the_budget_is_spent_exactly_at_every_income() {
    let C = default_content();
    for price in price_lists(&C) {
        let (share, unit) = service_mix(&price, &C.props, 0.25);
        let sub: f64 = (0..C.S).map(|s| C.subsistence[s] * unit[(0, s)]).sum();
        for k in STRENGTHS {
            let E = curve(k);
            // from a fifth of subsistence to sixty times it, in steps of 0.3 per cent
            let mut y = 0.2;
            while y < 60.0 {
                let budget = y * sub;
                let q = engel_q(&C, &E, budget, &unit);
                let on_needs: f64 = spending(&q, &unit).iter().sum();
                assert!((on_needs - budget).abs() <= 1e-12 * budget, "strength {k}, income {y}: needs cost {on_needs} of {budget}");
                // and the goods that meet those needs cost the same
                let d = goods_from_services(&q, &share, &C.props);
                let on_goods: f64 = (0..C.G).map(|g| d[(0, g)] * price[(0, g)]).sum();
                assert!((on_goods - budget).abs() <= 1e-12 * budget, "strength {k}, income {y}: goods cost {on_goods} of {budget}");
                assert!((0..C.S).all(|s| q[(0, s)] >= 0.0), "strength {k}, income {y}: a negative quantity");
                y *= 1.003;
            }
        }
    }
}

#[test]
fn up_to_the_reference_income_it_is_the_ports_rule() {
    let C = default_content();
    for price in price_lists(&C) {
        let (_, unit) = service_mix(&price, &C.props, 0.25);
        let sub: f64 = (0..C.S).map(|s| C.subsistence[s] * unit[(0, s)]).sum();
        let E = curve(3.0);
        for y in [0.0, 0.3, 0.999, 1.0, 1.4, 1.999, 2.0] {
            let (port, sub_port) = econ::les(&[y * sub], &unit, &C.subsistence, &C.shares);
            let (q, sub_q) = engel::les(&[y * sub], &unit, &C.subsistence, &C.shares, &C.shares_rich, &E);
            assert_eq!(bits(&q.data), bits(&port.data), "income {y}");
            assert_eq!(bits(&sub_q), bits(&sub_port));
        }
        // and a curve of no strength is the port's rule at every income
        for y in [2.5, 4.0, 30.0] {
            let (port, _) = econ::les(&[y * sub], &unit, &C.subsistence, &C.shares);
            assert_eq!(bits(&engel_q(&C, &curve(0.0), y * sub, &unit).data), bits(&port.data), "income {y}");
        }
        // past the reference it is not: less on food, more on comfort
        let (port, _) = econ::les(&[4.0 * sub], &unit, &C.subsistence, &C.shares);
        let q = engel_q(&C, &E, 4.0 * sub, &unit);
        assert!(q[(0, 0)] < port[(0, 0)] && q[(0, C.S - 1)] > port[(0, C.S - 1)]);
    }
}

#[test]
fn each_further_shilling_is_split_between_the_two_sets() {
    let C = default_content();
    let (_, unit) = service_mix(&prices(&C, |_| 1.0), &C.props, 0.25);
    let sub: f64 = (0..C.S).map(|s| C.subsistence[s] * unit[(0, s)]).sum();
    for k in STRENGTHS {
        let E = curve(k);
        let marginal = |y: f64| -> Vec<f64> {
            let h = 1e-6 * sub;
            let (a, b) = (spending(&engel_q(&C, &E, y * sub - h, &unit), &unit), spending(&engel_q(&C, &E, y * sub + h, &unit), &unit));
            (0..C.S).map(|s| (b[s] - a[s]) / (2.0 * h)).collect()
        };
        let mut y = 1.05;
        let mut last = spending(&engel_q(&C, &E, y * sub, &unit), &unit);
        while y < 40.0 {
            let m = marginal(y);
            let w = engel::weight(y, &E);
            for s in 0..C.S {
                // the share of a further shilling is the weighted mean of the two sets...
                let want = C.shares[s] + w * (C.shares_rich[s] - C.shares[s]);
                assert!((m[s] - want).abs() < 2e-4, "strength {k}, income {y}, need {s}: {} against {want}", m[s]);
            }
            // ...so it adds up to a shilling, and more income never buys less of anything
            assert!((m.iter().sum::<f64>() - 1.0).abs() < 1e-6);
            y *= 1.01;
            let now = spending(&engel_q(&C, &E, y * sub, &unit), &unit);
            assert!((0..C.S).all(|s| now[s] >= last[s]), "strength {k}, income {y}: spending on a need fell");
            last = now;
        }
        // at the reference income the split is the content's; far above, the richer set
        let (lo, hi) = (marginal(1.99), marginal(4000.0));
        for s in 0..C.S {
            assert!((lo[s] - C.shares[s]).abs() < 1e-6);
            if k >= 1.0 {
                assert!((hi[s] - C.shares_rich[s]).abs() < 1e-3, "strength {k}, need {s}: {} far above", hi[s]);
            }
        }
    }
}

#[test]
fn the_curve_has_no_step() {
    let C = default_content();
    let (_, unit) = service_mix(&prices(&C, |_| 1.0), &C.props, 0.25);
    let sub: f64 = (0..C.S).map(|s| C.subsistence[s] * unit[(0, s)]).sum();
    for k in STRENGTHS {
        let E = curve(k);
        let mut y = 1.9;
        let mut last = engel_q(&C, &E, y * sub, &unit);
        while y < 12.0 {
            y += 1e-4;
            let q = engel_q(&C, &E, y * sub, &unit);
            for s in 0..C.S {
                // a ten-thousandth more income moves no quantity by more than its own share of it
                let step = (q[(0, s)] - last[(0, s)]).abs() * unit[(0, s)];
                assert!(step <= 1.01e-4 * sub, "strength {k}, income {y}, need {s}: a step of {step}");
            }
            last = q;
        }
    }
}

#[test]
fn demand_falls_as_its_own_price_rises() {
    let C = default_content();
    let (_, unit0) = service_mix(&prices(&C, |_| 1.0), &C.props, 0.25);
    let sub0: f64 = (0..C.S).map(|s| C.subsistence[s] * unit0[(0, s)]).sum();
    for k in STRENGTHS {
        let E = curve(k);
        for budget in [0.6, 1.0, 1.7, 2.0, 2.2, 3.0, 3.7, 5.5, 9.0, 30.0].map(|y| y * sub0) {
            for s in 0..C.S {
                // the price of one need from a fifth to five times its level, the money held
                let mut last = f64::INFINITY;
                let mut mult = 0.2;
                while mult < 5.0 {
                    let mut unit = unit0.clone();
                    unit[(0, s)] *= mult;
                    let q = engel_q(&C, &E, budget, &unit)[(0, s)];
                    assert!(q <= last, "strength {k}, budget {budget}, need {s}, price x{mult}: demand rose from {last} to {q}");
                    assert!(q < last || q == 0.0, "strength {k}, budget {budget}, need {s}, price x{mult}: demand did not fall");
                    last = q;
                    mult *= 1.02;
                }
            }
        }
    }
}

#[test]
fn content_can_say_the_richer_set() {
    // the port's content has none, so it gets the mechanism's own, by name
    let C = default_content();
    assert_eq!(C.service_name, RICH_SHARES.iter().map(|(n, _)| n.to_string()).collect::<Vec<_>>());
    assert_eq!(C.shares_rich, RICH_SHARES.iter().map(|&(_, v)| v).collect::<Vec<_>>());
    assert!((C.shares_rich.iter().sum::<f64>() - C.shares.iter().sum::<f64>()).abs() < 1e-12);
    // against the content's: less food and warmth, more comfort, a little more clothing and shelter
    let need = |name: &str| C.service_name.iter().position(|n| n == name).unwrap();
    for name in ["food", "warmth"] {
        assert!(C.shares_rich[need(name)] < C.shares[need(name)]);
    }
    for name in ["shelter", "clothing", "comfort"] {
        assert!(C.shares_rich[need(name)] > C.shares[need(name)]);
    }

    let with = |line: &str| {
        let at = DEFAULT_CONTENT_TOML.find("lord_shares").expect("the services table has lord_shares");
        content_from_str(&format!("{}{line}\n{}", &DEFAULT_CONTENT_TOML[..at], &DEFAULT_CONTENT_TOML[at..]))
    };
    let own = with("shares_rich = [0.1, 0.1, 0.2, 0.2, 0.1, 0.3]").expect("a richer set is read");
    assert_eq!(own.shares_rich, vec![0.1, 0.1, 0.2, 0.2, 0.1, 0.3]);
    assert_eq!(own.shares, C.shares);
    let short = with("shares_rich = [0.5, 0.5]").unwrap_err();
    assert!(short.contains("shares_rich must be 6 shares"), "{short}");
    let light = with("shares_rich = [0.1, 0.1, 0.2, 0.2, 0.1, 0.2]").unwrap_err();
    assert!(light.contains("adds up to"), "{light}");
    assert!(with("shares_rich = [0.1, 0.1, 0.2, 0.2, -0.1, 0.5]").is_err());
}

fn run_to(P: &Params, end_year: i64) -> RunResult {
    let mut P = P.clone();
    P.end_year = Some(end_year);
    // run_with checks every month that money only changed by what was issued
    sim::run_with(&P, default_content(), false, true).expect("money is conserved every month")
}

fn engel_params(seed: u64, strength: f64) -> Params {
    let mut P = default_params();
    P.seed = seed;
    P.engel = curve(strength);
    P
}

#[test]
fn only_the_games_rules_have_the_curve() {
    assert!(!default_params().engel.enabled);
    assert!(game_params().engel.enabled);
    let port = serde_json::to_value(default_params()).unwrap();
    assert!(port.get("engel").is_none(), "a port run's params.json says nothing of the curve");
    assert_eq!(serde_json::to_value(game_params()).unwrap()["engel"]["enabled"], serde_json::Value::Bool(true));
    // switching it on by name, as the sweep does, gives the game's curve
    let set = default_params().with_json_overrides(r#"{"engel": {"enabled": true}}"#).unwrap();
    assert_eq!(set.engel, EngelParams::game());
}

#[test]
fn the_sweep_switches_it_on_by_name_and_gives_its_facts() {
    let dir = std::env::temp_dir().join(format!("isostate-engel-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let args = ["sweep", "--seeds", "2", "--end-year", "1756", "--set", "engel.enabled=true", "--out", dir.to_str().unwrap(), "--quiet"];
    isostate_core::cli::main_with_args(&args.map(String::from)).expect("the sweep runs");
    let doc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("facts.json")).unwrap()).unwrap();
    let run = &doc["runs"][0];
    assert_eq!(run["ok"], serde_json::Value::Bool(true), "run failed: {}", run["error"]);
    let fact = |name: &str| run["facts"][name].as_f64().unwrap_or_else(|| panic!("{name} is missing or has no value"));
    // the chained real wage starts where the fixed basket's does, and the shares are one budget
    assert!((fact("real_wage_chained_1756_vs_1750") / fact("real_wage_1756_vs_1750") - 1.0).abs() < 0.1);
    assert!(fact("real_income_1750") > 1.0 && fact("real_income_1750") < 3.0);
    let needs = ["food", "warmth", "shelter", "clothing", "implements", "comfort"];
    assert!((needs.iter().map(|n| fact(&format!("budget_{n}_1756"))).sum::<f64>() - 1.0).abs() < 1e-9);
    // land, mines and works are all of the work there is
    assert!((fact("farm_share_1756") + fact("mine_share_1756") + fact("industry_share_1756") - 1.0).abs() < 1e-9);
    assert!(fact("grain_output_1756_vs_1750") > 0.5);
    let annual = std::fs::read_to_string(dir.join("annual.csv")).unwrap();
    assert!(annual.lines().next().unwrap().ends_with("cpi_chained,real_wage_chained,real_income,budget_food,budget_warmth,budget_shelter,budget_clothing,budget_implements,budget_comfort"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_curve_conserves_money_and_runs_are_deterministic() {
    let P = engel_params(3, EngelParams::game().strength);
    let a = run_to(&P, 1794);
    let b = run_to(&P, 1794);
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    assert_eq!(a.state.series.names, b.state.series.names);
    for (k, name) in a.state.series.names.iter().enumerate() {
        assert_eq!(bits(&a.state.series.data[k]), bits(&b.state.series.data[k]), "series {name} differs between identical runs");
    }
    assert_eq!(a.events, b.events);
    // the new series are there, after the port's, and every month has a number
    let port = sim::start(&{ let mut P = default_params(); P.end_year = Some(1750); P }, default_content());
    let n = port.st.series.names.len();
    assert_eq!(a.state.series.names[..n], port.st.series.names[..]);
    let new = &a.state.series.names[n..];
    assert_eq!(new[..3], ["cpi_chained", "real_wage_chained", "real_income"]);
    assert_eq!(new[3..], ["budget_food", "budget_warmth", "budget_shelter", "budget_clothing", "budget_implements", "budget_comfort"]);
    for name in new {
        assert!(a.state.series.get(name).iter().all(|v| v.is_finite() && *v >= 0.0), "{name} has a month without a number");
    }
    // the budget shares are shares of one budget
    let months = a.state.series.get("budget_food").len();
    for t in (0..months).step_by(7) {
        let sum: f64 = new[3..].iter().map(|name| a.state.series.get(name)[t]).sum();
        assert!((sum - 1.0).abs() < 1e-9, "month {t}: the budget shares add up to {sum}");
    }
}

#[test]
fn six_families_and_the_curve_conserve_money() {
    let mut P = engel_params(7, EngelParams::game().strength);
    P.n_families = 6;
    let r = run_to(&P, 1774);
    let st = &r.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
}

#[test]
fn a_curve_of_no_strength_is_the_port_with_more_series() {
    let flat = run_to(&engel_params(5, 0.0), 1757);
    let port = run_to(&{ let mut P = default_params(); P.seed = 5; P }, 1757);
    for (k, name) in port.state.series.names.iter().enumerate() {
        assert_eq!(&flat.state.series.names[k], name);
        assert_eq!(bits(&flat.state.series.data[k]), bits(&port.state.series.data[k]), "series {name} differs from the port's");
    }
    assert_eq!(flat.events, port.events);
    assert!(flat.state.series.names.len() > port.state.series.names.len());
    // in the first month the chained index and the fixed basket's start from the same base
    let (chained, fixed) = (flat.state.series.get("cpi_chained"), flat.state.series.get("cpi"));
    assert!((chained[0] / fixed[0] - 1.0).abs() < 0.01, "{} against {}", chained[0], fixed[0]);
}

#[test]
fn richer_households_buy_less_bread_and_more_comfort() {
    // sixty years on one island, with the curve and with a flat one
    let on = run_to(&engel_params(3, EngelParams::game().strength), 1809);
    let flat = run_to(&engel_params(3, 0.0), 1809);
    let y0 = on.content.start_year;
    let mean = |r: &RunResult, name: &str, from: i64, to: i64| -> f64 {
        (from..=to).map(|y| annual(r.state.series.get(name), y0, y)).sum::<f64>() / (to - from + 1) as f64
    };
    // incomes have risen well past the reference by the 1800s
    assert!(mean(&flat, "real_income", 1800, 1809) > 2.4, "real income {}", mean(&flat, "real_income", 1800, 1809));
    let (food, food_flat) = (mean(&on, "budget_food", 1800, 1809), mean(&flat, "budget_food", 1800, 1809));
    let (comfort, comfort_flat) = (mean(&on, "budget_comfort", 1800, 1809), mean(&flat, "budget_comfort", 1800, 1809));
    assert!(food < food_flat - 0.01, "food takes {food} of the budget against {food_flat}");
    assert!(comfort > comfort_flat + 0.01, "comfort takes {comfort} of the budget against {comfort_flat}");
    // so more of the hands are in the works and more of the people in towns
    let (works, works_flat) = (mean(&on, "ind_share", 1800, 1809), mean(&flat, "ind_share", 1800, 1809));
    let (towns, towns_flat) = (mean(&on, "urban_share", 1800, 1809), mean(&flat, "urban_share", 1800, 1809));
    assert!(works > works_flat + 0.01, "works employ {works} against {works_flat}");
    assert!(towns > towns_flat + 0.01, "towns hold {towns} against {towns_flat}");
    // and 1750 is hardly moved
    for name in ["urban_share", "ind_share", "ag_share", "budget_food"] {
        let (a, b) = (annual(on.state.series.get(name), y0, y0), annual(flat.state.series.get(name), y0, y0));
        assert!((a - b).abs() < 0.015, "{name} in {y0}: {a} against {b}");
    }
    // the chained index says the cost of living rose less than the 1750 basket says
    let (chained, fixed) = (mean(&on, "cpi_chained", 1800, 1809), mean(&on, "cpi", 1800, 1809));
    assert!(chained.is_finite() && chained > 0.0 && chained < fixed, "chained {chained}, fixed basket {fixed}");
}
