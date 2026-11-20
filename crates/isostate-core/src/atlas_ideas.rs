//! The atlas, third plate: the ideas. Not in the Python; a tool, not a rule,
//! so it has no switch and changes nothing a run does. `docs/atlas.md` is its
//! page, with the first plate's.
//!
//! ```text
//! isostate-core atlas ideas --out DIR [--seeds 1-16] [--rules port|game] [--families N] [--end-year Y]
//!                           [--set NAME=VALUE]... [--pin NAME@YEAR]... [--hold NAME@YEAR]... [--edits FILE.json]
//!                           [--against DIR0] [--jobs N] [--quiet]
//! isostate-core atlas ideas --from DIR [--out DIR2] [--against DIR0]
//! ```
//!
//! When each idea is found on each island, against the year our own history
//! had it. An idea is one of the tree's (a method or a technique of the
//! content the runs were played on, `Content::tech_name`, those not known in
//! 1750) or a use of the tasks rule (`Content::tasks.uses`). The list is read
//! from the content when the command runs, never written here, so an idea
//! added to the data is drawn the next time. History's year is the data's own
//! `target` (the tree's in `data/game.toml`, the uses' in `data/tasks.toml`);
//! where the data has none the row says so.
//!
//! Each island is played as the lab plays an arm (`edits::play_noting`), with
//! the list of edits given (`--edits`, `--pin`, `--hold`; none by default, and
//! then each run is the sweep's to the bit), and watched at each year's end:
//! the year each town came to know each idea of the tree (its knowledge above
//! 0.02, from which a works of it may be built there), the year each of an
//! idea's thresholds was met and the year all were, and the year the first
//! works of each method stood.
//!
//! What it writes to DIR: `ideas.html` (the page, one file that opens without
//! the network), `ideas.json` (its figures; `--from DIR` draws the page again
//! from them, and `--against DIR0` lays another folder's spread under each
//! row), `ideas.txt` (the tables in words) and `README.txt`.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde_json::{Value, json};

use crate::cli::{check_end_year, parse_rules, read_edits};
use crate::content::Content;
use crate::edits::{self, Dated, Edit, PinWhen};
use crate::params::{Params, Rules};
use crate::sim::Sim;
use crate::sweep;

pub const USAGE: &str = "usage: isostate-core atlas ideas --out DIR [--seeds 1-16] [--rules port|game] [--families N] [--end-year Y]
                                 [--set NAME=VALUE]... [--pin NAME@YEAR]... [--hold NAME@YEAR]... [--edits FILE.json]
                                 [--against DIR0] [--jobs N] [--quiet]
       isostate-core atlas ideas --from DIR [--out DIR2] [--against DIR0]   (draw the page again from DIR/ideas.json)";

/// A town knows an idea once its knowledge is above this: from here a works of
/// it may be built in the town (`families.rs`, `firms.rs`, `edits.rs`).
pub const KNOWS: f64 = 0.02;

/// What the command line asked for.
#[derive(Clone, Debug)]
pub struct Options {
    pub out: PathBuf,
    pub seeds: Vec<u64>,
    pub rules: Rules,
    pub families: usize,
    pub end_year: i64,
    pub sets: Vec<(String, String)>,
    pub edits_file: Option<PathBuf>,
    /// Each `--pin` and `--hold`: an idea, a year, and what is asked.
    pub pins: Vec<(String, i64, PinWhen)>,
    pub against: Option<PathBuf>,
    pub from: Option<PathBuf>,
    pub jobs: Option<usize>,
    pub quiet: bool,
    pub command: String,
}

/// `NAME@YEAR`.
fn parse_pin(flag: &str, v: &str) -> Result<(String, i64), String> {
    let (name, year) = v.rsplit_once('@').ok_or_else(|| format!("{flag} {v:?}: write it as NAME@YEAR, the idea's name in the data and a year"))?;
    let year: i64 = year.trim().parse().map_err(|_| format!("{flag} {v:?}: {year:?} after the @ is not a year"))?;
    Ok((name.trim().to_string(), year))
}

