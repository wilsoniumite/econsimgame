//! The core of the island: every rule the game switches on, turned off in turn.
//! Not in the Python; a tool, not a rule, so it has no switch and changes
//! nothing a run does. `docs/core.md` is the page it writes, and the page's
//! last part says how to read it.
//!
//! ```text
//! isostate-core ablate --out DIR [--only NAME,...] [--seeds 1-8] [--families N] [--end-year Y]
//!                      [--jobs N] [--no-second] [--page FILE] [--quiet]
//! isostate-core ablate --again DIR [--page FILE]
//! ```
//!
//! When the paper's model was calibrated, it was gone through every so often
//! and things were cut, because a rule added to mend one behaviour may have
//! been made needless by a later one, and the simple core of a model is worth
//! knowing. Here nothing is cut: every rule of the game sits behind a switch,
//! so each can be turned off from 1750 and the island asked what changed.
//!
//! The command plays the islands under the game's rules as they are (the
//! control), and once more for each rule switched on under them, with that one
//! rule off from the start (an arm, exactly what `sweep --set NAME=false`
//! plays). The rules are found in the tree's own lists, not in one kept here
//! ([`asked`]): every true/false name the rulebook's walk finds on under
//! `game_params()` and off under the port's, and every lesser switch of a rule
//! that is on under the game's rules. So a rule added tomorrow is asked too.
//! A name the game changes whose "off" is not a switch (a number, a regime of
//! several kinds) is left out, and listed with why.
//!
//! Each arm is set against the control, island by island, and sorted into one
//! of four kinds by those numbers alone ([`role`]):
//!
//! - **leaned on**: a run fails with it off, so other rules lean on it;
//! - **holds up the century**: one of the standing check's twelve facts of the
//!   played century (`standing.rs`) is not inside the control's band;
//! - **shapes the island**: facts stand apart (every island of one arm above,
//!   or below, every island of the other, as `sweep --compare` stars them), and
//!   the twelve stay inside;
//! - **idle**: nothing stands apart.
//!
//! The facts counted are those both arms have, outside the Speed and the News
//! groups (how long a run took is no fact about the island, and the news is
//! what the chronicle writes about it). A fact the arm does not keep at all (a
//! gauge with the gauges off) is listed beside, and counts for nothing: a rule
//! that only watches is idle, and the page says it watches.
//!
//! Then the second pass ([`search`]): rules cover for each other, so two that
//! mend one behaviour each look idle alone. One more arm turns every idle rule
//! off together. If it is idle too, the island is the same island with fewer
//! rules. If not, the set is halved until the smallest groups that must not go
//! off together are found: a group is found, one of it is kept on, and the rest
//! are tried together again, until they are idle.
//!
//! Two refinements of the four kinds, both read from the same numbers. A fact
//! of the twelve that the arm keeps no value of (the books' work's share, with
//! the books off) holds nothing up: the rule measures it, and is listed as not
//! keeping it. And with eight islands a side a fact the rule only stirs stands
//! apart by chance once in 6,435, so among the 2,700 or so facts a rule stirs
//! about 0.4 stand apart by chance alone ([`chance`]); a rule that shapes the
//! island with no more than chance gives one time in twenty is marked faint
//! ([`faint`]).
//!
//! `--again DIR` reads a finished table back from the files it wrote and
//! writes its page anew with this code's reading, playing nothing (a table to
//! 2050 took 23 minutes on 20 November 2026, 392 runs at the default of half
//! the machine's processors, 24 at a time; 4 h 05 the day before, when the
//! threads still queued on one heap, `heaps.rs`); the second pass stays the
//! one played.
//!
//! `--jobs` is how many runs at a time: half the machine's processors unless
//! it says otherwise, which on a machine of 24 cores is one run a core. Each
//! run's thread has a heap of its own, so none waits on another: at 24 at a
//! time a run to 2050 took 80 s of processor and 81 on the clock, against 58
//! alone, the cores slowing each other through the clock, the caches and the
//! memory they share, not a queue.
//!
//! What it writes in `DIR`: `ablate.txt` (the table it prints), `core.md` and
//! `core.svg` (the page and its chart), `ablate.json` (every arm's numbers),
//! `README.txt`, and for each arm a directory (`control/`, `off-1/` ...) with
//! `facts.json`, `facts.csv` and `annual.csv` in the sweep's own form, so that
//! `sweep --compare DIR/control` reads an arm. With `--page FILE` the page is
//! written there too, and its chart beside it as `core.svg`.

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde_json::{Value, json};

use crate::cli::check_end_year;
use crate::content::Content;
use crate::edits::SWITCHES;
use crate::facts::{Kind, NEWS, SPEED};
use crate::params::{Params, Rules, default_params, game_params};
use crate::rulebook::{self, Entry};
use crate::standing::{self, FACTS, GROWTH, Verdict};
use crate::sweep::{self, Run, STAR_RUNS, Setting, Side, Sweep, Then, stat};

pub const USAGE: &str = "usage: isostate-core ablate --out DIR [--only NAME,...] [--seeds 1-8] [--families N] [--end-year Y]
                            [--jobs N] [--no-second] [--page FILE] [--quiet]
       isostate-core ablate --again DIR [--page FILE]";

/// How many of the largest facts apart are named for each rule.
pub const LARGEST: usize = 5;

// ---------------------------------------------------------------- which rules

/// A name the command leaves out, and why.
#[derive(Clone, Debug, PartialEq)]
pub struct LeftOut {
    pub name: String,
    pub why: String,
}

/// The rules asked, by the dotted name `--set` takes, in the order of
/// `Params`; and the names left out, with why. Read from the rulebook's walk of
/// `game` and of `port`:
///
/// - a true/false name on under `game` and off under `port` is asked: a rule's
///   own switch (`edits::SWITCHES`), or a lesser one (`works.keep_order`);
/// - a true/false name on under both is asked when its rule is the game's (its
///   own switch is on under `game`, or, with no switch of its own, the rule
///   differs from the port's: the tree), and left out otherwise (it is part of
///   the port);
/// - a rule's own switch off under `game` is left out: there is nothing to turn off;
/// - any other name that differs between the two and does not belong to a rule
///   with a switch asked is left out: its "off" is not a switch.
pub fn asked(game: &Params, port: &Params) -> (Vec<String>, Vec<LeftOut>) {
    let g = rulebook::rulebook(game);
    let p: HashMap<String, Value> = rulebook::rulebook(port).into_iter().map(|e| (e.name, e.value)).collect();
    let on = |name: &str| g.iter().any(|e| e.name == name && e.value == Value::Bool(true));
    let own_switch = |rule: &str| SWITCHES.iter().find(|s| s.split('.').next() == Some(rule)).copied();
    // a rule is the game's when its own switch is on, or, for a rule with no
    // switch of its own (the tree, whose stage says how much of it there is),
    // when any of its names differs from the port's
    let rule_on = |rule: &str| match own_switch(rule) {
        Some(s) => on(s),
        None => rule != "general" && g.iter().any(|e| e.rule == rule && p.get(&e.name) != Some(&e.value)),
    };
    let mut ask: Vec<String> = Vec::new();
    let mut out: Vec<LeftOut> = Vec::new();
    let leave = |out: &mut Vec<LeftOut>, e: &Entry, why: String| out.push(LeftOut { name: e.name.clone(), why });
    for e in &g {
        let port_value = p.get(&e.name);
        match &e.value {
            Value::Bool(true) => {
                if port_value != Some(&Value::Bool(true)) {
                    ask.push(e.name.clone());
                } else if own_switch(&e.rule) != Some(e.name.as_str()) && rule_on(&e.rule) {
                    // a lesser switch, on under both, of a rule the game switches on
                    ask.push(e.name.clone());
                } else {
                    leave(&mut out, e, "on under the port's rules too: it is part of the port, not a rule the game adds".to_string());
                }
            }
            Value::Bool(false) if SWITCHES.contains(&e.name.as_str()) => {
                leave(&mut out, e, "off under the game's rules already: there is nothing to turn off".to_string());
            }
            Value::Bool(false) => {}
            v => {
                if port_value == Some(v) {
                    continue;
                }
                // a number of a rule whose own switch is asked goes off with it
                if own_switch(&e.rule).is_some_and(|s| on(s)) {
                    continue;
                }
                let port_said = port_value.map_or("not walked".to_string(), |x| x.to_string());
                let why = if e.fixed.is_some() {
                    format!("{v} under the game's rules and {port_said} under the port's; fixed when the island is made, and not a switch: its off is not one value")
                } else {
                    format!("{v} under the game's rules and {port_said} under the port's: a number or a choice, not a switch, so its off is not one value")
                };
                leave(&mut out, e, why);
            }
        }
    }
    (ask, out)
}

// ---------------------------------------------------------------- the command line

/// What the command line asked for.
#[derive(Clone, Debug)]
pub struct Options {
    pub out: PathBuf,
    pub only: Option<Vec<String>>,
    pub seeds: Vec<u64>,
    pub families: usize,
    pub end_year: i64,
    pub jobs: Option<usize>,
    pub second: bool,
    pub page: Option<PathBuf>,
    /// `--again DIR`: read a finished table in DIR anew and write its page; nothing is played.
    pub again: bool,
    pub quiet: bool,
    /// The command line, for README.txt and the page.
    pub command: String,
}

pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        out: PathBuf::new(),
        only: None,
        seeds: (1..=8).collect(),
        families: 6,
        end_year: 2050,
        jobs: None,
        second: true,
        page: None,
        again: false,
        quiet: false,
        command: format!("isostate-core ablate {}", args.join(" ")),
    };
    let mut out: Option<PathBuf> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value\n{USAGE}"));
        match a.as_str() {
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            "--only" => {
                let v = value("--only")?;
                let list: Vec<String> = v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
                if list.is_empty() {
                    return Err(format!("--only {v:?}: no names"));
                }
                o.only = Some(list);
            }
            "--seeds" => o.seeds = sweep::parse_seeds(&value("--seeds")?)?,
            "--families" => o.families = value("--families")?.parse().map_err(|e| format!("--families: {e}"))?,
            "--end-year" => o.end_year = value("--end-year")?.parse().map_err(|e| format!("--end-year: {e}"))?,
            "--jobs" => {
                let n: usize = value("--jobs")?.parse().map_err(|e| format!("--jobs: {e}"))?;
                o.jobs = Some(n.max(1));
            }
            "--no-second" => o.second = false,
            "--page" => o.page = Some(PathBuf::from(value("--page")?)),
            "--again" => {
                out = Some(PathBuf::from(value("--again")?));
                o.again = true;
            }
            "--quiet" => o.quiet = true,
            "--rules" => return Err("--rules: the command asks the game's rules, one at a time; under the port's nothing is switched on to turn off".to_string()),
            other => return Err(format!("unknown option {other:?}\n{USAGE}")),
        }
    }
    o.out = out.ok_or_else(|| format!("--out is required\n{USAGE}"))?;
    if o.seeds.is_empty() {
        return Err("--seeds: no seeds".to_string());
    }
    Ok(o)
}

