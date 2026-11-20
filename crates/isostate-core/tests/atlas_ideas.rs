//! The atlas's third plate (docs/atlas.md, "The third plate: the ideas") and
//! the pin it gives the lab (docs/lab.md): every idea of the data is on the
//! page and none is left out silently, the page fetches nothing, a pin finds
//! its idea in its year (or holds it back until then), and with no pin a run
//! is the run.
//!
//! The cases are small islands (48 by 36 cells, no families) under the game's
//! rules, played to 1780 or 1795. The page's test leans on no rule's history:
//! it reads the names it checks from the content the run was played on. The
//! pins' tests name ideas of the data (the coke furnace and improved husbandry
//! of the port's content, the hoist of the tasks rule), and hold whatever the
//! island would have done without them.
#![allow(non_snake_case)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use isostate_core::atlas;
use isostate_core::atlas_ideas;
use isostate_core::edits::{self, Dated, Edit, PinWhen};
use isostate_core::params::{Params, Rules};
use serde_json::Value;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("isostate-atlas-ideas-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().copied().map(String::from).collect()
}

/// A small island under the game's rules, to `end`.
fn small(seed: u64, end: i64) -> Params {
    let mut P = Rules::Game.params();
    P.seed = seed;
    P.W = 48;
    P.H = 36;
    P.n_families = 0;
    P.end_year = Some(end);
    P
}

fn pin(idea: &str, year: i64, when: PinWhen) -> Dated {
    Dated { before: (1750, 1), edit: Edit::Pin { idea: idea.to_string(), year, when } }
}

#[test]
fn every_idea_of_the_data_is_on_the_page_and_the_page_fetches_nothing() {
    let dir = scratch("small");
    let args = strings(&["ideas", "--seeds", "3,4", "--families", "0", "--end-year", "1780", "--set", "W=48", "--set", "H=36", "--quiet", "--jobs", "2", "--out", dir.to_str().unwrap()]);
    atlas::command(&args, false).expect("the plate runs");
    let html = std::fs::read_to_string(dir.join("ideas.html")).unwrap();
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("ideas.json")).unwrap()).unwrap();
    let text = std::fs::read_to_string(dir.join("ideas.txt")).unwrap();

    assert!(!html.contains("/*ATLAS_DATA*/null"), "the figures are in the page");
    for bad in ["src=", "<link", "@import", "url(", "fetch(", "XMLHttpRequest", "<iframe"] {
        assert!(!html.contains(bad), "the page must open without the network, and it has {bad:?}");
    }
    // the header names the folder, not the machine's path to it
    let command = doc["command"].as_str().unwrap();
    assert!(command.contains("--out isostate-atlas-ideas-test-") && !command.contains(std::env::temp_dir().to_str().unwrap()), "{command}");

    // every idea of the content the runs were played on: the tree's not known in
    // 1750 are rows, those known are named, and every use of the tasks rule is a row
    let o = atlas_ideas::parse_args(&args).unwrap();
    let (P, content, _) = atlas_ideas::plan(&o).unwrap();
    let C = isostate_core::tree::content(&P, content).unwrap();
    let rows = doc["rows"].as_array().unwrap();
    let names = |kind: &str| -> Vec<String> { rows.iter().filter(|r| r["kind"] == kind).map(|r| r["name"].as_str().unwrap().to_string()).collect() };
    let tree: Vec<String> = names("method").into_iter().chain(names("technique")).collect();
    let unknown: Vec<String> = (0..C.NT).filter(|&t| !C.tech_known[t]).map(|t| C.tech_name[t].clone()).collect();
    assert_eq!(tree.iter().collect::<BTreeSet<_>>(), unknown.iter().collect::<BTreeSet<_>>(), "every idea of the tree not known in 1750 is a row, once");
    assert_eq!(tree.len(), unknown.len());
    let uses: Vec<String> = C.tasks.uses.iter().map(|u| u.name.clone()).collect();
    assert!(!uses.is_empty());
    assert_eq!(names("use"), uses, "every use of the tasks rule is a row");
    let known: Vec<String> = (0..C.NT).filter(|&t| C.tech_known[t]).map(|t| C.tech_label[t].clone()).collect();
    assert_eq!(doc["known"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect::<Vec<_>>(), known, "those known in 1750 are named");
    assert_eq!(doc["order"].as_array().unwrap().len(), rows.len(), "every row is drawn");
    // history's year is the data's own target, and a row with none says so
    for r in rows {
        let name = r["name"].as_str().unwrap();
        let target = match r["kind"].as_str().unwrap() {
            "use" => C.tasks.uses.iter().find(|u| u.name == name).unwrap().target,
            _ => C.tech_target.get(C.tech_name.iter().position(|n| n == name).unwrap()).copied().filter(|&y| y > 0),
        };
        assert_eq!(r["target"].as_i64(), target, "{name}");
        assert_eq!(r["years"].as_array().unwrap().len(), 2, "a year (or none) for each island");
        assert!(!r["on"].as_str().unwrap().is_empty(), "{name}: what it is found on is said");
        if target.is_none() {
            assert!(text.lines().any(|l| l.contains(&format!("{} (", r["label"].as_str().unwrap())) && l.contains(" none ")), "{name} has no year and the table does not say so");
        }
    }
    // each island's towns, with a year (or none) for each idea of the tree
    for isl in doc["islands"].as_array().unwrap() {
        let towns = isl["towns"].as_array().unwrap();
        assert!(!towns.is_empty());
        for t in towns {
            assert_eq!(t["knew"].as_array().unwrap().len(), C.NT);
        }
        // a town that found an idea knew it from the year it was found
        for (t, y) in isl["found"].as_array().unwrap().iter().enumerate() {
            if let (Some(y), Some(town)) = (y.as_i64(), isl["town"][t].as_u64()) {
                let tw = towns.iter().find(|x| x["t"].as_u64() == Some(town)).expect("the town that found it is drawn");
                assert_eq!(tw["knew"][t].as_i64(), Some(y), "{}", C.tech_name[t]);
            }
        }
    }

    // --from draws the same page again, and --against lays another folder under it
    let again = scratch("again");
    atlas::command(&strings(&["ideas", "--from", dir.to_str().unwrap(), "--out", again.to_str().unwrap(), "--quiet"]), false).unwrap();
    assert_eq!(std::fs::read_to_string(again.join("ideas.html")).unwrap(), html);
    atlas::command(&strings(&["ideas", "--from", dir.to_str().unwrap(), "--out", again.to_str().unwrap(), "--against", dir.to_str().unwrap(), "--quiet"]), false).unwrap();
    let laid = std::fs::read_to_string(again.join("ideas.html")).unwrap();
    assert!(laid.contains("\"against\":{") && !laid.contains("\"against\":null"));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&again);
}

