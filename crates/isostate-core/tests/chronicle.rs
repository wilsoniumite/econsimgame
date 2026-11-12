//! The chronicle (not in the Python; docs/rules/chronicle.md): switched off it
//! leaves no trace; switched on it never touches the economy, says why every
//! observer item is news, keeps a handful of items a decade, tells the towns'
//! stories, and ends the flood of the player's own works. A live game's marks
//! end where `sim::finish` puts them.
#![allow(non_snake_case)]

use std::collections::BTreeMap;
use std::sync::OnceLock;

use isostate_core::chronicle::{self, ChronicleParams, Why};
use isostate_core::*;

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn game(n_families: usize, end_year: i64) -> Params {
    let mut P = game_params();
    P.n_families = n_families;
    P.end_year = Some(end_year);
    P
}

/// Seed 7 with six families under the game's rules, the whole century and a
/// half: the game the viewer opens with. Run once for the tests that read it.
fn whole_game() -> &'static RunResult {
    static RUN: OnceLock<RunResult> = OnceLock::new();
    RUN.get_or_init(|| sim::run_with(&game(6, 1900), game_content(), false, true).expect("money is conserved every month"))
}

/// The same game with the chronicle switched off: the same economy to the last
/// bit, with the news as the port and the other mechanisms file it.
fn whole_game_untold() -> &'static RunResult {
    static RUN: OnceLock<RunResult> = OnceLock::new();
    RUN.get_or_init(|| {
        let mut P = game(6, 1900);
        P.chronicle = ChronicleParams::default();
        sim::run_with(&P, game_content(), false, true).expect("money is conserved every month")
    })
}

fn decade(e: &Event) -> i64 {
    ((e.year.floor() as i64 - 1750) / 10).min(14)
}

fn class(e: &Event) -> String {
    match e.kind.as_str() {
        "alarm" | "verdict" => "cycle".to_string(),
        "railway" | "rail_plan" => "railway".to_string(),
        "harvest" | "dearth" => "harvest".to_string(),
        "family" if e.family == Some(0) => "own".to_string(),
        kind => kind.to_string(),
    }
}

#[test]
fn switched_off_it_leaves_no_trace() {
    assert_eq!(default_params().chronicle, ChronicleParams::default());
    assert!(!default_params().chronicle.enabled && game_params().chronicle.enabled);
    let mut P = default_params();
    P.n_families = 6;
    P.end_year = Some(1775);
    let r = sim::run_with(&P, default_content(), false, true).unwrap();
    assert!(r.events.iter().all(|e| e.why.is_none() && e.kept.is_none()));
    let json = serde_json::to_string(&r.events).unwrap();
    assert!(!json.contains("\"why\"") && !json.contains("\"kept\""), "the port's events.json has no new keys");
    assert!(!serde_json::to_string(&P).unwrap().contains("chronicle"), "nor has its params.json");
    assert!(r.state.chron.pop.is_empty() && r.state.chron.kinds.is_empty(), "and the chronicle remembers nothing");
    assert!(r.events.iter().filter(|e| e.family == Some(0)).all(|e| e.weight >= 0.6), "the player's own news weighs 0.6, as it did");
}

#[test]
fn it_never_touches_the_economy_and_runs_are_deterministic() {
    // forty years with it on, the money stock checked every month
    let on = sim::run_with(&game(6, 1790), game_content(), false, true).expect("money is conserved with the chronicle on");
    let st = &on.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    let again = sim::run_with(&game(6, 1790), game_content(), false, true).unwrap();
    assert_eq!(serde_json::to_string(&on.events).unwrap(), serde_json::to_string(&again.events).unwrap(), "the same parameters give the same news");
    // and the same years with it off: every series is the same to the last bit
    let mut P = game(6, 1790);
    P.chronicle = ChronicleParams::default();
    let off = sim::run_with(&P, game_content(), false, true).unwrap();
    assert_eq!(on.state.series.names, off.state.series.names);
    for (name, (a, b)) in on.state.series.names.iter().zip(on.state.series.data.iter().zip(&off.state.series.data)) {
        assert_eq!(bits(a), bits(b), "series {name} is the same with the chronicle on");
    }
    assert_eq!(on.state.f_owner, off.state.f_owner);
    for (a, b) in on.state.families.iter().zip(&off.state.families) {
        assert_eq!(a.cash.to_bits(), b.cash.to_bits());
    }
    // and there is less news, not more. In these forty years that is said of
    // the player's own: with the life cycle for works a family's branches open
    // a twentieth of the works they did, so the flood the chronicle ends is no
    // longer greater than the stories it adds in every stretch of years. Of the
    // whole game it is said of all the news
    // (`the_players_works_no_longer_flood_the_news`).
    let own = |r: &RunResult| r.events.iter().filter(|e| e.family == Some(0)).count();
    assert!(own(&on) < own(&off), "and there is less of the player's news, not more: {} against {}", own(&on), own(&off));
    assert!(on.events.len() < off.events.len() + 10, "nor a flood of anything else: {} items against {}", on.events.len(), off.events.len());
}