/// One arm: its label, the names it turns off, and its rules (or why the
/// rules refuse it).
#[derive(Clone, Debug)]
pub struct Arm {
    pub label: String,
    pub off: Vec<String>,
    pub params: Result<Params, String>,
}

/// What the command will play, checked before anything runs or is written.
#[derive(Clone, Debug)]
pub struct Plan {
    pub base: Params,
    pub content: Content,
    /// Every rule found on under the game's rules, and what was left out.
    pub found: Vec<String>,
    pub left_out: Vec<LeftOut>,
    /// The rules asked this time (`--only`, or all found).
    pub asked: Vec<String>,
}

/// The game's rules with these names turned off from 1750, or why not.
pub fn off(base: &Params, names: &[String]) -> Result<Params, String> {
    let mut P = base.clone();
    for n in names {
        P = sweep::set_field(&P, n, &Value::Bool(false))?;
    }
    P.check()?;
    Ok(P)
}

pub fn plan(o: &Options) -> Result<Plan, String> {
    let mut base = game_params();
    base.n_families = o.families;
    base.end_year = Some(o.end_year);
    base.check()?;
    let content = Rules::Game.content();
    check_end_year(&base, &content)?;
    let (found, left_out) = asked(&game_params(), &default_params());
    let asked = match &o.only {
        None => found.clone(),
        Some(list) => {
            for n in list {
                if !found.contains(n) {
                    let why = match left_out.iter().find(|l| &l.name == n) {
                        Some(l) => format!("--only {n}: it is left out: {}", l.why),
                        None => format!("--only {n}: not a rule switched on under the game's rules; those are: {}", found.join(", ")),
                    };
                    return Err(why);
                }
            }
            found.iter().filter(|n| list.contains(n)).cloned().collect()
        }
    };
    Ok(Plan { base, content, found, left_out, asked })
}

// ---------------------------------------------------------------- playing

/// The arms' runs, a few at a time on their own threads, each arm returned as a
/// sweep of one setting. A run is `lab::one_arm` with no edits, which is the
/// sweep's own run to the last bit; an arm the rules refuse has no runs.
pub fn play(arms: &[Arm], seeds: &[u64], content: &Content, jobs: Option<usize>, quiet: bool) -> (Vec<Sweep>, usize, f64) {
    let todo: Vec<(usize, u64)> = (0..arms.len()).filter(|&a| arms[a].params.is_ok()).flat_map(|a| seeds.iter().map(move |&s| (a, s))).collect();
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    let jobs = jobs.unwrap_or((cores / 2).max(1)).min(todo.len()).max(1);
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
                    let mut P = arms[a].params.clone().expect("only arms the rules take are played");
                    P.seed = seed;
                    let t = Instant::now();
                    let content = content.clone();
                    let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::lab::one_arm(&P, content, &[]))) {
                        Ok(r) => r,
                        Err(payload) => Err(panic_message(payload)),
                    };
                    let seconds = t.elapsed().as_secs_f64();
                    if !quiet {
                        let n = finished.fetch_add(1, Ordering::Relaxed) + 1;
                        let how = match &outcome {
                            Ok(_) => format!("{seconds:.1} s on the clock"),
                            Err(e) => format!("FAILED: {e}"),
                        };
                        eprintln!("[{n}/{}] seed {seed}, {}: {how}", todo.len(), arms[a].label);
                    }
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
            let fact_index = sweep::index_runs(&mut runs);
            let params = arm.params.clone().unwrap_or_else(|_| game_params());
            Sweep {
                rules: Rules::Game,
                seeds: seeds.to_vec(),
                keys: Vec::new(),
                settings: vec![Setting { label: arm.label.clone(), values: Vec::new(), params }],
                runs,
                fact_index,
                jobs,
                seconds,
            }
        })
        .collect();
    (sweeps, jobs, seconds)
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()));
    format!("panicked: {}", text.unwrap_or_else(|| "no message".to_string()))
}

// ---------------------------------------------------------------- reading an arm

/// A fact that stands apart: its kind, the control's mean and the arm's, and
/// its size, the change of the means over the control's spread from island to
/// island (the arm's where the control's is nought; infinite where both are).
#[derive(Clone, Debug, PartialEq)]
pub struct Apart {
    pub name: String,
    pub kind: Kind,
    pub control: f64,
    pub arm: f64,
    pub size: f64,
    /// The control or the arm is the same on every island, which chance cannot give.
    pub certain: bool,
}

/// What an arm does to the facts, against the control.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    /// Facts both have, outside Speed and News.
    pub both: usize,
    /// Of those, the facts that differ on any island.
    pub differ: usize,
    /// Of those, the facts that stand apart, largest first.
    pub apart: Vec<Apart>,
    /// Facts the control keeps on some island and the arm on none.
    pub gone: Vec<String>,
    /// Facts the arm keeps on some island and the control on none.
    pub new: Vec<String>,
}

/// Each fact's group and kind, by name.
pub type Index = HashMap<String, (&'static str, Kind)>;

/// The arm's facts against the control's, island by island over the islands
/// both finished (as `sweep::compare` sets a sweep against an earlier one).
pub fn read_facts(control: &Then, arm: &Then, index: &Index) -> Facts {
    let common: Vec<u64> = control.seeds.iter().copied().filter(|s| arm.seeds.contains(s)).collect();
    let kept = |v: Option<&Vec<(u64, f64)>>| -> Vec<(u64, f64)> { v.into_iter().flatten().copied().filter(|(s, _)| common.contains(s)).collect() };
    let mut names: BTreeSet<&String> = control.values.keys().collect();
    names.extend(arm.values.keys());
    let mut f = Facts::default();
    for name in names {
        let (group, kind) = index.get(name).copied().unwrap_or(("", Kind::Real));
        if group == SPEED || group == NEWS {
            continue;
        }
        let (c, a) = (kept(control.values.get(name)), kept(arm.values.get(name)));
        match (c.is_empty(), a.is_empty()) {
            (true, true) => continue,
            (false, true) => {
                f.gone.push(name.clone());
                continue;
            }
            (true, false) => {
                f.new.push(name.clone());
                continue;
            }
            _ => {}
        }
        f.both += 1;
        let same = c.len() == a.len() && c.iter().all(|(s, x)| a.iter().any(|(t, y)| t == s && x.to_bits() == y.to_bits()));
        if same {
            continue;
        }
        f.differ += 1;
        let (Some(sc), Some(sa)) = (stat(&c), stat(&a)) else { continue };
        if sc.n >= STAR_RUNS && sa.n >= STAR_RUNS && (sa.min > sc.max || sa.max < sc.min) {
            let spread = if sc.max > sc.min { sc.max - sc.min } else { sa.max - sa.min };
            let d = (sa.mean - sc.mean).abs();
            let size = if spread > 0.0 { d / spread } else { f64::INFINITY };
            f.apart.push(Apart { name: name.clone(), kind, control: sc.mean, arm: sa.mean, size, certain: sc.max == sc.min || sa.max == sa.min });
        }
    }
    f.apart.sort_by(|a, b| b.size.total_cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
    f
}

/// What turning a rule off does: everything the page says of an arm.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub label: String,
    pub off: Vec<String>,
    /// Why the rules refuse the arm, if they do (then nothing ran).
    pub refused: Option<String>,
    /// The runs that failed: island and why.
    pub failed: Vec<(u64, String)>,
    /// The standing check's twelve, against the control's band.
    pub standing: Vec<(&'static str, Verdict)>,
    /// The three asked of the century after 1900: the mean across the islands
    /// and whether it is met.
    pub growth: [(Option<f64>, Option<bool>); 3],
    pub facts: Facts,
    /// Mean processor seconds and seconds on the clock of a run.
    pub cpu: Option<f64>,
    pub clock: Option<f64>,
    /// The islands both arms finished.
    pub islands: usize,
}

/// The growth after 1900 of an arm: each of the three's mean and whether it is met.
pub fn growth_of(arm: &Then) -> [(Option<f64>, Option<bool>); 3] {
    let v = standing::growth_values(arm);
    let met = standing::growth_met(&v);
    std::array::from_fn(|k| (stat(&v[k]).map(|s| s.mean), met[k]))
}

fn mean_fact(sw: &Sweep, name: &str) -> Option<f64> {
    let v: Vec<(u64, f64)> = sw.runs.iter().filter_map(|r| r.outcome.as_ref().ok().and_then(|d| d.facts.get(name)).map(|x| (r.seed, x))).collect();
    stat(&v).map(|s| s.mean)
}

fn mean_clock(sw: &Sweep) -> Option<f64> {
    let v: Vec<(u64, f64)> = sw.runs.iter().filter(|r| r.outcome.is_ok()).map(|r| (r.seed, r.seconds)).collect();
    stat(&v).map(|s| s.mean)
}

/// A sweep of one setting as an arm, each fact's values by island. A run's
/// facts can name one fact twice (a fact of the last year when that year is
/// also one of the marks); the last is kept, as the run's facts.json keeps it
/// and `sweep --compare` reads it.
pub fn then_of(sw: &Sweep) -> Then {
    let mut t = Then { label: sw.settings.first().map_or(String::new(), |s| s.label.clone()), ..Then::default() };
    for run in &sw.runs {
        let Ok(done) = &run.outcome else { continue };
        t.seeds.push(run.seed);
        let mut one: HashMap<&str, f64> = HashMap::new();
        for f in &done.facts.facts {
            match f.value {
                Some(v) => {
                    one.insert(f.name.as_str(), v);
                }
                None => {
                    one.remove(f.name.as_str());
                }
            }
        }
        for (name, v) in one {
            t.values.entry(name.to_string()).or_default().push((run.seed, v));
        }
    }
    for v in t.values.values_mut() {
        v.sort_by_key(|&(seed, _)| seed);
    }
    t
}

/// The twelve of the standing check, the arm against the control; a fact the
/// control has no value of (a run that ends before its year) is not asked.
pub fn standing_of(c: &Then, a: &Then) -> Vec<(&'static str, Verdict)> {
    let st = |t: &Then, name: &str| t.values.get(name).and_then(|v| stat(v));
    FACTS.iter().filter(|f| st(c, f).is_some()).map(|&f| (f, standing::verdict(st(c, f), st(a, f)))).collect()
}

/// One arm read against the control.
pub fn read(control: &Sweep, arm: &Arm, sw: &Sweep) -> Reading {
    let mut index: Index = HashMap::new();
    for (n, g, k) in control.fact_index.iter().chain(&sw.fact_index) {
        index.entry(n.clone()).or_insert((*g, *k));
    }
    let failed = sw.runs.iter().filter_map(|r| r.outcome.as_ref().err().map(|e| (r.seed, e.clone()))).collect();
    let mut r = read_thens(&then_of(control), &then_of(sw), &index, arm.params.as_ref().err().cloned(), failed);
    (r.label, r.off, r.cpu, r.clock) = (arm.label.clone(), arm.off.clone(), mean_fact(sw, "cpu_seconds"), mean_clock(sw));
    r
}

/// An arm read against the control from each one's facts by island (the
/// label, the names off and the times are left for the caller).
pub fn read_thens(c: &Then, a: &Then, index: &Index, refused: Option<String>, failed: Vec<(u64, String)>) -> Reading {
    Reading {
        label: String::new(),
        off: Vec::new(),
        failed,
        standing: if refused.is_some() { Vec::new() } else { standing_of(c, a) },
        growth: growth_of(a),
        facts: if refused.is_some() { Facts::default() } else { read_facts(c, a, index) },
        islands: c.seeds.iter().filter(|s| a.seeds.contains(s)).count(),
        cpu: None,
        clock: None,
        refused,
    }
}

/// How many facts would stand apart by chance alone among those an arm stirs:
/// two sets of n islands drawn from one spread lie wholly apart with chance
/// 2 / C(2n, n) (1 in 6,435 with eight a side), for each fact that differs.
/// Facts move together, so chance gives them in clusters and this is a floor.
pub fn chance(r: &Reading) -> f64 {
    let n = r.islands;
    if n < STAR_RUNS {
        return 0.0;
    }
    let ways = (0..n).fold(1.0f64, |c, k| c * (2 * n - k) as f64 / (k + 1) as f64);
    r.facts.differ as f64 * 2.0 / ways
}

/// The chance of `k` or more facts apart by chance alone, as if each stood
/// apart alone (a Poisson count of mean [`chance`]).
pub fn chance_of(k: usize, mean: f64) -> f64 {
    let mut term = (-mean).exp();
    let mut below = 0.0;
    for j in 0..k {
        below += term;
        term *= mean / (j + 1) as f64;
    }
    (1.0 - below).max(0.0)
}

/// A rule that shapes the island only faintly: no more facts stand apart
/// than chance alone gives one time in twenty, and none of them is one whose
/// control or arm is the same on every island (which chance cannot give).
pub fn faint(r: &Reading) -> bool {
    role(r) == Role::Shapes && chance_of(r.facts.apart.len(), chance(r)) >= 0.05 && r.facts.apart.iter().all(|a| a.size.is_finite() && !a.certain)
}

/// The four kinds a rule is sorted into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// Holds up the century: with it off a standing fact is not inside its band.
    HoldsUp,
    /// Shapes the island: facts stand apart, and the twelve stay inside.
    Shapes,
    /// Idle: nothing stands apart.
    Idle,
    /// Leaned on: a run fails without it, or the rules refuse to run without it.
    LeanedOn,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::HoldsUp => "holds up the century",
            Role::Shapes => "shapes the island",
            Role::Idle => "idle",
            Role::LeanedOn => "leaned on",
        }
    }

    /// As the page counts them: "4 hold up the century".
    pub fn plural(self) -> &'static str {
        match self {
            Role::HoldsUp => "hold up the century",
            Role::Shapes => "shape the island",
            Role::Idle => "are idle",
            Role::LeanedOn => "are leaned on",
        }
    }

    pub const ALL: [Role; 4] = [Role::HoldsUp, Role::Shapes, Role::Idle, Role::LeanedOn];
}

