//! The shelves go with the district (docs/rules/shelves.md; stage 3 of the pass
//! over the rules, docs/designs/rules-pass.md 4.2; not in the Python). When a
//! district changes market, its merchants' stock, their cash above nothing and
//! their measure of sales go with it to whichever town takes it, as its people
//! do. These tests hold the rule, hold that nothing is made or lost by it, to the
//! unit of goods and the penny, and hold the January it was made for. Each fails
//! with the rule taken out. With the switch off, tests/golden.rs and the game's
//! sweep against main hold that nothing changed.
#![allow(non_snake_case)]

use isostate_core::mat::Mat;
use isostate_core::*;

#[test]
fn off_by_default_and_on_in_the_game() {
    assert!(!default_params().people.shelves_go);
    assert!(game_params().people.shelves_go);
    // a port run's params.json does not mention it
    let j = serde_json::to_string(&default_params()).unwrap();
    assert!(!j.contains("shelves_go"), "the port's parameters name the switch");
}

/// An island of the game's in 1760 on island 3, its market areas as they stand
/// in both `prev_catch` and `catch`, so that a test can redraw them by hand.
fn island() -> (State, Content, Params) {
    let mut P = game_params();
    P.seed = 3;
    P.n_families = 6;
    P.end_year = Some(1760);
    let r = sim::run_with(&P, game_content(), false, true).expect("the run finishes");
    let mut st = r.state;
    st.prev_catch = st.catch.clone();
    assert!(st.town_new.iter().all(|&v| !v), "no town is new");
    (st, r.content, P)
}

/// A town that stood before 1760 and has over a thousand people.
fn old(st: &State, t: usize) -> bool {
    st.town_active[t] && st.town_founded[t] < 1760 && st.pop[t] > 1000.0
}

/// The length of the border between each pair of market areas, in cell sides.
fn borders(st: &State) -> std::collections::HashMap<(usize, usize), usize> {
    let (W, N) = (st.world.W, st.N);
    let mut border = std::collections::HashMap::new();
    for c in 0..N {
        let a = st.catch[c];
        if a < 0 {
            continue;
        }
        for d in [1, W] {
            let k = c + d;
            if k < N && (d == W || k % W != 0) {
                let b = st.catch[k];
                if b >= 0 && b != a {
                    *border.entry((a as usize, b as usize)).or_insert(0usize) += 1;
                }
            }
        }
    }
    border
}

/// Gives town `b` every cell of town `a`'s within `rings` cells of `b`'s area
/// (each ring the cells that touch the last), never `a`'s own seat, on
/// `st.catch`; returns how many cells went.
fn give(st: &mut State, a: usize, b: usize, rings: usize) -> usize {
    let (W, N) = (st.world.W, st.N);
    let mut gave = 0;
    for _ in 0..rings {
        let now = st.catch.clone();
        for c in 0..N {
            if now[c] != a as i64 || c == st.town_node[a] {
                continue;
            }
            let (y, x) = (c / W, c % W);
            let touches = [(0isize, 1isize), (0, -1), (1, 0), (-1, 0)].iter().any(|&(dy, dx)| {
                let (yy, xx) = (y as isize + dy, x as isize + dx);
                yy >= 0 && xx >= 0 && (xx as usize) < W && ((yy as usize) * W + xx as usize) < N && now[yy as usize * W + xx as usize] == b as i64
            });
            if touches {
                st.catch[c] = b as i64;
                gave += 1;
            }
        }
    }
    gave
}

/// An island of the game's in 1760, and two towns that stood before it whose
/// market areas touch: the first gives the second the cells along their border
/// that are not its own seat. Returns the island with `prev_catch` set to the
/// areas before and `catch` to the areas after, and the two towns.
fn a_redraw() -> (State, Content, Params, usize, usize) {
    let (mut st, C, P) = island();
    // the pair of old towns with the longest common border
    let mut best = (0usize, 0usize, 0usize);
    for (&(a, b), &n) in &borders(&st) {
        if old(&st, a) && old(&st, b) && n > best.2 {
            best = (a, b, n);
        }
    }
    let (a, b, n) = best;
    assert!(n > 0, "no two old towns touch");
    // every cell of a's within six cells of b's area goes to b
    assert!(give(&mut st, a, b, 6) > 0);
    (st, C, P, a, b)
}

