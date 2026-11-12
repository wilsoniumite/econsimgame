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
    assert_eq!(text.lines().count(), 15 + 2 + saved.orders.len() + 1, "{text}");
    assert!(text.contains("\"format\": 2,") && text.contains("\"seed\": 7,") && text.contains("\"year\": 1779,"));
    assert!(text.contains("\"month\": 0,") && text.contains("\"waiting\": []"), "{text}");
    assert!(text.contains("\"saved\": \"2026-11-11 03:12 UTC\""), "{text}");
    let read = Save::from_text(&text).unwrap();
    assert_eq!(read, saved);
    assert!(read.same_rules());

    // a file in the first layout, which had no months and nothing waiting, reads as the same game
    let first_layout: String = text.replace("\"format\": 2,", "\"format\": 1,").lines().filter(|l| !l.contains("\"month\": 0,") && !l.contains("\"waiting\": []")).collect::<Vec<_>>().join("\n");
    // (the orders were not the last field then, and now end the file)
    let first_layout = first_layout.replace("],\n}", "]\n}");
    assert!(!first_layout.contains("month") && !first_layout.contains("waiting"), "{first_layout}");
    let old = Save::from_text(&first_layout).unwrap();
    assert_eq!((old.format, old.month, old.waiting.len()), (1, 0, 0));
    assert_eq!(old.orders, saved.orders);
    let again = save::load(&old, |_| {}).unwrap();
    assert!(again.refused.is_empty() && !again.drifted, "a save in the first layout plays again as it was");
    assert_eq!(save::check(&again.game.sim), save::check(&a.sim));

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
    odd.orders.insert(0, save::Given { year: 1750, month: 0, order: Order::Found { method: "steam_loom_of_the_future".into(), cell: 10 } });
    odd.orders.insert(1, save::Given { year: 1750, month: 0, order: Order::Decide { id: 999, choice: Choice::University } });
    odd.orders.push(save::Given { year: 1900, month: 0, order: Order::MoveSeat { town: 0 } });
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

    // a game is played under the game's rules and its save says so; the
    // fingerprint is of the rules' numbers, and the same each time it is taken
    assert_eq!(saved.rules.name, "game");
    assert_eq!(saved.rules, save::rules_id(Rules::Game));
    assert_eq!(save::rules_id(Rules::Game), save::rules_id(Rules::Game));
    assert_eq!(saved.played_under(), Rules::Game);
    // a save that names the port is played again under the port
    let mut port = Game::under(Rules::Port, 7, 3, 5);
    play(&mut port, 2);
    let port_saved = port.save(0);
    assert_eq!(port_saved.rules.name, "port");
    let d = save::load(&port_saved, |_| {}).unwrap();
    assert_eq!((d.game.rules, d.drifted, d.refused.len()), (Rules::Port, false, 0));
    // rules this version has never heard of are taken for the game's, and are not today's
    let mut odd_rules = saved.clone();
    odd_rules.rules.name = "house".into();
    assert_eq!(odd_rules.played_under(), Rules::Game);
    assert!(!odd_rules.same_rules());

    // a later layout is refused by name; rubbish is not a save
    let later = saved.to_text().replace("\"format\": 2,", "\"format\": 3,");
    assert!(Save::from_text(&later).unwrap_err().contains("later version"));
    assert!(Save::from_text("{\"seed\": 7}").unwrap_err().contains("not a saved game"));
    assert!(Save::from_text("").unwrap_err().contains("not a saved game"));
}

fn months(g: &mut Game, n: usize) {
    for _ in 0..n {
        assert!(!g.month().unwrap(), "the year turned sooner than the test meant");
    }
}

/// Play on to the year's turn, and hand over what became of the orders that waited.
fn to_turn(g: &mut Game) -> Vec<save::Outcome> {
    while !g.month().unwrap() {}
    g.turn()
}

