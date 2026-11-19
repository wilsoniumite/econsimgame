//! The game's golden run: seed 7 to 1900 under the game's rules
//! (`game_params()` and `game_content()`) with six families, which must come
//! out byte for byte as it did when the hashes in `golden_game.txt` were taken.
//!
//! `golden.rs` holds the port and cannot see the game: under `default_params()`
//! nearly every hook of the game's rules returns on its first line. This test
//! is the same idea on the rules a game is played under (the architecture
//! review's change A1, `docs/designs/architecture.md`). It hashes four things
//! apart, so that a failure says which moved:
//!
//! - **economy**: `series_monthly.csv` as `isostate-core run` writes it, less
//!   the books' own columns;
//! - **books**: the books' own series (`acct_`, `accounts::own_series`) with the
//!   year and month, which watch the economy and never touch it, so a change to
//!   the books alone shows here and nowhere else in the series;
//! - **news**: `events.json`;
//! - **run.bin**: the record the viewer draws, as `record::pack` writes it, with
//!   the date it was written (`meta.generated`) set to `0000-00-00`, since that
//!   is the only byte of it that depends on the day.
//!
//! The hash is 64-bit FNV-1a, as in `golden.rs`. The island is run twice, side
//! by side on two threads, and the two runs must agree before anything is set
//! against the stored hashes: if they do not, the run is not repeatable (it
//! reads a clock, a thread, an environment variable or the order of a hash
//! map), which is a fault whatever the hashes say, and the test says so.
//!
//! Unlike the port's hashes, these are meant to move: every change to the
//! game's rules made on purpose moves at least one of them. When that happens
//! they are taken again on the merged tree with
//!
//! ```text
//! ISOSTATE_GOLDEN_PRINT=1 cargo test -p isostate-core --release --test golden_game
//! ```
//!
//! which checks nothing and stops with the four lines to paste into
//! `golden_game.txt`. The commit that does it says which moved and why
//! (`docs/rules/README.md`, "The game's golden test", says who may and when).
//!
//! To look at a change rather than only see it, run
//!
//! ```text
//! ISOSTATE_GOLDEN_DUMP=some/dir cargo test -p isostate-core --release --test golden_game
//! ```
//!
//! before and after: it writes `game6/` under that directory with
//! `series_monthly.csv`, `events.json` and `run.bin` (the last with its date
//! blanked, as hashed), ready for a diff.

use isostate_core::cli::{events_json, label_for, series_csv};
use isostate_core::record;
use isostate_core::sim::{self, RunResult};
use isostate_core::{accounts, game_content, game_params};

/// The stored hashes, read at build time: a new file rebuilds the test.
const STORED: &str = include_str!("golden_game.txt");

/// What is hashed, in the order the file lists it.
const NAMES: [&str; 4] = ["economy", "books", "news", "run.bin"];

/// What each name means, for the messages.
const MEANING: [&str; 4] = [
    "the economy's monthly series",
    "the books' own series (acct_)",
    "the news",
    "the run file the viewer draws (run.bin)",
];

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The four hashes as `golden_game.txt` holds them.
fn stored() -> [u64; 4] {
    let mut got = [None; 4];
    for (n, line) in STORED.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut words = line.split_whitespace();
        let (Some(name), Some(hex), None) = (words.next(), words.next(), words.next()) else {
            panic!("golden_game.txt, line {}: expected a name and a hash, found {line:?}", n + 1);
        };
        let Some(k) = NAMES.iter().position(|&x| x == name) else {
            panic!("golden_game.txt, line {}: {name:?} is not one of {NAMES:?}", n + 1);
        };
        let value = hex.strip_prefix("0x").and_then(|h| u64::from_str_radix(h, 16).ok());
        let Some(value) = value else {
            panic!("golden_game.txt, line {}: {hex:?} is not a hash written 0x and sixteen hex digits", n + 1);
        };
        assert!(got[k].is_none(), "golden_game.txt, line {}: {name} is given twice", n + 1);
        got[k] = Some(value);
    }
    let mut out = [0; 4];
    for k in 0..4 {
        out[k] = got[k].unwrap_or_else(|| panic!("golden_game.txt has no line for {}", NAMES[k]));
    }
    out
}

/// Split `series_monthly.csv` into the economy's columns and the books', each
/// with the year and the month and in the file's own order and spelling.
fn split_series(csv: &str) -> (String, String) {
    let mut rows = csv.split_terminator("\r\n");
    let header: Vec<&str> = rows.next().expect("the series have a header").split(',').collect();
    let books: Vec<bool> = header.iter().enumerate().map(|(k, name)| k >= 2 && accounts::own_series(name)).collect();
    let (mut economy, mut acct) = (String::new(), String::new());
    for row in std::iter::once(header.join(",")).chain(rows.map(str::to_string)) {
        let cells: Vec<&str> = row.split(',').collect();
        assert_eq!(cells.len(), header.len(), "every row of the series has a cell for every column");
        let (mut e, mut b) = (Vec::new(), Vec::new());
        for (k, cell) in cells.iter().enumerate() {
            if k < 2 {
                e.push(*cell);
                b.push(*cell);
            } else if books[k] {
                b.push(*cell);
            } else {
                e.push(*cell);
            }
        }
        economy.push_str(&e.join(","));
        economy.push_str("\r\n");
        acct.push_str(&b.join(","));
        acct.push_str("\r\n");
    }
    (economy, acct)
}

