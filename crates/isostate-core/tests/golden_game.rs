//! The game's golden run: seed 7 under the game's rules (`game_params()` and
//! `game_content()`) with six families, to 1900 and on to 2050, which must come
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
//! And it hashes them in two eras, so that a failure also says which century
//! moved: **1750-1900**, the played century, from a run that ends in 1900 as
//! the game always has; and **1901-2050**, the run to 2050's century, from a
//! run of the same island on to 2050: its months and its news from 1901 on, and
//! its whole `run.bin` (which holds the years before 1901 too, so it moves with
//! either era). A change that acts only after 1900 moves the second era alone.
//!
//! The first era is hashed blind to what content added after 1900 changes
//! without changing the run (`blind_series`, `blind_news`): a column of the
//! series that is nothing in every month to 1900 is left out (a new method's
//! output, which nothing makes before it is found), and the news names an
//! invention by its name, not by its place in the content's list, which a new
//! method placed before it moves on. A column that is anything but nothing in
//! any month still counts, and so does every other field of the news. The
//! first era has no `run.bin` of its own: the record carries the content's
//! lists, so a new entry anywhere moves it, and the second era's `run.bin`
//! already holds the years to 1900. The second era is hashed strictly, as it
//! was. (Before 20 November 2026 the first era was strict too, and a piece
//! that only added offices after 1900 moved three of its four.)
//!
//! The hash is 64-bit FNV-1a, as in `golden.rs`. Three runs go side by side on
//! three threads: the run to 1900, and the run to 2050 twice. Before anything
//! is set against the stored hashes, two things must hold. The two runs to
//! 2050 must agree: if they do not, the run is not repeatable (it reads a
//! clock, a thread, an environment variable or the order of a hash map), which
//! is a fault whatever the hashes say. And the run to 2050 must pass through
//! the run to 1900: its months and news to 1900 are those of the run that ends
//! there, or the end year reaches back into the years before it.
//!
//! Unlike the port's hashes, these are meant to move: every change to the
//! game's rules made on purpose moves at least one of them. When that happens
//! they are taken again on the merged tree with one command,
//!
//! ```text
//! ISOSTATE_GOLDEN_WRITE="what moved them and why" cargo test -p isostate-core --test golden_game -- --nocapture
//! ```
//!
//! which checks the two things above, writes the seven hashes into
//! `golden_game.txt` with the reason added below the file's earlier `# Taken:`
//! lines (its history, kept), prints which moved, and passes.
//! (`ISOSTATE_GOLDEN_PRINT=1` instead checks nothing and stops with the seven
//! lines, for pasting by hand.) The commit that does it
//! says which moved and why (`docs/rules/README.md`, "The game's golden test",
//! says who may and when).
//!
//! To look at a change rather than only see it, run
//!
//! ```text
//! ISOSTATE_GOLDEN_DUMP=some/dir cargo test -p isostate-core --test golden_game
//! ```
//!
//! before and after: it writes `game6/` (the run to 1900) and `game6-2050/`
//! (the run to 2050) under that directory, each with `series_monthly.csv`,
//! `events.json` and `run.bin` (the last with its date blanked, as hashed),
//! ready for a diff.

use isostate_core::cli::{events_json, label_for, python_json_pretty, series_csv};
use isostate_core::record;
use isostate_core::sim::{self, RunResult};
use isostate_core::{accounts, game_content, game_params};

/// A heap for each thread: the three runs go side by side (`heaps.rs`).
#[global_allocator]
static HEAPS: isostate_core::heaps::ThreadHeaps = isostate_core::heaps::ThreadHeaps;

/// The stored hashes, read at build time: a new file rebuilds the test.
const STORED: &str = include_str!("golden_game.txt");

/// The last year of the first era, and the end of the second.
const TURN: i64 = 1900;
const END: i64 = 2050;

/// The two eras, as the file names them.
const ERAS: [&str; 2] = ["1750-1900", "1901-2050"];

/// What is hashed, in the order the file lists it.
const NAMES: [&str; 4] = ["economy", "books", "news", "run.bin"];