/// Nothing is made or lost across a redraw: each good's stock and measure of
/// sales to a part in a million million of the island's total, money and each
/// purse to the penny, people to a part in a million million, and no shelf goes
/// below nothing.
fn nothing_made_or_lost(before: &State, after: &State, C: &Content) {
    for g in 0..C.G {
        for (m1, m0, what) in [(&after.inv, &before.inv, "stock"), (&after.outflow, &before.outflow, "sales")] {
            let (now, was) = (sum_col(m1, g), sum_col(m0, g));
            assert!((now - was).abs() <= 1e-12 * was.abs().max(1.0), "{what} of {}: {was} before, {now} after", C.good_name[g]);
            for t in 0..m0.rows {
                assert!(m1[(t, g)] >= 0.0 || m0[(t, g)] < 0.0, "{what} of {} in {} went below nothing", C.good_name[g], before.town_name[t]);
            }
        }
    }
    let (m0, m1) = (econ::money_total(before), econ::money_total(after));
    assert!((m1 - m0).abs() < 0.01, "money {m0} before the redraw, {m1} after");
    let s = |v: &[f64]| v.iter().sum::<f64>();
    for (v1, v0, what) in [(&after.merch_cash, &before.merch_cash, "merchants"), (&after.hh_cash, &before.hh_cash, "households"), (&after.lord_cash, &before.lord_cash, "landlords")] {
        assert!((s(v1) - s(v0)).abs() < 0.01, "the {what}' cash: {} before, {} after", s(v0), s(v1));
    }
    assert!((s(&after.pop) - s(&before.pop)).abs() <= 1e-12 * s(&before.pop), "people: {} before, {} after", s(&before.pop), s(&after.pop));
}

fn sum_col(m: &Mat, g: usize) -> f64 {
    (0..m.rows).map(|t| m[(t, g)]).sum()
}

#[test]
fn the_shelves_go_with_a_district_to_an_old_town_and_nothing_is_lost() {
    let (before, C, P, a, b) = a_redraw();
    let mut on = before.clone();
    let mut off = before.clone();
    let mut P_off = P.clone();
    P_off.people.shelves_go = false;
    peopling::move_with_cells(&mut on, &C, &P);
    peopling::move_with_cells(&mut off, &C, &P_off);

    // the share of a's people that went to b
    let went = 1.0 - on.pop[a] / before.pop[a];
    assert!(went > 0.01 && went <= 0.95 + 1e-12, "{} gave {went:.3} of its people", before.town_name[a]);
    assert!(on.pop[b] > before.pop[b]);
    println!("{} gives {went:.3} of its people to {}", before.town_name[a], before.town_name[b]);

    // with the rule: the same share of every good's stock and of the measure of
    // sales goes, and of the merchants' cash above nothing
    for g in 0..C.G {
        for (m_on, m_was, what) in [(&on.inv, &before.inv, "stock"), (&on.outflow, &before.outflow, "sales")] {
            let moved = went * m_was[(a, g)];
            let scale = m_was[(a, g)].abs() + m_was[(b, g)].abs() + 1e-9;
            assert!((m_on[(a, g)] - (m_was[(a, g)] - moved)).abs() <= 1e-12 * scale, "{what} of {} left in the giver", C.good_name[g]);
            assert!((m_on[(b, g)] - (m_was[(b, g)] + moved)).abs() <= 1e-12 * scale, "{what} of {} in the taker", C.good_name[g]);
            // nothing made or lost, to the unit: the island's total is the same
            let (now, was) = (sum_col(m_on, g), sum_col(m_was, g));
            assert!((now - was).abs() <= 1e-12 * was.abs().max(1.0), "{what} of {}: {was} before, {now} after", C.good_name[g]);
        }
    }
    let cash = went * before.merch_cash[a].max(0.0);
    assert!((on.merch_cash[b] - before.merch_cash[b] - cash).abs() <= 1e-9 * before.merch_cash[b].abs().max(1.0));
    // the stock in months of sales is what the giver's merchants mark their prices by: it does not move
    for g in 0..C.G {
        if before.outflow[(a, g)] > 1e-9 && before.inv[(a, g)] > 1e-9 {
            let (r0, r1) = (before.inv[(a, g)] / before.outflow[(a, g)], on.inv[(a, g)] / on.outflow[(a, g)]);
            assert!((r1 / r0 - 1.0).abs() < 1e-9, "the giver's months of {} went from {r0} to {r1}", C.good_name[g]);
        }
    }

    // money is the same to the penny, and the merchants' and the households' sums each
    for st in [&on, &off] {
        let (m0, m1) = (econ::money_total(&before), econ::money_total(st));
        assert!((m1 - m0).abs() < 0.01 && (m1 - m0).abs() <= 1e-12 * m0, "money {m0} before the redraw, {m1} after");
        let s = |v: &[f64]| v.iter().sum::<f64>();
        assert!((s(&st.merch_cash) - s(&before.merch_cash)).abs() < 0.01);
        assert!((s(&st.hh_cash) - s(&before.hh_cash)).abs() < 0.01);
    }

    // without the rule the old town keeps its shelves: this is what the rule changes
    for g in 0..C.G {
        assert_eq!(off.inv[(a, g)], before.inv[(a, g)]);
        assert_eq!(off.inv[(b, g)], before.inv[(b, g)]);
        assert_eq!(off.outflow[(b, g)], before.outflow[(b, g)]);
    }
    assert_eq!(off.merch_cash[b], before.merch_cash[b]);
    // and with it the taker's grain, in months of its new people's need, is near what the giver's was
    let grain = C.gi("grain");
    let need = |st: &State, t: usize| st.inv[(t, grain)] / st.pop[t];
    println!(
        "grain a head: giver {:.3} before; taker {:.3} before, {:.3} after with the rule, {:.3} without",
        need(&before, a),
        need(&before, b),
        need(&on, b),
        need(&off, b)
    );
}