#[test]
fn every_observer_item_says_why() {
    let r = whole_game();
    // the harvest's news is of the same sort: a test that fired on a number
    let observer = ["crossing", "milestone", "steep", "alarm", "verdict", "first", "region", "famine", "harvest", "dearth"];
    for e in r.events.iter().filter(|e| observer.contains(&e.kind.as_str())) {
        let w = e.why.as_ref().unwrap_or_else(|| panic!("{} ({}) has no provenance", e.title, e.kind));
        assert!(!w.test.is_empty() && !w.series.is_empty() && !w.note.is_empty(), "{}: {w:?}", e.title);
        assert!(w.value.is_finite() && w.threshold.is_finite(), "{}: {w:?}", e.title);
    }
    for kind in ["crossing", "milestone", "steep", "verdict", "region"] {
        assert!(r.events.iter().any(|e| e.kind == kind), "the run has a {kind} item");
    }
    if r.params.harvest.enabled {
        for kind in ["harvest", "dearth"] {
            assert!(r.events.iter().any(|e| e.kind == kind), "the run has a {kind} item");
        }
        // a dearth carries the price, where the earlier years pointed, and the line above that
        let dearth = r.events.iter().find(|e| e.kind == "dearth").unwrap();
        let w = dearth.why.as_ref().unwrap();
        let pointed = w.baseline.expect("a dearth is measured against a trend");
        assert!(w.value >= w.threshold && (w.threshold / pointed - (1.0 + r.params.harvest.dearth_rise)).abs() < 1e-4, "{w:?}");
        // a poor harvest is under its line and a bumper one over, both against an ordinary year
        for e in r.events.iter().filter(|e| e.title == "A poor harvest" || e.title == "A bumper harvest") {
            let w = e.why.as_ref().unwrap();
            assert_eq!(w.baseline, Some(1.0));
            assert!(if e.title == "A poor harvest" { w.value < w.threshold && w.threshold < 1.0 } else { w.value > w.threshold && w.threshold > 1.0 }, "{}: {w:?}", e.title);
        }
    }
    // a doubling is measured from the mean of the first five years, and says so
    let doubling = r.events.iter().find(|e| e.title.starts_with("Real output doubles")).expect("real output doubles in seed 7");
    assert!(doubling.title.ends_with("since 1750\u{2013}54"), "{}", doubling.title);
    let w = doubling.why.as_ref().unwrap();
    let series = r.state.series.get("gdp_real");
    let base: f64 = (0..5).map(|y| series[y * 12..(y + 1) * 12].iter().sum::<f64>() / 12.0).sum::<f64>() / 5.0;
    assert!((w.baseline.unwrap() / base - 1.0).abs() < 1e-5, "the baseline is the mean of 1750 to 1754: {w:?} against {base}");
    assert!(w.value >= w.threshold && w.threshold >= 2.0 * 0.99999 * base);
    // a crossing carries the share and the line it crossed
    let crossing = r.events.iter().find(|e| e.kind == "crossing" && e.series.is_some() && e.weight == 0.85).unwrap();
    let w = crossing.why.as_ref().unwrap();
    assert_eq!(Some(&w.series), crossing.series.as_ref());
    assert!(w.value > w.threshold && w.threshold == r.params.crossing_hi);
    // large closures say how large (filled in after the fact by the chronicle)
    assert!(r.events.iter().filter(|e| e.kind == "closure").all(|e| e.why.as_ref().is_some_and(|w| w.value > w.threshold)));
}

#[test]
fn a_decade_keeps_a_handful() {
    let r = whole_game();
    let Q = &r.params.chronicle;
    assert!(r.events.iter().all(|e| e.kept.is_some()), "every item is marked one way or the other");
    let mut by_decade: BTreeMap<i64, Vec<&Event>> = BTreeMap::new();
    for e in r.events.iter().filter(|e| e.kept == Some(true)) {
        by_decade.entry(decade(e)).or_default().push(e);
    }
    assert_eq!(by_decade.len(), 15, "every decade has something");
    for (d, kept) in &by_decade {
        assert!((Q.at_least..=Q.per_decade).contains(&kept.len()), "decade {d} keeps {}", kept.len());
        let mut of_class: BTreeMap<String, usize> = BTreeMap::new();
        for e in kept {
            *of_class.entry(class(e)).or_insert(0) += 1;
        }
        for (c, n) in of_class {
            assert!(n <= if c == "discovery" { Q.inventions } else { Q.per_kind }, "decade {d} keeps {n} of {c}");
        }
    }
    let total: usize = by_decade.values().map(|v| v.len()).sum();
    assert!((30..=60).contains(&total), "two to four a decade: {total} in the run");
    // what a history keeps: the inventions, the first railway, the dated slumps
    assert!(r.events.iter().any(|e| e.kept == Some(true) && e.kind == "discovery"));
    assert!(r.events.iter().any(|e| e.kept == Some(true) && e.title.starts_with("The first railway opens")));
    assert!(r.events.iter().any(|e| e.kept == Some(true) && e.kind == "verdict"));
    // an alarm that a verdict has since dated is never kept beside it
    for v in r.events.iter().filter(|e| e.kind == "verdict") {
        let span = v.span.unwrap();
        assert!(r.events.iter().filter(|e| e.kind == "alarm" && e.year >= span[0].floor() && e.year <= v.year).all(|e| e.kept == Some(false)), "{}", v.title);
    }
    // the mark is a harder test than the weight: of the items at 0.6 or more
    // (themselves far fewer than the port's, whose feed in this game was over
    // a thousand long) less than half are kept
    let heavy = r.events.iter().filter(|e| e.weight >= 0.6).count();
    assert!(total * 2 < heavy, "{total} kept of {heavy} at weight 0.6 or more");
}

