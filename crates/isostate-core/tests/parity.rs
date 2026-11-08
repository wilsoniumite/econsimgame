//! Unit parity with the Python oracle: load the state Python had just before one
//! deterministic step, run the Rust step, and compare with the state Python had
//! just after it.
//!
//! Fixtures (about 150 MB each, not committed), from the repository root:
//!
//!     python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1752 --month 3 \
//!         --out crates/isostate-core/parity/fixtures/s7_1752
//!     python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1836 --month 5 \
//!         --out crates/isostate-core/parity/fixtures/s7_1836
//!
//! Every fixture directory named `s*` under `parity/fixtures` (or `$ISOSTATE_PARITY_DIR`)
//! is checked. Tolerance: relative 1e-9, plus an absolute floor of 1e-12 times the
//! largest magnitude in the array, for entries that are differences of much larger
//! numbers (an inventory emptied to nearly zero, a firm's cash after paying out).

mod common;

use std::path::PathBuf;

use common::dump::{Dump, load_state};
use common::{atol_for, compare, fixtures_root};
use isostate_core::content::{Content, default_content};
use isostate_core::econ;
use isostate_core::land;
use isostate_core::mat::Mat;
use isostate_core::params::Params;

const RTOL: f64 = 1e-9;
const ATOL_SCALE: f64 = 1e-12;

fn fixture_sets() -> Vec<PathBuf> {
    let root = fixtures_root();
    let mut out: Vec<PathBuf> = std::fs::read_dir(&root)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir() && p.file_name().unwrap().to_string_lossy().starts_with('s'))
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    if out.is_empty() {
        let msg = format!(
            "no parity fixtures under {}; generate them with crates/isostate-core/parity/dump_state.py (see tests/parity.rs)",
            root.display()
        );
        if std::env::var("ISOSTATE_REQUIRE_PARITY").is_ok() {
            panic!("{msg}");
        }
        eprintln!("SKIPPED: {msg}");
    }
    out
}

#[allow(non_snake_case)]
fn setup(set: &PathBuf) -> (Content, Params) {
    let text = std::fs::read_to_string(set.join("params.json")).unwrap();
    (default_content(), Params::from_json_str(&text).unwrap())
}

struct Checker {
    what: String,
    failures: Vec<String>,
    checked: usize,
    worst: (f64, String),
}

impl Checker {
    fn new(what: String) -> Checker {
        Checker { what, failures: Vec::new(), checked: 0, worst: (0.0, String::new()) }
    }

    fn vec(&mut self, name: &str, got: &[f64], want: &[f64]) {
        self.checked += 1;
        let atol = atol_for(want, ATOL_SCALE);
        for (&g, &w) in got.iter().zip(want) {
            if g.is_finite() && w.is_finite() && g != w {
                let rel = (g - w).abs() / (g.abs().max(w.abs()) + atol);
                if rel > self.worst.0 {
                    self.worst = (rel, name.to_string());
                }
            }
        }
        if let Some(e) = compare(name, got, want, RTOL, atol_for(want, ATOL_SCALE)) {
            self.failures.push(e);
        }
    }

    fn mat(&mut self, name: &str, got: &Mat, want: &Mat) {
        if (got.rows, got.cols) != (want.rows, want.cols) {
            self.failures.push(format!("{name}: shape {}x{} vs {}x{}", got.rows, got.cols, want.rows, want.cols));
            return;
        }
        self.vec(name, &got.data, &want.data);
    }

    fn scalar(&mut self, name: &str, got: f64, want: f64) {
        self.vec(name, &[got], &[want]);
    }

    fn exact<T: PartialEq + std::fmt::Debug>(&mut self, name: &str, got: &[T], want: &[T]) {
        self.checked += 1;
        if got != want {
            let bad = got.iter().zip(want).filter(|(a, b)| a != b).count();
            let first = got.iter().zip(want).position(|(a, b)| a != b);
            self.failures.push(format!(
                "{name}: {bad} of {} differ (lengths {} vs {}), first at {first:?}",
                want.len(),
                got.len(),
                want.len()
            ));
        }
    }