/// `args[0]` is the plate's name, "ideas".
pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        out: PathBuf::new(),
        seeds: (1..=16).collect(),
        rules: Rules::Game,
        families: 6,
        end_year: 2050,
        sets: Vec::new(),
        edits_file: None,
        pins: Vec::new(),
        against: None,
        from: None,
        jobs: None,
        quiet: false,
        command: crate::atlas::command_line(args),
    };
    let mut out: Option<PathBuf> = None;
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value\n{USAGE}"));
        match a.as_str() {
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            "--seeds" => o.seeds = sweep::parse_seeds(&value("--seeds")?)?,
            "--rules" => o.rules = parse_rules(&value("--rules")?)?,
            "--families" => o.families = value("--families")?.parse().map_err(|e| format!("--families: {e}"))?,
            "--end-year" => o.end_year = value("--end-year")?.parse().map_err(|e| format!("--end-year: {e}"))?,
            "--set" => {
                let v = value("--set")?;
                let (k, x) = v.split_once('=').ok_or_else(|| format!("--set {v:?}: write it as NAME=VALUE"))?;
                if sweep::split_values(x).len() > 1 {
                    return Err(format!("--set {v:?}: the atlas draws one setting; a sweep of several is sweep --set's"));
                }
                o.sets.push((k.trim().to_string(), x.trim().to_string()));
            }
            "--pin" => {
                let (n, y) = parse_pin("--pin", &value("--pin")?)?;
                o.pins.push((n, y, PinWhen::In));
            }
            "--hold" => {
                let (n, y) = parse_pin("--hold", &value("--hold")?)?;
                o.pins.push((n, y, PinWhen::NotBefore));
            }
            "--edits" => o.edits_file = Some(PathBuf::from(value("--edits")?)),
            "--against" => o.against = Some(PathBuf::from(value("--against")?)),
            "--from" => o.from = Some(PathBuf::from(value("--from")?)),
            "--jobs" => o.jobs = Some(value("--jobs")?.parse::<usize>().map_err(|e| format!("--jobs: {e}"))?.max(1)),
            "--quiet" => o.quiet = true,
            other => return Err(format!("unknown option {other:?}\n{USAGE}")),
        }
    }
    o.out = match (out, &o.from) {
        (Some(d), _) => d,
        (None, Some(f)) => f.clone(),
        (None, None) => return Err(format!("--out is required\n{USAGE}")),
    };
    if o.seeds.is_empty() {
        return Err("--seeds: no seeds".to_string());
    }
    Ok(o)
}

/// The rules, the content (with the tree laid, as the runs play it) and the
/// list of edits every island is played with, checked before anything runs:
/// each pin must name an idea of the content.
pub fn plan(o: &Options) -> Result<(Params, Content, Vec<Dated>), String> {
    let mut P = o.rules.params();
    P.n_families = o.families;
    P.end_year = Some(o.end_year);
    for (k, v) in &o.sets {
        P = sweep::set_field(&P, k, &sweep::parse_value(v))?;
    }
    P.check()?;
    let content = o.rules.content();
    check_end_year(&P, &content)?;
    let played = crate::tree::content(&P, content.clone())?;
    let mut list = match &o.edits_file {
        Some(p) => read_edits(p)?,
        None => Vec::new(),
    };
    for (name, year, when) in &o.pins {
        list.push(Dated { before: (played.start_year, 1), edit: Edit::Pin { idea: name.clone(), year: *year, when: *when } });
    }
    for d in &list {
        if let Edit::Pin { idea, .. } = &d.edit {
            if edits::idea_named(&played, idea).is_none() {
                return Err(format!("the pin of {idea:?}: there is no such idea in the content these rules play (a method, a technique, or a use of the tasks rule)"));
            }
        }
    }
    Ok((P, content, list))
}

// ---------------------------------------------------------------- watching an island

/// One island's ideas, as it lived them.
#[derive(Clone, Debug, Default)]
pub struct Island {
    pub seed: u64,
    /// Per idea of the tree (`tech_name`): the year it was found (None never;
    /// the start year for one known in 1750), and the town it was found in.
    pub found: Vec<Option<i64>>,
    pub town: Vec<Option<usize>>,
    /// Per idea of the tree: the year every threshold was first met, and per
    /// good it is found on, (good, threshold, year met).
    pub ready: Vec<Option<i64>>,
    pub met: Vec<Vec<(usize, f64, Option<i64>)>>,
    /// Per use: the year found, and per good, (good, threshold, year met).
    pub use_found: Vec<Option<i64>>,
    pub use_met: Vec<Vec<(usize, f64, Option<i64>)>>,
    /// Per method: the first year a works of it stood at a year's end.
    pub works: Vec<Option<i64>>,
    pub towns: Vec<Town>,
    /// The island's width and height in squares, and its land, a row at a time.
    pub W: usize,
    pub H: usize,
    pub land: Vec<bool>,
    pub seconds: f64,
}

