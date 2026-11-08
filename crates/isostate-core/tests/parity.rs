//! Unit parity with the Python oracle: load the state Python had just before one
//! deterministic step, run the Rust step, and compare with the state Python had
//! just after it.
//!
//! The steps that draw random numbers (firms.review, firms.entry, tech.discovery,
//! people.found_towns, and the whole yearly step, init_state and a whole run) are
//! checked the same way, with the numbers numpy drew replayed in order
//! (`Rng::replaying`), so they must match draw for draw.
//!
//! Fixtures (0.5 to 0.8 GB each, not committed), from the repository root:
//!
//!     python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1752 --month 3 \
//!         --found-towns --stochastic --init --out crates/isostate-core/parity/fixtures/s7_1752
//!     python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1786 --month 11 \
//!         --stochastic --whole-year --out crates/isostate-core/parity/fixtures/s7_1786
//!     python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1843 --month 11 \
//!         --stochastic --whole-year --out crates/isostate-core/parity/fixtures/s7_1843
//!     python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1847 --month 5 \
//!         --stochastic --whole-year --out crates/isostate-core/parity/fixtures/s7_1847
//!     python3 crates/isostate-core/parity/record_run.py --seed 7 --end-year 1785 \
//!         --out crates/isostate-core/parity/fixtures/run_s7_1785
//!
//! Every fixture directory named `s*` under `parity/fixtures` (or `$ISOSTATE_PARITY_DIR`)
//! is checked, and every `run_*` directory by the whole-run test. Tolerance: relative 1e-9, plus an absolute floor of 1e-12 times the
//! largest magnitude in the array, for entries that are differences of much larger
//! numbers (an inventory emptied to nearly zero, a firm's cash after paying out).

mod common;

use std::path::PathBuf;

use common::dump::{Dump, load_state, load_world};
use common::{atol_for, compare, fixtures_root};
use isostate_core::content::{Content, default_content};
use isostate_core::econ;
use isostate_core::firms;
use isostate_core::land;
use isostate_core::people;
use isostate_core::rail;
use isostate_core::rng::{Rng, Rngs, make_rngs};
use isostate_core::observer;
use isostate_core::sim;
use isostate_core::tech;
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
        // money is conserved by the Rust step itself, up to rounding in the sums
        // (a relative drift of a few 1e-16; the comparison is absolute because the
        // ideal value is zero)
        let drift = econ::money_total(&st) - (money_before + (st.issued - issued_before));
        let rel_drift = (drift / econ::money_total(&st)).abs();
        ck.exact("money conserved to 1e-12", &[rel_drift < 1e-12], &[true]);

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

/// The fixture sets that captured `tag`.
fn sets_with(tag: &str) -> Vec<PathBuf> {
    fixture_sets().into_iter().filter(|s| s.join(format!("{tag}_before")).is_dir()).collect()
}

/// The numbers numpy drew during the step, if the fixture recorded them, ready to
/// replay; otherwise a fresh stream (for steps that draw nothing).
fn recorded_draws(set: &PathBuf, tag: &str) -> Option<Rng> {
    let dir = set.join(format!("{tag}_after"));
    dir.join("draws.npy").exists().then(|| Rng::replaying(Dump::open(&dir).f64("draws")))
}

/// The events Python added during the step (`<tag>_after/events.json`).
fn recorded_events(set: &PathBuf, tag: &str) -> Option<Vec<serde_json::Value>> {
    let p = set.join(format!("{tag}_after")).join("events.json");
    p.exists().then(|| serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap())
}

/// Compare the events a Rust step added with the ones the Python step added.
fn check_events(ck: &mut Checker, got: &[isostate_core::Event], want: &[serde_json::Value]) {
    ck.exact("event count", &[got.len()], &[want.len()]);
    for (k, (g, w)) in got.iter().zip(want).enumerate() {
        let g = serde_json::to_value(g).unwrap();
        for key in ["year", "kind", "title", "text", "town", "cell", "tech"] {
            ck.exact(&format!("event{k} {key}"), &[g[key].to_string()], &[w[key].to_string()]);
        }
    }
}

/// The draws must all be used: one left over means the Rust step drew less.
fn check_draws_used(ck: &mut Checker, rng: &Rng) {
    if let Some(left) = rng.replay_left() {
        ck.exact("recorded draws left unused", &[left], &[0]);
    }
}

