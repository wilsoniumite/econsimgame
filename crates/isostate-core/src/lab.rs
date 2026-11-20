//! The island as a laboratory: the same islands played with and without a
//! list of edits, side by side. Not in the Python; a tool, not a rule, so it
//! has no switch and changes nothing a run does. `docs/lab.md` is its page.
//!
//! ```text
//! isostate-core lab --out DIR [--rule NAME=V1[,V2...]@YEAR]... [--edits FILE.json] [--pin NAME@YEAR]... [--hold NAME@YEAR]...
//!                   [--seeds 1-8] [--rules port|game] [--set NAME=VALUE]... [--families N] [--end-year Y]
//!                   [--years 1850,1900,1950,2000,2050] [--facts NAME,...] [--jobs N] [--quiet]
//! ```
//!
//! A question put to the island ("what does turning the machines' dial tenfold
//! in 1900 do to wages by 1950?") is answered by playing each island twice from
//! 1750: once as it is (the control), and once with the edits (the treated
//! arm), and setting the two against each other island by island. Each arm is
//! a list of dated edits by hand (`edits.rs`), played by `edits::play_noting`
//! with the sweep's notes taken between the months, so that the control is the
//! sweep's own plain run to the last bit (`tests/lab.rs`).
//!
//! - `--rule NAME=V@YEAR` sets one of the rules' numbers or switches before
//!   January of YEAR, as the Rules tab does in a running island. It may be
//!   given more than once. One of them may carry several values
//!   (`tasks.dial=3,10,100@1950`), and then there is a treated arm for each.
//!   A rule set from the very start is the sweep's (`sweep --set`), not the lab's.
//! - `--set NAME=VALUE` sets one of the rules' numbers from the start, in
//!   every arm, the control's too, as a sweep's `--set` with one value does:
//!   `--set land.enabled=false` plays the islands on the port's island, which
//!   the game's rules no longer do (docs/rules/land.md, "The switch").
//! - `--edits FILE` reads a list of dated edits in the form `run --edits`
//!   reads, and puts it in every treated arm, before the `--rule` edits.
//! - `--pin NAME@YEAR` pins an idea (`edits::Edit::Pin`) to be found in YEAR
//!   on every island, and `--hold NAME@YEAR` holds it back until YEAR, both
//!   given before January of the run's first year, in every treated arm after
//!   the file's edits. The name is checked against the content before any run.
//! - The defaults are chosen for questions about work and machines: the game's
//!   rules, six families, islands 1 to 8, played to 2050, and the facts read in
//!   1850, 1900, 1950, 2000 and 2050 (those of them inside the run).
//!
//! Everything that can be refused before a run is refused before the first
//! run and before anything is written: a date outside the run or a month that
//! is not 1 to 12, and every rule edit, checked in the order of its date
//! against the rules as the earlier rule edits leave them
//! (`edits::rule_refused`, the same sentence the running island would give).
//! An idea or a town by hand can only be refused in the running island: such a
//! run is reported as failed, with the edit's sentence, and the rest carry on.
//!
//! What it prints, and writes to `DIR/lab.txt`: one line per treated arm
//! saying where the arms part (on each island, the first year whose yearly
//! means differ in any observer series, bit for bit), then a table with one
//! row for each fact and year: the control's mean across the islands, and for
//! each treated arm its mean, the change of the means, the least and the
//! greatest of the change island by island, and on how many islands the arm
//! is greater and less. Below it, the failed runs and the time.
//!
//! What it writes besides: `lab.csv` (the table's rows in long form, one line
//! for each arm, island, row and year), `edits.json` (each arm's list),
//! `README.txt`, and for each arm a directory (`control/`, `arm-1/` ...) with
//! `facts.json`, `facts.csv` and `annual.csv` in the sweep's own form, which
//! `sweep --compare` and the scorecard read.
//!
//! Every run begins in 1750: an experiment of eight islands to 2050 costs
//! about what two sweeps do. Grids of many settings will want the island
//! saved part way and played on from there, which does not exist yet.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde_json::{Value, json};

use crate::cli::{check_end_year, parse_rules, read_edits};
use crate::content::Content;
use crate::edits::{self, Dated, Edit, PinWhen};
use crate::facts::{self, Fact, Kind, SPEED};
use crate::params::{Params, Rules};
use crate::sim;
use crate::sweep::{self, Done, Run, Setting, Side, Sweep};

pub const USAGE: &str = "usage: isostate-core lab --out DIR [--rule NAME=V1[,V2...]@YEAR]... [--edits FILE.json] [--pin NAME@YEAR]... [--hold NAME@YEAR]...
                         [--seeds 1-8] [--rules port|game] [--set NAME=VALUE]... [--families N] [--end-year Y]
                         [--years 1850,1900,1950,2000,2050] [--facts NAME,...] [--jobs N] [--quiet]";

/// The years the facts are read in, unless `--years` says otherwise (those of
/// them outside the run are left out).
pub const YEARS: [i64; 5] = [1850, 1900, 1950, 2000, 2050];