/// One town and the year it came to know each idea of the tree.
#[derive(Clone, Debug, Default)]
pub struct Town {
    pub index: usize,
    pub name: String,
    pub founded: i64,
    pub x: usize,
    pub y: usize,
    pub knew: Vec<Option<i64>>,
}

/// What is watched at each year's end.
#[derive(Default)]
struct Watch {
    on: bool,
    ready: Vec<Option<i64>>,
    met: Vec<Vec<(usize, f64, Option<i64>)>>,
    use_met: Vec<Vec<(usize, f64, Option<i64>)>>,
    works: Vec<Option<i64>>,
    knew: Vec<Vec<Option<i64>>>,
}

impl Watch {
    fn year_end(&mut self, sim: &Sim) {
        let (st, C) = (&sim.st, &sim.content);
        let year = sim.year - 1;
        if !self.on {
            self.on = true;
            self.ready = vec![None; C.NT];
            self.met = (0..C.NT).map(|t| (0..C.G).filter(|&g| st.disc_X0[(t, g)] > 0.0).map(|g| (g, st.disc_X0[(t, g)], None)).collect()).collect();
            self.use_met = C.tasks.uses.iter().map(|u| u.discover.iter().map(|&(g, x)| (g, x, None)).collect()).collect();
            self.works = vec![None; C.M];
            self.knew = vec![vec![None; C.NT]; st.T_max];
        }
        for t in 0..C.NT {
            let mut all = !self.met[t].is_empty();
            for m in self.met[t].iter_mut() {
                if m.2.is_none() && st.cumprod[m.0] >= m.1 {
                    m.2 = Some(year);
                }
                all &= m.2.is_some();
            }
            if all && self.ready[t].is_none() {
                self.ready[t] = Some(year);
            }
        }
        let people: f64 = st.pop.iter().sum::<f64>().max(1.0);
        for (u, us) in C.tasks.uses.iter().enumerate() {
            for m in self.use_met[u].iter_mut() {
                let made = if us.discover_per_head { st.cumprod[m.0] / people } else { st.cumprod[m.0] };
                if m.2.is_none() && made >= m.1 {
                    m.2 = Some(year);
                }
            }
        }
        for f in 0..st.F_count {
            if st.f_alive[f] {
                let w = &mut self.works[st.f_method[f]];
                if w.is_none() {
                    *w = Some(year);
                }
            }
        }
        for i in (0..st.T_max).filter(|&i| st.town_active[i]) {
            for t in 0..C.NT {
                if self.knew[i][t].is_none() && st.know[(i, t)] > KNOWS {
                    self.knew[i][t] = Some(year);
                }
            }
        }
    }
}

/// Play one island with the list of edits, watching it.
pub fn one_island(P: &Params, content: Content, list: &[Dated], seed: u64) -> Result<(Island, Content), String> {
    let t0 = Instant::now();
    let mut w = Watch::default();
    let sim = edits::play_noting(P, content, list, |s, m| {
        if m.is_none() {
            w.year_end(s);
        }
    })?;
    let (st, C) = (&sim.st, &sim.content);
    let y0 = C.start_year;
    let found: Vec<Option<i64>> = (0..C.NT).map(|t| if C.tech_known[t] { Some(y0) } else { (st.tech_year[t] >= 0).then_some(st.tech_year[t]) }).collect();
    let town: Vec<Option<usize>> = (0..C.NT).map(|t| (st.tech_town[t] >= 0 && !C.tech_known[t]).then_some(st.tech_town[t] as usize)).collect();
    let use_found = (0..C.tasks.uses.len()).map(|u| st.tasks.found.get(u).copied().filter(|&y| y >= 0)).collect();
    let W = st.world.W.max(1);
    let towns = (0..st.T_max)
        .filter(|&i| w.knew.get(i).is_some_and(|k| k.iter().any(|y| y.is_some())) || st.town_active[i])
        .map(|i| Town {
            index: i,
            name: st.town_name.get(i).cloned().unwrap_or_else(|| format!("town {i}")),
            founded: st.town_founded.get(i).copied().unwrap_or(y0),
            x: st.town_node[i] % W,
            y: st.town_node[i] / W,
            knew: w.knew.get(i).cloned().unwrap_or_else(|| vec![None; C.NT]),
        })
        .collect();
    let isl = Island {
        seed,
        found,
        town,
        ready: w.ready,
        met: w.met,
        use_found,
        use_met: w.use_met,
        works: w.works,
        towns,
        W: st.world.W,
        H: st.world.H,
        land: st.world.land.clone(),
        seconds: t0.elapsed().as_secs_f64(),
    };
    Ok((isl, sim.content))
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()));
    format!("panicked: {}", text.unwrap_or_else(|| "no message".to_string()))
}

