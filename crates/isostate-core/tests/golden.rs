//! Golden runs: seed 7 to 1900 under `default_params()`, without families and
//! with six, must come out byte for byte as they did when these hashes were
//! taken (PLAN.md 1.5).
//!
//! `default_params()` and `default_content()` are the port of the Python. Every
//! mechanism added since is switched off under them and on under the game's
//! rules (`game_params()`), so nothing added to the game may move a single
//! digit here. What is hashed is exactly what `isostate-core run` writes: the
//! text of `series_monthly.csv` and of `events.json`. The hash is 64-bit FNV-1a,
//! small enough to write out here and good enough to notice any change. A
//! third, instant check holds `params.json` to the Python's file.
//!
//! The constants are those of commit a5ea2ea: when they were taken, the two
//! strings of each run were written out and compared byte for byte (`cmp`)
//! with the files that commit's `isostate-core run --seed 7` and
//! `--seed 7 --families 6` wrote.
//!
//! When the port is changed on purpose, take the new constants with
//!
//! ```text
//! ISOSTATE_GOLDEN_PRINT=1 cargo test -p isostate-core --release --test golden
//! ```
//!
//! Nothing is checked then: each of the three tests stops with the line to
//! paste over its constant below. Say in the commit message why the port moved.
//!
//! To look at a change rather than only detect it, run
//!
//! ```text
//! ISOSTATE_GOLDEN_DUMP=some/dir cargo test -p isostate-core --release --test golden
//! ```
//!
//! before and after: it writes `port/` and `families6/` under that directory,
//! each with `series_monthly.csv` and `events.json`, ready for a diff.

use isostate_core::cli::{events_json, python_json_pretty, series_csv};
use isostate_core::sim::{self, RunResult};
use isostate_core::{default_content, default_params};

/// (series_monthly.csv, events.json) of seed 7 to 1900, no families.
const PORT: (u64, u64) = (0x5a6d28c86afe7219, 0xa9fdb756ceaa30e5);
/// The same with `n_families = 6`.
const FAMILIES6: (u64, u64) = (0xe39e9e27bc66b1b6, 0xb73cb3a31964bf41);

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// With `ISOSTATE_GOLDEN_PRINT` set nothing is checked: each test stops here with
/// the constant this code gives. A panic, because the harness hides what a
/// passing test prints, and so that a run made to take new hashes can never be
/// mistaken for one that passed.
fn taken(line: &str) -> ! {
    panic!("\n\nISOSTATE_GOLDEN_PRINT is set, so nothing was checked. This code gives\n\n    {line}\n\n");
}

#[allow(non_snake_case)]
fn golden(name: &str, n_families: usize) -> RunResult {
    let mut P = default_params();
    P.seed = 7;
    P.n_families = n_families;
    let r = sim::run_with(&P, default_content(), false, true).expect("money is conserved");
    if let Ok(dir) = std::env::var("ISOSTATE_GOLDEN_DUMP") {
        let dir = std::path::Path::new(&dir).join(name);
        std::fs::create_dir_all(&dir).expect("the dump directory can be made");
        std::fs::write(dir.join("series_monthly.csv"), series_csv(&r)).expect("series written");
        std::fs::write(dir.join("events.json"), events_json(&r)).expect("events written");
    }
    r
}

fn check(name: &str, const_name: &str, n_families: usize, want: (u64, u64)) {
    let r = golden(name, n_families);
    let got = (fnv1a(series_csv(&r).as_bytes()), fnv1a(events_json(&r).as_bytes()));
    if std::env::var("ISOSTATE_GOLDEN_PRINT").is_ok() {
        taken(&format!("const {const_name}: (u64, u64) = (0x{:016x}, 0x{:016x});", got.0, got.1));
    }
    let what = match (got.0 == want.0, got.1 == want.1) {
        (true, true) => return,
        (false, true) => "the monthly series differ (the news is the same)",
        (true, false) => "the news differs (the monthly series are the same)",
        (false, false) => "the monthly series and the news both differ",
    };
    panic!(
        "\n\nThe port's behaviour changed: seed 7 to 1900 under default_params() with n_families = {n_families} no longer \
         gives the run it gave; {what}.\n\
         got      (0x{:016x}, 0x{:016x})\n\
         expected (0x{:016x}, 0x{:016x})\n\n\
         Either some new mechanism is leaking into the default rules (it must do nothing at all, not even draw a random \
         number, unless game_params() or a --set switches it on), or the port itself was changed on purpose.\n\
         If it was on purpose, take the new hashes with\n\n    \
         ISOSTATE_GOLDEN_PRINT=1 cargo test -p isostate-core --release --test golden\n\n\
         (each test then stops with its constant), paste them over the constants of the same names in \
         crates/isostate-core/tests/golden.rs, and say in the commit message why the port moved. \
         ISOSTATE_GOLDEN_DUMP=DIR writes the two files of each run for a diff.\n",
        got.0, got.1, want.0, want.1
    );
}

/// `params.json` of a port run: the Python's file, with no word of any mechanism
/// added since.
const PARAMS_JSON: u64 = 0x9bf2e7c672cc5afd;

#[test]
fn the_ports_params_json_is_unchanged() {
    let text = python_json_pretty(&serde_json::to_value(default_params()).unwrap());
    if let Ok(dir) = std::env::var("ISOSTATE_GOLDEN_DUMP") {
        std::fs::create_dir_all(&dir).expect("the dump directory can be made");
        std::fs::write(std::path::Path::new(&dir).join("params.json"), &text).expect("params written");
    }
    let got = fnv1a(text.as_bytes());
    if std::env::var("ISOSTATE_GOLDEN_PRINT").is_ok() {
        taken(&format!("const PARAMS_JSON: u64 = 0x{got:016x};"));
    }
    assert!(
        got == PARAMS_JSON,
        "\n\nparams.json of a run under default_params() changed (got 0x{got:016x}, expected 0x{PARAMS_JSON:016x}).\n\
         A field added to Params for a new mechanism must be left out of the file while it is switched off:\n\
         #[serde(default, skip_serializing_if = \"is_default\")]. If the port's own numbers were changed on purpose,\n\
         ISOSTATE_GOLDEN_PRINT=1 prints the new constant (see the top of this file).\n\n{text}\n"
    );
}

#[test]
fn seed_7_under_the_port_is_unchanged() {
    check("port", "PORT", 0, PORT);
}

#[test]
fn seed_7_with_six_families_is_unchanged() {
    check("families6", "FAMILIES6", 6, FAMILIES6);
}
