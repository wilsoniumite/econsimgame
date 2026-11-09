//! Families (not in the Python): money is conserved and runs are deterministic
//! with families on, the player's actions work through the API, AI families
//! invest, and the live API reports every year once. With families off the
//! parity tests (tests/parity.rs) cover that nothing changed.
#![allow(non_snake_case)]

use std::collections::BTreeSet;

use isostate_core::families::{self, Group, Strategy};
use isostate_core::*;

fn params(n_families: usize, end_year: i64) -> Params {
    let mut P = default_params();
    P.n_families = n_families;
    P.end_year = Some(end_year);
    P
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1.0)
}

#[test]
fn families_conserve_money_and_runs_are_deterministic() {
    // six families for sixty years; run_year checks the money stock every month
    let P = params(6, 1809);
    let a = sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
    let b = sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(rel(drift, 0.0) <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (x, y) in a.state.series.data.iter().zip(&b.state.series.data) {
        assert_eq!(bits(x), bits(y), "the same parameters give the same run");
    }
    assert_eq!(a.state.f_owner, b.state.f_owner);
    assert_eq!(a.state.cell_owner, b.state.cell_owner);
    for (fa, fb) in a.state.families.iter().zip(&b.state.families) {
        assert_eq!(fa.cash.to_bits(), fb.cash.to_bits());
        assert_eq!(fa.members.len(), fb.members.len());
    }
    // families own a part of the economy, not most of it
    let (works, rent, money) = families::ownership_shares(st, &a.content);
    assert!(works > 0.0 && rent > 0.0 && money > 0.0, "families own something: {works} {rent} {money}");
    assert!(works < 0.5 && rent < 0.5 && money < 0.5, "families do not own most of it: {works} {rent} {money}");
    // every family keeps at least three living members, with names
    for f in &st.families {
        assert!(f.members.iter().filter(|m| m.alive).count() >= 3);
        assert!(f.members.iter().all(|m| !m.first.is_empty() && (0.0..=1.0).contains(&m.skill)));
    }
}

#[test]
fn the_player_founds_works_buys_land_and_assigns_members() {
    let P = params(3, 1760);
    let mut s = sim::start(&P, default_content());
    sim::run_year(&mut s).unwrap();
    let home = s.st.families[0].home;
    assert_eq!(s.st.families[0].strategy, Strategy::Player);

    // the best affordable works in the player's home town's reach
    let cash = s.st.families[0].cash;
    let mut best: Option<(f64, usize, usize, f64)> = None;
    for c in (0..s.st.N).filter(|&c| s.st.catch[c] == home as i64) {
        let Ok(options) = site_options(&s.st, &s.content, &s.params, 0, c) else { continue };
        assert_eq!(options.len(), s.content.M, "one option per method");
        for o in options {
            if let Ok(x) = &o.sums {
                assert!(x.capital_cost > 0.0 && x.K > 0.0 && x.town == home);
                if x.capital_cost <= cash && best.is_none_or(|b| x.expected_return > b.0) {
                    best = Some((x.expected_return, o.method, c, x.capital_cost));
                }
            }
        }
    }
    let (_, m, cell, cost) = best.expect("the player can afford some works");
    let money0 = econ::money_total(&s.st);
    let n_events = s.events.len();
    let fid = found_works(&mut s.st, &s.content, &s.params, 0, m, cell, s.year, &mut s.events).unwrap();
    assert!(s.st.f_alive[fid] && s.st.f_owner[fid] == 0 && s.st.f_cell[fid] == cell && s.st.f_method[fid] == m);
    assert_eq!(s.st.f_reason[fid].kind, "family");
    assert!(rel(s.st.families[0].cash, cash - cost) < 1e-12, "the works is paid from the family's cash");
    assert!(rel(econ::money_total(&s.st), money0) < 1e-12, "founding works moves money, it makes none");
    assert_eq!(s.events.len(), n_events + 1);
    let e = s.events.last().unwrap();
    assert!(e.family == Some(0) && e.firm == Some(fid) && e.title.contains(&s.st.families[0].surname));

    // errors: sea, a method nobody knows yet, too little cash
    let sea = (0..s.st.N).find(|&c| !s.st.world.land[c]).unwrap();
    assert!(found_works(&mut s.st, &s.content, &s.params, 0, m, sea, s.year, &mut s.events).unwrap_err().contains("sea"));
    let unknown = (0..s.content.M).find(|&mm| !(s.st.know[(home, mm)] > 0.02)).unwrap();
    assert!(found_works(&mut s.st, &s.content, &s.params, 0, unknown, cell, s.year, &mut s.events).unwrap_err().contains("not known"));
    let kept = s.st.families[0].cash;
    s.st.families[0].cash = 0.0;
    s.st.lord_cash[home] += kept; // keep the ledger whole while the family is broke
    assert!(found_works(&mut s.st, &s.content, &s.params, 0, m, cell, s.year, &mut s.events).unwrap_err().contains("costs"));
    s.st.lord_cash[home] -= kept;
    s.st.families[0].cash = kept;
    assert!(found_works(&mut s.st, &s.content, &s.params, 9, m, cell, s.year, &mut s.events).is_err());

    // land: a farm of the anonymous gentry; the price goes to the town's landlords
    let farm = (0..s.st.N)
        .filter(|&c| s.st.catch[c] == home as i64 && s.st.cell_owner[c] < 0 && s.st.land_use[c] == 2 && s.st.cell_rent_year[c] > 0.0)
        .min_by(|&a, &b| families::land_price(&s.st, &s.params, a).total_cmp(&families::land_price(&s.st, &s.params, b)))
        .expect("a farm near home");
    let price = families::land_price(&s.st, &s.params, farm);
    let (cash, lords, money0) = (s.st.families[0].cash, s.st.lord_cash[home], econ::money_total(&s.st));
    assert!(price > 0.0 && price <= cash, "the player can buy a farm ({price:.0} of {cash:.0})");
    assert_eq!(buy_land(&mut s.st, &s.content, &s.params, 0, farm).unwrap(), price);
    assert_eq!(s.st.cell_owner[farm], 0);
    assert!(rel(s.st.families[0].cash, cash - price) < 1e-12 && rel(s.st.lord_cash[home], lords + price) < 1e-12);
    assert!(rel(econ::money_total(&s.st), money0) < 1e-12, "buying land moves money, it makes none");
    assert!(buy_land(&mut s.st, &s.content, &s.params, 0, farm).unwrap_err().contains("already"));
    assert!(buy_land(&mut s.st, &s.content, &s.params, 0, sea).unwrap_err().contains("sea"));

    // members: children cannot manage, adults can, and the bonus follows
    let fam = &s.st.families[0];
    let adult = fam.members.iter().position(|x| x.alive && x.age >= 18).unwrap();
    if let Some(child) = fam.members.iter().position(|x| x.alive && x.age < 18) {
        assert!(assign_member(&mut s.st, 0, child, Some(Group::Iron)).unwrap_err().contains("too young"));
    }
    let g = Group::of_method(&s.content, m).unwrap_or(Group::Estates);
    assign_member(&mut s.st, 0, adult, Some(g)).unwrap();
    let r = family_report(&s.st, &s.content, &s.params, 0).unwrap();
    assert!(r.bonus[g.index()] > 0.0 && r.bonus[g.index()] <= s.params.family_bonus);
    assert_eq!(r.members[adult].group, Some(g));
    assert!(r.firms.contains(&fid) && r.cells.contains(&farm));
    assert!(r.net_worth > r.cash && r.works_value > 0.0 && r.land_value > 0.0);

    // a year later the farm has paid its rent to the family, money still conserved
    sim::run_year(&mut s).expect("money is conserved");
    let r = family_report(&s.st, &s.content, &s.params, 0).unwrap();
    assert!(r.last_year.rents > 0.0, "the farm paid rent to the family");
    assert!(r.last_year.spending > 0.0 && r.last_year.invested >= cost + price);
}

#[test]
fn ai_families_invest() {
    let P = params(6, 1775);
    let mut s = sim::start(&P, default_content());
    let cells0: Vec<usize> = (0..6).map(|k| s.st.cell_owner.iter().filter(|&&o| o == k as i64).count()).collect();
    while s.year <= s.end_year {
        sim::run_year(&mut s).unwrap();
    }
    let st = &s.st;
    let built: Vec<usize> = (0..st.F_count).filter(|&f| st.f_reason[f].kind == "family").collect();
    assert!(!built.is_empty(), "AI families found works");
    assert!(built.iter().all(|&f| st.f_owner[f] >= 1), "the player (family 0) does nothing by itself");
    for k in 1..6 {
        let f = &st.families[k];
        let cells = st.cell_owner.iter().filter(|&&o| o == k as i64).count();
        match f.strategy {
            Strategy::Landed => assert!(cells > cells0[k], "{} bought land ({} -> {cells})", f.surname, cells0[k]),
            _ => assert!(built.iter().any(|&w| st.f_owner[w] == k as i64), "{} built works", f.surname),
        }
        // AI families put their adults in charge of what they hold
        assert!(f.members.iter().any(|m| m.group.is_some()), "{} has managers", f.surname);
    }
    assert!(s.events.iter().any(|e| e.kind == "family" && e.family.is_some()));
}

#[test]
fn live_api_reports_every_year_once() {
    let P = params(6, 1755);
    let mut s = sim::start(&P, default_content());
    let stat = live_static(&s.st, &s.content);
    let N = stat.W * stat.H;
    assert!(stat.elev.len() == N && stat.terrain.len() == N && stat.river.len() == N && stat.down.len() == N);
    assert!(stat.fields.len() == 7 && stat.fields.iter().all(|f| f.len() == N));
    assert_eq!(stat.series.len(), s.st.series.names.len());
    assert!(stat.groups.len() == 6 && stat.layers.len() == 9 && stat.methods.len() == s.content.M);
    let mut cursor = LiveCursor::new(&s.content);
    let mut firms_seen = BTreeSet::new();
    let mut towns_seen = BTreeSet::new();
    let mut n_events = 0;
    let mut slowest = 0.0f64;
    while s.year <= s.end_year {
        sim::run_year(&mut s).unwrap();
        let t = std::time::Instant::now();
        let y = live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        slowest = slowest.max(t.elapsed().as_secs_f64());
        assert_eq!(y.year, s.year - 1);
        assert_eq!(y.index as i64, y.year - s.content.start_year);
        for l in [&y.layers.land_use, &y.layers.mix_a, &y.layers.mix_s, &y.layers.rent, &y.layers.catch] {
            assert_eq!(l.len(), N);
        }
        assert_eq!(&y.layers.mix_a[..], &s.rec.layer("mix_a")[y.index * N..(y.index + 1) * N], "the same layers as run.bin");
        assert!(y.new_firms.iter().all(|f| firms_seen.insert(f.id)), "each firm is sent once");
        assert!(y.new_towns.iter().all(|t| towns_seen.insert(t.id)));
        assert!(y.firm_K.iter().all(|&(f, _)| s.st.f_alive[f]));
        assert!(y.closed_firms.iter().all(|&f| !s.st.f_alive[f]));
        assert_eq!(y.series.len(), stat.series.len());
        assert!(y.series[0].is_finite() && y.series[0] > 0.0, "population is recorded");
        assert_eq!(y.families.len(), 6);
        assert_eq!(y.cell_owner.len(), N);
        assert_eq!(y.towns.price.len(), s.st.T_max * s.content.G);
        n_events += y.events.len();
    }
    assert_eq!(firms_seen.len(), s.st.F_count);
    assert_eq!(n_events, s.events.len(), "each event is sent once");
    assert!(slowest < 0.1, "live_year took {:.1} ms", slowest * 1e3);
    // family works carry their owner in the firm info
    let owned = s.st.f_owner.iter().filter(|&&o| o >= 0).count();
    assert!(owned > 0);
}