    fn finish(self) {
        if !self.failures.is_empty() {
            panic!("{}: {} of {} fields differ:\n  {}", self.what, self.failures.len(), self.checked, self.failures.join("\n  "));
        }
        eprintln!("{}: {} fields match (largest relative difference {:.2e} in {})", self.what, self.checked, self.worst.0, self.worst.1);
    }
}

fn f(v: &[bool]) -> Vec<f64> {
    v.iter().map(|&b| b as u8 as f64).collect()
}

#[test]
#[allow(non_snake_case)]
fn econ_step_month_matches_python() {
    for set in fixture_sets() {
        let (C, P) = setup(&set);
        let mut st = load_state(&set.join("econ_before"));
        let want = load_state(&set.join("econ_after"));
        let money_before = econ::money_total(&st);
        let issued_before = st.issued;
        econ::step_month(&mut st, &C, &P);

        let mut ck = Checker::new(format!("{} econ.step_month", set.display()));
        // money is conserved by the Rust step itself
        let drift = econ::money_total(&st) - (money_before + (st.issued - issued_before));
        ck.scalar("money drift / money", drift / econ::money_total(&st), 0.0);

        for (name, got, w) in [
            ("hh_cash", &st.hh_cash, &want.hh_cash),
            ("lord_cash", &st.lord_cash, &want.lord_cash),
            ("merch_cash", &st.merch_cash, &want.merch_cash),
            ("hh_income", &st.hh_income, &want.hh_income),
            ("lord_income", &st.lord_income, &want.lord_income),
            ("hh_income_prev", &st.hh_income_prev, &want.hh_income_prev),
            ("lord_income_prev", &st.lord_income_prev, &want.lord_income_prev),
            ("wage", &st.wage, &want.wage),
            ("tools_mult", &st.tools_mult, &want.tools_mult),
            ("imp_pc", &st.imp_pc, &want.imp_pc),
            ("subcost", &st.subcost, &want.subcost),
            ("food_ratio", &st.food_ratio, &want.food_ratio),
            ("food_min_acc", &st.food_min_acc, &want.food_min_acc),
            ("y_acc", &st.y_acc, &want.y_acc),
            ("wage_acc", &st.wage_acc, &want.wage_acc),
            ("cumprod", &st.cumprod, &want.cumprod),
            ("method_Q", &st.method_Q, &want.method_Q),
            ("ex_labor", &st.ex_labor, &want.ex_labor),
            ("ex_out", &st.ex_out, &want.ex_out),
            ("ex_Lprev", &st.ex_Lprev, &want.ex_Lprev),
            ("f_labor", &st.f_labor, &want.f_labor),
            ("f_fill", &st.f_fill, &want.f_fill),
            ("f_u", &st.f_u, &want.f_u),
            ("f_cash", &st.f_cash, &want.f_cash),
            ("f_profit_acc", &st.f_profit_acc, &want.f_profit_acc),
            ("f_out_acc", &st.f_out_acc, &want.f_out_acc),
            ("f_cost_m", &st.f_cost_m, &want.f_cost_m),
            ("f_K", &st.f_K, &want.f_K),
            ("f_order_K", &st.f_order_K, &want.f_order_K),
            ("cover", &st.cover, &want.cover),
            ("cell_flow", &st.cell_flow, &want.cell_flow),
        ] {
            ck.vec(name, got, w);
        }
        for (name, got, w) in [
            ("price", &st.price, &want.price),
            ("pexp", &st.pexp, &want.pexp),
            ("inv", &st.inv, &want.inv),
            ("outflow", &st.outflow, &want.outflow),
            ("fill", &st.fill, &want.fill),
            ("price_acc", &st.price_acc, &want.price_acc),
            ("town_prod", &st.town_prod, &want.town_prod),
            ("trade_F", &st.trade_F, &want.trade_F),
            ("f_order", &st.f_order, &want.f_order),
        ] {
            ck.mat(name, got, w);
        }
        ck.scalar("fund_cash", st.fund_cash, want.fund_cash);
        ck.scalar("issued", st.issued, want.issued);
        ck.scalar("m_rent", st.m_rent, want.m_rent);
        ck.scalar("m_invest", st.m_invest, want.m_invest);
        ck.scalar("m_merch_margin", st.m_merch_margin, want.m_merch_margin);
        ck.scalar("m_trade_value", st.m_trade_value, want.m_trade_value);
        assert_eq!(st.projects.len(), want.projects.len());
        for (k, (a, b)) in st.projects.iter().zip(&want.projects).enumerate() {
            ck.scalar(&format!("project{k}.cash"), a.cash, b.cash);
            ck.vec(&format!("project{k}.order"), &a.order, &b.order);
        }
        let (ms, mw) = (&st.month_stats, &want.month_stats);
        ck.vec("ms.prod", &ms.prod, &mw.prod);
        ck.vec("ms.method_out", &ms.method_out, &mw.method_out);
        ck.mat("ms.hh_got", &ms.hh_got, &mw.hh_got);
        ck.mat("ms.lord_got", &ms.lord_got, &mw.lord_got);
        ck.scalar("ms.hh_spend", ms.hh_spend, mw.hh_spend);
        ck.scalar("ms.lord_spend", ms.lord_spend, mw.lord_spend);
        ck.scalar("ms.urban_rent", ms.urban_rent, mw.urban_rent);
        ck.vec("ms.employment", &ms.employment, &mw.employment);
        ck.vec("ms.ex_labor_by_sector", &ms.ex_labor_by_sector, &mw.ex_labor_by_sector);
        ck.vec("ms.firm_labor_by_method", &ms.firm_labor_by_method, &mw.firm_labor_by_method);
        ck.vec("ms.heat_use", &ms.heat_use, &mw.heat_use);
        ck.vec("ms.supply", &ms.supply, &mw.supply);
        ck.exact("f_alive", &f(&st.f_alive), &f(&want.f_alive));
        ck.finish();
    }
}

