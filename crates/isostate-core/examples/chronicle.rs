//! The news as a reader meets it: the major feed of a game, decade by decade,
//! and the counts behind it.
//!
//!     cargo run -p isostate-core --release --example chronicle [--rules port|game] [--seed N | --seeds A-B]
//!         [--families N] [--end-year Y] [--why] [--all] [--counts]
//!
//! With one seed (7 unless `--seed` says otherwise) it prints the major feed:
//! every item the viewer's "major" filter would show, under its decade, with
//! its text. Under the game's rules those are the items the chronicle keeps
//! (`Event::kept`); under `--rules port` they are the items of weight 0.6 or
//! more, which is the feed as it was. `--why` adds each item's provenance,
//! `--all` lists every item with its weight, kept or not.
//!
//! `--counts`, or more than one seed (`--seeds 1-8`, run side by side), prints
//! the numbers instead: items per decade by kind, the major feed's length per
//! decade, and how many of the player's own items are major.
//!
//! Six families by default, the player's taking every default, as in
//! `examples/live.rs`. The argument parsing is that small.
#![allow(non_snake_case)]
use std::collections::{BTreeMap, BTreeSet};

use isostate_core::*;

/// What the viewer's major feed, timeline and toasts show.
fn major(e: &Event) -> bool {
    e.kept.unwrap_or(e.weight >= 0.6)
}

fn kind_label(kind: &str) -> &'static str {
    match kind {
        "discovery" => "Invention",
        "first" => "First output",
        "railway" => "Railway",
        "rail_plan" => "Railway scheme",
        "milestone" => "Milestone",
        "crossing" => "Turning point",
        "steep" => "Markets",
        "alarm" => "Hard times",
        "verdict" => "In hindsight",
        "famine" => "Hunger",
        "town" => "New town",
        "region" => "Town and country",
        "closure" => "Closure",
        "family" => "Families",
        _ => "News",
    }
}

fn main() {
    let mut rules = Rules::Game;
    let mut seeds: Vec<u64> = vec![7];
    let (mut families, mut end_year) = (6usize, None);
    let (mut show_why, mut all, mut counts) = (false, false, false);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = |name: &str| args.next().unwrap_or_else(|| panic!("{name} needs a value"));
        match a.as_str() {
            "--rules" => rules = Rules::parse(&value("--rules")).expect("--rules port or --rules game"),
            "--seed" => seeds = vec![value("--seed").parse().expect("a seed")],
            "--seeds" => {
                let v = value("--seeds");
                let (a, b) = v.split_once('-').expect("--seeds A-B");
                seeds = (a.parse().expect("a seed")..=b.parse().expect("a seed")).collect();
            }
            "--families" => families = value("--families").parse().expect("a number of families"),
            "--end-year" => end_year = Some(value("--end-year").parse().expect("a year")),
            "--why" => show_why = true,
            "--all" => all = true,
            "--counts" => counts = true,
            other => panic!("unknown option {other:?}"),
        }
    }
    let runs: Vec<RunResult> = std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                scope.spawn(move || {
                    let mut P = rules.params();
                    P.seed = seed;
                    P.n_families = families;
                    P.end_year = end_year;
                    sim::run_with(&P, rules.content(), false, true).expect("money is conserved")
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("the run finishes")).collect()
    });
    println!("{} rules, {} families, seed{} {}", rules.as_str(), families, if seeds.len() == 1 { "" } else { "s" }, seeds.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "));
    if counts || runs.len() > 1 {
        print_counts(&runs);
    } else {
        print_feed(&runs[0], show_why, all);
    }
}

fn decade_of(r: &RunResult, e: &Event) -> i64 {
    let (y0, y1) = (r.content.start_year, r.content.end_year);
    // a final odd year goes with the decade before it, as the chronicle ranks it
    let last = y0 + 10 * ((y1 - y0 - 1).max(0) / 10);
    ((e.year.floor() as i64 - y0) / 10 * 10 + y0).min(last)
}

fn print_feed(r: &RunResult, show_why: bool, all: bool) {
    let player = r.state.families.first().map(|f| f.surname.clone());
    if let Some(name) = &player {
        println!("the player's family: {name}, of {}", r.state.town_name[r.state.families[0].seat]);
    }
    let mut decade = i64::MIN;
    let mut shown = 0;
    for e in r.events.iter().filter(|e| all || major(e)) {
        let d = decade_of(r, e);
        if d != decade {
            decade = d;
            println!("\nThe {d}s");
        }
        shown += 1;
        let label = if e.family == Some(0) { "Your family" } else { kind_label(&e.kind) };
        let mark = if !all {
            String::new()
        } else if major(e) {
            format!(" [{:.2} kept]", e.weight)
        } else {
            format!(" [{:.2}]", e.weight)
        };
        println!("  {}  {:<16} {}{mark}", e.year.floor(), label, e.title);
        if !e.text.is_empty() {
            println!("        {}", e.text);
        }
        if show_why {
            if let Some(w) = &e.why {
                let base = w.baseline.map_or(String::new(), |b| format!(", against {b}"));
                println!("        why: {} ({}: {} to pass {}{base}); {}", w.test, w.series, w.value, w.threshold, w.note);
            }
        }
    }
    println!("\n{shown} items shown of {}", r.events.len());
}