#[test]
fn the_players_works_no_longer_flood_the_news() {
    let r = whole_game();
    let own: Vec<&Event> = r.events.iter().filter(|e| e.family == Some(0)).collect();
    let surname = &r.state.families[0].surname;
    // no item for a works a branch opened on its own, except where the trade is new to the town
    assert!(own.iter().all(|e| !e.title.contains(" opens a ") || e.title.contains("branch at")), "no works of a branch is told one by one");
    let reached: Vec<&&Event> = own.iter().filter(|e| e.title.contains("sets up in")).collect();
    assert!(!reached.is_empty() && reached.len() < 40, "{} trades reaching towns", reached.len());
    let mut places = std::collections::BTreeSet::new();
    for e in &reached {
        assert!(e.weight == 0.6 && e.firm.is_some() && e.town.is_some());
        assert!(places.insert(e.title.clone()), "{} is told once", e.title);
    }
    // one summary a year at most, light enough to stay out of the major feed
    let summaries: Vec<&&Event> = own.iter().filter(|e| e.title.starts_with(&format!("The {surname} branch"))).collect();
    assert!(summaries.len() > 50, "{} summaries", summaries.len());
    let mut years = std::collections::BTreeSet::new();
    for e in &summaries {
        assert!(e.weight < 0.6 && e.kept == Some(false), "{} weighs {}", e.title, e.weight);
        assert!(years.insert(e.year.floor() as i64), "two summaries in {}", e.year.floor());
    }
    // nor for a works a branch extended (the life cycle for works, docs/rules/works.md):
    // those are in the summaries too
    assert!(own.iter().all(|e| !e.title.contains(" family extends ")), "no extension is told one by one");
    assert!(summaries.iter().any(|e| e.title.contains(" extend")), "the summaries tell of works extended");
    // what the family's branches did on their own in this game, counted in the
    // same game with the chronicle off, where every works a branch opens or
    // extends is an item: the news of them is a fraction of that. (Before the
    // life cycle for works the branches opened about a thousand works; with it
    // they open some fifty and extend them a couple of hundred times.)
    let untold = whole_game_untold();
    let one_by_one = |e: &&Event| e.family == Some(0) && (e.title.contains(" family opens a") || e.title.contains(" family extends "));
    let acts = untold.events.iter().filter(one_by_one).count();
    let works = (0..r.state.F_count).filter(|&f| r.state.f_reason[f].kind == "family" && r.state.f_owner[f] == 0).count();
    assert!(works > 40 && acts > 250 && own.len() < 320, "{} items for {works} works opened, {acts} opened or extended", own.len());
    let told = summaries.len() + reached.len();
    assert!(told * 2 < acts, "{told} items for the {acts} things the branches did");
    let untold_own = untold.events.iter().filter(|e| e.family == Some(0)).count();
    assert!(own.len() * 3 < untold_own * 2, "the player's news is {} items, {untold_own} without the chronicle", own.len());
    // and with the stories the chronicle adds, the whole game's news is still shorter
    assert!(r.events.len() < untold.events.len(), "there is less news, not more: {} against {}", r.events.len(), untold.events.len());
    // the game's rules for the player's own news: its great moments weigh 0.6 or
    // more, the everyday less, and a decade keeps one of them at most
    let Q = &r.params.chronicle;
    for e in &own {
        let great = e.title.contains("takes up") || e.title.contains("heads the family") || e.title.contains("sets up in");
        if e.kind == "family" {
            assert_eq!(e.weight >= 0.6, great, "{} weighs {}", e.title, e.weight);
        }
    }
    // a succession is one great item that says whom the new head follows; the burial is everyday news
    let successions: Vec<&&Event> = own.iter().filter(|e| e.title.contains("heads the family")).collect();
    assert!(successions.len() >= 5, "{} successions in a century and a half", successions.len());
    for e in &successions {
        assert!(e.text.contains(" follows ") && e.text.contains(", who died at ") && (e.text.contains(" are led from ") || e.text.ends_with("the family's seat.")), "{}", e.text);
        let burial = own.iter().find(|b| b.year == e.year && b.text.contains(" head of the family, is buried at ")).expect("the burial is told too");
        assert!(burial.weight == Q.own_small && e.text.contains(burial.title.split(" dies at ").next().unwrap()), "{} / {}", burial.title, e.text);
    }
    assert!(own.iter().any(|e| e.weight == Q.own_great) && own.iter().any(|e| e.weight == Q.own_small) && own.iter().any(|e| e.weight == Q.own_summary));
    let mut kept: BTreeMap<i64, usize> = BTreeMap::new();
    for e in own.iter().filter(|e| e.kept == Some(true)) {
        *kept.entry(decade(e)).or_insert(0) += 1;
    }
    assert!(!kept.is_empty() && kept.values().all(|&n| n == 1), "the player has one place a decade: {kept:?}");
    // rivals as before: the first of each kind, once
    let mut firsts = std::collections::BTreeSet::new();
    for e in r.events.iter().filter(|e| e.family.is_some_and(|k| k > 0) && e.title.contains("opens its first")) {
        assert_eq!(e.weight, 0.35);
        let method = r.state.f_method[e.firm.unwrap()];
        assert!(firsts.insert((e.family.unwrap(), method)), "{} is a first twice", e.title);
    }
    assert!(!firsts.is_empty());
}

#[test]
fn a_works_ordered_by_hand_is_told_as_it_was() {
    let mut s = sim::start(&game(3, 1770), game_content());
    let C = s.content.clone();
    let d = pending(&s.st, 0);
    let textiles = d[0].options.iter().position(|o| o.choice == Choice::Trade { group: Group::Textiles }).unwrap();
    decide(&mut s.st, &C, 0, d[0].id, textiles).unwrap();
    sim::run_year(&mut s).unwrap();
    // cash from nowhere, so the order cannot fail for want of it (the money check is off from here)
    s.check_money = false;
    s.st.families[0].cash += 1e6;
    let seat = s.st.families[0].seat;
    let (cell, method) = (0..s.st.N)
        .filter(|&c| s.st.catch[c] == seat as i64)
        .find_map(|c| site_options(&s.st, &C, &s.params, 0, c).ok()?.into_iter().find(|o| o.group == Some(Group::Textiles) && o.allowed.is_ok()).map(|o| (c, o.method)))
        .expect("a textile works can be built at the seat");
    let before = s.events.len();
    let fid = found_works(&mut s.st, &C, &s.params, 0, method, cell, s.year, &mut s.events).unwrap();
    assert_eq!(s.events.len(), before + 1);
    let e = s.events.last().unwrap().clone();
    assert!(e.family == Some(0) && e.firm == Some(fid) && e.weight == 0.6 && e.title.contains(" opens a"), "{e:?}");
    // a second one the same year is told as well: by hand, every works is news
    let cell2 = (0..s.st.N)
        .filter(|&c| s.st.catch[c] == seat as i64 && c != cell)
        .find(|&c| site_options(&s.st, &C, &s.params, 0, c).is_ok_and(|o| o.iter().any(|o| o.method == method && o.allowed.is_ok())))
        .unwrap();
    found_works(&mut s.st, &C, &s.params, 0, method, cell2, s.year, &mut s.events).unwrap();
    assert_eq!(s.events.len(), before + 2);
    assert!(s.st.chron.branch_opened.is_empty(), "and neither waits for the branches' summary");
    sim::run_year(&mut s).unwrap();
    assert!(s.events[before].kept.is_some(), "the year's ranking marks it");
}

