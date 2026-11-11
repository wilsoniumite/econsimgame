//! Checks that should hold however the model is tuned (ports of tests/test_sim.py),
//! plus invariants checked every year of a run.

use isostate_core::econ::{les, money_total, service_mix};
use isostate_core::mat::Mat;
use isostate_core::record::{self, DISP};
use isostate_core::sim::{self, RunResult};
use isostate_core::{Params, default_content, default_params, init_state};

#[allow(non_snake_case)]
fn short_params(seed: u64, end_year: i64) -> Params {
    let mut P = default_params();
    P.seed = seed;
    P.end_year = Some(end_year);
    P
}

fn short_run(seed: u64, end_year: i64) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(&short_params(seed, end_year), default_content(), false, true).expect("money conserved")
}

#[test]
fn money_is_conserved_and_runs_are_deterministic() {
    let a = short_run(3, 1756);
    let b = short_run(3, 1756);
    assert_eq!(a.state.series.names, b.state.series.names);
    for (k, name) in a.state.series.names.iter().enumerate() {
        let (x, y) = (&a.state.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs between identical runs");
    }
    let titles = |r: &RunResult| r.events.iter().map(|e| e.title.clone()).collect::<Vec<_>>();
    assert_eq!(titles(&a), titles(&b));
    let blob_a = record::pack(&a.state, &a.content, &a.params, &a.events, &a.rec, "a", false);
    let blob_b = record::pack(&b.state, &b.content, &b.params, &b.events, &b.rec, "a", false);
    assert!(blob_a == blob_b, "identical runs pack to different bytes");
}

#[test]
#[allow(non_snake_case)]
fn invariants_hold_every_year() {
    let P = short_params(5, 1765);
    let mut s = sim::start(&P, default_content());
    let C = s.content.clone();
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money conserved");
        let st = &s.st;
        let y = s.year - 1;
        // money: the stock is what was there at the start plus what was issued
        let drift = money_total(st) - (st.money_start + st.issued);
        assert!(drift.abs() < 1e-6 * money_total(st), "{y}: money drift {drift}");
        // no NaN, no negative stocks
        for t in 0..st.T_max {
            assert!(st.pop[t] >= 0.0 && st.pop[t].is_finite(), "{y}: pop[{t}] = {}", st.pop[t]);
            for g in 0..C.G {
                let (iv, p) = (st.inv[(t, g)], st.price[(t, g)]);
                assert!(iv >= 0.0 && iv.is_finite(), "{y}: inv[{t},{g}] = {iv}");
                assert!(p.is_finite() && p > 0.0, "{y}: price[{t},{g}] = {p}");
                if st.town_active[t] {
                    assert!(p >= P.price_floor * C.base_price[g] * (1.0 - 1e-12) && p <= P.price_ceiling * C.base_price[g] * (1.0 + 1e-12));
                }
            }
            for v in [st.hh_cash[t], st.lord_cash[t], st.merch_cash[t], st.wage[t]] {
                assert!(v.is_finite(), "{y}: non-finite town account");
            }
        }
        for f in 0..st.F_count {
            assert!(st.f_K[f] >= 0.0 && st.f_order_K[f] >= 0.0 && st.f_cash[f].is_finite(), "{y}: firm {f}");
            assert!(st.f_order.row(f).iter().all(|&v| v >= 0.0), "{y}: firm {f} has a negative order");
        }
        for pr in &st.projects {
            assert!(pr.order.iter().all(|&v| v >= 0.0) && pr.cash.is_finite());
        }
        // Land shares sum to one minus the industrial footprint when update_land_use
        // sets them (checked exactly in tests/parity.rs); the same year's closures and
        // new firms then move the footprint, so at year end they are only bounded.
        for c in 0..st.N {
            let tot: f64 = (0..=C.NS).map(|r| st.lam[(r, c)]).sum();
            assert!((0..=C.NS).all(|r| st.lam[(r, c)] >= 0.0), "{y}: negative land share on cell {c}");
            assert!(tot <= 1.0 + 1e-9, "{y}: cell {c} shares sum to {tot}");
            assert!(st.cover[c] >= 0.0 && st.cover[c] <= st.world.forest_pot[c] + 1e-12, "{y}: cover[{c}]");
        }
        // the recorded series are finite except shares of an empty total (NaN in Python too)
        for (k, name) in st.series.names.iter().enumerate() {
            let v = st.series.data[k][s.t - 1];
            assert!(v.is_finite() || (name.starts_with("share_") && v.is_nan()), "{y}: series {name} = {v}");
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn rent_falls_with_distance_to_market() {
    let (st, _) = init_state(&default_params_seed(5), &default_content());
    let mut pts: Vec<(f64, f64)> = Vec::new();
    for c in 0..st.N {
        if st.world.land[c] && st.catch[c] >= 0 && !st.urban[c] && st.dcatch[c].is_finite() {
            pts.push((st.land_rent_only[c], st.dcatch[c]));
        }
    }
    // Spearman rank correlation
    let rank = |v: Vec<f64>| -> Vec<f64> {
        let order = isostate_core::util::argsort(&v);
        let mut r = vec![0.0; v.len()];
        for (k, &i) in order.iter().enumerate() {
            r[i] = k as f64;
        }
        r
    };
    let r1 = rank(pts.iter().map(|p| p.0).collect());
    let r2 = rank(pts.iter().map(|p| p.1).collect());
    let n = r1.len() as f64;
    let (m1, m2) = (r1.iter().sum::<f64>() / n, r2.iter().sum::<f64>() / n);
    let cov: f64 = r1.iter().zip(&r2).map(|(a, b)| (a - m1) * (b - m2)).sum();
    let v1: f64 = r1.iter().map(|a| (a - m1) * (a - m1)).sum();
    let v2: f64 = r2.iter().map(|b| (b - m2) * (b - m2)).sum();
    let rho = cov / (v1 * v2).sqrt();
    assert!(rho < -0.3, "von Thunen gradient missing: rank correlation {rho:.2}");
}

fn default_params_seed(seed: u64) -> Params {
    let mut p = default_params();
    p.seed = seed;
    p
}

#[test]
fn no_hard_walls_in_household_demand() {
    // one town, three goods that can all serve one need at different efficiencies
    let props = Mat::from_vec(3, 1, vec![1.0, 0.35, 3.0]);
    let price = Mat::from_vec(1, 3, vec![1.0, 1.0, 40.0]);
    let (share, _) = service_mix(&price, &props, 0.25);
    let s: Vec<f64> = (0..3).map(|g| share[(0, g)]).collect();
    assert!(s.iter().all(|&x| x > 0.0), "every good with the property should get some share");
    assert!((s.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    // the expensive but efficient good still loses to the cheap one per unit of service
    assert!(s[0] > s[2]);
}

#[test]
fn linear_expenditure_spends_the_budget() {
    let unit = Mat::from_vec(1, 3, vec![1.0, 2.0, 0.5]);
    let gamma = [0.5, 0.1, 0.2];
    let beta = [0.5, 0.3, 0.2];
    let spend = |q: &Mat| (0..3).map(|s| q[(0, s)] * unit[(0, s)]).sum::<f64>();
    let (q, _) = les(&[10.0], &unit, &gamma, &beta);
    assert!((spend(&q) - 10.0).abs() < 1e-9);
    let (poor, _) = les(&[0.3], &unit, &gamma, &beta);
    assert!((spend(&poor) - 0.3).abs() < 1e-9);
}

#[test]
fn pack_round_trip_and_display_mix() {
    let res = short_run(3, 1753);
    let blob = record::pack(&res.state, &res.content, &res.params, &res.events, &res.rec, "test", false);
    let hlen = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    assert_eq!((4 + hlen) % 8, 0);
    let header: serde_json::Value = serde_json::from_slice(&blob[4..4 + hlen]).unwrap();
    let base = 4 + hlen;
    let arrays = header["arrays"].as_array().unwrap();
    let find = |name: &str| arrays.iter().find(|a| a["name"] == name).unwrap_or_else(|| panic!("no array {name}"));
    for name in ["use", "mix_a", "mix_s", "series", "firm_Kvals"] {
        assert_eq!(find(name)["offset"].as_u64().unwrap() % 8, 0, "{name} misaligned");
    }
    let n_years = header["meta"]["n_years"].as_u64().unwrap() as usize;
    assert_eq!(find("use")["shape"], serde_json::json!([n_years, res.state.N]));
    // the bytes of the use layer are the recorded layer
    let a = find("use");
    let (off, nbytes) = (a["offset"].as_u64().unwrap() as usize, a["nbytes"].as_u64().unwrap() as usize);
    assert_eq!(&blob[base + off..base + off + nbytes], res.rec.layer("use"));
    let lite = record::pack(&res.state, &res.content, &res.params, &res.events, &res.rec, "lite", true);
    let lh = u32::from_le_bytes(lite[..4].try_into().unwrap()) as usize;
    let lheader: serde_json::Value = serde_json::from_slice(&lite[4..4 + lh]).unwrap();
    assert!(lheader["meta"]["lite"].as_bool().unwrap() && lheader["firms"].as_array().unwrap().is_empty());
    assert!(lheader["arrays"].as_array().unwrap().iter().all(|a| a["name"] != "use"));
    // display mix: classes in range, the first class is the larger of the two
    let (ma, mb, ms) = (res.rec.layer("mix_a"), res.rec.layer("mix_b"), res.rec.layer("mix_s"));
    assert!(ma.iter().chain(mb).all(|&x| (x as usize) < DISP.len()));
    assert!(ms.iter().all(|&x| x >= 127));
}

#[test]
#[allow(non_snake_case)]
fn params_round_trip_and_overrides() {
    let P = default_params();
    let json = serde_json::to_string(&P).unwrap();
    assert_eq!(Params::from_json_str(&json).unwrap(), P);
    let q = Params::from_toml_str("seed = 3\nland_theta = 2.5\nrent_regime = \"commons\"\nend_year = 1800\n").unwrap();
    assert_eq!((q.seed, q.land_theta, q.end_year), (3, 2.5, Some(1800)));
    assert_eq!(q.rent_regime, isostate_core::RentRegime::Commons);
    assert_eq!(q.hurdle, P.hurdle, "fields left out keep their defaults");
    assert!(Params::from_toml_str("land_thetaa = 2.5").is_err(), "a misspelt field is an error");
    let r = P.with_json_overrides(r#"{"seed": 11, "decay": [0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0]}"#).unwrap();
    assert_eq!(r.seed, 11);
    assert_eq!(r.decay.len(), 16);
    // the same on top of the game's rules, from TOML
    let g = isostate_core::game_params();
    let t = g.with_toml_overrides("seed = 5\nhurdle = 0.2\n").unwrap();
    assert_eq!((t.seed, t.hurdle, t.land_theta), (5, 0.2, g.land_theta));
    assert!(g.with_toml_overrides("hurdel = 0.2").is_err());
    // the fields params.json leaves out while they are at their defaults come through an
    // override whether they are at their defaults or not
    let mut f = g.clone();
    (f.n_families, f.family_bonus, f.end_year) = (6, 0.5, Some(1800));
    let t = f.with_toml_overrides("hurdle = 0.2\n").unwrap();
    f.hurdle = 0.2;
    assert_eq!(t, f);
    assert_eq!(isostate_core::Rules::parse("port").map(|r| r.params()), Some(P));
    assert_eq!(isostate_core::Rules::parse("game").map(|r| r.params()), Some(g));
    assert_eq!(isostate_core::Rules::parse("python"), None);
}

#[test]
fn the_run_command_refuses_what_cannot_run() {
    let run = |args: &[&str]| isostate_core::cli::main_with_args(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    let out = std::env::temp_dir().join(format!("isostate-run-test-{}-refused", std::process::id()));
    let dir = out.to_str().unwrap();
    let e = run(&["run", "--out", dir, "--end-year", "1740"]).unwrap_err();
    assert!(e.contains("1740") && e.contains("before the start year, 1750"), "{e}");
    let e = run(&["run", "--out", dir, "--rules", "python"]).unwrap_err();
    assert!(e.contains("not port or game"), "{e}");
    // a params file with a misspelt field is named in the refusal
    let file = std::env::temp_dir().join(format!("isostate-run-test-{}-params.toml", std::process::id()));
    std::fs::write(&file, "hurdel = 0.2\n").unwrap();
    let e = run(&["run", "--out", dir, "--rules", "game", "--params", file.to_str().unwrap()]).unwrap_err();
    assert!(e.contains("params.toml") && e.contains("hurdel"), "{e}");
    let _ = std::fs::remove_file(&file);
    assert!(!out.exists(), "nothing is written when the command line is wrong");
}
