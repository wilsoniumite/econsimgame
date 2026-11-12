//! A scorecard: one sweep read against the aims the project's documents state (R8). Not in the Python.
//!
//! ```text
//! isostate-core scorecard DIR [--spread-from OTHER_DIR]
//! ```
//!
//! reads the sweep in `DIR` (its `facts.json`) and prints, for each setting of
//! it, every aim with the fact that reads it: the mean across the seeds, the aim,
//! a tolerance drawn from the spread across the seeds, how far the mean misses
//! the aim in tolerances, whether the aim is met, and on how many islands it is
//! met one by one. Then a single loss over the aims, which a search could later
//! make as small as it can, and the facts a scorecard should be read with that
//! have no aim. Every sweep also writes this card into its directory as
//! `scorecard.txt`.
//!
//! The aims are only those the documents already state, each with the place it
//! is stated (`AIMS`); nothing here invents one. What each means and how to read
//! the card is in `docs/rules/scorecard.md`. There is no search yet.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::facts::Kind;
use crate::sweep::{Then, fmt_fact, read_then};

pub const USAGE: &str = "usage: isostate-core scorecard DIR [--spread-from OTHER_DIR]";

/// Which side of its aim a fact should be on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Test {
    AtLeast(f64),
    AtMost(f64),
}

impl Test {
    pub fn aim(self) -> f64 {
        match self {
            Test::AtLeast(a) | Test::AtMost(a) => a,
        }
    }

    /// How far a value falls on the wrong side of the aim; nought on the right side.
    pub fn miss(self, v: f64) -> f64 {
        match self {
            Test::AtLeast(a) => (a - v).max(0.0),
            Test::AtMost(a) => (v - a).max(0.0),
        }
    }

    fn text(self, kind: Kind) -> String {
        match self {
            Test::AtLeast(a) => format!("at least {}", fmt_fact(kind, a)),
            Test::AtMost(a) => format!("at most {}", fmt_fact(kind, a)),
        }
    }
}

/// One aim, the fact it is read by, and where the documents state it.
#[derive(Clone, Copy, Debug)]
pub struct Aim {
    /// The fact the aim is judged on: a twenty-year mean where the aim is about a
    /// level by a year (R11), since a single year misleads.
    pub fact: &'static str,
    /// A fact printed beside it for the reader (the single year, or a fact that
    /// answers the aim's question better), or "".
    pub beside: &'static str,
    pub test: Test,
    /// Whether it counts in the loss. The speed of the code is shown and not counted:
    /// a search over the economy's numbers should not be steered by the machine.
    pub in_loss: bool,
    /// Where the aim is stated, and in what words.
    pub stated: &'static str,
}

/// Every aim the documents state with a number, as of 12 November 2026.
pub const AIMS: [Aim; 5] = [
    Aim {
        fact: "urban_share_1881_1900",
        beside: "urban_share_1900",
        test: Test::AtLeast(0.5),
        in_loss: true,
        stated: "PLAN.md 1.2, the test of structural change: the urban share \"rises from about 0.25 to 0.5 over the run in most seeds\"; README, Where it stands: \"The plan asks for a half.\"",
    },
    Aim {
        fact: "farm_zigzag",
        beside: "farm_zigzag_beyond_weather",
        test: Test::AtMost(1.2),
        in_loss: true,
        stated: "docs/rules/prices.md, Measuring it again: \"It has worked if the farm zigzag is under 1.2\". Set before the harvests came in; the weather alone gives about 1.95 (docs/rules/harvest.md), and the fact beside it is the zigzag the weather does not account for.",
    },
    Aim {
        fact: "price_swing_iron",
        beside: "",
        test: Test::AtMost(0.16),
        in_loss: true,
        stated: "docs/rules/prices.md, Measuring it again: \"the iron price swings under 0.16 a year\".",
    },
    Aim {
        fact: "years_to_half_coke_furnace",
        beside: "discovered_coke_furnace",
        test: Test::AtMost(10.0),
        in_loss: true,
        stated: "docs/rules/prices.md, Measuring it again: \"coke iron passes half of output within 10 years of its discovery\".",
    },
    Aim {
        fact: "cpu_seconds",
        beside: "seconds",
        test: Test::AtMost(15.0),
        in_loss: false,
        stated: "PLAN.md 1.1: \"150 years in under 15 seconds\" (set for the port of the Python; a run under the game's rules does more). Not in the loss.",
    },
];