#[test]
fn a_live_games_marks_end_where_the_recorded_run_puts_them() {
    let mut s = sim::start(&game(6, 1812), game_content());
    let mut cursor = LiveCursor::new(&s.content);
    // what a viewer holds: each item as it was sent, then the changes
    let mut held: Vec<Option<bool>> = Vec::new();
    let (mut changes, mut taken_back) = (0, 0);
    while s.year <= s.end_year {
        sim::run_year(&mut s).unwrap();
        let y = live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        for &(i, kept) in &y.rekept {
            assert!(i < held.len(), "a change is to an item already sent");
            assert_ne!(held[i], Some(kept), "and is a change");
            taken_back += (held[i] == Some(true)) as usize;
            held[i] = Some(kept);
            changes += 1;
        }
        held.extend(y.events.iter().map(|e| e.kept));
        assert!(y.events.iter().all(|e| e.kept.is_some()), "every item is sent with a mark");
        // nothing older than the decade before last is changed by a quiet year
        let live_kept = held.iter().filter(|k| **k == Some(true)).count();
        assert!(live_kept <= 4 * ((y.year - 1750) / 10 + 1) as usize);
    }
    assert_eq!(held, s.events.iter().map(|e| e.kept).collect::<Vec<_>>(), "the viewer's marks are the simulation's");
    assert!(changes > 0 && taken_back > 0, "decades were ranked again as they filled ({changes} changes, {taken_back} taken back)");
    let mut live: Vec<(String, String)> = s.events.iter().filter(|e| e.kept == Some(true)).map(|e| (format!("{:.3}", e.year), e.title.clone())).collect();
    let r = sim::finish(s);
    let mut recorded: Vec<(String, String)> = r.events.iter().filter(|e| e.kept == Some(true)).map(|e| (format!("{:.3}", e.year), e.title.clone())).collect();
    live.sort();
    recorded.sort();
    assert_eq!(live, recorded, "sim::finish keeps what the live game kept");
}

fn item(year: f64, kind: &str, weight: f64, title: &str) -> Event {
    Event::new(year, kind, weight, title.to_string(), String::new())
}

fn kept_titles(events: &[Event]) -> Vec<&str> {
    events.iter().filter(|e| e.kept == Some(true)).map(|e| e.title.as_str()).collect()
}

#[test]
fn ranking_a_decade() {
    let P = game_params();
    // caps: two inventions, one of any other kind, four in all; ties to the earlier
    let mut ev = vec![
        item(1761.5, "discovery", 0.9, "invention a"),
        item(1762.5, "discovery", 0.9, "invention b"),
        item(1763.5, "discovery", 0.9, "invention c"),
        item(1764.8, "crossing", 0.85, "crossing a"),
        item(1765.8, "crossing", 0.85, "crossing b"),
        item(1766.8, "steep", 0.8, "boom"),
        item(1767.8, "milestone", 0.75, "milestone"),
        item(1768.8, "town", 0.6, "town"),
    ];
    chronicle::rank(&mut ev, &P, 1750, 1900, 1900);
    assert_eq!(kept_titles(&ev), ["invention a", "invention b", "crossing a", "boom"]);
    // the player's own great moment has a place, whatever else happened
    let mut own = item(1769.95, "family", 0.7, "the family takes up iron");
    own.family = Some(0);
    let mut rival = item(1763.95, "family", 0.7, "a rival's moment");
    rival.family = Some(2);
    ev.push(own);
    ev.push(rival);
    chronicle::rank(&mut ev, &P, 1750, 1900, 1900);
    assert_eq!(kept_titles(&ev), ["invention a", "invention b", "crossing a", "the family takes up iron"]);

    // a thin decade: nothing under the floor while it is running; the best of the rest once it is over
    let mut thin = vec![item(1771.6, "famine", 0.5, "hunger"), item(1772.9, "closure", 0.4, "closure"), item(1773.9, "first", 0.5, "first"), item(1774.9, "rail_plan", 0.45, "scheme")];
    chronicle::rank(&mut thin, &P, 1750, 1900, 1775);
    assert!(kept_titles(&thin).is_empty());
    chronicle::rank(&mut thin, &P, 1750, 1900, 1779);
    assert_eq!(kept_titles(&thin), ["hunger", "first"]);

    // an alarm is kept while it is the latest word, and gives way to the verdict that dates its slump
    let mut slump = vec![item(1811.9, "alarm", 0.65, "output falls"), item(1812.8, "steep", 0.8, "grain slump")];
    chronicle::rank(&mut slump, &P, 1750, 1900, 1812);
    assert_eq!(kept_titles(&slump), ["output falls", "grain slump"]);
    let mut verdict = item(1821.95, "verdict", 0.75, "the depression of 1810 to 1812");
    verdict.span = Some([1810.5, 1812.5]);
    verdict.why = Some(Why { test: "output passes its old peak".into(), series: "gdp_real".into(), value: 101.0, threshold: 100.0, baseline: Some(85.0), note: String::new() });
    slump.push(verdict);
    slump.push(item(1823.8, "steep", 0.8, "iron boom"));
    slump.push(item(1824.5, "discovery", 0.9, "an invention"));
    chronicle::rank(&mut slump, &P, 1750, 1900, 1825);
    // the alarm's decade is ranked again without it; the verdict stands by its depth, above the boom
    assert_eq!(kept_titles(&slump), ["grain slump", "the depression of 1810 to 1812", "iron boom", "an invention"]);
    assert_eq!(slump[0].kept, Some(false));

    // a short crop and the dear bread that follows are one story: the heavier is kept
    let mut bread = vec![item(1887.7, "dearth", 0.7, "a dearth"), item(1889.55, "harvest", 0.75, "a poor harvest"), item(1883.8, "steep", 0.8, "grain shortage")];
    chronicle::rank(&mut bread, &P, 1750, 1900, 1889);
    assert_eq!(kept_titles(&bread), ["a poor harvest", "grain shortage"]);

    // 1900 goes with the 1890s; and off, nothing is marked
    let mut last = vec![item(1895.5, "discovery", 0.9, "a"), item(1900.5, "discovery", 0.9, "b"), item(1900.6, "discovery", 0.9, "c")];
    chronicle::rank(&mut last, &P, 1750, 1900, 1900);
    assert_eq!(kept_titles(&last), ["a", "b"]);
    let mut off = vec![item(1761.5, "discovery", 0.9, "a")];
    chronicle::rank(&mut off, &default_params(), 1750, 1900, 1900);
    assert_eq!(off[0].kept, None);
}

