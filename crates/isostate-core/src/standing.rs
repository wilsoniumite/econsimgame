//! The standing check (`isostate-core sweep ... --standing`; docs/rules/README.md,
//! "The standing check"). Not in the Python.
//!
//! Every stage of the run to 2050 changes rules that the played century, 1750
//! to 1900, also runs under. The standing check asks whether the century still
//! stands where it stood: twelve facts of the island to 1900, each set against
//! the spread a reference arm has across its seeds. A fact is
//!
//! - **inside** when the later arm's mean lies within the reference arm's
//!   [least .. greatest];
//! - **moved** when every seed of one arm lies on one side of every seed of the
//!   other (as `sweep::compare` stars a fact, and with as many runs a side): a
//!   change on every island, which the stage's page must say it meant;
//! - **OUTSIDE** when the mean has left the spread and the seeds have not
//!   stood apart.
//!
//! Only "inside" passes. The reference is the sweep's first setting, each later
//! setting set against it; or, with one setting and `--compare DIR`, the
//! earlier sweep's setting of the same name (or its first). Beneath the twelve,
//! `last_town_founded` is shown and not counted. The check changes no run.
//!
//! It is a gate: when any of the twelve is not inside, the sweep ends with an
//! error after writing everything, and its last lines name each such fact, the
//! arm, its mean, the band (the reference's [least .. greatest]) and the
//! islands that lie outside the band. Run on to 2050 against main's sweep, it
//! is the gate's guard on the played century:
//!
//! ```text
//! isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --standing --compare MAIN_SWEEP --out DIR
//! ```
//!
//! Beside the twelve it reports the growth the design asks of the century after
//! 1900 (`GROWTH`): the real wage rising at least 1 per cent a year from 1900 to
//! 1970 and no lower in 2050 than in 1970, and fewer hands on the land in 2050
//! than in 1900. Those are printed with their values, and fail the sweep only
//! with `--growth`, for the stage that mends them. The wage they read is the
//! linked one, nine-year means (docs/rules/measures.md), where a run keeps it,
//! and the chained one, single years, where it does not; the other is printed
//! beneath, not counted.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::facts::Kind;
use crate::sweep::{STAR_RUNS, Side, Stat, Sweep, Then, fmt_fact, layout, stat};

/// The twelve facts, in the order the table prints them.
pub const FACTS: [&str; 12] = [
    "pop_1800",
    "pop_1850",
    "pop_1900",
    "real_wage_chained_1850_vs_1750",
    "real_wage_chained_1900_vs_1750",
    "towns_1900",
    "urban_share_1900",
    "rail_km_1900",
    "rail_first_year",
    "income_share_work_1850",
    "income_share_work_1900",
    "income_share_land_1900",
];

/// Shown beneath the twelve and not counted.
pub const ALSO: [&str; 1] = ["last_town_founded"];

/// Where one fact of a later arm stands against the reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Inside,
    Moved,
    Outside,
    /// One of the two arms has no value.
    Missing,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Inside => "inside",
            Verdict::Moved => "moved",
            Verdict::Outside => "OUTSIDE",
            Verdict::Missing => "-",
        }
    }
}

/// The later arm's values of a fact against the reference arm's.
pub fn verdict(reference: Option<Stat>, later: Option<Stat>) -> Verdict {
    let (Some(r), Some(l)) = (reference, later) else { return Verdict::Missing };
    if r.n >= STAR_RUNS && l.n >= STAR_RUNS && (l.min > r.max || l.max < r.min) {
        Verdict::Moved
    } else if l.mean < r.min || l.mean > r.max {
        Verdict::Outside
    } else {
        Verdict::Inside
    }
}

/// Each setting of a finished sweep as an arm, as `sweep::read_then` reads an
/// earlier sweep's.
pub fn arms(sw: &Sweep) -> Vec<Then> {
    let mut out: Vec<Then> = sw.settings.iter().map(|s| Then { label: s.label.clone(), start: s.start.to_string(), ..Then::default() }).collect();
    for run in &sw.runs {
        let Ok(done) = &run.outcome else { continue };
        let arm = &mut out[run.setting];
        arm.seeds.push(run.seed);
        for f in &done.facts.facts {
            if let Some(v) = f.value {
                arm.values.entry(f.name.clone()).or_default().push((run.seed, v));
            }
        }
    }
    out
}