/// The rows printed unless `--facts` says otherwise, under their headings.
/// `{Y}` stands for each of the years; `series:NAME` is the year's mean of an
/// observer series, which has a value in every year.
pub const ROWS: &[(&str, &[&str])] = &[
    ("Wages", &["real_wage_linked_{Y}_vs_1750", "real_wage_chained_{Y}_vs_1750", "made_linked_pc_{Y}_vs_1750", "real_income_{Y}"]),
    ("Work's share and the rest of income", &["income_share_work_{Y}", "gauge_work_net_{Y}", "income_share_works_{Y}", "income_share_land_{Y}"]),
    ("Who owns the works", &["owners_works_families_{Y}", "owners_works_lords_{Y}", "owners_works_hh_{Y}"]),
    ("Which tasks machines take", &["auto_haul_works_{Y}", "auto_haul_pits_{Y}", "machines_years_{Y}", "gauge_plant_years_{Y}"]),
    ("People and places", &["pop_{Y}", "farm_share_{Y}", "urban_share_{Y}"]),
    ("Money", &["series:money"]),
];

/// The heading of the rows `--facts` asks for.
const ASKED: &str = "The facts asked for";

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// One `--rule`: the rule's dotted name, its values as typed, and the year
/// it is set from (before January).
#[derive(Clone, Debug, PartialEq)]
pub struct RuleEdit {
    pub name: String,
    pub values: Vec<String>,
    pub year: i64,
}

/// What the command line asked for.
#[derive(Clone, Debug)]
pub struct Options {
    pub out: PathBuf,
    pub rule_edits: Vec<RuleEdit>,
    pub edits_file: Option<PathBuf>,
    /// Each `--pin` and `--hold`: an idea pinned from the start of the run.
    pub pins: Vec<(String, i64, PinWhen)>,
    pub seeds: Vec<u64>,
    pub rules: Rules,
    /// `--set`: rules' numbers set from the start in every arm, as typed.
    pub sets: Vec<(String, String)>,
    pub families: usize,
    pub end_year: i64,
    pub years: Option<Vec<i64>>,
    pub facts: Option<Vec<String>>,
    pub jobs: Option<usize>,
    pub quiet: bool,
    /// The command line, for README.txt.
    pub command: String,
}

/// One arm of the experiment: its label and its list of dated edits (none
/// for the control).
#[derive(Clone, Debug)]
pub struct Arm {
    pub label: String,
    pub edits: Vec<Dated>,
}

/// Where a row's value comes from.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    /// A stylised fact, by its full name.
    Fact(String),
    /// The year's mean of an observer series.
    Series(String, i64),
}

/// One row of the table.
#[derive(Clone, Debug)]
pub struct Row {
    pub heading: &'static str,
    /// As printed: the fact's name, or `series:money 1950`.
    pub name: String,
    pub year: Option<i64>,
    pub source: Source,
}

/// What a lab will run, worked out and checked before anything runs or is written.
#[derive(Clone, Debug)]
pub struct Plan {
    /// The rules every arm begins with (the seed is set run by run).
    pub params: Params,
    pub content: Content,
    /// The control first.
    pub arms: Vec<Arm>,
    pub years: Vec<i64>,
    pub rows: Vec<Row>,
}

/// A finished lab: for each arm, a sweep of one setting holding its runs.
#[derive(Clone, Debug)]
pub struct Lab {
    pub sweeps: Vec<Sweep>,
    pub jobs: usize,
    pub seconds: f64,
}

// ---------------------------------------------------------------- the command line

/// `NAME=V@YEAR`, or `NAME=V1,V2@YEAR` with several values, split at the
/// commas outside brackets (`road_thresholds=[1500,9000,40000]@1800` is one).
pub fn parse_rule(s: &str) -> Result<RuleEdit, String> {
    let form = "write it as NAME=VALUE@YEAR, or NAME=V1,V2@YEAR for an arm for each value";
    let Some((body, year)) = s.rsplit_once('@') else {
        return Err(format!(
            "--rule {s:?}: say the year the rule is set from, as NAME=VALUE@YEAR; a rule set from the start of the run is a setting of its own, for sweep --set"
        ));
    };
    let year: i64 = year.trim().parse().map_err(|_| format!("--rule {s:?}: {year:?} after the @ is not a year; {form}"))?;
    let (name, values) = body.split_once('=').ok_or_else(|| format!("--rule {s:?}: {form}"))?;
    let (name, values) = (name.trim().to_string(), sweep::split_values(values));
    if name.is_empty() || values.iter().any(|v| v.is_empty()) {
        return Err(format!("--rule {s:?}: {form}"));
    }
    if let Some(twice) = values.iter().enumerate().find(|(k, x)| values[..*k].contains(x)).map(|(_, x)| x) {
        return Err(format!("--rule {name}: {twice} is listed twice"));
    }
    Ok(RuleEdit { name, values, year })
}