/// The succession is told from the facts the families hand over, not read back
/// out of the burial's sentence: a burial worded any way at all still gives
/// way to the succession, which says whom the new head follows.
#[test]
fn a_succession_holds_to_the_facts_not_to_the_burials_wording() {
    let P = game_params();
    let own = |title: &str, text: &str| -> Option<Event> {
        let mut e = item(1801.95, "family", P.chronicle.own_great, title);
        e.text = text.to_string();
        e.family = Some(0);
        Some(e)
    };
    let year = |place: &str| -> Vec<Option<Event>> {
        vec![
            own("Anne Ashby is born to the Ashby family", "A child at Fenmere."),
            own("Henry Ashby passes away aged 71", &format!("Laid to rest in {place}, as he wished.")),
            own("Mary Ashby heads the family", "The Ashbys are led from Fenmere."),
        ]
    };
    let burial = |place: &str, at_seat: bool| chronicle::Burial { item: 1, who: "Henry Ashby".into(), age: 71, place: place.into(), at_seat };
    // buried at the seat
    let mut items = year("Fenmere");
    chronicle::succession(&P, 0, &mut items, "Mary", Some(&burial("Fenmere", true)));
    assert_eq!(items[2].as_ref().unwrap().text, "Mary follows Henry Ashby, who died at 71 and is buried at Fenmere, the family's seat.");
    assert_eq!(items[1].as_ref().unwrap().weight, P.chronicle.own_small, "the burial is everyday news");
    assert_eq!(items[0].as_ref().unwrap().weight, P.chronicle.own_great, "nothing else is touched");
    // buried elsewhere: the new head's own sentence follows
    let mut items = year("Kirkby");
    chronicle::succession(&P, 0, &mut items, "Mary", Some(&burial("Kirkby", false)));
    assert_eq!(items[2].as_ref().unwrap().text, "Mary follows Henry Ashby, who died at 71 and is buried at Kirkby. The Ashbys are led from Fenmere.");
    // a rival's, the chronicle off, or nobody buried: as filed
    for (P, family, b) in [(game_params(), 2, Some(burial("Fenmere", true))), (default_params(), 0, Some(burial("Fenmere", true))), (game_params(), 0, None)] {
        let mut items = year("Fenmere");
        chronicle::succession(&P, family, &mut items, "Mary", b.as_ref());
        assert_eq!(items[2].as_ref().unwrap().text, "The Ashbys are led from Fenmere.");
        assert_eq!(items[1].as_ref().unwrap().weight, game_params().chronicle.own_great);
    }
}

/// The decades are the calendar's whatever year a run starts in: from 1755 the
/// first decade is 1755 to 1759, and the 1760s are a decade of their own.
#[test]
fn a_run_from_1755_ranks_by_the_calendars_decades() {
    let P = game_params();
    let kinds = ["crossing", "steep", "milestone", "town", "region"];
    // five heavy items of five kinds in the late 1750s, and five of the same kinds in the early 1760s
    let mut ev: Vec<Event> = kinds.iter().enumerate().map(|(i, k)| item(1755.5 + 0.8 * i as f64, k, 0.9 - 0.01 * i as f64, &format!("{k} in the 1750s"))).collect();
    ev.extend(kinds.iter().enumerate().map(|(i, k)| item(1760.5 + 0.6 * i as f64, k, 0.9 - 0.01 * i as f64, &format!("{k} in the 1760s"))));
    chronicle::rank(&mut ev, &P, 1755, 1900, 1900);
    let kept = kept_titles(&ev);
    assert_eq!(kept.iter().filter(|t| t.ends_with("1750s")).count(), P.chronicle.per_decade, "{kept:?}");
    assert_eq!(kept.iter().filter(|t| t.ends_with("1760s")).count(), P.chronicle.per_decade, "{kept:?}");
    // the short first decade is over at the end of 1759, not five years later
    let mut thin = vec![item(1756.6, "famine", 0.5, "hunger"), item(1757.9, "first", 0.5, "first")];
    chronicle::rank(&mut thin, &P, 1755, 1900, 1758);
    assert!(kept_titles(&thin).is_empty());
    chronicle::rank(&mut thin, &P, 1755, 1900, 1759);
    assert_eq!(kept_titles(&thin), ["hunger", "first"]);
    // and from 1750 nothing has moved: 1900 still goes with the 1890s
    let mut last = vec![item(1895.5, "discovery", 0.9, "a"), item(1900.5, "discovery", 0.9, "b"), item(1900.6, "discovery", 0.9, "c")];
    chronicle::rank(&mut last, &P, 1755, 1900, 1900);
    assert_eq!(kept_titles(&last), ["a", "b"]);
}

