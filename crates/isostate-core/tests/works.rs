//! The life cycle for works (`docs/rules/works.md`, not in the Python): switched
//! off it is the port (tests/golden.rs holds that to the byte); switched on,
//! money is conserved, a run is deterministic, works grow where they stand
//! instead of multiplying, no cell holds more works than it has ground for, a
//! year's new capacity stays within the year's room, and a family extends the
//! works it has.
#![allow(non_snake_case)]

use isostate_core::mat::Mat;
use isostate_core::works::{self, WorksParams};
use isostate_core::*;

/// The port with the life cycle switched on, as `--set works.enabled=true` runs it.
fn params(seed: u64, n_families: usize, end_year: i64) -> Params {
    let mut P = default_params();
    P.seed = seed;
    P.n_families = n_families;
    P.end_year = Some(end_year);
    P.works = WorksParams::game();
    P
}

fn port(seed: u64, n_families: usize, end_year: i64) -> Params {
    let mut P = params(seed, n_families, end_year);
    P.works = WorksParams::default();
    P
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn alive(st: &State) -> Vec<usize> {
    (0..st.F_count).filter(|&f| st.f_alive[f]).collect()
}

fn method(C: &Content, name: &str) -> usize {
    C.method_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("no method {name}"))
}

#[test]
fn off_is_the_port_and_on_is_the_game() {
    assert_eq!(default_params().works, WorksParams::default());
    assert!(!default_params().works.enabled, "the port has no life cycle");
    assert!(game_params().works.enabled, "the game has");
    // the numbers are the same either way, so --set works.enabled=true measures the game's
    assert_eq!(WorksParams { enabled: false, ..WorksParams::game() }, WorksParams::default());
    // and a port run never fills the state's tables
    let r = sim::run_with(&port(3, 0, 1752), default_content(), false, true).expect("money is conserved");
    assert_eq!(r.state.works_added.rows, 0);
    assert!(r.state.works_lease.is_empty());
}

#[test]
fn the_sums_of_the_life_cycle() {
    let C = default_content();
    let (off, on) = (port(7, 0, 1750), params(7, 0, 1750));
    let m = method(&C, "smithy");
    let K0 = C.m_K0[m];

    // A. the port calls any open order building; here only one worth a twentieth of the works
    assert!(works::building(&off, 10.0, 1e-4));
    assert!(!works::building(&on, 10.0, 1e-4));
    assert!(!works::building(&on, 10.0, 0.5), "0.5 on order beside 10 built is under 5 per cent");
    assert!(works::building(&on, 10.0, 0.6));
    assert!(works::building(&on, 0.0, 1.0), "a works with nothing built yet is building");

    // B. land by size under the port (a quarter of a cell at the usual size, four sizes the limit) ...
    assert!((works::land_take(&C, &off, m, K0) - 0.25).abs() < 1e-12);
    assert!((works::max_K(&C, &off, m) - 4.0 * K0).abs() < 1e-12);
    assert_eq!(works::sister_K(&C, &off, m, 0.01), K0);
    // ... and by hands here: the limit is the capacity whose hands take `cell_limit` of a cell
    let take = works::land_take(&C, &on, m, K0);
    assert!((take - K0 * C.m_labor[m] / on.works.hands_per_cell).abs() < 1e-12);
    assert!(take < 0.25, "a smithy of the usual size takes a yard, not a square kilometre: {take}");
    let limit = works::max_K(&C, &on, m);
    assert!((works::land_take(&C, &on, m, limit) - on.works.cell_limit).abs() < 1e-9);
    assert!(limit > 4.0 * K0, "the cell is a looser limit than four usual sizes");
    // a sister works is the size of the growth it stands in for, never under the smallest works built
    assert_eq!(works::sister_K(&C, &on, m, 3.0 * K0), 3.0 * K0);
    assert_eq!(works::sister_K(&C, &on, m, 0.01), on.entry_K_frac * K0);

    // C. the spread comes off the price entry reckons with
    assert_eq!(works::entry_spread(&off, 2.0), 0.0);
    assert!((works::entry_spread(&on, 2.0) - 2.0 * on.merchant_spread).abs() < 1e-12);
    let mut no_c = on.clone();
    no_c.works.count_spread = false;
    assert_eq!(works::entry_spread(&no_c, 2.0), 0.0);
}

