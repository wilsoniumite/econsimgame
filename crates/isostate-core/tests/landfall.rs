//! The two starts (`docs/rules/landfall.md`), births that answer to free land
//! and hamlets (`docs/rules/frontier.md`), and settlers from abroad
//! (`docs/rules/immigration.md`), none of them in the Python.
//!
//! The settled island is what every run had before: none of the three is on
//! under the game's rules, and the word "settled" changes nothing. Landfall is
//! the same island with one town in a corner. The runs here are played under
//! the game's rules with `--start landfall` (six families), with each rule
//! named by its switch where a case depends on it.
#![allow(non_snake_case)]

use isostate_core::*;

/// The game's rules with six families on island `seed` to `end_year`, at landfall.
fn landfall_params(seed: u64, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end_year);
    landfall::set_start(&mut P, "landfall").unwrap();
    P
}

#[test]
fn the_settled_island_is_the_game_as_it_was() {
    // none of the three rules is on under the game's rules or the port's
    for P in [game_params(), default_params()] {
        assert!(!P.start.landfall && !P.frontier.enabled && !P.frontier.hamlets && !P.immigration.enabled);
        assert_eq!(P.hands.short_rise, 2.0);
        let json = serde_json::to_string(&P).unwrap();
        for word in ["\"start\"", "\"frontier\"", "\"immigration\"", "short_rise"] {
            assert!(!json.contains(word), "{word} is written out while it is off");
        }
    }
    // and the word "settled" changes nothing; a word that is not a start is refused
    let mut P = game_params();
    landfall::set_start(&mut P, "settled").unwrap();
    assert_eq!(P, game_params());
    assert!(landfall::set_start(&mut P, "colony").unwrap_err().contains("the starts are settled and landfall"));
    // with hamlets off the founding asks what the people's rule asks
    assert_eq!(frontier::founding_needs(&P, 1.0), (P.people.found_people, P.people.donor_max));
}

#[test]
fn landfall_is_one_harbour_in_a_corner_of_the_same_island() {
    for seed in [1, 4, 7] {
        let mut P = game_params();
        P.seed = seed;
        let settled = landcells::generate_land(P.seed, P.W, P.H, P.n_towns, &P.land);
        let mut world = settled.clone();
        landfall::settle(&mut world);
        // the same island: land, heights and soils as the makers left them
        assert_eq!(world.land, settled.land);
        assert_eq!(world.fertility, settled.fertility);
        assert_eq!(world.coast, settled.coast);
        // one town, on the coast, within a quarter of the land's box of one of its corners
        assert_eq!(world.town_sites.len(), 1);
        let c = world.town_sites[0];
        assert!(world.land[c] && world.coast[c], "island {seed}: the landfall is a harbour");
        let W = world.W;
        let (xs, ys): (Vec<usize>, Vec<usize>) = (0..world.land.len()).filter(|&i| world.land[i]).map(|i| (i % W, i / W)).unzip();
        let (x0, x1, y0, y1) = (*xs.iter().min().unwrap(), *xs.iter().max().unwrap(), *ys.iter().min().unwrap(), *ys.iter().max().unwrap());
        let (x, y) = (c % W, c / W);
        let near_x = x - x0 < (x1 - x0) / 4 || x1 - x < (x1 - x0) / 4;
        let near_y = y - y0 < (y1 - y0) / 3 || y1 - y < (y1 - y0) / 3;
        assert!(near_x && near_y, "island {seed}: the landfall at {x}, {y} is not near a corner of {x0}..{x1}, {y0}..{y1}");
        // the woods are thinned round that town alone: far from it they stand as the makers grew them
        let far = (0..world.land.len()).filter(|&i| world.land[i] && ((i % W) as f64 - x as f64).hypot((i / W) as f64 - y as f64) > 60.0).collect::<Vec<_>>();
        assert!(far.iter().all(|&i| (world.forest0[i] - world.forest_pot[i]).abs() < 0.01 * world.forest_pot[i].max(1e-9)));
    }
}