/// What each name means, for the messages.
const MEANING: [&str; 4] = [
    "the economy's monthly series",
    "the books' own series (acct_)",
    "the news",
    "the run file the viewer draws (run.bin)",
];

/// The seven hashes, each an era and a name (indices into `ERAS` and `NAMES`):
/// the first era's three, blind to content added after 1900, then the second
/// era's four.
const SLOTS: [(usize, usize); 7] = [(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (1, 3)];

/// How many of `SLOTS` are the first era's.
const FIRST: usize = 3;

/// Seven hashes, in the order of `SLOTS`.
type Hashes = [u64; 7];

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The seven hashes as `golden_game.txt` holds them, each line an era, a name
/// and a hash.
fn stored() -> Hashes {
    let mut got = [None; 7];
    for (n, line) in STORED.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut words = line.split_whitespace();
        let (Some(era), Some(name), Some(hex), None) = (words.next(), words.next(), words.next(), words.next()) else {
            panic!("golden_game.txt, line {}: expected an era, a name and a hash, found {line:?}", n + 1);
        };
        let Some(e) = ERAS.iter().position(|&x| x == era) else {
            panic!("golden_game.txt, line {}: {era:?} is not one of {ERAS:?}", n + 1);
        };
        let Some(k) = NAMES.iter().position(|&x| x == name) else {
            panic!("golden_game.txt, line {}: {name:?} is not one of {NAMES:?}", n + 1);
        };
        let value = hex.strip_prefix("0x").and_then(|h| u64::from_str_radix(h, 16).ok());
        let Some(value) = value else {
            panic!("golden_game.txt, line {}: {hex:?} is not a hash written 0x and sixteen hex digits", n + 1);
        };
        let Some(i) = SLOTS.iter().position(|&s| s == (e, k)) else {
            panic!(
                "golden_game.txt, line {}: {era} {name} is not hashed: the first era has no run.bin of its own (the \
                 second era's holds the years to 1900 too)",
                n + 1
            );
        };
        assert!(got[i].is_none(), "golden_game.txt, line {}: {era} {name} is given twice", n + 1);
        got[i] = Some(value);
    }
    let mut out = [0; 7];
    for i in 0..7 {
        let (e, k) = SLOTS[i];
        out[i] = got[i].unwrap_or_else(|| panic!("golden_game.txt has no line for {} {}", ERAS[e], NAMES[k]));
    }
    out
}

/// Whether a cell of the series is nothing: nought, or empty.
fn nothing(cell: &str) -> bool {
    cell.is_empty() || cell.parse::<f64>().is_ok_and(|v| v == 0.0)
}

/// The series without the columns that are nothing in every row: those of
/// methods and goods that nothing makes in these months. The year and the
/// month are always kept, and every other column in the file's own order.
fn blind_series(csv: &str) -> String {
    let rows: Vec<Vec<&str>> = csv.split_terminator("\r\n").map(|r| r.split(',').collect()).collect();
    let n = rows.first().expect("the series have a header").len();
    let keep: Vec<usize> = (0..n).filter(|&k| k < 2 || rows[1..].iter().any(|r| !nothing(r[k]))).collect();
    let mut out = String::new();
    for r in &rows {
        assert_eq!(r.len(), n, "every row of the series has a cell for every column");
        out.push_str(&keep.iter().map(|&k| r[k]).collect::<Vec<_>>().join(","));
        out.push_str("\r\n");
    }
    out
}

