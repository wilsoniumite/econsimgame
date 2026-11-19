//! Many runs at once: every seed against every setting (PLAN.md 1.1 and 1.3). Not in the Python.
//!
//! ```text
//! isostate-core sweep --out DIR [--seeds 1-8 | 1,2,5] [--rules port|game] [--families N] [--end-year Y]
//!                     [--set key=v1[,v2...]]... [--params FILE] [--content FILE] [--jobs N]
//!                     [--compare OTHER_DIR] [--quiet]
//! ```
//!
//! One island proves little: a claim about the model should hold in most seeds.
//! The sweep runs every seed against every combination of the `--set` values, a
//! few runs at a time on their own threads, and reduces each run to its stylised
//! facts (`facts.rs`). It writes
//!
//! - `annual.csv`: one row per run and year, every observer series averaged over
//!   the year's twelve months;
//! - `facts.csv` and `facts.json`: one row or object per run;
//! - `summary.txt`: the table it prints, each fact's mean and range across seeds,
//!   one column per setting;
//! - `scorecard.txt`: each setting read against the aims the documents state
//!   (`scorecard.rs`, `docs/rules/scorecard.md`);
//! - `README.txt`: the command line and what was run.
//!
//! `--set` takes any field of `Params` by name, and a dotted path into a
//! mechanism's own struct (`canals.cost=0.2,0.3`). The value goes in through the
//! parameters' JSON form, so a field added tomorrow can be swept with no table of
//! names to keep up to date here. `--compare OTHER_DIR` sets the new means beside
//! those of an earlier sweep, seed for seed where the two have seeds in common,
//! counts the seeds in which each fact rose and fell, and marks the facts whose
//! seed ranges do not overlap.
//!
//! Whatever is wrong with the command line is refused before the first run and
//! before anything is written. A sweep already in `DIR` is replaced; `--compare`
//! may name `DIR` itself, since the earlier sweep is read before the first run.
//!
//! A run that panics or fails the monthly money check is reported as failed and
//! the others carry on.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::cli::{check_end_year, parse_rules};
use crate::content::{Content, load_content};
use crate::facts::{self, Fact, Facts, GROUPS, Kind, NEWS, SPEED};
use crate::params::{Params, Rules};
use crate::sim;
use crate::util::{fmt_g6, fmt_thousands_f0};

pub const USAGE: &str = "usage: isostate-core sweep --out DIR [--seeds 1-8 | 1,2,5] [--rules port|game] [--families N] [--end-year Y]
                           [--set key=v1[,v2...]]... [--params FILE.toml|FILE.json] [--content content.toml]
                           [--jobs N] [--compare OTHER_DIR] [--quiet]";

/// What the command line asked for.
#[derive(Clone, Debug)]
pub struct Options {
    pub out: PathBuf,
    pub seeds: Vec<u64>,
    pub rules: Rules,
    pub families: Option<usize>,
    pub end_year: Option<i64>,
    /// Each `--set`: the key and its values as typed.
    pub sets: Vec<(String, Vec<String>)>,
    pub params_file: Option<PathBuf>,
    pub content_file: Option<PathBuf>,
    pub jobs: Option<usize>,
    pub compare: Option<PathBuf>,
    pub quiet: bool,
    /// The command line, for README.txt.
    pub command: String,
}

/// One combination of the swept values.
#[derive(Clone, Debug)]
pub struct Setting {
    /// `land_theta=2 hurdle=0.1`; the rules' name when nothing is swept.
    pub label: String,
    /// One value per swept key, as typed.
    pub values: Vec<String>,
    pub params: Params,
}

/// What a finished run leaves behind.
#[derive(Clone, Debug)]
pub struct Done {
    pub facts: Facts,
    /// `[series][year]`: the yearly means of every observer series.
    pub annual: Vec<Vec<f64>>,
    pub names: Vec<String>,
    pub start_year: i64,
}

#[derive(Clone, Debug)]
pub struct Run {
    pub id: usize,
    pub seed: u64,
    /// Index into `Sweep::settings`.
    pub setting: usize,
    /// The facts, or why the run failed.
    pub outcome: Result<Done, String>,
    pub seconds: f64,
}

/// A finished sweep.
#[derive(Clone, Debug)]
pub struct Sweep {
    pub rules: Rules,
    pub seeds: Vec<u64>,
    /// The swept keys, in the order given.
    pub keys: Vec<String>,
    pub settings: Vec<Setting>,
    pub runs: Vec<Run>,
    /// Every fact any run has: (name, heading, kind), in printing order.
    pub fact_index: Vec<(String, &'static str, Kind)>,
    pub jobs: usize,
    pub seconds: f64,
}

// ---------------------------------------------------------------- the command line

/// More seeds than this in one `--seeds` is taken for a slip of the hand (`1-80000`
/// for `1-8`): ten thousand whole runs are already most of a day.
pub const MAX_SEEDS: u64 = 10_000;

/// `1-8`, `1,2,5` or a mixture (`1-3,7`).
pub fn parse_seeds(s: &str) -> Result<Vec<u64>, String> {
    let bad = |e: std::num::ParseIntError| format!("--seeds {s:?}: {e}; write it as 1-8 or 1,2,5");
    let mut out = Vec::new();
    for part in s.split(',') {
        match part.split_once('-') {
            Some((a, b)) => {
                let (a, b): (u64, u64) = (a.trim().parse().map_err(bad)?, b.trim().parse().map_err(bad)?);
                if a > b {
                    return Err(format!("--seeds {s:?}: {a}-{b} runs backwards"));
                }
                // checked before the range is laid out: 0-99999999999 would not fit in memory
                if b - a >= MAX_SEEDS || out.len() as u64 + (b - a) >= MAX_SEEDS {
                    return Err(format!("--seeds {s:?}: more than {MAX_SEEDS} seeds"));
                }
                out.extend(a..=b);
            }
            None => out.push(part.trim().parse().map_err(bad)?),
        }
    }
    let mut seen = HashSet::new();
    out.retain(|x| seen.insert(*x));
    if out.len() as u64 > MAX_SEEDS {
        return Err(format!("--seeds {s:?}: more than {MAX_SEEDS} seeds"));
    }
    Ok(out)
}

/// The values of one `--set`, split at the commas outside brackets and quotes, so
/// `road_thresholds=[1500,9000,40000],[1000,6000,30000]` is two values.
pub fn split_values(s: &str) -> Vec<String> {
    let (mut out, mut cur, mut depth, mut quoted) = (Vec::new(), String::new(), 0i32, false);
    for ch in s.chars() {
        match ch {
            '"' => quoted = !quoted,
            '[' | '{' if !quoted => depth += 1,
            ']' | '}' if !quoted => depth -= 1,
            ',' if !quoted && depth == 0 => {
                out.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(ch);
    }
    out.push(cur);
    out.iter().map(|v| v.trim().to_string()).collect()
}

/// A value as typed: JSON where it parses as JSON (`2.5`, `true`, `[1,2]`), else
/// a plain word (`commons`).
pub fn parse_value(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.to_string()))
}

pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        out: PathBuf::new(),
        seeds: (1..=8).collect(),
        rules: Rules::Port,
        families: None,
        end_year: None,
        sets: Vec::new(),
        params_file: None,
        content_file: None,
        jobs: None,
        compare: None,
        quiet: false,
        command: format!("isostate-core sweep {}", args.join(" ")),
    };
    let mut out: Option<PathBuf> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value\n{USAGE}"));
        match a.as_str() {
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            "--seeds" => o.seeds = parse_seeds(&value("--seeds")?)?,
            "--rules" => o.rules = parse_rules(&value("--rules")?)?,
            "--families" => o.families = Some(value("--families")?.parse().map_err(|e| format!("--families: {e}"))?),
            "--end-year" => o.end_year = Some(value("--end-year")?.parse().map_err(|e| format!("--end-year: {e}"))?),
            "--set" => {
                let v = value("--set")?;
                let (key, values) = v.split_once('=').ok_or_else(|| format!("--set {v:?}: write it as key=value or key=v1,v2"))?;
                let (key, values) = (key.trim().to_string(), split_values(values));
                if key.is_empty() || values.iter().any(|x| x.is_empty()) {
                    return Err(format!("--set {v:?}: write it as key=value or key=v1,v2"));
                }
                if key == "seed" {
                    return Err("--set seed: the seeds are chosen with --seeds".to_string());
                }
                if o.sets.iter().any(|(k, _)| *k == key) {
                    return Err(format!("--set {key}: given twice; list its values in one --set, separated by commas"));
                }
                // the same value twice would be two columns with one name
                if let Some(twice) = values.iter().enumerate().find(|(k, x)| values[..*k].contains(x)).map(|(_, x)| x) {
                    return Err(format!("--set {key}: {twice} is listed twice"));
                }
                o.sets.push((key, values));
            }
            "--params" => o.params_file = Some(PathBuf::from(value("--params")?)),
            "--content" => o.content_file = Some(PathBuf::from(value("--content")?)),
            "--jobs" => {
                let n: usize = value("--jobs")?.parse().map_err(|e| format!("--jobs: {e}"))?;
                o.jobs = Some(n.max(1));
            }
            "--compare" => o.compare = Some(PathBuf::from(value("--compare")?)),
            "--quiet" => o.quiet = true,
            other => return Err(format!("unknown option {other:?}\n{USAGE}")),
        }
    }
    o.out = out.ok_or_else(|| format!("--out is required\n{USAGE}"))?;
    if o.seeds.is_empty() {
        return Err("--seeds: no seeds".to_string());
    }
    Ok(o)
}

