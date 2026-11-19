//! The atlas's first plate (docs/atlas.md): the page is made for a small
//! island, every good, kind of works, use of the land and need of the content
//! the run was played on is drawn, every line the content names is drawn, what
//! the drawing leaves out is said and not dropped, the page fetches nothing,
//! and `--from` draws the same page again.
//!
//! The case is one small island (48 by 36 cells, no families) under the game's
//! rules to 1765. It leans on no rule's history: it holds whatever content and
//! rules are on, since it reads the names it checks from the content the run
//! was played on.
#![allow(non_snake_case)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use isostate_core::atlas::{self, Keys};
use isostate_core::params::Rules;
use serde_json::Value;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("isostate-atlas-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().copied().map(String::from).collect()
}

const SMALL: [&str; 14] = ["making", "--seeds", "3", "--families", "0", "--end-year", "1765", "--set", "W=48", "--set", "H=36", "--quiet", "--jobs", "1"];

#[test]
fn the_page_is_made_for_a_small_island_and_draws_everything() {
    let dir = scratch("small");
    let mut args = strings(&SMALL);
    args.extend(strings(&["--out", dir.to_str().unwrap()]));
    atlas::command(&args, false).expect("the atlas runs");
    let html = std::fs::read_to_string(dir.join("making.html")).unwrap();
    let json = std::fs::read_to_string(dir.join("making.json")).unwrap();
    let text = std::fs::read_to_string(dir.join("making.txt")).unwrap();
    let doc: Value = serde_json::from_str(&json).expect("the figures are JSON");

    // the page carries its figures and fetches nothing
    assert!(!html.contains("/*ATLAS_DATA*/null"), "the figures are in the page");
    for bad in ["src=", "<link", "@import", "url(", "fetch(", "XMLHttpRequest", "<iframe"] {
        assert!(!html.contains(bad), "the page must open without the network, and it has {bad:?}");
    }

    // the content the run was played on: the game's overlay over the port's file
    let o = atlas::parse_args(&args).unwrap();
    let (P, content) = atlas::params(&o).unwrap();
    let C = isostate_core::tree::content(&P, content).unwrap();
    let nodes = doc["nodes"].as_array().unwrap();
    let names = |kind: &str| -> Vec<String> { nodes.iter().filter(|n| n["k"] == kind).map(|n| n["name"].as_str().unwrap().to_string()).collect() };
    assert_eq!(names("good"), C.good_name, "every good is drawn, once, in the content's order");
    assert_eq!(names("works"), C.method_name, "every kind of works is drawn, once");
    assert_eq!(names("sector"), C.sector_name, "every use of the land is drawn");
    assert_eq!(names("need"), C.service_name, "every need is drawn");
    assert_eq!(names("ways").len(), 1);
    assert_eq!(doc["goods"].as_array().unwrap().len(), C.G);
    assert_eq!(doc["methods"].as_array().unwrap().len(), C.M);
    // each node has a place of its own on the page
    let (w, h) = (doc["width"].as_f64().unwrap(), doc["height"].as_f64().unwrap());
    let mut seen = BTreeSet::new();
    for n in nodes {
        let (x, y) = (n["x"].as_f64().unwrap(), n["y"].as_f64().unwrap());
        assert!(x > 0.0 && x < w && y > 0.0 && y < h, "{n} is off the page");
        assert!(seen.insert((x.to_bits(), y.to_bits())), "{n} stands on another node");
    }

    // every line the content names is drawn: the layout's lines hold every content key
    let L = atlas::layout(&C, &BTreeSet::new());
    let drawn: BTreeSet<usize> = L.key_of_edge.iter().copied().collect();
    let named = atlas::content_keys(&C);
    assert!(named.is_subset(&drawn), "lines the content names and the drawing leaves out: {:?}", named.difference(&drawn).collect::<Vec<_>>());
    assert!(drawn.iter().all(|&k| k < Keys::of(&C).len()));
    let edges = doc["edges"].as_array().unwrap();
    assert!(edges.len() >= named.len(), "the page has {} lines for the content's {}", edges.len(), named.len());
    for e in edges {
        let e: Vec<u64> = e.as_array().unwrap().iter().map(|x| x.as_u64().unwrap()).collect();
        assert!((e[0] as usize) < nodes.len() && (e[1] as usize) < nodes.len() && e[2] < 4 && (e[3] as usize) < C.G, "line {e:?}");
    }
    // the island's figures: one entry per period, one line value per line
    let place = &doc["places"][0];
    assert_eq!(place["key"], "3");
    let per = &place["island"];
    let periods = doc["periods"].as_array().unwrap().len();
    assert_eq!(periods, 2, "1750s and 1760s");
    assert_eq!(per["edge"][0].as_array().unwrap().len(), edges.len());
    assert!(per["wage"][0].as_f64().unwrap() > 0.0);
    assert!(!place["towns"].as_array().unwrap().is_empty(), "the towns are kept for the picker");

    // recipe usage: a row for every kind of works, a cell for the island, and the idle named
    let usage = doc["usage"].as_array().unwrap();
    assert_eq!(usage.len(), C.M);
    let never: Vec<&str> = (0..C.M).filter(|&m| usage[m][0][0].is_null()).map(|m| C.method_label[m].as_str()).collect();
    assert!(!never.is_empty() && never.len() < C.M, "by 1765 some kinds run and some do not");
    let line = text.lines().find(|l| l.starts_with("Never run on any island")).expect("the idle part is named");
    for m in &never {
        assert!(line.contains(m), "{m} never ran and is not named: {line}");
    }
    // what the drawing could not share out is said, as a share of what it draws
    let lo = &doc["left_out"];
    let drawn_value = lo["drawn"].as_f64().unwrap();
    assert!(drawn_value > 0.0);
    for k in ["plant", "land", "households"] {
        let v = lo[k].as_f64().unwrap();
        assert!(v >= 0.0 && v < 0.01 * drawn_value, "{k}: {v} of {drawn_value} is left out");
    }
    assert!(text.contains("Left out of the drawing"));

    // --from draws the same page again, without playing
    let again = scratch("again");
    atlas::command(&strings(&["making", "--from", dir.to_str().unwrap(), "--out", again.to_str().unwrap(), "--quiet"]), false).unwrap();
    assert_eq!(std::fs::read_to_string(again.join("making.html")).unwrap(), html);
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&again);
}