/// `1850,1900` or `1850-1900` (every year between).
fn parse_years(s: &str) -> Result<Vec<i64>, String> {
    let bad = |_| format!("--years {s:?}: write it as 1850,1900,1950");
    let mut out = Vec::new();
    for part in s.split(',') {
        match part.split_once('-') {
            Some((a, b)) => {
                let (a, b): (i64, i64) = (a.trim().parse().map_err(bad)?, b.trim().parse().map_err(bad)?);
                out.extend(a..=b);
            }
            None => out.push(part.trim().parse().map_err(bad)?),
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        out: PathBuf::new(),
        rule_edits: Vec::new(),
        edits_file: None,
        pins: Vec::new(),
        seeds: (1..=8).collect(),
        rules: Rules::Game,
        sets: Vec::new(),
        families: 6,
        end_year: 2050,
        years: None,
        facts: None,
        jobs: None,
        quiet: false,
        command: format!("isostate-core lab {}", args.join(" ")),
    };
    let mut out: Option<PathBuf> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value\n{USAGE}"));
        match a.as_str() {
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            "--rule" => o.rule_edits.push(parse_rule(&value("--rule")?)?),
            "--edits" => o.edits_file = Some(PathBuf::from(value("--edits")?)),
            "--pin" | "--hold" => {
                let v = value(a)?;
                let (name, year) = v.rsplit_once('@').ok_or_else(|| format!("{a} {v:?}: write it as NAME@YEAR, the idea's name in the data and a year"))?;
                let year: i64 = year.trim().parse().map_err(|_| format!("{a} {v:?}: {year:?} after the @ is not a year"))?;
                o.pins.push((name.trim().to_string(), year, if a == "--pin" { PinWhen::In } else { PinWhen::NotBefore }));
            }
            "--seeds" => o.seeds = sweep::parse_seeds(&value("--seeds")?)?,
            "--rules" => o.rules = parse_rules(&value("--rules")?)?,
            "--set" => {
                let v = value("--set")?;
                match v.split_once('=') {
                    Some((key, one)) if !key.trim().is_empty() && !one.trim().is_empty() && !one.contains(',') => o.sets.push((key.trim().to_string(), one.trim().to_string())),
                    _ => return Err(format!("--set {v:?}: write it as NAME=VALUE, one value (an arm for each of several values is --rule's)")),
                }
            }
            "--families" => o.families = value("--families")?.parse().map_err(|e| format!("--families: {e}"))?,
            "--end-year" => o.end_year = value("--end-year")?.parse().map_err(|e| format!("--end-year: {e}"))?,
            "--years" => o.years = Some(parse_years(&value("--years")?)?),
            "--facts" => {
                let v = value("--facts")?;
                let list: Vec<String> = v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
                if list.is_empty() {
                    return Err(format!("--facts {v:?}: no names"));
                }
                o.facts = Some(list);
            }
            "--jobs" => {
                let n: usize = value("--jobs")?.parse().map_err(|e| format!("--jobs: {e}"))?;
                o.jobs = Some(n.max(1));
            }
            "--quiet" => o.quiet = true,
            other => return Err(format!("unknown option {other:?}\n{USAGE}")),
        }
    }
    o.out = out.ok_or_else(|| format!("--out is required\n{USAGE}"))?;
    if o.seeds.is_empty() {
        return Err("--seeds: no seeds".to_string());
    }
    let many: Vec<&str> = o.rule_edits.iter().filter(|r| r.values.len() > 1).map(|r| r.name.as_str()).collect();
    if many.len() > 1 {
        return Err(format!(
            "--rule: {} each carry several values; only one --rule may, and each of its values is an arm of its own (a grid of settings is not the lab's yet)",
            many.join(" and ")
        ));
    }
    if o.rule_edits.is_empty() && o.edits_file.is_none() && o.pins.is_empty() {
        return Err(format!("the lab needs at least one edit: --rule NAME=VALUE@YEAR, --pin NAME@YEAR or --edits FILE.json\n{USAGE}"));
    }
    Ok(o)
}

/// The control, then a treated arm for each value of the `--rule` with
/// several (one arm if none has): the file's edits, then the `--rule` edits.
pub fn arms(o: &Options, from_file: &[Dated]) -> Vec<Arm> {
    let choices = o.rule_edits.iter().map(|r| r.values.len()).max().unwrap_or(1);
    let file = o.edits_file.as_ref().map(|p| p.file_name().map_or(p.display().to_string(), |n| n.to_string_lossy().into_owned()));
    let mut out = vec![Arm { label: "control".to_string(), edits: Vec::new() }];
    for c in 0..choices {
        let mut edits = from_file.to_vec();
        let mut said: Vec<String> = file.iter().cloned().collect();
        // the pins are in the list `plan` reads (dated from the run's start), after the file's
        for (name, year, when) in &o.pins {
            said.push(format!("{name} {} {year}", if *when == PinWhen::In { "pinned to" } else { "held back until" }));
        }
        for r in &o.rule_edits {
            let v = if r.values.len() > 1 { &r.values[c] } else { &r.values[0] };
            edits.push(Dated { before: (r.year, 1), edit: Edit::Rule { name: r.name.clone(), value: sweep::parse_value(v) } });
            said.push(format!("{}={v} from {}", r.name, r.year));
        }
        out.push(Arm { label: said.join(" + "), edits });
    }
    out
}