/// The merchants of a giving town that owe keep what they owe: only cash above
/// nothing goes (the design's rule, 4.2). The critic asked that this be read
/// (10.3); here it is held, so that a change to it is a decision and not an
/// accident.
#[test]
fn a_debt_stays_with_the_merchants_who_owe_it() {
    let (mut before, C, P, a, b) = a_redraw();
    before.merch_cash[a] = -50_000.0;
    let mut on = before.clone();
    peopling::move_with_cells(&mut on, &C, &P);
    assert_eq!(on.merch_cash[a], before.merch_cash[a], "the debt stays");
    assert_eq!(on.merch_cash[b], before.merch_cash[b], "the taker's merchants are given nothing and owe nothing more");
    // and the stock goes all the same
    let grain = C.gi("grain");
    assert!(on.inv[(b, grain)] > before.inv[(b, grain)]);
}

/// A town that gives away every cell but its own seat keeps a twentieth of its
/// people at least (`move_with_cells` never takes more than 0.95 of a town), and
/// keeps the same share of its stock and its measure of sales: nothing more
/// leaves it than the people who go, and nothing is made or lost.
#[test]
fn a_town_that_gives_away_its_last_district() {
    let (mut before, C, P, a, b) = a_redraw();
    for c in 0..before.N {
        if before.catch[c] == a as i64 && c != before.town_node[a] {
            before.catch[c] = b as i64;
        }
    }
    let mut on = before.clone();
    peopling::move_with_cells(&mut on, &C, &P);
    nothing_made_or_lost(&before, &on, &C);
    let kept = on.pop[a] / before.pop[a];
    println!("{} keeps {kept:.3} of its people", before.town_name[a]);
    assert!(kept >= 0.05 - 1e-12 && kept < 0.5, "{} keeps {kept:.3} of its people", before.town_name[a]);
    for g in 0..C.G {
        for (m1, m0, what) in [(&on.inv, &before.inv, "stock"), (&on.outflow, &before.outflow, "sales")] {
            let want = kept * m0[(a, g)];
            assert!((m1[(a, g)] - want).abs() <= 1e-12 * m0[(a, g)].abs().max(1e-9), "{what} of {}: kept {} for {want}", C.good_name[g], m1[(a, g)]);
        }
    }
}

/// A town that takes a district from each of three neighbours in one year gets
/// each one's share of its shelves, and the island loses nothing.
#[test]
fn a_town_that_takes_three_districts_in_one_year() {
    let (mut before, C, P) = island();
    // the old town with the most old neighbours
    let border = borders(&before);
    let T = before.T_max;
    let near = |b: usize| -> Vec<usize> { (0..T).filter(|&a| a != b && old(&before, a) && border.get(&(a, b)).or(border.get(&(b, a))).is_some()).collect() };
    let b = (0..T).filter(|&b| old(&before, b)).max_by_key(|&b| near(b).len()).unwrap();
    let givers: Vec<usize> = near(b).into_iter().take(3).collect();
    assert_eq!(givers.len(), 3, "{} has fewer than three old neighbours", before.town_name[b]);
    for &a in &givers {
        assert!(give(&mut before, a, b, 2) > 0);
    }
    let mut on = before.clone();
    peopling::move_with_cells(&mut on, &C, &P);
    nothing_made_or_lost(&before, &on, &C);
    let went: Vec<f64> = givers.iter().map(|&a| 1.0 - on.pop[a] / before.pop[a]).collect();
    println!("{} takes {went:.3?} of the people of {:?}", before.town_name[b], givers.iter().map(|&a| &before.town_name[a]).collect::<Vec<_>>());
    assert!(went.iter().all(|&w| w > 0.0));
    for g in 0..C.G {
        for (m1, m0, what) in [(&on.inv, &before.inv, "stock"), (&on.outflow, &before.outflow, "sales")] {
            let want = m0[(b, g)] + givers.iter().zip(&went).map(|(&a, &w)| w * m0[(a, g)]).sum::<f64>();
            assert!((m1[(b, g)] - want).abs() <= 1e-12 * want.abs().max(1e-9), "{what} of {} in {}: {} for {want}", C.good_name[g], before.town_name[b], m1[(b, g)]);
        }
    }
}