// ---------------------------------------------------------------- setting a parameter by name

fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut diag = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let next = (row[j] + 1).min(row[j - 1] + 1).min(diag + (a[i - 1] != b[j - 1]) as usize);
            diag = row[j];
            row[j] = next;
        }
    }
    row[b.len()]
}

/// The known names closest to a misspelt one: a few letters off, or one inside the other.
fn near_matches<'a>(bad: &str, known: &[&'a str]) -> Vec<&'a str> {
    let len = bad.chars().count();
    let limit = (len / 3).clamp(2, 3).min(len.saturating_sub(1));
    let mut scored: Vec<(usize, &str)> = known
        .iter()
        .filter_map(|&k| {
            let d = edit_distance(bad, k);
            // the one-letter parameters W and H are inside most words: a name counts from three letters
            let inside = bad.len() >= 3 && (k.contains(bad) || (k.len() >= 3 && bad.contains(k)));
            (d <= limit || inside).then_some((d, k))
        })
        .collect();
    scored.sort();
    // land: with a name a letter or two off, a shorter name that merely sits inside
    // the misspelt one (`land` in `land_thetta`) is noise, and is not offered
    if scored.iter().any(|&(d, _)| d <= limit) {
        scored.retain(|&(d, k)| d <= limit || k.contains(bad));
    }
    scored.into_iter().take(6).map(|(_, k)| k).collect()
}

/// serde's refusal in the words of the command line. Its message for a field a
/// struct does not have lists the fields it does have, between backticks, which
/// is where the near matches come from.
fn explain(key: &str, value: &Value, msg: &str) -> String {
    let Some(rest) = msg.strip_prefix("unknown field `") else {
        return format!("--set {key}={value}: {msg}");
    };
    let bad = rest.split('`').next().unwrap_or("");
    let listed = rest.split_once("expected").map_or("", |(_, tail)| tail);
    let known: Vec<&str> = listed.split('`').skip(1).step_by(2).collect();
    let parts: Vec<&str> = key.split('.').collect();
    let owner = parts.iter().position(|p| *p == bad).map(|d| parts[..d].join(".")).unwrap_or_default();
    let place = if owner.is_empty() { String::new() } else { format!(" in {owner}") };
    let near = near_matches(bad, &known);
    if known.is_empty() {
        format!("--set {key}: there is no parameter {bad:?}{place}")
    } else if near.is_empty() {
        format!("--set {key}: there is no parameter {bad:?}{place}. The parameters{place} are: {}", known.join(", "))
    } else {
        format!("--set {key}: there is no parameter {bad:?}{place}. Did you mean {}?", near.join(" or "))
    }
}

/// `base` with the field at the dotted path `key` set to `value`. The change is
/// made to the JSON form and read back, so every field serde knows can be set and
/// a wrong name or a wrong type is refused in serde's own words.
pub fn set_field<T: Serialize + DeserializeOwned>(base: &T, key: &str, value: &Value) -> Result<T, String> {
    let mut doc = serde_json::to_value(base).map_err(|e| format!("--set {key}: {e}"))?;
    let parts: Vec<&str> = key.split('.').collect();
    if parts.iter().any(|p| p.is_empty()) {
        return Err(format!("--set {key}: not a parameter name"));
    }
    let mut cur = &mut doc;
    for (d, part) in parts.iter().enumerate() {
        let Some(obj) = cur.as_object_mut() else {
            return Err(format!("--set {key}: {} is a single value, not a group of parameters", parts[..d].join(".")));
        };
        if d + 1 == parts.len() {
            obj.insert(part.to_string(), value.clone());
            break;
        }
        // a mechanism left at its default is not written out: start it from there
        cur = obj.entry(part.to_string()).or_insert_with(|| json!({}));
    }
    // a setting retired in stage 5 of the pass over the rules is refused by
    // what it became, not as a name nobody has heard of
    if let Some(why) = crate::params::retired(&doc) {
        return Err(format!("--set {key}: {why}"));
    }
    serde_json::from_value(doc).map_err(|e| explain(key, value, &e.to_string()))
}

/// The base parameters, then one `Setting` per combination of the `--set` values
/// (the first key varies slowest).
pub fn settings(o: &Options) -> Result<Vec<Setting>, String> {
    let mut base = match &o.params_file {
        Some(p) => o.rules.params().load_over(p)?,
        None => o.rules.params(),
    };
    if let Some(n) = o.families {
        base.n_families = n;
    }
    if o.end_year.is_some() {
        base.end_year = o.end_year;
    }
    let mut combos: Vec<Vec<String>> = vec![Vec::new()];
    for (_, values) in &o.sets {
        combos = combos.iter().flat_map(|c| values.iter().map(move |v| [c.clone(), vec![v.clone()]].concat())).collect();
    }
    let mut out = Vec::new();
    for values in combos {
        let mut P = base.clone();
        for ((key, _), v) in o.sets.iter().zip(&values) {
            P = set_field(&P, key, &parse_value(v))?;
        }
        // what a --params file is refused for is refused when swept, before the first run
        P.check()?;
        let label = if values.is_empty() {
            // nothing swept: name the column after what was run (the families may come from --params)
            let families = match base.n_families {
                0 => String::new(),
                1 => ", 1 family".to_string(),
                n => format!(", {n} families"),
            };
            format!("{} rules{families}", o.rules.as_str())
        } else {
            o.sets.iter().zip(&values).map(|((k, _), v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")
        };
        out.push(Setting { label, values, params: P });
    }
    Ok(out)
}

// ---------------------------------------------------------------- running

/// The processor time this thread has had so far, in seconds (user and system
/// together), where the platform says: on Windows. None elsewhere, for now.
///
/// A run is one thread from start to finish, so the difference across a run is
/// the time the processor spent on it, which other work on the machine moves
/// much less than it moves `seconds`, the time on the clock (R6), but still moves:
/// threads that share a core, and a slower clock when the machine is hot.
#[cfg(windows)]
pub fn thread_cpu_seconds() -> Option<f64> {
    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentThread() -> *mut std::ffi::c_void;
        fn GetThreadTimes(thread: *mut std::ffi::c_void, created: *mut FileTime, ended: *mut FileTime, system: *mut FileTime, user: *mut FileTime) -> i32;
    }
    let (mut created, mut ended, mut system, mut user) = (FileTime::default(), FileTime::default(), FileTime::default(), FileTime::default());
    // SAFETY: GetCurrentThread gives a handle to this thread that needs no closing, and
    // the four times written are ours, alive for the call
    let ok = unsafe { GetThreadTimes(GetCurrentThread(), &mut created, &mut ended, &mut system, &mut user) };
    // in hundreds of nanoseconds
    let secs = |f: &FileTime| ((u64::from(f.high) << 32) | u64::from(f.low)) as f64 * 1e-7;
    (ok != 0).then(|| secs(&system) + secs(&user))
}

#[cfg(not(windows))]
pub fn thread_cpu_seconds() -> Option<f64> {
    None
}

/// One run, stepped year by year so the facts' gathering can look in between.
fn one_run(P: &Params, content: Content) -> Result<Done, String> {
    let t0 = Instant::now();
    let cpu0 = thread_cpu_seconds();
    let mut s = sim::start(P, content);
    let mut gathered = facts::Gathered::default();
    while s.year <= s.end_year {
        // sim::run_year, with the facts' notes taken after each month and the year's end
        facts::run_year_noting(&mut s, &mut gathered)?;
    }
    let r = sim::finish(s);
    let seconds = t0.elapsed().as_secs_f64();
    let cpu = match (cpu0, thread_cpu_seconds()) {
        (Some(a), Some(b)) => Some(b - a),
        _ => None,
    };
    let mut found = facts::stylised_facts(&r, &gathered, seconds);
    // beside `seconds`, the time on the clock: the processor's time on this run (R6)
    found.facts.push(Fact { name: "cpu_seconds".to_string(), group: SPEED, kind: Kind::Real, value: cpu });
    Ok(Done {
        facts: found,
        annual: facts::annual_means(&r),
        names: r.state.series.names.clone(),
        start_year: r.content.start_year,
    })
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()));
    format!("panicked: {}", text.unwrap_or_else(|| "no message".to_string()))
}

