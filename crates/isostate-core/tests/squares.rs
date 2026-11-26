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
    // the record, packed with the same parameters, is the same to the byte but for the squares' log:
    // the squares write nothing else to it
    let mut on_rec = on.rec.clone();
    assert!(!on_rec.squares.is_empty() && off.rec.squares_first.is_empty());
    on_rec.squares.clear();
    on_rec.squares_first.clear();
    let pack = |r: &RunResult, rec: &record::Record| record::pack(&r.state, &r.content, &off.params, &r.events, rec, "game", false);
    let (a, b) = (pack(on, &on_rec), pack(off, &off.rec));
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

/// Squares as a reader of the log holds them: by `Change::at`, each square
/// something holds with its tenant's code, its year and its face.
type Held = BTreeMap<u32, (u32, i16, u8)>;

fn apply(held: &mut Held, changes: &[squares::Change]) {
    for c in changes {
        if c.tenant == 0 {
            held.remove(&c.at);
        } else {
            held.insert(c.at, (c.tenant, c.year, c.face));
        }
    }
}

/// The book itself, as a reader of the log should hold it.
fn book_of(st: &State) -> Held {
    let mut held = Held::new();
    for (c, &i) in st.sq.index.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let t = &st.sq.tables[i as usize - 1];
        for k in 0..PER_CELL {
            if t.tenant[k] != 0 {
                held.insert((c * PER_CELL + k) as u32, (t.tenant[k], t.year[k], t.face[k]));
            }
        }
    }
    held
}

/// The squares' arrays of a packed record (`sq_first`, `sq_log`) and its
/// header's word of them, if it has them.
fn log_of(blob: &[u8]) -> Option<(Vec<i32>, Vec<u8>, serde_json::Value)> {
    let n = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let header: serde_json::Value = serde_json::from_slice(&blob[4..4 + n]).unwrap();
    let array = |name: &str| {
        let a = header["arrays"].as_array().unwrap().iter().find(|a| a["name"] == name)?;
        let (off, len) = (a["offset"].as_u64().unwrap() as usize, a["nbytes"].as_u64().unwrap() as usize);
        Some(blob[4 + n + off..4 + n + off + len].to_vec())
    };
    let first = array("sq_first")?.chunks_exact(4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
    Some((first, array("sq_log")?, header.get("squares")?.clone()))
}

#[test]
fn a_replay_and_a_live_game_hold_the_squares_of_the_book() {
    // Island 3 to 1800, played as a live game is (`live_year` after every year): each
    // year the live year's changes are the record's, the squares a reader makes of
    // the live years are the book's square for square, and so are those it makes of
    // the record packed and read back. Played under the game's rules with the squares
    // switched on by name; the case leans on no history, only on something being built.
    let mut P = params(3, 1800);
    P.squares.enabled = true;
    let mut s = sim::start(&P, game_content());
    let mut cursor = LiveCursor::new(&s.content);
    let mut live = Held::new();
    let mut each_year: Vec<Held> = Vec::new();
    let mut changed = 0;
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved");
        let y = live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        let sq = y.squares.expect("a game with the squares hands them over each year");
        assert!(!sq.whole, "{year}: a cursor told every year is sent the year's changes");
        let k = y.index;
        let (a, b) = (s.rec.squares_first[k] as usize, s.rec.squares_first[k + 1] as usize);
        assert!(s.rec.squares[a..b] == sq.changes[..], "{year}: the live year's changes are not the record's");
        assert!(sq.changes.windows(2).all(|w| w[0].at < w[1].at), "{year}: the changes in order, each square once");
        apply(&mut live, &sq.changes);
        let book = book_of(&s.st);
        assert!(live == book, "{year}: the live game holds {} squares and the book {}", live.len(), book.len());
        changed += sq.changes.len();
        each_year.push(book);
    }
    assert!(live.len() > 10_000 && changed > live.len(), "{} squares held, {changed} changes", live.len());
    // the record as the viewer reads it
    let blob = record::pack(&s.st, &s.content, &s.params, &s.events, &s.rec, "game", false);
    let (first, log, head) = log_of(&blob).expect("the record carries the squares' log");
    assert_eq!(head["layout"], squares::LOG_LAYOUT);
    assert_eq!(first.len(), each_year.len() + 1);
    let mut replay = Held::new();
    for (k, book) in each_year.iter().enumerate() {
        let year = s.content.start_year + k as i64;
        let changes = squares::decode_year(&log[first[k] as usize..first[k + 1] as usize], year).expect("the log reads");
        apply(&mut replay, &changes);
        assert!(&replay == book, "{year}: the replay holds {} squares and the book {}", replay.len(), book.len());
    }
    println!("island 3 to 1800: {} squares held, {changed} changes, a log of {} bytes", live.len(), log.len());
}