/// The check's text: a first line for each later arm, "standing check: N of 12
/// inside", then the table. `kinds` says how each fact prints; `whence` names
/// the reference ("the first setting", or the earlier sweep's directory).
pub fn check(reference: &Then, later: &[&Then], kinds: &HashMap<String, Kind>, whence: &str) -> String {
    let st = |arm: &Then, name: &str| arm.values.get(name).and_then(|v| stat(v));
    let mut out = String::new();
    for arm in later {
        let v: Vec<Verdict> = FACTS.iter().map(|f| verdict(st(reference, f), st(arm, f))).collect();
        let count = |w: Verdict| v.iter().filter(|&&x| x == w).count();
        let _ = writeln!(
            out,
            "standing check: {} of {} inside ({} moved, {} OUTSIDE, {} without a value): {} against {}",
            count(Verdict::Inside),
            FACTS.len(),
            count(Verdict::Moved),
            count(Verdict::Outside),
            count(Verdict::Missing),
            arm.label,
            reference.label
        );
    }
    let _ = writeln!(
        out,
        "reference: {} ({} runs) from {whence}\n\
         inside: the mean lies within the reference's [least .. greatest] across seeds; moved: every seed of one\n\
         arm lies on one side of every seed of the other ({STAR_RUNS} runs or more a side); OUTSIDE: the mean has left\n\
         the spread and the seeds have not stood apart. Only inside passes.\n",
        reference.label,
        reference.seeds.len()
    );
    let mut head = vec![String::new(), reference.label.clone(), String::new()];
    let mut sides = vec![Side::Left, Side::Right, Side::Left];
    for arm in later {
        head.extend([arm.label.clone(), String::new()]);
        sides.extend([Side::Right, Side::Left]);
    }
    let mut rows: Vec<(Option<&'static str>, Vec<String>)> = Vec::new();
    for (heading, names) in [("the twelve", &FACTS[..]), ("also", &ALSO[..])] {
        rows.push((Some(heading), Vec::new()));
        for &name in names {
            let kind = kinds.get(name).copied().unwrap_or(Kind::Real);
            let r = st(reference, name);
            let mut cells = vec![name.to_string()];
            match r {
                Some(s) => cells.extend([fmt_fact(kind, s.mean), format!("[{} .. {}]", fmt_fact(kind, s.min), fmt_fact(kind, s.max))]),
                None => cells.extend(["-".to_string(), String::new()]),
            }
            for arm in later {
                let l = st(arm, name);
                cells.push(l.map_or("-".to_string(), |s| fmt_fact(kind, s.mean)));
                cells.push(verdict(r, l).as_str().to_string());
            }
            rows.push((None, cells));
        }
    }
    out.push_str(&layout(&head, &sides, &rows));
    out
}

/// One line for each of the twelve that is not inside, in each later arm: the
/// fact, the arm, its mean and word, the band, and the islands outside the band
/// with their values. Empty when every fact of every arm is inside.
pub fn outside(reference: &Then, later: &[&Then], kinds: &HashMap<String, Kind>) -> Vec<String> {
    let st = |arm: &Then, name: &str| arm.values.get(name).and_then(|v| stat(v));
    let mut out = Vec::new();
    for arm in later {
        for &name in &FACTS {
            let (r, l) = (st(reference, name), st(arm, name));
            let v = verdict(r, l);
            if v == Verdict::Inside {
                continue;
            }
            let kind = kinds.get(name).copied().unwrap_or(Kind::Real);
            let f = |x: f64| fmt_fact(kind, x);
            let (Some(r), Some(l)) = (r, l) else {
                let whose = if r.is_none() { format!("the reference ({})", reference.label) } else { arm.label.clone() };
                out.push(format!("{name}, {}: no value in {whose}, so it cannot be set against its band", arm.label));
                continue;
            };
            let mut islands: Vec<(u64, f64)> = arm.values[name].iter().copied().filter(|&(_, x)| x < r.min || x > r.max).collect();
            islands.sort_by_key(|&(seed, _)| seed);
            let list: Vec<String> = islands.iter().map(|&(seed, x)| format!("island {seed} at {}", f(x))).collect();
            let how = if v == Verdict::Moved { "moved: every island apart from the reference's" } else { "OUTSIDE: the islands overlap the reference's" };
            out.push(format!(
                "{name}, {}: mean {} is outside the band [{} .. {}] of {} across its {} islands ({how}); outside the band: {}",
                arm.label,
                f(l.mean),
                f(r.min),
                f(r.max),
                reference.label,
                r.n,
                if list.is_empty() { "no island alone (only the mean)".to_string() } else { list.join(", ") }
            ));
        }
    }
    out
}

/// What the design asks of the century after 1900, each read from facts every
/// run to 2050 has, island by island: (what is asked, what is read, the rule).
/// The wage is the one `wage_read` names: the linked real wage
/// (`real_wage_linked`, linked.rs, docs/rules/measures.md), whose facts are the
/// mean of the nine years round each mark against 1750, where the run keeps it
/// (the game's rules since 20 November 2026, when the test of the measures found
/// it inside the band of two fixed baskets on every span and on every island, and
/// the island met all three under it); else the chained real wage
/// (`real_wage_chained`, engel.rs), whose facts are single years.
pub const GROWTH: [(&str, &str); 3] = [
    ("the real wage grows 1 per cent a year or more from 1900 to 1970", "per cent a year, 1900 to 1970"),
    ("the real wage is no lower in 2050 than in 1970", "2050 over 1970"),
    ("the share of hands on the land is lower in 2050 than in 1900", "2050 less 1900"),
];

/// The two wages the growth may be read on, the one read first: (the facts'
/// stem, its words).
pub const WAGES: [(&str, &str); 2] = [
    ("real_wage_linked", "the linked real wage, nine-year means (linked.rs)"),
    ("real_wage_chained", "the chained real wage, single years (engel.rs)"),
];

/// Which wage the growth reads for an arm: the first of `WAGES` it has the
/// facts of (the chained one, by default, so that an arm with neither says so).
pub fn wage_read(arm: &Then) -> usize {
    WAGES.iter().position(|(stem, _)| arm.values.contains_key(&format!("{stem}_1970_vs_1750"))).unwrap_or(1)
}

/// The least growth a year, per cent, the design asks of the chained wage from 1900 to 1970.
pub const WAGE_GROWTH_PCT: f64 = 1.0;

/// Each island's value of each of `GROWTH`'s three, from its facts: the wage's
/// growth from 1900 to 1970 in per cent a year, its level in 2050 over 1970, and
/// the share of hands on the land (`farm_share`) in 2050 less 1900. An island
/// lacking either of a pair's facts (a run that ends before 2050) is left out.
/// The wage is the one `wage_read` names.
pub fn growth_values(arm: &Then) -> [Vec<(u64, f64)>; 3] {
    growth_values_on(arm, WAGES[wage_read(arm)].0)
}

/// `growth_values` on the wage whose facts begin with `stem`.
pub fn growth_values_on(arm: &Then, stem: &str) -> [Vec<(u64, f64)>; 3] {
    let pair = |a: &str, b: &str, f: &dyn Fn(f64, f64) -> f64| -> Vec<(u64, f64)> {
        let (Some(a), Some(b)) = (arm.values.get(a), arm.values.get(b)) else { return Vec::new() };
        let mut out: Vec<(u64, f64)> =
            a.iter().filter_map(|&(seed, x)| b.iter().find(|&&(s, _)| s == seed).map(|&(_, y)| (seed, f(x, y)))).filter(|(_, v)| v.is_finite()).collect();
        out.sort_by_key(|&(seed, _)| seed);
        out
    };
    [
        pair(&format!("{stem}_1900_vs_1750"), &format!("{stem}_1970_vs_1750"), &|w00, w70| 100.0 * ((w70 / w00).powf(1.0 / 70.0) - 1.0)),
        pair(&format!("{stem}_1970_vs_1750"), &format!("{stem}_2050_vs_1750"), &|w70, w50| w50 / w70),
        pair("farm_share_1900", "farm_share_2050", &|f00, f50| f50 - f00),
    ]
}

/// Whether the mean of each of `GROWTH`'s three across the islands meets its
/// rule; None where no island has a value.
pub fn growth_met(values: &[Vec<(u64, f64)>; 3]) -> [Option<bool>; 3] {
    std::array::from_fn(|k| {
        stat(&values[k]).map(|s| match k {
            0 => s.mean >= WAGE_GROWTH_PCT,
            1 => s.mean >= 1.0,
            _ => s.mean < 0.0,
        })
    })
}

/// The growth after 1900 for each arm: the text, and a line for each of the two
/// growth facts an arm missed (the wage's two parts make one fact).
pub fn growth(arms: &[&Then]) -> (String, Vec<String>) {
    let mut text = String::new();
    let mut missed = Vec::new();
    for arm in arms {
        let values = growth_values(arm);
        let met = growth_met(&values);
        // the two facts: the wage's two parts together, and the land's
        let facts = [(met[0].zip(met[1]).map(|(a, b)| a && b), "the real wage grows 1 per cent a year or more from 1900 to 1970, and is no lower in 2050 than in 1970"),
                     (met[2], "the share of hands on the land is lower in 2050 than in 1900")];
        let on = wage_read(arm);
        let word = |m: Option<bool>| match m {
            Some(true) => "met",
            Some(false) => "MISSED",
            None => "-",
        };
        let _ = writeln!(
            text,
            "growth after 1900: {} of 2 met: {} (reported, not counted; with --growth a fact missed or without a value fails the sweep)\n  the wage read: {}",
            facts.iter().filter(|(m, _)| *m == Some(true)).count(),
            arm.label,
            WAGES[on].1
        );
        let rules = ["1.00 or more", "1.00 or more", "below 0"];
        for (k, (asked, read)) in GROWTH.iter().enumerate() {
            let s = stat(&values[k]);
            let shown = |x: f64| if k == 2 { format!("{x:+.3}") } else { format!("{x:.2}") };
            let cells = match s {
                Some(s) => format!("{:>7}  [{} .. {}]", shown(s.mean), shown(s.min), shown(s.max)),
                None => format!("{:>7}  (no island has the facts: the runs must reach 2050)", "-"),
            };
            let islands = values[k].iter().map(|&(seed, x)| format!("{seed}: {}", shown(x))).collect::<Vec<_>>().join(", ");
            let _ = writeln!(text, "  {asked}\n    {read:<30} {cells}   needs {:<13} {}{}", rules[k], word(met[k]), if islands.is_empty() { String::new() } else { format!("\n    island by island: {islands}") });
        }
        // the other wage, beside it and not counted
        let other = 1 - on;
        let ov = growth_values_on(arm, WAGES[other].0);
        if let (Some(g), Some(l)) = (stat(&ov[0]), stat(&ov[1])) {
            let _ = writeln!(
                text,
                "  beside it, not read: {}: {:.2} per cent a year 1900 to 1970 [{:.2} .. {:.2}], 2050 over 1970 {:.2} [{:.2} .. {:.2}]",
                WAGES[other].1, g.mean, g.min, g.max, l.mean, l.min, l.max
            );
        }
        for (m, what) in facts {
            if m != Some(true) {
                let why = match m {
                    Some(_) => "missed".to_string(),
                    None => "no value: the runs do not reach 2050".to_string(),
                };
                missed.push(format!("{what}, {}: {why}", arm.label));
            }
        }
        text.push('\n');
    }
    (text, missed)
}

// ---------------------------------------------------------------- the colony's check

/// How a fact of the colony's check is read from an arm, island by island.
#[derive(Clone, Copy, Debug)]
pub enum Read {
    /// A fact as it is.
    Fact(&'static str),
    /// Per cent a year from the first fact to the second, over so many years.
    Growth(&'static str, &'static str, f64),
    /// Per cent a year of a fact that is a level against 1.0 so many years before.
    Rate(&'static str, f64),
    /// The second fact less the first.
    Change(&'static str, &'static str),
    /// The fact over the settled island's on the same island number.
    OverSettled(&'static str),
    /// The fact less the settled island's on the same island number.
    LessSettled(&'static str),
}

/// One fact of the colony's check: what is asked, how it is read, its band
/// (either end may be open), how sure the band is, and its number in the table
/// of history it comes from (`docs/rules/scorecard.md`, "The colony's check").
#[derive(Clone, Copy, Debug)]
pub struct ColonyFact {
    pub asked: &'static str,
    pub read: Read,
    pub kind: Kind,
    pub lo: Option<f64>,
    pub hi: Option<f64>,
    /// `read` (read in its source), `memory` (from memory) or `reasoned` (no
    /// source; reasoned from the island).
    pub sure: &'static str,
    /// The row of the table of history on the scorecard's page.
    pub row: &'static str,
}

const fn fact(asked: &'static str, read: Read, kind: Kind, lo: Option<f64>, hi: Option<f64>, sure: &'static str, row: &'static str) -> ColonyFact {
    ColonyFact { asked, read, kind, lo, hi, sure, row }
}

/// The colony's check: what a colony of settlement did, set beside the landfall
/// start. Written from history first (the study of 26 November 2026, its table
/// of 24 facts, and the earlier proposals of docs/rules/landfall.md), not fitted
/// to the runs: a fact today's colony misses stays here as a miss, the list of
/// what this start must still learn. The facts struck, and why, are in
/// `STRUCK`.
pub const COLONY: [ColonyFact; 24] = [
    fact("births less deaths, per cent a year, 1751-1800", Read::Fact("natural_increase_pct_1751_1800"), Kind::Ratio, Some(2.3), Some(3.0), "read", "1"),
    fact("people grow, births and ships, per cent a year, 1750-1800", Read::Growth("pop_1750", "pop_1800", 50.0), Kind::Ratio, Some(3.5), Some(8.0), "memory", "2"),
    fact("people in 1900, year 150", Read::Fact("pop_1900"), Kind::Count, Some(100_000.0), Some(500_000.0), "memory", "3"),
    fact("births less deaths as the land fills, per cent a year, 1851-1880", Read::Fact("natural_increase_pct_1851_1880"), Kind::Ratio, Some(1.5), Some(2.0), "memory", "4"),
    fact("people grow once the land is full, per cent a year, 1950-2050", Read::Growth("pop_1950", "pop_2050", 100.0), Kind::Ratio, Some(0.0), Some(0.5), "memory", "5"),
    fact("settlers' share of the growth, 1750-1770", Read::Fact("settlers_share_of_growth_1750_1770"), Kind::Share, Some(0.5), Some(0.9), "memory", "6"),
    fact("settlers' share of the growth, 1750-1850", Read::Fact("settlers_share_of_growth_1750_1850"), Kind::Share, Some(0.2), Some(0.6), "read", "7"),
    fact("under 15, share of people, 1800", Read::Fact("ages_under15_1800"), Kind::Share, Some(0.33), Some(0.45), "memory", "10"),
    fact("under 15, share of people, 1850", Read::Fact("ages_under15_1850"), Kind::Share, Some(0.33), Some(0.45), "memory", "10"),
    fact("under 15, share of people, 1900", Read::Fact("ages_under15_1900"), Kind::Share, Some(0.32), Some(0.40), "memory", "11"),
    fact("second town founded, year", Read::Fact("town_second_year"), Kind::Year, Some(1750.0), Some(1755.0), "memory", "12"),
    fact("share in towns, 1850 less 1750", Read::Change("urban_share_1750", "urban_share_1850"), Kind::Share, Some(-0.05), Some(0.05), "read", "15"),
    fact("years with hunger, 1770-1900", Read::Fact("famine_years_1770_1900"), Kind::Count, Some(0.0), Some(0.0), "memory", "16"),
    fact("years with hunger, 1901-2050", Read::Fact("famine_years_1901_2050"), Kind::Count, Some(0.0), Some(0.0), "memory", "16"),
    fact("wage in grain over the settled island's, 1760", Read::OverSettled("wage_in_grain_1760"), Kind::Ratio, Some(0.6), Some(1.1), "read", "18"),
    fact("wage in grain over the settled island's, 1800", Read::OverSettled("wage_in_grain_1800"), Kind::Ratio, Some(1.1), Some(1.7), "read", "17"),
    fact("wage in grain over the settled island's, 1850", Read::OverSettled("wage_in_grain_1850"), Kind::Ratio, Some(1.1), Some(1.7), "read", "17"),
    fact("wage in grain over the settled island's, 1900", Read::OverSettled("wage_in_grain_1900"), Kind::Ratio, Some(1.2), Some(2.4), "read", "a"),
    fact("the real wage's growth while land feeds it, per cent a year, 1750-1850", Read::Rate("real_wage_chained_1850_vs_1750", 100.0), Kind::Ratio, Some(0.0), Some(0.5), "read", "19"),
    fact("the cost of living over three years at most, to 1900", Read::Fact("price_level_3y_max_1750_1900"), Kind::Ratio, None, Some(1.5), "memory", "21"),
    fact("the cost of living over three years at least, to 1900", Read::Fact("price_level_3y_min_1750_1900"), Kind::Ratio, Some(0.7), None, "memory", "21"),
    fact("first railway, years after the settled island's", Read::LessSettled("rail_first_year"), Kind::Real, Some(20.0), Some(35.0), "memory", "22"),
    fact("people in 2050 over the settled island's", Read::OverSettled("pop_2050"), Kind::Ratio, Some(0.5), Some(1.5), "reasoned", "-"),
    fact("towns in 1900 over the settled island's", Read::OverSettled("towns_1900"), Kind::Ratio, Some(0.5), Some(1.5), "reasoned", "-"),
];

/// Held to both starts and shown beside the check, not counted: the largest
/// rise of a town's wage over the year before, to 1900 (history: between 0.75
/// and 1.33 of the year before, outside a gold rush or war paper; row 20). The
/// settled island has it too (3.28 on island 8 in 1786), so it is no fact of
/// one start alone.
pub const BOTH: [ColonyFact; 1] =
    [fact("a town's wage over the year before at most, to 1900", Read::Fact("town_wage_rise_max_1752_1900"), Kind::Ratio, None, Some(1.33), "memory", "20")];

/// The facts of the table of history not in the check, with the reason.
pub const STRUCK: [(&str, &str); 6] = [
    ("8", "settlers in a year, of the colony's people: the immigration rule's own caps (0.015 and 0.05) hold it, so it checks a number of the rule, not the run"),
    ("9", "under 15 with births alone, 0.42 to 0.50: the landfall start brings settlers, so rows 10 and 11, with ships, are read instead"),
    ("13", "new towns founded at a few hundred to 2,000 people: the hamlets' rule's own number, 500"),
    ("14", "share in towns of 2,500 and more: the game's urban_share is the share living on town ground, another measure; row 15 reads its change, which both measures share"),
    ("23", "a machine that saves hands on land no later than on the settled island: no fact yet says when the first one runs (not measured, kept for the fact that will)"),
    ("24", "runs failing their money check: the sweep fails any such run itself"),
];

/// A fact's values island by island, from the colony's arm and, where it reads
/// one, the settled island's arm of the same island numbers.
pub fn colony_values(f: &ColonyFact, arm: &Then, settled: Option<&Then>) -> Vec<(u64, f64)> {
    let get = |a: &Then, name: &str| -> Vec<(u64, f64)> { a.values.get(name).cloned().unwrap_or_default() };
    let pair = |a: &[(u64, f64)], b: &[(u64, f64)], f: &dyn Fn(f64, f64) -> f64| -> Vec<(u64, f64)> {
        let mut out: Vec<(u64, f64)> =
            a.iter().filter_map(|&(seed, x)| b.iter().find(|&&(s, _)| s == seed).map(|&(_, y)| (seed, f(x, y)))).filter(|(_, v)| v.is_finite()).collect();
        out.sort_by_key(|&(seed, _)| seed);
        out
    };
    let mut v = match f.read {
        Read::Fact(name) => get(arm, name),
        Read::Growth(a, b, years) => pair(&get(arm, a), &get(arm, b), &|x, y| 100.0 * ((y / x).powf(1.0 / years) - 1.0)),
        Read::Rate(name, years) => get(arm, name).into_iter().map(|(s, x)| (s, 100.0 * (x.powf(1.0 / years) - 1.0))).collect(),
        Read::Change(a, b) => pair(&get(arm, a), &get(arm, b), &|x, y| y - x),
        Read::OverSettled(name) => settled.map_or(Vec::new(), |s| pair(&get(arm, name), &get(s, name), &|x, y| x / y)),
        Read::LessSettled(name) => settled.map_or(Vec::new(), |s| pair(&get(arm, name), &get(s, name), &|x, y| x - y)),
    };
    v.retain(|(_, x)| x.is_finite());
    v.sort_by_key(|&(seed, _)| seed);
    v
}

/// Whether a number lies in a fact's band.
pub fn in_band(f: &ColonyFact, x: f64) -> bool {
    f.lo.is_none_or(|lo| x >= lo - 1e-12) && f.hi.is_none_or(|hi| x <= hi + 1e-12)
}

/// Whether the mean across the islands lies in the band; None with no value.
pub fn colony_met(f: &ColonyFact, values: &[(u64, f64)]) -> Option<bool> {
    stat(values).map(|s| in_band(f, s.mean))
}

/// The band in words.
fn band_words(f: &ColonyFact) -> String {
    let show = |x: f64| fmt_fact(f.kind, x);
    match (f.lo, f.hi) {
        (Some(a), Some(b)) if a == b => show(a),
        (Some(a), Some(b)) => format!("{} to {}", show(a), show(b)),
        (Some(a), None) => format!("{} or more", show(a)),
        (None, Some(b)) => format!("{} or less", show(b)),
        (None, None) => "any".to_string(),
    }
}

/// The colony's check for each of `colonies`, set beside `settled` (the settled
/// island's arm of the same islands, named by `whence`) where a fact reads it:
/// the text, and a line for each fact missed or without a value.
pub fn colony(colonies: &[&Then], settled: Option<&Then>, whence: &str) -> (String, Vec<String>) {
    let mut text = String::new();
    let mut missed = Vec::new();
    for arm in colonies {
        let rows: Vec<(ColonyFact, Vec<(u64, f64)>, Option<bool>)> =
            COLONY.iter().map(|f| (*f, colony_values(f, arm, settled), colony_met(f, &colony_values(f, arm, settled)))).collect();
        let met = rows.iter().filter(|r| r.2 == Some(true)).count();
        let none = rows.iter().filter(|r| r.2.is_none()).count();
        let _ = writeln!(
            text,
            "the colony's check: {met} of {} met, {} missed, {none} without a value: {} ({} islands){}",
            COLONY.len(),
            COLONY.len() - met - none,
            arm.label,
            arm.seeds.len(),
            match settled {
                Some(s) => format!(", beside {} from {whence}", s.label),
                None => ", with no settled island beside it (--beside DIR, or --start settled,landfall in one sweep)".to_string(),
            }
        );
        let _ = writeln!(
            text,
            "bands from history (docs/rules/scorecard.md, \"The colony's check\": row, source, how sure); met: the mean of\n\
             the islands lies in the band. Printed, not counted, unless --colony.\n"
        );
        let head: Vec<String> = ["", "mean", "[least .. greatest]", "band", "islands in", "", "sure", "row"].iter().map(|s| s.to_string()).collect();
        let sides = [Side::Left, Side::Right, Side::Left, Side::Left, Side::Right, Side::Left, Side::Left, Side::Left];
        let mut table: Vec<(Option<&'static str>, Vec<String>)> = vec![(Some("the colony's facts"), Vec::new())];
        let row = |f: &ColonyFact, v: &[(u64, f64)], word: &str| -> Vec<String> {
            let show = |x: f64| fmt_fact(f.kind, x);
            let s = stat(v);
            vec![
                f.asked.to_string(),
                s.map_or("-".to_string(), |s| show(s.mean)),
                s.map_or(String::new(), |s| format!("[{} .. {}]", show(s.min), show(s.max))),
                band_words(f),
                if v.is_empty() { String::new() } else { format!("{} of {}", v.iter().filter(|&&(_, x)| in_band(f, x)).count(), v.len()) },
                word.to_string(),
                f.sure.to_string(),
                f.row.to_string(),
            ]
        };
        for (f, v, m) in &rows {
            let word = match m {
                Some(true) => "met",
                Some(false) => "MISSED",
                None => "-",
            };
            table.push((None, row(f, v, word)));
            if *m != Some(true) {
                let why = match m {
                    Some(false) => format!("mean {} outside {}", fmt_fact(f.kind, stat(v).map_or(f64::NAN, |s| s.mean)), band_words(f)),
                    _ => "no value".to_string(),
                };
                missed.push(format!("{}, {}: {why}", f.asked, arm.label));
            }
        }
        // held to both starts: the colony's number, and the settled island's beside it
        table.push((Some("held to both starts, not counted"), Vec::new()));
        for f in &BOTH {
            let v = colony_values(f, arm, None);
            table.push((None, row(f, &v, if colony_met(f, &v) == Some(true) { "met" } else { "missed" })));
            if let Some(s) = settled {
                let w = colony_values(f, s, None);
                let mut cells = row(f, &w, if colony_met(f, &w) == Some(true) { "met" } else { "missed" });
                cells[0] = "  the same, the settled island".to_string();
                table.push((None, cells));
            }
        }
        text.push_str(&layout(&head, &sides, &table));
        let _ = writeln!(text, "struck from the table of history, with the reason:");
        for (row, why) in STRUCK {
            let _ = writeln!(text, "  row {row}: {why}");
        }
        text.push('\n');
    }
    (text, missed)
}

/// What the check makes of a finished sweep: the text it prints and writes, the
/// lines that fail the sweep (any of the twelve not inside), the growth facts
/// missed (which fail it only with `--growth`) and the colony's facts missed
/// (which fail it only with `--colony`).
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub text: String,
    pub outside: Vec<String>,
    pub growth_missed: Vec<String>,
    pub colony_missed: Vec<String>,
}

/// Whether an arm is of the landfall start.
fn is_colony(arm: &Then) -> bool {
    arm.start == "landfall"
}

/// The twelve's reference and later arms among the arms of one start: the
/// first against each later one, or, with one, the earlier sweep's setting of
/// the same start and name (or its first of that start) against it.
fn twelve_of<'a>(arms: &[&'a Then], then: Option<(&'a [Then], &std::path::Path)>, colony: bool) -> Option<(&'a Then, Vec<&'a Then>, String)> {
    if arms.len() >= 2 {
        return Some((arms[0], arms[1..].to_vec(), "this sweep's first setting".to_string()));
    }
    let (arm, (then, dir)) = (arms.first()?, then?);
    let pool: Vec<&Then> = then.iter().filter(|t| is_colony(t) == colony).collect();
    pool.iter().find(|t| t.label == arm.label).or(pool.first()).map(|r| (*r, vec![*arm], format!("the earlier sweep in {}", dir.display())))
}

/// The check for a finished sweep. The settings of each start take the twelve
/// among themselves (`twelve_of`): a start is never set against the other,
/// since one town in a corner stands nowhere near six towns in 1900. The
/// landfall start's settings also take the colony's check, set beside the
/// settled island's first setting in this sweep or, failing that, the first in
/// `beside`. Every setting takes the growth after 1900. None when there is
/// nothing to set the twelve against and no colony.
pub fn for_sweep(sw: &Sweep, then: Option<(&[Then], &std::path::Path)>, beside: Option<(&[Then], &std::path::Path)>) -> Option<Report> {
    let kinds: HashMap<String, Kind> = sw.fact_index.iter().map(|(n, _, k)| (n.clone(), *k)).collect();
    let now = arms(sw);
    let settled: Vec<&Then> = now.iter().filter(|a| !is_colony(a)).collect();
    let colonies: Vec<&Then> = now.iter().filter(|a| is_colony(a)).collect();
    let twelves: Vec<(&Then, Vec<&Then>, String)> = [twelve_of(&settled, then, false), twelve_of(&colonies, then, true)].into_iter().flatten().collect();
    if twelves.is_empty() && colonies.is_empty() {
        return None;
    }
    let mut text = String::new();
    let mut outside_lines = Vec::new();
    for (reference, later, whence) in &twelves {
        text.push_str(&check(reference, later, &kinds, whence));
        let lines = outside(reference, later, &kinds);
        if !lines.is_empty() {
            let _ = writeln!(text, "\nnot inside, so the sweep fails:");
            for line in &lines {
                let _ = writeln!(text, "  {line}");
            }
        }
        outside_lines.extend(lines);
        text.push('\n');
    }
    let mut colony_missed = Vec::new();
    if !colonies.is_empty() {
        let (beside_arm, whence) = match (settled.first(), beside) {
            (Some(s), _) => (Some(*s), "this sweep".to_string()),
            (None, Some((b, dir))) => (b.iter().find(|t| !is_colony(t)), format!("the sweep in {}", dir.display())),
            (None, None) => (None, String::new()),
        };
        let (colony_text, missed) = colony(&colonies, beside_arm, &whence);
        text.push_str(&colony_text);
        colony_missed = missed;
    }
    let (growth_text, growth_missed) = growth(&now.iter().collect::<Vec<_>>());
    text.push_str(&growth_text);
    Some(Report { text, outside: outside_lines, growth_missed, colony_missed })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A made-up pair of arms on four seeds: one fact inside, one moved, one
    /// OUTSIDE, the rest without a value.
    #[test]
    fn inside_moved_and_outside() {
        let arm = |label: &str, facts: &[(&str, [f64; 4])]| {
            let mut t = Then { label: label.to_string(), seeds: vec![1, 2, 3, 4], ..Then::default() };
            for (name, v) in facts {
                t.values.insert(name.to_string(), (1..=4u64).zip(v.iter().copied()).collect());
            }
            t
        };
        let reference = arm(
            "off",
            &[("pop_1900", [700_000.0, 720_000.0, 740_000.0, 760_000.0]), ("towns_1900", [14.0, 17.0, 18.0, 20.0]), ("rail_km_1900", [2_100.0, 2_200.0, 2_400.0, 2_500.0])],
        );
        let later = arm(
            "on",
            // inside; every seed above every reference seed; a mean below the least with the seeds overlapping
            &[("pop_1900", [710_000.0, 730_000.0, 735_000.0, 750_000.0]), ("towns_1900", [21.0, 22.0, 23.0, 25.0]), ("rail_km_1900", [1_500.0, 1_600.0, 1_700.0, 2_300.0])],
        );
        let st = |a: &Then, n: &str| a.values.get(n).and_then(|v| stat(v));
        assert_eq!(verdict(st(&reference, "pop_1900"), st(&later, "pop_1900")), Verdict::Inside);
        assert_eq!(verdict(st(&reference, "towns_1900"), st(&later, "towns_1900")), Verdict::Moved);
        assert_eq!(verdict(st(&reference, "rail_km_1900"), st(&later, "rail_km_1900")), Verdict::Outside);
        assert_eq!(verdict(st(&reference, "pop_1800"), st(&later, "pop_1800")), Verdict::Missing);
        let kinds: HashMap<String, Kind> =
            [("pop_1900", Kind::Count), ("towns_1900", Kind::Count), ("rail_km_1900", Kind::Count)].iter().map(|(n, k)| (n.to_string(), *k)).collect();
        let text = check(&reference, &[&later], &kinds, "by hand");
        println!("{text}");
        assert!(text.starts_with("standing check: 1 of 12 inside (1 moved, 1 OUTSIDE, 9 without a value): on against off\n"), "{text}");
        let line = |name: &str| text.lines().find(|l| l.trim_start().starts_with(name)).unwrap().to_string();
        assert!(line("pop_1900").ends_with("inside") && line("pop_1900").contains("[700,000 .. 760,000]"), "{}", line("pop_1900"));
        assert!(line("towns_1900").ends_with("moved"));
        assert!(line("rail_km_1900").ends_with("OUTSIDE"));
        assert!(text.contains("also\n  last_town_founded"));
        // what fails the sweep: the moved, the OUTSIDE and the nine without a value, each named
        // with its arm, its mean, the band and the islands outside the band
        let bad = outside(&reference, &[&later], &kinds);
        println!("{}", bad.join("\n"));
        assert_eq!(bad.len(), 11, "{bad:?}");
        assert!(bad.iter().all(|l| !l.starts_with("pop_1900")));
        let towns = bad.iter().find(|l| l.starts_with("towns_1900")).unwrap();
        assert!(towns.contains("mean 22.8 is outside the band [14 .. 20] of off across its 4 islands (moved") && towns.contains("island 1 at 21, island 2 at 22"), "{towns}");
        let rail = bad.iter().find(|l| l.starts_with("rail_km_1900")).unwrap();
        assert!(rail.contains("outside the band [2,100 .. 2,500] of off across its 4 islands (OUTSIDE") && rail.ends_with("island 1 at 1,500, island 2 at 1,600, island 3 at 1,700"), "{rail}");
        assert!(bad.iter().any(|l| l.starts_with("pop_1800, on: no value in the reference (off)")));
        // with every fact inside, nothing fails
        assert!(outside(&reference, &[&reference], &kinds).iter().all(|l| l.contains("no value")));
    }

    /// The colony's check on made-up arms of two islands: a fact read as it is,
    /// a growth, a ratio to the settled island of the same island number, a
    /// fact the arm lacks; and the settled island's number beside the fact held
    /// to both starts.
    #[test]
    fn the_colonys_check() {
        let arm = |label: &str, start: &str, facts: &[(&str, [f64; 2])]| {
            let mut t = Then { label: label.to_string(), start: start.to_string(), seeds: vec![1, 2], ..Then::default() };
            for (name, v) in facts {
                t.values.insert(name.to_string(), vec![(1, v[0]), (2, v[1])]);
            }
            t
        };
        let colony = arm(
            "landfall",
            "landfall",
            &[
                ("natural_increase_pct_1751_1800", [2.4, 2.8]),
                ("pop_1750", [5_000.0, 5_000.0]),
                ("pop_1800", [40_000.0, 50_000.0]),
                ("wage_in_grain_1900", [9.0, 10.0]),
                ("famine_years_1770_1900", [0.0, 2.0]),
                ("town_wage_rise_max_1752_1900", [1.2, 2.1]),
            ],
        );
        let settled = arm("settled", "settled", &[("wage_in_grain_1900", [36.0, 40.0]), ("town_wage_rise_max_1752_1900", [3.28, 1.1])]);
        let find = |asked: &str| COLONY.iter().find(|f| f.asked.starts_with(asked)).unwrap();
        // read as it is, and met
        let f = find("births less deaths, per cent a year, 1751-1800");
        assert_eq!(colony_met(f, &colony_values(f, &colony, None)), Some(true));
        // a growth: (40,000 / 5,000)^(1/50) - 1 is 4.25 per cent a year, inside 3.5 to 8
        let f = find("people grow, births and ships");
        let v = colony_values(f, &colony, None);
        assert!((v[0].1 - 100.0 * (8f64.powf(0.02) - 1.0)).abs() < 1e-9 && colony_met(f, &v) == Some(true), "{v:?}");
        // against the settled island island by island: 9/36 and 10/40 are a quarter, missed
        let f = find("wage in grain over the settled island's, 1900");
        let v = colony_values(f, &colony, Some(&settled));
        assert_eq!(v, vec![(1, 0.25), (2, 0.25)]);
        assert_eq!(colony_met(f, &v), Some(false));
        // without the settled island it has no value
        assert!(colony_values(f, &colony, None).is_empty() && colony_met(f, &[]).is_none());
        // a band of nothing at all: one hungry year on one island misses it
        let f = find("years with hunger, 1770-1900");
        assert_eq!(colony_met(f, &colony_values(f, &colony, None)), Some(false));
        let (text, missed) = super::colony(&[&colony], Some(&settled), "by hand");
        println!("{text}");
        assert!(text.starts_with("the colony's check: 2 of 24 met, 2 missed, 20 without a value: landfall (2 islands), beside settled from by hand"), "{text}");
        assert_eq!(missed.len(), 22);
        assert!(missed.iter().any(|m| m.starts_with("wage in grain over the settled island's, 1900, landfall: mean 0.25 outside 1.20 to 2.40")), "{missed:?}");
        // held to both starts, with the settled island's own number beside it, and not counted
        let line = |name: &str| text.lines().find(|l| l.trim_start().starts_with(name)).unwrap().to_string();
        assert!(line("a town's wage over the year before").contains("1.65") && line("a town's wage over the year before").contains("missed"));
        assert!(line("the same, the settled island").contains("2.19") && line("the same, the settled island").contains("1 of 2"));
        assert!(text.contains("row 8: settlers in a year"));
    }

    /// The growth after 1900, island by island, from the facts a run to 2050 has:
    /// a wage that grows 1.5 per cent a year to 1970 and then halves, and the land's
    /// share falling on one arm and rising on the other.
    #[test]
    fn growth_after_1900() {
        let arm = |label: &str, farm_2050: f64| {
            let mut t = Then { label: label.to_string(), seeds: vec![1, 2], ..Then::default() };
            let mut put = |name: &str, v: [f64; 2]| {
                t.values.insert(name.to_string(), vec![(1, v[0]), (2, v[1])]);
            };
            let w70 = 4.0 * 1.015f64.powi(70);
            put("real_wage_chained_1900_vs_1750", [4.0, 4.0]);
            put("real_wage_chained_1970_vs_1750", [w70, w70]);
            put("real_wage_chained_2050_vs_1750", [w70 / 2.0, w70 / 2.0]);
            put("farm_share_1900", [0.25, 0.25]);
            put("farm_share_2050", [farm_2050, farm_2050]);
            t
        };
        let (falls, rises) = (arm("falls", 0.10), arm("rises", 0.47));
        let v = growth_values(&falls);
        assert!((v[0][0].1 - 1.5).abs() < 1e-9 && (v[1][1].1 - 0.5).abs() < 1e-12 && (v[2][0].1 + 0.15).abs() < 1e-12, "{v:?}");
        assert_eq!(growth_met(&v), [Some(true), Some(false), Some(true)]);
        assert_eq!(growth_met(&growth_values(&rises)), [Some(true), Some(false), Some(false)]);
        let (text, missed) = growth(&[&falls, &rises]);
        println!("{text}");
        assert!(text.starts_with("growth after 1900: 1 of 2 met: falls"), "{text}");
        assert!(text.contains("growth after 1900: 0 of 2 met: rises"), "{text}");
        assert_eq!(missed.len(), 3, "{missed:?}");
        assert!(missed[0].starts_with("the real wage grows") && missed[0].ends_with("falls: missed"));
        // an arm that ends before 2050 has no value, which --growth counts as a miss
        let short = Then { label: "to 1900".to_string(), seeds: vec![1], ..Then::default() };
        assert_eq!(growth_met(&growth_values(&short)), [None, None, None]);
        let (_, missed) = growth(&[&short]);
        assert!(missed.len() == 2 && missed.iter().all(|m| m.ends_with("no value: the runs do not reach 2050")), "{missed:?}");
        // with the linked wage's facts the growth reads them, and prints the chained beneath
        let mut both = falls.clone();
        for (y, v) in [(1900, 4.0), (1970, 4.0 * 1.012f64.powi(70)), (2050, 4.0 * 1.012f64.powi(70) * 1.1)] {
            both.values.insert(format!("real_wage_linked_{y}_vs_1750"), vec![(1, v), (2, v)]);
        }
        assert_eq!(wage_read(&both), 0);
        assert_eq!(wage_read(&falls), 1);
        assert_eq!(growth_met(&growth_values(&both)), [Some(true), Some(true), Some(true)]);
        let (text, missed) = growth(&[&both]);
        println!("{text}");
        assert!(text.contains("the wage read: the linked real wage") && text.contains("beside it, not read: the chained real wage, single years (engel.rs): 1.50"), "{text}");
        assert!(missed.is_empty(), "{missed:?}");
    }
}