/// What a sweep will run, worked out and checked before anything runs or is written.
#[derive(Clone, Debug)]
pub struct Plan {
    pub settings: Vec<Setting>,
    pub content: Content,
}

/// The settings and the content of a sweep, or what is wrong with the command line.
pub fn plan(o: &Options) -> Result<Plan, String> {
    let settings = settings(o)?;
    let content = match &o.content_file {
        Some(p) => load_content(p)?,
        None => o.rules.content(),
    };
    for s in &settings {
        check_end_year(&s.params, &content)?;
    }
    Ok(Plan { settings, content })
}

/// Run the whole sweep. Nothing is written or printed here but the progress lines.
pub fn run(o: &Options, plan: Plan) -> Sweep {
    let Plan { settings, content } = plan;
    let todo: Vec<(usize, u64)> = (0..settings.len()).flat_map(|k| o.seeds.iter().map(move |&s| (k, s))).collect();
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    let jobs = o.jobs.unwrap_or((cores / 2).max(1)).min(todo.len()).max(1);
    let t0 = Instant::now();
    let next = AtomicUsize::new(0);
    let finished = AtomicUsize::new(0);
    let slots: Mutex<Vec<Option<Run>>> = Mutex::new(vec![None; todo.len()]);
    std::thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| {
                loop {
                    let id = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&(k, seed)) = todo.get(id) else { break };
                    let mut P = settings[k].params.clone();
                    P.seed = seed;
                    let t = Instant::now();
                    let content = content.clone();
                    let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| one_run(&P, content))) {
                        Ok(r) => r,
                        Err(payload) => Err(panic_message(payload)),
                    };
                    let seconds = t.elapsed().as_secs_f64();
                    if !o.quiet {
                        let n = finished.fetch_add(1, Ordering::Relaxed) + 1;
                        let cpu = |d: &Done| d.facts.get("cpu_seconds").map_or(String::new(), |c| format!(", {c:.1} s of processor"));
                        let how = match &outcome {
                            Ok(d) => format!("{seconds:.1} s on the clock{}", cpu(d)),
                            Err(e) => format!("FAILED: {e}"),
                        };
                        eprintln!("[{n}/{}] seed {seed}, {}: {how}", todo.len(), settings[k].label);
                    }
                    slots.lock().unwrap()[id] = Some(Run { id, seed, setting: k, outcome, seconds });
                }
            });
        }
    });
    let mut runs: Vec<Run> = slots.into_inner().unwrap().into_iter().map(|r| r.expect("every run reports")).collect();

    // news: one fact per kind any run had, so every run has the same columns
    let kinds: BTreeSet<String> = runs.iter().filter_map(|r| r.outcome.as_ref().ok()).flat_map(|d| d.facts.news.keys().cloned()).collect();
    for run in &mut runs {
        let Ok(done) = &mut run.outcome else { continue };
        let count = |k: &str| done.facts.news.get(k).copied().unwrap_or(0) as f64;
        let mut news: Vec<Fact> =
            kinds.iter().map(|k| Fact { name: format!("news_{k}"), group: NEWS, kind: Kind::Count, value: Some(count(k)) }).collect();
        let total: usize = done.facts.news.values().sum();
        news.push(Fact { name: "news_total".to_string(), group: NEWS, kind: Kind::Count, value: Some(total as f64) });
        done.facts.facts.extend(news);
    }
    let mut fact_index: Vec<(String, &'static str, Kind)> = Vec::new();
    let mut seen = HashSet::new();
    for done in runs.iter().filter_map(|r| r.outcome.as_ref().ok()) {
        for f in &done.facts.facts {
            if seen.insert(f.name.clone()) {
                fact_index.push((f.name.clone(), f.group, f.kind));
            }
        }
    }
    fact_index.sort_by_key(|(_, group, _)| GROUPS.iter().position(|g| g == group));
    Sweep {
        rules: o.rules,
        seeds: o.seeds.clone(),
        keys: o.sets.iter().map(|(k, _)| k.clone()).collect(),
        settings,
        runs,
        fact_index,
        jobs,
        seconds: t0.elapsed().as_secs_f64(),
    }
}

// ---------------------------------------------------------------- the files

fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\n']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() }
}

fn csv_num(v: f64) -> String {
    if v.is_finite() { fmt_g6(v) } else { String::new() }
}

/// `run,seed,<swept keys>` for one run.
fn csv_lead(sw: &Sweep, run: &Run) -> String {
    let mut s = format!("{},{}", run.id, run.seed);
    for v in &sw.settings[run.setting].values {
        s.push(',');
        s.push_str(&csv_cell(v));
    }
    s
}

fn csv_keys(sw: &Sweep) -> String {
    sw.keys.iter().map(|k| format!(",{}", csv_cell(k))).collect()
}

/// One row per run and year: every observer series averaged over the year's months.
pub fn annual_csv(sw: &Sweep) -> String {
    let done: Vec<(&Run, &Done)> = sw.runs.iter().filter_map(|r| r.outcome.as_ref().ok().map(|d| (r, d))).collect();
    let mut out = format!("run,seed{},year", csv_keys(sw));
    let Some((_, first)) = done.first() else { return out + "\n" };
    for n in &first.names {
        out.push(',');
        out.push_str(n);
    }
    out.push('\n');
    for (run, d) in done {
        let lead = csv_lead(sw, run);
        let years = d.annual.first().map_or(0, |a| a.len());
        for k in 0..years {
            let _ = write!(out, "{lead},{}", d.start_year + k as i64);
            for a in &d.annual {
                out.push(',');
                out.push_str(&csv_num(a[k]));
            }
            out.push('\n');
        }
    }
    out
}