/// Every island played, a few at a time (the sweep's pool, copied).
pub fn run(o: &Options, P: &Params, content: &Content, list: &[Dated]) -> (Vec<Island>, Vec<(u64, String)>, Option<Content>, usize, f64) {
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    let jobs = o.jobs.unwrap_or((cores / 2).max(1)).min(o.seeds.len()).max(1);
    let t0 = Instant::now();
    let next = AtomicUsize::new(0);
    let finished = AtomicUsize::new(0);
    let slots: Mutex<Vec<Option<Result<(Island, Content), String>>>> = Mutex::new(vec![None; o.seeds.len()]);
    std::thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| {
                loop {
                    let id = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&seed) = o.seeds.get(id) else { break };
                    let mut P = P.clone();
                    P.seed = seed;
                    let t = Instant::now();
                    let c = content.clone();
                    let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| one_island(&P, c, list, seed))) {
                        Ok(r) => r,
                        Err(payload) => Err(panic_message(payload)),
                    };
                    if !o.quiet {
                        let n = finished.fetch_add(1, Ordering::Relaxed) + 1;
                        let how = match &outcome {
                            Ok(_) => format!("{:.1} s on the clock", t.elapsed().as_secs_f64()),
                            Err(e) => format!("FAILED: {e}"),
                        };
                        eprintln!("[{n}/{}] seed {seed}: {how}", o.seeds.len());
                    }
                    slots.lock().unwrap()[id] = Some(outcome);
                }
            });
        }
    });
    let mut islands = Vec::new();
    let mut failed = Vec::new();
    let mut played = None;
    for (seed, r) in o.seeds.iter().zip(slots.into_inner().unwrap()) {
        match r.expect("every run reports") {
            Ok((isl, c)) => {
                islands.push(isl);
                played.get_or_insert(c);
            }
            Err(e) => failed.push((*seed, e)),
        }
    }
    (islands, failed, played, jobs, t0.elapsed().as_secs_f64())
}

// ---------------------------------------------------------------- the ideas, read off the islands

/// One row of the drawing.
#[derive(Clone, Debug)]
pub struct Row {
    /// "method", "technique" or "use".
    pub kind: &'static str,
    pub name: String,
    pub label: String,
    /// Its place in `tech_name`, or in the uses.
    pub index: usize,
    /// History's year, the data's `target`.
    pub target: Option<i64>,
    /// The year each island found it, in the islands' order.
    pub years: Vec<Option<i64>>,
    /// What it is found on, in words.
    pub on: String,
}

impl Row {
    pub fn found(&self) -> Vec<i64> {
        let mut v: Vec<i64> = self.years.iter().flatten().copied().collect();
        v.sort();
        v
    }
    /// The years from the first island to find it to the last (None if fewer than two did).
    pub fn spread(&self) -> Option<i64> {
        let v = self.found();
        (v.len() >= 2).then(|| v[v.len() - 1] - v[0])
    }
    pub fn mean(&self) -> Option<f64> {
        let v = self.found();
        (!v.is_empty()).then(|| v.iter().sum::<i64>() as f64 / v.len() as f64)
    }
    pub fn median(&self) -> Option<f64> {
        let v = self.found();
        let n = v.len();
        match n {
            0 => None,
            _ if n % 2 == 1 => Some(v[n / 2] as f64),
            _ => Some(0.5 * (v[n / 2 - 1] + v[n / 2]) as f64),
        }
    }
    pub fn never(&self) -> usize {
        self.years.iter().filter(|y| y.is_none()).count()
    }
    /// Where the row stands: history's year, or (with none in the data) its median find.
    pub fn key(&self, end: i64) -> f64 {
        self.target.map(|y| y as f64).or(self.median()).unwrap_or(end as f64 + 1.0)
    }
}