#[test]
fn a_site_promises_what_a_works_gets() {
    // the same 1750, looked at under the port's sums and under the life cycle's
    let off = port(7, 0, 1750);
    let (st, _) = init_state(&off, &default_content());
    let C = default_content();
    let on = params(7, 0, 1750);
    let m = method(&C, "hand_loom");
    let i = (0..st.T_max).find(|&i| st.town_active[i]).expect("a town");
    let mut cells: Vec<usize> = st.town_cells[i].iter().copied().filter(|&c| st.land_use[c] != 0 && st.dcatch[c].is_finite()).collect();
    cells.sort_by(|&a, &b| st.dcatch[a].total_cmp(&st.dcatch[b]));
    let (near, far) = (cells[0], *cells.last().unwrap());
    assert!(st.dcatch[far] > st.dcatch[near] + 3.0, "the town's reach has a far side");
    let K = on.entry_K_frac * C.m_K0[m];
    let (pbar, wbar) = (&st.price, &st.wage);
    let a = firms::evaluate_sites(&st, &C, &off, i, m, &[near, far], K, pbar, wbar);
    let b = firms::evaluate_sites(&st, &C, &on, i, m, &[near, far], K, pbar, wbar);
    let out = C.method_out[m];
    for j in 0..2 {
        // C. the merchant's spread is off the price
        let spread = on.merchant_spread * pbar[(i, out)];
        assert!((a.pout[j] - b.pout[j] - spread).abs() < 1e-9, "site {j}: {} against {}", a.pout[j], b.pout[j]);
        // F. hands walk: labour per unit rises with the path cost to the market
        let d = st.dcatch[[near, far][j]];
        assert!((b.l[j] / a.l[j] - (1.0 + on.works.walk * d)).abs() < 1e-9);
        assert!(b.margin[j] < a.margin[j], "so a unit promises less than the port said");
        // B. the land bid is for the land its hands need
        assert!(b.rent_bid[j] <= a.rent_bid[j]);
    }
    assert!(b.l[1] > b.l[0], "the far site needs more hands for the same cloth");
}

#[test]
fn the_life_cycle_conserves_money_and_is_deterministic() {
    // six families for fifty years; run_with checks the money stock every month
    let P = params(3, 6, 1799);
    let a = sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
    let b = sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (x, y) in a.state.series.data.iter().zip(&b.state.series.data) {
        assert_eq!(bits(x), bits(y), "the same parameters give the same run");
    }
    assert_eq!(bits(&a.state.f_K), bits(&b.state.f_K));
    assert_eq!(bits(&a.state.f_rent), bits(&b.state.f_rent));
    assert_eq!(a.state.f_owner, b.state.f_owner);
    let titles = |r: &sim::RunResult| r.events.iter().map(|e| e.title.clone()).collect::<Vec<_>>();
    assert_eq!(titles(&a), titles(&b));
    // and without families, a little longer
    let P = params(5, 0, 1809);
    sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
}

/// What the test below reads off a run's last state.
struct Shape {
    works: usize,
    largest: f64,
    industry_cells: usize,
    /// mean path cost to market of the hands of mills, looms, smithies and engine works
    walk: f64,
}

fn shape(st: &State, C: &Content) -> Shape {
    let ids = alive(st);
    let light: Vec<usize> = ["textile_mill", "hand_loom", "smithy", "engine_works"].iter().map(|n| method(C, n)).collect();
    let hands: f64 = ids.iter().filter(|&&f| light.contains(&st.f_method[f])).map(|&f| st.f_labor[f]).sum();
    let walked: f64 = ids.iter().filter(|&&f| light.contains(&st.f_method[f])).map(|&f| st.f_labor[f] * st.dcatch[st.f_cell[f]]).sum();
    Shape {
        works: ids.len(),
        largest: ids.iter().map(|&f| st.f_K[f] / C.m_K0[st.f_method[f]]).fold(0.0, f64::max),
        industry_cells: st.land_use.iter().filter(|&&u| u == land::USE_INDUSTRY).count(),
        walk: walked / hands.max(1e-9),
    }
}