/// The cheapest square by the seat that the player's family does not hold.
fn cheapest_square(g: &Game) -> usize {
    let seat = report(g, 0).seat;
    (0..g.sim.st.N)
        .filter(|&c| g.sim.st.catch[c] == seat as i64 && g.sim.st.world.land[c] && g.sim.st.cell_owner[c] != 0)
        .map(|c| (families::land_price(&g.sim.st, &g.sim.params, c), c))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap()
        .1
}

#[test]
fn a_month_at_a_time_is_the_same_game_as_a_year_at_a_time() {
    let mut a = Game::new(7, 6, 0);
    let mut b = Game::new(7, 6, 0);
    for _ in 0..4 {
        a.year().unwrap();
        orders_for(&mut a);
        for m in 0..12 {
            assert_eq!(b.month, m);
            assert_eq!(b.month().unwrap(), m == 11, "the year turns after December and not before");
        }
        assert!(b.turn().is_empty(), "nothing waited");
        assert!(b.at_turn());
        orders_for(&mut b);
    }
    assert_eq!(b.orders, a.orders);
    assert_eq!(save::check(&b.sim), save::check(&a.sim));
    assert_eq!(series_bits(&b), series_bits(&a));
}

#[test]
fn orders_given_in_the_middle_of_a_year_wait_for_its_end() {
    let mut a = Game::new(7, 6, 77);
    play(&mut a, 3);
    let at_turn = a.clone();
    months(&mut a, 4);
    assert!(!a.at_turn());
    let untouched = save::check(&a.sim);
    let me = report(&a, 0);
    let square = cheapest_square(&a);
    let other = (0..a.sim.st.T_max).find(|&t| a.sim.st.town_active[t] && t != me.seat).unwrap();
    let log = a.orders.len();

    // given in May: nothing happens yet, and the family is shown as it will stand
    assert_eq!(a.give(Order::BuyLand { cell: square }), Ok(save::Done::Waits));
    assert_eq!(a.give(Order::MoveSeat { town: other }), Ok(save::Done::Waits));
    let sea = (0..a.sim.st.N).find(|&c| !a.sim.st.world.land[c]).unwrap();
    assert!(a.give(Order::BuyLand { cell: sea }).is_err(), "an order that could not be carried out today is refused today");
    assert!(a.give(Order::BuyLand { cell: square }).is_err(), "and so is one that what already waits has made impossible");
    assert_eq!(a.waiting, [Order::BuyLand { cell: square }, Order::MoveSeat { town: other }]);
    assert_eq!((a.orders.len(), save::check(&a.sim)), (log, untouched.clone()), "an order that waits changes nothing and is not in the log");
    let seen = a.foreseen().expect("the family as it will stand");
    assert_eq!(seen.next_seat, Some(other));
    assert!(seen.cells.contains(&square) && !report(&a, 0).cells.contains(&square));
    assert!(seen.cash < report(&a, 0).cash, "the land's price is counted as spent");

    // an order taken back is gone
    assert_eq!(a.give(Order::MoveSeat { town: me.seat }), Ok(save::Done::Waits));
    assert_eq!(a.cancel(2), Ok(Order::MoveSeat { town: me.seat }));
    assert!(a.cancel(2).is_err());
    assert_eq!(a.foreseen().unwrap().next_seat, Some(other));

    // saved in May and loaded: the same month, the same orders waiting, the same island
    months(&mut a, 2);
    let saved = a.save(0);
    assert_eq!((saved.year, saved.month, saved.waiting.len()), (1752, 6, 2));
    let text = saved.to_text();
    assert!(text.contains("\"month\": 6,") && text.contains("\"waiting\": [{"), "{text}");
    let read = Save::from_text(&text).unwrap();
    assert_eq!(read, saved);
    let mut years = Vec::new();
    let b = save::load(&read, |sim| years.push(sim.year - 1)).unwrap();
    assert_eq!(years, [1750, 1751, 1752], "the year in progress is not handed over as a year");
    assert!(b.refused.is_empty() && !b.drifted, "{:?}", b.refused);
    let mut b = b.game;
    assert_eq!((b.month, &b.waiting, b.last_year()), (6, &a.waiting, 1752));
    assert_eq!(save::check(&b.sim), save::check(&a.sim));
    assert_eq!(serde_json::to_string(&b.foreseen()).unwrap(), serde_json::to_string(&a.foreseen()).unwrap());
    assert_eq!(b.save(0), saved, "saving the loaded game writes the same file");

    // the year's end: the move is carried out before the books are closed and
    // happens with them. The land comes up at the turn, after, and by then the
    // seat has gone from beside it: an order that could be carried out when it
    // was given can still be refused when its time comes, and the family is
    // told why
    for g in [&mut a, &mut b] {
        while !g.month().unwrap() {}
        assert_eq!(report(g, 0).seat, other, "the seat moved with the year's end");
        let done = g.turn();
        assert_eq!(done.len(), 2);
        assert_eq!((&done[0].order, &done[0].result), (&Order::MoveSeat { town: other }, &Ok(save::Done::Done)));
        assert_eq!(done[1].order, Order::BuyLand { cell: square });
        assert!(done[1].result.as_ref().is_err_and(|why| why.contains("only around the seat")), "{done:?}");
        assert!(!report(g, 0).cells.contains(&square));
        assert!(g.waiting.is_empty() && g.foreseen().is_none() && g.turn().is_empty());
        assert_eq!(g.orders.len(), log + 1, "the order that was refused is not in the log");
        assert_eq!(g.orders[log], save::Given { year: 1752, month: 12, order: Order::MoveSeat { town: other } });
    }
    assert_eq!(save::check(&b.sim), save::check(&a.sim));

    // the log plays again to the same island, the order of December in its place
    let text = a.save(0).to_text();
    assert!(text.contains("{\"year\":1752,\"month\":12,\"kind\":\"move_seat\""), "{text}");
    let c = save::load(&Save::from_text(&text).unwrap(), |_| {}).unwrap();
    assert!(c.refused.is_empty() && !c.drifted);
    assert_eq!(c.game.orders, a.orders);
    play(&mut a, 2);
    play(&mut b, 2);
    let mut c = c.game;
    play(&mut c, 2);
    assert_eq!(save::check(&b.sim), save::check(&a.sim));
    assert_eq!(save::check(&c.sim), save::check(&a.sim));

    // land ordered in May is bought exactly where land ordered at the following turn is:
    // the two games are the same game
    let mut waited = at_turn.clone();
    months(&mut waited, 4);
    let square = cheapest_square(&waited);
    assert_eq!(waited.give(Order::BuyLand { cell: square }), Ok(save::Done::Waits));
    assert!(matches!(to_turn(&mut waited)[0].result, Ok(save::Done::Bought(_))));
    assert_eq!(waited.orders.last(), Some(&save::Given { year: 1753, month: 0, order: Order::BuyLand { cell: square } }));
    let mut prompt = at_turn.clone();
    prompt.year().unwrap();
    assert!(matches!(prompt.give(Order::BuyLand { cell: square }), Ok(save::Done::Bought(_))));
    assert_eq!(waited.orders, prompt.orders, "both are in the log as carried out at the turn");
    waited.year().unwrap();
    prompt.year().unwrap();
    assert_eq!(save::check(&waited.sim), save::check(&prompt.sim));

    // and a seat moved in May moves when a seat moved at the turn before would have
    let mut late = at_turn.clone();
    months(&mut late, 4);
    late.give(Order::MoveSeat { town: other }).unwrap();
    to_turn(&mut late);
    let mut early = at_turn.clone();
    early.give(Order::MoveSeat { town: other }).unwrap();
    early.year().unwrap();
    assert_eq!(reports(&late), reports(&early));
    assert_eq!(save::check(&late.sim), save::check(&early.sim));
}