#[test]
fn a_pin_finds_its_idea_in_its_year_and_a_hold_holds_it_back() {
    let P = small(3, 1795);
    let C = Rules::Game.content();
    let plain = edits::play(&P, C.clone(), &[]).unwrap();
    let t = |s: &isostate_core::sim::Sim, name: &str| s.content.tech_name.iter().position(|n| n == name).unwrap();
    let (coke, husbandry) = (t(&plain, "coke_furnace"), t(&plain, "improved_husbandry"));
    let hoist = plain.content.tasks.uses.iter().position(|u| u.name == "hoist").unwrap();
    // without pins: the island finds improved husbandry before 1790, and not the hoist (from 1901)
    let natural = plain.st.tech_year[husbandry];
    assert!((1750..1790).contains(&natural), "improved husbandry was found in {natural} on this island");
    assert!(plain.st.tasks.found[hoist] < 0);

    let list = [pin("coke_furnace", 1760, PinWhen::In), pin("improved_husbandry", 1790, PinWhen::NotBefore), pin("hoist", 1770, PinWhen::In)];
    let s = edits::play(&P, C, &list).unwrap();
    assert_eq!(s.st.tech_year[coke], 1760, "pinned to 1760, found in 1760");
    let town = s.st.tech_town[coke];
    assert!(town >= 0 && s.st.know[(town as usize, coke)] >= 0.25, "found in a town, which knows it");
    assert!(s.events.iter().any(|e| e.kind == "discovery" && e.tech == Some(coke) && e.title.contains("pinned")), "and the news says so");
    let held = s.st.tech_year[husbandry];
    assert!(held < 0 || held >= 1790, "held back until 1790, and found in {held}");
    assert_eq!(s.st.tasks.found[hoist], 1770, "a use is pinned as an idea of the tree is");
    assert_eq!(s.st.by_hand.pins.len(), 3);
    // a pinned idea still takes its yearly number, so the island's draws are its own
    // until the pin first changes a find: every other idea found before then is found
    // in the same year
    let coke_natural = if plain.st.tech_year[coke] < 0 { i64::MAX } else { plain.st.tech_year[coke] };
    let first_change = coke_natural.min(1760).min(natural);
    for k in (0..s.content.NT).filter(|&k| k != coke && k != husbandry && !s.content.tech_known[k]) {
        let y = plain.st.tech_year[k];
        if y >= 0 && y < first_change {
            assert_eq!(s.st.tech_year[k], y, "{} moved, though found before the pins changed anything", s.content.tech_name[k]);
        }
    }
}

