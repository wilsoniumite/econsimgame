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
//! - `README.txt`: the command line and what was run.
//!
//! `--set` takes any field of `Params` by name, and a dotted path into a
//! mechanism's own struct (`canals.cost=0.2,0.3`). The value goes in through the
//! parameters' JSON form, so a field added tomorrow can be swept with no table of
//! names to keep up to date here. `--compare OTHER_DIR` sets the new means beside
//! those of an earlier sweep and marks the facts whose seed ranges do not overlap.
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

use crate::cli::parse_rules;
use crate::content::{Content, load_content};
use crate::facts::{self, Fact, Facts, GROUPS, Kind};
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

/// `1-8`, `1,2,5` or a mixture (`1-3,7`).
pub fn parse_seeds(s: &str) -> Result<Vec<u64>, String> {
    let bad = |e: std::num::ParseIntError| format!("--seeds {s:?}: {e}");
    let mut out = Vec::new();
    for part in s.split(',') {
        match part.split_once('-') {
            Some((a, b)) => {
                let (a, b): (u64, u64) = (a.trim().parse().map_err(bad)?, b.trim().parse().map_err(bad)?);
                if a > b {
                    return Err(format!("--seeds {s:?}: {a}-{b} runs backwards"));
                }
                out.extend(a..=b);
            }
            None => out.push(part.trim().parse().map_err(bad)?),
        }
    }
    let mut seen = HashSet::new();
    out.retain(|x| seen.insert(*x));
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
            let inside = bad.len() >= 3 && (k.contains(bad) || bad.contains(k));
            (d <= limit || inside).then_some((d, k))
        })
        .collect();
    scored.sort();
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
        let label = if values.is_empty() {
            // nothing swept: name the column after what was run
            let families = o.families.filter(|&n| n > 0).map_or(String::new(), |n| format!(", {n} families"));
            format!("{} rules{families}", o.rules.as_str())
        } else {
            o.sets.iter().zip(&values).map(|((k, _), v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")
        };
        out.push(Setting { label, values, params: P });
    }
    Ok(out)
}

// ---------------------------------------------------------------- running

/// One run, stepped year by year so the facts' gathering can look in between.
fn one_run(P: &Params, content: Content) -> Result<Done, String> {
    let t0 = Instant::now();
    let mut s = sim::start(P, content);
    let mut gathered = facts::Gathered::default();
    while s.year <= s.end_year {
        sim::run_year(&mut s)?;
        facts::gather_year(&mut gathered, &s);
    }
    let r = sim::finish(s);
    let seconds = t0.elapsed().as_secs_f64();
    Ok(Done {
        facts: facts::stylised_facts(&r, &gathered, seconds),
        annual: facts::annual_means(&r),
        names: r.state.series.names.clone(),
        start_year: r.content.start_year,
    })
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()));
    format!("panicked: {}", text.unwrap_or_else(|| "no message".to_string()))
}

