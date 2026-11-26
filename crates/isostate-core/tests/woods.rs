//! The woods (docs/rules/woods.md): managed woodland, an invention of stage 6
//! of the game's overlay that grows the woods back faster where it is known,
//! and the woods' cover and timber's price in months of the wage recorded. Off
//! under the port; the port's golden test holds that nothing reaches it, and
//! the game's golden test holds the island with it on.
#![allow(non_snake_case)]

use isostate_core::content::tech_index;
use isostate_core::sim::{self, Sim};
use isostate_core::woods::{self, WoodsParams, MANAGED};
use isostate_core::*;

#[test]
fn the_rule_is_off_under_the_port_and_on_in_the_game() {
    assert_eq!(default_params().woods, WoodsParams::default());
    let off = WoodsParams::default();
    assert!(!off.enabled && !off.record && off.regrow == 3.0);
    let on = game_params().woods;
    assert!(on.enabled && on.record && on.regrow == 3.0);
    // a port run's params.json does not name it
    let written = serde_json::to_value(default_params()).unwrap();
    assert!(written.get("woods").is_none());
    // the invention is stage 6's, found on timber made and not before 1895; the
    // port's content has no year before which an invention may not be found
    let six = tree::game_content_at(6);
    let t = tech_index(&six, MANAGED);
    assert_eq!(six.tech_kind[t], "tech");
    assert!(!six.tech_known[t]);
    assert_eq!(six.tech_after[t], 1895);
    let on: Vec<&str> = (0..six.G).filter(|&g| six.tech_disc[(t, g)] > 0.0).map(|g| six.good_name[g].as_str()).collect();
    assert_eq!(on, ["timber"]);
    assert!(six.tech_headline[t].contains("{town}") && !six.tech_story[t].is_empty());
    assert!(!tree::game_content_at(1).tech_name.iter().any(|n| n == MANAGED));
    let port = default_content();
    assert_eq!(port.tech_after.len(), port.NT);
    assert!(port.tech_after.iter().all(|&y| y == 0));
}

/// The game's content at stage 6 with managed woodland found on a single unit
/// of timber from 1760, so that a short run finds it.
fn found_early() -> Content {
    let mut C = tree::game_content_at(6);
    let t = tech_index(&C, MANAGED);
    let timber = C.gi("timber");
    C.tech_disc[(t, timber)] = 1.0;
    C.tech_after[t] = 1760;
    C
}

/// Island 3 under the game's rules with six families, played to 1790 on `C`.
fn played(P: &Params, C: Content) -> Sim {
    let mut s = sim::start(P, C);
    while s.year <= 1790 {
        sim::run_year(&mut s).unwrap();
    }
    s
}

fn game(seed: u64, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end_year);
    P
}

#[test]
fn it_is_found_after_its_year_spreads_and_grows_the_woods_back_faster() {
    // island 3 under the game's rules with six families, to 1790, with managed woodland
    // to be found from 1760 on a unit of timber (`found_early`)
    let P = game(3, 1790);
    let s = played(&P, found_early());
    let t = tech_index(&s.content, MANAGED);
    let year = s.st.tech_year[t];
    assert!((1760..=1790).contains(&year), "found in {year}");
    // the town it was found in knows it, and it has spread to others
    let (st, C) = (&s.st, &s.content);
    let towns: Vec<usize> = (0..st.T_max).filter(|&i| st.town_active[i]).collect();
    assert!(st.know[(st.tech_town[t] as usize, t)] >= 0.25);
    let knowing = towns.iter().filter(|&&i| st.know[(i, t)] > 0.01).count();
    assert!(knowing > 1, "{knowing} towns of {} know it", towns.len());
    // a cell whose market knows it grows back at the port's rate times 1 + 2 * know;
    // with the rule off, at the port's
    let managed = woods::managed(C, &P);
    assert_eq!(managed, Some(t));
    let mut seen = 0;
    for c in (0..st.N).filter(|&c| st.catch[c] >= 0) {
        let k = st.know[(st.catch[c] as usize, t)];
        let want = P.forest_regrow * (1.0 + 2.0 * k);
        assert!((woods::regrow(st, &P, managed, c) - want).abs() < 1e-15);
        assert_eq!(woods::regrow(st, &P, None, c), P.forest_regrow);
        if k > 0.5 {
            seen += 1;
        }
    }
    assert!(seen > 0, "no cell's market knows it by half");
    // both series are kept every month, and read as they should
    let cover = st.series.get("woods_cover");
    let timber = st.series.get("timber_in_wages");
    for m in 0..s.t {
        assert!(cover[m] > 0.0 && cover[m] <= 1.0, "month {m}: cover {}", cover[m]);
        assert!(timber[m] > 0.0 && timber[m].is_finite(), "month {m}: timber {}", timber[m]);
    }
    // (the land's yearly step has moved the cover since December was recorded)
    let now = woods::cover(st);
    assert!(now > 0.0 && now <= 1.0 && (now - cover[s.t - 1]).abs() < 0.1, "{now} against December's {}", cover[s.t - 1]);
}

#[test]
fn with_the_rule_off_it_is_never_sought_and_the_island_is_the_island_without_it() {
    // island 3 under the game's rules with six families, to 1790: managed woodland to be
    // found from 1760 (`found_early`), with the rule off, against the same island with no
    // such invention in its content at all. Neither records the woods, so the series match
    let mut P = game(3, 1790);
    P.woods = WoodsParams::default();
    let off = played(&P, found_early());
    let t = tech_index(&off.content, MANAGED);
    assert!(off.st.tech_year[t] < 0, "found in {} with the rule off", off.st.tech_year[t]);
    assert!((0..off.st.T_max).all(|i| off.st.know[(i, t)] == 0.0));
    // the overlay without the invention
    // (read with its line ends made plain, so that a checkout that writes the file with
    // Windows line ends finds the blank line after the invention too)
    let overlay = tree::GAME_OVERLAY_TOML.replace("\r\n", "\n");
    let start = overlay.find("name = \"managed_woodland\"").unwrap();
    let start = overlay[..start].rfind("[[tech]]").unwrap();
    let end = overlay[start..].find("\n\n").map_or(overlay.len(), |k| start + k);
    let without = format!("{}{}", &overlay[..start], &overlay[end..]);
    let C = tree::lay(&game_content(), &without, 6).unwrap();
    assert!(!C.tech_name.iter().any(|n| n == MANAGED));
    let none = played(&P, C);
    assert_eq!(save::check(&off), save::check(&none));
    assert!(!off.st.series.index.contains_key("woods_cover"));
}