/// The news to 1900 as `events.json` holds it, with each invention named by
/// its name in the content (`tech_name`) where the file gives its place in the
/// content's list.
fn blind_news(r: &RunResult) -> String {
    let mut news = serde_json::to_value(&r.events).expect("the news serialise");
    for item in news.as_array_mut().expect("the news are a list") {
        let tech = item.get("tech").and_then(|t| t.as_u64());
        if let Some(t) = tech {
            item["tech"] = serde_json::Value::String(r.content.tech_name[t as usize].clone());
        }
    }
    python_json_pretty(&news)
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

/// The series' header with the rows of the years up to `TURN` (`late` false)
/// or after it (`late` true).
fn era_rows(csv: &str, late: bool) -> String {
    let mut rows = csv.split_terminator("\r\n");
    let mut out = String::new();
    out.push_str(rows.next().expect("the series have a header"));
    out.push_str("\r\n");
    for row in rows {
        let year: i64 = row.split(',').next().and_then(|y| y.parse().ok()).expect("each row begins with its year");
        if (year > TURN) == late {
            out.push_str(row);
            out.push_str("\r\n");
        }
    }
    out
}

/// The news of the years up to `TURN` or after it, as `events.json` would
/// hold it.
fn era_news(r: &RunResult, late: bool) -> String {
    let events: Vec<_> = r.events.iter().filter(|e| (e.year >= (TURN + 1) as f64) == late).collect();
    python_json_pretty(&serde_json::to_value(&events).expect("the news serialise"))
}

/// The news up to `TURN` without the chronicle's marks of what a history would
/// keep (`Event::kept`), for setting the run to 2050 against the run to 1900.
/// The marks are made with the whole run in view, decade by decade, and the year
/// 1900 goes with the 1890s in a run that ends there and with the 1900s in one
/// that goes on (`chronicle::rank`), so an item of 1900 may be kept in one and
/// not the other. That is the history's reading of the news, not the news.
fn news_unmarked(r: &RunResult) -> u64 {
    let mut events: Vec<_> = r.events.iter().filter(|e| e.year < (TURN + 1) as f64).cloned().collect();
    for e in &mut events {
        e.kept = None;
    }
    fnv1a(python_json_pretty(&serde_json::to_value(&events).expect("the news serialise")).as_bytes())
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

/// One run of the island, to 1900 as the game has always ended (`to_2050`
/// false: the parameters are left as they were, so that the first era's hashes
/// are those of every run before the second era was added) or on to 2050.
#[allow(non_snake_case)]
fn run(to_2050: bool) -> RunResult {
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    if to_2050 {
        P.end_year = Some(END);
    }
    sim::run_with(&P, game_content(), false, true).expect("money is conserved")
}

fn dump(dir: &std::path::Path, csv: &str, news: &str, bin: &[u8]) {
    std::fs::create_dir_all(dir).expect("the dump directory can be made");
    std::fs::write(dir.join("series_monthly.csv"), csv).expect("series written");
    std::fs::write(dir.join("events.json"), news).expect("events written");
    std::fs::write(dir.join("run.bin"), bin).expect("run.bin written");
}

/// The run to 1900: the first era's three, blind to content added after 1900
/// (`blind_series`, `blind_news`); and, to set against the run to 2050, its
/// months hashed strictly and its news without the chronicle's marks
/// (`news_unmarked`).
fn first_era(dump_to: Option<&std::path::Path>) -> ([u64; FIRST], [u64; 3]) {
    let r = run(false);
    let csv = series_csv(&r);
    if let Some(dir) = dump_to {
        dump(dir, &csv, &events_json(&r), &run_bin(&r));
    }
    let (economy, books) = split_series(&blind_series(&csv));
    let blind = [fnv1a(economy.as_bytes()), fnv1a(books.as_bytes()), fnv1a(blind_news(&r).as_bytes())];
    let (economy, books) = split_series(&csv);
    (blind, [fnv1a(economy.as_bytes()), fnv1a(books.as_bytes()), news_unmarked(&r)])
}

/// The run to 2050: the second era's four (its months and news after 1900,
/// and its whole run.bin), and, to set against the run to 1900, the hashes of
/// its months up to 1900 and of its news up to 1900 without the marks.
fn to_2050(dump_to: Option<&std::path::Path>) -> ([u64; 4], [u64; 3]) {
    let r = run(true);
    let csv = series_csv(&r);
    let bin = run_bin(&r);
    if let Some(dir) = dump_to {
        dump(dir, &csv, &events_json(&r), &bin);
    }
    let (economy, books) = split_series(&era_rows(&csv, true));
    let late = [fnv1a(economy.as_bytes()), fnv1a(books.as_bytes()), fnv1a(era_news(&r, true).as_bytes()), fnv1a(&bin)];
    let (economy, books) = split_series(&era_rows(&csv, false));
    let early = [fnv1a(economy.as_bytes()), fnv1a(books.as_bytes()), news_unmarked(&r)];
    (late, early)
}

/// Which of the seven differ, in words, by era.
fn which(a: &Hashes, b: &Hashes) -> String {
    let mut parts = Vec::new();
    for e in 0..2 {
        let moved: Vec<&str> = (0..7).filter(|&i| SLOTS[i].0 == e && a[i] != b[i]).map(|i| MEANING[SLOTS[i].1]).collect();
        let list = match moved.len() {
            0 => "nothing".to_string(),
            1 => moved[0].to_string(),
            n => format!("{} and {}", moved[..n - 1].join(", "), moved[n - 1]),
        };
        parts.push(format!("in {}, {list}", ERAS[e]));
    }
    parts.join("; ")
}

/// The seven lines of `golden_game.txt` for these hashes.
fn lines(h: &Hashes) -> String {
    (0..7).map(|i| format!("{} {:<8} 0x{:016x}\n", ERAS[SLOTS[i].0], NAMES[SLOTS[i].1], h[i])).collect()
}

/// Words wrapped to lines of at most `width` characters.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

/// `golden_game.txt` written again with these hashes: every comment above the
/// hashes kept as it is, the history of earlier `# Taken:` lines with it, `why`
/// added below them as the newest, and the hashes below that, in the file's own
/// line ends. (Until 20 November 2026 this replaced the history with `why`
/// alone, and the history was put back by hand from git.)
fn write_file(h: &Hashes, why: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("golden_game.txt");
    let old = std::fs::read_to_string(&path).expect("golden_game.txt can be read");
    let nl = if old.contains("\r\n") { "\r\n" } else { "\n" };
    let mut text = String::new();
    for line in old.lines() {
        let t = line.trim();
        if !t.is_empty() && !t.starts_with('#') {
            break;
        }
        text.push_str(line);
        text.push_str(nl);
    }
    for line in wrap(&format!("Taken: {why}"), 76) {
        text.push_str("# ");
        text.push_str(&line);
        text.push_str(nl);
    }
    for line in lines(h).lines() {
        text.push_str(line);
        text.push_str(nl);
    }
    std::fs::write(&path, text).expect("golden_game.txt can be written");
}

#[test]
fn seed_7_under_the_games_rules_is_unchanged() {
    let want = stored();
    let dump_to = std::env::var("ISOSTATE_GOLDEN_DUMP").ok().map(std::path::PathBuf::from);
    // the run to 1900 and two runs to 2050 at once: each has its own state, so the two
    // runs to 2050 must agree to the last bit
    let ((early, early_strict), (late, late_early), (again, again_early)) = std::thread::scope(|s| {
        let a = s.spawn(|| first_era(dump_to.as_ref().map(|d| d.join("game6")).as_deref()));
        let b = s.spawn(|| to_2050(dump_to.as_ref().map(|d| d.join("game6-2050")).as_deref()));
        let c = s.spawn(|| to_2050(None));
        (
            a.join().expect("the run to 1900 finished"),
            b.join().expect("the first run to 2050 finished"),
            c.join().expect("the second run to 2050 finished"),
        )
    });
    let glue = |e: [u64; FIRST], l: [u64; 4]| -> Hashes { std::array::from_fn(|i| if i < FIRST { e[i] } else { l[i - FIRST] }) };
    let first = glue(early, late);
    if (late, late_early) != (again, again_early) {
        let one = glue(late_early, late);
        let two = glue(again_early, again);
        panic!(
            "\n\nThe game's run is not repeatable: two runs of seed 7 to 2050 under game_params() with six families, \
             side by side in one process, disagree: {}.\n\
             first\n{}second\n{}\n\
             (The first era's three are here each run's months and news to 1900, hashed strictly, the news without \
             the chronicle's marks.)\n\
             This is a fault whatever the stored hashes say. Something in the run reads what is not the rules and the \
             island: a clock, a thread, an environment variable, a global, or the order of a hash map that is walked. \
             The date in run.bin is already set aside. Find it before taking any hash.\n",
            which(&one, &two),
            lines(&one),
            lines(&two)
        );
    }
    if late_early != early_strict {
        let moved: Vec<&str> = (0..3).filter(|&k| late_early[k] != early_strict[k]).map(|k| MEANING[k]).collect();
        panic!(
            "\n\nThe run to 2050 does not pass through the run to 1900: seed 7 under game_params() with six families, \
             run on to 2050, differs in its years to 1900 from the same island run to 1900, in {} (the news is set \
             side by side without the chronicle's marks of what a history would keep, which are made with the whole \
             run in view).\n\
             Something in the run reads the year it will end (`end_year`, the record's length, the number of years \
             left) before that year comes. The game is played on to 2050 in the viewer, so that is a fault: the \
             played century must not depend on how far the run will go. ISOSTATE_GOLDEN_DUMP=DIR writes both runs' \
             files (game6 and game6-2050) for a diff.\n",
            moved.join(", ")
        );
    }
    if let Ok(why) = std::env::var("ISOSTATE_GOLDEN_WRITE") {
        assert!(!why.trim().is_empty(), "ISOSTATE_GOLDEN_WRITE must say what moved the hashes and why");
        write_file(&first, why.trim());
        let moved = if first == want { "nothing".to_string() } else { which(&first, &want) };
        eprintln!(
            "\nISOSTATE_GOLDEN_WRITE is set, so nothing was checked against the file: the runs are repeatable and the \
             run to 2050 passes through the run to 1900, and the seven hashes were written to \
             crates/isostate-core/tests/golden_game.txt.\nmoved: {moved}"
        );
        for i in 0..7 {
            let (e, k) = SLOTS[i];
            eprintln!("  {} {:<8} 0x{:016x} -> 0x{:016x}{}", ERAS[e], NAMES[k], want[i], first[i], if want[i] != first[i] { "  moved" } else { "" });
        }
        return;
    }
    if std::env::var("ISOSTATE_GOLDEN_PRINT").is_ok() {
        panic!(
            "\n\nISOSTATE_GOLDEN_PRINT is set, so nothing was checked. The runs agree, and give\n\n{}\n\
             Paste these seven lines over those in crates/isostate-core/tests/golden_game.txt, and say in the commit \
             message which moved and why (docs/rules/README.md, \"The game's golden test\"); or let \
             ISOSTATE_GOLDEN_WRITE=\"why\" do both.\n",
            lines(&first)
        );
    }
    if first != want {
        let only_late = first[..FIRST] == want[..FIRST];
        panic!(
            "\n\nThe game moved: seed 7 under game_params() and game_content() with six families no longer gives the \
             run it gave. What moved: {}.{}\n\
             got\n{}expected (crates/isostate-core/tests/golden_game.txt)\n{}\n\
             The runs agree with each other, so the run is repeatable and the rules or the code changed.\n\
             If that was meant (a stage of the rules pass or of the run to 2050, a rule mended or retuned, a new rule \
             switched on in game_params()), take the new hashes on the merged tree with\n\n    \
             ISOSTATE_GOLDEN_WRITE=\"what moved them and why\" cargo test -p isostate-core --test golden_game -- --nocapture\n\n\
             and say in the commit message which moved and why.\n\
             If nothing was meant to move, the change is at fault, not the file. Look for a hook moved across a \
             read or a write of the month (the comment above econ::step_month lists the order that must hold), \
             a rule acting where it should not, or, if the economy moved when only the books were touched, the \
             books touching the economy. ISOSTATE_GOLDEN_DUMP=DIR writes the runs' files for a diff \
             against the same from the commit before.\n",
            which(&first, &want),
            if only_late { " The played century, 1750 to 1900, held: only the years after 1900 moved." } else { "" },
            lines(&first),
            lines(&want)
        );
    }
}
