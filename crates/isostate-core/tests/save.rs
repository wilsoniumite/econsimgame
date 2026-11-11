//! Saved games (not in the Python): a game played for thirty years with orders
//! of every kind, saved, and loaded comes back the same to the last digit, and
//! goes on the same; a save the rules no longer agree with loads anyway, and
//! says what it could not repeat.
#![allow(non_snake_case)]

use isostate_core::families::{self, Post};
use isostate_core::save::{self, Game, Order, Save};
use isostate_core::*;

fn report(g: &Game, k: usize) -> FamilyReport {
    family_report(&g.sim.st, &g.sim.content, &g.sim.params, k).unwrap()
}

/// Everything the families' reports say, as text: two runs print the same
/// only if every number in them is the same (serde_json writes the shortest
/// digits that read back to the same float).
fn reports(g: &Game) -> String {
    (0..g.sim.st.families.len()).map(|k| serde_json::to_string(&report(g, k)).unwrap()).collect::<Vec<_>>().join("\n")
}

fn series_bits(g: &Game) -> Vec<Vec<u64>> {
    g.sim.st.series.data.iter().map(|d| d.iter().map(|x| x.to_bits()).collect()).collect()
}

/// A scripted player, given the year just played: answers what is waiting
/// (not always with the default), builds, buys land, posts people, changes
/// what a branch does and moves the seat. Orders are only ever given between
/// years, as in the game. Returns how many orders the simulation refused.
fn orders_for(g: &mut Game) -> usize {
    let year = g.last_year();
    let mut refused = 0;
    let mut give = |g: &mut Game, o: Order| {
        if g.give(o).is_err() {
            refused += 1;
        }
    };
    // decisions: the default in even years, the last option in odd ones
    for d in pending(&g.sim.st, 0) {
        let option = if year % 2 == 0 { d.default } else { d.options.len() - 1 };
        let o = Order::decide(&g.sim, d.id, option).unwrap();
        give(g, o);
    }
    let r = report(g, 0);
    let seat = r.seat;
    let near: Vec<usize> = (0..g.sim.st.N).filter(|&c| g.sim.st.catch[c] == seat as i64 && g.sim.st.world.land[c]).collect();
    // every third year: the best-paying works the family may build at the seat for half its cash
    if year % 3 == 0 {
        let mut best: Option<(f64, usize, usize)> = None;
        for &c in near.iter().take(60) {
            let Ok(opts) = site_options(&g.sim.st, &g.sim.content, &g.sim.params, 0, c) else { continue };
            for o in opts {
                if let (Ok(s), Ok(())) = (o.sums, o.allowed) {
                    if s.capital_cost <= 0.5 * r.cash && best.is_none_or(|b| s.expected_return > b.0) {
                        best = Some((s.expected_return, o.method, c));
                    }
                }
            }
        }
        if let Some((_, m, c)) = best {
            let o = Order::found(&g.sim, m, c).unwrap();
            give(g, o);
        }
    }
    // every other year: the cheapest square around the seat that is not the family's
    if year % 2 == 1 {
        let cheapest = near
            .iter()
            .filter(|&&c| g.sim.st.cell_owner[c] != 0)
            .map(|&c| (families::land_price(&g.sim.st, &g.sim.params, c), c))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, c)) = cheapest {
            give(g, Order::BuyLand { cell: c });
        }
    }
    // every fifth year: an idle adult goes to run the first known trade in another town
    if year % 5 == 2 {
        let r = report(g, 0);
        let other = (0..g.sim.st.T_max).find(|&t| g.sim.st.town_active[t] && t != seat);
        let idle = r.members.iter().find(|m| m.alive && m.age >= 18 && m.post == Post::Unposted);
        if let (Some(town), Some(m), Some(&group)) = (other, idle, r.known.first()) {
            give(g, Order::Post { member: m.index, posting: Posting::Manager { group, town } });
        }
    }
    // every fourth year: the first branch holds, or grows again
    if year % 4 == 3 {
        let r = report(g, 0);
        if let Some(b) = r.branches.first() {
            let stance = if b.stance == Stance::Grow { Stance::Hold } else { Stance::Grow };
            give(g, Order::Stance { group: b.group, town: b.town, stance });
        }
    }
    // once: the seat moves to another town
    if year == 1768 {
        if let Some(town) = (0..g.sim.st.T_max).find(|&t| g.sim.st.town_active[t] && t != seat) {
            give(g, Order::MoveSeat { town });
        }
    }
    // something the simulation refuses: it must leave no trace
    let sea = (0..g.sim.st.N).find(|&c| !g.sim.st.world.land[c]).unwrap();
    let before = save::check(&g.sim);
    assert!(g.give(Order::BuyLand { cell: sea }).is_err());
    assert_eq!(save::check(&g.sim), before, "a refused order changes nothing");
    refused
}

fn play(g: &mut Game, years: usize) {
    for _ in 0..years {
        g.year().unwrap();
        orders_for(g);
    }
}

