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
    let mut out: Vec<Then> = sw.settings.iter().map(|s| Then { label: s.label.clone(), ..Then::default() }).collect();
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

/// The check for a finished sweep: its first setting against each later one,
/// or, with one setting, the earlier sweep `then` (read from `dir`) against it.
/// None when there is nothing to set it against.
pub fn for_sweep(sw: &Sweep, then: Option<(&[Then], &std::path::Path)>) -> Option<String> {
    let kinds: HashMap<String, Kind> = sw.fact_index.iter().map(|(n, _, k)| (n.clone(), *k)).collect();
    let now = arms(sw);
    if now.len() >= 2 {
        let later: Vec<&Then> = now[1..].iter().collect();
        return Some(check(&now[0], &later, &kinds, "this sweep's first setting"));
    }
    let (then, dir) = then?;
    let arm = now.first()?;
    let reference = then.iter().find(|t| t.label == arm.label).or(then.first())?;
    Some(check(reference, &[arm], &kinds, &format!("the earlier sweep in {}", dir.display())))
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
    }
}