/// A district that goes to another town and comes back the next year: nothing
/// is made or lost on the way there or on the way back.
#[test]
fn a_district_that_goes_and_comes_back() {
    let (before, C, P, a, b) = a_redraw();
    let mut st = before.clone();
    peopling::move_with_cells(&mut st, &C, &P);
    let there = st.clone();
    let grain = C.gi("grain");
    assert!(there.inv[(b, grain)] > before.inv[(b, grain)], "the shelves went with the district");
    st.prev_catch = st.catch.clone();
    st.catch = before.prev_catch.clone();
    peopling::move_with_cells(&mut st, &C, &P);
    nothing_made_or_lost(&before, &there, &C);
    nothing_made_or_lost(&there, &st, &C);
    nothing_made_or_lost(&before, &st, &C);
    assert!(st.inv[(a, grain)] > there.inv[(a, grain)], "the shelves came back with the district");
    println!(
        "{}'s grain {:.0}, {:.0} while the district was away, {:.0} when it came back",
        before.town_name[a],
        before.inv[(a, grain)],
        there.inv[(a, grain)],
        st.inv[(a, grain)]
    );
}

/// The January the rule was made for. On island 4 under the game's rules
/// Penfell takes a district at the end of 1769 and has 1.9 times its people in
/// January 1770, with the grain its old people needed: without the rule its
/// households get 0.78 of their food in February (`examples/spring_hunger.rs`;
/// the case was island 14's Wyrside in the same winter, docs/probes/tr-rent.md,
/// section 4, until towns stopped hiring hands they do not have,
/// docs/rules/hands.md). With it, no town that took a district goes short in the
/// first three months after. The run is played twice, the second time without
/// the rule, to show that the case is still there to mend. (Money issued on its
/// true total, docs/rules/money.md, gives the island another history, in which
/// Penfell's winter does not come, so both runs are played with that rule off.)
#[test]
fn no_hungry_january_after_a_town_takes_a_district() {
    let run = |shelves: bool| {
        let mut P = game_params();
        P.seed = 4;
        P.n_families = 6;
        P.end_year = Some(1771);
        P.money.true_total = false;
        P.people.shelves_go = shelves;
        let mut s = sim::start(&P, game_content());
        let T = s.st.T_max;
        // (year, town, the worst of January to March) for every town that went short after taking a district
        let mut short = Vec::new();
        let mut took = vec![false; T];
        while s.year <= s.end_year {
            let year = s.year;
            let mut worst = vec![9.0f64; T];
            for month in 0..12 {
                sim::run_month(&mut s, month).expect("money is conserved");
                if month < 3 {
                    for t in 0..T {
                        if s.st.town_active[t] {
                            worst[t] = worst[t].min(s.st.food_ratio[t]);
                        }
                    }
                }
            }
            for t in 0..T {
                if took[t] && worst[t] < 0.82 {
                    short.push((year, s.st.town_name[t].clone(), worst[t]));
                }
            }
            let dec = s.st.pop.clone();
            let stood: Vec<bool> = (0..T).map(|t| s.st.town_active[t] && s.st.town_founded[t] < year).collect();
            sim::end_year(&mut s);
            // a town that stood before and has half as many people again took a district
            took = (0..T).map(|t| stood[t] && dec[t] > 0.0 && s.st.pop[t] >= 1.5 * dec[t]).collect();
        }
        short
    };
    let (with, without) = std::thread::scope(|sc| {
        let h = sc.spawn(|| run(false));
        (run(true), h.join().unwrap())
    });
    assert!(!without.is_empty(), "the case is gone: no town went short after taking a district on island 14 without the rule");
    println!("without the rule: {without:?}");
    assert!(with.is_empty(), "short after taking a district, with the rule: {with:?}");
}