fn amount(x: f64) -> String {
    if x >= 100.0 { crate::util::fmt_thousands_f0(x) } else { format!("{x}") }
}

/// What an idea of the tree is found on, in words: each good's threshold (and
/// for one written "xN", N years of the island's own 1750 making, with what
/// that came to on these islands), the draw, and the year it is sought from.
fn tree_on(C: &Content, P: &Params, islands: &[Island], t: usize) -> String {
    let mut parts = Vec::new();
    for g in 0..C.G {
        let v = C.tech_disc[(t, g)];
        if v > 0.0 {
            parts.push(format!("{} {}", C.good_label[g].to_lowercase(), amount(v)));
        } else if v < 0.0 {
            let xs: Vec<f64> = islands.iter().filter_map(|i| i.met[t].iter().find(|m| m.0 == g).map(|m| m.1)).filter(|x| *x < 1e17).collect();
            let range = match (xs.iter().copied().reduce(f64::min), xs.iter().copied().reduce(f64::max)) {
                (Some(a), Some(b)) if a == b => format!(", {}", amount(a.round())),
                (Some(a), Some(b)) => format!(", {} to {} on these islands", amount(a.round()), amount(b.round())),
                _ => String::new(),
            };
            parts.push(format!("{} {} years of the island's own 1750 making{range}", C.good_label[g].to_lowercase(), -v));
        }
    }
    let after = C.tech_after.get(t).copied().filter(|&y| y > 0).map_or(String::new(), |y| format!(", sought from {y}"));
    let woods = if C.tech_name[t] == "managed_woodland" { " (only with the woods' rule on)" } else { "" };
    format!(
        "made since 1750: {}; then a draw each year at {} p\u{2074} / (1 + p\u{2074}), p the least of made over threshold{after}{woods}",
        if parts.is_empty() { "nothing (never sought)".to_string() } else { parts.join(", ") },
        P.discovery_rate
    )
}

/// What a use of the tasks rule is found on, in words.
fn use_on(C: &Content, P: &Params, u: usize) -> String {
    let us = &C.tasks.uses[u];
    let goods: Vec<String> = us.discover.iter().map(|&(g, x)| format!("{} {}{}", C.good_label[g].to_lowercase(), amount(x), if us.discover_per_head { " a head" } else { "" })).collect();
    let switch = us.switch.as_ref().map_or(String::new(), |s| format!("; behind the switch tasks.{s}, {}", if P.tasks.switch_on(s) { "on" } else { "off: never found" }));
    let tasks = if P.tasks.enabled { "" } else { "; the tasks rule is off: never found" };
    format!("made since 1750: {}; found at the first year's turn from {} once every one is met, with no draw{switch}{tasks}", goods.join(", "), us.found_after)
}

/// Every idea of the content, as rows: the tree's not known in 1750, then the
/// uses, in the order of the data; and the names of those known in 1750.
pub fn rows(C: &Content, P: &Params, islands: &[Island]) -> (Vec<Row>, Vec<String>) {
    let mut out = Vec::new();
    let mut known = Vec::new();
    for t in 0..C.NT {
        if C.tech_known[t] {
            known.push(C.tech_label[t].clone());
            continue;
        }
        out.push(Row {
            kind: if t < C.M { "method" } else { "technique" },
            name: C.tech_name[t].clone(),
            label: C.tech_label[t].clone(),
            index: t,
            target: C.tech_target.get(t).copied().filter(|&y| y > 0),
            years: islands.iter().map(|i| i.found[t]).collect(),
            on: tree_on(C, P, islands, t),
        });
    }
    for (u, us) in C.tasks.uses.iter().enumerate() {
        out.push(Row {
            kind: "use",
            name: us.name.clone(),
            label: edits::use_label(&us.name),
            index: u,
            target: us.target,
            years: islands.iter().map(|i| i.use_found.get(u).copied().flatten()).collect(),
            on: use_on(C, P, u),
        });
    }
    (out, known)
}

/// The rows in the order they are drawn: by history's year, those with none by their median find.
pub fn ordered(rows: &[Row], end: i64) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..rows.len()).collect();
    idx.sort_by(|&a, &b| rows[a].key(end).total_cmp(&rows[b].key(end)).then(a.cmp(&b)));
    idx
}

fn yr(v: Option<i64>) -> String {
    v.map_or("-".to_string(), |y| y.to_string())
}