#[test]
fn thirty_years_saved_and_loaded_are_the_same_to_the_last_digit() {
    let mut a = Game::new(7, 6, 1_000);
    // the first trade is chosen before the first year, as a script (not the viewer) can
    let first = pending(&a.sim.st, 0);
    assert_eq!(first.len(), 1);
    let o = Order::decide(&a.sim, first[0].id, first[0].options.len() - 1).unwrap();
    a.give(o).unwrap();
    play(&mut a, 30);
    assert_eq!(a.last_year(), 1779);

    // the script gave orders of every kind, and none of them is the refused one
    let kinds = |name: &str| a.orders.iter().filter(|g| serde_json::to_value(&g.order).unwrap()["kind"] == name).count();
    for kind in ["found", "buy_land", "decide", "post", "stance", "move_seat"] {
        assert!(kinds(kind) > 0, "the script gave no {kind} order: {:?}", a.orders);
    }
    eprintln!(
        "{} orders in thirty years: {} found, {} buy_land, {} decide, {} post, {} stance, {} move_seat",
        a.orders.len(),
        kinds("found"),
        kinds("buy_land"),
        kinds("decide"),
        kinds("post"),
        kinds("stance"),
        kinds("move_seat")
    );
    assert!(a.orders.windows(2).all(|w| w[0].year <= w[1].year), "the log is in the order it was given");
    assert_eq!(a.orders[0].year, 1749, "an order before the first year is dated the year before it");

    // the file: readable, a line an order, and it reads back to the same save
    let saved = a.save(1_794_366_765);
    let text = saved.to_text();
    assert_eq!(text.lines().count(), 13 + 2 + saved.orders.len() + 1, "{text}");
    assert!(text.contains("\"format\": 1,") && text.contains("\"seed\": 7,") && text.contains("\"year\": 1779,"));
    assert!(text.contains("\"saved\": \"2026-11-11 03:12 UTC\""), "{text}");
    let read = Save::from_text(&text).unwrap();
    assert_eq!(read, saved);
    assert!(read.same_rules());

    // loading plays it again
    let mut years = Vec::new();
    let b = save::load(&read, |sim| years.push(sim.year - 1)).unwrap();
    assert_eq!(years, (1750..=1779).collect::<Vec<i64>>(), "every year is handed over once, in order");
    assert!(b.refused.is_empty(), "{:?}", b.refused);
    assert!(!b.drifted);
    let mut b = b.game;
    assert_eq!(b.orders, a.orders);
    assert_eq!(b.started, 1_000);
    assert_eq!(b.sim.year, a.sim.year);
    assert_eq!(reports(&b), reports(&a), "every family's report is the same");
    assert_eq!(series_bits(&b), series_bits(&a), "every month of every series is the same");
    assert_eq!(b.sim.st.cell_owner, a.sim.st.cell_owner);
    assert_eq!(b.sim.st.f_owner, a.sim.st.f_owner);
    assert_eq!(b.sim.events.len(), a.sim.events.len());
    assert_eq!(b.save(1_794_366_765), saved, "saving the loaded game writes the same file");

    // and both go on the same: the random streams and everything unseen came back too
    play(&mut a, 3);
    play(&mut b, 3);
    assert_eq!(b.orders, a.orders);
    assert_eq!(reports(&b), reports(&a));
    assert_eq!(series_bits(&b), series_bits(&a));
    assert_eq!(save::check(&b.sim), save::check(&a.sim));
}

#[test]
fn a_save_the_rules_no_longer_agree_with_says_so() {
    let mut a = Game::new(7, 3, 5);
    play(&mut a, 6);
    let saved = a.save(0);
    let bought = saved.orders.iter().position(|g| matches!(g.order, Order::BuyLand { .. })).expect("the script bought land");

    // orders the simulation refuses now are skipped and counted, and the rest is played
    let mut odd = saved.clone();
    odd.orders.insert(0, save::Given { year: 1750, order: Order::Found { method: "steam_loom_of_the_future".into(), cell: 10 } });
    odd.orders.insert(1, save::Given { year: 1750, order: Order::Decide { id: 999, choice: Choice::University } });
    odd.orders.push(save::Given { year: 1900, order: Order::MoveSeat { town: 0 } });
    let b = save::load(&odd, |_| {}).unwrap();
    assert_eq!(b.refused.len(), 3, "{:?}", b.refused);
    assert!(b.refused[0].1.contains("no method called"), "{}", b.refused[0].1);
    assert!(b.refused[1].1.contains("no decision 999"), "{}", b.refused[1].1);
    assert!(b.refused[2].1.contains("after the save's last year"), "{}", b.refused[2].1);
    assert!(!b.drifted, "refused orders change nothing, so the island stands as it was saved");
    assert_eq!(b.game.orders, saved.orders, "the log to go on with holds what was carried out");
    assert_eq!(reports(&b.game), reports(&a));

    // an order that is missing (as if the rules had refused it) leaves the island standing differently
    let mut less = saved.clone();
    less.orders.remove(bought);
    let c = save::load(&less, |_| {}).unwrap();
    assert!(c.drifted, "the island is not as the save says it stood");
    assert_eq!(c.game.last_year(), a.last_year(), "it is still played to the year it was saved in");

    // other rules are noticed before anything is played
    let mut old = saved.clone();
    old.rules.fingerprint = "0000000000000000".into();
    assert!(!old.same_rules());

    // a later layout is refused by name; rubbish is not a save
    let later = saved.to_text().replace("\"format\": 1,", "\"format\": 2,");
    assert!(Save::from_text(&later).unwrap_err().contains("later version"));
    assert!(Save::from_text("{\"seed\": 7}").unwrap_err().contains("not a saved game"));
    assert!(Save::from_text("").unwrap_err().contains("not a saved game"));
}

#[test]
fn dates_are_written_in_utc() {
    assert_eq!(save::utc(0), "1970-01-01 00:00 UTC");
    assert_eq!(save::utc(951_782_400), "2000-02-29 00:00 UTC");
    assert_eq!(save::utc(1_794_366_765), "2026-11-11 03:12 UTC");
}