/// The same for the items a decade's end writes: told at the end of a calendar
/// decade, named for it, and counting from the first year recorded.
#[test]
fn a_run_from_1755_tells_its_decades_by_the_calendar() {
    let mut C = game_content();
    C.start_year = 1755;
    let r = sim::run_with(&game(0, 1790), C, false, true).expect("money is conserved every month");
    let of_decade: Vec<&Event> = r.events.iter().filter(|e| e.title.contains("fastest-growing town of the") || e.title.contains(" new towns in the ")).collect();
    assert!(!of_decade.is_empty(), "no decade's end was told in 1755 to 1790");
    for e in &of_decade {
        let year = e.year.floor() as i64;
        assert_eq!(year % 10, 9, "{} is told in {year}", e.title);
        assert!(e.title.ends_with(&format!("{}s", year / 10 * 10)), "{} is told in {year}", e.title);
        if e.kind == "town" {
            let from = (year / 10 * 10).max(1755);
            let note = &e.why.as_ref().expect("why it is news").note;
            assert!(note.contains(&format!("founded from {from} to {year}")), "{note}");
        }
    }
}

#[test]
fn every_town_has_its_milestones() {
    let P = game_params();
    let mut st = State { T_max: 3, ..State::default() };
    st.town_active = vec![true; 3];
    st.town_name = vec!["Ashby".to_string(), "Kelgate".to_string(), "Norford".to_string()];
    st.town_node = vec![10, 20, 30];
    st.urban_pop = vec![12_000.0, 4_000.0, 9_000.0];
    let mut events = Vec::new();
    // the first year only notes who is already past a size
    chronicle::city_milestones(&mut st, &P, 1750, &mut events, 1750);
    assert!(events.is_empty());
    st.urban_pop = vec![26_000.0, 4_000.0, 11_000.0];
    chronicle::city_milestones(&mut st, &P, 1790, &mut events, 1750);
    st.urban_pop = vec![26_000.0, 10_500.0, 11_000.0];
    chronicle::city_milestones(&mut st, &P, 1801, &mut events, 1750);
    chronicle::city_milestones(&mut st, &P, 1802, &mut events, 1750);
    let titles: Vec<&str> = events.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["Ashby passes 25,000 people", "Norford passes 10,000 people", "Kelgate passes 10,000 people"]);
    // the port's weight for the first town past 25,000: 0.55 + 0.05
    assert!(events[0].text.contains("first town in the country") && (events[0].weight - 0.6).abs() < 1e-12);
    assert_eq!(events[1].text, "Norford is the second town on the island with more than 10,000 inhabitants, after Ashby.");
    assert_eq!(events[2].text, "Kelgate is the third town on the island with more than 10,000 inhabitants, after Ashby and Norford (1790).");
    assert!(events[1].weight < 0.6 && events[2].town == Some(1) && events[2].cell == Some(20));
    assert!(events.iter().all(|e| e.why.as_ref().is_some_and(|w| w.value >= w.threshold)));
    // past the tenth the place is a figure, with the ending it takes in English
    let mut st = State { T_max: 23, ..State::default() };
    st.town_active = vec![true; 23];
    st.town_name = (0..23).map(|t| format!("Town{t}")).collect();
    st.town_node = (0..23).collect();
    st.urban_pop = (0..23).map(|t| if t < 20 { 12_000.0 } else { 9_000.0 }).collect();
    let mut events = Vec::new();
    chronicle::city_milestones(&mut st, &P, 1750, &mut events, 1750);
    for t in 20..23 {
        st.urban_pop[t] = 10_500.0;
        chronicle::city_milestones(&mut st, &P, 1760 + t as i64, &mut events, 1750);
    }
    let places: Vec<&str> = events.iter().map(|e| e.text.split(' ').nth(3).unwrap()).collect();
    assert_eq!(places, ["21st", "22nd", "23rd"]);
    // and in the whole game the first town past a size is not the only one told
    assert!(whole_game().events.iter().any(|e| e.kind == "milestone" && e.text.contains("second town on the island")));
}

#[test]
fn a_first_of_its_kind_is_the_first_ever() {
    let (off, on) = (default_params(), game_params());
    let mut st = State::default();
    assert!(chronicle::first_of_kind(&st, &on, 2, 5), "nothing held yet");
    // the family held a works of method 5 once, and has sold it
    st.chron.kinds.insert((2, 5));
    assert!(!chronicle::first_of_kind(&st, &on, 2, 5), "the chronicle remembers");
    assert!(chronicle::first_of_kind(&st, &on, 2, 6) && chronicle::first_of_kind(&st, &on, 3, 5));
    assert!(chronicle::first_of_kind(&st, &off, 2, 5), "the port asks only what is owned today");
    // the works the families start with are remembered from the first day
    let s = sim::start(&game(6, 1751), game_content());
    let held = (0..s.st.F_count).filter(|&f| s.st.f_owner[f] >= 0).count();
    assert!(held > 0 && !s.st.chron.kinds.is_empty());
    for f in (0..s.st.F_count).filter(|&f| s.st.f_owner[f] >= 0) {
        assert!(s.st.chron.kinds.contains(&(s.st.f_owner[f] as usize, s.st.f_method[f])));
    }
}

#[test]
fn a_doubling_is_measured_from_the_first_years() {
    let (off, on) = (default_params(), game_params());
    // seven years of a series that starts low: 1750 alone would be a poor baseline
    let mut series = vec![50.0; 12];
    series.extend(vec![100.0; 12 * 6]);
    assert_eq!(chronicle::baseline(&off, &series, 1750, 1753, 1750, 50.0), Some(("1750".to_string(), 50.0)), "the port keeps its first year");
    assert_eq!(chronicle::baseline(&on, &series, 1750, 1753, 1750, 50.0), None, "nothing doubles while the baseline forms");
    assert_eq!(chronicle::baseline(&on, &series, 1750, 1755, 1750, 50.0), Some(("1750\u{2013}54".to_string(), 90.0)));
}