/// The kind of a rule, by its arm's numbers alone.
pub fn role(r: &Reading) -> Role {
    if r.refused.is_some() || !r.failed.is_empty() {
        Role::LeanedOn
    } else if !held_up(r).is_empty() {
        Role::HoldsUp
    } else if !r.facts.apart.is_empty() {
        Role::Shapes
    } else {
        Role::Idle
    }
}

/// The twelve facts that move out of their band, moved or OUTSIDE. A fact the
/// arm keeps no value of is not counted: the rule measures it, and with the
/// rule off it is not measured (`unkept`), which says nothing of the century.
pub fn held_up(r: &Reading) -> Vec<(&'static str, Verdict)> {
    r.standing.iter().copied().filter(|(_, v)| matches!(v, Verdict::Moved | Verdict::Outside)).collect()
}

/// The twelve facts the arm keeps no value of.
pub fn unkept(r: &Reading) -> Vec<&'static str> {
    r.standing.iter().filter(|(_, v)| *v == Verdict::Missing).map(|(f, _)| *f).collect()
}

/// Whether an arm is the same island as the control, as far as the command
/// asks: no run fails, the twelve stay inside, and nothing stands apart.
pub fn same_island(r: &Reading) -> bool {
    role(r) == Role::Idle
}

// ---------------------------------------------------------------- the second pass

/// What the second pass found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Second {
    /// The idle rules it began with.
    pub idle: Vec<String>,
    /// Whether all of them off together is the same island; None if there was
    /// nothing to play (fewer than two idle rules).
    pub joint_same: Option<bool>,
    /// The smallest groups found that must not go off together.
    pub groups: Vec<Vec<String>>,
    /// The idle rules that can go off together at the end (all, if the joint arm was the same).
    pub together: Vec<String>,
    /// Every set tried, in order, and whether it was the same island.
    pub tried: Vec<(Vec<String>, bool)>,
}

/// Asks of sets of rules (by their place in the idle list) whether turning
/// them off together is the same island, a batch at a time, remembering what
/// it has asked.
struct Searcher<'a> {
    names: &'a [String],
    test: &'a mut dyn FnMut(&[Vec<String>]) -> Vec<bool>,
    known: HashMap<Vec<usize>, bool>,
    tried: Vec<(Vec<String>, bool)>,
}

fn union(a: &[usize], b: &[usize]) -> Vec<usize> {
    let mut u: Vec<usize> = a.iter().chain(b).copied().collect();
    u.sort();
    u.dedup();
    u
}

impl Searcher<'_> {
    fn same(&mut self, sets: &[Vec<usize>]) -> Vec<bool> {
        let ask: Vec<Vec<usize>> = {
            let mut v: Vec<Vec<usize>> = sets.iter().filter(|s| !self.known.contains_key(*s)).cloned().collect();
            v.dedup();
            v
        };
        if !ask.is_empty() {
            let named: Vec<Vec<String>> = ask.iter().map(|s| s.iter().map(|&k| self.names[k].clone()).collect()).collect();
            let said = (self.test)(&named);
            for ((s, n), v) in ask.into_iter().zip(named).zip(said) {
                self.known.insert(s, v);
                self.tried.push((n, v));
            }
        }
        sets.iter().map(|s| self.known[s]).collect()
    }

    /// A smallest subset of `s` that is not the same island, given that `s` is not.
    fn minimal(&mut self, s: &[usize]) -> Vec<usize> {
        if s.len() <= 1 {
            return s.to_vec();
        }
        let (a, b) = s.split_at(s.len() / 2);
        let r = self.same(&[a.to_vec(), b.to_vec()]);
        if !r[0] {
            return self.minimal(a);
        }
        if !r[1] {
            return self.minimal(b);
        }
        // each half is the same island alone: the clash spans the two
        let a2 = self.reduce(a, b);
        let b2 = self.reduce(b, &a2);
        union(&a2, &b2)
    }

    /// A smallest part of `x` that, off with all of `fixed`, is not the same
    /// island, given that `x` and `fixed` together are not and `fixed` alone is.
    fn reduce(&mut self, x: &[usize], fixed: &[usize]) -> Vec<usize> {
        if x.len() <= 1 {
            return x.to_vec();
        }
        let (x1, x2) = x.split_at(x.len() / 2);
        let r = self.same(&[union(x1, fixed), union(x2, fixed)]);
        if !r[0] {
            return self.reduce(x1, fixed);
        }
        if !r[1] {
            return self.reduce(x2, fixed);
        }
        let x1r = self.reduce(x1, &union(fixed, x2));
        let x2r = self.reduce(x2, &union(fixed, &x1r));
        union(&x1r, &x2r)
    }
}

/// The second pass over the idle rules. `test` is asked of a batch of sets of
/// rules whether each, off together, is the same island (the command plays an
/// arm for each; a test asks a made-up island). Each idle rule is the same
/// island alone, as the first pass found.
pub fn search(idle: &[String], test: &mut dyn FnMut(&[Vec<String>]) -> Vec<bool>) -> Second {
    let mut out = Second { idle: idle.to_vec(), ..Second::default() };
    if idle.len() < 2 {
        out.together = idle.to_vec();
        return out;
    }
    let mut s = Searcher { names: idle, test, known: HashMap::new(), tried: Vec::new() };
    for k in 0..idle.len() {
        s.known.insert(vec![k], true);
    }
    let mut left: Vec<usize> = (0..idle.len()).collect();
    // each round finds one group and keeps one of it on, so there are fewer rounds than rules
    for _ in 0..idle.len() {
        if left.len() < 2 || s.same(&[left.clone()])[0] {
            break;
        }
        let g = s.minimal(&left);
        let keep = *g.last().expect("a group is never empty");
        out.groups.push(g.iter().map(|&k| idle[k].clone()).collect());
        left.retain(|&k| k != keep);
    }
    out.joint_same = s.known.get(&(0..idle.len()).collect::<Vec<_>>()).copied();
    out.together = left.iter().map(|&k| idle[k].clone()).collect();
    out.tried = s.tried;
    out
}

// ---------------------------------------------------------------- the table and the page

/// "1 island", "8 islands".
fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn fmt_size(s: f64) -> String {
    if s.is_infinite() { "from nothing".to_string() } else { format!("{s:.1} spreads") }
}