/// `run.bin` as `isostate-core run` writes it, with its date set to `0000-00-00`.
fn run_bin(r: &RunResult) -> Vec<u8> {
    let mut blob = record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, label_for(r.params.rent_regime), false);
    let hlen = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let key = b"\"generated\":\"";
    let at = blob[4..4 + hlen].windows(key.len()).position(|w| w == key).expect("run.bin's header gives the date it was written")
        + 4
        + key.len();
    let date = &mut blob[at..at + 10];
    assert!(
        date.iter().enumerate().all(|(k, &c)| if k == 4 || k == 7 { c == b'-' } else { c.is_ascii_digit() }),
        "run.bin's date is written YYYY-MM-DD"
    );
    date.copy_from_slice(b"0000-00-00");
    blob
}

/// One run of the island, hashed. With `dump`, its files are written there too.
#[allow(non_snake_case)]
fn hashed(dump: Option<&std::path::Path>) -> [u64; 4] {
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    let r = sim::run_with(&P, game_content(), false, true).expect("money is conserved");
    let csv = series_csv(&r);
    let (economy, books) = split_series(&csv);
    let news = events_json(&r);
    let bin = run_bin(&r);
    if let Some(dir) = dump {
        std::fs::create_dir_all(dir).expect("the dump directory can be made");
        std::fs::write(dir.join("series_monthly.csv"), &csv).expect("series written");
        std::fs::write(dir.join("events.json"), &news).expect("events written");
        std::fs::write(dir.join("run.bin"), &bin).expect("run.bin written");
    }
    [fnv1a(economy.as_bytes()), fnv1a(books.as_bytes()), fnv1a(news.as_bytes()), fnv1a(&bin)]
}

/// Which of the four differ, in words.
fn which(a: &[u64; 4], b: &[u64; 4]) -> String {
    let moved: Vec<&str> = (0..4).filter(|&k| a[k] != b[k]).map(|k| MEANING[k]).collect();
    match moved.len() {
        1 => moved[0].to_string(),
        n => format!("{} and {}", moved[..n - 1].join(", "), moved[n - 1]),
    }
}

/// The four lines of `golden_game.txt` for these hashes.
fn lines(h: &[u64; 4]) -> String {
    (0..4).map(|k| format!("{:<8} 0x{:016x}\n", NAMES[k], h[k])).collect()
}

#[test]
fn seed_7_under_the_games_rules_is_unchanged() {
    let want = stored();
    let dump = std::env::var("ISOSTATE_GOLDEN_DUMP").ok().map(|d| std::path::Path::new(&d).join("game6"));
    // two runs of the same island at once: each has its own state, so they
    // must agree to the last bit
    let (first, second) = std::thread::scope(|s| {
        let a = s.spawn(|| hashed(dump.as_deref()));
        let b = s.spawn(|| hashed(None));
        (a.join().expect("the first run finished"), b.join().expect("the second run finished"))
    });
    if first != second {
        panic!(
            "\n\nThe game's run is not repeatable: two runs of seed 7 under game_params() with six families, side by \
             side in one process, disagree in {}.\n\
             first\n{}second\n{}\n\
             This is a fault whatever the stored hashes say. Something in the run reads what is not the rules and the \
             island: a clock, a thread, an environment variable, a global, or the order of a hash map that is walked. \
             The date in run.bin is already set aside. Find it before taking any hash.\n",
            which(&first, &second),
            lines(&first),
            lines(&second)
        );
    }
    if std::env::var("ISOSTATE_GOLDEN_PRINT").is_ok() {
        panic!(
            "\n\nISOSTATE_GOLDEN_PRINT is set, so nothing was checked. The two runs agree, and give\n\n{}\n\
             Paste these four lines over those in crates/isostate-core/tests/golden_game.txt, and say in the commit \
             message which moved and why (docs/rules/README.md, \"The game's golden test\").\n",
            lines(&first)
        );
    }
    if first != want {
        panic!(
            "\n\nThe game moved: seed 7 to 1900 under game_params() and game_content() with six families no longer \
             gives the run it gave. What moved: {}.\n\
             got\n{}expected (crates/isostate-core/tests/golden_game.txt)\n{}\n\
             The two runs agree with each other, so the run is repeatable and the rules or the code changed.\n\
             If that was meant (a stage of the rules pass, a rule mended or retuned, a new rule switched on in \
             game_params()), take the new hashes on the merged tree with\n\n    \
             ISOSTATE_GOLDEN_PRINT=1 cargo test -p isostate-core --release --test golden_game\n\n\
             paste the four lines it gives into golden_game.txt, and say in the commit message which moved and why.\n\
             If nothing was meant to move, the change is at fault, not the file. Look for a hook moved across a \
             read or a write of the month (the comment above econ::step_month lists the order that must hold), \
             a rule acting where it should not, or, if the economy moved when only the books were touched, the \
             books touching the economy. ISOSTATE_GOLDEN_DUMP=DIR writes the run's files for a diff \
             against the same from the commit before.\n",
            which(&first, &want),
            lines(&first),
            lines(&want)
        );
    }
}