#[test]
#[allow(non_snake_case)]
fn people_growth_and_migration_match_python() {
    for set in sets_with("growth") {
        let (C, P) = setup(&set);
        let d = Dump::open(&set.join("growth_before"));
        let mut st = load_state(&set.join("growth_before"));
        let want = load_state(&set.join("growth_after"));
        let mut events = Vec::new();
        people::growth(&mut st, &C, &P, d.arg_f64("year") as i64, &mut events);
        let mut ck = Checker::new(format!("{} people.growth", set.display()));
        ck.vec("pop", &st.pop, &want.pop);
        ck.vec("y_ratio", &st.y_ratio, &want.y_ratio);
        ck.vec("y_acc", &st.y_acc, &want.y_acc);
        ck.vec("food_min_acc", &st.food_min_acc, &want.food_min_acc);
        ck.finish();
    }
    for set in sets_with("migration") {
        let (C, P) = setup(&set);
        let mut st = load_state(&set.join("migration_before"));
        let want = load_state(&set.join("migration_after"));
        people::migration(&mut st, &C, &P);
        let mut ck = Checker::new(format!("{} people.migration", set.display()));
        ck.vec("pop", &st.pop, &want.pop);
        ck.vec("hh_cash", &st.hh_cash, &want.hh_cash);
        ck.mat("migration_flow", &st.migration_flow, &want.migration_flow);
        ck.finish();
    }
}

#[test]
#[allow(non_snake_case)]
fn tech_diffusion_and_firm_productivity_match_python() {
    for set in sets_with("diffusion") {
        let (C, P) = setup(&set);
        let mut st = load_state(&set.join("diffusion_before"));
        let want = load_state(&set.join("diffusion_after"));
        tech::diffusion(&mut st, &C, &P);
        let mut ck = Checker::new(format!("{} tech.diffusion", set.display()));
        ck.mat("know", &st.know, &want.know);
        ck.finish();
    }
    for set in sets_with("productivity") {
        let (C, P) = setup(&set);
        let mut st = load_state(&set.join("productivity_before"));
        let want = load_state(&set.join("productivity_after"));
        firms::update_productivity(&mut st, &C, &P);
        let mut ck = Checker::new(format!("{} firms.update_productivity", set.display()));
        ck.vec("method_A", &st.method_A, &want.method_A);
        ck.mat("same_counts", &st.same_counts, &want.same_counts);
        ck.vec("f_prod", &st.f_prod, &want.f_prod);
        ck.finish();
    }
}

#[test]
#[allow(non_snake_case)]
fn firms_review_matches_python() {
    for set in sets_with("review") {
        let (C, P) = setup(&set);
        let d = Dump::open(&set.join("review_before"));
        let mut st = load_state(&set.join("review_before"));
        let want = load_state(&set.join("review_after"));
        // review draws no random numbers (sister firms are named after their parent),
        // which the recorded draws, when present, confirm
        let mut rng = recorded_draws(&set, "review").unwrap_or_else(|| make_rngs(0).firms);
        let closed = firms::review(&mut st, &C, &P, &mut rng, d.arg_f64("year") as i64, &d.arg_mat("pbar"), &d.arg_vec("wbar"));
        let mut ck = Checker::new(format!("{} firms.review", set.display()));
        check_draws_used(&mut ck, &rng);
        ck.exact("F_count", &[st.F_count], &[want.F_count]);
        ck.exact("closed", &closed, &want.closed_year);
        ck.exact("new_firms_year", &st.new_firms_year, &want.new_firms_year);
        ck.exact("f_name", &st.f_name, &want.f_name);
        ck.exact("f_alive", &f(&st.f_alive), &f(&want.f_alive));
        ck.exact("f_died", &iv(&st.f_died), &iv(&want.f_died));
        ck.exact("f_loss_years", &iv(&st.f_loss_years), &iv(&want.f_loss_years));
        ck.exact("f_parent", &iv(&st.f_parent), &iv(&want.f_parent));
        ck.exact("derelict_timer", &iv(&st.derelict_timer), &iv(&want.derelict_timer));
        for (name, got, w) in [
            ("f_K", &st.f_K, &want.f_K),
            ("f_order_K", &st.f_order_K, &want.f_order_K),
            ("f_cash", &st.f_cash, &want.f_cash),
            ("f_rent", &st.f_rent, &want.f_rent),
            ("f_profit_acc", &st.f_profit_acc, &want.f_profit_acc),
            ("hh_cash", &st.hh_cash, &want.hh_cash),
            ("lord_cash", &st.lord_cash, &want.lord_cash),
            ("merch_cash", &st.merch_cash, &want.merch_cash),
            ("cell_K_share", &st.cell_K_share, &want.cell_K_share),
        ] {
            ck.vec(name, got, w);
        }
        ck.mat("f_order", &st.f_order, &want.f_order);
        ck.finish();
    }
}