fn u(v: &[u8]) -> Vec<f64> {
    v.iter().map(|&x| x as f64).collect()
}

fn iv(v: &[i64]) -> Vec<f64> {
    v.iter().map(|&x| x as f64).collect()
}

fn uv(v: &[usize]) -> Vec<f64> {
    v.iter().map(|&x| x as f64).collect()
}

#[test]
#[allow(non_snake_case)]
fn land_update_land_use_matches_python() {
    for set in fixture_sets() {
        let (C, P) = setup(&set);
        let dir = set.join("land_before");
        let d = Dump::open(&dir);
        let mut st = load_state(&dir);
        let want = load_state(&set.join("land_after"));
        let year = d.arg_f64("year") as i64;
        let initial = d.args["initial"].as_bool().unwrap_or(false);
        land::update_land_use(&mut st, &C, &P, year, &d.arg_mat("pbar"), &d.arg_vec("wbar"), initial);

        let mut ck = Checker::new(format!("{} land.update_land_use({year})", set.display()));
        ck.exact("urban", &f(&st.urban), &f(&want.urban));
        ck.exact("use", &u(&st.land_use), &u(&want.land_use));
        ck.exact("derelict_timer", &iv(&st.derelict_timer), &iv(&want.derelict_timer));
        ck.exact("ex_cell", &uv(&st.ex_cell), &uv(&want.ex_cell));
        ck.exact("ex_sector", &uv(&st.ex_sector), &uv(&want.ex_sector));
        ck.exact("ex_town", &uv(&st.ex_town), &uv(&want.ex_town));
        for (name, got, w) in [
            ("urban_rent", &st.urban_rent, &want.urban_rent),
            ("cell_urban_rent", &st.cell_urban_rent, &want.cell_urban_rent),
            ("urban_res", &st.urban_res, &want.urban_res),
            ("urban_pop", &st.urban_pop, &want.urban_pop),
            ("cover", &st.cover, &want.cover),
            ("ex_lam", &st.ex_lam, &want.ex_lam),
            ("ex_field", &st.ex_field, &want.ex_field),
            ("ex_tau", &st.ex_tau, &want.ex_tau),
            ("ex_Lprev", &st.ex_Lprev, &want.ex_Lprev),
            ("land_rent_only", &st.land_rent_only, &want.land_rent_only),
            ("cell_rent", &st.cell_rent, &want.cell_rent),
        ] {
            ck.vec(name, got, w);
        }
        ck.mat("lam", &st.lam, &want.lam);
        ck.mat("inv", &st.inv, &want.inv);
        ck.mat("bids", &st.bids, &want.bids);
        ck.scalar("urban_density", st.urban_density, want.urban_density);
        // land shares of every cell with any sum to its room: one minus the industrial
        // footprint, or one on cells that are taken (urban, mostly industrial,
        // derelict, or outside every catchment; those are all wild)
        for c in 0..st.N {
            let tot: f64 = (0..=C.NS).map(|r| st.lam[(r, c)]).sum();
            if tot > 0.0 {
                let ks = st.cell_K_share[c];
                let taken = st.urban[c] || ks >= 0.6 || st.derelict_timer[c] > 0 || !st.world.land[c] || st.catch[c] < 0;
                let room = if taken { 1.0 } else { 1.0 - ks };
                assert!((tot - room).abs() < 1e-9, "cell {c}: land shares sum to {tot}, expected {room}");
            }
        }
        ck.finish();
    }
}

