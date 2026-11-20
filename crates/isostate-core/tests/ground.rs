//! The ground (docs/rules/ground.md): from the end of 1900, house rent that
//! clears the room a town has built, and how far a town may reach from its
//! centre. Off under the port; the port's golden test holds that nothing
//! reaches it, and the game's golden test holds the island with it on.
//!
//! The cases are island 7 under the game's rules with six families, with the
//! ground's switches set by name in each; the rest of the game's rules are as
//! `game_params()` has them.
#![allow(non_snake_case)]

use isostate_core::ground::GroundParams;
use isostate_core::sim::{self, Sim};
use isostate_core::*;

/// A heap for each thread: the tests play islands side by side (`heaps.rs`).
#[global_allocator]
static HEAPS: isostate_core::heaps::ThreadHeaps = isostate_core::heaps::ThreadHeaps;

#[test]
fn the_rule_is_off_under_the_port_and_on_in_the_game() {
    assert_eq!(default_params().ground, GroundParams::default());
    let off = GroundParams::default();
    assert!(!off.enabled && off.room && off.reach && off.belt.is_none());
    assert_eq!((off.from, off.blend_years, off.reach_walk, off.reach_tram), (1900, 20, 5, 8));
    assert_eq!((off.room_min, off.beta, off.dens_rate, off.floors, off.floors_frame), (0.3, 0.09, 0.02, 1.5, 3.0));
    let on = game_params().ground;
    assert!(on.enabled && on.room && on.reach && on.belt.is_none());
    // a port run's params.json does not name it
    let written = serde_json::to_value(default_params()).unwrap();
    assert!(written.get("ground").is_none());
}

/// Island 7 under the game's rules with six families, the ground switched as
/// `ground`, played to the end of `to`.
fn played(ground: GroundParams, to: i64) -> Sim {
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(to);
    P.ground = ground;
    let mut s = sim::start(&P, game_content());
    while s.year <= to {
        // the month's money check is on (`Sim::check_money`): a month that
        // loses or makes money stops the run here
        sim::run_year(&mut s).unwrap_or_else(|e| panic!("{e}"));
    }
    s
}

#[test]
fn before_1901_the_island_is_the_island_without_it() {
    // island 7 to the end of 1900: the rule on (room, reach and the tramway as
    // the game has them) and off
    let on = played(GroundParams::game(), 1900);
    let off = played(GroundParams::default(), 1900);
    assert_eq!(save::check(&on), save::check(&off));
    // every month of every series, to the bit (a series not yet written holds NaN)
    let bits = |s: &Sim| s.st.series.data.iter().map(|v| v.iter().map(|x| x.to_bits()).collect::<Vec<u64>>()).collect::<Vec<_>>();
    assert_eq!(bits(&on), bits(&off));
    assert_eq!(on.st.town_ground, off.st.town_ground);
    assert_eq!(on.st.urban_rent, off.st.urban_rent);
    // at the end of 1900 the rule has taken over, and nothing is built up yet
    let g = &on.st.ground;
    assert!(g.on && g.year_now == 1901 && g.years.is_empty() && g.tram_year.is_none());
    assert!(g.dens.iter().all(|&d| d == 1.0));
    // each town's room is what it has, and its rent a unit of room today's bill over it
    for t in (0..on.st.T_max).filter(|&t| on.st.town_active[t]) {
        let room: f64 = on.st.town_ground[t].iter().map(|&c| on.st.town_room[c]).sum();
        assert_eq!(g.room[t], room);
        assert!((g.rent0[t] - on.st.urban_rent[t] / room).abs() <= 1e-12 * g.rent0[t].abs(), "town {t}");
    }
    // off, it stays empty
    assert!(!off.st.ground.on && off.st.ground.dens.is_empty());
}

#[test]
fn after_1900_no_cell_joins_beyond_reach_and_a_belt_of_one_holds() {
    // island 7 to 1930 with room and reach on, a belt of one cell a year, and
    // no tramway (found no sooner than 3000), so that every town's reach is the walk
    let mut ground = GroundParams::game();
    ground.belt = Some(1);
    ground.tram_after = 3000;
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(1930);
    P.ground = ground.clone();
    let mut s = sim::start(&P, game_content());
    let mut joined = 0;
    while s.year <= 1930 {
        let year = s.year;
        sim::run_year(&mut s).unwrap_or_else(|e| panic!("{e}"));
        if year <= 1900 {
            continue;
        }
        let st = &s.st;
        let W = st.world.W;
        for t in (0..st.T_max).filter(|&t| st.town_active[t]) {
            let node = st.town_node[t];
            // the cells it took this year (a town founded this year stands on its centre first)
            let new: Vec<usize> = st.town_ground[t].iter().copied().filter(|&c| st.town_built[c] == year && c != node).collect();
            assert!(new.len() <= 1, "{year}: town {t} took {} cells", new.len());
            for c in new {
                let d = ((c / W) as f64 - (node / W) as f64).hypot((c % W) as f64 - (node % W) as f64);
                assert!(d <= ground.reach_walk as f64, "{year}: town {t} took a cell {d:.2} cells from its centre");
                joined += 1;
            }
        }
    }
    let g = &s.st.ground;
    assert_eq!(g.years.len(), 30);
    assert!(joined > 0, "no town took a cell after 1900: the case does not test reach");
    // the burden never passes a half, in any town in any month
    for y in &g.years {
        assert!(y.burden_worst <= 0.5 + 1e-12, "{}: {}", y.year, y.burden_worst);
        assert!(y.burden > 0.0 && y.burden < 0.5, "{}: {}", y.year, y.burden);
        assert!(y.joined_most <= 1);
    }
    // what is built up stays between 1 and the frames' floors
    assert!(g.dens.iter().all(|&d| (1.0..=ground.floors_frame).contains(&d)));
    assert!(g.dens.iter().any(|&d| d > 1.0), "nothing was built up by 1930");
}