#[test]
fn a_landfall_run_settles_its_people_and_its_settlers_by_age() {
    // played under --start landfall: births that answer to free land (frontier.enabled),
    // hamlets (frontier.hamlets), settlers (immigration.enabled), a short town's wage at
    // most doubling in a year (hands.short_rise), and the ages watched (ages.watch)
    let P = landfall_params(7, 1765);
    assert!(P.frontier.enabled && P.frontier.hamlets && P.immigration.enabled && P.ages.watch);
    let mut sim = sim::start(&P, game_content());
    let act0: Vec<usize> = (0..sim.st.T_max).filter(|&t| sim.st.town_active[t]).collect();
    assert_eq!(act0.len(), 1, "one town at the start");
    assert!((sim.st.pop[act0[0]] - P.start.people).abs() < 1e-6 * P.start.people);
    let mut years = 0;
    while sim.year <= sim.end_year {
        sim::run_year(&mut sim).expect("the money check holds");
        years += 1;
        let st = &sim.st;
        // the ages are reckoned with the settlers in them, and add up to the people
        let people: f64 = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.pop[t]).sum();
        let rows: f64 = st.ages.rows.iter().map(|r| r.iter().sum::<f64>()).sum();
        assert!((rows - people).abs() < 1e-6 * people, "{}: the ages hold {rows}, the towns {people}", sim.year - 1);
        assert!(st.frontier.free.iter().chain(&st.frontier.lifted).all(|&f| (0.0..=1.0).contains(&f)));
    }
    assert_eq!(years, 16);
    let st = &sim.st;
    // settlers came, and were counted by age as young adults more than births would be
    assert!(st.immigration.total > 1000.0, "settlers {}", st.immigration.total);
    assert!(sim.events.iter().any(|e| e.kind == "settlers"), "the first ship is news");
    // a second town was founded, at a hamlet's size, within the first fifteen years
    assert!(st.pp_foundings.iter().any(|f| f.year <= 1765), "no town founded by 1765");
    // and the people grew faster than births alone at the island's ceiling could make them
    let people: f64 = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.pop[t]).sum();
    assert!(people > P.start.people * (1.016f64).powi(16), "{people}");
}

#[test]
fn landfall_carts_at_prices_from_1750() {
    // the setting comes with the start, and only with it; the sweep's word puts it back
    let mut P = landfall_params(7, 1770);
    assert_eq!((P.longrun.freight_from, game_params().longrun.freight_from), (1750, 1900));
    assert_eq!(sweep::set_field(&P, "longrun.freight_from", &sweep::parse_value("1900")).unwrap().longrun.freight_from, 1900);
    // played under --start landfall (frontier.enabled, frontier.hamlets, immigration.enabled,
    // hands.short_rise 1.06, longrun.enabled), once with the freight on prices from 1750 and
    // once from 1900: the colony's prices fall as its people outgrow its money, and so does
    // its carriage, with fewer of its hands on the road (to 1770: the merchants' debt of the
    // first years lifts the colony's prices to about 1760 before they fall)
    assert!(P.longrun.enabled && P.frontier.enabled && P.immigration.enabled && P.hands.short_rise < 2.0);
    let mut seen = Vec::new();
    for from in [1750, 1900] {
        P.longrun.freight_from = from;
        let mut sim = sim::start(&P, game_content());
        while sim.year <= sim.end_year {
            sim::run_year(&mut sim).expect("the money check holds");
        }
        let st = &sim.st;
        let works: f64 = (0..st.F_count).filter(|&k| st.f_alive[k]).map(|k| st.f_labor[k]).sum();
        let (land, carts): (f64, f64) = (st.ex_labor.iter().sum(), st.carters.iter().sum());
        seen.push((longrun::freight(st, &P) / P.freight, carts / (works + land + carts)));
    }
    let ((rate_1750, carts_1750), (rate_1900, carts_1900)) = (seen[0], seen[1]);
    assert_eq!(rate_1900, 1.0, "before 1900 the freight is the constant");
    assert!(rate_1750 < 0.7, "the colony's freight in 1770 is {rate_1750} of 1750's");
    assert!(carts_1750 < carts_1900, "carters over all hands: {carts_1750} on prices, {carts_1900} in coins");
}