/// One row per run: its facts, an empty cell where a fact has no value.
pub fn facts_csv(sw: &Sweep) -> String {
    let mut out = format!("run,seed{},status,error", csv_keys(sw));
    for (name, _, _) in &sw.fact_index {
        out.push(',');
        out.push_str(name);
    }
    out.push_str(",never_built\n");
    for run in &sw.runs {
        out.push_str(&csv_lead(sw, run));
        match &run.outcome {
            Ok(d) => {
                out.push_str(",ok,");
                let by_name: HashMap<&str, Option<f64>> = d.facts.facts.iter().map(|f| (f.name.as_str(), f.value)).collect();
                for (name, _, _) in &sw.fact_index {
                    out.push(',');
                    if let Some(Some(v)) = by_name.get(name.as_str()) {
                        // in full: the shortest text that reads back as the same number
                        let _ = write!(out, "{v}");
                    }
                }
                out.push(',');
                out.push_str(&csv_cell(&d.facts.never_built.join(" ")));
            }
            Err(e) => {
                out.push_str(",failed,");
                out.push_str(&csv_cell(e));
                out.push_str(&",".repeat(sw.fact_index.len() + 1));
            }
        }
        out.push('\n');
    }
    out
}

fn set_object(sw: &Sweep, setting: &Setting) -> Value {
    let mut m = serde_json::Map::new();
    for (k, v) in sw.keys.iter().zip(&setting.values) {
        m.insert(k.clone(), parse_value(v));
    }
    Value::Object(m)
}

/// One object per run under `runs`, with what the sweep was above them.
pub fn facts_json(sw: &Sweep, command: &str, code: Option<&str>) -> Value {
    let runs: Vec<Value> = sw
        .runs
        .iter()
        .map(|run| {
            let setting = &sw.settings[run.setting];
            let mut o = json!({
                "run": run.id, "seed": run.seed, "setting": setting.label, "set": set_object(sw, setting),
                "ok": run.outcome.is_ok(), "seconds": run.seconds,
            });
            match &run.outcome {
                Ok(d) => {
                    let facts: serde_json::Map<String, Value> = d.facts.facts.iter().map(|f| (f.name.clone(), json!(f.value))).collect();
                    o["facts"] = Value::Object(facts);
                    o["never_built"] = json!(d.facts.never_built);
                }
                Err(e) => o["error"] = json!(e),
            }
            o
        })
        .collect();
    json!({
        "command": command,
        "rules": sw.rules.as_str(),
        "code": code,
        "seeds": sw.seeds,
        "keys": sw.keys,
        "settings": sw.settings.iter().map(|s| json!({"label": s.label, "set": set_object(sw, s)})).collect::<Vec<_>>(),
        "facts": sw.fact_index.iter().map(|(n, g, k)| json!({"name": n, "group": g, "kind": k.as_str()})).collect::<Vec<_>>(),
        "runs": runs,
    })
}

