//! A town's making (`making.rs`, the viewer's Making panel; asked for as
//! `inspect::Thing::Making`): on island 7, played as the viewer plays a game
//! (the game's rules as `save::Game::new` sets them, six families), in
//! January 1760 and January 1830, every town that stands is asked for its
//! making, and for each good what came into its pool (made here, came in from
//! other towns, out of store) equals what left it (bought, sold to other towns,
//! into store or spoiled) within a hundredth, and the drawing's own ribbons
//! carry exactly the books' figures. Each box's worth in and out is the sum of
//! its ribbons, no more than the budget of boxes stands by name, the columns
//! run from the sources on the left to the sinks on the right, and every word
//! is a word. Before a year has closed, under the port's rules (no books), and
//! for a town that is not there, the question is refused in a sentence.
//!
//! The case leans on one rule's switch, `accounts.enabled` (the books the
//! chart reads), which it checks is on; nothing else here leans on what becomes
//! of the island: whatever towns and works there are, each is asked about.
#![allow(non_snake_case)]

use isostate_core::accounts;
use isostate_core::inspect::{Thing, tell};
use isostate_core::making::{Kind, MOST_LAND, MOST_WORKS, Making};
use isostate_core::params::Rules;
use isostate_core::save::Game;

fn sum(m: &Making, g: usize, from: &dyn Fn(Kind) -> bool, to: &dyn Fn(Kind) -> bool) -> f64 {
    m.ribbons.iter().filter(|r| r.good == g && from(m.nodes[r.from].kind) && to(m.nodes[r.to].kind)).map(|r| r.qty).sum()
}

fn close(a: f64, b: f64, of: f64, what: &str) {
    assert!((a - b).abs() <= 1e-6 * of.max(1.0), "{what}: {a} against {b}");
}