/// Run the whole sweep. Nothing is written or printed here but the progress lines.
pub fn run(o: &Options) -> Result<Sweep, String> {
    let settings = settings(o)?;
    let content = match &o.content_file {
        Some(p) => load_content(p)?,
        None => o.rules.content(),
    };
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
                        let how = match &outcome {
                            Ok(_) => format!("{seconds:.1} s"),
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
            kinds.iter().map(|k| Fact { name: format!("news_{k}"), group: GROUPS[7], kind: Kind::Count, value: Some(count(k)) }).collect();
        let total: usize = done.facts.news.values().sum();
        news.push(Fact { name: "news_total".to_string(), group: GROUPS[7], kind: Kind::Count, value: Some(total as f64) });
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
    Ok(Sweep {
        rules: o.rules,
        seeds: o.seeds.clone(),
        keys: o.sets.iter().map(|(k, _)| k.clone()).collect(),
        settings,
        runs,
        fact_index,
        jobs,
        seconds: t0.elapsed().as_secs_f64(),
    })
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
                        // whole numbers in full, the rest to 9 digits
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
    put("README.txt", readme(sw, o, code.as_deref()))
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

fn stat(values: &[f64]) -> Option<Stat> {
    if values.is_empty() {
        return None;
    }
    Some(Stat {
        n: values.len(),
        mean: values.iter().sum::<f64>() / values.len() as f64,
        min: values.iter().copied().fold(f64::INFINITY, f64::min),
        max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
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
fn setting_values(sw: &Sweep, k: usize) -> HashMap<&str, Vec<f64>> {
    let mut m: HashMap<&str, Vec<f64>> = HashMap::new();
    for done in sw.runs.iter().filter(|r| r.setting == k).filter_map(|r| r.outcome.as_ref().ok()) {
        for f in &done.facts.facts {
            if let Some(v) = f.value {
                m.entry(f.name.as_str()).or_default().push(v);
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

/// Rows under their headings, the columns padded to line up: the first to the left, the rest to the right.
fn layout(head: &[String], rows: &[(Option<&'static str>, Vec<String>)]) -> String {
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

/// The summary: for every fact its mean and `[least .. greatest]` across seeds, one column per setting.
pub fn table(sw: &Sweep) -> String {
    let failed: Vec<&Run> = sw.runs.iter().filter(|r| r.outcome.is_err()).collect();
    let seeds: Vec<String> = sw.seeds.iter().map(|s| s.to_string()).collect();
    let mut out = format!(
        "{} under the {} rules: seeds {}, {} setting{}; {} failed; {:.0} s, {} at a time\n\
         each fact: mean [least .. greatest] across seeds; (n/N) when only n of N runs have a value\n\n",
        count_runs(sw.runs.len()),
        sw.rules.as_str(),
        seeds.join(" "),
        sw.settings.len(),
        if sw.settings.len() == 1 { "" } else { "s" },
        failed.len(),
        sw.seconds,
        sw.jobs
    );
    let values: Vec<HashMap<&str, Vec<f64>>> = (0..sw.settings.len()).map(|k| setting_values(sw, k)).collect();
    let mut head = vec![String::new()];
    head.extend(sw.settings.iter().map(|s| s.label.clone()));
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
            cells.push(match vals.get(name.as_str()).and_then(|v| stat(v)) {
                None => "-".to_string(),
                Some(s) => {
                    let mut cell = fmt_fact(*kind, s.mean);
                    if total > 1 {
                        let _ = write!(cell, " [{} .. {}]", fmt_fact(*kind, s.min), fmt_fact(*kind, s.max));
                    }
                    if s.n < total {
                        let _ = write!(cell, " ({}/{total})", s.n);
                    }
                    cell
                }
            });
        }
        rows.push((None, cells));
    }
    out.push_str(&layout(&head, &rows));

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
    pub runs: usize,
    pub values: HashMap<String, Vec<f64>>,
}

/// The settings of the sweep in `dir`, with each fact's values across its finished runs.
pub fn read_then(dir: &Path) -> Result<Vec<Then>, String> {
    let path = dir.join("facts.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("--compare: {}: {e}", path.display()))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("--compare: {}: {e}", path.display()))?;
    let runs = doc["runs"].as_array().ok_or_else(|| format!("--compare: {} has no runs; is it a sweep's facts.json?", path.display()))?;
    let mut out: Vec<Then> = Vec::new();
    for run in runs.iter().filter(|r| r["ok"].as_bool() == Some(true)) {
        let label = run["setting"].as_str().unwrap_or("").to_string();
        let k = match out.iter().position(|t| t.label == label) {
            Some(k) => k,
            None => {
                out.push(Then { label, ..Then::default() });
                out.len() - 1
            }
        };
        out[k].runs += 1;
        for (name, v) in run["facts"].as_object().into_iter().flatten() {
            if let Some(v) = v.as_f64() {
                out[k].values.entry(name.clone()).or_default().push(v);
            }
        }
    }
    if out.is_empty() {
        return Err(format!("--compare: {} has no finished runs", path.display()));
    }
    Ok(out)
}

/// Each fact's mean now, its mean then and the change, for every setting of this
/// sweep against the earlier sweep's setting of the same name (or its first).
/// A star marks a fact whose ranges across seeds do not overlap.
pub fn compare(sw: &Sweep, then: &[Then], dir: &Path) -> String {
    let mut out = String::new();
    for (k, setting) in sw.settings.iter().enumerate() {
        let other = then.iter().find(|t| t.label == setting.label).unwrap_or(&then[0]);
        let now = setting_values(sw, k);
        let _ = writeln!(
            out,
            "\nnow: {} ({}); then: {} ({}) in {}\n\
             * the ranges across seeds do not overlap\n",
            setting.label,
            count_runs(ok_runs(sw, k)),
            other.label,
            count_runs(other.runs),
            dir.display()
        );
        let head: Vec<String> = ["", "now", "then", "change", ""].iter().map(|s| s.to_string()).collect();
        let mut rows: Vec<(Option<&'static str>, Vec<String>)> = Vec::new();
        let mut group = "";
        for (name, g, kind) in &sw.fact_index {
            if *g != group {
                group = *g;
                rows.push((Some(*g), Vec::new()));
            }
            let a = now.get(name.as_str()).and_then(|v| stat(v));
            let b = other.values.get(name).and_then(|v| stat(v));
            let show = |s: Option<Stat>| s.map_or("-".to_string(), |s| fmt_fact(*kind, s.mean));
            let (change, mark) = match (a, b) {
                (Some(a), Some(b)) => {
                    (fmt_change(*kind, a.mean, b.mean), if a.min > b.max || a.max < b.min { "*" } else { "" })
                }
                _ => (String::new(), ""),
            };
            rows.push((None, vec![name.clone(), show(a), show(b), change, mark.to_string()]));
        }
        out.push_str(&layout(&head, &rows));
    }
    out
}

/// The `sweep` subcommand: run, write the files, print the table.
pub fn main_with_args(args: &[String]) -> Result<(), String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let o = parse_args(args)?;
    // read the earlier sweep first, so a wrong path costs nothing
    let then = match &o.compare {
        Some(dir) => Some(read_then(dir)?),
        None => None,
    };
    let sw = run(&o)?;
    let mut text = table(&sw);
    if let (Some(then), Some(dir)) = (&then, &o.compare) {
        text.push_str(&compare(&sw, then, dir));
    }
    write_outputs(&sw, &o, &text)?;
    print!("{text}");
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
        assert!(parse_seeds("3-1").is_err() && parse_seeds("a").is_err());
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