#[test]
fn landfall_pits_work_at_the_size_of_their_market() {
    // the switch comes with the start, and only with it; the sweep's word puts it back
    let mut P = landfall_params(7, 1754);
    assert!(P.mines.glut && !game_params().mines.glut && !default_params().mines.glut);
    assert!(!sweep::set_field(&P, "mines.glut", &sweep::parse_value("false")).unwrap().mines.glut);
    // played under --start landfall (frontier.enabled, frontier.hamlets, immigration.enabled,
    // hands.short_rise 1.06, longrun.freight_from 1750, mines.enabled, prices.enabled), once
    // with the pits at the size of their market and once without. Without, island 7's
    // new iron-ore pit of 1751 and colliery of 1753 raise far more than the colony asks,
    // the merchants buy it all, and in some December they owe more than twice the money;
    // with it they owe at most what the warm-up left them owing, and the colony still digs
    assert!(P.mines.enabled && P.prices.enabled && P.frontier.enabled && P.immigration.enabled);
    let coal = game_content().gi("coal");
    let mut seen = Vec::new();
    for glut in [true, false] {
        P.mines.glut = glut;
        let mut sim = sim::start(&P, game_content());
        let (mut owe_max, mut coal_1754) = (0.0f64, 0.0);
        while sim.year <= sim.end_year {
            sim::run_year(&mut sim).expect("the money check holds");
            let st = &sim.st;
            let owe: f64 = st.merch_cash.iter().map(|&c| (-c).max(0.0)).sum();
            owe_max = owe_max.max(owe / st.money_now);
            let r = st.mine_sectors.iter().position(|&s| sim.content.sector_good[s] == coal).unwrap();
            coal_1754 = st.mine_out.row(r).iter().sum();
        }
        seen.push((owe_max, coal_1754));
    }
    let ((owe_on, coal_on), (owe_off, _)) = (seen[0], seen[1]);
    assert!(owe_off > 2.0, "without the rule the merchants owe at most {owe_off} of the money in a December");
    assert!(owe_on < 0.6, "with it they owe {owe_on} of the money in a December");
    assert!(coal_on > 10_000.0, "the colony raised {coal_on} of coal in 1754");
}

#[test]
fn with_immigration_off_nobody_comes() {
    // landfall with immigration.enabled off: the same start, nobody arrives
    let mut P = landfall_params(7, 1754);
    P.immigration.enabled = false;
    let r = sim::run_with(&P, game_content(), false, true).unwrap();
    assert_eq!(r.state.immigration.total, 0.0);
    assert!(r.state.immigration.years.is_empty());
    assert!(!r.events.iter().any(|e| e.kind == "settlers"));
}

#[test]
fn the_living_abroad_stands_still_and_then_grows() {
    let mut P = game_params();
    P.immigration = immigration::ImmigrationParams::game();
    assert_eq!(immigration::abroad(&P, 1750), P.immigration.abroad);
    assert_eq!(immigration::abroad(&P, P.immigration.abroad_from), P.immigration.abroad);
    let later = immigration::abroad(&P, P.immigration.abroad_from + 70);
    assert!((later / P.immigration.abroad - 2.0).abs() < 0.01, "about a doubling in seventy years: {later}");
    // the settlers' ages are one in all, two in three of them 15 to 34
    let all: f64 = immigration::AGES.iter().sum();
    assert!((all - 1.0).abs() < 1e-12);
    let young: f64 = immigration::AGES[ages::MOVERS].iter().sum();
    assert!(young > 0.6 && young < 0.7, "{young}");
}