fn check(g: &Game) {
    let (sim, st, C) = (&g.sim, &g.sim.st, &g.sim.content);
    assert!(sim.params.accounts.enabled, "the chart reads the books, which the game's rules keep");
    let towns: Vec<usize> = (0..st.town_name.len()).filter(|&t| st.town_active[t]).collect();
    assert!(!towns.is_empty());
    for &t in &towns {
        let name = &st.town_name[t];
        let told = tell(sim, Thing::Making(t)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(told.title, *name);
        let m = told.making.expect("a making");
        let tb = accounts::town_books(st, C, t, None).expect("the books of the last year");
        assert_eq!((m.town, m.year), (t, sim.year - 1), "{name}: the chart is of the last year closed");

        // good by good: in equals out within a hundredth, and the ribbons carry the books' figures
        let any = |_: Kind| true;
        let makes = |k: Kind| k.makes();
        for gb in &tb.goods {
            let gi = gb.good;
            let label = format!("{name}, {} in {}", C.good_name[gi], m.year);
            let ins = sum(&m, gi, &any, &any);
            let flow = m.flow(gi);
            if ins == 0.0 {
                assert!(gb.made <= 0.0 && gb.received == 0.0 && gb.bought == 0.0 && gb.sent == 0.0, "{label}: the books have it, the chart does not");
                continue;
            }
            let f = flow.unwrap_or_else(|| panic!("{label}: no flow kept"));
            let inflow = f.made + f.received + f.from_store;
            let outflow = f.bought + f.sent + f.to_store;
            let of = inflow.max(outflow);
            assert!((inflow - outflow).abs() <= 0.01 * of, "{label}: in {inflow} against out {outflow}");
            // the drawing: every unit leaves one box and enters another
            close(ins, inflow, of, &format!("{label}: the ribbons against what came in"));
            // what was made, came in and went out, as the books keep it
            close(f.made, gb.made.max(0.0), of, &format!("{label}: made"));
            close(f.received, gb.received, of, &format!("{label}: came in"));
            close(f.bought, gb.bought, of, &format!("{label}: bought"));
            close(f.sent, gb.sent, of, &format!("{label}: sold away"));
            close(sum(&m, gi, &makes, &any), gb.made.max(0.0), of, &format!("{label}: the makers' ribbons"));
            close(sum(&m, gi, &|k| k == Kind::FromTowns, &any), gb.received, of, &format!("{label}: from other towns"));
            close(sum(&m, gi, &any, &|k| k == Kind::ToTowns), gb.sent, of, &format!("{label}: to other towns"));
            close(sum(&m, gi, &any, &|k| k == Kind::Households), gb.households + gb.landlords + gb.families, of, &format!("{label}: the households"));
            close(sum(&m, gi, &any, &|k| matches!(k, Kind::Works | Kind::OtherWorks)), gb.works_inputs + gb.works_fuel, of, &format!("{label}: the works"));
            close(sum(&m, gi, &any, &|k| k == Kind::LandBuys), gb.land, of, &format!("{label}: the land"));
            close(sum(&m, gi, &any, &|k| k == Kind::Building), gb.building + gb.projects, of, &format!("{label}: building"));
            for r in m.ribbons.iter().filter(|r| r.good == gi) {
                close(r.value, r.qty * gb.price, of * gb.price.max(1.0), &format!("{label}: a ribbon's worth"));
            }
        }

        // the boxes: their worth is their ribbons', the budget holds, the columns run one way
        for (i, x) in m.nodes.iter().enumerate() {
            let vin: f64 = m.ribbons.iter().filter(|r| r.to == i).map(|r| r.value).sum();
            let vout: f64 = m.ribbons.iter().filter(|r| r.from == i).map(|r| r.value).sum();
            close(x.value_in, vin, vin.max(vout), &format!("{name}, {}: worth in", x.label));
            close(x.value_out, vout, vin.max(vout), &format!("{name}, {}: worth out", x.label));
            if x.kind.source() {
                assert_eq!((x.col, x.value_in), (0, 0.0), "{name}, {}: a source stands on the left and takes nothing", x.label);
            }
            if x.kind.sink() {
                assert_eq!((x.col, x.value_out), (m.cols - 1, 0.0), "{name}, {}: a sink stands on the right and gives nothing", x.label);
            }
            assert!(!x.said.is_empty() && x.said.iter().all(|l| !l.is_empty() && !l.contains("NaN") && !l.contains("inf")), "{name}, {}: {:?}", x.label, x.said);
        }
        let named = |k: Kind| m.nodes.iter().filter(|x| x.kind == k).count();
        assert!(named(Kind::Land) <= MOST_LAND + 1 && named(Kind::Works) <= MOST_WORKS + 1, "{name}: {} land and {} works by name", named(Kind::Land), named(Kind::Works));
        assert!(named(Kind::OtherLand) <= 1 && named(Kind::OtherWorks) <= 1);
        for r in &m.ribbons {
            let (a, b) = (&m.nodes[r.from], &m.nodes[r.to]);
            assert!(r.qty > 0.0 && r.value.is_finite(), "{name}: a ribbon of {} from {} to {}", C.good_name[r.good], a.label, b.label);
            assert_eq!(r.back, r.from != r.to && a.col >= b.col, "{name}: {} to {}", a.label, b.label);
            assert!(!r.said.is_empty() && r.said.iter().all(|l| !l.contains("NaN")));
        }
        // the kinds it could have: known here, none standing, none of their good made by works here
        for f in &m.faint {
            assert!(!m.nodes.iter().any(|x| x.kind == Kind::Works && x.good == Some(f.good)), "{name}: {} is faint and stands", f.label);
            assert!(f.col >= 1 && f.col < m.cols - 1);
        }
    }
    // a town that is not there
    assert!(tell(sim, Thing::Making(st.town_name.len())).is_err());
}

#[test]
fn a_towns_making_balances_good_by_good_and_carries_the_books_figures() {
    let mut g = Game::new(7, 6, 0);
    // before a year has closed the books have nothing to show, and it says so
    let e = tell(&g.sim, Thing::Making(0)).expect_err("no year closed yet");
    assert!(e.starts_with("No year has closed yet") && e.ends_with('.'), "{e}");
    for year in [1760, 1830] {
        while g.sim.year < year {
            if g.month().expect("a month is played") {
                g.turn();
            }
        }
        check(&g);
    }
}

#[test]
fn without_the_books_the_making_is_refused_in_a_sentence() {
    // the port's rules keep no books
    let mut g = Game::under(Rules::Port, 7, 6, 0);
    while g.sim.year < 1752 {
        if g.month().expect("a month is played") {
            g.turn();
        }
    }
    assert!(!g.sim.params.accounts.enabled);
    let e = tell(&g.sim, Thing::Making(0)).expect_err("no books");
    assert!(e.starts_with("The books of account are off") && e.ends_with('.'), "{e}");
}