#[test]
#[allow(non_snake_case)]
fn land_update_transport_and_delta_fields_match_python() {
    for set in fixture_sets() {
        let (C, P) = setup(&set);
        let mut st = load_state(&set.join("transport_before"));
        let want = load_state(&set.join("transport_after"));
        land::update_transport(&mut st, &C, &P);
        // Python's update_transport ends with delta_fields (see land.rs)
        land::delta_fields(&mut st, &C, &P);

        let mut ck = Checker::new(format!("{} land.update_transport + delta_fields", set.display()));
        ck.vec("graph.w", &st.graph.w, &want.graph.w);
        ck.exact("graph.railed", &f(&st.graph.railed), &f(&want.graph.railed));
        ck.mat("dist", &st.dist, &want.dist);
        ck.exact("catch", &iv(&st.catch), &iv(&want.catch));
        ck.vec("dcatch", &st.dcatch, &want.dcatch);
        ck.mat("town_cost", &st.town_cost, &want.town_cost);
        ck.mat("delta", &st.delta, &want.delta);
        // Predecessor trees can differ only where two paths cost exactly the same
        // (common at sea, where every sea edge costs the same per step). Count the
        // land cells of active towns whose predecessor differs.
        let mut differ = 0usize;
        let mut total = 0usize;
        for i in 0..st.T_max {
            if !st.town_active[i] {
                continue;
            }
            for c in 0..st.N {
                if st.world.land[c] {
                    total += 1;
                    let (pr, pp) = (st.pred[i][c], want.pred[i][c]);
                    if pr != pp {
                        differ += 1;
                        // both predecessors must reach c at exactly its distance
                        let via = |p: i64| -> f64 {
                            let k = (0..8).find(|&k| isostate_core::transport::neighbour(st.world.H, st.world.W, p as usize, k) == Some(c)).unwrap();
                            st.dist[(i, p as usize)] + st.graph.w[p as usize * 8 + k]
                        };
                        assert!(pr >= 0 && pp >= 0, "town {i} cell {c}: predecessor {pr} vs {pp}");
                        assert_eq!(via(pr), st.dist[(i, c)], "town {i} cell {c}: Rust predecessor is not on a shortest path");
                        assert_eq!(via(pp), st.dist[(i, c)], "town {i} cell {c}: Python predecessor is not on a shortest path");
                    }
                }
            }
        }
        eprintln!("pred: {differ} of {total} land-cell predecessors differ (exact ties)");
        assert!((differ as f64) < 0.01 * total as f64, "too many predecessor differences: {differ} of {total}");
        ck.finish();
    }
}
