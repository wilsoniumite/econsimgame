//! People (docs/rules/people.md; not in the Python): the population rule has the
//! shape its page says, money is conserved and runs are deterministic with the
//! rules on, no town is founded on a district too small to keep a market, and a
//! town whose market area is a handful of cells all the same does not run away.
//! With the rules off, tests/golden.rs covers that nothing changed.
#![allow(non_snake_case)]

use isostate_core::peopling::{self, PeopleParams};
use isostate_core::*;

/// The port's rules with people on, at the two numbers people was tuned to on
/// the port's island (`y_still` 1.5, `growth_top` 0.0165). The game's are 1.3
/// and 0.016, set where its own island stands in 1750 (docs/rules/people.md, "In
/// company"). The histories below (Kelrigg, the hemmed-in sites) are the port's
/// island's under the first pair, so they are kept on it.
fn params(end_year: i64) -> Params {
    let mut P = default_params();
    P.people = PeopleParams { y_still: 1.5, growth_top: 0.0165, ..PeopleParams::game() };
    P.end_year = Some(end_year);
    P
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// Straight-line distance between two cells, in cells.
fn apart(st: &State, a: usize, b: usize) -> f64 {
    let W = st.world.W;
    ((a / W) as f64 - (b / W) as f64).hypot((a % W) as f64 - (b % W) as f64)
}

#[test]
fn off_by_default_and_on_in_the_game() {
    assert!(!default_params().people.enabled);
    assert!(game_params().people.enabled);
    assert_eq!(PeopleParams::game(), PeopleParams { enabled: true, ..PeopleParams::default() });
}

#[test]
fn the_rate_answers_to_income_and_does_not_explode() {
    let pp = PeopleParams::game();
    // births balance deaths at y_still; below it the town shrinks
    assert!(peopling::income_term(&pp, pp.y_still).abs() < 1e-12);
    assert!(peopling::income_term(&pp, 0.8 * pp.y_still) < 0.0);
    // it rises with income all the way, and by less each time
    let ys = [1.6, 2.0, 2.5, 3.0, 4.0, 6.0, 12.0, 100.0];
    let r: Vec<f64> = ys.iter().map(|&y| peopling::income_term(&pp, y)).collect();
    for k in 1..r.len() {
        assert!(r[k] > r[k - 1], "more income, more people: {r:?}");
    }
    assert!(r[2] - r[1] > r[4] - r[3], "the gain flattens: {r:?}");
    // the ceiling holds however rich the town
    assert!(r[7] < pp.growth_top);
    // incomes of 1900 doubled add less than a third of a point a year
    assert!(r[5] - r[3] < 0.0033, "doubling income from 3 to 6 adds {}", r[5] - r[3]);
    // on today's incomes (2 to 3.5 times subsistence) the rate brackets the port's 1.1 per cent
    assert!(peopling::income_term(&pp, 2.0) < 0.011 && peopling::income_term(&pp, 3.5) > 0.011);
}

#[test]
fn crowding_and_dearth_take_their_toll() {
    let pp = PeopleParams::game();
    // a village costs nothing, a great town where everyone lives on town ground nearly all of town_deaths
    assert_eq!(peopling::crowding_term(&pp, 0.0, 5000.0), 0.0);
    let small = peopling::crowding_term(&pp, 2000.0, 20_000.0);
    let great = peopling::crowding_term(&pp, 60_000.0, 80_000.0);
    assert!(small > 0.0 && small < great && great < pp.town_deaths, "{small} {great}");
    // dearth: none when the worst month fed everyone, complete at dearth_full
    assert_eq!(peopling::dearth_depth(&pp, 1.3), 0.0);
    assert_eq!(peopling::dearth_depth(&pp, pp.dearth_full - 0.1), 1.0);
    let half = peopling::dearth_depth(&pp, 0.5 * (pp.dearth_from + pp.dearth_full));
    assert!((half - 0.5).abs() < 1e-12);
}

#[test]
fn the_probes_district_is_refused() {
    // dx-growth: a town with a catchment of 4 cells and 363 people brought seed 10 down
    let pp = PeopleParams::game();
    assert!(!peopling::district_ok(&pp, 4, 363.0, 0.01));
    // either safeguard alone is enough to refuse, and so is taking too much of one town
    assert!(!peopling::district_ok(&pp, pp.found_cells - 1, 50_000.0, 0.01));
    assert!(!peopling::district_ok(&pp, 500, pp.found_people - 1.0, 0.01));
    assert!(!peopling::district_ok(&pp, 500, 50_000.0, pp.donor_max + 0.01));
    assert!(peopling::district_ok(&pp, pp.found_cells, pp.found_people, pp.donor_max));
}

#[test]
fn people_conserve_money_and_runs_are_deterministic() {
    // six families too, as the game is played; run_year checks the money stock every month
    let mut P = params(1809);
    P.n_families = 6;
    let run = || sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
    let (a, b) = std::thread::scope(|s| {
        let h = s.spawn(run);
        (run(), h.join().unwrap())
    });
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (x, y) in a.state.series.data.iter().zip(&b.state.series.data) {
        assert_eq!(bits(x), bits(y), "the same parameters give the same run");
    }
    assert_eq!(bits(&a.state.pop), bits(&b.state.pop));
    assert_eq!(a.state.town_name, b.state.town_name);
    assert_eq!(a.state.pp_foundings, b.state.pp_foundings);

    // towns were founded, each on a district that can keep a market, well apart
    let pp = &P.people;
    assert!(!st.pp_foundings.is_empty(), "sixty years and no town");
    assert!(st.pp_foundings.len() <= 60, "at most one a year");
    for f in &st.pp_foundings {
        assert!(f.cells >= pp.found_cells && f.people >= pp.found_people, "{f:?}");
        assert!(f.saving >= pp.found_saving, "{f:?}");
        for t in 0..st.T_max {
            if st.town_active[t] && t != f.town && st.town_founded[t] <= f.year {
                let d = apart(st, st.town_node[t], st.town_node[f.town]);
                assert!(d > pp.found_spacing, "{} stands {d:.1} cells from {}", st.town_name[f.town], st.town_name[t]);
            }
        }
    }
    // a founding leaves its givers alone for donor_rest years, so foundings are paced
    let mut years: Vec<i64> = st.pp_foundings.iter().map(|f| f.year).collect();
    years.dedup();
    assert_eq!(years.len(), st.pp_foundings.len(), "one town a year at most");
    // every town still has a market area and people in it
    for t in 0..st.T_max {
        if st.town_active[t] {
            assert!(st.town_cells[t].len() >= pp.found_cells, "{} is left with {} cells", st.town_name[t], st.town_cells[t].len());
            assert!(st.pop[t] > 500.0 && st.pop[t].is_finite(), "{} has {} people", st.town_name[t], st.pop[t]);
        }
    }
    // the towns do not all grow at one rate, and none at the port's cap by rule
    let g: Vec<f64> = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.pp_growth[t]).collect();
    let (lo, hi) = (g.iter().cloned().fold(f64::INFINITY, f64::min), g.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!(hi > lo && hi < pp.growth_top, "rates from {lo} to {hi}");
}

#[test]
fn with_the_rules_on_the_run_differs_and_with_them_off_it_does_not() {
    // the same seed to 1765, three ways
    let mut off = default_params();
    off.end_year = Some(1765);
    let mut off_other_numbers = off.clone();
    off_other_numbers.people.growth_top = 0.03;
    off_other_numbers.people.found_saving = 0.001;
    let on = params(1765);
    let run = |P: &Params| sim::run_with(P, default_content(), false, true).unwrap();
    let (a, b, c) = std::thread::scope(|s| {
        let (hb, hc) = (s.spawn(|| run(&off_other_numbers)), s.spawn(|| run(&on)));
        (run(&off), hb.join().unwrap(), hc.join().unwrap())
    });
    // off: the numbers are not read, and the mechanism's state stays empty
    for (x, y) in a.state.series.data.iter().zip(&b.state.series.data) {
        assert_eq!(bits(x), bits(y), "a switched-off mechanism's numbers change nothing");
    }
    assert!(b.state.pp_growth.is_empty() && b.state.pp_foundings.is_empty() && b.state.pp_gave.is_empty());
    assert!(b.state.pp_rng.is_none() && b.state.pp_emigrants == 0.0);
    // on: fewer towns in the first rush, and a population that is not the capped path
    let towns = |r: &RunResult| (0..r.state.T_max).filter(|&t| r.state.town_active[t]).count();
    let people = |r: &RunResult| r.state.pop.iter().sum::<f64>();
    assert!(towns(&c) < towns(&a), "{} towns with the rules, {} without", towns(&c), towns(&a));
    assert!((people(&c) / people(&a) - 1.0).abs() > 0.01, "{} against {}", people(&c), people(&a));
    assert!(!c.state.pp_foundings.is_empty());
}

/// A wage that ran away without a tiny town. On this seed Kelrigg is founded in
/// 1750 among quarries, on 217 cells. Its carts make roads, and a year later its
/// market area has trebled and taken in a coalfield that used to send its coal
/// to Sandmere. Before prices and the wage went with a district, those pits were
/// offered Kelrigg's price for coal (a buyer's price: Kelrigg had no pits) at the
/// pit mouth, wanted three times the hands the town had, and the wage doubled
/// every month of 1752 to a hundred times the island's. With the body of
/// `peopling::prices_go_with_trade` taken out this test fails in 1752, Kelrigg's
/// wage for the year averaging 15.6 times the middle town's.
#[test]
fn a_district_that_changes_market_does_not_set_off_the_wage() {
    let mut P = params(1756);
    P.seed = 7;
    let mut sim = sim::start(&P, default_content());
    sim.check_money = true;
    let (mut cells_at_founding, mut grew) = (vec![0usize; sim.st.T_max], false);
    while sim.year <= sim.end_year {
        let year = sim.year;
        sim::run_year(&mut sim).unwrap_or_else(|e| panic!("{year}: {e}"));
        let st = &sim.st;
        let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
        for &t in &act {
            if st.town_founded[t] == year {
                cells_at_founding[t] = st.town_cells[t].len();
            } else if st.town_founded[t] >= 1750 && st.town_cells[t].len() > 2 * cells_at_founding[t] {
                grew = true;
            }
        }
        // the year's average wage, town by town, against the middle town's (a town
        // founded this year has no year behind it yet)
        let mut wages: Vec<f64> = act.iter().filter(|&&t| st.town_founded[t] < year).map(|&t| st.wbar[t]).collect();
        wages.sort_by(f64::total_cmp);
        let middle = wages[wages.len() / 2];
        for &t in &act {
            if st.town_founded[t] < year {
                let rel = st.wbar[t] / middle;
                assert!((1.0 / 3.0..=3.0).contains(&rel), "{year}: the wage in {} averaged {rel:.1} times the middle town's", st.town_name[t]);
            }
        }
    }
    assert!(grew, "the case is gone: no new town's market area doubled after its founding");
}

/// The crash the growth probe hit (dx-growth, section 4): a town founded close
/// to a rival got a market area of four cells and 363 people, migrants poured
/// in, its wage went from 0.02 to 500 and the money check failed. The founding
/// rule now refuses such a site (`the_probes_district_is_refused`). This test
/// plants one all the same, six times, each hard by an existing town where it
/// draws a handful of cells, and watches them to 1840. Some take a district
/// from their neighbour the year after. Two stay hamlets for the rest of the
/// run: Strathope on 4 to 22 cells with 55 to 450 people, and Elmmoor on 2 cells
/// with 56 to 150. People go with their cells and arrivals are limited to what
/// a town can house, so nobody pours in. A hamlet with more hands than its two
/// cells can use is poor (Elmmoor's wage sinks to a twelfth of the island's
/// while half its people leave, and comes back to a third), but its wage never
/// runs up, never sits on the floor, and the money check goes on passing.
#[test]
fn a_town_with_a_tiny_market_area_does_not_run_away() {
    const PLANTED: [i64; 6] = [1770, 1775, 1780, 1785, 1790, 1795];
    let P = params(1840);
    let mut sim = sim::start(&P, default_content());
    sim.check_money = true;
    // (town, year planted, cells and people a year ago)
    let mut planted: Vec<(usize, i64, usize, f64)> = Vec::new();
    let (mut least_cells, mut small_years) = (usize::MAX, 0);
    while sim.year <= sim.end_year {
        let year = sim.year;
        if PLANTED.contains(&year) {
            // the inland cell near a town that would draw the fewest cells to a market of its own
            let st = &sim.st;
            let nodes: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.town_node[t]).collect();
            let (mut site, mut fewest) = (usize::MAX, usize::MAX);
            for c in 0..st.N {
                let near = nodes.iter().map(|&n| apart(st, c, n)).fold(f64::INFINITY, f64::min);
                if !(st.world.land[c] && !st.world.coast[c] && st.world.terrain[c] < 3 && st.catch[c] >= 0 && (1.0..=4.0).contains(&near)) {
                    continue;
                }
                let d = transport::distances(&st.graph, &[(c, 0.0)]);
                let cells = (0..st.N).filter(|&k| st.catch[k] >= 0 && d[k] < st.dcatch[k]).count();
                if cells < fewest {
                    (site, fewest) = (c, cells);
                }
            }
            println!("{year}: planting a town on cell {site}, which draws {fewest} cells");
            assert!(fewest <= 8, "no hemmed-in site: the least draws {fewest} cells");
            sim.st.pp_found_next = Some(site);
        }
        sim::run_year(&mut sim).unwrap_or_else(|e| panic!("{year}: {e}"));
        let st = &sim.st;
        if PLANTED.contains(&year) {
            let t = (0..st.T_max).filter(|&t| st.town_active[t]).max_by_key(|&t| st.town_founded[t]).unwrap();
            assert_eq!(st.town_founded[t], year, "the town was planted");
            planted.push((t, year, 0, 0.0));
        }
        let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
        let people: f64 = act.iter().map(|&t| st.pop[t]).sum();
        let wage = act.iter().map(|&t| st.wage[t] * st.pop[t]).sum::<f64>() / people;
        for (t, since, cells_before, people_before) in planted.iter_mut() {
            let (t, cells) = (*t, st.town_cells[*t].len());
            least_cells = least_cells.min(cells);
            small_years += (cells < P.people.found_cells) as usize;
            let rel = st.wage[t] / wage;
            println!("{year} {:12} {cells:4} cells {:8.0} people, wage {:.2} of the island's", st.town_name[t], st.pop[t], rel);
            assert!(st.pop[t].is_finite() && st.pop[t] >= 0.0, "{year}: {} people", st.pop[t]);
            // in its first year a town has its donor's wage and nobody yet at work
            if year > *since {
                // (the wage's floor of 0.02 is under a hundredth of the island's)
                assert!((0.04..=4.0).contains(&rel), "{year}: the wage in {} is {rel:.2} of the island's", st.town_name[t]);
                // nobody pours in: while the market area is no larger, a year adds at most
                // the births and the arrivals a town can house
                if cells <= *cells_before {
                    let most = *people_before * (1.0 + P.people.growth_top) * (1.0 + P.people.arrivals_max);
                    assert!(st.pop[t] <= most + 1.0, "{year}: {} went from {people_before:.0} to {:.0} people on {cells} cells", st.town_name[t], st.pop[t]);
                }
            }
            (*cells_before, *people_before) = (cells, st.pop[t]);
        }
    }
    assert!(least_cells <= 8, "the planted towns were meant to be tiny; the smallest area was {least_cells} cells");
    assert!(small_years >= 60, "a town was meant to stay hemmed in; it was for {small_years} town-years");
}

