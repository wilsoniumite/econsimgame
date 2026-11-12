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

/// An island of the game's in 1760, and two towns that stood before it whose
/// market areas touch: the first gives the second the cells along their border
/// that are not its own seat. Returns the island with `prev_catch` set to the
/// areas before and `catch` to the areas after, and the two towns.
fn a_redraw() -> (State, Content, Params, usize, usize) {
    let mut P = game_params();
    P.seed = 3;
    P.n_families = 6;
    P.end_year = Some(1760);
    let r = sim::run_with(&P, game_content(), false, true).expect("the run finishes");
    let mut st = r.state;
    let C = r.content;
    let (W, N) = (st.world.W, st.N);
    // the pair of old towns with the longest common border
    let mut best = (0usize, 0usize, 0usize);
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
    for (&(a, b), &n) in &border {
        let old = |t: usize| st.town_active[t] && st.town_founded[t] < 1760 && st.pop[t] > 1000.0;
        if old(a) && old(b) && n > best.2 {
            best = (a, b, n);
        }
    }
    let (a, b, n) = best;
    assert!(n > 0, "no two old towns touch");
    st.prev_catch = st.catch.clone();
    // every cell of a's within six cells of b's area goes to b (six rings, each
    // the cells that touch the last), but never a's own seat
    let mut gave = 0;
    for _ in 0..6 {
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
    assert!(gave > 0);
    assert!(st.town_new.iter().all(|&v| !v), "neither town is new");
    (st, C, P, a, b)
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

/// The January the rule was made for. On island 14 under the game's rules
/// Wyrside takes a district at the end of 1769 and has two and a half times its
/// people in January 1770, with the grain its old people needed: without the
/// rule its households get 0.79 of their food in February (docs/probes/tr-rent.md,
/// section 4; `examples/spring_hunger.rs`). With it, no town that took a district
/// goes short in the first three months after. The run is played twice, the
/// second time without the rule, to show that the case is still there to mend.
#[test]
fn no_hungry_january_after_a_town_takes_a_district() {
    let run = |shelves: bool| {
        let mut P = game_params();
        P.seed = 14;
        P.n_families = 6;
        P.end_year = Some(1771);
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