#[test]
fn a_cursor_that_missed_a_year_is_sent_the_whole_book() {
    let P = params(2, 1752);
    let mut s = sim::start(&P, game_content());
    let mut early = LiveCursor::new(&s.content);
    sim::run_year(&mut s).unwrap();
    let first = live_year(&mut early, &s.st, &s.content, &s.params, &s.events).squares.unwrap();
    assert!(!first.whole && first.changes == s.st.sq.changes);
    sim::run_year(&mut s).unwrap();
    sim::run_year(&mut s).unwrap();
    // the early cursor was not told 1751, and a new one was told nothing
    for mut c in [early, LiveCursor::new(&s.content)] {
        let sq = live_year(&mut c, &s.st, &s.content, &s.params, &s.events).squares.unwrap();
        assert!(sq.whole);
        let mut held = Held::new();
        apply(&mut held, &sq.changes);
        assert!(held == book_of(&s.st));
    }
}

#[test]
fn the_log_reads_back_what_was_written() {
    use squares::Change;
    let year = 1900;
    let at = |cell: u32, k: u32| cell * PER_CELL as u32 + k;
    let changes = vec![
        Change { at: at(0, 0), tenant: Tenant::Works(0).code(), year: 1900, face: 3, floor: 0 },
        Change { at: at(0, 1), tenant: 0, year: 0, face: 0, floor: 0 },
        Change { at: at(0, 255), tenant: Tenant::Works(0).code(), year: 1900, face: 255, floor: 0 },
        Change { at: at(7, 9), tenant: Tenant::Houses(41).code(), year: 1645, face: 64, floor: 0x22 },
        Change { at: at(7, 10), tenant: Tenant::Houses(41).code(), year: 1900, face: 0, floor: 0x28 },
        Change { at: at(7, 11), tenant: Tenant::Open(41).code(), year: 1901, face: 1, floor: 0 },
        Change { at: at(7, 12), tenant: Tenant::Houses(41).code(), year: 1900, face: 5, floor: 0 },
        Change { at: at(40_000, 128), tenant: Tenant::Works((1 << 28) - 1).code(), year: 2050, face: 9, floor: 0 },
        Change { at: at(40_000, 129), tenant: Tenant::Pit(3).code(), year: 1900, face: 200, floor: 0 },
    ];
    let mut shuffled = changes.clone();
    shuffled.reverse();
    let bytes = squares::encode_year(&shuffled, year);
    assert_eq!(squares::decode_year(&bytes, year).unwrap(), changes);
    // a square taken for a tenant that takes another in its cell that year is two bytes more
    let one = squares::encode_year(&changes[..1], year).len();
    let two = squares::encode_year(&[changes[0], changes[2]], year).len();
    assert_eq!(two - one, 2);
    assert!(squares::decode_year(&bytes[..bytes.len() - 1], year).is_err(), "a log cut short is refused");
    assert!(squares::encode_year(&[], year).is_empty());
}

#[test]
fn a_save_plays_again_with_its_squares_and_a_port_record_keeps_none() {
    // a game under the game's rules, three years, saved and played again
    let mut g = save::Game::new(4, 6, 0);
    assert!(g.sim.params.squares.enabled);
    for _ in 0..3 {
        g.year().unwrap();
        g.turn();
    }
    let text = g.save(0).to_text();
    let again = save::load(&save::Save::from_text(&text).unwrap(), |_| {}).unwrap();
    assert!(!again.drifted && again.refused.is_empty());
    let (a, b) = (&g.sim.st.sq, &again.game.sim.st.sq);
    assert!(a.told == 3 && b.told == 3 && !a.whole().is_empty());
    assert!(a.whole() == b.whole() && g.sim.rec.squares == again.game.sim.rec.squares, "the save's squares came out otherwise");
    // the port keeps no squares, and its record says nothing of them
    let mut P = default_params();
    P.seed = 4;
    P.end_year = Some(1751);
    let r = sim::run_with(&P, default_content(), false, true).unwrap();
    let blob = record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "port", false);
    assert!(log_of(&blob).is_none());
    let n = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let header = String::from_utf8_lossy(&blob[4..4 + n]);
    assert!(!header.contains("\"sq_") && !header.contains("\"squares\""));
}