/// `git describe` in the directory the sweep was started from, if git is there.
fn git_describe() -> Option<String> {
    let out = std::process::Command::new("git").args(["describe", "--always", "--dirty"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !text.is_empty()).then_some(text)
}

fn readme(sw: &Sweep, o: &Options, code: Option<&str>) -> String {
    let failed = sw.runs.iter().filter(|r| r.outcome.is_err()).count();
    let seeds: Vec<String> = sw.seeds.iter().map(|s| s.to_string()).collect();
    let file = |p: &Option<PathBuf>, none: &str| p.as_ref().map_or(none.to_string(), |p| p.display().to_string());
    let mut s = String::new();
    let _ = writeln!(s, "A sweep of Isolated State (isostate-core sweep), {} (UTC).\n", crate::record::today());
    let _ = writeln!(s, "command:  {}", o.command);
    let _ = writeln!(s, "rules:    {} ({})", sw.rules.as_str(), match sw.rules {
        Rules::Port => "default_params and default_content: the port of the Python",
        Rules::Game => "game_params and game_content: what a live game is played under",
    });
    let _ = writeln!(s, "code:     {}", code.map_or("unknown (git describe gave nothing)".to_string(), |c| format!("{c} (git describe where it was run)")));
    let _ = writeln!(s, "seeds:    {}", seeds.join(" "));
    let _ = writeln!(s, "families: {}", o.families.map_or("as the parameters say".to_string(), |n| n.to_string()));
    let _ = writeln!(s, "end year: {}", o.end_year.map_or("as the parameters and content say".to_string(), |y| y.to_string()));
    let _ = writeln!(s, "params:   {}", file(&o.params_file, "the rules' own"));
    let _ = writeln!(s, "content:  {}", file(&o.content_file, "the rules' own"));
    let _ = writeln!(s, "settings: {}", sw.settings.len());
    for st in &sw.settings {
        let _ = writeln!(s, "          {}", st.label);
    }
    let _ = writeln!(s, "runs:     {} ({failed} failed), {} at a time, {:.0} s in all", sw.runs.len(), sw.jobs, sw.seconds);
    s.push_str(
        "\nFiles\n\
         annual.csv   one row per run and year: run, seed, the swept values, the year, then every observer\n\
         \x20            series averaged over the year's twelve months (empty where a share had no total)\n\
         facts.csv    one row per run: its stylised facts (empty where a fact has no value, as when a\n\
         \x20            railway was never built), and the methods never built\n\
         facts.json   the same, one object per run under \"runs\"; what --compare reads\n\
         summary.txt  the table the sweep printed: each fact's mean and [least .. greatest] across seeds\n\
         scorecard.txt  the aims the documents state, each against its fact's mean, with a tolerance from\n\
         \x20            the spread across seeds and a loss (isostate-core scorecard DIR prints it again)\n\
         \nSpeed: seconds is each run's time on the clock, with the others of the sweep running beside it and\n\
         whatever else the machine was doing, so it says as much about the machine that day as about the\n\
         code. cpu_seconds is the time the processor spent on the run's own thread, which other work\n\
         moves much less but still moves: the same sixteen islands read 32 s and 48 s of it a run with\n\
         the machine busy and busier (39 s and 172 s on the clock) (Windows only; empty elsewhere).\n\
         \nWhat each fact means is in docs/rules/README.md and at the top of crates/isostate-core/src/facts.rs.\n",
    );
    s
}

pub fn write_outputs(sw: &Sweep, o: &Options, summary: &str) -> Result<(), String> {
    let dir = &o.out;
    let put = |name: &str, text: String| std::fs::write(dir.join(name), text).map_err(|e| format!("{}: {e}", dir.join(name).display()));
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let code = git_describe();
    put("annual.csv", annual_csv(sw))?;
    put("facts.csv", facts_csv(sw))?;
    put("facts.json", serde_json::to_string_pretty(&facts_json(sw, &o.command, code.as_deref())).map_err(|e| e.to_string())? + "\n")?;
    put("summary.txt", summary.to_string())?;
    put("README.txt", readme(sw, o, code.as_deref()))?;
    // the scorecard (docs/rules/scorecard.md), read back from the facts.json just written;
    // written and not printed, so the table and the comparison stay the last thing said
    if sw.runs.iter().any(|r| r.outcome.is_ok()) {
        put("scorecard.txt", crate::scorecard::card(dir, None)?)?;
    }
    Ok(())
}

// ---------------------------------------------------------------- the table

/// A fact across the seeds of one setting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stat {
    pub n: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

/// One fact's values across the runs of a setting, each with its run's seed.
type Seeded = Vec<(u64, f64)>;

fn stat(values: &[(u64, f64)]) -> Option<Stat> {
    if values.is_empty() {
        return None;
    }
    Some(Stat {
        n: values.len(),
        mean: values.iter().map(|&(_, v)| v).sum::<f64>() / values.len() as f64,
        min: values.iter().map(|&(_, v)| v).fold(f64::INFINITY, f64::min),
        max: values.iter().map(|&(_, v)| v).fold(f64::NEG_INFINITY, f64::max),
    })
}

fn fmt_real(v: f64) -> String {
    let a = v.abs();
    if a == 0.0 {
        "0".to_string()
    } else if a >= 100.0 {
        fmt_thousands_f0(v)
    } else if a >= 10.0 {
        format!("{v:.1}")
    } else if a >= 1.0 {
        format!("{v:.2}")
    } else if a >= 0.01 {
        format!("{v:.3}")
    } else {
        format!("{v:.2e}")
    }
}

/// A fact's value as the table shows it.
pub fn fmt_fact(kind: Kind, v: f64) -> String {
    match kind {
        Kind::Year => format!("{v:.0}"),
        Kind::Count if v.abs() >= 1000.0 || v.fract() == 0.0 => fmt_thousands_f0(v),
        Kind::Count => format!("{v:.1}"),
        Kind::Share => format!("{v:.3}"),
        Kind::Ratio => format!("{v:.2}"),
        Kind::Real => fmt_real(v),
    }
}

/// The change from the mean then to the mean now, with its sign, to the
/// precision the two are shown in; counts and plain numbers also in per cent.
fn fmt_change(kind: Kind, now: f64, then: f64) -> String {
    let d = now - then;
    let scale = now.abs().max(then.abs());
    let body = match kind {
        Kind::Year => format!("{d:+.1}"),
        Kind::Share => format!("{d:+.3}"),
        Kind::Ratio => format!("{d:+.2}"),
        Kind::Count if scale >= 1000.0 => format!("{}{}", if d < 0.0 { "-" } else { "+" }, fmt_thousands_f0(d.abs())),
        Kind::Count => format!("{d:+.1}"),
        Kind::Real => {
            let places = if scale >= 100.0 { 0 } else if scale >= 10.0 { 1 } else if scale >= 1.0 { 2 } else { 3 };
            format!("{d:+.places$}")
        }
    };
    match kind {
        Kind::Count | Kind::Real if then != 0.0 => format!("{body} ({:+.1}%)", 100.0 * d / then.abs()),
        _ => body,
    }
}

/// Each fact's values over the finished runs of one setting, by name.
fn setting_values(sw: &Sweep, k: usize) -> HashMap<&str, Seeded> {
    let mut m: HashMap<&str, Seeded> = HashMap::new();
    for run in sw.runs.iter().filter(|r| r.setting == k) {
        let Ok(done) = &run.outcome else { continue };
        for f in &done.facts.facts {
            if let Some(v) = f.value {
                m.entry(f.name.as_str()).or_default().push((run.seed, v));
            }
        }
    }
    m
}

/// "1 run", "8 runs".
fn count_runs(n: usize) -> String {
    format!("{n} run{}", if n == 1 { "" } else { "s" })
}

fn ok_runs(sw: &Sweep, k: usize) -> usize {
    sw.runs.iter().filter(|r| r.setting == k && r.outcome.is_ok()).count()
}

/// Which side of its column a cell is set against.
#[derive(Clone, Copy, PartialEq)]
enum Side {
    Left,
    Right,
}

/// Rows under their headings, the columns padded to line up. The first column is
/// the fact's name, indented under its heading; `sides` says how each of the
/// others is set.
fn layout(head: &[String], sides: &[Side], rows: &[(Option<&'static str>, Vec<String>)]) -> String {
    let cols = head.len();
    let mut width: Vec<usize> = head.iter().map(|h| h.chars().count()).collect();
    for (_, cells) in rows.iter().filter(|(h, _)| h.is_none()) {
        for (c, cell) in cells.iter().enumerate() {
            width[c] = width[c].max(cell.chars().count() + if c == 0 { 2 } else { 0 });
        }
    }
    let line = |cells: &[String], indent: &str| -> String {
        let mut s = String::new();
        for c in 0..cols {
            let cell = cells.get(c).map_or("", |x| x.as_str());
            if c == 0 {
                let _ = write!(s, "{indent}{cell:<w$}", w = width[0] - indent.len());
            } else if sides[c] == Side::Left {
                let _ = write!(s, "  {cell:<w$}", w = width[c]);
            } else {
                let _ = write!(s, "  {cell:>w$}", w = width[c]);
            }
        }
        s.trim_end().to_string() + "\n"
    };
    let mut out = line(head, "");
    for (heading, cells) in rows {
        match heading {
            Some(h) => {
                let _ = writeln!(out, "{h}");
            }
            None => out.push_str(&line(cells, "  ")),
        }
    }
    out
}

/// The summary: for every fact its mean across seeds and, beside it, the least
/// and the greatest, one pair of columns per setting. The means stand in a
/// column of their own so the eye can run down them.
pub fn table(sw: &Sweep) -> String {
    let failed: Vec<&Run> = sw.runs.iter().filter(|r| r.outcome.is_err()).collect();
    let seeds: Vec<String> = sw.seeds.iter().map(|s| s.to_string()).collect();
    let mut out = format!(
        "{} under the {} rules: seeds {}, {} setting{}; {} failed; {:.0} s, {} at a time\n\
         each fact: its mean across seeds, then [least .. greatest] unless the two print the same;\n\
         (n/N) when only n of N runs have a value, - when none has\n\n",
        count_runs(sw.runs.len()),
        sw.rules.as_str(),
        seeds.join(" "),
        sw.settings.len(),
        if sw.settings.len() == 1 { "" } else { "s" },
        failed.len(),
        sw.seconds,
        sw.jobs
    );
    let values: Vec<HashMap<&str, Seeded>> = (0..sw.settings.len()).map(|k| setting_values(sw, k)).collect();
    // per setting: the mean under the setting's name, then the range and the (n/N) note
    let mut head = vec![String::new()];
    let mut sides = vec![Side::Left];
    for s in &sw.settings {
        head.extend([s.label.clone(), String::new()]);
        sides.extend([Side::Right, Side::Left]);
    }
    let mut rows: Vec<(Option<&'static str>, Vec<String>)> = Vec::new();
    let mut group = "";
    for (name, g, kind) in &sw.fact_index {
        if *g != group {
            group = *g;
            rows.push((Some(*g), Vec::new()));
        }
        let mut cells = vec![name.clone()];
        for (k, vals) in values.iter().enumerate() {
            let total = ok_runs(sw, k);
            match vals.get(name.as_str()).and_then(|v| stat(v)) {
                None => cells.extend(["-".to_string(), String::new()]),
                Some(s) => {
                    let (least, greatest) = (fmt_fact(*kind, s.min), fmt_fact(*kind, s.max));
                    let mut note = if least == greatest { String::new() } else { format!("[{least} .. {greatest}]") };
                    if s.n < total {
                        let _ = write!(note, "{}({}/{total})", if note.is_empty() { "" } else { " " }, s.n);
                    }
                    cells.extend([fmt_fact(*kind, s.mean), note]);
                }
            }
        }
        rows.push((None, cells));
    }
    out.push_str(&layout(&head, &sides, &rows));

    // methods never built, with the number of runs each was missing from
    for (k, setting) in sw.settings.iter().enumerate() {
        let mut missing: Vec<(String, usize)> = Vec::new();
        for done in sw.runs.iter().filter(|r| r.setting == k).filter_map(|r| r.outcome.as_ref().ok()) {
            for m in &done.facts.never_built {
                match missing.iter_mut().find(|(name, _)| name == m) {
                    Some((_, n)) => *n += 1,
                    None => missing.push((m.clone(), 1)),
                }
            }
        }
        let total = ok_runs(sw, k);
        let list: Vec<String> = missing.iter().map(|(m, n)| format!("{m} (in {n} of {})", count_runs(total))).collect();
        let _ = writeln!(out, "\nnever built, {}: {}", setting.label, if list.is_empty() { "nothing".to_string() } else { list.join(", ") });
    }
    for run in &sw.runs {
        if let Some(y) = run.outcome.as_ref().ok().and_then(|d| d.facts.get("works_table_full")) {
            let (label, places) = (&sw.settings[run.setting].label, sw.settings[run.setting].params.F_max);
            let _ = writeln!(out, "\nFULL run {} (seed {}, {label}): {}", run.id, run.seed, crate::firms::table_full_message(y as i64, places));
        }
    }
    for run in failed {
        let _ = writeln!(out, "\nFAILED run {} (seed {}, {}): {}", run.id, run.seed, sw.settings[run.setting].label, run.outcome.as_ref().err().unwrap());
    }
    out
}

// ---------------------------------------------------------------- against an earlier sweep

/// One setting of an earlier sweep, read back from its facts.json.
#[derive(Clone, Debug, Default)]
pub struct Then {
    pub label: String,
    /// The seeds of its finished runs.
    pub seeds: Vec<u64>,
    pub values: HashMap<String, Seeded>,
}

/// The settings of the sweep in `dir`, with each fact's values across its finished runs.
pub fn read_then(dir: &Path) -> Result<Vec<Then>, String> {
    let path = dir.join("facts.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("--compare: {}: {e}", path.display()))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("--compare: {}: {e}", path.display()))?;
    let not_a_sweep = |what: &str| format!("--compare: {} has {what}; is it a sweep's facts.json?", path.display());
    let runs = doc["runs"].as_array().ok_or_else(|| not_a_sweep("no runs"))?;
    let mut out: Vec<Then> = Vec::new();
    for run in runs.iter().filter(|r| r["ok"].as_bool() == Some(true)) {
        let label = run["setting"].as_str().unwrap_or("").to_string();
        let seed = run["seed"].as_u64().ok_or_else(|| not_a_sweep("a run without a seed"))?;
        let k = match out.iter().position(|t| t.label == label) {
            Some(k) => k,
            None => {
                out.push(Then { label, ..Then::default() });
                out.len() - 1
            }
        };
        out[k].seeds.push(seed);
        for (name, v) in run["facts"].as_object().into_iter().flatten() {
            if let Some(v) = v.as_f64() {
                out[k].values.entry(name.clone()).or_default().push((seed, v));
            }
        }
    }
    if out.is_empty() {
        return Err(format!("--compare: {} has no finished runs", path.display()));
    }
    Ok(out)
}

/// A star needs this many runs on each side. With fewer, two sets of seeds under
/// the same rules stand clear of each other too often for a star to mean much:
/// one time in three with two a side, one in ten with three, one in 35 with four.
pub const STAR_RUNS: usize = 4;

/// Each fact's mean now, its mean then and the change, for every setting of this
/// sweep against the earlier sweep's setting of the same name (or its first).
///
/// Like is compared with like: where the two have seeds in common, only those
/// seeds' runs are used on either side, so that eight seeds are never set
/// against four of them and the difference between islands taken for a
/// difference between rules. Seed by seed the table then counts where a fact
/// rose and where it fell. `=` is a fact that came out the same; a star marks
/// one whose ranges across seeds do not overlap.
pub fn compare(sw: &Sweep, then: &[Then], dir: &Path) -> String {
    let mut out = String::new();
    for (k, setting) in sw.settings.iter().enumerate() {
        let other = then.iter().find(|t| t.label == setting.label).unwrap_or(&then[0]);
        let now = setting_values(sw, k);
        let seeds: Vec<u64> = sw.runs.iter().filter(|r| r.setting == k && r.outcome.is_ok()).map(|r| r.seed).collect();
        let common: Vec<u64> = seeds.iter().copied().filter(|s| other.seeds.contains(s)).collect();
        let paired = !common.is_empty();
        let kept = |v: Option<&Seeded>| -> Seeded {
            v.into_iter().flatten().copied().filter(|(s, _)| !paired || common.contains(s)).collect()
        };
        let head: Vec<String> = ["", "now", "then", "change", if paired { "up/down" } else { "" }, ""].iter().map(|s| s.to_string()).collect();
        let sides = [Side::Left, Side::Right, Side::Right, Side::Right, Side::Right, Side::Left];
        let mut rows: Vec<(Option<&'static str>, Vec<String>)> = Vec::new();
        let mut group = "";
        // the facts both sweeps have, those of them that moved, and those that moved clear of the spread
        let (mut both, mut moved, mut clear) = (0, 0, 0);
        for (name, g, kind) in &sw.fact_index {
            if *g != group {
                group = *g;
                rows.push((Some(*g), Vec::new()));
            }
            let (va, vb) = (kept(now.get(name.as_str())), kept(other.values.get(name)));
            let (a, b) = (stat(&va), stat(&vb));
            // seed by seed: where the fact is greater now, where it is less
            let (mut up, mut down) = (0, 0);
            for &(seed, x) in va.iter().filter(|_| paired) {
                if let Some(&(_, y)) = vb.iter().find(|(s, _)| *s == seed) {
                    up += (x > y) as usize;
                    down += (x < y) as usize;
                }
            }
            let same = if paired {
                // the same seeds have a value, and in each it is the value it was
                va.len() == vb.len() && va.iter().all(|(s, _)| vb.iter().any(|(t, _)| t == s)) && up + down == 0
            } else {
                a == b
            };
            let show = |s: Option<Stat>| s.map_or("-".to_string(), |s| fmt_fact(*kind, s.mean));
            let (change, tally, mark) = match (a, b) {
                (Some(_), Some(_)) if same => ("=".to_string(), String::new(), ""),
                (Some(a), Some(b)) => {
                    let apart = a.n >= STAR_RUNS && b.n >= STAR_RUNS && (a.min > b.max || a.max < b.min);
                    let tally = if paired { format!("{up}/{down}") } else { String::new() };
                    (fmt_change(*kind, a.mean, b.mean), tally, if apart { "*" } else { "" })
                }
                _ => (String::new(), String::new(), ""),
            };
            // how long the runs took is no fact about the rules: it is shown, not counted
            if a.is_some() && b.is_some() && *g != SPEED {
                both += 1;
                moved += (change != "=") as usize;
                clear += (mark == "*") as usize;
            }
            rows.push((None, vec![name.clone(), show(a), show(b), change, tally, mark.to_string()]));
        }
        let list = |v: &[u64]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ");
        let on = if !paired {
            "no seed in common: every run of each is used, and the islands may differ by more than the rules do".to_string()
        } else if common.len() == seeds.len() && common.len() == other.seeds.len() {
            format!("seed for seed, on seeds {}", list(&common))
        } else {
            format!("seed for seed, on the seeds both have: {}", list(&common))
        };
        let _ = writeln!(
            out,
            "\nnow: {} ({}); then: {} ({}) in {}\n\
             {on}\n\
             of the {both} facts both have, the seconds aside, {moved} differ and {clear} of those are starred\n\
             = {}\n\
             {}\
             * the ranges across seeds do not overlap: every seed now is above, or below, every seed then\n  \
             (marked only with {STAR_RUNS} runs or more a side)\n",
            setting.label,
            count_runs(seeds.len()),
            other.label,
            count_runs(other.seeds.len()),
            dir.display(),
            if paired { "every seed gives the number it gave" } else { "the same number of runs with the same mean, least and greatest" },
            if paired { "up/down: the seeds in which the fact is greater now, and those in which it is less\n" } else { "" },
        );
        out.push_str(&layout(&head, &sides, &rows));
    }
    out
}

/// The `sweep` subcommand: run, write the files, print the table.
pub fn main_with_args(args: &[String]) -> Result<(), String> {
    command(args, true)
}

/// The subcommand with or without the table on stdout (it is in summary.txt
/// either way; the tests leave it off).
pub fn command(args: &[String], print: bool) -> Result<(), String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let o = parse_args(args)?;
    // whatever is wrong is found before the first run, so that it costs nothing: the earlier
    // sweep's path, a setting, the content, a directory that cannot be made. The earlier sweep
    // is read here, so --compare may name --out itself: what was there is compared, then replaced.
    let then = match &o.compare {
        Some(dir) => Some(read_then(dir)?),
        None => None,
    };
    let plan = plan(&o)?;
    std::fs::create_dir_all(&o.out).map_err(|e| format!("--out {}: {e}", o.out.display()))?;
    let sw = run(&o, plan);
    let mut text = table(&sw);
    if let (Some(then), Some(dir)) = (&then, &o.compare) {
        text.push_str(&compare(&sw, then, dir));
    }
    write_outputs(&sw, &o, &text)?;
    if print {
        // not print!, which panics when the reader has gone (sweep ... | head)
        let _ = std::io::Write::write_all(&mut std::io::stdout(), text.as_bytes());
    }
    let failed = sw.runs.iter().filter(|r| r.outcome.is_err()).count();
    if failed > 0 {
        return Err(format!("{failed} of {} runs failed; the rest are in {}", sw.runs.len(), o.out.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn seeds_and_values_parse() {
        assert_eq!(parse_seeds("1-4").unwrap(), vec![1, 2, 3, 4]);
        assert_eq!(parse_seeds("1,2,5").unwrap(), vec![1, 2, 5]);
        assert_eq!(parse_seeds("1-3,7,2").unwrap(), vec![1, 2, 3, 7]);
        assert!(parse_seeds("3-1").is_err() && parse_seeds("a").is_err() && parse_seeds("-1").is_err() && parse_seeds("1-").is_err());
        // a slip of the hand is refused before the list is laid out in memory
        assert_eq!(parse_seeds("1-10000").unwrap().len(), 10_000);
        assert!(parse_seeds("0-99999999999").unwrap_err().contains("more than 10000 seeds"));
        assert!(parse_seeds("1-6000,7001-12000").unwrap_err().contains("more than 10000 seeds"));
        assert_eq!(split_values("2,3,4"), vec!["2", "3", "4"]);
        assert_eq!(split_values("[1500,9000,40000],[1,2,3]"), vec!["[1500,9000,40000]", "[1,2,3]"]);
        assert_eq!(parse_value("2.5"), json!(2.5));
        assert_eq!(parse_value("true"), json!(true));
        assert_eq!(parse_value("commons"), json!("commons"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn any_params_field_can_be_set_by_name() {
        let P = crate::params::default_params();
        assert_eq!(set_field(&P, "land_theta", &json!(2.5)).unwrap().land_theta, 2.5);
        assert_eq!(set_field(&P, "rent_regime", &json!("commons")).unwrap().rent_regime, crate::params::RentRegime::Commons);
        assert_eq!(set_field(&P, "road_thresholds", &json!([1.0, 2.0, 3.0])).unwrap().road_thresholds, [1.0, 2.0, 3.0]);
        // fields left out of params.json while at their defaults can still be set
        assert_eq!(set_field(&P, "n_families", &json!(6)).unwrap().n_families, 6);
        assert_eq!(set_field(&P, "end_year", &json!(1800)).unwrap().end_year, Some(1800));
        let e = set_field(&P, "land_thetaa", &json!(2.5)).unwrap_err();
        assert!(e.contains("land_thetaa") && e.contains("Did you mean land_theta?"), "{e}");
        let e = set_field(&P, "zzzzzz", &json!(1)).unwrap_err();
        assert!(e.contains("The parameters are: seed, W, H") && e.contains("university_gain"), "{e}");
        let e = set_field(&P, "land_theta", &json!("wide")).unwrap_err();
        assert!(e.contains("--set land_theta=\"wide\"") && e.contains("invalid type"), "{e}");
        let e = set_field(&P, "hurdle.rate", &json!(1)).unwrap_err();
        assert!(e.contains("hurdle is a single value"), "{e}");
    }

    /// A stand-in for `Params` with a mechanism's struct in it, as the convention in params.rs lays out.
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(default, deny_unknown_fields)]
    struct Canals {
        enabled: bool,
        cost: f64,
    }

    impl Default for Canals {
        fn default() -> Canals {
            Canals { enabled: false, cost: 0.3 }
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(default, deny_unknown_fields)]
    struct Stand {
        hurdle: f64,
        #[serde(default, skip_serializing_if = "crate::params::is_default")]
        canals: Canals,
    }

    #[test]
    fn a_dotted_path_reaches_into_a_mechanism() {
        let base = Stand::default();
        assert_eq!(serde_json::to_string(&base).unwrap(), r#"{"hurdle":0.0}"#, "a mechanism left off is not written out");
        // switched off and so absent from the JSON form: set from its defaults
        let on = set_field(&base, "canals.enabled", &json!(true)).unwrap();
        assert_eq!(on.canals, Canals { enabled: true, cost: 0.3 });
        // switched on and so written out: the other fields are kept
        let cheap = set_field(&on, "canals.cost", &json!(0.2)).unwrap();
        assert_eq!(cheap.canals, Canals { enabled: true, cost: 0.2 });
        let e = set_field(&base, "canals.cots", &json!(0.2)).unwrap_err();
        assert!(e.contains("no parameter \"cots\" in canals") && e.contains("Did you mean cost?"), "{e}");
        let e = set_field(&base, "canal.cost", &json!(0.2)).unwrap_err();
        assert!(e.contains("no parameter \"canal\"") && e.contains("Did you mean canals?"), "{e}");
    }

    #[test]
    fn settings_are_every_combination() {
        let args: Vec<String> =
            ["--out", "x", "--seeds", "1,2", "--set", "land_theta=2,3", "--set", "hurdle=0.1,0.12,0.14"].iter().map(|s| s.to_string()).collect();
        let o = parse_args(&args).unwrap();
        let s = settings(&o).unwrap();
        assert_eq!(s.len(), 6);
        assert_eq!(s[0].label, "land_theta=2 hurdle=0.1");
        assert_eq!(s[5].label, "land_theta=3 hurdle=0.14");
        assert_eq!((s[4].params.land_theta, s[4].params.hurdle), (3.0, 0.12));
        let none = parse_args(&["--out".to_string(), "x".to_string()]).unwrap();
        assert_eq!(settings(&none).unwrap()[0].label, "port rules");
        assert_eq!(none.seeds, (1..=8).collect::<Vec<u64>>());
        assert!(parse_args(&["--out".to_string(), "x".to_string(), "--set".to_string(), "seed=3".to_string()]).is_err());
        let twice = parse_args(&["--out".to_string(), "x".to_string(), "--set".to_string(), "land_theta=2,3,2".to_string()]);
        assert!(twice.unwrap_err().contains("2 is listed twice"));
        // the column is named after the families that run, wherever they were asked for
        let mut six = none.clone();
        six.families = Some(6);
        assert_eq!(settings(&six).unwrap()[0].label, "port rules, 6 families");
        six.rules = Rules::Game;
        six.families = Some(1);
        assert_eq!(settings(&six).unwrap()[0].label, "game rules, 1 family");
    }

    #[test]
    fn a_swept_setting_is_checked_as_a_file_would_be() {
        // a stock target of 100 months leaves the game's brake on a glut (8 months)
        // easing off below it: refused from a --params file, and so from --set
        let args: Vec<String> =
            ["--out", "x", "--rules", "game", "--set", "price_target_months=3,100"].iter().map(|s| s.to_string()).collect();
        let e = settings(&parse_args(&args).unwrap()).unwrap_err();
        assert!(e.contains("prices.glut_stop_months") && e.contains("price_target_months (100)"), "{e}");
        assert!(crate::game_params().with_json_overrides(r#"{"price_target_months": 100}"#).is_err());
    }

    /// A sweep written out by hand: two settings, four seeds, a few facts of each kind.
    /// The second setting is ten of every unit above the first; seed 2 never builds a railway.
    fn by_hand() -> Sweep {
        let fact = |name: &str, group: &'static str, kind: Kind, value: Option<f64>| Fact { name: name.to_string(), group, kind, value };
        let mut runs = Vec::new();
        for (k, seed) in (0..2).flat_map(|k| (1..=4u64).map(move |seed| (k, seed))) {
            let x = (k * 10) as f64 + seed as f64;
            let facts = vec![
                fact("pop_1750", GROUPS[0], Kind::Count, Some(135_000.0)),
                fact("pop_1800", GROUPS[0], Kind::Count, Some(229_000.0 + 1000.0 * x)),
                fact("urban_share_1800", GROUPS[0], Kind::Share, Some(0.2 + 0.01 * x)),
                fact("rail_first_year", GROUPS[2], Kind::Year, (seed != 2).then_some(1820.0 + x)),
                fact("half_of_output_bessemer", GROUPS[5], Kind::Year, None),
                fact("seconds", SPEED, Kind::Real, Some(17.0 + x)),
            ];
            let done = Done { facts: Facts { facts, ..Facts::default() }, annual: Vec::new(), names: Vec::new(), start_year: 1750 };
            runs.push(Run { id: runs.len(), seed, setting: k, outcome: Ok(done), seconds: 17.0 });
        }
        let setting = |v: &str| Setting { label: format!("hurdle={v}"), values: vec![v.to_string()], params: crate::params::default_params() };
        let fact_index = match &runs[0].outcome {
            Ok(done) => done.facts.facts.iter().map(|f| (f.name.clone(), f.group, f.kind)).collect(),
            Err(_) => unreachable!(),
        };
        Sweep {
            rules: Rules::Port,
            seeds: vec![1, 2, 3, 4],
            keys: vec!["hurdle".to_string()],
            settings: vec![setting("0.1"), setting("0.12")],
            runs,
            fact_index,
            jobs: 2,
            seconds: 51.0,
        }
    }

    #[test]
    fn the_table_lines_the_means_up() {
        let text = table(&by_hand());
        println!("{text}");
        let want = "\
8 runs under the port rules: seeds 1 2 3 4, 2 settings; 0 failed; 51 s, 2 at a time
each fact: its mean across seeds, then [least .. greatest] unless the two print the same;
(n/N) when only n of N runs have a value, - when none has

                           hurdle=0.1                        hurdle=0.12
People and towns
  pop_1750                    135,000                            135,000
  pop_1800                    231,500  [230,000 .. 233,000]      241,500  [240,000 .. 243,000]
  urban_share_1800              0.225  [0.210 .. 0.240]            0.325  [0.310 .. 0.340]
Railways
  rail_first_year                1823  [1821 .. 1824] (3/4)         1833  [1831 .. 1834] (3/4)
Methods and inventions
  half_of_output_bessemer           -                                  -
Speed
  seconds                        19.5  [18.0 .. 21.0]               29.5  [28.0 .. 31.0]

never built, hurdle=0.1: nothing

never built, hurdle=0.12: nothing
";
        assert_eq!(text, want);
    }

    #[test]
    fn a_sweep_set_beside_itself_shows_no_change() {
        let sw = by_hand();
        // the earlier sweep as --compare reads it back: through facts.json's text
        let dir = std::env::temp_dir().join(format!("isostate-sweep-unit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("facts.json"), serde_json::to_string_pretty(&facts_json(&sw, "by hand", None)).unwrap()).unwrap();
        let then = read_then(&dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(then.len(), 2);
        let text = compare(&sw, &then, Path::new("then"));
        println!("{text}");
        assert!(text.contains("now: hurdle=0.12 (4 runs); then: hurdle=0.12 (4 runs) in then"), "{text}");
        assert_eq!(text.matches("of the 4 facts both have, the seconds aside, 0 differ and 0 of those are starred").count(), 2, "{text}");
        // not a change and not a star anywhere, the seconds included
        assert!(!text.contains('+') && !text.contains("  *"), "{text}");

        // with only the first setting to compare with, the second is set against it: everything
        // but the 1750 population moves clear of the seeds' spread. The railway's year has three
        // runs a side, too few for a star.
        let text = compare(&sw, &then[..1], Path::new("then"));
        println!("{text}");
        assert!(text.contains("now: hurdle=0.12 (4 runs); then: hurdle=0.1 (4 runs) in then"), "{text}");
        assert!(text.contains("seed for seed, on seeds 1 2 3 4\n"), "{text}");
        assert!(text.contains("of the 4 facts both have, the seconds aside, 3 differ and 2 of those are starred"), "{text}");
        assert!(text.contains("  pop_1750                 135,000  135,000                =\n"), "{text}");
        assert!(text.contains("  pop_1800                 241,500  231,500  +10,000 (+4.3%)      4/0  *\n"), "{text}");
        assert!(text.contains("  urban_share_1800           0.325    0.225           +0.100      4/0  *\n"), "{text}");
        assert!(text.contains("  rail_first_year             1833     1823            +10.0      3/0\n"), "{text}");

        // an earlier sweep with more seeds than this one: only the seeds both have are used on
        // either side, so the same rules show no change though the earlier mean was over more islands
        let mut wider = then[0].clone();
        wider.seeds.push(9);
        for values in wider.values.values_mut() {
            values.push((9, 1e6));
        }
        let text = compare(&sw, &[wider], Path::new("then"));
        println!("{text}");
        assert!(text.contains("now: hurdle=0.1 (4 runs); then: hurdle=0.1 (5 runs) in then\n"), "{text}");
        assert!(text.contains("seed for seed, on the seeds both have: 1 2 3 4\n"), "{text}");
        assert!(text.contains("of the 4 facts both have, the seconds aside, 0 differ and 0 of those are starred"), "{text}");

        // no seed in common: all the runs of each, no tally, and the reader is told
        let mut elsewhere = then[0].clone();
        elsewhere.seeds = vec![11, 12, 13, 14];
        for values in elsewhere.values.values_mut() {
            values.iter_mut().for_each(|(seed, _)| *seed += 10);
        }
        let text = compare(&sw, &[elsewhere], Path::new("then"));
        println!("{text}");
        assert!(text.contains("no seed in common"), "{text}");
        assert!(!text.contains("up/down") && !text.contains("4/0"), "{text}");
        // the same numbers on other seeds: equal as far as a mean and a range can tell
        assert!(text.contains("  pop_1800                 231,500  231,500       =\n"), "{text}");
    }

    #[test]
    fn a_number_read_back_from_json_is_the_number_written() {
        // what `=` in a comparison rests on, and what serde_json's float_roundtrip feature
        // (Cargo.toml) is switched on for: without it some numbers come back one ulp off
        // (3.5968292645345246, among these)
        let mut x = 0.123_456_789_f64;
        for _ in 0..20_000 {
            x = (x * 1.618_033_988_75 + 0.000_271_828) % 1000.0;
            let text = serde_json::to_string(&x).unwrap();
            assert_eq!(serde_json::from_str::<f64>(&text).unwrap().to_bits(), x.to_bits(), "{text}");
        }
    }

    #[test]
    fn numbers_print_by_kind() {
        assert_eq!(fmt_fact(Kind::Year, 1823.4), "1823");
        assert_eq!(fmt_fact(Kind::Count, 412_345.6), "412,346");
        assert_eq!(fmt_fact(Kind::Count, 7.5), "7.5");
        assert_eq!(fmt_fact(Kind::Share, 0.31234), "0.312");
        assert_eq!(fmt_fact(Kind::Ratio, 1.2345), "1.23");
        assert_eq!(fmt_fact(Kind::Real, 0.0456), "0.046");
        assert_eq!(fmt_change(Kind::Count, 1050.0, 1000.0), "+50 (+5.0%)");
        assert_eq!(fmt_change(Kind::Count, 19.5, 46.25), "-26.8 (-57.8%)");
        assert_eq!(fmt_change(Kind::Year, 1797.0, 1800.0), "-3.0");
        assert_eq!(fmt_change(Kind::Real, 0.086, 0.083), "+0.003 (+3.6%)");
    }
}
