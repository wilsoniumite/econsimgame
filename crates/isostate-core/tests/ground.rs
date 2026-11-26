//! The brush on the ground (edits.rs, GROUND-BRUSH; docs/designs/sandbox.md,
//! "Built: the brush on the ground"): coal laid down beside a town that has
//! none is found and worked by a pit the mines' rule opens, and coal is
//! cheaper there than on the untouched island; the coal taken from under the
//! first city's pits leaves them idle until the rule gives them up; a stroke
//! is refused in its own words where it would change nothing; a session with
//! strokes saved and loaded, and a run with no window given the same strokes
//! as a dated list, come back the same to the last digit.
//!
//! Every case is played on island 7 under the game's rules with six families
//! (`Game::new`), and compares a game with a copy of itself left untouched. It
//! leans on the mines' rule (`mines.enabled`), which finds and works a deposit
//! and gives up a working that stands idle; `at` asserts that it is on. The
//! numbers it prints are the outcome of night/g-ground-brush.
#![allow(non_snake_case)]

use isostate_core::edits::{self, Dated, Edit, Ground};
use isostate_core::econ;
use isostate_core::params::Rules;
use isostate_core::save::{self, Done, Game, Order, Save};

fn by_hand(edit: Edit) -> Order {
    Order::ByHand { edit }
}

/// Island 7 played `years` years from its start, at the turn.
fn at(years: usize) -> Game {
    let mut g = Game::new(7, 6, 1_000);
    for _ in 0..years {
        g.year().unwrap();
        g.turn();
    }
    assert!(g.sim.params.mines.enabled, "played under the mines' rule");
    g
}

fn on(g: &mut Game, years: usize) {
    for _ in 0..years {
        g.year().unwrap();
        g.turn();
    }
}

fn drift(g: &Game) -> f64 {
    let st = &g.sim.st;
    let now = econ::money_total(st);
    (now - (st.money_start + st.issued + st.by_hand.money)).abs() / now.max(1.0)
}

/// The row of the mines' tables that holds the collieries.
fn coal_row(g: &Game) -> usize {
    let C = &g.sim.content;
    let s = C.sector_name.iter().position(|n| n == "coalmine").unwrap();
    isostate_core::mines::row(&g.sim.st, s).unwrap()
}

/// The collieries open now whose pit head stands on one of `cells`, as (cell, the year it was opened).
fn pits_on(g: &Game, cells: &[usize]) -> Vec<(usize, i32)> {
    let (st, r) = (&g.sim.st, coal_row(g));
    cells.iter().filter(|&&c| st.mine_opened[r * st.N + c] != 0).map(|&c| (c, st.mine_opened[r * st.N + c])).collect()
}

/// The coal a town's district holds.
fn coal_in(g: &Game, t: usize) -> f64 {
    let st = &g.sim.st;
    (0..st.N).filter(|&c| st.catch[c] == t as i64).map(|c| st.world.coal[c]).sum()
}

/// Coal's price in town t, and in the town's wages, as December closed.
fn coal_price(g: &Game, t: usize) -> (f64, f64) {
    let st = &g.sim.st;
    let coal = g.sim.content.gi("coal");
    (st.price[(t, coal)], st.price[(t, coal)] / st.wage[t].max(1e-9))
}