/// Aims the documents state that no fact reads yet, or state with no number.
pub const UNREAD: [&str; 6] = [
    "iron output swinging under 0.09 a year in every half-century (docs/rules/prices.md, Measuring it again): no fact gives iron output's swing by half-century",
    "iron stock near 2 months with under 4 per cent of orders unfilled (docs/rules/prices.md, Measuring it again): no fact gives either",
    "iron and rails no longer cycling with a period of a few years (PLAN.md 1.2): no number; read price_swing_iron and price_swing_rails",
    "the railway going on growing after the first rush instead of stopping (PLAN.md 1.2): no number; read rail_last_growth_year and rail_km_1870",
    "a table of facts with tolerances for population growth, the real wage's path, coke iron's share by 1800, mill cloth's by 1830 and rail km by 1870 (PLAN.md 1.3): the facts are named and no numbers given; they are listed below",
    "two to four items a decade in the major feed (PLAN.md 1.4): held by the chronicle's own rule (per_decade and at_least), so not a thing to calibrate",
];

/// Facts a scorecard should be read with that have no aim: PLAN.md 1.3's list,
/// the README's headlines, and the worries the rules' pages carry. Twenty-year
/// means where there are any.
pub const WITHOUT_AIM: [&str; 24] = [
    "pop_1900",
    "real_wage_1751_1770_vs_1750",
    "real_wage_1791_1810_vs_1750",
    "real_wage_1831_1850_vs_1750",
    "real_wage_1871_1890_vs_1750",
    "real_wage_1881_1900_vs_1750",
    "output_per_head_1881_1900_vs_1750",
    "output_per_head_peak_year",
    "farm_share_1881_1900",
    "industry_share_1881_1900",
    "mine_share_1881_1900",
    "income_share_work_1881_1900",
    "share_coke_furnace_1800",
    "share_textile_mill_1830",
    "share_steel_rolling_1890",
    "rail_first_year",
    "rail_km_1870",
    "rail_last_growth_year",
    "price_swing_rails",
    "price_tools_dearest_vs_median",
    "price_tools_years_over_twice_median",
    "real_wage_worst_year_median_town",
    "famine_years",
    "emigration_years",
];

/// Wages, swings and hard spells want this many islands (docs/rules/works.md,
/// "A run differs from its neighbour by a lot").
pub const SEEDS_WANTED: usize = 16;

/// What a tolerance is never less than, as a share of the aim: a fact that every
/// island gives alike would otherwise make any miss infinitely large.
pub const TOLERANCE_FLOOR: f64 = 0.01;

/// A fact across the seeds of one setting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spread {
    pub n: usize,
    pub mean: f64,
    pub least: f64,
    pub greatest: f64,
    /// The standard deviation across the seeds (n - 1 in the denominator); NaN with one seed.
    pub sd: f64,
}

pub fn spread(values: &[(u64, f64)]) -> Option<Spread> {
    let v: Vec<f64> = values.iter().map(|&(_, x)| x).filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    let n = v.len();
    let mean = v.iter().sum::<f64>() / n as f64;
    let sd = if n >= 2 { (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1) as f64).sqrt() } else { f64::NAN };
    Some(Spread { n, mean, least: v.iter().copied().fold(f64::INFINITY, f64::min), greatest: v.iter().copied().fold(f64::NEG_INFINITY, f64::max), sd })
}

/// One aim judged on one setting.
#[derive(Clone, Debug)]
pub struct Judged {
    pub aim: Aim,
    pub spread: Option<Spread>,
    pub tolerance: f64,
    /// Whether the tolerance is the floor and not a spread.
    pub floored: bool,
    /// How far the mean misses the aim, in tolerances (nought when it does not).
    pub misses_by: f64,
    pub met: bool,
    /// Islands that meet the aim on their own, of those with a value.
    pub seeds_met: usize,
}