#[test]
fn a_save_made_at_a_turn_before_its_orders_are_carried_out_keeps_them() {
    let mut a = Game::new(7, 6, 0);
    play(&mut a, 3);
    months(&mut a, 4);
    let square = cheapest_square(&a);
    assert_eq!(a.give(Order::BuyLand { cell: square }), Ok(save::Done::Waits));
    // December is played and the books are closed, but `turn` has not been called: the land still waits
    while !a.month().unwrap() {}
    let saved = a.save(0);
    assert_eq!((saved.year, saved.month, saved.waiting.len()), (1753, 0, 1));
    let b = save::load(&Save::from_text(&saved.to_text()).unwrap(), |_| {}).unwrap();
    assert!(b.refused.is_empty() && !b.drifted, "{:?}", b.refused);
    let mut b = b.game;
    assert_eq!(b.waiting, a.waiting);
    // the turn carries it out in both, and they are the same game after
    for g in [&mut a, &mut b] {
        let done = g.turn();
        assert!(matches!(done[0].result, Ok(save::Done::Bought(_))), "{done:?}");
        g.year().unwrap();
    }
    assert_eq!(b.orders, a.orders);
    assert_eq!(save::check(&b.sim), save::check(&a.sim));
}

#[test]
fn a_decision_answered_late_in_the_year_takes_the_place_of_its_default() {
    // the family's first trade waits from the start; it is left unanswered into the spring
    let mut g = Game::new(7, 6, 0);
    g.year().unwrap();
    let d = pending(&g.sim.st, 0).into_iter().next().expect("the first trade is still to choose");
    let other = (0..d.options.len()).find(|&i| i != d.default).unwrap();
    let Choice::Trade { group } = d.options[other].choice else { panic!("the first decision is the family's trade") };
    assert!(!report(&g, 0).known.contains(&group));
    months(&mut g, 3);
    let o = Order::decide(&g.sim, d.id, other).unwrap();
    assert_eq!(g.give(o.clone()), Ok(save::Done::Waits));
    assert!(g.foreseen().unwrap().pending.iter().all(|p| p.id != d.id), "the family is shown as having decided");
    assert!(pending(&g.sim.st, 0).iter().any(|p| p.id == d.id), "though nothing has changed yet");
    let done = to_turn(&mut g);
    assert_eq!(done, [save::Outcome { order: o, result: Ok(save::Done::Done) }]);
    assert!(report(&g, 0).known.contains(&group), "the choice was made at the year's end, where the default would have been");
    assert_eq!(g.orders.last().unwrap().month, 12);
}