/// The sentence that refuses an arm before anything runs, if any: a date
/// outside the run or a month that is not 1 to 12, and each rule edit, in the
/// order of its date, against the rules as the earlier ones leave them.
pub fn check_arm(P: &Params, content: &Content, arm: &Arm) -> Result<(), String> {
    let (y0, y1) = (content.start_year, P.end_year.unwrap_or(content.end_year));
    if arm.edits.is_empty() {
        return Err(format!("{}: there is no edit in it, and an arm with none is the control", arm.label));
    }
    let mut list: Vec<&Dated> = arm.edits.iter().collect();
    list.sort_by_key(|d| d.before);
    // a pin names an idea of the content as the runs will play it, with the tree laid
    if list.iter().any(|d| matches!(d.edit, Edit::Pin { .. })) {
        let played = crate::tree::content(P, content.clone())?;
        for d in &list {
            if let Edit::Pin { idea, .. } = &d.edit {
                if edits::idea_named(&played, idea).is_none() {
                    return Err(format!("{}: the pin of {idea:?}: there is no such idea in the content these rules play (a method, a technique, or a use of the tasks rule)", arm.label));
                }
            }
        }
    }
    let mut rules = P.clone();
    for d in list {
        let (year, month) = d.before;
        if !(1..=12).contains(&month) {
            return Err(format!("{}: an edit is dated before month {month} of {year}, and the months are 1 to 12", arm.label));
        }
        if year < y0 || year > y1 {
            return Err(format!("{}: an edit is dated before {} {year}, outside the run's years, {y0} to {y1}", arm.label, MONTHS[month - 1]));
        }
        if let Edit::Rule { name, value } = &d.edit {
            if let Some(why) = edits::rule_refused(&rules, name, value) {
                return Err(format!("{}: the edit before {} {year} is refused: {why}", arm.label, MONTHS[month - 1]));
            }
            rules = sweep::set_field(&rules, name, value)?;
        }
    }
    Ok(())
}

/// The rows of the table, each template laid out over the years.
pub fn rows(asked: Option<&[String]>, years: &[i64]) -> Vec<Row> {
    let groups: Vec<(&'static str, Vec<String>)> = match asked {
        Some(list) => vec![(ASKED, list.to_vec())],
        None => ROWS.iter().map(|(h, names)| (*h, names.iter().map(|n| n.to_string()).collect())).collect(),
    };
    let mut out = Vec::new();
    for (heading, names) in groups {
        for t in names {
            if let Some(series) = t.strip_prefix("series:") {
                for &y in years {
                    out.push(Row { heading, name: format!("{t} {y}"), year: Some(y), source: Source::Series(series.to_string(), y) });
                }
            } else if t.contains("{Y}") {
                for &y in years {
                    let name = t.replace("{Y}", &y.to_string());
                    out.push(Row { heading, name: name.clone(), year: Some(y), source: Source::Fact(name) });
                }
            } else {
                out.push(Row { heading, name: t.clone(), year: None, source: Source::Fact(t) });
            }
        }
    }
    out
}

/// The rules, the content, the arms and the rows, or what is wrong with the
/// command line. The edits file is read here.
pub fn plan(o: &Options) -> Result<Plan, String> {
    let mut P = o.rules.params();
    for (key, v) in &o.sets {
        P = sweep::set_field(&P, key, &sweep::parse_value(v))?;
    }
    P.n_families = o.families;
    P.end_year = Some(o.end_year);
    P.check()?;
    let content = o.rules.content();
    check_end_year(&P, &content)?;
    let mut from_file = match &o.edits_file {
        Some(p) => read_edits(p)?,
        None => Vec::new(),
    };
    for (name, year, when) in &o.pins {
        from_file.push(Dated { before: (content.start_year, 1), edit: Edit::Pin { idea: name.clone(), year: *year, when: *when } });
    }
    let arms = arms(o, &from_file);
    for arm in &arms[1..] {
        check_arm(&P, &content, arm)?;
    }
    let (y0, y1) = (content.start_year, o.end_year);
    let mut years: Vec<i64> = o.years.clone().unwrap_or(YEARS.to_vec()).into_iter().filter(|y| (y0..=y1).contains(y)).collect();
    if years.is_empty() {
        if o.years.is_some() {
            return Err(format!("--years: none of them falls in the run's years, {y0} to {y1}"));
        }
        years.push(y1);
    }
    let rows = rows(o.facts.as_deref(), &years);
    Ok(Plan { params: P, content, arms, years, rows })
}

// ---------------------------------------------------------------- running

/// One arm on one island: `sweep::one_run`, by way of the list of edits.
/// With an empty list it gives the sweep's facts to the last bit.
pub fn one_arm(P: &Params, content: Content, list: &[Dated]) -> Result<Done, String> {
    let t0 = Instant::now();
    let cpu0 = sweep::thread_cpu_seconds();
    let mut gathered = facts::Gathered::default();
    let s = edits::play_noting(P, content, list, |s, month| match month {
        Some(m) => facts::gather_month(&mut gathered, s, m),
        None => facts::gather_year(&mut gathered, s),
    })?;
    // as edits::run_with finishes it: the rules the run began with
    let mut r = sim::finish(s);
    r.params = P.clone();
    let seconds = t0.elapsed().as_secs_f64();
    let cpu = match (cpu0, sweep::thread_cpu_seconds()) {
        (Some(a), Some(b)) => Some(b - a),
        _ => None,
    };
    let mut found = facts::stylised_facts(&r, &gathered, seconds);
    found.facts.push(Fact { name: "cpu_seconds".to_string(), group: SPEED, kind: Kind::Real, value: cpu });
    Ok(Done { facts: found, annual: facts::annual_means(&r), names: r.state.series.names.clone(), start_year: r.content.start_year })
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()));
    format!("panicked: {}", text.unwrap_or_else(|| "no message".to_string()))
}

