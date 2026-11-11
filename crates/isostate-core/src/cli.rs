//! Command line: run a scenario and save it. Python: `cli.py`.
//!
//! ```text
//! isostate-core run --seed 7 --regime landlords --out runs/landlords [--end-year 1800] [--lite]
//! ```
//!
//! writes run.bin, series_monthly.csv, events.json and params.json like
//! `python -m isostate run`. Embedding runs into the HTML viewer stays in Python:
//! `python3 -m isostate html runs/landlords --out dist/x.html` reads the run.bin.
//!
//! `--rules port` (the default) runs the port of the Python; `--rules game` runs
//! the rules a live game is played under (`params::game_params`).
//!
//! `isostate-core sweep` runs many seeds against many settings and tabulates
//! their stylised facts; see `sweep.rs`.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::content::{Content, load_content};
use crate::params::{Params, RentRegime, Rules};
use crate::record::{self, clean, ensure_ascii};
use crate::sim::{RunResult, run_with};
use crate::util::fmt_g6;

pub fn label_for(regime: RentRegime) -> &'static str {
    match regime {
        RentRegime::Landlords => "Landlords keep the rent",
        RentRegime::Commons => "Rent paid out as a dividend",
    }
}

/// JSON with one-space indentation and ASCII escapes, like `json.dumps(_clean(x), indent=1)`.
pub fn python_json_pretty(v: &serde_json::Value) -> String {
    let mut v = v.clone();
    clean(&mut v);
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    serde::Serialize::serialize(&v, &mut ser).expect("json");
    ensure_ascii(&String::from_utf8(buf).expect("utf8"))
}

/// The monthly series as Python's csv writer leaves them (CRLF line ends, `%.6g`).
pub fn series_csv(result: &RunResult) -> String {
    let s = &result.state.series;
    let y0 = result.content.start_year;
    let mut out = String::new();
    out.push_str("year,month");
    for n in &s.names {
        out.push(',');
        out.push_str(n);
    }
    out.push_str("\r\n");
    let n = s.data.first().map_or(0, |d| d.len());
    for t in 0..n {
        out.push_str(&format!("{},{}", y0 + (t / 12) as i64, t % 12 + 1));
        for d in &s.data {
            out.push(',');
            out.push_str(&fmt_g6(d[t]));
        }
        out.push_str("\r\n");
    }
    out
}

/// The news as `events.json` holds it.
pub fn events_json(result: &RunResult) -> String {
    python_json_pretty(&serde_json::to_value(&result.events).unwrap())
}

/// Write run.bin, series_monthly.csv, events.json and params.json for one finished run.
pub fn save_run(result: &RunResult, out: &Path, lite: bool) -> std::io::Result<()> {
    std::fs::create_dir_all(out)?;
    let P = &result.params;
    let blob = record::pack(&result.state, &result.content, P, &result.events, &result.rec, label_for(P.rent_regime), lite);
    std::fs::write(out.join("run.bin"), blob)?;
    std::fs::write(out.join("series_monthly.csv"), series_csv(result))?;
    std::fs::write(out.join("events.json"), events_json(result))?;
    std::fs::write(out.join("params.json"), python_json_pretty(&serde_json::to_value(P).unwrap()))?;
    Ok(())
}

pub fn run_scenario(P: &Params, content: Content, out: &Path, progress: bool, lite: bool) -> Result<RunResult, String> {
    let result = run_with(P, content, progress, true)?;
    save_run(&result, out, lite).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(result)
}

const USAGE: &str = "usage: isostate-core run --out DIR [--seed N] [--rules port|game] [--regime landlords|commons] [--end-year Y]
                         [--lite] [--families N] [--params FILE.toml|FILE.json] [--content content.toml] [--quiet]";

/// `--rules port|game`.
pub fn parse_rules(v: &str) -> Result<Rules, String> {
    Rules::parse(v).ok_or_else(|| format!("--rules: {v:?} is not port or game"))
}

/// A run that ends before it starts has no years in it: `sim::start` would ask
/// for series of a negative length. Refused here, in words.
pub fn check_end_year(P: &Params, content: &Content) -> Result<(), String> {
    let end = P.end_year.unwrap_or(content.end_year);
    if end < content.start_year {
        return Err(format!("the end year, {end}, is before the start year, {}", content.start_year));
    }
    Ok(())
}

pub fn main_with_args(args: &[String]) -> Result<(), String> {
    let mut it = args.iter();
    match it.next().map(String::as_str) {
        Some("run") => {}
        Some("sweep") => return crate::sweep::main_with_args(&args[1..]),
        Some("-h") | Some("--help") | None => {
            println!("{USAGE}\n{}", crate::sweep::USAGE);
            return Ok(());
        }
        Some(other) => return Err(format!("unknown command {other:?}\n{USAGE}\n{}", crate::sweep::USAGE)),
    }
    let mut seed: Option<u64> = None;
    let mut rules = Rules::Port;
    let mut regime: Option<RentRegime> = None;
    let mut out: Option<PathBuf> = None;
    let mut end_year: Option<i64> = None;
    let mut n_families: Option<usize> = None;
    let mut lite = false;
    let mut quiet = false;
    let mut params_file: Option<PathBuf> = None;
    let mut content_file: Option<PathBuf> = None;
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "--seed" => seed = Some(value("--seed")?.parse().map_err(|e| format!("--seed: {e}"))?),
            "--rules" => rules = parse_rules(&value("--rules")?)?,
            "--regime" => {
                let v = value("--regime")?;
                regime = Some(RentRegime::parse(&v).ok_or_else(|| format!("--regime: {v:?} is not landlords or commons"))?);
            }
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            "--end-year" => end_year = Some(value("--end-year")?.parse().map_err(|e| format!("--end-year: {e}"))?),
            "--families" => n_families = Some(value("--families")?.parse().map_err(|e| format!("--families: {e}"))?),
            "--lite" => lite = true,
            "--quiet" => quiet = true,
            "--params" => params_file = Some(PathBuf::from(value("--params")?)),
            "--content" => content_file = Some(PathBuf::from(value("--content")?)),
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            other => return Err(format!("unknown option {other:?}\n{USAGE}")),
        }
    }
    let out = out.ok_or_else(|| format!("--out is required\n{USAGE}"))?;
    // a params file overrides the chosen rules, field by field
    let mut P = match &params_file {
        Some(p) => rules.params().load_over(p)?,
        None => rules.params(),
    };
    P.seed = seed.unwrap_or(if params_file.is_some() { P.seed } else { 7 });
    if let Some(r) = regime {
        P.rent_regime = r;
    }
    if end_year.is_some() {
        P.end_year = end_year;
    }
    if let Some(n) = n_families {
        P.n_families = n;
    }
    let content = match &content_file {
        Some(p) => load_content(p)?,
        None => rules.content(),
    };
    check_end_year(&P, &content)?;
    let t0 = std::time::Instant::now();
    run_scenario(&P, content, &out, !quiet, lite)?;
    if !quiet {
        eprintln!("ran in {:.2} s", t0.elapsed().as_secs_f64());
    }
    let _ = std::io::stdout().write_all(b"done\n");
    Ok(())
}

pub fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = main_with_args(&args) {
        eprintln!("{e}");
        std::process::exit(2);
    }
}