#[test]
fn a_file_that_says_what_no_game_could_have_written_is_refused() {
    let mut g = Game::new(7, 3, 5);
    play(&mut g, 2);
    let good = g.save(0);
    assert_eq!(good.sound(), Ok(()));
    // a game saved before its first year says the year before it, and one that is over its last
    let unplayed = Game::new(7, 3, 5).save(0);
    assert_eq!((unplayed.year, unplayed.sound()), (1749, Ok(())));
    assert_eq!(Save { year: 1900, ..good.clone() }.sound(), Ok(()));

    // refused as text, before a year is played, and refused by `load` when it never was text.
    // Before the island is made, too: no refusal, the quickest of three tries, takes a quarter of
    // what making the island takes, measured here in the same run (so that a busy machine slows
    // both alike; it used to be held to five seconds, which a busy machine could run out)
    let making = {
        let t0 = std::time::Instant::now();
        std::hint::black_box(Game::new(7, 3, 5));
        t0.elapsed().as_secs_f64()
    };
    let refused = |s: Save| {
        let as_text = Save::from_text(&s.to_text()).unwrap_err();
        let mut quickest = f64::INFINITY;
        let mut loaded = String::new();
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            loaded = save::load(&s, |_| panic!("a year of a save that is refused was played")).unwrap_err();
            quickest = quickest.min(t0.elapsed().as_secs_f64());
        }
        assert!(quickest < making / 4.0, "the refusal came after the island was made: it took {:.1} ms, and making the island {:.1} ms: {loaded}", quickest * 1e3, making * 1e3);
        assert_eq!(as_text, loaded);
        assert!(as_text.starts_with("not a saved game the game wrote: "), "{as_text}");
        as_text
    };
    // the year 5000 was once taken at its word and announced as 3,251 years to play
    assert!(refused(Save { year: 5000, ..good.clone() }).ends_with("it says the game had reached the year 5000, and the island's years are 1750 to 1900"));
    assert!(refused(Save { year: 1901, ..good.clone() }).contains("the year 1901"));
    assert!(refused(Save { year: 1748, ..good.clone() }).contains("the year 1748"));
    assert!(refused(Save { year: -3, ..good.clone() }).contains("the year -3"));
    // families: the player's at least, and no more than have names
    assert_eq!(save::MAX_FAMILIES, 30);
    assert!(refused(Save { families: 0, ..good.clone() }).ends_with("it names 0 families, and a game has from 1 to 30 families, not 0"));
    assert!(refused(Save { families: 31, ..good.clone() }).contains("it names 31 families"));
    assert!(refused(Save { families: 4_000_000_000, ..good.clone() }).contains("it names 4000000000 families"));
    assert_eq!(Save { families: 30, ..good.clone() }.sound(), Ok(()));
    assert_eq!(save::families_allowed(1), Ok(()));
    assert_eq!(save::families_allowed(0).unwrap_err(), "a game has from 1 to 30 families, not 0");
    // months: a save is made with none to eleven of the year played
    assert_eq!(Save { month: 11, ..good.clone() }.sound(), Ok(()));
    assert!(refused(Save { month: 12, ..good.clone() }).contains("it says 12 months of a year had been played"));
    assert!(refused(Save { format: 0, ..good.clone() }).contains("layout 0"));
    // an order is dated twelve months into a year at the most
    let mut late = good.clone();
    late.orders.push(save::Given { year: 1751, month: 13, order: Order::MoveSeat { town: 0 } });
    assert!(refused(late).contains("an order dated 13 months into 1752"));

    // what an order names is the simulation's to refuse, one order at a time: the rest is played
    let mut wild = good.clone();
    wild.orders.push(save::Given { year: 1751, month: 0, order: Order::BuyLand { cell: usize::MAX } });
    wild.orders.push(save::Given { year: 1751, month: 0, order: Order::MoveSeat { town: usize::MAX } });
    wild.orders.push(save::Given { year: 1751, month: 0, order: Order::Post { member: usize::MAX, posting: Posting::Unposted } });
    wild.orders.push(save::Given { year: 1751, month: 0, order: Order::Found { method: g.sim.content.method_name[0].clone(), cell: usize::MAX } });
    let back = save::load(&Save::from_text(&wild.to_text()).unwrap(), |_| {}).unwrap();
    assert_eq!(back.refused.len(), 4, "{:?}", back.refused);
    assert_eq!(back.game.orders, good.orders);
    assert!(!back.drifted);
}