/// Each aim judged on one setting's values, with tolerances from `reference`'s
/// spread if given (another sweep's setting), else from the setting's own.
pub fn judge(values: &HashMap<String, Vec<(u64, f64)>>, reference: Option<&HashMap<String, Vec<(u64, f64)>>>) -> Vec<Judged> {
    AIMS.iter()
        .map(|&aim| {
            let own = values.get(aim.fact).map(|v| &v[..]).unwrap_or(&[]);
            let here = spread(own);
            let sd = match reference {
                Some(r) => r.get(aim.fact).and_then(|v| spread(v)).map_or(f64::NAN, |s| s.sd),
                None => here.map_or(f64::NAN, |s| s.sd),
            };
            let floor = TOLERANCE_FLOOR * aim.test.aim().abs();
            let floored = !(sd >= floor);
            let tolerance = if floored { floor } else { sd };
            let misses_by = here.map_or(f64::NAN, |s| aim.test.miss(s.mean) / tolerance);
            let seeds_met = own.iter().filter(|&&(_, v)| v.is_finite() && aim.test.miss(v) == 0.0).count();
            Judged { aim, spread: here, tolerance, floored, misses_by, met: misses_by <= 1.0, seeds_met }
        })
        .collect()
}

/// The loss a search would make as small as it can: over the aims in the loss,
/// the sum of the squares of how far each mean misses its aim, in tolerances. An
/// aim with no value at all counts as missed by ten tolerances, so that a setting
/// is never better for losing a fact.
pub fn loss(judged: &[Judged]) -> f64 {
    judged.iter().filter(|j| j.aim.in_loss).map(|j| if j.misses_by.is_finite() { j.misses_by * j.misses_by } else { 100.0 }).sum()
}