/// Every arm on every island, a few at a time on their own threads. The pool
/// is the sweep's (`sweep::run`), copied. Nothing is written or printed here
/// but the progress lines.
pub fn run(o: &Options, plan: &Plan) -> Lab {
    let Plan { params, content, arms, .. } = plan;
    let todo: Vec<(usize, u64)> = (0..arms.len()).flat_map(|a| o.seeds.iter().map(move |&s| (a, s))).collect();
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
                    let Some(&(a, seed)) = todo.get(id) else { break };
                    let mut P = params.clone();
                    P.seed = seed;
                    let t = Instant::now();
                    let content = content.clone();
                    let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| one_arm(&P, content, &arms[a].edits))) {
                        Ok(r) => r,
                        Err(payload) => Err(panic_message(payload)),
                    };
                    let seconds = t.elapsed().as_secs_f64();
                    if !o.quiet {
                        let n = finished.fetch_add(1, Ordering::Relaxed) + 1;
                        let how = match &outcome {
                            Ok(_) => format!("{seconds:.1} s on the clock"),
                            Err(e) => format!("FAILED: {e}"),
                        };
                        eprintln!("[{n}/{}] seed {seed}, {}: {how}", todo.len(), arms[a].label);
                    }
                    // `setting` holds the arm until the runs are dealt out to their arms
                    slots.lock().unwrap()[id] = Some(Run { id, seed, setting: a, outcome, seconds });
                }
            });
        }
    });
    let all: Vec<Run> = slots.into_inner().unwrap().into_iter().map(|r| r.expect("every run reports")).collect();
    let seconds = t0.elapsed().as_secs_f64();
    let sweeps = arms
        .iter()
        .enumerate()
        .map(|(a, arm)| {
            let mut runs: Vec<Run> = all.iter().filter(|r| r.setting == a).cloned().collect();
            for (k, r) in runs.iter_mut().enumerate() {
                r.id = k;
                r.setting = 0;
            }
            // each arm's news columns are its own: the control gains no news_hand
            let fact_index = sweep::index_runs(&mut runs);
            Sweep {
                rules: o.rules,
                seeds: o.seeds.clone(),
                keys: Vec::new(),
                settings: vec![Setting { label: arm.label.clone(), values: Vec::new(), params: params.clone() }],
                runs,
                fact_index,
                jobs,
                seconds,
            }
        })
        .collect();
    Lab { sweeps, jobs, seconds }
}

// ---------------------------------------------------------------- reading the arms

/// The first year in which any observer series' yearly mean differs between
/// two runs of one island, bit for bit (two NaNs are the same); None if no
/// year does.
pub fn parting_year(control: &Done, arm: &Done) -> Option<i64> {
    let same = |a: Option<&f64>, b: Option<&f64>| match (a, b) {
        (Some(x), Some(y)) => x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()),
        (None, None) => true,
        _ => false,
    };
    let years = control.annual.iter().chain(&arm.annual).map(|a| a.len()).max().unwrap_or(0);
    if control.names != arm.names {
        return Some(control.start_year);
    }
    (0..years).find(|&k| control.annual.iter().zip(&arm.annual).any(|(a, b)| !same(a.get(k), b.get(k)))).map(|k| control.start_year + k as i64)
}

/// A row's value in one run: the fact, or the year's mean of the series;
/// None where the run has none, or it is not a number.
pub fn value(done: &Done, row: &Row) -> Option<f64> {
    let v = match &row.source {
        Source::Fact(name) => done.facts.get(name),
        Source::Series(name, y) => {
            let s = done.names.iter().position(|n| n == name)?;
            let k = usize::try_from(y - done.start_year).ok()?;
            done.annual.get(s)?.get(k).copied()
        }
    };
    v.filter(|x| x.is_finite())
}

/// Each finished run of an arm, by island.
fn by_seed(sw: &Sweep) -> HashMap<u64, &Done> {
    sw.runs.iter().filter_map(|r| r.outcome.as_ref().ok().map(|d| (r.seed, d))).collect()
}

/// How a row's numbers are printed: as its fact is, from any arm that has it.
fn kind_of(lab: &Lab, row: &Row) -> Kind {
    match &row.source {
        Source::Series(..) => Kind::Real,
        Source::Fact(name) => lab.sweeps.iter().flat_map(|sw| sw.fact_index.iter()).find(|(n, _, _)| n == name).map_or(Kind::Real, |(_, _, k)| *k),
    }
}