#[test]
fn coal_laid_beside_a_town_with_none_is_worked_and_cheapens_its_coal_and_the_session_comes_back() {
    let base = at(40);
    let st = &base.sim.st;
    // the most peopled town whose district holds no coal
    let t = (0..st.T_max).filter(|&t| st.town_active[t] && coal_in(&base, t) == 0.0).max_by(|&a, &b| st.pop[a].total_cmp(&st.pop[b])).expect("a town with no coal");
    let node = st.town_node[t];
    let w = st.world.W;
    // a square of open land in its district, three squares or more from the market place, the nearest such
    let d2 = |c: usize| ((c % w) as i64 - (node % w) as i64).pow(2) + ((c / w) as i64 - (node / w) as i64).pow(2);
    let square = (0..st.N).filter(|&c| st.catch[c] == t as i64 && st.world.land[c] && !st.urban[c] && d2(c) >= 9).min_by_key(|&c| (d2(c), c)).unwrap();
    eprintln!("{} ({:.0} people) has no coal; the brush goes down at {}, {}, {:.1} squares from its market place", st.town_name[t], st.pop[t], square % w, square / w, (d2(square) as f64).sqrt());
    let edit = Edit::Ground { kind: Ground::Coal, lay: true, size: 2, squares: vec![square, square + 1] };
    let (changed, rich) = edits::ground_plan(st, &base.sim.content, Ground::Coal, true, 2, &[square, square + 1]);
    let laid: Vec<usize> = changed.iter().map(|x| x.0).collect();
    eprintln!("laid on {} squares, {rich:.3} under the middle", laid.len());
    assert!(laid.len() >= 20 && rich > 0.3, "a stroke of size 2 over two squares lays a coalfield");

    let mut g = base.clone();
    let t0 = std::time::Instant::now();
    assert_eq!(g.give(by_hand(edit.clone())).unwrap(), Done::Done);
    eprintln!("the stroke was carried out in {:.2} ms", t0.elapsed().as_secs_f64() * 1e3);
    let ev = g.sim.events.last().unwrap();
    eprintln!("{}\n  {}", ev.title, ev.text);
    assert_eq!((ev.kind.as_str(), ev.cell), ("hand", Some(square + 1)));
    assert!(ev.title.starts_with("By hand: coal laid down on ") && ev.text.contains("No pit is opened by the hand"), "{}", ev.title);
    assert!(pits_on(&g, &laid).is_empty(), "no pit is opened by the hand");
    assert!(laid.iter().all(|&c| g.sim.st.by_hand.ground.contains(&(0, c))));
    // the same stroke again lays nothing more, and says so
    let why = g.clone().give(by_hand(edit.clone())).unwrap_err();
    assert!(why.contains("holds as much coal as the brush lays already"), "{why}");

    // thirty years on, beside the untouched island
    let mut plain = base.clone();
    let mut first: Option<i32> = None;
    for y in 1..=30 {
        on(&mut g, 1);
        on(&mut plain, 1);
        let pits = pits_on(&g, &laid);
        if first.is_none() && !pits.is_empty() {
            first = Some(pits.iter().map(|p| p.1).min().unwrap());
        }
        if y == 10 || y == 30 {
            let ((p, pw), (q, qw)) = (coal_price(&g, t), coal_price(&plain, t));
            eprintln!(
                "{}: {} collieries on the laid ground; coal in {} {p:.3} ({pw:.3} of a wage), untouched {q:.3} ({qw:.3}); its people {:.0}, untouched {:.0}",
                g.sim.year - 1,
                pits.len(),
                g.sim.st.town_name[t],
                g.sim.st.pop[t],
                plain.sim.st.pop[t]
            );
        }
        assert!(drift(&g) < 1e-6);
    }
    let opened = first.expect("a colliery is opened on the coal laid down within thirty years");
    eprintln!("the first colliery on it was opened in {opened}");
    let ((_, pw), (_, qw)) = (coal_price(&g, t), coal_price(&plain, t));
    assert!(pw < qw, "coal is cheaper in the town than on the untouched island, in its wages: {pw} against {qw}");
    // a colliery's own pits never stood on the laid ground on the untouched island
    assert!(pits_on(&plain, &laid).is_empty());

    // saved and loaded, the same to the last digit; and a run with no window given the stroke
    let b = save::load(&Save::from_text(&g.save(2_000).to_text()).unwrap(), |_| {}).unwrap();
    assert!(!b.drifted && b.refused.is_empty(), "{:?}", b.refused);
    assert_eq!(save::check(&b.game.sim), save::check(&g.sim));
    assert_eq!(b.game.sim.st.by_hand, g.sim.st.by_hand);
    let mut P = save::params(Rules::Game, 7, 6);
    P.end_year = Some(g.sim.year - 1);
    let run = edits::play(&P, Rules::Game.content(), &[Dated { before: (1790, 1), edit }]).unwrap();
    assert_eq!(save::check(&run), save::check(&g.sim), "the stroke and the same list played with no window give the same island");
    assert_eq!(run.st.world.coal, g.sim.st.world.coal);
}