#[test]
fn a_run_without_families_has_its_history_too() {
    // seed 2 with nobody playing, the whole run. (It once stopped in mid-century:
    // a coal boom looked for its busiest pit in last month's output, which no
    // longer lines up with the pits once the year's land has been shared out.)
    let mut P = game_params();
    P.seed = 2;
    let r = sim::run_with(&P, game_content(), false, true).expect("the run finishes and money is conserved");
    assert!(r.state.chron.kinds.is_empty() && r.events.iter().all(|e| e.kind != "family"), "no families, none of their news");
    let booms: Vec<&Event> = r.events.iter().filter(|e| e.title.starts_with("Coal boom around")).collect();
    assert!(!booms.is_empty(), "seed 2 has a coal boom");
    for e in &booms {
        assert!(e.cell.is_some_and(|c| c < r.state.N) && e.why.is_some(), "{}", e.title);
    }
    let mut by_decade: BTreeMap<i64, usize> = BTreeMap::new();
    for e in r.events.iter().filter(|e| e.kept == Some(true)) {
        *by_decade.entry(decade(e)).or_insert(0) += 1;
    }
    let Q = &r.params.chronicle;
    assert_eq!(by_decade.len(), 15);
    assert!(by_decade.values().all(|n| (Q.at_least..=Q.per_decade).contains(n)), "{by_decade:?}");
}

#[test]
fn a_founding_redraws_only_the_districts_it_was_cut_from() {
    // fifty years of seed 7, in which most of its towns are founded. (Forty were
    // enough until the canals were merged beside the mines, docs/rules/canals.md:
    // with both the island founds no town from 1772 to 1790, the years its canals
    // are dug and take carting a new market would have saved, and its fifth town
    // comes in 1791, not 1780. By 1810 it has founded nine, with canals or without.)
    let mut s = sim::start(&game(6, 1800), game_content());
    let T = s.st.T_max;
    let (mut foundings, mut spared, mut carried) = (0, 0, 0);
    let mut before = vec![1750; T];
    while s.year <= s.end_year {
        let year = s.year;
        let told = s.events.len();
        sim::run_year(&mut s).unwrap();
        let st = &s.st;
        let new: Vec<bool> = (0..T).map(|t| st.town_active[t] && st.town_founded[t] == year).collect();
        // the towns a new one took ground from: where its cells looked the year before
        let mut cut = new.clone();
        for c in 0..st.N {
            if st.catch[c] >= 0 && new[st.catch[c] as usize] && st.prev_catch[c] >= 0 {
                cut[st.prev_catch[c] as usize] = true;
            }
        }
        // and, under the game's rules for people, the towns a tenth of whose people
        // came or went with their cells this year, founding or no founding
        for t in 0..T {
            if st.town_active[t] && st.pp_moved[t] >= chronicle::REDRAWN_SHARE && !cut[t] {
                cut[t] = true;
                carried += 1;
            }
        }
        for t in 0..T {
            if cut[t] {
                assert_eq!(st.chron.redrawn[t], year, "{} was redrawn in {year}", st.town_name[t]);
                assert_eq!(st.chron.peak[t], (year, st.pop[t]), "and its peak starts again");
            } else {
                assert_eq!(st.chron.redrawn[t], before[t], "town {t} was not touched in {year}");
            }
        }
        if new.contains(&true) {
            foundings += 1;
            assert!((0..T).any(|t| cut[t] && !new[t]), "the new town of {year} was cut out of somebody's district");
            spared += (0..T).filter(|&t| st.town_active[t] && !cut[t]).count();
        }
        before = st.chron.redrawn.clone();
        // no story sets a town against what it was before its district was redrawn
        let Q = &s.params.chronicle;
        for e in s.events[told..].iter().filter(|e| e.kind == "region") {
            let t = e.town.unwrap();
            let since = st.chron.redrawn[t];
            if e.title.starts_with("Coal boom") {
                assert!(year - Q.boom_years >= since, "{} in {year}, redrawn in {since}", e.title);
            } else if e.title.contains("fastest-growing") {
                assert!(year - 9 >= since, "{} in {year}, redrawn in {since}", e.title);
            } else if e.title.ends_with("in decline") {
                assert!(st.chron.peak[t].0 >= since && year - st.chron.peak[t].0 >= Q.decline_years, "{} in {year}, redrawn in {since}", e.title);
            }
        }
    }
    assert!(foundings >= 5 && spared > foundings, "{foundings} foundings, and towns they did not touch ({spared})");
    assert!(carried > 0, "and a district that changed market with no founding redrew a town");
}