#[test]
fn works_grow_where_they_stand() {
    let end = 1829;
    let old = sim::run_with(&port(7, 0, end), default_content(), false, true).expect("money is conserved");
    let before = shape(&old.state, &old.content);

    let P = params(7, 0, end);
    let mut s = sim::start(&P, default_content());
    let C = s.content.clone();
    let (mut founded, mut sisters, mut grew) = (0, 0, 0);
    while s.year <= s.end_year {
        let K_before: Vec<f64> = s.st.f_K.iter().zip(&s.st.f_order_K).map(|(k, o)| k + o).collect();
        sim::run_year(&mut s).expect("money is conserved every month");
        let st = &s.st;
        let ids = alive(st);

        // B. no works is larger than its cell allows, and no cell holds more than its ground
        let mut held = vec![0.0; st.N];
        for &f in &ids {
            let m = st.f_method[f];
            let take = works::land_held(st, &C, &P, f);
            assert!(take <= P.works.cell_limit.max(works::land_take(&C, &P, m, P.entry_K_frac * C.m_K0[m])) + 1e-9, "{} takes {take} of its cell", st.f_name[f]);
            assert!(st.works_lease[f] >= take - 1e-9, "{} pays rent on less land than it holds", st.f_name[f]);
            assert!(st.f_rent[f] > 0.0);
            held[st.f_cell[f]] += take;
        }
        for c in 0..st.N {
            assert!(held[c] <= 1.0 + 1e-9, "cell {c} holds {} of itself in {}", held[c], s.year - 1);
            assert!((st.cell_K_share[c] - held[c]).abs() < 1e-9, "the cell's built share is the land its works hold");
        }

        // D. what a town's trade ordered this year fits the year's room, give or
        // take the smallest works (which a sister works or a first works is never under)
        let mut floors = Mat::zeros(st.T_max, C.M);
        for &f in &st.new_firms_year {
            floors[(st.f_town[f], st.f_method[f])] += 1.0;
            match st.f_reason[f].kind.as_str() {
                "expansion" => sisters += 1,
                _ => founded += 1,
            }
        }
        for i in (0..st.T_max).filter(|&i| st.town_active[i]) {
            for m in 0..C.M {
                let room = works::year_budget(st, &C, &P, i, m);
                let slack = floors[(i, m)] * P.entry_K_frac * C.m_K0[m];
                assert!(st.works_added[(i, m)] <= room + slack + 1e-9, "{} in {} ordered {} with room for {room}", C.method_name[m], st.town_name[i], st.works_added[(i, m)]);
            }
        }
        grew += ids.iter().filter(|&&f| f < K_before.len() && K_before[f] > 0.0 && st.f_K[f] + st.f_order_K[f] > 1.05 * K_before[f]).count();
    }
    let after = shape(&s.st, &C);

    assert!(founded > 0 && sisters > 0 && grew > 100, "works are founded ({founded}), open sister works ({sisters}) and grow ({grew})");
    assert!(2 * after.works < before.works, "fewer works: {} against the port's {}", after.works, before.works);
    assert!(before.largest <= 4.0 && after.largest > 8.0, "larger works: {} against the port's {}", after.largest, before.largest);
    assert!(4 * after.industry_cells < before.industry_cells, "less land: {} cells against the port's {}", after.industry_cells, before.industry_cells);
    assert!(2.0 * after.walk < before.walk, "light trades stand by their town: {} against the port's {}", after.walk, before.walk);
    // the economy is no smaller for it
    let gdp = |st: &State| *st.series.get("gdp_pc").last().unwrap();
    assert!(gdp(&s.st) > 0.9 * gdp(&old.state));
}

#[test]
fn a_family_extends_the_works_it_has() {
    let count = |st: &State| -> (usize, f64) {
        let own: Vec<usize> = alive(st).into_iter().filter(|&f| st.f_owner[f] >= 0).collect();
        (own.len(), own.iter().map(|&f| st.f_K[f]).sum::<f64>())
    };
    let old = sim::run_with(&port(7, 6, 1799), default_content(), false, true).expect("money is conserved");
    let new = sim::run_with(&params(7, 6, 1799), default_content(), false, true).expect("money is conserved");
    let ((n_old, _), (n_new, K_new)) = (count(&old.state), count(&new.state));
    assert!(n_new > 0 && K_new > 0.0, "families have works");
    assert!(3 * n_new < n_old, "and far fewer of them: {n_new} against the port's {n_old}");
    // no family has two works of one method in one town while the first has room
    let st = &new.state;
    let C = &new.content;
    let P = &new.params;
    let own: Vec<usize> = alive(st).into_iter().filter(|&f| st.f_owner[f] >= 0 && st.f_parent[f] < 0).collect();
    let mut twice = 0;
    for &f in &own {
        let m = st.f_method[f];
        let roomy = |g: usize| st.f_K[g] + st.f_order_K[g] < 0.5 * works::max_K(C, P, m) && st.cell_K_share[st.f_cell[g]] < 0.5;
        if own.iter().any(|&g| g < f && st.f_owner[g] == st.f_owner[f] && st.f_town[g] == st.f_town[f] && st.f_method[g] == m && roomy(g)) {
            twice += 1;
        }
    }
    assert!(10 * twice <= own.len(), "{twice} of {} family works were founded beside one with room", own.len());
    // the port has no such rule
    assert!(works::family_extension(&old.state, &old.content, &old.params, 0, 0, 0, 1.0).is_none());
}