#[test]
fn coal_taken_from_under_the_first_citys_pits_leaves_them_idle_until_the_rule_gives_them_up() {
    let base = at(60);
    let st = &base.sim.st;
    let r = coal_row(&base);
    let N = st.N;
    // the collieries at work in the districts of the towns, the town with most of them first
    let working: Vec<usize> = (0..N).filter(|&c| st.mine_opened[r * N + c] != 0 && st.mine_hands[(r, c)] >= 1.0 && st.catch[c] >= 0).collect();
    let t = (0..st.T_max).filter(|&t| st.town_active[t]).max_by_key(|&t| (working.iter().filter(|&&c| st.catch[c] == t as i64).count(), std::cmp::Reverse(t))).unwrap();
    let heads: Vec<usize> = working.iter().copied().filter(|&c| st.catch[c] == t as i64).collect();
    let w = st.world.W;
    let first = (0..st.T_max).filter(|&t| st.town_active[t]).max_by(|&a, &b| st.pop[a].total_cmp(&st.pop[b])).unwrap();
    eprintln!(
        "the first city is {} ({:.0} people, at {}, {}), with {} collieries at work in its district",
        st.town_name[first],
        st.pop[first],
        st.town_node[first] % w,
        st.town_node[first] / w,
        working.iter().filter(|&&c| st.catch[c] == first as i64).count()
    );
    let squares: Vec<String> = heads.iter().map(|&c| format!("{},{}", c % w, c / w)).collect();
    eprintln!("{} ({:.0} people, at {}, {}) has {} collieries at work in January {}: {}", st.town_name[t], st.pop[t], st.town_node[t] % w, st.town_node[t] / w, heads.len(), base.sim.year, squares.join(" "));
    assert!(!heads.is_empty());
    // the stroke passes over every pit head, wide enough to take every square its lease reaches
    let size = base.sim.params.mines.reach.ceil() as u8;
    let mut g = base.clone();
    g.give(by_hand(Edit::Ground { kind: Ground::Coal, lay: false, size, squares: heads.clone() })).unwrap();
    let ev = g.sim.events.last().unwrap();
    eprintln!("{}\n  {}", ev.title, ev.text);
    assert!(ev.title.starts_with("By hand: coal taken away from ") && ev.text.contains("raise nothing more"), "{}", ev.text);
    assert!(heads.iter().all(|&c| g.sim.st.world.coal[c] == 0.0));
    // nothing is closed by the hand
    assert_eq!(pits_on(&g, &heads).len(), heads.len());
    let why = g.clone().give(by_hand(Edit::Ground { kind: Ground::Coal, lay: false, size, squares: heads.clone() })).unwrap_err();
    assert!(why.starts_with("there is no coal round"), "{why}");

    let mut plain = base.clone();
    let mut closed = None;
    for y in 1..=20 {
        on(&mut g, 1);
        on(&mut plain, 1);
        let open = pits_on(&g, &heads).len();
        let hands: f64 = heads.iter().map(|&c| g.sim.st.mine_hands[(r, c)]).sum();
        if y <= 2 || y % 5 == 0 || (open == 0 && closed.is_none()) {
            eprintln!("{}: {open} of the {} still stand, with {hands:.0} hands; untouched {} stand", g.sim.year - 1, heads.len(), pits_on(&plain, &heads).len());
        }
        if open == 0 && closed.is_none() {
            closed = Some(g.sim.year - 1);
        }
        if y >= 2 {
            assert!(hands < 1.0 * heads.len() as f64, "a working with no deposit raises nothing: {hands} hands");
        }
        assert!(drift(&g) < 1e-6);
    }
    let year = closed.expect("every colliery is given up within twenty years");
    let idle = base.sim.params.mines.idle_years as i64;
    eprintln!("the last of them was given up in {year}; the rule gives up a working after {idle} idle years");
    assert!(year - base.sim.year <= idle + 2);
}

#[test]
fn a_stroke_is_refused_in_its_own_words_and_timber_is_planted_only_where_woods_may_stand() {
    let base = at(10);
    let mut g = base.clone();
    let n = g.sim.st.N;
    let why = g.give(by_hand(Edit::Ground { kind: Ground::Stone, lay: true, size: 1, squares: vec![] })).unwrap_err();
    assert!(why.contains("at least one square"), "{why}");
    let why = g.give(by_hand(Edit::Ground { kind: Ground::Stone, lay: true, size: 9, squares: vec![0] })).unwrap_err();
    assert!(why.contains("at most 4 squares"), "{why}");
    let why = g.give(by_hand(Edit::Ground { kind: Ground::Stone, lay: true, size: 1, squares: vec![n] })).unwrap_err();
    assert!(why.contains("no square"), "{why}");
    // the sea's corner holds nothing and takes nothing
    let why = g.give(by_hand(Edit::Ground { kind: Ground::Clay, lay: true, size: 1, squares: vec![0] })).unwrap_err();
    assert!(why.contains("holds as much clay"), "{why}");
    assert_eq!(save::check(&g.sim), save::check(&base.sim), "a refused stroke changes nothing");

    // timber: planted where the woods' and the wild's share lets trees stay, never on town ground
    let st = &g.sim.st;
    let t = (0..st.T_max).find(|&t| st.town_active[t]).unwrap();
    let node = st.town_node[t];
    let (changed, _) = edits::ground_plan(st, &g.sim.content, Ground::Timber, true, 3, &[node]);
    assert!(changed.iter().all(|&(c, v)| !st.urban[c] && v <= st.world.forest_pot[c] + 1e-12), "trees are planted on no town ground, and no thicker than the ground bears");
    if !changed.is_empty() {
        g.give(by_hand(Edit::Ground { kind: Ground::Timber, lay: true, size: 3, squares: vec![node] })).unwrap();
        let ev = g.sim.events.last().unwrap();
        eprintln!("{}\n  {}", ev.title, ev.text);
        assert!(changed.iter().all(|&(c, v)| g.sim.st.cover[c] == v));
        on(&mut g, 2);
        assert!(drift(&g) < 1e-6);
    }
    let mut cut = base.clone();
    cut.give(by_hand(Edit::Ground { kind: Ground::Timber, lay: false, size: 3, squares: vec![node] })).unwrap();
    eprintln!("{}", cut.sim.events.last().unwrap().title);
    on(&mut cut, 2);
    assert!(drift(&cut) < 1e-6);
}