#[test]
fn orders_that_name_anything_at_all_are_refused_or_carried_out_and_the_game_goes_on() {
    use isostate_core::families::{Choice, Group, Stance};
    let mut g = Game::new(7, 3, 5);
    play(&mut g, 3);
    let good = g.save(0);
    // every number an order can hold, at its ends, just inside them and just outside: squares, towns,
    // members of the family, decisions
    let numbers = [0usize, 1, 2, 3, 7, 63, 64, 65, 255, 9_999, 10_000, 10_001, 1 << 20, 1 << 32, usize::MAX / 2, usize::MAX - 1, usize::MAX];
    let methods: Vec<String> = g.sim.content.method_name.iter().take(4).cloned().chain(["a works nobody knows".to_string(), String::new()]).collect();
    let mut wild: Vec<Order> = Vec::new();
    for (i, &x) in numbers.iter().enumerate() {
        let y = numbers[(i * 7 + 3) % numbers.len()];
        let group = Group::ALL[i % Group::ALL.len()];
        let stance = [Stance::Grow, Stance::Hold, Stance::WindDown][i % 3];
        wild.push(Order::BuyLand { cell: x });
        wild.push(Order::MoveSeat { town: x });
        wild.push(Order::Found { method: methods[i % methods.len()].clone(), cell: x });
        wild.push(Order::Found { method: methods[0].clone(), cell: x });
        wild.push(Order::Stance { group, town: x, stance });
        wild.push(Order::Post { member: x, posting: Posting::Manager { group, town: y } });
        wild.push(Order::Post { member: y, posting: Posting::Unposted });
        wild.push(Order::Decide { id: x as u64, choice: Choice::Apprentice { group, town: y } });
        wild.push(Order::Decide { id: y as u64, choice: Choice::Head { member: x } });
        wild.push(Order::Decide { id: x as u64, choice: Choice::Trade { group } });
        wild.push(Order::Decide { id: x as u64, choice: Choice::University });
    }
    // dated at every turn of the game, at each year's end (twelve months in) and, as no game dates
    // an order, in the middle of a year; in any order; and left waiting
    let mut doctored = good.clone();
    doctored.orders = wild.iter().enumerate().map(|(i, o)| save::Given { year: 1749 + (i as i64 * 5) % 4, month: [0, 12, 5, 0, 11, 1][i % 6], order: o.clone() }).collect();
    doctored.orders.push(save::Given { year: i64::MIN, month: 0, order: Order::MoveSeat { town: 1 } });
    doctored.orders.push(save::Given { year: i64::MAX, month: 12, order: Order::MoveSeat { town: 1 } });
    doctored.month = 7;
    doctored.waiting = wild.clone();
    doctored.seed = u64::MAX;
    doctored.started = u64::MAX;
    doctored.rules.name = "rules nobody knows".to_string();
    let text = doctored.to_text();
    // the file is read and played: what cannot be done is refused, an order at a time, and nothing stops the program
    let played = std::panic::catch_unwind(|| {
        let mut back = save::load(&Save::from_text(&text).expect("it is a saved game by its layout"), |_| {}).expect("it loads");
        let refused = back.refused.len();
        let carried = back.game.orders.len();
        // and it goes on: the orders that wait are tried at the year's end and its turn, and two more years are played
        for _ in 0..2 {
            back.game.year().expect("the year is played");
            back.game.turn();
        }
        let again = save::load(&Save::from_text(&back.game.save(0).to_text()).unwrap(), |_| {}).expect("what it saves loads");
        (refused, carried, again.refused.len(), again.drifted)
    });
    let (refused, carried, refused_again, drifted) = played.unwrap_or_else(|_| panic!("an order in a file stopped the program"));
    assert!(refused > 100, "{refused} of {} orders were refused", doctored.orders.len());
    assert_eq!(refused + carried, doctored.orders.len());
    // what was carried out is a game like any other: saved and loaded, it comes back as it was
    assert_eq!((refused_again, drifted), (0, false));
}

#[test]
fn a_game_with_as_many_families_as_there_are_names_starts_and_plays() {
    let mut g = Game::new(7, save::MAX_FAMILIES, 0);
    assert_eq!(g.sim.st.families.len(), 30);
    let names: std::collections::BTreeSet<&str> = g.sim.st.families.iter().map(|f| f.surname.as_str()).collect();
    assert_eq!(names.len(), 30, "every family has a surname of its own");
    g.year().unwrap();
    let back = save::load(&Save::from_text(&g.save(0).to_text()).unwrap(), |_| {}).unwrap();
    assert!(!back.drifted);
}

#[test]
fn dates_are_written_in_utc() {
    assert_eq!(save::utc(0), "1970-01-01 00:00 UTC");
    assert_eq!(save::utc(951_782_400), "2000-02-29 00:00 UTC");
    assert_eq!(save::utc(1_794_366_765), "2026-11-11 03:12 UTC");
}