/// A fact apart as the page names it: `pop_1900 735,871 to 612,004 (2.3 spreads)`.
pub fn fmt_apart(a: &Apart) -> String {
    format!("{} {} to {} ({})", a.name, sweep::fmt_fact(a.kind, a.control), sweep::fmt_fact(a.kind, a.arm), fmt_size(a.size))
}

/// The same, for the page: the fact's name as code.
fn md_apart(a: &Apart) -> String {
    format!("`{}` {} to {} ({})", a.name, sweep::fmt_fact(a.kind, a.control), sweep::fmt_fact(a.kind, a.arm), fmt_size(a.size))
}

fn md_largest(r: &Reading) -> String {
    r.facts.apart.iter().take(LARGEST).map(md_apart).collect::<Vec<_>>().join("; ")
}

/// What one arm of the second pass did, in a line for the page.
fn md_arm(r: &Reading) -> String {
    match role(r) {
        Role::LeanedOn => format!("runs fail ({})", r.failed.iter().map(|(s, w)| format!("island {s}: {w}")).chain(r.refused.clone()).collect::<Vec<_>>().join("; ")),
        Role::HoldsUp => format!(
            "the century does not stand ({}); {} apart{}",
            held_up(r).iter().map(|(f, v)| format!("`{f}` {}", v.as_str())).collect::<Vec<_>>().join(", "),
            r.facts.apart.len(),
            if r.facts.apart.is_empty() { String::new() } else { format!(", the largest {}", md_largest(r)) }
        ),
        Role::Shapes => format!("{} apart, the largest {}", r.facts.apart.len(), md_largest(r)),
        Role::Idle => "the same island".to_string(),
    }
}

/// The second pass, for the page.
fn second_md(t: &Table) -> String {
    let Some(s) = t.second.as_ref() else { return "The second pass was not played (`--no-second`).\n".to_string() };
    let find = |set: &[String]| t.second_readings.iter().find(|r| r.off.len() == set.len() && set.iter().all(|n| r.off.contains(n)));
    let mut out = String::new();
    match s.joint_same {
        None if s.idle.is_empty() => out.push_str("**The second pass**: no rule is idle, so there is nothing to turn off together.\n"),
        None => {
            let _ = writeln!(out, "**The second pass**: one rule is idle, `{}`, so there is nothing to turn off together.", s.idle.join(""));
        }
        Some(true) => {
            let _ = writeln!(out, "**The second pass**: all {} idle rules turned off together are the same island. That island is the same island with fewer rules.", s.idle.len());
        }
        Some(false) => {
            let joint = find(&s.idle).map_or(String::new(), |r| format!(" With all of them off: {}.", md_arm(r)));
            let _ = writeln!(out, "**The second pass**: the {} idle rules turned off together are not the same island.{joint} The smallest groups that must not go off together, each idle alone:\n", s.idle.len());
            for g in &s.groups {
                let what = find(g).map_or(String::new(), |r| format!(": with {} off, {}", if g.len() == 2 { "both" } else { "all of them" }, md_arm(r)));
                let _ = writeln!(out, "- {}{what}.", g.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(" with "));
            }
            let kept: Vec<String> = s.idle.iter().filter(|n| !s.together.contains(n)).map(|n| format!("`{n}`")).collect();
            let _ = writeln!(
                out,
                "\nWith {} kept on, the other {} go off together and the island is the same: {}.",
                kept.join(" and "),
                s.together.len(),
                s.together.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ")
            );
        }
    }
    // a table read again (--again) with a reading that sorts the rules otherwise
    let now: Vec<&str> = t.readings.iter().filter(|r| role(r) == Role::Idle).map(|r| r.label.as_str()).collect();
    let missing: Vec<String> = now.iter().filter(|n| !s.idle.iter().any(|m| m == *n)).map(|n| format!("`{n}`")).collect();
    let no_longer: Vec<String> = s.idle.iter().filter(|m| !now.contains(&m.as_str())).map(|n| format!("`{n}`")).collect();
    if !missing.is_empty() {
        let _ = writeln!(
            out,
            "\n{} {} idle as this page reads the table, and {} not among the rules the second pass turned off together, since the pass was played with the reading of its day; the next table puts {} there.",
            missing.join(" and "),
            if missing.len() == 1 { "is" } else { "are" },
            if missing.len() == 1 { "was" } else { "were" },
            if missing.len() == 1 { "it" } else { "them" }
        );
    }
    if !no_longer.is_empty() {
        let _ = writeln!(out, "\n{} {} in the second pass and {} not idle as this page reads the table.", no_longer.join(" and "), if no_longer.len() == 1 { "was" } else { "were" }, if no_longer.len() == 1 { "is" } else { "are" });
    }
    if !s.tried.is_empty() {
        let _ = writeln!(out, "\nThe arms the second pass played, in order:\n");
        for (set, same) in &s.tried {
            let _ = writeln!(out, "- {} off: {}", plural(set.len(), "rule", "rules"), if *same { "the same island" } else { "not the same island" });
            let _ = writeln!(out, "  ({})", set.join(", "));
        }
    }
    out
}

fn fmt_growth(k: usize, v: Option<f64>) -> String {
    match v {
        None => "-".to_string(),
        Some(x) if k == 0 => format!("{x:.2}%"),
        Some(x) if k == 1 => format!("{x:.2}"),
        Some(x) => format!("{x:+.3}"),
    }
}

fn met_word(m: Option<bool>) -> &'static str {
    match m {
        Some(true) => "met",
        Some(false) => "missed",
        None => "-",
    }
}

fn time_against(r: &Reading, control: &Reading) -> String {
    match (r.cpu.or(r.clock), control.cpu.or(control.clock)) {
        (Some(a), Some(c)) if c > 0.0 => format!("{:.0}%", 100.0 * a / c),
        _ => "-".to_string(),
    }
}

/// Everything one table needs.
#[derive(Clone, Debug)]
pub struct Table {
    pub control: Reading,
    pub readings: Vec<Reading>,
    pub second: Option<Second>,
    /// The arms the second pass played, read against the control.
    pub second_readings: Vec<Reading>,
    pub left_out: Vec<LeftOut>,
    pub found: usize,
    pub seeds: Vec<u64>,
    pub families: usize,
    pub years: (i64, i64),
    pub runs: usize,
    pub failed: usize,
    pub jobs: usize,
    pub seconds_first: f64,
    pub seconds_second: f64,
    pub command: String,
    /// The command as the page shows it, with no path of this machine.
    pub shown: String,
    /// When the table was read again (`--again`): the day and the code it was read with.
    pub again: Option<(String, Option<String>)>,
    /// Rules this code finds on that the table, played earlier, did not ask.
    pub since: Vec<String>,
}

/// The command line as the page shows it: `--out`'s directory as DIR and an
/// absolute `--page` as FILE, so that no path of the machine it ran on is kept.
pub fn shown(args: &[String]) -> String {
    let mut out = vec!["isostate-core ablate".to_string()];
    let mut it = args.iter();
    while let Some(a) = it.next() {
        out.push(a.clone());
        match a.as_str() {
            "--out" => {
                it.next();
                out.push("DIR".to_string());
            }
            "--page" => {
                if let Some(p) = it.next() {
                    let absolute = Path::new(p).is_absolute() || p.starts_with('/') || p.contains(':');
                    out.push(if absolute { "FILE".to_string() } else { p.replace('\\', "/") });
                }
            }
            _ => {}
        }
    }
    out.join(" ")
}

fn standing_cell(r: &Reading) -> String {
    let h = held_up(r);
    let gone = unkept(r);
    let mut s = if r.standing.is_empty() {
        "-".to_string()
    } else if h.is_empty() {
        format!("all {} inside", r.standing.len() - gone.len())
    } else {
        h.iter().map(|(f, v)| format!("{f} {}", v.as_str())).collect::<Vec<_>>().join(", ")
    };
    if !gone.is_empty() {
        let _ = write!(s, "; not kept: {}", gone.join(", "));
    }
    s
}

/// A rule's kind as the table names it: "shapes the island (faint)" where
/// no more stand apart than chance often gives.
fn kind_word(r: &Reading) -> String {
    if faint(r) { format!("{} (faint)", role(r).as_str()) } else { role(r).as_str().to_string() }
}