#[test]
#[allow(non_snake_case)]
fn rail_planning_matches_python() {
    for (tag, spurs) in [("rail_plan", false), ("rail_spurs", true)] {
        for set in sets_with(tag) {
            let (C, P) = setup(&set);
            let d = Dump::open(&set.join(format!("{tag}_before")));
            let mut st = load_state(&set.join(format!("{tag}_before")));
            let want = load_state(&set.join(format!("{tag}_after")));
            // the planners draw nothing, which recorded draws, when present, confirm
            let mut rng = recorded_draws(&set, tag).unwrap_or_else(|| make_rngs(0).rail);
            let mut events = Vec::new();
            let (year, pbar) = (d.arg_f64("year") as i64, d.arg_mat("pbar"));
            if spurs {
                rail::plan_spurs(&mut st, &C, &P, &mut rng, year, &pbar, &mut events);
            } else {
                rail::plan(&mut st, &C, &P, &mut rng, year, &pbar, &mut events);
            }
            let mut ck = Checker::new(format!("{} rail.{tag} ({} projects)", set.display(), want.projects.len()));
            check_draws_used(&mut ck, &rng);
            if let Some(want_events) = recorded_events(&set, tag) {
                check_events(&mut ck, &events, &want_events);
            }
            ck.exact("project count", &[st.projects.len()], &[want.projects.len()]);
            for (k, (a, b)) in st.projects.iter().zip(&want.projects).enumerate() {
                ck.exact(&format!("project{k} towns"), &[a.a, a.b, a.town, a.spur as usize], &[b.a, b.b, b.town, b.spur as usize]);
                ck.exact(&format!("project{k} cells"), &a.cells, &b.cells);
                ck.exact(&format!("project{k} new"), &a.new, &b.new);
                ck.vec(&format!("project{k} need"), &a.need, &b.need);
                ck.vec(&format!("project{k} order"), &a.order, &b.order);
                ck.scalar(&format!("project{k} cash"), a.cash, b.cash);
                ck.scalar(&format!("project{k} roi"), a.roi, b.roi);
                ck.scalar(&format!("project{k} cost"), a.cost, b.cost);
                ck.scalar(&format!("project{k} km"), a.length_km, b.length_km);
            }
            ck.vec("hh_cash", &st.hh_cash, &want.hh_cash);
            ck.vec("lord_cash", &st.lord_cash, &want.lord_cash);
            let cool = |s: &isostate_core::State| s.rail_cooldown.iter().map(|(&(a, b), &y)| (a, b, y)).collect::<Vec<_>>();
            ck.exact("rail_cooldown", &cool(&st), &cool(&want));
            ck.exact("rail_links", &st.rail_links.iter().copied().collect::<Vec<_>>(), &want.rail_links.iter().copied().collect::<Vec<_>>());
            ck.exact("spur_done", &st.spur_done.iter().copied().collect::<Vec<_>>(), &want.spur_done.iter().copied().collect::<Vec<_>>());
            ck.finish();
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn people_found_towns_matches_python() {
    for set in sets_with("found_towns") {
        let (C, P) = setup(&set);
        let d = Dump::open(&set.join("found_towns_before"));
        let mut st = load_state(&set.join("found_towns_before"));
        let want = load_state(&set.join("found_towns_after"));
        let replay = recorded_draws(&set, "found_towns");
        let names_replayed = replay.is_some();
        let mut rng = replay.unwrap_or_else(|| make_rngs(0).people);
        let mut events = Vec::new();
        let founded = people::found_towns(&mut st, &C, &P, &mut rng, d.arg_f64("year") as i64, &mut events);
        let mut ck = Checker::new(format!("{} people.found_towns (founded {founded:?})", set.display()));
        // where and when a town is founded is deterministic; only its name is drawn,
        // so names and events are compared only when the draws were recorded
        if names_replayed {
            check_draws_used(&mut ck, &rng);
            ck.exact("town_name", &st.town_name, &want.town_name);
            check_events(&mut ck, &events, &recorded_events(&set, "found_towns").unwrap());
        }
        ck.exact("town_active", &f(&st.town_active), &f(&want.town_active));
        ck.exact("town_node", &uv(&st.town_node), &uv(&want.town_node));
        ck.exact("town_founded", &iv(&st.town_founded), &iv(&want.town_founded));
        ck.exact("town_new", &f(&st.town_new), &f(&want.town_new));
        ck.exact("town count", &[st.town_name.len()], &[want.town_name.len()]);
        ck.mat("price", &st.price, &want.price);
        ck.mat("fill", &st.fill, &want.fill);
        ck.mat("know", &st.know, &want.know);
        ck.vec("wage", &st.wage, &want.wage);
        ck.vec("tools_mult", &st.tools_mult, &want.tools_mult);
        ck.vec("imp_pc", &st.imp_pc, &want.imp_pc);
        assert!(!founded.is_empty(), "the fixture year should found a town");
        ck.finish();
    }
}

#[test]
#[allow(non_snake_case)]
fn firms_entry_matches_python_draw_for_draw() {
    for set in sets_with("entry") {
        let (C, P) = setup(&set);
        let d = Dump::open(&set.join("entry_before"));
        let mut st = load_state(&set.join("entry_before"));
        let want = load_state(&set.join("entry_after"));
        let mut rng = recorded_draws(&set, "entry").expect("entry fixtures record their draws");
        let n0 = st.F_count;
        let made = firms::entry(&mut st, &C, &P, &mut rng, d.arg_f64("year") as i64, &d.arg_mat("pbar"), &d.arg_vec("wbar"), None);
        let mut ck = Checker::new(format!("{} firms.entry ({made} new firms)", set.display()));
        check_draws_used(&mut ck, &rng);
        ck.exact("F_count", &[st.F_count], &[want.F_count]);
        ck.exact("new_firms_year", &st.new_firms_year, &want.new_firms_year);
        ck.exact("f_name", &st.f_name, &want.f_name);
        ck.exact("f_alive", &f(&st.f_alive), &f(&want.f_alive));
        ck.exact("f_town", &uv(&st.f_town), &uv(&want.f_town));
        ck.exact("f_method", &uv(&st.f_method), &uv(&want.f_method));
        ck.exact("f_cell", &uv(&st.f_cell), &uv(&want.f_cell));
        ck.exact("f_born", &iv(&st.f_born), &iv(&want.f_born));
        ck.exact("f_parent", &iv(&st.f_parent), &iv(&want.f_parent));
        ck.exact("firm_at", &iv(&st.firm_at), &iv(&want.firm_at));
        for (name, got, w) in [
            ("f_K", &st.f_K, &want.f_K),
            ("f_order_K", &st.f_order_K, &want.f_order_K),
            ("f_cash", &st.f_cash, &want.f_cash),
            ("f_rent", &st.f_rent, &want.f_rent),
            ("f_u", &st.f_u, &want.f_u),
            ("hh_cash", &st.hh_cash, &want.hh_cash),
            ("lord_cash", &st.lord_cash, &want.lord_cash),
            ("cell_K_share", &st.cell_K_share, &want.cell_K_share),
        ] {
            ck.vec(name, got, w);
        }
        ck.mat("f_order", &st.f_order, &want.f_order);
        // the inspector's record of why each new firm was made (rounded as Python rounds)
        let reasons = Dump::open(&set.join("entry_after")).scalars["f_reason"].as_array().unwrap().clone();
        let want_new = &reasons[n0..];
        let got_new = serde_json::to_value(&st.f_reason[st.f_reason.len() - made..]).unwrap();
        ck.exact("reason count", &[got_new.as_array().unwrap().len()], &[want_new.len()]);
        for (k, (g, w)) in got_new.as_array().unwrap().iter().zip(want_new).enumerate() {
            ck.exact(&format!("f_reason[{}]", n0 + k), &[g.to_string()], &[w.to_string()]);
        }
        assert!(made > 0, "the fixture year should make new firms");
        ck.finish();
    }
}

#[test]
#[allow(non_snake_case)]
fn tech_discovery_matches_python_draw_for_draw() {
    for set in sets_with("discovery") {
        let (C, P) = setup(&set);
        let d = Dump::open(&set.join("discovery_before"));
        let mut st = load_state(&set.join("discovery_before"));
        let want = load_state(&set.join("discovery_after"));
        let mut rng = recorded_draws(&set, "discovery").expect("discovery fixtures record their draws");
        let mut events = Vec::new();
        let found = tech::discovery(&mut st, &C, &P, &mut rng, d.arg_f64("year") as i64, &mut events);
        let mut ck = Checker::new(format!("{} tech.discovery (found {found:?})", set.display()));
        check_draws_used(&mut ck, &rng);
        ck.exact("tech_year", &iv(&st.tech_year), &iv(&want.tech_year));
        ck.exact("tech_town", &iv(&st.tech_town), &iv(&want.tech_town));
        ck.mat("know", &st.know, &want.know);
        check_events(&mut ck, &events, &recorded_events(&set, "discovery").unwrap());
        ck.finish();
    }
}

/// Compare the railway projects of two states.
fn check_projects(ck: &mut Checker, got: &isostate_core::State, want: &isostate_core::State) {
    ck.exact("project count", &[got.projects.len()], &[want.projects.len()]);
    for (k, (a, b)) in got.projects.iter().zip(&want.projects).enumerate() {
        ck.exact(&format!("project{k} towns"), &[a.a, a.b, a.town, a.spur as usize], &[b.a, b.b, b.town, b.spur as usize]);
        ck.exact(&format!("project{k} cells"), &a.cells, &b.cells);
        ck.vec(&format!("project{k} need"), &a.need, &b.need);
        ck.vec(&format!("project{k} order"), &a.order, &b.order);
        ck.scalar(&format!("project{k} cash"), a.cash, b.cash);
    }
}

#[test]
#[allow(non_snake_case)]
fn rail_progress_matches_python() {
    for set in sets_with("rail_progress") {
        let (C, P) = setup(&set);
        let d = Dump::open(&set.join("rail_progress_before"));
        let mut st = load_state(&set.join("rail_progress_before"));
        let want = load_state(&set.join("rail_progress_after"));
        let mut events = Vec::new();
        rail::progress(&mut st, &C, &P, d.arg_f64("year_float"), &mut events);
        let mut ck = Checker::new(format!("{} rail.progress ({} projects)", set.display(), want.projects.len()));
        check_projects(&mut ck, &st, &want);
        ck.exact("rail", &u(&st.rail), &u(&want.rail));
        ck.vec("lord_cash", &st.lord_cash, &want.lord_cash);
        ck.exact("rail_links", &st.rail_links.iter().copied().collect::<Vec<_>>(), &want.rail_links.iter().copied().collect::<Vec<_>>());
        check_events(&mut ck, &events, &recorded_events(&set, "rail_progress").unwrap());
        ck.finish();
    }
}

/// The whole yearly step, glue included (roads, after_transport, resettlement,
/// notable closures and the year's bookkeeping), with every stream replayed.
#[test]
#[allow(non_snake_case)]
fn sim_yearly_matches_python_draw_for_draw() {
    for set in sets_with("yearly") {
        let (C, P) = setup(&set);
        let d = Dump::open(&set.join("yearly_before"));
        let mut st = load_state(&set.join("yearly_before"));
        let want = load_state(&set.join("yearly_after"));
        let after = Dump::open(&set.join("yearly_after"));
        let stream = |k: &str| Rng::replaying(after.f64(&format!("draws_{k}")));
        let mut rngs = Rngs {
            world: stream("world"),
            names: stream("names"),
            firms: stream("firms"),
            tech: stream("tech"),
            people: stream("people"),
            rail: stream("rail"),
            misc: stream("misc"),
        };
        let mut events = Vec::new();
        let year = d.arg_f64("year") as i64;
        sim::yearly(&mut st, &C, &P, &mut rngs, year, &mut events, false);
        let mut ck = Checker::new(format!("{} sim.yearly({year})", set.display()));
        for rng in [&rngs.world, &rngs.names, &rngs.firms, &rngs.tech, &rngs.people, &rngs.rail, &rngs.misc] {
            check_draws_used(&mut ck, rng);
        }
        check_events(&mut ck, &events, &recorded_events(&set, "yearly").unwrap());
        // towns and people
        ck.exact("town_active", &f(&st.town_active), &f(&want.town_active));
        ck.exact("town_name", &st.town_name, &want.town_name);
        ck.exact("tech_year", &iv(&st.tech_year), &iv(&want.tech_year));
        for (name, got, w) in [
            ("pop", &st.pop, &want.pop),
            ("hh_cash", &st.hh_cash, &want.hh_cash),
            ("lord_cash", &st.lord_cash, &want.lord_cash),
            ("merch_cash", &st.merch_cash, &want.merch_cash),
            ("wage", &st.wage, &want.wage),
            ("y_ratio", &st.y_ratio, &want.y_ratio),
            ("tools_mult", &st.tools_mult, &want.tools_mult),
            ("ag_labor", &st.ag_labor, &want.ag_labor),
            ("ind_labor", &st.ind_labor, &want.ind_labor),
            ("food_min_acc", &st.food_min_acc, &want.food_min_acc),
            ("wbar", &st.wbar, &want.wbar),
        ] {
            ck.vec(name, got, w);
        }
        for (name, got, w) in [
            ("price", &st.price, &want.price),
            ("know", &st.know, &want.know),
            ("pbar", &st.pbar, &want.pbar),
            ("town_prod_year", &st.town_prod_year, &want.town_prod_year),
            ("trade_F_year", &st.trade_F_year, &want.trade_F_year),
            ("migration_flow", &st.migration_flow, &want.migration_flow),
            ("dist", &st.dist, &want.dist),
            ("town_cost", &st.town_cost, &want.town_cost),
            ("delta", &st.delta, &want.delta),
            ("lam", &st.lam, &want.lam),
        ] {
            ck.mat(name, got, w);
        }
        // the map: roads, rails, land use, rents
        ck.exact("road", &u(&st.road), &u(&want.road));
        ck.exact("road_quiet", &iv(&st.road_quiet), &iv(&want.road_quiet));
        ck.exact("rail", &u(&st.rail), &u(&want.rail));
        ck.exact("land_use", &u(&st.land_use), &u(&want.land_use));
        ck.exact("catch", &iv(&st.catch), &iv(&want.catch));
        ck.exact("urban", &f(&st.urban), &f(&want.urban));
        ck.exact("derelict_timer", &iv(&st.derelict_timer), &iv(&want.derelict_timer));
        for (name, got, w) in [
            ("traffic", &st.traffic, &want.traffic),
            ("cell_K_share", &st.cell_K_share, &want.cell_K_share),
            ("cell_rent", &st.cell_rent, &want.cell_rent),
            ("land_rent_only", &st.land_rent_only, &want.land_rent_only),
            ("cover", &st.cover, &want.cover),
            ("dcatch", &st.dcatch, &want.dcatch),
            ("urban_res", &st.urban_res, &want.urban_res),
        ] {
            ck.vec(name, got, w);
        }
        // the extractive units
        ck.exact("ex_cell", &uv(&st.ex_cell), &uv(&want.ex_cell));
        ck.exact("ex_sector", &uv(&st.ex_sector), &uv(&want.ex_sector));
        ck.exact("ex_town", &uv(&st.ex_town), &uv(&want.ex_town));
        for (name, got, w) in [
            ("ex_lam", &st.ex_lam, &want.ex_lam),
            ("ex_field", &st.ex_field, &want.ex_field),
            ("ex_tau", &st.ex_tau, &want.ex_tau),
            ("ex_Lprev", &st.ex_Lprev, &want.ex_Lprev),
        ] {
            ck.vec(name, got, w);
        }
        // the firms
        ck.exact("F_count", &[st.F_count], &[want.F_count]);
        ck.exact("f_name", &st.f_name, &want.f_name);
        ck.exact("f_alive", &f(&st.f_alive), &f(&want.f_alive));
        ck.exact("f_cell", &uv(&st.f_cell), &uv(&want.f_cell));
        ck.exact("f_method", &uv(&st.f_method), &uv(&want.f_method));
        ck.exact("f_died", &iv(&st.f_died), &iv(&want.f_died));
        ck.exact("new_firms_year", &st.new_firms_year, &want.new_firms_year);
        ck.exact("closed_year", &st.closed_year, &want.closed_year);
        for (name, got, w) in [
            ("f_K", &st.f_K, &want.f_K),
            ("f_order_K", &st.f_order_K, &want.f_order_K),
            ("f_cash", &st.f_cash, &want.f_cash),
            ("f_rent", &st.f_rent, &want.f_rent),
            ("f_prod", &st.f_prod, &want.f_prod),
            ("f_profit_acc", &st.f_profit_acc, &want.f_profit_acc),
        ] {
            ck.vec(name, got, w);
        }
        ck.mat("f_order", &st.f_order, &want.f_order);
        // the railways
        check_projects(&mut ck, &st, &want);
        let cool = |s: &isostate_core::State| s.rail_cooldown.iter().map(|(&(a, b), &y)| (a, b, y)).collect::<Vec<_>>();
        ck.exact("rail_cooldown", &cool(&st), &cool(&want));
        ck.finish();
    }
}

/// The observer's monthly series (what series_monthly.csv holds), from the same state.
#[test]
#[allow(non_snake_case)]
fn observer_record_month_matches_python() {
    for set in sets_with("record_month") {
        let (C, P) = setup(&set);
        let d = Dump::open(&set.join("record_month_before"));
        let mut st = load_state(&set.join("record_month_before"));
        let (t, n_months) = (d.arg_f64("t") as usize, d.arg_f64("n_months") as usize);
        observer::init(&mut st, &C, &P, n_months);
        observer::record_month(&mut st, &C, &P, t);
        let text = std::fs::read_to_string(set.join("record_month_after").join("series.json")).unwrap();
        let want: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&text).unwrap();
        let mut ck = Checker::new(format!("{} observer.record_month(t = {t})", set.display()));
        let got_names: Vec<&str> = st.series.names.iter().map(|s| s.as_str()).collect();
        let want_names: Vec<&str> = want.keys().map(|s| s.as_str()).collect();
        ck.exact("series names", &got_names, &want_names);
        for (name, w) in &want {
            let got = st.series.get(name)[t];
            match w.as_f64() {
                Some(w) => ck.scalar(name, got, w),
                None => ck.exact(&format!("{name} is NaN"), &[got.is_nan()], &[true]),
            }
        }
        ck.finish();
    }
}

fn check_world(ck: &mut Checker, w: &isostate_core::world::World, ww: &isostate_core::world::World) {
    ck.exact("world W, H", &[w.W, w.H], &[ww.W, ww.H]);
    ck.exact("world land", &f(&w.land), &f(&ww.land));
    ck.exact("world terrain", &u(&w.terrain), &u(&ww.terrain));
    ck.exact("world river", &f(&w.river), &f(&ww.river));
    ck.exact("world navigable", &f(&w.navigable), &f(&ww.navigable));
    ck.exact("world coast", &f(&w.coast), &f(&ww.coast));
    ck.exact("world town_sites", &w.town_sites, &ww.town_sites);
    for (name, got, wv) in [
        ("world h", &w.h, &ww.h),
        ("world fertility", &w.fertility, &ww.fertility),
        ("world pasture", &w.pasture, &ww.pasture),
        ("world forest_pot", &w.forest_pot, &ww.forest_pot),
        ("world rock", &w.rock, &ww.rock),
        ("world clay", &w.clay, &ww.clay),
        ("world ore", &w.ore, &ww.ore),
        ("world coal", &w.coal, &ww.coal),
    ] {
        ck.vec(name, got, wv);
    }
}

/// The initial state, world generation included, with every stream replayed.
#[test]
#[allow(non_snake_case)]
fn sim_init_state_matches_python_draw_for_draw() {
    for set in fixture_sets().into_iter().filter(|s| s.join("init_after").is_dir()) {
        let (C, P) = setup(&set);
        let after = Dump::open(&set.join("init_after"));
        let want = load_state(&set.join("init_after"));
        let stream = |k: &str| Rng::replaying(after.f64(&format!("draws_{k}")));
        // the world first, on its own stream
        let mut world_rng = stream("world");
        let world = isostate_core::world::generate_world(&mut world_rng, P.W, P.H, P.n_towns, 0.58);
        let mut ck = Checker::new(format!("{} world::generate_world", set.display()));
        check_draws_used(&mut ck, &world_rng);
        check_world(&mut ck, &world, &want.world);
        ck.finish();

        // Then the rest of init_state on Python's world. The two worlds differ by
        // rounding (scipy's cubic zoom is matched to about 1e-12, not bit for bit),
        // and that is enough to break exact ties in the first road paths
        // differently, so the rest is checked from Python's world.
        let mut rngs = Rngs {
            world: stream("world"),
            names: stream("names"),
            firms: stream("firms"),
            tech: stream("tech"),
            people: stream("people"),
            rail: stream("rail"),
            misc: stream("misc"),
        };
        let python_world = load_state(&set.join("seed_firms_before")).world;
        let (mut st, pbar, wbar) = sim::init_before_firms(&P, &C, python_world, &mut rngs);
        let want = load_state(&set.join("seed_firms_before"));
        let mut ck = Checker::new(format!("{} sim.init_state up to seed_firms, from Python's world", set.display()));
        check_draws_used(&mut ck, &rngs.names);
        ck.exact("town_name", &st.town_name, &want.town_name);
        ck.exact("road", &u(&st.road), &u(&want.road));
        ck.exact("land_use", &u(&st.land_use), &u(&want.land_use));
        ck.exact("catch", &iv(&st.catch), &iv(&want.catch));
        ck.exact("water", &f(&st.water), &f(&want.water));
        ck.exact("ex_cell", &uv(&st.ex_cell), &uv(&want.ex_cell));
        for (name, got, wv) in [
            ("pop", &st.pop, &want.pop),
            ("dcatch", &st.dcatch, &want.dcatch),
            ("cover", &st.cover, &want.cover),
            ("any_counts", &st.any_counts, &want.any_counts),
            ("ex_lam", &st.ex_lam, &want.ex_lam),
            ("wage", &st.wage, &want.wage),
        ] {
            ck.vec(name, got, wv);
        }
        for (name, got, wv) in [
            ("price", &st.price, &want.price),
            ("dist", &st.dist, &want.dist),
            ("delta", &st.delta, &want.delta),
            ("same_counts", &st.same_counts, &want.same_counts),
            ("lam", &st.lam, &want.lam),
        ] {
            ck.mat(name, got, wv);
        }
        ck.finish();
        sim::seed_firms(&mut st, &C, &P, &mut rngs, &pbar, &wbar);
        let want = load_state(&set.join("init_after"));
        let mut ck = Checker::new(format!("{} sim.init_state, from Python's world", set.display()));
        for rng in [&rngs.names, &rngs.firms, &rngs.tech, &rngs.people, &rngs.rail, &rngs.misc] {
            check_draws_used(&mut ck, rng);
        }
        // towns, land and the first firms
        ck.exact("town_active", &f(&st.town_active), &f(&want.town_active));
        ck.exact("town_node", &uv(&st.town_node), &uv(&want.town_node));
        ck.exact("town_name", &st.town_name, &want.town_name);
        ck.exact("land_use", &u(&st.land_use), &u(&want.land_use));
        ck.exact("catch", &iv(&st.catch), &iv(&want.catch));
        ck.exact("ex_cell", &uv(&st.ex_cell), &uv(&want.ex_cell));
        ck.exact("ex_sector", &uv(&st.ex_sector), &uv(&want.ex_sector));
        ck.exact("F_count", &[st.F_count], &[want.F_count]);
        ck.exact("f_name", &st.f_name, &want.f_name);
        ck.exact("f_cell", &uv(&st.f_cell), &uv(&want.f_cell));
        ck.exact("f_method", &uv(&st.f_method), &uv(&want.f_method));
        ck.exact("f_town", &uv(&st.f_town), &uv(&want.f_town));
        for (name, got, wv) in [
            ("pop", &st.pop, &want.pop),
            ("hh_cash", &st.hh_cash, &want.hh_cash),
            ("lord_cash", &st.lord_cash, &want.lord_cash),
            ("merch_cash", &st.merch_cash, &want.merch_cash),
            ("wage", &st.wage, &want.wage),
            ("cover", &st.cover, &want.cover),
            ("cell_K_share", &st.cell_K_share, &want.cell_K_share),
            ("dcatch", &st.dcatch, &want.dcatch),
            ("ex_lam", &st.ex_lam, &want.ex_lam),
            ("ex_field", &st.ex_field, &want.ex_field),
            ("ex_tau", &st.ex_tau, &want.ex_tau),
            ("f_K", &st.f_K, &want.f_K),
            ("f_cash", &st.f_cash, &want.f_cash),
            ("f_rent", &st.f_rent, &want.f_rent),
        ] {
            ck.vec(name, got, wv);
        }
        for (name, got, wv) in [
            ("price", &st.price, &want.price),
            ("inv", &st.inv, &want.inv),
            ("know", &st.know, &want.know),
            ("dist", &st.dist, &want.dist),
            ("town_cost", &st.town_cost, &want.town_cost),
            ("delta", &st.delta, &want.delta),
            ("lam", &st.lam, &want.lam),
            ("f_order", &st.f_order, &want.f_order),
        ] {
            ck.mat(name, got, wv);
        }
        ck.finish();
    }
}

/// A whole run: Python's world (recorded by parity/record_run.py), every stream
/// replayed, and every monthly series compared. Fixture directories named
/// `run_*` under the fixtures root.
///
/// Every draw lining up means every stochastic decision of the run (each site
/// picked, firm founded and named, discovery made) is the one Python made. The
/// continuous values agree to rounding at first and then drift apart: the wage
/// search is a bisection, so a rounding difference that flips the sign of excess
/// demand at a midpoint moves the wage by about 1e-12, and the economy amplifies
/// such differences by roughly a factor of ten every two to three years. The
/// first recorded decade must agree to 1e-8; later decades are reported. Sooner
/// or later a near-tie falls the other way and the draws stop lining up (for
/// seed 7 in 1786; the yearly step of 1786 from Python's state matches exactly),
/// so the recorded run should end before that.
#[test]
#[allow(non_snake_case)]
fn full_run_matches_python_draw_for_draw() {
    let root = fixtures_root();
    let mut runs: Vec<PathBuf> = std::fs::read_dir(&root)
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir() && p.file_name().unwrap().to_string_lossy().starts_with("run_")).collect())
        .unwrap_or_default();
    runs.sort();
    for dir in runs {
        let P = Params::from_json_str(&std::fs::read_to_string(dir.join("params.json")).unwrap()).unwrap();
        let C = default_content();
        let d = Dump::open(&dir);
        let stream = |k: &str| Rng::replaying(d.f64(&format!("draws_{k}")));
        let mut rngs = Rngs {
            world: Rng::replaying(Vec::new()),
            names: stream("names"),
            firms: stream("firms"),
            tech: stream("tech"),
            people: stream("people"),
            rail: stream("rail"),
            misc: stream("misc"),
        };
        let (mut st, pbar, wbar) = sim::init_before_firms(&P, &C, load_world(&d), &mut rngs);
        sim::seed_firms(&mut st, &C, &P, &mut rngs, &pbar, &wbar);
        let mut run = sim::start_from(&P, C, st, rngs);
        while run.year <= run.end_year {
            sim::run_year(&mut run).unwrap();
        }
        let mut ck = Checker::new(format!("{} whole run to {}", dir.display(), run.end_year));
        for rng in [&run.rngs.names, &run.rngs.firms, &run.rngs.tech, &run.rngs.people, &run.rngs.rail, &run.rngs.misc] {
            check_draws_used(&mut ck, rng);
        }
        let names: Vec<String> = serde_json::from_str(&std::fs::read_to_string(dir.join("series.json")).unwrap()).unwrap();
        let want = d.mat("series");
        let got_names: Vec<&str> = run.st.series.names.iter().map(|s| s.as_str()).collect();
        ck.exact("series names", &got_names, &names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        // the largest relative difference over all series, decade by decade
        let y0 = run.content.start_year;
        let mut worst_by_decade: Vec<(i64, f64, String)> = Vec::new();
        for (k, name) in names.iter().enumerate() {
            let got = run.st.series.get(name);
            let w = want.row(k);
            for (t, (&g, &x)) in got.iter().zip(w).enumerate() {
                let decade = (y0 + (t / 12) as i64) / 10 * 10;
                let rel = if g.is_nan() && x.is_nan() { 0.0 } else { (g - x).abs() / x.abs().max(1e-300) };
                let rel = if (g - x).abs() <= 1e-12 * x.abs().max(1.0) { 0.0 } else { rel };
                match worst_by_decade.iter_mut().find(|e| e.0 == decade) {
                    Some(e) => {
                        if rel > e.1 || rel.is_nan() {
                            *e = (decade, rel, name.clone());
                        }
                    }
                    None => worst_by_decade.push((decade, rel, name.clone())),
                }
            }
        }
        for (decade, rel, name) in &worst_by_decade {
            eprintln!("  {decade}s: largest relative difference {rel:.2e} ({name})");
        }
        let first = &worst_by_decade[0];
        ck.exact(&format!("series in the {}s agree to 1e-8", first.0), &[first.1 <= 1e-8], &[true]);
        ck.finish();
    }
}
