//! Squares of 125 m kept where something is built (`docs/rules/squares.md`, not
//! in the Python): switched off they are the port (tests/golden.rs holds that
//! to the byte) and the book stays empty; switched on, every year on islands 1
//! to 8 each cell's squares come to its shares within a square, no square has
//! two tenants, a works' squares are in one piece, a tenant's squares keep
//! their year while it holds them, and nothing in the economy, the books, the
//! news or the record moves with the switch.
//!
//! The cases are played under the whole of the game's rules (`game_params()`,
//! six families), whatever they are as this is read: the squares read the
//! books of every rule and are read by none, so no rule's switch is set here.
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use isostate_core::cli::{events_json, series_csv};
use isostate_core::squares::{self, PER_CELL, SquaresParams, Tenant};
use isostate_core::*;

fn params(seed: u64, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end_year);
    P
}

#[test]
fn off_is_the_port_and_on_is_the_game() {
    assert_eq!(default_params().squares, SquaresParams::default());
    assert!(!default_params().squares.enabled, "the port keeps no squares");
    assert!(game_params().squares.enabled, "the game does");
    let mut P = default_params();
    P.seed = 3;
    P.end_year = Some(1752);
    let r = sim::run_with(&P, default_content(), false, true).expect("money is conserved");
    assert!(!r.state.sq.kept() && r.state.sq.tables.is_empty() && r.state.sq.works.is_empty());
    assert!(!serde_json::to_string(&r.params).unwrap().contains("\"squares\""), "params.json does not mention it");
}

/// Every year of island `seed` to `to`: the book checked against the books,
/// and every square that keeps its tenant keeps its year. Returns the census
/// of the last year.
fn holds(seed: u64, to: i64) -> squares::Census {
    let P = params(seed, to);
    let mut s = sim::start(&P, game_content());
    let mut last: BTreeMap<usize, (Vec<u32>, Vec<i16>)> = BTreeMap::new();
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved");
        let st = &s.st;
        if let Err(e) = squares::check(st, &s.content, &s.params, year) {
            panic!("island {seed}: {e}");
        }
        let mut now = BTreeMap::new();
        for (c, &k) in st.sq.index.iter().enumerate() {
            if k == 0 {
                continue;
            }
            let t = &st.sq.tables[k as usize - 1];
            if let Some((ta, ya)) = last.get(&c) {
                for q in 0..PER_CELL {
                    assert!(
                        ta[q] != t.tenant[q] || ta[q] == 0 || ya[q] == t.year[q],
                        "island {seed}, {year}: square {q} of cell {c} kept {:?} and its year went from {} to {}",
                        Tenant::of(ta[q]),
                        ya[q],
                        t.year[q]
                    );
                }
            }
            now.insert(c, (t.tenant.to_vec(), t.year.to_vec()));
        }
        // and a works' squares are those it held, or more or fewer of the same order
        last = now;
    }
    squares::census(&s.st)
}

#[test]
fn the_book_holds_every_year_on_islands_1_to_8() {
    let found: Vec<(u64, squares::Census)> = std::thread::scope(|sc| {
        let hs: Vec<_> = (1..=8u64).map(|seed| sc.spawn(move || (seed, holds(seed, 1790)))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (seed, c) in &found {
        // something is built on every island, and the houses outnumber the works in 1790
        assert!(c.tables > 50 && c.works > 500 && c.houses > c.works, "island {seed}: {c:?}");
        // a works in more than one piece is one that was closed in (check allows only those):
        // a twentieth of them or so in 1790 (22 of 436 on island 1 on 26 November 2026)
        println!("island {seed}, 1790: {c:?}");
        assert!(c.works_split * 10 <= c.works_alive, "island {seed}: {} works of {} in pieces", c.works_split, c.works_alive);
    }
}

#[test]
fn a_works_keeps_its_first_square_while_it_stands() {
    // island 7 to 1800: every works that held squares in 1780 and still stands
    // holds the square it first stood on
    let P = params(7, 1800);
    let mut s = sim::start(&P, game_content());
    let mut first: BTreeMap<usize, (usize, u8)> = BTreeMap::new();
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved");
        let st = &s.st;
        for (f, h) in st.sq.works.iter().enumerate() {
            match (first.get(&f), h.squares.first()) {
                (Some(&(c, k)), Some(&k2)) => assert_eq!((c, k), (h.cell, k2), "{year}: works {f} moved"),
                (None, Some(&k)) => {
                    first.insert(f, (h.cell, k));
                }
                (Some(_), None) => {
                    first.remove(&f);
                }
                (None, None) => {}
            }
        }
    }
    assert!(first.len() > 300, "{} works with squares", first.len());
}

#[test]
fn nothing_moves_with_the_switch() {
    let runs: Vec<RunResult> = std::thread::scope(|sc| {
        let hs: Vec<_> = [true, false]
            .into_iter()
            .map(|on| {
                sc.spawn(move || {
                    let mut P = params(5, 1790);
                    if !on {
                        P.squares = SquaresParams::default();
                    }
                    sim::run_with(&P, game_content(), false, true).expect("money is conserved")
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let (on, off) = (&runs[0], &runs[1]);
    assert!(on.state.sq.kept() && !off.state.sq.kept());
    assert!(series_csv(on) == series_csv(off), "the months (the economy and the books) moved with the squares");
    assert!(events_json(on) == events_json(off), "the news moved with the squares");
    // the record, packed with the same parameters, is the same to the byte: the squares write nothing to it
    let pack = |r: &RunResult| record::pack(&r.state, &r.content, &off.params, &r.events, &r.rec, "game", false);
    let (a, b) = (pack(on), pack(off));
    let n = u32::from_le_bytes([a[0], a[1], a[2], a[3]]) as usize;
    let strip = |blob: &[u8]| {
        // the header's date of writing is the one thing that may differ
        let h = String::from_utf8_lossy(&blob[4..4 + n]).to_string();
        let at = h.find("\"generated\":\"").map(|i| i + 13).unwrap_or(0);
        let mut v = blob.to_vec();
        if at > 0 {
            v[4 + at..4 + at + 10].copy_from_slice(b"0000-00-00");
        }
        v
    };
    assert!(strip(&a) == strip(&b), "the record moved with the squares");
}