/// The text the command prints and writes to ablate.txt.
pub fn text(t: &Table) -> String {
    let seeds: Vec<String> = t.seeds.iter().map(|s| s.to_string()).collect();
    let mut out = format!(
        "The core of the island: {} of the {} rules on under the game's rules, each turned off from {} alone, against the control;\n\
         islands {}, {} families, {} to {}; {} runs, {} failed\n\
         apart: every island of the arm above, or below, every island of the control ({STAR_RUNS} or more a side), of the facts both keep\n\
         outside Speed and News; size: the change of the means over the control's spread from island to island;\n\
         chance: how many would stand apart by chance alone among the facts that differ (faint: no more than chance gives one time in twenty)\n\n",
        t.readings.len(),
        t.found,
        t.years.0,
        seeds.join(" "),
        t.families,
        t.years.0,
        t.years.1,
        t.runs,
        t.failed,
    );
    let head: Vec<String> = ["", "kind", "apart", "chance", "differ", "gone", "time", "the twelve"].iter().map(|s| s.to_string()).collect();
    let sides = [Side::Left, Side::Left, Side::Right, Side::Right, Side::Right, Side::Right, Side::Right, Side::Left];
    let mut rows: Vec<(Option<&'static str>, Vec<String>)> = Vec::new();
    for role_ in Role::ALL {
        let these: Vec<&Reading> = t.readings.iter().filter(|r| role(r) == role_).collect();
        if these.is_empty() {
            continue;
        }
        rows.push((Some(role_.as_str()), Vec::new()));
        for r in these {
            rows.push((
                None,
                vec![
                    r.label.clone(),
                    kind_word(r),
                    r.facts.apart.len().to_string(),
                    format!("{:.2}", chance(r)),
                    format!("{}/{}", r.facts.differ, r.facts.both),
                    r.facts.gone.len().to_string(),
                    time_against(r, &t.control),
                    standing_cell(r),
                ],
            ));
        }
    }
    out.push_str(&sweep::layout(&head, &sides, &rows));
    out.push_str("\nthe largest facts apart, rule by rule\n");
    for r in &t.readings {
        if r.facts.apart.is_empty() {
            continue;
        }
        let list: Vec<String> = r.facts.apart.iter().take(LARGEST).map(fmt_apart).collect();
        let _ = writeln!(out, "  {}: {}", r.label, list.join("; "));
    }
    out.push_str("\nthe century after 1900 (wage growth 1900-1970 a year; wage 2050 over 1970; hands on the land 2050 less 1900)\n");
    let g = |r: &Reading| (0..3).map(|k| format!("{} {}", fmt_growth(k, r.growth[k].0), met_word(r.growth[k].1))).collect::<Vec<_>>().join("  ");
    let _ = writeln!(out, "  control: {}", g(&t.control));
    for r in &t.readings {
        if r.growth != t.control.growth {
            let _ = writeln!(out, "  {}: {}", r.label, g(r));
        }
    }
    for r in &t.readings {
        if let Some(why) = &r.refused {
            let _ = writeln!(out, "\nREFUSED: {}: {why}", r.label);
        }
        for (seed, why) in &r.failed {
            let _ = writeln!(out, "\nFAILED: island {seed}, {}: {why}", r.label);
        }
    }
    out.push('\n');
    out.push_str(&second_text(t.second.as_ref()));
    if !t.left_out.is_empty() {
        out.push_str("\nleft out\n");
        for l in &t.left_out {
            let _ = writeln!(out, "  {}: {}", l.name, l.why);
        }
    }
    let _ = writeln!(
        out,
        "\n{:.0} s on the clock for the first pass and {:.0} s for the second, {} at a time; a run of the control took {} s on the clock",
        t.seconds_first,
        t.seconds_second,
        t.jobs,
        t.control.clock.map_or("-".to_string(), |c| format!("{c:.0}"))
    );
    out
}

fn second_text(s: Option<&Second>) -> String {
    let Some(s) = s else { return "the second pass was not played (--no-second)\n".to_string() };
    let mut out = String::new();
    match s.joint_same {
        None if s.idle.is_empty() => out.push_str("the second pass: no rule is idle, so there is nothing to turn off together\n"),
        None => {
            let _ = writeln!(out, "the second pass: one rule is idle ({}), so there is nothing to turn off together", s.idle.join(""));
        }
        Some(true) => {
            let _ = writeln!(out, "the second pass: all {} idle rules off together are the same island: {}", s.idle.len(), s.idle.join(", "));
        }
        Some(false) => {
            let _ = writeln!(out, "the second pass: the {} idle rules off together are NOT the same island; the groups that must not go off together:", s.idle.len());
            for g in &s.groups {
                let _ = writeln!(out, "  {}", g.join(" with "));
            }
            let _ = writeln!(out, "  these can go off together: {}", if s.together.is_empty() { "none".to_string() } else { s.together.join(", ") });
        }
    }
    if !s.tried.is_empty() {
        let _ = writeln!(out, "  {} played:", plural(s.tried.len(), "arm", "arms"));
        for (set, same) in &s.tried {
            let _ = writeln!(out, "    {}: {}", set.join(" + "), if *same { "the same island" } else { "not the same island" });
        }
    }
    out
}

/// The colour of each kind in the chart (the reference palette's first four,
/// checked for colour blindness; each bar is labelled too).
fn colour(r: Role) -> &'static str {
    match r {
        Role::HoldsUp => "#eb6834",
        Role::Shapes => "#2a78d6",
        Role::Idle => "#1baf7a",
        Role::LeanedOn => "#eda100",
    }
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The chart: a row for each rule, coloured by its kind, with a bar for how
/// many facts stand apart without it.
pub fn chart(t: &Table) -> String {
    let mut rows: Vec<&Reading> = t.readings.iter().collect();
    rows.sort_by(|a, b| role(a).cmp(&role(b)).then(b.facts.apart.len().cmp(&a.facts.apart.len())).then(a.label.cmp(&b.label)));
    let most = rows.iter().map(|r| r.facts.apart.len()).max().unwrap_or(0).max(1);
    let (label_w, bar_w, row_h, top) = (230.0, 420.0, 22.0, 64.0);
    let width = label_w + bar_w + 90.0;
    let height = top + row_h * rows.len() as f64 + 36.0;
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\" font-family=\"Georgia, serif\" font-size=\"13\">\n\
         <rect width=\"100%\" height=\"100%\" fill=\"#fcfcfb\"/>\n\
         <text x=\"16\" y=\"24\" font-size=\"15\" fill=\"#0b0b0b\">Facts that stand apart with each rule off from 1750</text>\n"
    );
    // the legend, one swatch and word per kind present
    let mut x = 16.0;
    for role_ in Role::ALL {
        let n = rows.iter().filter(|r| role(r) == role_).count();
        if n == 0 {
            continue;
        }
        let word = format!("{} ({n})", role_.as_str());
        let _ = writeln!(s, "<rect x=\"{x}\" y=\"36\" width=\"12\" height=\"12\" rx=\"2\" fill=\"{}\"/><text x=\"{}\" y=\"46\" fill=\"#52514e\">{}</text>", colour(role_), x + 16.0, xml(&word));
        x += 16.0 + 7.2 * word.len() as f64 + 18.0;
    }
    for (k, r) in rows.iter().enumerate() {
        let y = top + row_h * k as f64;
        let n = r.facts.apart.len();
        let w = (bar_w * n as f64 / most as f64).max(if n > 0 { 2.0 } else { 0.0 });
        let _ = writeln!(s, "<text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"#0b0b0b\">{}</text>", label_w - 8.0, y + 15.0, xml(&r.label));
        // an idle rule or one leaned on has a mark at the axis so its colour shows
        let _ = writeln!(s, "<rect x=\"{label_w}\" y=\"{}\" width=\"{}\" height=\"14\" rx=\"2\" fill=\"{}\"/>", y + 4.0, w.max(4.0), colour(role(r)));
        let what = if role(r) == Role::LeanedOn {
            "fails".to_string()
        } else if faint(r) {
            format!("{n}, faint: chance gives {:.1}", chance(r))
        } else {
            n.to_string()
        };
        let _ = writeln!(s, "<text x=\"{}\" y=\"{}\" fill=\"#52514e\">{what}</text>", label_w + w.max(4.0) + 6.0, y + 15.0);
    }
    let base = top + row_h * rows.len() as f64 + 4.0;
    let _ = writeln!(s, "<line x1=\"{label_w}\" y1=\"{}\" x2=\"{label_w}\" y2=\"{base}\" stroke=\"#c3c2b7\"/>", top);
    let seeds = t.seeds.len();
    let _ = writeln!(
        s,
        "<text x=\"16\" y=\"{}\" fill=\"#52514e\" font-size=\"12\">bar: facts apart on every one of {seeds} islands (of the facts both keep, Speed and News aside), {} to {}</text>",
        base + 22.0,
        t.years.0,
        t.years.1
    );
    s.push_str("</svg>\n");
    s
}

/// The page, docs/core.md: nothing in it is written by hand.
pub fn page(t: &Table, date: &str, code: Option<&str>) -> String {
    let mut p = String::new();
    let seeds: Vec<String> = t.seeds.iter().map(|s| s.to_string()).collect();
    let by = |r: Role| -> Vec<&Reading> { t.readings.iter().filter(|x| role(x) == r).collect() };
    let _ = writeln!(p, "# The core of the island\n");
    let _ = writeln!(
        p,
        "Written by `isostate-core ablate`, played on {date} (UTC) from the code at {}{}. Nothing on this page is written by hand: to make it again, run\n",
        code.map_or("an unknown commit".to_string(), |c| format!("`{c}`")),
        t.again.as_ref().map_or(String::new(), |(d, c)| format!(
            ", and read again from its files on {d} with the code at {} (`--again`, which plays nothing)",
            c.as_deref().map_or("an unknown commit".to_string(), |c| format!("`{c}`"))
        ))
    );
    let _ = writeln!(p, "```text\nisostate-core ablate --out DIR --page docs/core.md\n```\n");
    let _ = writeln!(p, "This one was made by `{}`.\n", t.shown);
    let _ = writeln!(
        p,
        "Every rule the game switches on is turned off from {}, one at a time, and the island is played again: islands {} under the game's rules with {} families, {} to {}. Each such arm is set against the control, the islands as they are, island by island. {} runs were played, {} failed.\n",
        t.years.0,
        seeds.join(", "),
        t.families,
        t.years.0,
        t.years.1,
        t.runs,
        t.failed
    );
    let counts: Vec<String> = Role::ALL.iter().map(|&r| format!("{} {}", by(r).len(), r.plural())).collect();
    let _ = writeln!(p, "Of the {} rules asked, {}.\n", t.readings.len(), counts.join(", "));
    let n = t.seeds.len();
    if n >= STAR_RUNS {
        // two sets of n islands drawn from one spread lie wholly apart with chance 2 / C(2n, n)
        let ways = (0..n).fold(1.0f64, |c, k| c * (2 * n - k) as f64 / (k + 1) as f64);
        let stirred: Vec<f64> = t.readings.iter().filter(|r| r.facts.differ > 0).map(|r| r.facts.differ as f64).collect();
        let typical = if stirred.is_empty() { 0.0 } else { stirred.iter().sum::<f64>() / stirred.len() as f64 };
        let _ = writeln!(
            p,
            "Read the small counts with care. With {n} islands a side, a fact the rule does not move, only stirs (a rule that changes anything sends each island down another path), stands apart by chance once in {} facts; a rule that stirs the island stirs some {} facts, so {:.2} of them would stand apart by chance alone, and more than that where facts move together. A rule with no more facts apart than chance gives one time in twenty is marked *faint*, unless one of its facts is the same on every island of one arm, which chance cannot give.\n",
            sweep::fmt_fact(Kind::Count, (ways / 2.0).round()),
            sweep::fmt_fact(Kind::Count, (typical / 100.0).round() * 100.0),
            typical * 2.0 / ways
        );
    }
    let _ = writeln!(p, "![Facts that stand apart with each rule off](core.svg)\n");

    let _ = writeln!(p, "## Every rule, by kind\n");
    let _ = writeln!(p, "| rule | kind | facts apart | by chance alone | differ, of those both keep | not kept | the twelve out of their band | time of a run |");
    let _ = writeln!(p, "|---|---|---:|---:|---:|---:|---|---:|");
    for r in Role::ALL {
        for x in by(r) {
            let held = held_up(x);
            let mut twelve = if x.standing.is_empty() { "-".to_string() } else if held.is_empty() { "none".to_string() } else { held.iter().map(|(f, _)| format!("`{f}`")).collect::<Vec<_>>().join(", ") };
            let gone = unkept(x);
            if !gone.is_empty() {
                let _ = write!(twelve, " (not kept without it: {})", gone.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", "));
            }
            let _ = writeln!(
                p,
                "| `{}` | {} | {} | {:.2} | {} of {} | {} | {} | {} |",
                x.label,
                kind_word(x),
                x.facts.apart.len(),
                chance(x),
                x.facts.differ,
                x.facts.both,
                x.facts.gone.len(),
                twelve,
                time_against(x, &t.control)
            );
        }
    }
    let _ = writeln!(
        p,
        "\nThe time of a run is the processor's time on a run with the rule off, against the control's ({} s on the clock for a run of the control): under a hundred per cent, the rule costs that much to run. It is taken on a machine shared with other work, whose load changed through the table, so only a time far from a hundred per cent says much.\n",
        t.control.clock.map_or("-".to_string(), |c| format!("{c:.0}"))
    );

    let _ = writeln!(p, "## What each rule that holds up the century holds up\n");
    let held = by(Role::HoldsUp);
    if held.is_empty() {
        let _ = writeln!(p, "None: with any one rule off, the twelve facts of the played century stay inside the control's band.\n");
    }
    for x in held {
        let list: Vec<String> = held_up(x).iter().map(|(f, v)| format!("`{f}` ({})", v.as_str())).collect();
        let _ = writeln!(p, "- **`{}`**: {}.", x.label, list.join(", "));
        if !x.facts.apart.is_empty() {
            let _ = writeln!(p, "  Largest apart: {}.", md_largest(x));
        }
    }
    p.push('\n');

    let _ = writeln!(p, "## The rules that shape the island\n");
    let shapes = by(Role::Shapes);
    if shapes.is_empty() {
        let _ = writeln!(p, "None.\n");
    }
    for x in shapes {
        let _ = writeln!(p, "- **`{}`**, {} apart: {}.", x.label, x.facts.apart.len(), md_largest(x));
    }
    p.push('\n');

    let _ = writeln!(p, "## The idle rules, and the second pass\n");
    let idle = by(Role::Idle);
    if idle.is_empty() {
        let _ = writeln!(p, "No rule is idle.\n");
    }
    for x in &idle {
        let how = if x.facts.differ == 0 && !x.facts.gone.is_empty() {
            format!("no fact both keep differs on any island, and it keeps {} the island does not keep without it: it watches", plural(x.facts.gone.len(), "fact", "facts"))
        } else if x.facts.differ == 0 {
            "no fact differs on any island".to_string()
        } else {
            format!("it stirs the island ({} of the {} facts both keep differ) and moves no fact clear of the spread", x.facts.differ, x.facts.both)
        };
        let _ = writeln!(p, "- **`{}`**: {how}.", x.label);
    }
    if !idle.is_empty() {
        p.push('\n');
    }
    p.push_str(&second_md(t));
    p.push('\n');

    let leaned = by(Role::LeanedOn);
    if !leaned.is_empty() {
        let _ = writeln!(p, "## The rules other rules lean on\n");
        for x in leaned {
            let why = match &x.refused {
                Some(w) => format!("the rules refuse to run without it: {w}"),
                None => format!(
                    "{} failed without it: {}",
                    plural(x.failed.len(), "run", "runs"),
                    x.failed.iter().map(|(s, w)| format!("island {s}, {w}")).collect::<Vec<_>>().join("; ")
                ),
            };
            let _ = writeln!(p, "- **`{}`**: {why}.", x.label);
        }
        p.push('\n');
    }

    let _ = writeln!(p, "## The century after 1900\n");
    let _ = writeln!(p, "What the design asks of the years after 1900, as the standing check reports it (`docs/rules/README.md`, \"The standing check\"); reported, not counted in the kinds. Only the arms that differ from the control are listed.\n");
    let _ = writeln!(p, "| | {} | {} | {} |", GROWTH[0].1, GROWTH[1].1, GROWTH[2].1);
    let _ = writeln!(p, "|---|---:|---:|---:|");
    let row = |name: &str, r: &Reading| format!("| {name} | {} |", (0..3).map(|k| format!("{} ({})", fmt_growth(k, r.growth[k].0), met_word(r.growth[k].1))).collect::<Vec<_>>().join(" | "));
    let _ = writeln!(p, "{}", row("the control", &t.control));
    for x in &t.readings {
        if x.growth != t.control.growth && x.refused.is_none() {
            let _ = writeln!(p, "{}", row(&format!("`{}` off", x.label), x));
        }
    }
    p.push('\n');

    let _ = writeln!(p, "## What was left out, and why\n");
    if t.left_out.is_empty() {
        let _ = writeln!(p, "Nothing.\n");
    }
    for l in &t.left_out {
        let _ = writeln!(p, "- `{}`: {}.", l.name, l.why);
    }
    if t.readings.len() < t.found {
        let _ = writeln!(p, "- {} of the {} rules found on were not asked this time (`--only`).", t.found - t.readings.len(), t.found);
    }
    if !t.since.is_empty() {
        let _ = writeln!(
            p,
            "- {}: switched on under the game's rules since this table was played, so not in it; the next table asks {}.",
            t.since.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", "),
            if t.since.len() == 1 { "it" } else { "them" }
        );
    }
    p.push('\n');

    let _ = writeln!(p, "## How to read it\n");
    let _ = writeln!(
        p,
        "The rules asked are found in the tree's own lists, not in one kept for this page: every true-or-false name the rulebook's walk (`rulebook.rs`) finds on under the game's rules and off under the port's, and every lesser switch of a rule the game switches on. A rule added tomorrow is asked too. An arm turns one of them off from {} with everything else as it is, exactly what `sweep --set NAME=false` plays.\n",
        t.years.0
    );
    let _ = writeln!(
        p,
        "A fact **stands apart** when every island of the arm lies above, or below, every island of the control, as `sweep --compare` stars a fact ({STAR_RUNS} islands or more a side). Only the facts both arms keep are counted, and not those of Speed or News: how long a run took is no fact about the island, and the news is what the chronicle writes about it. A fact's **size** is the change of the means over the control's spread from island to island; \"from nothing\" is a fact that is the same on every island of the control.\n"
    );
    let _ = writeln!(
        p,
        "The kinds, by those numbers alone: **leaned on**, a run fails with the rule off (other rules lean on it); **holds up the century**, one of the standing check's twelve facts of 1750 to 1900 is not inside the control's band (moved, OUTSIDE, or without a value); **shapes the island**, facts stand apart and the twelve stay inside; **idle**, nothing stands apart. An idle rule that keeps facts the island does not keep without it watches the island (the books, the gauges) and is idle by design.\n"
    );
    let _ = writeln!(
        p,
        "Rules cover for each other: two that mend one behaviour each look idle alone. So **the second pass** turns every idle rule off together. If that arm is idle too, the island is the same island with fewer rules. If not, the set is halved until the smallest groups that must not go off together are found; one of each group is kept on, and the rest are tried again.\n"
    );
    let _ = writeln!(p, "Nothing here switches a rule off under the game's rules. What is switched off is decided after the table is read, and the rule's page then says since when and why.");
    p
}

