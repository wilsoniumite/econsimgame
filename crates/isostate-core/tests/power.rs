//! The first content after 1900 (docs/rules/tree.md, "After 1900:
//! electricity"): electricity and electrical goods, their two works, and a good
//! that does not keep. Stage 6 of the game's overlay; at stage 1 the island is
//! the island as it was.
#![allow(non_snake_case)]

use isostate_core::accounts::{KMO, MONTH_FIELDS};
use isostate_core::content::method_index;
use isostate_core::families::Group;
use isostate_core::sim;
use isostate_core::tree::{self, GAME_OVERLAY_TOML};
use isostate_core::{Content, Params, game_content, game_params, save};

fn game(seed: u64, families: usize, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = Some(end_year);
    P
}

#[test]
fn the_overlay_at_stage_six_parses_and_every_name_resolves() {
    assert_eq!(tree::GAME_STAGE, 6);
    assert_eq!(game_params().tree.stage, 6);
    let one = tree::game_content_at(1);
    let six = tree::game_content_at(6);
    // four goods (electricity, electrical goods, oil and fertiliser), six methods (the offices and
    // the steel-frame builders among them: docs/rules/tree.md, "Offices and steel frames") and two
    // inventions (breeding: docs/rules/tree.md, "Oil, nitrogen and the land"; managed woodland:
    // docs/rules/woods.md), and the four ideas the machines that tend wait on (docs/rules/tending.md),
    // nothing else; the tree's stages 2 to 5 have no entries
    assert_eq!((six.G, six.S, six.NS, six.M), (one.G + 4, one.S, one.NS, one.M + 6));
    assert_eq!(six.tech_name.len() - six.M, one.tech_name.len() - one.M + 6);
    for k in 2..=5 {
        assert_eq!(format!("{:?}", tree::game_content_at(k).good_name), format!("{:?}", one.good_name), "stage {k}");
        assert_eq!(tree::game_content_at(k).M, one.M, "stage {k}");
    }
    // stage 1's goods and methods keep their numbers
    assert_eq!(six.good_name[..one.G], one.good_name[..]);
    assert_eq!(six.method_name[..one.M], one.method_name[..]);
    assert_eq!(six.tech_name[six.M..six.M + one.tech_name.len() - one.M], one.tech_name[one.M..]);
    for name in ["electricity", "electricals", "coal_power_station", "electrical_works"] {
        assert!(!one.good_index.contains_key(name) && !one.method_name.iter().any(|n| n == name), "{name} is not there at stage 1");
    }
    let (e, x) = (six.gi("electricity"), six.gi("electricals"));
    // electricity: not carried, does not keep, opens dear, warms and lights and is a works' fuel
    assert!(!six.traded[e] && six.traded[x]);
    assert_eq!((six.stock_months[e], six.stock_months[x]), (Some(0.0), None));
    assert!(!tree::keeps(&six, e) && tree::keeps(&six, x) && (0..one.G).all(|g| tree::keeps(&six, g)));
    assert!(six.opens_dear[e] && six.opens_dear[x] && (0..one.G).all(|g| !six.opens_dear[g]));
    assert_eq!((six.base_price[e], six.weight[e], six.base_price[x], six.weight[x]), (1.2, 20.0, 7.0, 0.5));
    let need = |n: &str| six.service_name.iter().position(|s| s == n).unwrap();
    assert_eq!((six.props[(e, need("warmth"))], six.props[(x, need("wares"))]), (11.0, 5.0));
    assert_eq!(six.heat[e], 5.0);
    assert!(six.fuel_goods.contains(&e) && !one.fuel_goods.iter().any(|&g| one.good_name[g] == "electricity"));
    // the two works: what they make, whose trade they are, their classes of task and their names
    let (st, ew) = (method_index(&six, "coal_power_station"), method_index(&six, "electrical_works"));
    assert_eq!((six.method_out[st], six.method_out[ew]), (e, x));
    assert_eq!(six.m_in[(st, six.gi("coal"))], 1.1);
    assert_eq!((six.m_heat[st], six.m_labor[st]), (0.0, 0.1));
    assert!(six.m_in[(ew, six.gi("steel"))] == 0.25 && six.m_in[(ew, six.gi("chemicals"))] == 0.1 && six.m_in[(ew, six.gi("glass"))] == 0.05);
    assert_eq!(Group::of_method(&six, st), Some(Group::CoalMining));
    assert_eq!(Group::of_method(&six, ew), Some(Group::Engineering));
    assert_eq!(six.tasks.method[st], [0.30, 0.50, 0.05, 0.15, 0.0]);
    assert_eq!(six.tasks.method[ew], [0.15, 0.55, 0.05, 0.25, 0.0]);
    for m in [st, ew] {
        let name = &six.method_name[m];
        assert!(!six.tech_known[m] && six.tech_target[m] > 1880, "{name}");
        assert!(isostate_core::names::firm_patterns(name).len() > 1, "{name} has names of its own");
        assert!(six.tech_headline[m].contains("{town}") && !six.tech_story[m].is_empty(), "{name}");
        // plant of kind M, at base prices: three tenths building work, a tenth steel, six tenths machinery
        let worth = |g: &str| six.m_cap[(m, six.gi(g))] * six.base_price[six.gi(g)];
        let all: f64 = (0..six.G).map(|g| six.m_cap[(m, g)] * six.base_price[g]).sum();
        for (g, share) in [("building", 0.3), ("steel", 0.1), ("machinery", 0.6)] {
            assert!((worth(g) / all - share).abs() < 0.01, "{name}: {g} is {} of its plant", worth(g) / all);
        }
        // the station's plant is five years of its output, the works' 1.2
        let years = all / (12.0 * six.base_price[six.method_out[m]]);
        assert!((years - if m == st { 5.0 } else { 1.2 }).abs() < 0.02, "{name}: {years} years");
        // found on what the island has made
        let on: Vec<&str> = (0..six.G).filter(|&g| six.tech_disc[(m, g)] > 0.0).map(|g| six.good_name[g].as_str()).collect();
        assert_eq!(on, if m == st { vec!["coal", "electricals"] } else { vec!["steel", "machinery"] }, "{name}");
    }
    // the works' heat is shared out over its fuels with electricity among them
    let groups = tree::share_groups(&six, &[("heat", "heat", &["timber", "charcoal", "coal"])]);
    assert!(groups.iter().any(|(n, k, m)| n == "heat" && k == "heat" && m.iter().any(|x| x == "electricity")));
    let groups = tree::share_groups(&one, &[("heat", "heat", &["timber", "charcoal", "coal"])]);
    assert!(groups.iter().any(|(n, k, m)| n == "heat" && k == "heat" && m.len() == 3));
    // laid again it is the same content
    let mut a = six.clone();
    let mut b = tree::lay(&six, GAME_OVERLAY_TOML, 6).unwrap();
    a.good_index.clear();
    b.good_index.clear();
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}