/// In company (docs/rules/people.md, "In company"): the whole of the game's
/// rules with six families, seed 4 to 1800, month by month. When people was
/// merged a new town's wage went to three or four times the middle town's in
/// its first spring, on every island, and a town's wage doubled and halved month
/// about in a quarter of all town-months. Both were the carters: hired all at
/// once on last month's bill, in a district that had just become an exporter
/// with a market of its own. With carters taken on by degrees
/// (`carting.adjust`), and the merchants' anchors going with the prices a
/// district brings (`prices::anchor_follows`), neither happens.
#[test]
fn in_company_no_wage_runs_away_or_flips() {
    let mut P = game_params();
    P.seed = 4;
    P.n_families = 6;
    P.end_year = Some(1800);
    let mut sim = sim::start(&P, game_content());
    sim.check_money = true;
    let T = sim.st.T_max;
    let (mut flips, mut months) = (0usize, 0usize);
    let mut last = sim.st.wage.clone();
    let mut young_months = 0;
    while sim.year <= sim.end_year {
        let year = sim.year;
        for month in 0..12 {
            sim::run_month(&mut sim, month).unwrap_or_else(|e| panic!("{year}: {e}"));
            let st = &sim.st;
            let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t] && st.pop[t] > 0.0).collect();
            let mut wages: Vec<f64> = act.iter().map(|&t| st.wage[t]).collect();
            wages.sort_by(f64::total_cmp);
            let middle = wages[wages.len() / 2];
            for &t in &act {
                let rel = st.wage[t] / middle;
                assert!((0.2..=3.0).contains(&rel), "{year}-{:02}: the wage in {} (founded {}) is {rel:.2} times the middle town's", month + 1, st.town_name[t], st.town_founded[t]);
                if month > 0 && last[t] > 0.0 {
                    months += 1;
                    flips += ((st.wage[t] / last[t]).ln().abs() >= 1.5f64.ln()) as usize;
                }
                young_months += (st.town_founded[t] > sim.content.start_year && year - st.town_founded[t] <= 2) as usize;
                // the anchors go with the prices: no market's price is left far from what
                // its merchants think the good is worth (the markup is at most e^0.5)
                if st.town_founded[t] == year - 1 && month == 0 {
                    for g in 0..sim.content.G {
                        let r = st.price[(t, g)] / st.price_anchor[(t, g)];
                        assert!((0.5..=2.0).contains(&r), "{year}: {} opens with {} at {r:.2} times its anchor", st.town_name[t], sim.content.good_name[g]);
                    }
                }
            }
            last = st.wage.clone();
        }
        let Sim { st, content: C, params: P, rngs, events, rec, .. } = &mut sim;
        sim::yearly(st, C, P, rngs, year, events, false);
        observer::yearly(st, C, P, year, events);
        record::snapshot(rec, st, C, P, (year - C.start_year) as usize);
        sim.year += 1;
    }
    assert!(sim.st.pp_foundings.len() >= 3 && young_months >= 60, "the case is gone: {} towns founded", sim.st.pp_foundings.len());
    let share = flips as f64 / months as f64;
    assert!(share < 0.005, "the wage moved by half or more in {share:.4} of town-months");
}