/// The tables in words: every row, the ten furthest apart, the ten furthest
/// from history, and what each is found on.
pub fn summary(rows: &[Row], known: &[String], seeds: &[u64], end: i64, failed: &[(u64, String)]) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "The ideas: when each was found on islands {}, against history's year (the data's target)\n", seeds.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "));
    let _ = writeln!(s, "  {:<34} {:>7} {:>6} {:>6} {:>7} {:>7} {:>6}  years, island by island", "idea", "history", "first", "last", "spread", "mean", "never");
    for &k in &ordered(rows, end) {
        let r = &rows[k];
        let f = r.found();
        let _ = writeln!(
            s,
            "  {:<34} {:>7} {:>6} {:>6} {:>7} {:>7} {:>6}  {}",
            format!("{} ({})", r.label, r.kind),
            r.target.map_or("none".to_string(), |y| y.to_string()),
            yr(f.first().copied()),
            yr(f.last().copied()),
            r.spread().map_or("-".to_string(), |x| x.to_string()),
            r.mean().map_or("-".to_string(), |x| format!("{x:.1}")),
            r.never(),
            r.years.iter().map(|y| yr(*y)).collect::<Vec<_>>().join(" ")
        );
    }
    let _ = writeln!(s, "\nKnown in 1750, and so not drawn as rows ({}): {}", known.len(), known.join(", "));
    let _ = writeln!(s, "\nThe ten found furthest apart (the last island's year less the first's; islands that never found it are counted under never)");
    for k in furthest_apart(rows).into_iter().take(10) {
        let r = &rows[k];
        let _ = writeln!(s, "  {:<34} {:>3} years  ({} to {}; never on {})", r.label, r.spread().unwrap_or(0), yr(r.found().first().copied()), yr(r.found().last().copied()), r.never());
    }
    let _ = writeln!(s, "\nThe ten furthest from history on the mean (the mean of the islands that found it, less history's year)");
    for k in furthest_from_history(rows).into_iter().take(10) {
        let r = &rows[k];
        let _ = writeln!(s, "  {:<34} {:>+6.1} years  (history {}, mean {:.1}; never on {})", r.label, r.mean().unwrap() - r.target.unwrap() as f64, r.target.unwrap(), r.mean().unwrap(), r.never());
    }
    let _ = writeln!(s, "\nWhat each is found on");
    for r in rows {
        let _ = writeln!(s, "  {}: {}", r.label, r.on);
    }
    for (seed, why) in failed {
        let _ = writeln!(s, "FAILED: island {seed}: {why}");
    }
    s
}

/// The rows found on two islands or more, widest spread first.
pub fn furthest_apart(rows: &[Row]) -> Vec<usize> {
    let mut v: Vec<usize> = (0..rows.len()).filter(|&k| rows[k].spread().is_some()).collect();
    v.sort_by(|&a, &b| rows[b].spread().cmp(&rows[a].spread()).then(a.cmp(&b)));
    v
}

/// The rows with a history's year and a find, furthest from history on the mean first.
pub fn furthest_from_history(rows: &[Row]) -> Vec<usize> {
    let gap = |r: &Row| r.mean().zip(r.target).map(|(m, t)| (m - t as f64).abs());
    let mut v: Vec<usize> = (0..rows.len()).filter(|&k| gap(&rows[k]).is_some()).collect();
    v.sort_by(|&a, &b| gap(&rows[b]).unwrap().total_cmp(&gap(&rows[a]).unwrap()).then(a.cmp(&b)));
    v
}

// ---------------------------------------------------------------- the page's figures

fn opt(v: Option<i64>) -> Value {
    v.map_or(Value::Null, |y| json!(y))
}

/// The land as rows of 0 and 1, run-length written: "W,H;" then counts of
/// alternating sea and land squares, row by row from the north-west.
fn land_runs(isl: &Island) -> String {
    let mut s = format!("{},{};", isl.W, isl.H);
    let mut runs = Vec::new();
    let (mut cur, mut n) = (false, 0usize);
    for &l in &isl.land {
        if l == cur {
            n += 1;
        } else {
            runs.push(n.to_string());
            cur = l;
            n = 1;
        }
    }
    runs.push(n.to_string());
    s.push_str(&runs.join(","));
    s
}