/// The fingerprint (`save::check`) of island 7 with six families played to
/// 1900 under the game's rules with `tree.stage = 1`, as the build this stage
/// was cut from (night/g-to2050-5c-plant at 7e42fa6, where the game's rules
/// were stage 1) plays it. Taken there with the code of this test.
const STAGE_ONE_1900: &str = "87f7b4e58257d704";

#[test]
fn at_stage_one_the_island_is_the_island_as_it_was() {
    // played under the game's rules as they are, with the tree's ceiling at stage 1:
    // nothing of stage 6 is laid, and nothing else may move. When a later change moves
    // the game at stage 1 on purpose, it takes this again (ISOSTATE_GOLDEN_PRINT=1)
    let mut P = game(7, 6, 1900);
    P.tree.stage = 1;
    // played with the woods' rule off (docs/rules/woods.md), which came after: its two
    // series of the woods' cover and timber's price are in the fingerprint, though they
    // touch nothing, and its invention is stage 6's
    P.woods = Default::default();
    // and on the old island, which the fingerprint was taken on: since the game's rules
    // took the new land (docs/rules/land.md, "The switch") island 7 is another island
    P.land = Default::default();
    // and with the merchants' crumb off (docs/rules/prices.md, "A crumb is no glut"),
    // which came after and acts before 1900
    P.prices.crumb = 0.0;
    // and with the year's mean of works' hands, which the game's rules switched off
    // after the fingerprint was taken (docs/rules/towns.md, 20 November 2026)
    P.towns.mean_hands = true;
    // and with the linked measure off (docs/rules/measures.md), which came after: its three
    // series are in the fingerprint, though they touch nothing, and the news's slump clock reads it
    P.linked = Default::default();
    // and with the state's purse off (docs/rules/state.md), which came after: its series
    // are in the fingerprint, though they are nothing to 1900
    P.state = Default::default();
    // and with who works by age off (docs/rules/who-works.md), which came after: its
    // series are in the fingerprint, though they are nothing to 1900
    P.who_works = Default::default();
    let mut s = sim::start(&P, game_content());
    while s.year <= 1900 {
        sim::run_year(&mut s).unwrap();
    }
    let got = save::check(&s);
    if std::env::var_os("ISOSTATE_GOLDEN_PRINT").is_some() {
        panic!("const STAGE_ONE_1900: &str = \"{got}\";");
    }
    assert_eq!(got, STAGE_ONE_1900, "the island at tree.stage = 1 is no longer the island of stage 1");
}