/// People who take ship after a hungry year are news of their own kind, and not
/// hunger news (R195 in the backlog's sweep of findings; docs/rules/scorecard.md):
/// the sweep counts them apart, as `news_emigration` and `emigration_years`.
/// Island 6 under the game's rules has the one sailing of sixteen islands, in
/// 1759 (it was island 2 in 1827 until the money was issued on its true total,
/// docs/rules/money.md, which sent every island down another history).
#[test]
fn a_sailing_is_news_of_its_own_kind() {
    let rules = isostate_core::params::Rules::Game;
    let mut P = rules.params();
    P.seed = 6;
    P.n_families = 6;
    P.end_year = Some(1759);
    let r = sim::run_with(&P, rules.content(), false, false).expect("the run finishes");
    let sailed: Vec<&String> = r.events.iter().filter(|e| e.kind == "emigration").map(|e| &e.title).collect();
    assert!(!sailed.is_empty(), "the case is gone: nobody took ship on island 6 by 1759");
    assert!(sailed.iter().all(|t| t.contains("leave the island")), "{sailed:?}");
    let hunger: Vec<&String> = r.events.iter().filter(|e| e.kind == "famine").map(|e| &e.title).collect();
    assert!(hunger.iter().all(|t| !t.contains("leave the island")), "a sailing filed as hunger: {hunger:?}");
}
