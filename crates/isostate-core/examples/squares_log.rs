//! The squares' log (`docs/rules/squares.md`, "In the record"): whether a
//! replay holds the squares a live game holds in every year, how large the log
//! is, and what it adds to the time to write a record.
//!
//!     cargo run --profile work -p isostate-core --example squares_log -- [--seeds 7,2] [--to 2050]
//!         [--families 6] [--write DIR]
//!
//! Each island runs under the game's rules on its own thread, with a live
//! cursor beside it as a game has (`live_year` after every year). Every year:
//! the live year's changes are the record's, and the squares a reader makes of
//! the live years are the book's. At the end the record is packed as
//! `isostate-core run` packs it, its log read back from the bytes year by year
//! and held against the book of each year; and it is packed again without the
//! log, so that the two can be set side by side. `--write DIR` writes both,
//! DIR/run-SEED.bin and DIR/run-SEED-nosquares.bin, for timing the viewer's
//! opening of them.
#![allow(non_snake_case)]

use std::collections::BTreeMap;
use std::time::Instant;

use isostate_core::squares::{self, Change, PER_CELL};
use isostate_core::*;

type Held = BTreeMap<u32, (u32, i16, u8)>;

fn apply(held: &mut Held, changes: &[Change]) {
    for c in changes {
        if c.tenant == 0 {
            held.remove(&c.at);
        } else {
            held.insert(c.at, (c.tenant, c.year, c.face));
        }
    }
}

fn book_of(st: &State) -> Held {
    let mut held = Held::new();
    for (c, &i) in st.sq.index.iter().enumerate() {
        if i > 0 {
            let t = &st.sq.tables[i as usize - 1];
            for k in 0..PER_CELL {
                if t.tenant[k] != 0 {
                    held.insert((c * PER_CELL + k) as u32, (t.tenant[k], t.year[k], t.face[k]));
                }
            }
        }
    }
    held
}