/// The game's content at stage 6 with both works known from the start, so
/// that a short run has stations in some towns and not in others. A station of
/// the usual size (1,500 a month, with five years of its output in plant) is
/// more than a town of 1750 can raise, so here its usual size is 20.
fn known_from_the_start() -> Content {
    let mut C = tree::game_content_at(6);
    for name in ["coal_power_station", "electrical_works"] {
        let m = method_index(&C, name);
        C.tech_known[m] = true;
    }
    let st = method_index(&C, "coal_power_station");
    C.m_K0[st] = 20.0;
    C
}

#[test]
fn electricity_does_not_keep_and_a_town_without_a_station_has_none() {
    // under the game's rules at stage 6, played on island 3 with six families with the
    // two works known in 1750, so that stations stand in some towns early; to 1765
    let P = game(3, 6, 1765);
    let C = known_from_the_start();
    let mut s = sim::start(&P, C);
    let (e, G) = (s.content.gi("electricity"), s.content.G);
    let station = method_index(&s.content, "coal_power_station");
    let (made, bought, stock) = (1, 2, 3);
    assert_eq!(MONTH_FIELDS[made], "made");
    assert_eq!(MONTH_FIELDS[bought], "bought");
    assert_eq!(MONTH_FIELDS[stock], "stock");
    // on the game's island it opens at its ceiling everywhere, as electrical goods do:
    // nobody has either
    let T = s.st.T_max;
    let six = tree::game_content_at(6);
    let (st0, _) = sim::init_state(&P, &six);
    for g in [e, six.gi("electricals")] {
        for t in 0..T {
            assert_eq!(st0.price[(t, g)], P.price_ceiling * six.base_price[g], "town {t} opens {} at {}", six.good_name[g], st0.price[(t, g)]);
        }
    }
    let (mut with, mut without, mut lit_months) = (0, 0, 0);
    while s.year <= 1765 {
        // which towns have a station standing through the year
        let had: Vec<bool> = (0..T)
            .map(|t| (0..s.st.F_count).any(|f| s.st.f_alive[f] && s.st.f_town[f] == t && s.st.f_method[f] == station))
            .collect();
        for month in 0..12 {
            sim::run_month(&mut s, month).unwrap();
            // the run checks its money each month, and so does this: what there is now is
            // what there was at the start and what has been issued since, and nothing else
            let drift = s.st.money_now - (s.st.money_start + s.st.issued);
            assert!(drift.abs() < 1e-6 * s.st.money_now.max(1.0), "{}-{month}: money drifted by {drift}", s.year);
        }
        sim::end_year(&mut s);
        // the year's months, as the books kept them
        let m = &s.st.acct.last_months;
        for month in 0..12 {
            for t in (0..T).filter(|&t| s.st.town_active[t]) {
                let at = |c: usize| m[((month * T + t) * G + e) * KMO + c] as f64;
                // what is in store at a month's end is that month's make and nothing older
                assert!((at(stock) - at(made)).abs() <= 1e-6 * at(made).max(1.0), "{} town {t} month {month}: {} in store, {} made", s.year, at(stock), at(made));
                // (what is sold is not held to the month before's make: a works burns its
                // mix of fuels in the shares their prices give, whatever each fuel's fill,
                // and the books count what it burnt beyond the stock as `oversold`, as
                // they do for coal)
                if at(made) > 0.0 {
                    lit_months += 1;
                }
                if !had[t] && month > 0 {
                    // a town with no station has none to sell, and buys none. (Not in
                    // January, which sells December's make: a district that came to the
                    // town at the year's end brings its merchants' stock with it, as of
                    // every good. And its works' fuel mix, read at the price, may put a
                    // hundred-thousandth of a unit on it where a station has just closed.)
                    assert_eq!(at(made), 0.0, "{} town {t}", s.year);
                    assert!(at(bought) < 1e-3, "{} town {t} month {month}: bought {} electricity with no station", s.year, at(bought));
                }
            }
        }
        with += had.iter().zip(&s.st.town_active).filter(|(h, a)| **h && **a).count();
        without += had.iter().zip(&s.st.town_active).filter(|(h, a)| !**h && **a).count();
    }
    // the case was a case: towns with stations and towns without, and electricity made
    assert!(with > 0 && without > 0 && lit_months > 12, "town-years with a station {with}, without {without}; months lit {lit_months}");
    // the books count what was lost as spoiled, and they balance
    let worst = s.st.acct.years.iter().map(|y| y.balance_goods).fold(0.0, f64::max);
    assert!(worst < 1e-6, "the books' goods balance by {worst}");
}