// ---------------------------------------------------------------- the files

/// `git describe` in the directory the command was started from, if git is there.
fn git_describe() -> Option<String> {
    let out = std::process::Command::new("git").args(["describe", "--always", "--dirty"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !text.is_empty()).then_some(text)
}

fn reading_json(r: &Reading) -> Value {
    json!({
        "label": r.label,
        "off": r.off,
        "kind": role(r).as_str(),
        "refused": r.refused,
        "failed": r.failed.iter().map(|(s, w)| json!({"seed": s, "why": w})).collect::<Vec<_>>(),
        "standing": r.standing.iter().map(|(f, v)| json!({"fact": f, "verdict": v.as_str()})).collect::<Vec<_>>(),
        "growth": (0..3).map(|k| json!({"asked": GROWTH[k].0, "mean": r.growth[k].0, "met": r.growth[k].1})).collect::<Vec<_>>(),
        "facts_both": r.facts.both,
        "facts_differ": r.facts.differ,
        "facts_apart": r.facts.apart.iter().map(|a| json!({"fact": a.name, "control": a.control, "arm": a.arm, "size": if a.size.is_finite() { json!(a.size) } else { Value::Null }})).collect::<Vec<_>>(),
        "facts_not_kept": r.facts.gone,
        "facts_new": r.facts.new,
        "cpu_seconds": r.cpu,
        "clock_seconds": r.clock,
    })
}

fn put(dir: &Path, name: &str, body: &str) -> Result<(), String> {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(&path, body).map_err(|e| format!("{}: {e}", path.display()))
}

/// The directory each arm's own files go in.
fn arm_dir(a: usize) -> String {
    if a == 0 { "control".to_string() } else { format!("off-{a}") }
}

fn write_arms(dir: &Path, arms: &[Arm], sweeps: &[Sweep], first: usize, command: &str, code: Option<&str>) -> Result<(), String> {
    for (k, (arm, sw)) in arms.iter().zip(sweeps).enumerate() {
        let sub = arm_dir(first + k);
        put(dir, &format!("{sub}/annual.csv"), &sweep::annual_csv(sw))?;
        put(dir, &format!("{sub}/facts.csv"), &sweep::facts_csv(sw))?;
        let facts = serde_json::to_string_pretty(&sweep::facts_json(sw, command, code)).map_err(|e| e.to_string())? + "\n";
        put(dir, &format!("{sub}/facts.json"), &facts)?;
        put(dir, &format!("{sub}/off.txt"), &format!("{}\n", if arm.off.is_empty() { "nothing (the control)".to_string() } else { arm.off.join("\n") }))?;
    }
    Ok(())
}

// ---------------------------------------------------------------- the command

/// One arm's facts read back from its directory's facts.json: each fact by
/// island, each fact's group and kind, the failed runs, and the mean
/// processor seconds and seconds on the clock of a run.
pub struct ArmFile {
    pub then: Then,
    pub index: Index,
    pub failed: Vec<(u64, String)>,
    pub cpu: Option<f64>,
    pub clock: Option<f64>,
}

fn kind_named(s: &str) -> Kind {
    match s {
        "year" => Kind::Year,
        "count" => Kind::Count,
        "share" => Kind::Share,
        "ratio" => Kind::Ratio,
        _ => Kind::Real,
    }
}

pub fn read_arm_file(sub: &Path) -> Result<ArmFile, String> {
    let path = sub.join("facts.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut index: Index = HashMap::new();
    for f in doc["facts"].as_array().into_iter().flatten() {
        let group = match f["group"].as_str().unwrap_or("") {
            g if g == SPEED => SPEED,
            g if g == NEWS => NEWS,
            _ => "",
        };
        index.insert(f["name"].as_str().unwrap_or("").to_string(), (group, kind_named(f["kind"].as_str().unwrap_or(""))));
    }
    let mut then = Then { label: sub.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned()), ..Then::default() };
    let (mut failed, mut cpu, mut clock) = (Vec::new(), Vec::new(), Vec::new());
    for run in doc["runs"].as_array().into_iter().flatten() {
        let seed = run["seed"].as_u64().unwrap_or(0);
        if run["ok"].as_bool() != Some(true) {
            failed.push((seed, run["error"].as_str().unwrap_or("failed").to_string()));
            continue;
        }
        then.seeds.push(seed);
        clock.push((seed, run["seconds"].as_f64().unwrap_or(f64::NAN)));
        for (name, v) in run["facts"].as_object().into_iter().flatten() {
            if let Some(v) = v.as_f64() {
                then.values.entry(name.clone()).or_default().push((seed, v));
                if name == "cpu_seconds" {
                    cpu.push((seed, v));
                }
            }
        }
    }
    Ok(ArmFile { then, index, failed, cpu: stat(&cpu).map(|s| s.mean), clock: stat(&clock).map(|s| s.mean) })
}

/// `--again DIR`: a finished table read anew from the files it wrote, with
/// this code's reading, and its page written again (in `DIR/again/`, and at
/// `--page`). Nothing is played: the second pass is the one that was played,
/// and a rule this reading calls idle that the pass did not have is named.
pub fn again(o: &Options, print: bool) -> Result<(), String> {
    let dir = &o.out;
    let path = dir.join("ablate.json");
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).map_err(|e| format!("--again: {}: {e}", path.display()))?).map_err(|e| format!("--again: {}: {e}", path.display()))?;
    let played_args: Vec<String> = doc["command"].as_str().unwrap_or("").split_whitespace().skip(2).map(String::from).collect();
    let played = parse_args(&played_args)?;
    let rules = doc["rules"].as_array().cloned().unwrap_or_default();
    let control_file = read_arm_file(&dir.join(arm_dir(0)))?;
    let mut all = Vec::new();
    let mut k = 1;
    while dir.join(arm_dir(k)).join("off.txt").exists() {
        let sub = dir.join(arm_dir(k));
        let off_text = std::fs::read_to_string(sub.join("off.txt")).map_err(|e| format!("{}: {e}", sub.display()))?;
        let off: Vec<String> = off_text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
        let file = read_arm_file(&sub)?;
        let refused = rules.get(k - 1).and_then(|r| r["refused"].as_str()).map(String::from);
        let mut index = control_file.index.clone();
        for (n, g) in &file.index {
            index.entry(n.clone()).or_insert(*g);
        }
        let mut r = read_thens(&control_file.then, &file.then, &index, refused, file.failed.clone());
        (r.label, r.off, r.cpu, r.clock) = (off.join(" + "), off, file.cpu, file.clock);
        all.push(r);
        k += 1;
    }
    let first = rules.len().min(all.len());
    let second_readings = all.split_off(first);
    let mut control = read_thens(&control_file.then, &control_file.then, &control_file.index, None, control_file.failed.clone());
    (control.label, control.cpu, control.clock) = ("control".to_string(), control_file.cpu, control_file.clock);
    let strings = |v: &Value| -> Vec<String> { v.as_array().into_iter().flatten().filter_map(|x| x.as_str().map(String::from)).collect() };
    let second = doc["second"].as_object().map(|s| Second {
        idle: strings(&s["idle"]),
        joint_same: s["joint_same"].as_bool(),
        groups: s["groups"].as_array().into_iter().flatten().map(strings).collect(),
        together: strings(&s["together"]),
        tried: s["tried"].as_array().into_iter().flatten().map(|t| (strings(&t["off"]), t["same"].as_bool().unwrap_or(false))).collect(),
    });
    let mut runs = control_file.then.seeds.len() + control_file.failed.len();
    let mut failed = control_file.failed.len();
    for r in all.iter().chain(&second_readings) {
        runs += r.islands + r.failed.len();
        failed += r.failed.len();
    }
    let found = if played.only.is_some() { asked(&game_params(), &default_params()).0.len() } else { all.len() };
    // the rules switched on since the table was played: named, and no longer listed as left out
    let since: Vec<String> = {
        let played_names: Vec<&str> = rules.iter().filter_map(|r| r["label"].as_str()).collect();
        let now = asked(&game_params(), &default_params()).0;
        if played.only.is_some() { Vec::new() } else { now.into_iter().filter(|n| !played_names.contains(&n.as_str())).collect() }
    };
    let mut left_out: Vec<LeftOut> = doc["left_out"].as_array().into_iter().flatten().map(|l| LeftOut { name: l["name"].as_str().unwrap_or("").to_string(), why: l["why"].as_str().unwrap_or("").to_string() }).collect();
    left_out.retain(|l| !since.contains(&l.name));
    let table = Table {
        control,
        readings: all,
        second,
        second_readings,
        left_out,
        found,
        seeds: played.seeds.clone(),
        families: played.families,
        years: (Rules::Game.content().start_year, played.end_year),
        runs,
        failed,
        jobs: doc["jobs"].as_u64().unwrap_or(0) as usize,
        seconds_first: doc["seconds_first"].as_f64().unwrap_or(0.0),
        seconds_second: doc["seconds_second"].as_f64().unwrap_or(0.0),
        command: played.command.clone(),
        shown: shown(&played_args),
        again: Some((crate::record::today(), git_describe())),
        since,
    };
    let txt = text(&table);
    let md = page(&table, doc["date"].as_str().unwrap_or("an unknown day"), doc["code"].as_str());
    let svg = chart(&table);
    put(dir, "again/ablate.txt", &txt)?;
    put(dir, "again/core.md", &md)?;
    put(dir, "again/core.svg", &svg)?;
    if let Some(p) = &o.page {
        std::fs::write(p, &md).map_err(|e| format!("--page {}: {e}", p.display()))?;
        let svg_path = p.with_file_name("core.svg");
        std::fs::write(&svg_path, &svg).map_err(|e| format!("--page: {}: {e}", svg_path.display()))?;
    }
    if print {
        let _ = std::io::Write::write_all(&mut std::io::stdout(), txt.as_bytes());
    }
    Ok(())
}