/// FNV-1a of the squares held.
fn print(held: &Held) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for (&at, &(t, y, f)) in held {
        for b in at.to_le_bytes().into_iter().chain(t.to_le_bytes()).chain(y.to_le_bytes()).chain([f]) {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

fn run(seed: u64, to: i64, families: usize, write: Option<&str>) -> String {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = Some(to);
    let t0 = Instant::now();
    let mut s = sim::start(&P, game_content());
    let mut cursor = LiveCursor::new(&s.content);
    let mut live = Held::new();
    let mut prints: Vec<u64> = Vec::new();
    let (mut faults, mut first_fault) = (0usize, None::<String>);
    let mut fault = |what: String| {
        faults += 1;
        first_fault.get_or_insert(what);
    };
    let mut changes = 0usize;
    let mut most = (0usize, 0i64);
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved");
        let y = live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        let sq = y.squares.expect("the game's rules keep the squares");
        let k = y.index;
        let (a, b) = (s.rec.squares_first[k] as usize, s.rec.squares_first[k + 1] as usize);
        if sq.whole || s.rec.squares[a..b] != sq.changes[..] {
            fault(format!("{year}: the live year's changes are not the record's"));
        }
        apply(&mut live, &sq.changes);
        let book = book_of(&s.st);
        if live != book {
            fault(format!("{year}: the live game holds {} squares and the book {}", live.len(), book.len()));
        }
        changes += sq.changes.len();
        if sq.changes.len() > most.0 && k > 0 {
            most = (sq.changes.len(), year);
        }
        prints.push(print(&book));
    }
    let run_s = t0.elapsed().as_secs_f64();
    let held = live.len();
    // the record with its log, and without
    let mut with_t = Vec::new();
    let mut blob = Vec::new();
    for _ in 0..3 {
        let t = Instant::now();
        blob = record::pack(&s.st, &s.content, &s.params, &s.events, &s.rec, "game", false);
        with_t.push(t.elapsed().as_secs_f64());
    }
    let mut bare = s.rec.clone();
    bare.squares.clear();
    bare.squares_first.clear();
    let mut without_t = Vec::new();
    let mut plain = Vec::new();
    for _ in 0..3 {
        let t = Instant::now();
        plain = record::pack(&s.st, &s.content, &s.params, &s.events, &bare, "game", false);
        without_t.push(t.elapsed().as_secs_f64());
    }
    let mut enc_t = Vec::new();
    for _ in 0..3 {
        let t = Instant::now();
        let _ = record::squares_log(&s.rec, s.content.start_year);
        enc_t.push(t.elapsed().as_secs_f64());
    }
    // the log read back from the record's bytes, year by year, against the book of each year
    let n = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let header: serde_json::Value = serde_json::from_slice(&blob[4..4 + n]).unwrap();
    let array = |name: &str| {
        let a = header["arrays"].as_array().unwrap().iter().find(|a| a["name"] == name).expect("the log is in the record");
        let (off, len) = (a["offset"].as_u64().unwrap() as usize, a["nbytes"].as_u64().unwrap() as usize);
        &blob[4 + n + off..4 + n + off + len]
    };
    let first: Vec<usize> = array("sq_first").chunks_exact(4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize).collect();
    let log = array("sq_log");
    let mut replay = Held::new();
    let t = Instant::now();
    let mut decoded = Vec::new();
    for k in 0..prints.len() {
        decoded.push(squares::decode_year(&log[first[k]..first[k + 1]], s.content.start_year + k as i64).expect("the log reads"));
    }
    let dec_s = t.elapsed().as_secs_f64();
    let mut years_held = 0;
    for (k, ch) in decoded.iter().enumerate() {
        apply(&mut replay, ch);
        if print(&replay) == prints[k] {
            years_held += 1;
        } else {
            fault(format!("{}: the replay's squares are not the book's", s.content.start_year + k as i64));
        }
    }
    if let Some(dir) = write {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(format!("{dir}/run-{seed}.bin"), &blob).unwrap();
        std::fs::write(format!("{dir}/run-{seed}-nosquares.bin"), &plain).unwrap();
    }
    let mb = |b: usize| b as f64 / 1e6;
    format!(
        "island {seed} to {to} ({run_s:.0} s): {} years; the replay held the book's squares in {years_held} of them, the live game in every year but {faults} (first fault: {})\n  \
         squares held at the end {held}; changes in all {changes}, the most in a year after the first {} ({}); the first year's (with the opening) {}\n  \
         the log {:.2} MB ({:.2} bytes a change); run.bin {:.1} MB with it, {:.1} MB without (+{:.1}%)\n  \
         packing the record: {:.0} ms with the log, {:.0} ms without (medians of 3); writing the log alone {:.0} ms; reading it back {:.0} ms",
        prints.len(),
        first_fault.as_deref().unwrap_or("none"),
        most.0,
        most.1,
        s.rec.squares_first[1],
        mb(log.len()),
        log.len() as f64 / changes.max(1) as f64,
        mb(blob.len()),
        mb(plain.len()),
        100.0 * (blob.len() as f64 / plain.len() as f64 - 1.0),
        median(with_t) * 1e3,
        median(without_t) * 1e3,
        median(enc_t) * 1e3,
        dec_s * 1e3,
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = vec![7, 2];
    let (mut to, mut families, mut write) = (2050i64, 6usize, None::<String>);
    let mut k = 0;
    while k < args.len() {
        match args[k].as_str() {
            "--seeds" => {
                k += 1;
                seeds = args[k].split(',').map(|p| p.parse().unwrap()).collect();
            }
            "--to" => {
                k += 1;
                to = args[k].parse().unwrap();
            }
            "--families" => {
                k += 1;
                families = args[k].parse().unwrap();
            }
            "--write" => {
                k += 1;
                write = Some(args[k].clone());
            }
            a => panic!("unknown argument {a}"),
        }
        k += 1;
    }
    let lines: Vec<String> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.iter().map(|&s| sc.spawn({
            let write = write.clone();
            move || run(s, to, families, write.as_deref())
        })).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for l in lines {
        println!("{l}");
    }
}