#[test]
fn a_pin_is_refused_for_no_such_idea_one_known_or_found_and_a_year_gone() {
    let P = small(3, 1775);
    let C = Rules::Game.content();
    let refused = |list: &[Dated]| edits::play(&P, C.clone(), list).map(|_| ()).unwrap_err();
    assert!(refused(&[pin("no_such_idea", 1760, PinWhen::In)]).contains("there is no idea called"));
    assert!(refused(&[pin("smithy", 1760, PinWhen::In)]).contains("known before the island's history began"));
    let late = Dated { before: (1770, 1), edit: Edit::Pin { idea: "coke_furnace".into(), year: 1765, when: PinWhen::In } };
    assert!(refused(&[late]).contains("cannot be found in 1765"));
    let twice = [pin("coke_furnace", 1760, PinWhen::In), Dated { before: (1765, 1), edit: Edit::Pin { idea: "coke_furnace".into(), year: 1770, when: PinWhen::In } }];
    assert!(refused(&twice).contains("was found already, in 1760"));
    // the form a list of edits writes it in
    let line = serde_json::to_string(&pin("coal_power_station", 1888, PinWhen::In)).unwrap();
    assert_eq!(line, r#"{"before":[1750,1],"edit":{"what":"pin","idea":"coal_power_station","year":1888,"when":"in"}}"#);
    let read: Dated = serde_json::from_str(r#"{"before":[1750,1],"edit":{"what":"pin","idea":"x","year":1900}}"#).unwrap();
    assert_eq!(read.edit, Edit::Pin { idea: "x".into(), year: 1900, when: PinWhen::In }, "\"when\" may be left out");
}

#[test]
fn with_no_pin_the_run_is_the_run() {
    // the hooks ask after pins only when one is given: an island played with no
    // edits is the plain run's to the bit, ideas, towns and every observer series
    let P = small(4, 1790);
    let C = Rules::Game.content();
    let plain = isostate_core::sim::run_with(&P, C.clone(), false, true).unwrap();
    let played = edits::play(&P, C.clone(), &[]).unwrap();
    assert!(played.st.by_hand.pins.is_empty());
    assert_eq!(plain.state.tech_year, played.st.tech_year);
    assert_eq!(plain.state.tech_town, played.st.tech_town);
    assert_eq!(plain.state.tasks.found, played.st.tasks.found);
    let bits = |d: &[Vec<f64>]| d.iter().map(|v| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>()).collect::<Vec<_>>();
    assert_eq!(bits(&plain.state.series.data), bits(&played.st.series.data));
    // and a pin that changes nothing the island would not have done (a hold to a year
    // before it is found anyway) leaves the ideas where they were
    let husbandry = C.tech_name.iter().position(|n| n == "improved_husbandry");
    if let Some(h) = husbandry {
        let y = plain.state.tech_year[h];
        if y > 1751 {
            let s = edits::play(&P, C, &[pin("improved_husbandry", 1751, PinWhen::NotBefore)]).unwrap();
            assert_eq!(s.st.tech_year[h], y, "a hold that ends before the find moves nothing");
        }
    }
}

#[test]
fn the_command_line_is_checked_before_anything_runs() {
    let o = atlas_ideas::parse_args(&strings(&["ideas", "--out", "x"])).unwrap();
    assert_eq!((o.seeds.clone(), o.rules, o.families, o.end_year), ((1..=16).collect::<Vec<u64>>(), Rules::Game, 6, 2050));
    assert!(atlas_ideas::parse_args(&strings(&["ideas"])).unwrap_err().contains("--out is required"));
    assert!(atlas_ideas::parse_args(&strings(&["ideas", "--out", "x", "--pin", "coal_power_station"])).unwrap_err().contains("NAME@YEAR"));
    let o = atlas_ideas::parse_args(&strings(&["ideas", "--out", "x", "--pin", "no_such_idea@1900"])).unwrap();
    assert!(atlas_ideas::plan(&o).unwrap_err().contains("no such idea"));
    let o = atlas_ideas::parse_args(&strings(&["ideas", "--out", "x", "--pin", "coal_power_station@1888", "--hold", "office_computing@1970"])).unwrap();
    let (_, _, list) = atlas_ideas::plan(&o).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[1].edit, Edit::Pin { idea: "office_computing".into(), year: 1970, when: PinWhen::NotBefore });
    // the first plate's header names the folder only, too
    let cmd = atlas::command_line(&strings(&["making", "--out", "C:/somewhere/deep/M1", "--seeds", "1-8"]));
    assert_eq!(cmd, "isostate-core atlas making --out M1 --seeds 1-8");
    // the lab takes the same pins, and checks their names before any run
    let o = isostate_core::lab::parse_args(&strings(&["--out", "unused", "--pin", "coal_power_station@1888", "--pin", "electrical_works@1885"])).unwrap();
    let plan = isostate_core::lab::plan(&o).unwrap();
    assert_eq!(plan.arms.len(), 2);
    assert_eq!(plan.arms[1].label, "coal_power_station pinned to 1888 + electrical_works pinned to 1885");
    assert_eq!(plan.arms[1].edits.len(), 2);
    let o = isostate_core::lab::parse_args(&strings(&["--out", "unused", "--pin", "no_such_idea@1888"])).unwrap();
    assert!(isostate_core::lab::plan(&o).unwrap_err().contains("no such idea"));
}