/// The `ablate` subcommand: check, play, read, write the files and the page.
pub fn main_with_args(args: &[String]) -> Result<(), String> {
    command(args, true)
}

/// The subcommand with or without the table on stdout (it is in ablate.txt
/// either way; the tests leave it off).
pub fn command(args: &[String], print: bool) -> Result<(), String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let o = parse_args(args)?;
    if o.again {
        return again(&o, print);
    }
    let plan = plan(&o)?;
    std::fs::create_dir_all(&o.out).map_err(|e| format!("--out {}: {e}", o.out.display()))?;
    let code = git_describe();
    let date = crate::record::today();

    // the first pass: the control and one arm for each rule
    let mut arms = vec![Arm { label: "control".to_string(), off: Vec::new(), params: Ok(plan.base.clone()) }];
    for n in &plan.asked {
        arms.push(Arm { label: n.clone(), off: vec![n.clone()], params: off(&plan.base, std::slice::from_ref(n)) });
    }
    let (sweeps, jobs, seconds_first) = play(&arms, &o.seeds, &plan.content, o.jobs, o.quiet);
    write_arms(&o.out, &arms, &sweeps, 0, &o.command, code.as_deref())?;
    let control = read(&sweeps[0], &arms[0], &sweeps[0]);
    let readings: Vec<Reading> = arms.iter().zip(&sweeps).skip(1).map(|(a, sw)| read(&sweeps[0], a, sw)).collect();
    let mut runs: usize = sweeps.iter().map(|sw| sw.runs.len()).sum();
    let mut failed: usize = sweeps.iter().map(|sw| sw.runs.iter().filter(|r| r.outcome.is_err()).count()).sum();

    // the second pass: the idle rules off together, and halved if they are not the same island
    let t1 = Instant::now();
    let mut more: Vec<Reading> = Vec::new();
    let second = if o.second {
        let idle: Vec<String> = readings.iter().filter(|r| same_island(r)).map(|r| r.label.clone()).collect();
        let mut next_dir = arms.len();
        let mut errors: Vec<String> = Vec::new();
        let mut test = |sets: &[Vec<String>]| -> Vec<bool> {
            let batch: Vec<Arm> = sets.iter().map(|s| Arm { label: s.join(" + "), off: s.clone(), params: off(&plan.base, s) }).collect();
            let (sw, _, _) = play(&batch, &o.seeds, &plan.content, o.jobs, o.quiet);
            if let Err(e) = write_arms(&o.out, &batch, &sw, next_dir, &o.command, code.as_deref()) {
                errors.push(e);
            }
            next_dir += batch.len();
            runs += sw.iter().map(|s| s.runs.len()).sum::<usize>();
            failed += sw.iter().map(|s| s.runs.iter().filter(|r| r.outcome.is_err()).count()).sum::<usize>();
            batch
                .iter()
                .zip(&sw)
                .map(|(a, s)| {
                    let r = read(&sweeps[0], a, s);
                    let same = same_island(&r);
                    more.push(r);
                    same
                })
                .collect()
        };
        let s = search(&idle, &mut test);
        if let Some(e) = errors.first() {
            return Err(e.clone());
        }
        put(&o.out, "second.json", &(serde_json::to_string_pretty(&Value::Array(more.iter().map(reading_json).collect())).map_err(|e| e.to_string())? + "\n"))?;
        Some(s)
    } else {
        None
    };
    let seconds_second = if o.second { t1.elapsed().as_secs_f64() } else { 0.0 };

    let table = Table {
        control,
        readings,
        second,
        second_readings: more,
        left_out: plan.left_out.clone(),
        found: plan.found.len(),
        seeds: o.seeds.clone(),
        families: o.families,
        years: (plan.content.start_year, o.end_year),
        runs,
        failed,
        jobs,
        seconds_first,
        seconds_second,
        command: o.command.clone(),
        shown: shown(args),
        again: None,
        since: Vec::new(),
    };
    let txt = text(&table);
    let md = page(&table, &date, code.as_deref());
    let svg = chart(&table);
    put(&o.out, "ablate.txt", &txt)?;
    put(&o.out, "core.md", &md)?;
    put(&o.out, "core.svg", &svg)?;
    let all = json!({
        "command": o.command,
        "code": code,
        "date": date,
        "control": reading_json(&table.control),
        "rules": table.readings.iter().map(reading_json).collect::<Vec<_>>(),
        "left_out": table.left_out.iter().map(|l| json!({"name": l.name, "why": l.why})).collect::<Vec<_>>(),
        "second": table.second.as_ref().map(|s| json!({"idle": s.idle, "joint_same": s.joint_same, "groups": s.groups, "together": s.together, "tried": s.tried.iter().map(|(set, same)| json!({"off": set, "same": same})).collect::<Vec<_>>()})),
        "seconds_first": table.seconds_first,
        "seconds_second": table.seconds_second,
        "jobs": table.jobs,
    });
    put(&o.out, "ablate.json", &(serde_json::to_string_pretty(&all).map_err(|e| e.to_string())? + "\n"))?;
    let readme = format!(
        "The core of the island (isostate-core ablate), {date} (UTC).\n\ncommand:  {}\ncode:     {}\n\nFiles\n\
         ablate.txt   the table the command printed\n\
         core.md      the page, as docs/core.md holds it; core.svg its chart\n\
         ablate.json  every arm's numbers; second.json the second pass's arms\n\
         control/, off-1/ ...  each arm as a sweep of one setting (facts.json, facts.csv, annual.csv), and off.txt,\n\
         \x20            the rules it turns off; the arms of the second pass follow the first's\n",
        o.command,
        code.as_deref().unwrap_or("unknown")
    );
    put(&o.out, "README.txt", &readme)?;
    if let Some(p) = &o.page {
        std::fs::write(p, &md).map_err(|e| format!("--page {}: {e}", p.display()))?;
        let svg_path = p.with_file_name("core.svg");
        std::fs::write(&svg_path, &svg).map_err(|e| format!("--page: {}: {e}", svg_path.display()))?;
    }
    if print {
        let _ = std::io::Write::write_all(&mut std::io::stdout(), txt.as_bytes());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn then(label: &str, facts: &[(&str, [f64; 4])]) -> Then {
        let mut t = Then { label: label.to_string(), seeds: vec![1, 2, 3, 4], ..Then::default() };
        for (name, v) in facts {
            t.values.insert(name.to_string(), (1..=4u64).zip(v.iter().copied()).collect());
        }
        t
    }

    fn index() -> Index {
        [("pop_1900", ("People and towns", Kind::Count)), ("towns_1900", ("People and towns", Kind::Count)), ("gauge_floor_1900", ("Work and wages", Kind::Share)), ("news_total", (NEWS, Kind::Count)), ("seconds", (SPEED, Kind::Real))]
            .iter()
            .map(|(n, g)| (n.to_string(), *g))
            .collect()
    }

    /// A made-up rule that changes nothing the island keeps, and only watches:
    /// with it off its gauge is not kept, the news and the seconds differ, and
    /// nothing else does. It is idle, and the page would say it watches.
    #[test]
    fn a_rule_that_changes_nothing_is_idle() {
        let control = then(
            "control",
            &[("pop_1900", [7e5, 7.2e5, 7.4e5, 7.6e5]), ("towns_1900", [14.0, 17.0, 18.0, 20.0]), ("gauge_floor_1900", [0.1; 4]), ("news_total", [50.0; 4]), ("seconds", [80.0; 4])],
        );
        let arm = then("off", &[("pop_1900", [7e5, 7.2e5, 7.4e5, 7.6e5]), ("towns_1900", [14.0, 17.0, 18.0, 20.0]), ("news_total", [10.0; 4]), ("seconds", [60.0; 4])]);
        let f = read_facts(&control, &arm, &index());
        assert_eq!((f.both, f.differ, f.apart.len()), (2, 0, 0), "{f:?}");
        assert_eq!(f.gone, ["gauge_floor_1900"]);
        let r = Reading {
            label: "made_up.enabled".into(),
            off: vec!["made_up.enabled".into()],
            refused: None,
            failed: Vec::new(),
            standing: FACTS.iter().map(|&f| (f, Verdict::Inside)).collect(),
            growth: [(None, None); 3],
            facts: f,
            cpu: Some(60.0),
            clock: Some(60.0),
            islands: 4,
        };
        assert_eq!(role(&r), Role::Idle);
        // a fact of the twelve that the arm does not keep (the rule measures it) holds nothing up
        let mut watches = r.clone();
        watches.standing[9].1 = Verdict::Missing;
        assert_eq!((role(&watches), unkept(&watches)), (Role::Idle, vec!["income_share_work_1850"]));
        // a fact that moves inside the spread keeps it idle; one that stands apart makes it shape the island
        let inside = then("off", &[("pop_1900", [7.1e5, 7.2e5, 7.4e5, 7.5e5]), ("towns_1900", [14.0, 17.0, 18.0, 20.0])]);
        let f2 = read_facts(&control, &inside, &index());
        assert_eq!((f2.differ, f2.apart.len()), (1, 0));
        let apart = then("off", &[("pop_1900", [7e5, 7.2e5, 7.4e5, 7.6e5]), ("towns_1900", [21.0, 22.0, 23.0, 25.0])]);
        let f3 = read_facts(&control, &apart, &index());
        assert_eq!(f3.apart.len(), 1);
        // 22.75 against 17.25, over the control's spread of 6: 0.92 spreads
        assert!((f3.apart[0].size - 5.5 / 6.0).abs() < 1e-12, "{:?}", f3.apart);
        let one = Reading { facts: f3, ..r.clone() };
        assert_eq!(role(&one), Role::Shapes);
        // one fact apart among a thousand stirred, on eight islands a side, is what chance gives: faint
        let mut stirred = one.clone();
        (stirred.islands, stirred.facts.differ) = (8, 1000);
        assert!((chance(&stirred) - 1000.0 * 2.0 / 12870.0).abs() < 1e-12 && faint(&stirred));
        // four apart among them is not, nor is one whose control is the same on every island
        stirred.facts.apart = vec![stirred.facts.apart[0].clone(); 4];
        assert!(chance_of(4, chance(&stirred)) < 0.05 && !faint(&stirred));
        stirred.facts.apart.truncate(1);
        stirred.facts.apart[0].certain = true;
        assert!(!faint(&stirred));
        // a standing fact outside its band holds up the century; a failed run is leaned on
        let mut held = r.clone();
        held.standing[2].1 = Verdict::Moved;
        assert_eq!(role(&held), Role::HoldsUp);
        assert_eq!(held_up(&held), [("pop_1900", Verdict::Moved)]);
        let mut fails = held.clone();
        fails.failed.push((3, "money check".into()));
        assert_eq!(role(&fails), Role::LeanedOn);
    }

    /// A made-up island of eight idle rules, two pairs of which cover for each
    /// other: the island is not the same with both of a pair off. The second
    /// pass finds both pairs, and the rest go off together.
    #[test]
    fn rules_that_cover_for_each_other_are_found() {
        let names: Vec<String> = ["a", "b", "c", "d", "e", "f", "g", "h"].iter().map(|s| s.to_string()).collect();
        let pairs = [("c", "f"), ("b", "h")];
        let mut asked = 0;
        let mut test = |sets: &[Vec<String>]| -> Vec<bool> {
            asked += sets.len();
            sets.iter().map(|s| !pairs.iter().any(|(x, y)| s.iter().any(|n| n == x) && s.iter().any(|n| n == y))).collect()
        };
        let s = search(&names, &mut test);
        println!("{s:#?}");
        assert_eq!(s.joint_same, Some(false));
        let mut groups: Vec<Vec<String>> = s.groups.clone();
        for g in &mut groups {
            g.sort();
        }
        groups.sort();
        assert_eq!(groups, [vec!["b".to_string(), "h".to_string()], vec!["c".to_string(), "f".to_string()]]);
        // one of each pair is kept on; the six left go off together
        assert_eq!(s.together.len(), 6);
        assert!(s.tried.last().is_some_and(|(set, same)| *same && set == &s.together));
        assert!(asked < 30, "{asked} arms");
        // with no pair, the joint arm is the same island and nothing more is played
        let mut calm = |sets: &[Vec<String>]| -> Vec<bool> { vec![true; sets.len()] };
        let s = search(&names, &mut calm);
        assert_eq!((s.joint_same, s.groups.len(), s.tried.len(), s.together.len()), (Some(true), 0, 1, 8));
        // a group of three that clashes only whole is found whole
        let mut three = |sets: &[Vec<String>]| -> Vec<bool> { sets.iter().map(|s| !["a", "e", "h"].iter().all(|n| s.iter().any(|m| m == n))).collect() };
        let s = search(&names, &mut three);
        assert_eq!(s.groups.len(), 1);
        let mut g = s.groups[0].clone();
        g.sort();
        assert_eq!(g, ["a", "e", "h"]);
        // one idle rule, or none: nothing to play
        let mut never = |_: &[Vec<String>]| -> Vec<bool> { panic!("nothing should be played") };
        assert_eq!(search(&names[..1], &mut never).joint_same, None);
    }

    /// The rules asked are read from the tree's lists: every rule's own switch
    /// on under the game's rules, the lesser switches the game turns on, and
    /// none that is off; the names left out each say why.
    #[test]
    fn the_rules_asked_are_the_trees_own() {
        let (ask, out) = asked(&game_params(), &default_params());
        println!("asked ({}): {ask:?}", ask.len());
        for l in &out {
            println!("left out: {}: {}", l.name, l.why);
        }
        let g = rulebook::rulebook(&game_params());
        for &s in SWITCHES {
            let on = g.iter().any(|e| e.name == s && e.value == Value::Bool(true));
            assert_eq!(ask.contains(&s.to_string()), on, "{s}");
            if !on {
                assert!(out.iter().any(|l| l.name == s), "{s} is off and not listed as left out");
            }
        }
        for name in ["people.shelves_go", "works.keep_order", "tasks.land"] {
            assert!(ask.contains(&name.to_string()), "{name} is not asked");
        }
        assert!(out.iter().any(|l| l.name == "tree.stage"), "tree.stage is not listed as left out");
        // every name asked can be set to false from the start
        for n in &ask {
            assert!(sweep::set_field(&game_params(), n, &Value::Bool(false)).is_ok(), "{n}");
        }
    }
}