#[test]
fn the_gap_of_a_good_that_does_not_keep_is_asking_against_making() {
    let C = tree::game_content_at(6);
    let P = game_params();
    let (e, coal) = (C.gi("electricity"), C.gi("coal"));
    // asked and made alike: no gap; asked and none made: all the way up; made and none asked: all the way down
    assert_eq!(tree::price_gap(&C, &P, e, 50.0, 10.0, 7.0, 7.0), 0.0);
    assert_eq!(tree::price_gap(&C, &P, e, 0.0, 0.0, 7.0, 0.0), 1.0);
    assert_eq!(tree::price_gap(&C, &P, e, 9.0, 9.0, 0.0, 9.0), -1.0);
    assert_eq!(tree::price_gap(&C, &P, e, 0.0, 0.0, 0.0, 0.0), 0.0);
    assert!((tree::price_gap(&C, &P, e, 123.0, 4.0, 3.0, 4.0) + 0.25).abs() < 1e-12, "the stock is not read");
    // a good that keeps is the port's rule: two months of sales against the stock
    let port = |stock: f64, of: f64| {
        let target = P.price_target_months * of;
        (target - stock) / target.max(stock).max(1e-9)
    };
    for (stock, of) in [(0.0, 5.0), (10.0, 5.0), (30.0, 5.0), (0.0, 0.0)] {
        assert_eq!(tree::price_gap(&C, &P, coal, stock, of, 99.0, 1.0), port(stock, of));
    }
    // the loss at a month's end: all that is left of electricity, each other good's decay
    let loss = tree::month_loss(&C, &P);
    let decay = tree::decay(&C, &P);
    for g in 0..C.G {
        assert_eq!(loss[g], if g == e { 1.0 } else { decay[g] }, "{}", C.good_name[g]);
    }
}