/// The kinds of the facts, as the sweep wrote them, for printing.
fn kinds(dir: &Path) -> HashMap<String, Kind> {
    let text = std::fs::read_to_string(dir.join("facts.json")).unwrap_or_default();
    let doc: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    doc["facts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|f| Some((f["name"].as_str()?.to_string(), Kind::parse(f["kind"].as_str()?)?)))
        .collect()
}

/// The seeds of the runs that failed, by setting, in the order the sweep wrote
/// them. `read_then` reads finished runs only, so without this a setting would be
/// scored on the islands it survived and look no worse for those it lost.
fn failed(dir: &Path) -> Vec<(String, Vec<u64>)> {
    let text = std::fs::read_to_string(dir.join("facts.json")).unwrap_or_default();
    let doc: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    let mut out: Vec<(String, Vec<u64>)> = Vec::new();
    for run in doc["runs"].as_array().into_iter().flatten().filter(|r| r["ok"].as_bool() != Some(true)) {
        let label = run["setting"].as_str().unwrap_or("").to_string();
        let seed = run["seed"].as_u64().unwrap_or(0);
        match out.iter_mut().find(|(l, _)| *l == label) {
            Some((_, seeds)) => seeds.push(seed),
            None => out.push((label, vec![seed])),
        }
    }
    out
}

/// The card of the sweep in `dir`, every setting of it, with tolerances from the
/// sweep in `spread_from` (its setting of the same name, or its first) if given.
pub fn card(dir: &Path, spread_from: Option<&Path>) -> Result<String, String> {
    let now = read_then(dir)?;
    let reference = match spread_from {
        Some(p) => Some(read_then(p)?),
        None => None,
    };
    let kinds = kinds(dir);
    let kind = |name: &str| kinds.get(name).copied().unwrap_or(Kind::Real);
    let show = |name: &str, v: f64| if v.is_finite() { fmt_fact(kind(name), v) } else { "-".to_string() };
    let failed = failed(dir);
    let mut out = String::new();
    // a setting whose every run failed has no card, and says so first
    for (label, seeds) in failed.iter().filter(|(l, _)| !now.iter().any(|t| t.label == *l)) {
        let _ = writeln!(out, "Scorecard: {label}: every run failed (seeds {}), so there is nothing to score\n", seeds.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "));
    }
    for setting in &now {
        let refset: Option<&Then> = reference.as_ref().map(|r| r.iter().find(|t| t.label == setting.label).unwrap_or(&r[0]));
        let judged = judge(&setting.values, refset.map(|t| &t.values));
        let mut seeds = setting.seeds.clone();
        seeds.sort();
        let _ = writeln!(out, "Scorecard: {}, {} run{} (seed{} {}) in {}", setting.label, seeds.len(), if seeds.len() == 1 { "" } else { "s" },
            if seeds.len() == 1 { "" } else { "s" },
            seeds.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "), dir.display());
        if let Some((_, lost)) = failed.iter().find(|(l, _)| *l == setting.label) {
            let _ = writeln!(
                out,
                "  ({} run{} FAILED and {} not read: seed{} {}. The card reads only the islands that finished, which may flatter this setting)",
                lost.len(),
                if lost.len() == 1 { "" } else { "s" },
                if lost.len() == 1 { "is" } else { "are" },
                if lost.len() == 1 { "" } else { "s" },
                lost.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ")
            );
        }
        if seeds.len() < 2 && refset.is_none() {
            let _ = writeln!(out, "  (one island has no spread: every tolerance is the floor and the loss means little; give --spread-from a sweep of many)");
        }
        if seeds.len() < SEEDS_WANTED {
            let _ = writeln!(out, "  (fewer than {SEEDS_WANTED} islands: wages, swings and hard spells want {SEEDS_WANTED}, docs/rules/works.md)");
        }
        let _ = writeln!(
            out,
            "tolerance: each fact's standard deviation across {}, at least {} of the aim\n\
             met: the mean misses the aim by no more than one tolerance; islands: how many meet it on their own\n\
             loss: the sum over the aims in the loss of (miss / tolerance) squared, where miss is how far the\n  \
             mean falls on the wrong side of the aim (an aim with no value counts as missed by ten)\n",
            match (spread_from, refset) {
                (Some(p), Some(t)) => format!("the seeds of \"{}\" in {}", t.label, p.display()),
                _ => "these seeds".to_string(),
            },
            TOLERANCE_FLOOR
        );
        let total = loss(&judged);
        let head = ["fact", "mean", "least .. greatest", "aim", "tolerance", "misses by", "met", "islands"];
        let mut rows: Vec<[String; 8]> = Vec::new();
        for j in &judged {
            let k = kind(j.aim.fact);
            let (mean, range) = match j.spread {
                Some(s) => (fmt_fact(k, s.mean), format!("[{} .. {}]", fmt_fact(k, s.least), fmt_fact(k, s.greatest))),
                None => ("-".to_string(), String::new()),
            };
            let tol = if j.spread.is_none() { "-".to_string() } else { format!("{}{}", fmt_tol(j.tolerance), if j.floored { " (floor)" } else { "" }) };
            let by = if j.misses_by.is_finite() { format!("{:.2}", j.misses_by) } else { "-".to_string() };
            let met = match (j.spread.is_some(), j.met) {
                (false, _) => "-",
                (true, true) => "yes",
                (true, false) => "no",
            };
            // of every island that finished: one with no value has not met the aim
            let n = setting.seeds.len();
            let name = if j.aim.in_loss { j.aim.fact.to_string() } else { format!("{} (not in the loss)", j.aim.fact) };
            rows.push([name, mean, range, j.aim.test.text(k), tol, by, met.to_string(), format!("{}/{n}", j.seeds_met)]);
        }
        let width: Vec<usize> = (0..8).map(|c| rows.iter().map(|r| r[c].chars().count()).chain([head[c].len()]).max().unwrap_or(0)).collect();
        let line = |cells: &[String]| -> String {
            let mut s = String::from("  ");
            for (c, cell) in cells.iter().enumerate() {
                if c == 0 || c == 2 {
                    let _ = write!(s, "{cell:<w$}  ", w = width[c]);
                } else {
                    let _ = write!(s, "{cell:>w$}  ", w = width[c]);
                }
            }
            s.trim_end().to_string() + "\n"
        };
        out.push_str("Aims\n");
        out.push_str(&line(&head.map(String::from)));
        for (j, row) in judged.iter().zip(&rows) {
            out.push_str(&line(row));
            if j.spread.is_none() {
                let _ = writeln!(out, "      no island has a value: the sweep has no fact of this name (an older sweep, or runs that do not end in 1900), or no run reached it");
            }
            if !j.aim.beside.is_empty() {
                let b = setting.values.get(j.aim.beside).and_then(|v| spread(v));
                let _ = writeln!(out, "      beside it, {}: {}", j.aim.beside, b.map_or("-".to_string(), |s| format!(
                    "{} [{} .. {}]",
                    show(j.aim.beside, s.mean),
                    show(j.aim.beside, s.least),
                    show(j.aim.beside, s.greatest)
                )));
            }
            let _ = writeln!(out, "      stated: {}", j.aim.stated);
        }
        let mut parts: Vec<(f64, &str)> =
            judged.iter().filter(|j| j.aim.in_loss).map(|j| (if j.misses_by.is_finite() { j.misses_by * j.misses_by } else { 100.0 }, j.aim.fact)).collect();
        parts.sort_by(|a, b| b.0.total_cmp(&a.0));
        let share = |x: f64| if total > 0.0 { format!("{:.0}%", 100.0 * x / total) } else { "-".to_string() };
        let _ = writeln!(out, "\nloss: {:.2} ({})", total, parts.iter().map(|(x, f)| format!("{f} {}", share(*x))).collect::<Vec<_>>().join(", "));

        out.push_str("\nStated, and not read by a number here\n");
        for u in UNREAD {
            let _ = writeln!(out, "  - {u}");
        }
        out.push_str("\nRead without an aim\n");
        let w = WITHOUT_AIM.iter().map(|f| f.len()).max().unwrap_or(0);
        for f in WITHOUT_AIM {
            let s = setting.values.get(f).and_then(|v| spread(v));
            let text = match s {
                Some(s) if s.n < setting.seeds.len() => format!("{:>9}  [{} .. {}] ({}/{})", show(f, s.mean), show(f, s.least), show(f, s.greatest), s.n, setting.seeds.len()),
                Some(s) => format!("{:>9}  [{} .. {}]", show(f, s.mean), show(f, s.least), show(f, s.greatest)),
                None => format!("{:>9}", "-"),
            };
            let _ = writeln!(out, "  {f:<w$}  {text}");
        }
        out.push('\n');
    }
    Ok(out)
}

/// A tolerance to three significant figures.
fn fmt_tol(v: f64) -> String {
    if !v.is_finite() {
        "-".to_string()
    } else if v == 0.0 {
        "0".to_string()
    } else {
        let places = (2 - v.abs().log10().floor() as i64).max(0) as usize;
        format!("{v:.places$}")
    }
}

pub fn parse_args(args: &[String]) -> Result<(PathBuf, Option<PathBuf>), String> {
    let (mut dir, mut spread_from) = (None, None);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--spread-from" => spread_from = Some(PathBuf::from(it.next().ok_or_else(|| format!("--spread-from needs a directory\n{USAGE}"))?)),
            s if s.starts_with("--") => return Err(format!("unknown option {s:?}\n{USAGE}")),
            s if dir.is_none() => dir = Some(PathBuf::from(s)),
            s => return Err(format!("one sweep at a time: {s:?} as well\n{USAGE}")),
        }
    }
    Ok((dir.ok_or_else(|| format!("which sweep?\n{USAGE}"))?, spread_from))
}

/// The `scorecard` subcommand: print the card of a sweep already made.
pub fn main_with_args(args: &[String]) -> Result<(), String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let (dir, spread_from) = parse_args(args)?;
    let text = card(&dir, spread_from.as_deref())?;
    let _ = std::io::Write::write_all(&mut std::io::stdout(), text.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(pairs: &[(&str, &[f64])]) -> HashMap<String, Vec<(u64, f64)>> {
        pairs.iter().map(|(name, v)| (name.to_string(), v.iter().enumerate().map(|(k, &x)| (k as u64 + 1, x)).collect())).collect()
    }

    #[test]
    fn an_aim_is_met_within_one_tolerance_and_the_loss_counts_the_misses() {
        // urban share: mean 0.48, sd 0.02 (0.46 and 0.50 alternately): misses 0.5 by one tolerance, so met
        let v = values(&[
            ("urban_share_1881_1900", &[0.46, 0.50, 0.46, 0.50]),
            ("farm_zigzag", &[2.0, 2.4, 2.0, 2.4]),
            ("price_swing_iron", &[0.1, 0.12, 0.14, 0.16]),
            ("cpu_seconds", &[30.0, 30.0, 30.0, 30.0]),
        ]);
        let j = judge(&v, None);
        let by = |f: &str| j.iter().find(|x| x.aim.fact == f).unwrap();
        let u = by("urban_share_1881_1900");
        assert!((u.tolerance - 0.0230940).abs() < 1e-6, "{u:?}");
        assert!(u.met && (u.misses_by - 0.02 / u.tolerance).abs() < 1e-12 && u.seeds_met == 2, "{u:?}");
        // the zigzag misses 1.2 by 1.0 with a tolerance of 0.23: by more than four, not met, no island meets it
        let z = by("farm_zigzag");
        assert!(!z.met && z.misses_by > 4.0 && z.seeds_met == 0, "{z:?}");
        // the iron swing is on the right side: nought, met, every island
        let i = by("price_swing_iron");
        assert!(i.met && i.misses_by == 0.0 && i.seeds_met == 4, "{i:?}");
        // every island alike: the tolerance is the floor, a hundredth of the aim
        let c = by("cpu_seconds");
        assert!(c.floored && (c.tolerance - 0.15).abs() < 1e-12 && !c.met, "{c:?}");
        // the coke furnace has no value: it counts as missed by ten
        assert!(by("years_to_half_coke_furnace").spread.is_none());
        let want = u.misses_by * u.misses_by + z.misses_by * z.misses_by + 100.0;
        assert!((loss(&j) - want).abs() < 1e-9, "the speed is not in the loss: {} against {want}", loss(&j));

        // tolerances from another sweep's spread: a wider spread there makes the same miss smaller
        let wide = values(&[("urban_share_1881_1900", &[0.40, 0.56, 0.40, 0.56])]);
        let j2 = judge(&v, Some(&wide));
        let u2 = j2.iter().find(|x| x.aim.fact == "urban_share_1881_1900").unwrap();
        assert!(u2.tolerance > 3.0 * u.tolerance &&u2.misses_by < u.misses_by && u2.spread == u.spread, "{u2:?}");
    }

    /// A sweep's facts.json made by hand: (setting, seed, facts or None for a failed run).
    fn sweep_dir(name: &str, runs: &[(&str, u64, Option<Value>)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("isostate-scorecard-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let runs: Vec<Value> = runs
            .iter()
            .map(|(label, seed, facts)| match facts {
                Some(f) => serde_json::json!({"setting": label, "seed": seed, "ok": true, "facts": f}),
                None => serde_json::json!({"setting": label, "seed": seed, "ok": false, "error": "made up"}),
            })
            .collect();
        std::fs::write(dir.join("facts.json"), serde_json::json!({"facts": [], "runs": runs}).to_string()).unwrap();
        dir
    }

    #[test]
    fn a_failed_run_a_lone_island_and_a_missing_fact_are_said_out_loud() {
        let f = |urban: f64, coke: Option<f64>| serde_json::json!({"urban_share_1881_1900": urban, "years_to_half_coke_furnace": coke});
        let dir = sweep_dir("cases", &[
            ("a", 1, Some(f(0.40, Some(8.0)))),
            ("a", 2, None),
            ("a", 3, Some(f(0.42, None))),
            ("b", 1, None),
            ("b", 2, None),
            ("c", 4, Some(f(0.41, Some(9.0)))),
        ]);
        let card = card(&dir, None).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        // a setting that never finished is named, not left out
        assert!(card.contains("Scorecard: b: every run failed (seeds 1 2)"), "{card}");
        // a failed run is named on its setting's card
        assert!(card.contains("(1 run FAILED and is not read: seed 2."), "{card}");
        // an island with no value has not met the aim: of two that finished, one meets it
        let coke = card.lines().find(|l| l.trim_start().starts_with("years_to_half_coke_furnace")).unwrap();
        assert!(coke.trim_end().ends_with("1/2"), "{coke}");
        // a lone island has no spread, and the card says so
        assert!(card.contains("Scorecard: c, 1 run (seed 4)") && card.contains("one island has no spread"), "{card}");
        // a fact no island has
        assert!(card.contains("cpu_seconds (not in the loss)") && card.contains("no island has a value"), "{card}");
    }

    #[test]
    fn every_aim_says_where_it_is_stated() {
        for a in AIMS {
            assert!(a.stated.contains(".md"), "{}: {}", a.fact, a.stated);
        }
        assert_eq!(parse_args(&["d".to_string()]).unwrap(), (PathBuf::from("d"), None));
        assert!(parse_args(&[]).is_err() && parse_args(&["a".to_string(), "b".to_string()]).is_err());
        assert!(parse_args(&["d".to_string(), "--spread".to_string()]).is_err());
    }
}