#[test]
fn the_command_line_is_checked_before_anything_runs() {
    let e = atlas::parse_args(&strings(&["moving", "--out", "x"])).unwrap_err();
    assert!(e.contains("designed") && e.contains("not built"), "{e}");
    let e = atlas::parse_args(&strings(&["maps", "--out", "x"])).unwrap_err();
    assert!(e.contains("not a plate"), "{e}");
    assert!(atlas::parse_args(&strings(&["making"])).unwrap_err().contains("--out is required"));
    let e = atlas::parse_args(&strings(&["making", "--out", "x", "--set", "woods.enabled=true,false"])).unwrap_err();
    assert!(e.contains("one setting"), "{e}");
    // the port keeps no books, and the atlas reads them
    let o = atlas::parse_args(&strings(&["making", "--out", "x", "--rules", "port"])).unwrap();
    assert!(atlas::params(&o).unwrap_err().contains("books of account"));
    let o = atlas::parse_args(&strings(&["making", "--out", "x", "--set", "no_such_rule=1"])).unwrap();
    assert!(atlas::params(&o).is_err());
    let o = atlas::parse_args(&strings(&["making", "--out", "x"])).unwrap();
    assert_eq!((o.seeds.clone(), o.rules, o.families, o.end_year, o.step), ((1..=8).collect::<Vec<u64>>(), Rules::Game, 6, 2050, 10));
}

#[test]
fn every_kind_of_works_stands_right_of_the_land() {
    // the drawing is laid out from the data: the land on the left, the needs on the
    // right, and a kind of works to the right of every material it uses
    let P = Rules::Game.params();
    let C = isostate_core::tree::content(&P, Rules::Game.content()).unwrap();
    let L = atlas::layout(&C, &BTreeSet::new());
    let col = |i: usize| L.nodes[i].col;
    let last = L.cols - 1;
    for &s in &L.sector_node {
        assert_eq!(col(s), 0);
    }
    for &n in L.need_node.iter().chain([&L.ways_node]) {
        assert_eq!(col(n), last);
    }
    let (gd, md) = atlas::depths(&C);
    for m in 0..C.M {
        for g in (0..C.G).filter(|&g| C.m_in[(m, g)] > 0.0) {
            // a cycle of materials is cut where it is met, so a material may be as deep as its user
            assert!(md[m] >= gd[g], "{} at depth {} uses {} at depth {}", C.method_name[m], md[m], C.good_name[g], gd[g]);
        }
        assert!(col(L.works_node[m]) > 0 && col(L.works_node[m]) < last);
    }
}