#[test]
fn towns_and_districts_have_stories() {
    let r = whole_game();
    let stories: Vec<&Event> = r.events.iter().filter(|e| e.kind == "region").collect();
    assert!((10..=80).contains(&stories.len()), "{} regional stories", stories.len());
    for e in &stories {
        // each with a place, a year and a reason
        let t = e.town.expect("a town");
        assert!(e.cell.is_some() && r.state.town_active[t] && e.title.contains(&r.state.town_name[t]), "{}", e.title);
        assert!(e.text.split(". ").count() >= 2, "{} gives a reason: {}", e.title, e.text);
        assert!(e.weight >= 0.6 && e.why.is_some());
    }
    let tests: std::collections::BTreeSet<&str> = stories.iter().map(|e| e.why.as_ref().unwrap().test.as_str()).collect();
    for t in ["coal raised against a few years before", "people against the town's own peak", "one town's people against another's", "the decade's fastest-growing town", "works closed in one district in a year"] {
        assert!(tests.contains(t), "seed 7 has a story of the kind: {t}");
    }
    // under the game's railways (docs/rules/railways.md) a line with no second town
    // may run to farms, woods or works: a coal boom names the line that reached the
    // town or a branch to its collieries, and nothing is called a mineral line
    if r.params.railways.enabled {
        assert!(r.events.iter().all(|e| !e.text.contains("mineral line")), "no branch is told as a mineral line");
        let lines = &r.state.rw.lines;
        for e in stories.iter().filter(|e| e.text.contains("brings it down from the pits")) {
            let (t, year) = (e.town.unwrap(), e.year.floor());
            assert!(e.text.contains("A branch line to the collieries, opened in "), "{}", e.text);
            assert!(lines.iter().any(|l| l.a == t && l.opened < year + 1.0 && ways::serves_word(l.serves) == "collieries" && e.text.contains(&format!("opened in {},", l.opened.floor()))), "{}", e.text);
        }
    }
    // a founding somewhere does not silence the rest of the island: seed 7's last
    // town is Ruthdale, in 1831 (it was Golddale, in 1783, before carting and the
    // game's railways), and towns it did not touch have stories before then
    let last_founding = (0..r.state.T_max).filter(|&t| r.state.town_active[t]).map(|t| r.state.town_founded[t]).max().unwrap();
    assert!(stories.iter().any(|e| !e.title.contains("works close") && (e.year.floor() as i64) < last_founding), "a story of a town's people before {last_founding}");
    // but no town is said to decline from the people it was founded with
    for e in stories.iter().filter(|e| e.title.ends_with("in decline")) {
        let note = &e.why.as_ref().unwrap().note;
        let peak_year: i64 = note.split("the peak of ").nth(1).and_then(|s| s.get(..4)).and_then(|y| y.parse().ok()).unwrap_or_else(|| panic!("a year in: {note}"));
        assert!(peak_year > r.state.town_founded[e.town.unwrap()], "{}: {}", e.title, e.text);
    }
    // the same town is not said to decline twice within a few years
    let mut told: BTreeMap<&str, f64> = BTreeMap::new();
    for e in stories.iter().filter(|e| e.title.ends_with("in decline")) {
        if let Some(last) = told.insert(e.title.as_str(), e.year) {
            assert!(e.year - last >= r.params.chronicle.decline_years as f64, "{} twice", e.title);
        }
    }
    // a decade of new towns is one item, with each town's year, and it is kept
    // unless the decade is full of heavier things. With the life cycle for works
    // (docs/rules/works.md) seed 7's towns come later: three in the 1760s, where
    // two inventions, a coal boom and the family's new trade outweigh them.
    // (Before it the digest was of the 1750s and was one of that decade's four.)
    // With carting and the game's railways merged (docs/rules/carting.md) the
    // digest is of the 1770s, three towns again, where two inventions, coke
    // iron's crossing and the family's new trade outweigh it.
    // With kept town ground merged (docs/rules/towns.md) it is of the 1750s
    // again, three towns: coal's crossing, an iron glut and a coal boom round
    // Granness outweigh it, and the decade's fourth place is the player's, which
    // `chronicle::rank` keeps for the family's heaviest item whatever else
    // happened (here the Fairfaxes setting up in building materials, the lighter
    // of the two). So the player's place is left out of "all heavier" below.
    // With the mines as workings (docs/rules/mines.md) the towns of a game move
    // again (before kept town ground was merged beside them, seed 7 founded two
    // a decade at most and had no digest at all), so the test no longer takes
    // one for granted: it holds that a digest is filed for exactly the decades
    // that have the towns for one, and checks whatever digests there are.
    digests_of_new_towns(r);
    // So that the digest itself is still read by a test, a second island: the
    // first fifty years of seed 8, which founds three towns in the 1750s and
    // three in the 1790s with the mines as workings and kept town ground.
    let mut P = game(6, 1800);
    P.seed = 8;
    let early = sim::run_with(&P, game_content(), false, true).expect("money is conserved every month");
    assert!(digests_of_new_towns(&early) >= 1, "seed 8 has a decade of new towns before 1800");
}

/// A decade of new towns is one item, filed for exactly the decades of the
/// eighteenth century that have the towns for one, with each town's year, and
/// kept unless the decade is full of heavier things (the player's place aside).
/// Gives the number of digests there are.
fn digests_of_new_towns(r: &RunResult) -> usize {
    let digests: Vec<&Event> = r.events.iter().filter(|e| e.kind == "town" && e.title.contains("new towns in the 17")).collect();
    let min = r.params.chronicle.new_towns_min.max(1);
    let founded_in = |d: i64| (0..r.state.T_max).filter(|&t| r.state.town_active[t] && (1750 + 10 * d..1760 + 10 * d).contains(&r.state.town_founded[t])).count();
    let due: std::collections::BTreeSet<i64> = (0..5).filter(|&d| founded_in(d) >= min).collect();
    assert_eq!(digests.iter().map(|e| decade(e)).collect::<std::collections::BTreeSet<i64>>(), due, "a digest for each decade of new towns, and no other");
    let mut decades = std::collections::BTreeSet::new();
    for digest in &digests {
        assert!(decades.insert(decade(digest)), "one digest a decade: {}", digest.title);
        let years = format!("({}", 175 + decade(digest));
        assert!(digest.text.contains(&years) && digest.why.is_some(), "{}: {}", digest.title, digest.text);
        let kept: Vec<&Event> = r.events.iter().filter(|e| e.kept == Some(true) && decade(e) == decade(digest)).collect();
        // the player's place: the heaviest of the family's own items that were kept, the earlier of equals
        let own = kept.iter().enumerate().filter(|(_, e)| class(e) == "own").fold(None, |best: Option<usize>, (k, e)| match best {
            Some(b) if kept[b].weight >= e.weight => Some(b),
            _ => Some(k),
        });
        let outweighed = kept.len() == r.params.chronicle.per_decade && kept.iter().enumerate().all(|(k, e)| Some(k) == own || e.weight > digest.weight);
        assert!(digest.kept == Some(true) || outweighed, "{} is kept, or the decade's {} are all heavier, the player's place aside", digest.title, kept.len());
    }
    digests.len()
}