/// A row in one treated arm against the control, over the islands both have
/// a value on: (island, control, arm).
fn paired(control: &HashMap<u64, &Done>, arm: &HashMap<u64, &Done>, seeds: &[u64], row: &Row) -> Vec<(u64, f64, f64)> {
    seeds
        .iter()
        .filter_map(|s| Some((*s, value(control.get(s)?, row)?, value(arm.get(s)?, row)?)))
        .collect()
}

/// "1 island", "8 islands".
fn islands(n: usize) -> String {
    format!("{n} island{}", if n == 1 { "" } else { "s" })
}

/// The line that says where an arm parts from the control, island by island,
/// and a warning if it parts before its first edit.
fn parting_line(lab: &Lab, a: usize, first_edit: i64) -> String {
    let (control, arm) = (by_seed(&lab.sweeps[0]), by_seed(&lab.sweeps[a]));
    let seeds: Vec<u64> = lab.sweeps[0].seeds.iter().copied().filter(|s| control.contains_key(s) && arm.contains_key(s)).collect();
    let mut years: Vec<(Option<i64>, Vec<u64>)> = Vec::new();
    for &s in &seeds {
        let y = parting_year(control[&s], arm[&s]);
        match years.iter_mut().find(|(z, _)| *z == y) {
            Some((_, list)) => list.push(s),
            None => years.push((y, vec![s])),
        }
    }
    years.sort_by_key(|(y, _)| y.unwrap_or(i64::MAX));
    let label = &lab.sweeps[a].settings[0].label;
    let when = |y: Option<i64>| y.map_or("never".to_string(), |y| format!("in {y}"));
    let mut s = match years.as_slice() {
        [] => format!("{label}: no island has both arms finished"),
        [(y, _)] => format!("{label}: the arms part {} on {} of {}", when(*y), seeds.len(), islands(seeds.len())),
        _ => {
            let parts: Vec<String> = years
                .iter()
                .map(|(y, list)| format!("{} on {} ({})", when(*y), islands(list.len()), list.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")))
                .collect();
            format!("{label}: the arms part {}", parts.join("; "))
        }
    };
    if years.iter().any(|(y, _)| y.is_some_and(|y| y < first_edit)) {
        let _ = write!(s, "\n  BEFORE THE FIRST EDIT, IN {first_edit}: a fault in the lab, not in the island");
    }
    s
}

/// The table: the parting lines, then a row for each fact and year, then the
/// failed runs and the time.
pub fn table(lab: &Lab, plan: &Plan) -> String {
    let control = &lab.sweeps[0];
    let seeds: Vec<String> = control.seeds.iter().map(|s| s.to_string()).collect();
    let runs: usize = lab.sweeps.iter().map(|sw| sw.runs.len()).sum();
    let failed: Vec<(&str, &Run)> = lab.sweeps.iter().flat_map(|sw| sw.runs.iter().filter(|r| r.outcome.is_err()).map(move |r| (sw.settings[0].label.as_str(), r))).collect();
    let (y0, y1) = (plan.content.start_year, plan.params.end_year.unwrap_or(plan.content.end_year));
    let families = match plan.params.n_families {
        0 => String::new(),
        1 => ", 1 family".to_string(),
        n => format!(", {n} families"),
    };
    let mut out = format!(
        "The lab: {} treated arm{} against the control, on islands {} under the {} rules{families}, {y0} to {y1}; {runs} runs, {} failed\n\
         each row: the control's mean across the islands; then for each arm its mean, the change of the means,\n\
         [least .. greatest] of the change island by island, and up/down: the islands where the arm is greater, and where less\n",
        lab.sweeps.len() - 1,
        if lab.sweeps.len() == 2 { "" } else { "s" },
        seeds.join(" "),
        control.rules.as_str(),
        failed.len(),
    );
    let marks = facts::marks(y0, y1);
    let between: Vec<String> = plan.years.iter().filter(|y| !marks.contains(y)).map(|y| y.to_string()).collect();
    if !between.is_empty() {
        let _ = writeln!(
            out,
            "{} not a year the facts are taken in (the first, every fiftieth year to 1900, every tenth after, and the last), so its facts show -; a series row has every year",
            if between.len() == 1 { format!("{} is", between[0]) } else { format!("{} are", between.join(", ")) }
        );
    }
    out.push_str("\nwhere the arms part: the first year whose yearly means differ in any observer series, island by island\n");
    for (a, arm) in plan.arms.iter().enumerate().skip(1) {
        let first = arm.edits.iter().map(|d| d.before.0).min().unwrap_or(y0);
        let _ = writeln!(out, "  {}", parting_line(lab, a, first));
    }
    out.push('\n');

    let by: Vec<HashMap<u64, &Done>> = lab.sweeps.iter().map(by_seed).collect();
    let mut head = vec![String::new(), "control".to_string()];
    let mut sides = vec![Side::Left, Side::Right];
    for (a, sw) in lab.sweeps.iter().enumerate().skip(1) {
        let name = if lab.sweeps.len() == 2 { sw.settings[0].label.clone() } else { format!("arm {a}") };
        head.extend([name, "change".to_string(), String::new(), "up/down".to_string()]);
        sides.extend([Side::Right, Side::Right, Side::Left, Side::Right]);
    }
    let mut rows: Vec<(Option<&'static str>, Vec<String>)> = Vec::new();
    let mut heading = "";
    for row in &plan.rows {
        if row.heading != heading {
            heading = row.heading;
            rows.push((Some(row.heading), Vec::new()));
        }
        let control_values: Vec<(u64, f64)> = control.seeds.iter().filter_map(|s| Some((*s, value(by[0].get(s)?, row)?))).collect();
        let mut kind = kind_of(lab, row);
        // a series in the thousands (the money) is printed as a count is, with its commas
        if matches!(row.source, Source::Series(..)) && sweep::stat(&control_values).is_some_and(|s| s.mean.abs() >= 1000.0) {
            kind = Kind::Count;
        }
        let mut cells = vec![row.name.clone(), sweep::stat(&control_values).map_or("-".to_string(), |s| sweep::fmt_fact(kind, s.mean))];
        for a in 1..lab.sweeps.len() {
            let pairs = paired(&by[0], &by[a], &control.seeds, row);
            if pairs.is_empty() {
                cells.extend(["-".to_string(), String::new(), String::new(), String::new()]);
                continue;
            }
            let n = pairs.len() as f64;
            let (c, t) = (pairs.iter().map(|p| p.1).sum::<f64>() / n, pairs.iter().map(|p| p.2).sum::<f64>() / n);
            let (up, down) = (pairs.iter().filter(|p| p.2 > p.1).count(), pairs.iter().filter(|p| p.2 < p.1).count());
            let mean = sweep::fmt_fact(kind, t);
            if up + down == 0 {
                cells.extend([mean, "=".to_string(), String::new(), "0/0".to_string()]);
                continue;
            }
            let d: Vec<(u64, f64)> = pairs.iter().map(|p| (p.0, p.2 - p.1)).collect();
            let s = sweep::stat(&d).expect("pairs are not empty");
            let (least, greatest) = (sweep::fmt_change(kind, s.min, 0.0), sweep::fmt_change(kind, s.max, 0.0));
            let mut range = format!("[{least} .. {greatest}]");
            if pairs.len() < control.seeds.len() {
                let _ = write!(range, " ({} paired)", pairs.len());
            }
            cells.extend([mean, sweep::fmt_change(kind, t, c), range, format!("{up}/{down}")]);
        }
        rows.push((None, cells));
    }
    if lab.sweeps.len() > 2 {
        for (a, sw) in lab.sweeps.iter().enumerate().skip(1) {
            let _ = writeln!(out, "arm {a}: {}", sw.settings[0].label);
        }
        out.push('\n');
    }
    out.push_str(&sweep::layout(&head, &sides, &rows));

    for (label, run) in &failed {
        let _ = writeln!(out, "\nFAILED: island {}, {label}: {}", run.seed, run.outcome.as_ref().err().unwrap());
    }
    let mean = lab.sweeps.iter().flat_map(|sw| sw.runs.iter().map(|r| r.seconds)).sum::<f64>() / runs.max(1) as f64;
    let _ = writeln!(out, "\n{:.0} s on the clock for the whole, {} at a time; {mean:.1} s a run on average", lab.seconds, lab.jobs);
    out
}

// ---------------------------------------------------------------- the files

fn csv_num(v: Option<f64>) -> String {
    v.map_or(String::new(), |v| format!("{v}"))
}

fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\n']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() }
}

/// The table in long form: one line for each treated arm, island, row and year.
pub fn lab_csv(lab: &Lab, plan: &Plan) -> String {
    let mut out = "arm,label,seed,row,year,control,value,change\n".to_string();
    let control = by_seed(&lab.sweeps[0]);
    for (a, sw) in lab.sweeps.iter().enumerate().skip(1) {
        let arm = by_seed(sw);
        for &seed in &sw.seeds {
            for row in &plan.rows {
                let c = control.get(&seed).and_then(|d| value(d, row));
                let t = arm.get(&seed).and_then(|d| value(d, row));
                let change = c.zip(t).map(|(c, t)| t - c);
                let year = row.year.map_or(String::new(), |y| y.to_string());
                let _ = writeln!(out, "{a},{},{seed},{},{year},{},{},{}", csv_cell(&sw.settings[0].label), csv_cell(&row.name), csv_num(c), csv_num(t), csv_num(change));
            }
        }
    }
    out
}

/// `git describe` in the directory the lab was started from, if git is there.
fn git_describe() -> Option<String> {
    let out = std::process::Command::new("git").args(["describe", "--always", "--dirty"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !text.is_empty()).then_some(text)
}

/// The directory each arm's own files go in.
fn arm_dir(a: usize) -> String {
    if a == 0 { "control".to_string() } else { format!("arm-{a}") }
}

fn edits_json(plan: &Plan) -> Value {
    Value::Array(plan.arms.iter().enumerate().map(|(a, arm)| json!({"arm": arm_dir(a), "label": arm.label, "edits": arm.edits})).collect())
}

fn readme(lab: &Lab, plan: &Plan, o: &Options, code: Option<&str>) -> String {
    let runs: usize = lab.sweeps.iter().map(|sw| sw.runs.len()).sum();
    let failed: usize = lab.sweeps.iter().map(|sw| sw.runs.iter().filter(|r| r.outcome.is_err()).count()).sum();
    let seeds: Vec<String> = o.seeds.iter().map(|s| s.to_string()).collect();
    let mut s = String::new();
    let _ = writeln!(s, "A laboratory of Isolated State (isostate-core lab), {} (UTC).\n", crate::record::today());
    let _ = writeln!(s, "command:  {}", o.command);
    let _ = writeln!(s, "rules:    {}, {} families, {} to {}", o.rules.as_str(), o.families, plan.content.start_year, o.end_year);
    if !o.sets.is_empty() {
        let set: Vec<String> = o.sets.iter().map(|(k, v)| format!("{k}={v}")).collect();
        let _ = writeln!(s, "set:      {} (in every arm, from the start)", set.join(" "));
    }
    let _ = writeln!(s, "code:     {}", code.map_or("unknown (git describe gave nothing)".to_string(), |c| format!("{c} (git describe where it was run)")));
    let _ = writeln!(s, "seeds:    {}", seeds.join(" "));
    let _ = writeln!(s, "runs:     {runs} ({failed} failed), {} at a time, {:.0} s in all", lab.jobs, lab.seconds);
    let _ = writeln!(s, "arms:");
    for (a, arm) in plan.arms.iter().enumerate() {
        let _ = writeln!(s, "  {:<9} {}: {}", arm_dir(a), arm.label, serde_json::to_string(&arm.edits).unwrap_or_default());
    }
    s.push_str(
        "\nFiles\n\
         lab.txt      the table the lab printed\n\
         lab.csv      the table in long form: one line for each treated arm, island, row and year, with the\n\
         \x20            control's value, the arm's and the change (empty where a run has no value)\n\
         edits.json   every arm's label and list of dated edits\n\
         control/, arm-1/ ...  each arm as a sweep of one setting: facts.json, facts.csv and annual.csv in the\n\
         \x20            sweep's own form, so sweep --compare DIR/control and the scorecard read them; and\n\
         \x20            edits.json, the arm's list in the form run --edits and lab --edits read\n\
         \nWhat the lab does is in docs/lab.md; what each fact means, in docs/rules/ and at the top of\n\
         crates/isostate-core/src/facts.rs.\n",
    );
    s
}

pub fn write_outputs(lab: &Lab, plan: &Plan, o: &Options, text: &str) -> Result<(), String> {
    let dir = &o.out;
    let put = |name: &str, body: String| std::fs::write(dir.join(name), body).map_err(|e| format!("{}: {e}", dir.join(name).display()));
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let code = git_describe();
    put("lab.txt", text.to_string())?;
    put("lab.csv", lab_csv(lab, plan))?;
    put("edits.json", serde_json::to_string_pretty(&edits_json(plan)).map_err(|e| e.to_string())? + "\n")?;
    put("README.txt", readme(lab, plan, o, code.as_deref()))?;
    for (a, sw) in lab.sweeps.iter().enumerate() {
        let sub = arm_dir(a);
        std::fs::create_dir_all(dir.join(&sub)).map_err(|e| format!("{}: {e}", dir.join(&sub).display()))?;
        put(&format!("{sub}/annual.csv"), sweep::annual_csv(sw))?;
        put(&format!("{sub}/facts.csv"), sweep::facts_csv(sw))?;
        put(&format!("{sub}/facts.json"), serde_json::to_string_pretty(&sweep::facts_json(sw, &o.command, code.as_deref())).map_err(|e| e.to_string())? + "\n")?;
        // the arm's own list, as run --edits and lab --edits read it, to play it again
        put(&format!("{sub}/edits.json"), serde_json::to_string_pretty(&plan.arms[a].edits).map_err(|e| e.to_string())? + "\n")?;
    }
    Ok(())
}

// ---------------------------------------------------------------- the command

/// The `lab` subcommand: check, run, write the files, print the table.
pub fn main_with_args(args: &[String]) -> Result<(), String> {
    command(args, true)
}

/// The subcommand with or without the table on stdout (it is in lab.txt
/// either way; the tests leave it off).
pub fn command(args: &[String], print: bool) -> Result<(), String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let o = parse_args(args)?;
    // whatever is wrong is found before the first run and before the directory is made
    let plan = plan(&o)?;
    std::fs::create_dir_all(&o.out).map_err(|e| format!("--out {}: {e}", o.out.display()))?;
    let lab = run(&o, &plan);
    let text = table(&lab, &plan);
    write_outputs(&lab, &plan, &o, &text)?;
    if print {
        // not print!, which panics when the reader has gone (lab ... | head)
        let _ = std::io::Write::write_all(&mut std::io::stdout(), text.as_bytes());
    }
    let runs: usize = lab.sweeps.iter().map(|sw| sw.runs.len()).sum();
    let failed: usize = lab.sweeps.iter().map(|sw| sw.runs.iter().filter(|r| r.outcome.is_err()).count()).sum();
    if failed > 0 {
        return Err(format!("{failed} of {runs} runs failed; the rest are in {}", o.out.display()));
    }
    Ok(())
}
