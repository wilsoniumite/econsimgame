//! The island's settings (isostate_core::island, docs/rules/land.md, "The
//! island's settings"): a game made with some of them is remembered by its save
//! and played again on the same island, a save made before them reads as
//! before, and each setting does to the land what it says.
//!
//! Every case is played under the game's rules on the new land, on the settled
//! island; they lean on no island's history, only on a game being the same twice
//! and on what the land is made of, which no rule of the economy touches.
#![allow(non_snake_case)]

use isostate_core::island::Island;
use isostate_core::landcells::{generate_land, land_alone, sketch_land};
use isostate_core::params::{Rules, game_params};
use isostate_core::save::{self, Game, Order, Save, Switches};

fn series_bits(g: &Game) -> Vec<Vec<u64>> {
    g.sim.st.series.data.iter().map(|d| d.iter().map(|x| x.to_bits()).collect()).collect()
}

fn years(g: &mut Game, n: usize) {
    for _ in 0..n {
        g.year().unwrap();
        g.turn();
    }
}

#[test]
fn a_game_made_with_three_settings_and_gone_on_with_is_the_one_played_straight_through() {
    // island 7, six families, to 1780: drier, with twice the coal, and half as many again of people
    let island = Island::parse("rain=600,coal=2,people=200000").unwrap();
    let begin = || Game::begin_with(Rules::Game, 7, 6, 1_000, 1900, true, false, Switches::default(), &island).unwrap();
    let mut straight = begin();
    assert_eq!((straight.sim.params.land.rain, straight.sim.params.land.riches.coal, straight.sim.params.pop0), (600.0, 2.0, 200_000));
    assert_eq!(straight.island(), island);
    let mut played = begin();
    for g in [&mut straight, &mut played] {
        years(g, 6);
        g.give(Order::MoveSeat { town: 1 }).unwrap();
        years(g, 6);
        for _ in 0..5 {
            g.month().unwrap();
        }
    }
    // saved five months into 1762 and gone on with
    let text = played.save(2_000).to_text();
    assert!(text.contains(r#""island": {"rain":600.0,"coal":2.0,"people":200000.0}"#), "{text}");
    let loaded = save::load(&Save::from_text(&text).unwrap(), |_| {}).unwrap();
    assert!(!loaded.drifted && loaded.refused.is_empty());
    let mut back = loaded.game;
    assert_eq!(back.sim.params, straight.sim.params);
    assert_eq!(back.island(), island);
    for g in [&mut straight, &mut back] {
        for _ in 0..7 {
            g.month().unwrap();
        }
        g.turn();
        years(g, 18);
        assert_eq!(g.last_year(), 1780);
    }
    assert_eq!(save::check(&back.sim), save::check(&straight.sim));
    assert!(series_bits(&back) == series_bits(&straight), "every series to the bit");
    // and a game made with no setting writes none
    let plain = Game::begin_with(Rules::Game, 7, 2, 0, 1900, true, false, Switches::default(), &Island::default()).unwrap();
    assert!(!plain.save(0).to_text().contains("island"));
}

#[test]
fn a_save_made_before_the_settings_opens_and_plays() {
    // made by main's code of 27 November 2026, before the settings (island 7, six families, the
    // seat moved to town 1 at the turn of 1755, saved four months into 1760): it has no line
    // for them, is the island as it comes, and is written again with none. (Whether it stands
    // as it was saved leans on island 7's history to 1760, which later rules may move, so it
    // is not asked here; it did on the day, docs/rules/land.md says.)
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/settled_main_1760.json")).unwrap();
    let s = Save::from_text(&text).unwrap();
    assert!(s.island.plain());
    let loaded = save::load(&s, |_| {}).unwrap();
    println!("the save of main, played again: drifted {}, {} orders refused", loaded.drifted, loaded.refused.len());
    let mut g = loaded.game;
    assert!(g.island().plain() && g.sim.params.land == isostate_core::landform::LandParams::game());
    for _ in 0..8 {
        g.month().unwrap();
    }
    g.turn();
    assert_eq!(g.last_year(), 1760);
    assert!(!g.save(0).to_text().contains("island"));
}

#[test]
fn a_game_refuses_a_setting_that_does_not_apply() {
    let ask = |words: &str, land: bool, landfall: bool| Game::begin_with(Rules::Game, 7, 1, 0, 1900, land, landfall, Switches::default(), &Island::parse(words).unwrap()).err().unwrap_or_default();
    assert!(ask("hills=3000", true, false).contains("outside its range"));
    assert!(ask("coal=2", false, false).contains("new land only"));
    assert!(ask("towns=3", true, true).contains("landfall begins with one town"));
}

#[test]
fn each_setting_does_to_the_land_what_it_says() {
    let seed = 2;
    let P = game_params();
    let with = |words: &str| {
        let mut Q = game_params();
        Island::parse(words).unwrap().set(&mut Q).unwrap();
        Q.land
    };
    let plain = land_alone(seed, P.W, P.H, &P.land);
    // the land alone is the island to the bit, with no first towns and the woods not thinned
    let full = generate_land(seed, P.W, P.H, P.n_towns, &P.land);
    assert!(plain.town_sites.is_empty() && full.town_sites.len() == 6);
    assert_eq!((&plain.land, &plain.h, &plain.fertility, &plain.coal), (&full.land, &full.h, &full.fertility, &full.coal));
    assert_eq!(plain.forest0, plain.forest_pot);
    let count = |v: &[f64]| v.iter().filter(|&&x| x > 0.0).count();
    let sum = |v: &[f64]| v.iter().sum::<f64>();
    // riches: plentiful coal is more coalfield as well as richer; scarce stone is less stone
    let rich = land_alone(seed, P.W, P.H, &with("coal=2,stone=0.5,woods=0.5"));
    assert_eq!(rich.land, plain.land);
    assert!(count(&rich.coal) > count(&plain.coal) && sum(&rich.coal) > sum(&plain.coal), "coal cells {} against {}", count(&rich.coal), count(&plain.coal));
    assert!(sum(&rich.rock) < sum(&plain.rock) && sum(&rich.forest_pot) < sum(&plain.forest_pot));
    assert!(rich.coal.iter().chain(&rich.ore).chain(&rich.rock).chain(&rich.clay).chain(&rich.forest_pot).all(|&x| (0.0..=1.0).contains(&x)));
    assert_eq!(rich.fertility, plain.fertility);
    // no lakes
    let dry = land_alone(seed, P.W, P.H, &with("lakes=0"));
    assert_eq!(dry.landform.as_ref().unwrap().lakes.len(), 0);
    // more rain, more river
    let rivers = |w: &isostate_core::world::World| w.river.iter().filter(|&&r| r).count();
    let wet = land_alone(seed, P.W, P.H, &with("rain=1400"));
    assert!(rivers(&wet) > rivers(&plain), "{} river cells against {}", rivers(&wet), rivers(&plain));
    // more towns stand closer, and are all found
    let mut many = game_params();
    Island::parse("towns=12").unwrap().set(&mut many).unwrap();
    assert_eq!(generate_land(seed, P.W, P.H, many.n_towns, &many.land).town_sites.len(), 12);
    // the sketch is the same island near enough to choose by
    let sketch = sketch_land(seed, P.W, P.H, &P.land);
    let same = (0..plain.land.len()).filter(|&c| plain.land[c] == sketch.land[c]).count() as f64 / plain.land.len() as f64;
    assert!(same > 0.97, "the sketch calls {same} of the cells as the whole does");
}