fn print_counts(runs: &[RunResult]) {
    let n = runs.len() as f64;
    let decades: BTreeSet<i64> = runs.iter().flat_map(|r| r.events.iter().map(|e| decade_of(r, e))).collect();
    let kinds: BTreeSet<String> = runs.iter().flat_map(|r| r.events.iter().map(|e| e.kind.clone())).collect();
    // (decade, kind) -> items in all runs, and those of them that are major
    let mut every: BTreeMap<(i64, &str), (usize, usize)> = BTreeMap::new();
    for r in runs {
        for e in &r.events {
            let c = every.entry((decade_of(r, e), kinds.get(&e.kind).expect("listed").as_str())).or_default();
            c.0 += 1;
            c.1 += major(e) as usize;
        }
    }
    println!("\nitems per decade by kind, mean over {} run{}: all / major", runs.len(), if runs.len() == 1 { "" } else { "s" });
    print!("{:<8}", "decade");
    for k in &kinds {
        print!("{:>13}", k);
    }
    println!();
    for &d in &decades {
        print!("{:<8}", format!("{d}s"));
        for k in &kinds {
            let (a, m) = every.get(&(d, k.as_str())).copied().unwrap_or((0, 0));
            print!("{:>13}", format!("{:.1}/{:.1}", a as f64 / n, m as f64 / n));
        }
        println!();
    }
    print!("{:<8}", "run");
    for k in &kinds {
        let (a, m) = decades.iter().map(|&d| every.get(&(d, k.as_str())).copied().unwrap_or((0, 0))).fold((0, 0), |s, c| (s.0 + c.0, s.1 + c.1));
        print!("{:>13}", format!("{:.0}/{:.0}", a as f64 / n, m as f64 / n));
    }
    println!();

    println!("\nthe major feed's length per decade: mean [least .. greatest] over the runs");
    let mut whole = Vec::new();
    for &d in &decades {
        let per_run: Vec<usize> = runs.iter().map(|r| r.events.iter().filter(|e| major(e) && decade_of(r, e) == d).count()).collect();
        let mean = per_run.iter().sum::<usize>() as f64 / n;
        println!("  {d}s  {mean:>7.1} [{} .. {}]", per_run.iter().min().unwrap(), per_run.iter().max().unwrap());
        whole.push(mean);
    }
    let per_run: Vec<usize> = runs.iter().map(|r| r.events.iter().filter(|e| major(e)).count()).collect();
    println!("  run    {:>7.1} [{} .. {}]", per_run.iter().sum::<usize>() as f64 / n, per_run.iter().min().unwrap(), per_run.iter().max().unwrap());

    let own = |r: &RunResult, f: &dyn Fn(&Event) -> bool| r.events.iter().filter(|e| e.family == Some(0) && f(e)).count();
    let stat = |f: &dyn Fn(&Event) -> bool| {
        let v: Vec<usize> = runs.iter().map(|r| own(r, f)).collect();
        format!("{:.1} [{} .. {}]", v.iter().sum::<usize>() as f64 / n, v.iter().min().unwrap(), v.iter().max().unwrap())
    };
    println!("\nthe player's own items in a run: {}; of them major: {}", stat(&|_| true), stat(&|e| major(e)));
    let total: Vec<usize> = runs.iter().map(|r| r.events.len()).collect();
    println!("all items in a run: {:.1} [{} .. {}]", total.iter().sum::<usize>() as f64 / n, total.iter().min().unwrap(), total.iter().max().unwrap());
    let stories: BTreeMap<String, usize> = runs.iter().flat_map(|r| r.events.iter()).filter(|e| e.kind == "region").fold(BTreeMap::new(), |mut m, e| {
        let story = e.why.as_ref().map_or("region".to_string(), |w| w.test.clone());
        *m.entry(story).or_insert(0) += 1;
        m
    });
    if !stories.is_empty() {
        println!("\nregional stories in a run, mean over the runs (and how many were kept):");
        for (story, c) in &stories {
            let kept = runs.iter().flat_map(|r| r.events.iter()).filter(|e| e.kind == "region" && major(e) && e.why.as_ref().is_some_and(|w| &w.test == story)).count();
            println!("  {story:<45} {:>5.1}  ({:.1})", *c as f64 / n, kept as f64 / n);
        }
    }
}