/// The figures the page carries (`ideas.json`).
#[allow(clippy::too_many_arguments)]
pub fn page_json(C: &Content, P: &Params, o: &Options, islands: &[Island], failed: &[(u64, String)], list: &[Dated], code: Option<&str>, seconds: f64) -> Value {
    let (rows, known) = rows(C, P, islands);
    let end = P.end_year.unwrap_or(C.end_year);
    let order = ordered(&rows, end);
    let row_json: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "kind": r.kind, "name": r.name, "label": r.label, "index": r.index,
                "target": opt(r.target), "years": r.years.iter().map(|y| opt(*y)).collect::<Vec<_>>(), "on": r.on,
            })
        })
        .collect();
    let isl_json: Vec<Value> = islands
        .iter()
        .map(|i| {
            let met = |m: &Vec<Vec<(usize, f64, Option<i64>)>>| -> Vec<Value> {
                m.iter().map(|v| Value::Array(v.iter().map(|(g, x, y)| json!([g, x, opt(*y)])).collect())).collect()
            };
            json!({
                "seed": i.seed,
                "found": i.found.iter().map(|y| opt(*y)).collect::<Vec<_>>(),
                "town": i.town.iter().map(|t| t.map_or(Value::Null, |t| json!(t))).collect::<Vec<_>>(),
                "ready": i.ready.iter().map(|y| opt(*y)).collect::<Vec<_>>(),
                "met": met(&i.met),
                "use_found": i.use_found.iter().map(|y| opt(*y)).collect::<Vec<_>>(),
                "use_met": met(&i.use_met),
                "works": i.works.iter().map(|y| opt(*y)).collect::<Vec<_>>(),
                "land": land_runs(i),
                "towns": i.towns.iter().map(|t| json!({
                    "t": t.index, "name": t.name, "from": t.founded, "x": t.x, "y": t.y,
                    "knew": t.knew.iter().map(|y| opt(*y)).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "plate": "ideas",
        "made": crate::record::today(),
        "command": o.command,
        "code": code.unwrap_or("unknown"),
        "rules": o.rules.as_str(),
        "families": o.families,
        "sets": o.sets.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" "),
        "edits": list,
        "y0": C.start_year,
        "y1": end,
        "knows": KNOWS,
        "goods": C.good_label,
        "tech_name": C.tech_name,
        "tech_label": C.tech_label,
        "methods": C.M,
        "rows": row_json,
        "order": order,
        "known": known,
        "islands": isl_json,
        "failed": failed.iter().map(|(s, w)| json!([s, w])).collect::<Vec<_>>(),
        "seconds": seconds,
        "against": Value::Null,
    })
}

/// Another folder's spread laid under each row, by the idea's name: (label of
/// that folder, name -> [first, last, median, never]).
pub fn against(dir: &std::path::Path) -> Result<Value, String> {
    let src = dir.join("ideas.json");
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&src).map_err(|e| format!("--against: {}: {e}", src.display()))?).map_err(|e| format!("--against: {}: {e}", src.display()))?;
    let mut by = serde_json::Map::new();
    for r in doc["rows"].as_array().cloned().unwrap_or_default() {
        let mut ys: Vec<i64> = r["years"].as_array().map(|a| a.iter().filter_map(|y| y.as_i64()).collect()).unwrap_or_default();
        ys.sort();
        let n = r["years"].as_array().map_or(0, |a| a.len());
        let med = if ys.is_empty() { Value::Null } else if ys.len() % 2 == 1 { json!(ys[ys.len() / 2]) } else { json!(0.5 * (ys[ys.len() / 2 - 1] + ys[ys.len() / 2]) as f64) };
        let name = format!("{}:{}", r["kind"].as_str().unwrap_or(""), r["name"].as_str().unwrap_or(""));
        by.insert(name, json!([ys.first(), ys.last(), med, n - ys.len()]));
    }
    let label = dir.file_name().map_or(dir.display().to_string(), |n| n.to_string_lossy().into_owned());
    Ok(json!({"label": label, "command": doc["command"], "rows": by}))
}

/// The page: the drawing's code and its figures in one file, nothing fetched.
pub fn page(json: &str) -> String {
    let safe = json.replace("</", "<\\/");
    PAGE.replace("/*ATLAS_DATA*/null", &safe)
}

const PAGE: &str = include_str!("atlas_ideas.html");

fn git_describe() -> Option<String> {
    let out = std::process::Command::new("git").args(["describe", "--always", "--dirty"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !text.is_empty()).then_some(text)
}

fn readme(o: &Options, list: &[Dated], code: Option<&str>, n: usize, failed: usize, jobs: usize, seconds: f64) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "The atlas, third plate: the ideas (isostate-core atlas ideas), {} (UTC).\n", crate::record::today());
    let _ = writeln!(s, "command:  {}", o.command);
    let _ = writeln!(s, "rules:    {}, {} families, to {}{}", o.rules.as_str(), o.families, o.end_year, if o.sets.is_empty() { String::new() } else { format!(", with {}", o.sets.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")) });
    let _ = writeln!(s, "edits:    {}", if list.is_empty() { "none: each island is the sweep's".to_string() } else { serde_json::to_string(list).unwrap_or_default() });
    let _ = writeln!(s, "code:     {}", code.unwrap_or("unknown (git describe gave nothing)"));
    let _ = writeln!(s, "seeds:    {}", o.seeds.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "));
    let _ = writeln!(s, "runs:     {n} ({failed} failed), {jobs} at a time, {seconds:.0} s in all");
    s.push_str(
        "\nFiles\n\
         ideas.html  the page: open it in any browser; it fetches nothing\n\
         ideas.json  the figures the page carries; atlas ideas --from DIR draws the page again from it\n\
         ideas.txt   the tables, in words\n\
         \nHow the page is made and how to read it: docs/atlas.md.\n",
    );
    s
}

/// The plate's command, with or without its tables on stdout (the tests leave it off).
pub fn command(args: &[String], print: bool) -> Result<(), String> {
    let o = parse_args(args)?;
    let against_doc = match &o.against {
        Some(d) => Some(against(d)?),
        None => None,
    };
    if let Some(from) = &o.from {
        let src = from.join("ideas.json");
        let mut doc: Value = serde_json::from_str(&std::fs::read_to_string(&src).map_err(|e| format!("--from: {}: {e}", src.display()))?).map_err(|e| format!("--from: {}: {e}", src.display()))?;
        if let Some(a) = against_doc {
            doc["against"] = a;
        }
        std::fs::create_dir_all(&o.out).map_err(|e| format!("--out {}: {e}", o.out.display()))?;
        let dst = o.out.join("ideas.html");
        std::fs::write(&dst, page(&doc.to_string())).map_err(|e| format!("{}: {e}", dst.display()))?;
        if !o.quiet {
            eprintln!("drew {} again from {}", dst.display(), src.display());
        }
        return Ok(());
    }
    let (P, content, list) = plan(&o)?;
    std::fs::create_dir_all(&o.out).map_err(|e| format!("--out {}: {e}", o.out.display()))?;
    let (islands, failed, played, jobs, seconds) = run(&o, &P, &content, &list);
    let Some(C) = played else {
        let why: Vec<String> = failed.iter().map(|(s, w)| format!("island {s}: {w}")).collect();
        return Err(format!("every run failed: {}", why.join("; ")));
    };
    let code = git_describe();
    let mut doc = page_json(&C, &P, &o, &islands, &failed, &list, code.as_deref(), seconds);
    if let Some(a) = against_doc {
        doc["against"] = a;
    }
    let (rows, known) = rows(&C, &P, &islands);
    let text = summary(&rows, &known, &islands.iter().map(|i| i.seed).collect::<Vec<_>>(), P.end_year.unwrap_or(C.end_year), &failed);
    let json = doc.to_string();
    let put = |name: &str, body: &str| std::fs::write(o.out.join(name), body).map_err(|e| format!("{}: {e}", o.out.join(name).display()));
    put("ideas.json", &json)?;
    put("ideas.html", &page(&json))?;
    put("ideas.txt", &text)?;
    put("README.txt", &readme(&o, &list, code.as_deref(), o.seeds.len(), failed.len(), jobs, seconds))?;
    if print {
        let _ = std::io::Write::write_all(&mut std::io::stdout(), text.as_bytes());
    }
    if !o.quiet {
        eprintln!("{seconds:.0} s on the clock, {jobs} at a time; the page is {} ({:.1} MB)", o.out.join("ideas.html").display(), json.len() as f64 / 1e6);
    }
    if !failed.is_empty() {
        return Err(format!("{} of {} runs failed; the rest are drawn in {}", failed.len(), o.seeds.len(), o.out.display()));
    }
    Ok(())
}
