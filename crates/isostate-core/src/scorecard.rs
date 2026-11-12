//! A scorecard: one sweep read against the aims of the pass over the rules (R8). Not in the Python.
//!
//! ```text
//! isostate-core scorecard DIR [--spread-from OTHER_DIR]
//! ```
//!
//! reads the sweep in `DIR` (its `facts.json`) and prints, for each setting of
//! it, every aim with the fact that reads it: the mean across the seeds and the
//! least and greatest, the aim, its tolerance, how far the mean misses the aim in
//! tolerances, whether the aim is met, and on how many islands it is met one by
//! one. Then how many of the twenty aims are met, a single loss over them, which
//! a search could later make as small as it can, and the facts a scorecard should
//! be read with that have no aim. Every sweep also writes this card into its
//! directory as `scorecard.txt`.
//!
//! The aims and their tolerances are the twenty of the pass over the rules
//! (`docs/designs/rules-pass.md`, section 2, as its critic left them in section
//! 10), each tied to the document that asks for it (`AIMS`); nothing here invents
//! one. The tolerances are written beside the aims and held still, so a sweep is
//! not scored more kindly for its islands differing more. What each means and how
//! to read the card is in `docs/rules/scorecard.md`. There is no search yet.

use std::collections::{BTreeSet, HashMap};
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
    /// Between the two, both ends in; the same number twice is a point aimed at.
    Between(f64, f64),
}

impl Test {
    /// The number aimed at (the middle of a band).
    pub fn aim(self) -> f64 {
        match self {
            Test::AtLeast(a) | Test::AtMost(a) => a,
            Test::Between(lo, hi) => 0.5 * (lo + hi),
        }
    }

    /// How far a value falls on the wrong side of the aim; nought on the right side.
    pub fn miss(self, v: f64) -> f64 {
        match self {
            Test::AtLeast(a) => (a - v).max(0.0),
            Test::AtMost(a) => (v - a).max(0.0),
            Test::Between(lo, hi) => (lo - v).max(0.0) + (v - hi).max(0.0),
        }
    }

    fn text(self, kind: Kind) -> String {
        match self {
            Test::AtLeast(a) => format!("at least {}", fmt_fact(kind, a)),
            Test::AtMost(a) => format!("at most {}", fmt_fact(kind, a)),
            Test::Between(lo, hi) if lo == hi => format!("about {}", fmt_fact(kind, lo)),
            Test::Between(lo, hi) => format!("{} to {}", fmt_fact(kind, lo), fmt_fact(kind, hi)),
        }
    }
}

/// One aim, or one part of an aim that asks two things, the fact it is read by,
/// and where it is asked for.
#[derive(Clone, Copy, Debug)]
pub struct Aim {
    /// Its number in the design's table (docs/designs/rules-pass.md, section 2).
    pub number: u8,
    /// ' ' for an aim of one part, or 'a' and 'b' for the two parts of one.
    pub part: char,
    /// The fact the aim is judged on: a twenty-year mean where the aim is about a
    /// level by a year (R11), since a single year misleads.
    pub fact: &'static str,
    /// A fact printed beside it for the reader (the single year, or a fact that
    /// answers the aim's question another way), or "".
    pub beside: &'static str,
    /// What the mean of the islands should be, or, for an aim asked only of the
    /// islands one by one (`on_mean` false), what each island should be.
    pub test: Test,
    /// How far the mean may miss and still meet the aim, in the fact's own units, as
    /// the design writes it; nought where it says "none": then the mean must be on
    /// the aim's side.
    pub tolerance: f64,
    /// Whether the mean is judged. An aim asked only of the islands is not.
    pub on_mean: bool,
    /// What each island's own value should be, and on at least how many islands of
    /// sixteen (on fewer islands, as many in proportion, rounded up).
    pub islands: Option<(Test, usize)>,
    /// Whether it counts in the loss: every aim on the mean with a tolerance. The
    /// others have no scale to be counted in, and are met or missed.
    pub in_loss: bool,
    /// Who asks for it, in what words.
    pub stated: &'static str,
}

impl Aim {
    /// "6", or "12a".
    pub fn name(&self) -> String {
        format!("{}{}", self.number, self.part).trim_end().to_string()
    }
}

/// An aim on the mean with a tolerance, which is counted in the loss.
const fn mean(number: u8, part: char, fact: &'static str, beside: &'static str, test: Test, tolerance: f64, stated: &'static str) -> Aim {
    Aim { number, part, fact, beside, test, tolerance, on_mean: true, islands: None, in_loss: tolerance > 0.0, stated }
}

/// An aim asked of the islands one by one, which is met or missed.
const fn islands(number: u8, part: char, fact: &'static str, beside: &'static str, test: Test, of16: usize, stated: &'static str) -> Aim {
    Aim { number, part, fact, beside, test, tolerance: 0.0, on_mean: false, islands: Some((test, of16)), in_loss: false, stated }
}

/// An aim on the mean that also asks something of the islands one by one.
const fn both(number: u8, fact: &'static str, beside: &'static str, test: Test, tolerance: f64, each: Test, of16: usize, stated: &'static str) -> Aim {
    Aim { number, part: ' ', fact, beside, test, tolerance, on_mean: true, islands: Some((each, of16)), in_loss: tolerance > 0.0, stated }
}

/// How many aims the card has.
pub const AIM_COUNT: usize = 20;

/// The twenty aims of the pass over the rules, with their tolerances
/// (docs/designs/rules-pass.md, section 2, and section 10 for aims 6 and 15). An
/// aim of two parts is met when both are.
pub const AIMS: [Aim; 24] = [
    mean(1, ' ', "pop_1900_vs_1750", "pop_1900", Test::Between(5.0, 5.0), 0.5,
        "docs/rules/people.md: growth_top \"was set so that the island grows about fivefold\". History: England and Wales grew a little over fivefold."),
    mean(2, ' ', "urban_share_1751_1770", "urban_share_1750", Test::Between(0.25, 0.25), 0.05,
        "PLAN.md 1.2, the test of structural change: the urban share rises \"from about 0.25\"."),
    mean(3, ' ', "urban_share_1881_1900", "urban_share_1900", Test::AtLeast(0.5), 0.03,
        "PLAN.md 1.2: \"to 0.5 over the run in most seeds\"; README, Where it stands: \"The plan asks for a half.\""),
    mean(4, ' ', "urban_share_1881_1900_less_1861_1880", "", Test::AtLeast(0.02), 0.01,
        "README.md: the towns' share has been flat since about 1880. History: the towns' share never stopped rising."),
    mean(5, ' ', "real_wage_1881_1900_vs_1750", "real_wage_1900_vs_1750", Test::Between(2.5, 3.5), 0.15,
        "README.md's headline; PLAN.md 1.3 asks for \"the real wage's path\" with a tolerance and gives none. History: Britain's about 2.3 to 2.6 times. On the basket of 1750."),
    both(6, "spending_per_head_chained_1890s_vs_1870s", "spending_per_head_chained_peak_year", Test::Between(1.10, 1.40), 0.03, Test::AtLeast(1.10), 12,
        "README.md: \"Growth comes early and then stops.\" Spending a head over the chained cost of living (docs/probes/tr-works.md, section 1); the upper end is the critic's (docs/designs/rules-pass.md, section 10). History: Britain's output a head grew by about a fifth in those twenty years."),
    mean(7, ' ', "real_wage_five_worst_years_mean", "real_wage_worst_year", Test::AtLeast(-0.10), 0.015,
        "README.md: \"a building cycle nobody chose\" (docs/probes/tr-works.md, section 4). History: a dearth year took a tenth to a seventh off what a wage bought."),
    mean(8, ' ', "price_swing_timber_from_1800", "price_swing_timber", Test::AtMost(0.15), 0.02,
        "docs/rules/works.md, In company: the building cycle, whose mark is timber's price (docs/probes/tr-cycles.md)."),
    mean(9, ' ', "price_swing_iron", "", Test::AtMost(0.16), 0.02,
        "docs/rules/prices.md, Measuring it again: \"the iron price swings under 0.16 a year\"."),
    mean(10, ' ', "farm_zigzag_beyond_weather", "farm_zigzag", Test::AtMost(1.2), 0.25,
        "docs/rules/prices.md, Measuring it again: \"It has worked if the farm zigzag is under 1.2\", set before the harvests came in; moved to the zigzag the weather does not account for (docs/designs/rules-pass.md, section 2)."),
    mean(11, ' ', "price_tools_years_over_twice_local_median", "price_tools_years_over_twice_median", Test::AtMost(4.0), 2.0,
        "README.md: \"Tools run short\" (docs/rules/works.md, the famine of tools). Against the twenty-five years about each year, since with the production tree tools grow cheaper all century."),
    mean(12, 'a', "famine_years", "emigration_years", Test::AtMost(0.2), 0.2,
        "README.md; docs/rules/people.md. History: dearths, and no famine after the 1740s."),
    islands(12, 'b', "hungry_months_after_new_cells", "", Test::AtMost(0.0), 16,
        "none in the two months after a town takes a district: docs/probes/tr-carriage.md, (d), and docs/probes/tr-rent.md, sections 4 and 5."),
    islands(13, ' ', "town_months_wage_over_ten_middle", "town_wage_greatest_vs_middle", Test::AtMost(0.0), 16,
        "no town-month in sixteen runs: docs/rules/mines.md, machines.md and prices.md, each among its worries; docs/probes/tr-rent.md, section 1."),
    both(14, "rail_first_year", "discovered_railway", Test::Between(1826.0, 1826.0), 2.0, Test::Between(1820.0, 1832.0), 14,
        "README.md: \"anywhere from 1809 to 1842\" (docs/probes/tr-steel.md). History: 1825 and 1830."),
    both(15, "rail_km_1850", "rail_km_1870", Test::Between(1200.0, 1700.0), 100.0, Test::Between(1200.0, 1700.0), 14,
        "PLAN.md 1.2: the network \"keeps growing after the first rush\"; the band is docs/probes/tr-steel.md's trial at power 12 and rate 0.5, on fourteen islands of sixteen as the critic has it (docs/designs/rules-pass.md, section 10)."),
    islands(15, 'b', "rail_last_growth_year", "", Test::AtLeast(1890.0), 16,
        "PLAN.md 1.2: the network \"keeps growing after the first rush\": still growing in the 1890s on every island."),
    islands(16, ' ', "share_bessemer_1880", "", Test::AtLeast(0.9), 16,
        "docs/rules/machines.md: converter steel's share of steel in 1880, on every island."),
    islands(17, ' ', "share_steel_rolling_1890", "", Test::AtLeast(0.5), 14,
        "README.md: \"The steel rail hardly arrives\". History: nearly all new rail by 1890. The pass expects to miss it with the tree in (docs/designs/rules-pass.md, section 4.9)."),
    islands(18, 'a', "last_town_founded", "", Test::AtLeast(1861.0), 12,
        "docs/rules/people.md: towns \"go on being founded where a market would save the most carting\": the last after 1860."),
    mean(18, 'b', "towns_1900", "late_towns", Test::Between(18.0, 26.0), 2.0,
        "docs/rules/people.md, with docs/designs/rules-pass.md, section 4.7: 18 to 26 towns in 1900."),
    mean(19, 'a', "carters_between_towns_share_of_hands_1750", "carters_share_of_hands_1750", Test::Between(0.02, 0.02), 0.01,
        "History only: carriage was a growing trade all century (docs/designs/rules-pass.md, section 4.6): about 2 per cent of hands in 1750."),
    mean(19, 'b', "carters_between_towns_share_of_hands_1900", "carters_share_of_hands_1900", Test::AtLeast(0.05), 0.01,
        "History only: about one worker in eleven in all of transport by the census of 1901 (docs/designs/rules-pass.md, section 4.6): 5 per cent or more of hands by 1900."),
    mean(20, ' ', "years_to_half_coke_furnace", "discovered_coke_furnace", Test::AtMost(10.0), 0.0,
        "docs/rules/prices.md, Measuring it again: \"coke iron passes half of output within 10 years of its discovery\"."),
];

/// Two of PLAN.md 1.3's six facts that the plan gives no number and the pass no
/// aim (docs/designs/rules-pass.md, section 2): read beside the card with every stage.
pub const BESIDE_CARD: [&str; 2] = ["share_textile_mill_1830", "rail_km_1870"];

/// Aims the documents state that no fact reads, or state with no number.
pub const UNREAD: [&str; 5] = [
    "iron output swinging under 0.09 a year in every half-century (docs/rules/prices.md, Measuring it again): no fact gives iron output's swing by half-century",
    "iron stock near 2 months with under 4 per cent of orders unfilled (docs/rules/prices.md, Measuring it again): no fact gives either",
    "iron and rails no longer cycling with a period of a few years (PLAN.md 1.2): no number; read price_swing_iron and price_swing_rails",
    "150 years in under 15 seconds (PLAN.md 1.1): set for the port of the Python; the speed of the code is read below and not judged, so that a search over the economy's numbers is not steered by the machine",
    "two to four items a decade in the major feed (PLAN.md 1.4): held by the chronicle's own rule (per_decade and at_least), so not a thing to calibrate",
];

/// Facts a scorecard should be read with that have no aim: the README's
/// headlines, the measures of the faults the pass mends, and the worries the
/// rules' pages carry. Twenty-year means where there are any.
pub const WITHOUT_AIM: [&str; 24] = [
    "real_wage_1751_1770_vs_1750",
    "real_wage_1791_1810_vs_1750",
    "real_wage_1831_1850_vs_1750",
    "real_wage_1871_1890_vs_1750",
    "real_wage_chained_1881_1900_vs_1750",
    "output_per_head_1881_1900_vs_1750",
    "output_per_head_peak_year",
    "farm_share_1881_1900",
    "industry_share_1881_1900",
    "diggings_share_1881_1900",
    "income_share_work_1881_1900",
    "share_coke_furnace_1800",
    "price_swing_rails",
    "real_wage_worst_year_median_town",
    "town_months_more_at_work_than_people",
    "town_months_wage_moved_by_half",
    "carters_share_of_hands_1900",
    "carters_per_1000_weight_path_1900_vs_1750",
    "works_hands_within_2_of_market_1900",
    "works_closing_chance_age_3_5",
    "works_closing_chance_age_25_49",
    "news_emigration",
    "cpu_seconds",
    "seconds",
];

/// Wages, swings and hard spells want this many islands (docs/rules/works.md,
/// "A run differs from its neighbour by a lot").
pub const SEEDS_WANTED: usize = 16;

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
    /// The aim's own tolerance (nought: none).
    pub tolerance: f64,
    /// The spread between islands, for reading beside the tolerance: the standard
    /// deviation across this setting's seeds, or across another sweep's if given.
    pub islands_sd: f64,
    /// How far the mean misses the aim, in the fact's units (nought when it does not).
    pub miss: f64,
    /// The same in tolerances; infinite for a miss where there is no tolerance, NaN
    /// where the mean is not judged or there is no value.
    pub misses_by: f64,
    pub met: bool,
    /// Islands whose own value is on the aim's side (the islands' test where the aim
    /// has one), of every island that finished: one with no value has not met it.
    pub seeds_met: usize,
    /// Islands that finished.
    pub seeds: usize,
    /// How many islands the aim asks for, scaled to the islands there are.
    pub seeds_needed: Option<usize>,
}

/// Each aim judged on one setting's values. `reference`, another sweep's setting,
/// gives the spread between islands printed beside each tolerance; the tolerances
/// themselves are the aims' own and do not move.
pub fn judge(values: &HashMap<String, Vec<(u64, f64)>>, reference: Option<&HashMap<String, Vec<(u64, f64)>>>) -> Vec<Judged> {
    // the islands that finished: every one has some fact
    let seeds = values.values().flat_map(|v| v.iter().map(|&(s, _)| s)).collect::<BTreeSet<u64>>().len();
    AIMS.iter()
        .map(|&aim| {
            let own = values.get(aim.fact).map(|v| &v[..]).unwrap_or(&[]);
            let here = spread(own);
            let islands_sd = match reference {
                Some(r) => r.get(aim.fact).and_then(|v| spread(v)).map_or(f64::NAN, |s| s.sd),
                None => here.map_or(f64::NAN, |s| s.sd),
            };
            let miss = here.map_or(f64::NAN, |s| aim.test.miss(s.mean));
            let misses_by = if !aim.on_mean || !miss.is_finite() {
                f64::NAN
            } else if aim.tolerance > 0.0 {
                miss / aim.tolerance
            } else if miss > 0.0 {
                f64::INFINITY
            } else {
                0.0
            };
            let each = aim.islands.map_or(aim.test, |(t, _)| t);
            let seeds_met = own.iter().filter(|&&(_, v)| v.is_finite() && each.miss(v) == 0.0).count();
            let seeds_needed = aim.islands.map(|(_, of16)| (of16 * seeds).div_ceil(SEEDS_WANTED));
            let mean_met = !aim.on_mean || misses_by <= 1.0;
            let islands_met = seeds_needed.is_none_or(|k| seeds_met >= k);
            let met = here.is_some() && mean_met && islands_met;
            Judged { aim, spread: here, tolerance: aim.tolerance, islands_sd, miss, misses_by, met, seeds_met, seeds, seeds_needed }
        })
        .collect()
}

/// Whether each of the twenty aims is met, by number: an aim of two parts is met
/// when both are.
pub fn aims_met(judged: &[Judged]) -> Vec<(u8, bool)> {
    let mut out: Vec<(u8, bool)> = Vec::new();
    for j in judged {
        match out.iter_mut().find(|(n, _)| *n == j.aim.number) {
            Some((_, met)) => *met &= j.met,
            None => out.push((j.aim.number, j.met)),
        }
    }
    out
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

/// The card of the sweep in `dir`, every setting of it, with the spread between
/// islands taken from the sweep in `spread_from` (its setting of the same name, or
/// its first) if given.
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
        if seeds.len() < SEEDS_WANTED {
            let _ = writeln!(out, "  (fewer than {SEEDS_WANTED} islands: the aims are stated for {SEEDS_WANTED}, and wages, swings and hard spells want them, docs/rules/works.md)");
        }
        let _ = writeln!(
            out,
            "the aims of the pass over the rules (docs/designs/rules-pass.md, section 2), each on the mean of the islands\n\
             tolerance: the aim's own, held still (none: the mean must be on the aim's side); spread: the islands'\n  \
             standard deviation, from {}, for reading beside it\n\
             met: the mean misses by no more than one tolerance, and enough islands meet it where the aim asks that\n\
             islands: how many meet the aim on their own (the islands' test where the aim has one)\n\
             loss: the sum over the aims with a tolerance of (miss / tolerance) squared, where miss is how far the\n  \
             mean falls on the wrong side of the aim (an aim with no value counts as missed by ten)\n",
            match (spread_from, refset) {
                (Some(p), Some(t)) => format!("the seeds of \"{}\" in {}", t.label, p.display()),
                _ => "these seeds".to_string(),
            },
        );
        let total = loss(&judged);
        let head = ["aim", "fact", "mean", "least .. greatest", "aim's band", "tolerance", "spread", "misses by", "met", "islands"];
        let mut rows: Vec<[String; 10]> = Vec::new();
        for j in &judged {
            let a = &j.aim;
            let k = kind(a.fact);
            let (mean, range) = match j.spread {
                Some(s) => (fmt_fact(k, s.mean), format!("[{} .. {}]", fmt_fact(k, s.least), fmt_fact(k, s.greatest))),
                None => ("-".to_string(), String::new()),
            };
            let band = match (a.on_mean, a.islands) {
                (true, None) => a.test.text(k),
                (true, Some((each, of16))) => format!("{}; each {} on {of16} of 16", a.test.text(k), each.text(k)),
                (false, Some((each, of16))) if of16 >= SEEDS_WANTED => format!("each {}, every island", each.text(k)),
                (false, Some((each, of16))) => format!("each {} on {of16} of 16", each.text(k)),
                (false, None) => String::new(),
            };
            let tol = if !a.on_mean { "-".to_string() } else if a.tolerance > 0.0 { fmt_tol(a.tolerance) } else { "none".to_string() };
            let by = if j.misses_by.is_finite() { format!("{:.2}", j.misses_by) } else if j.misses_by.is_infinite() { format!("by {}", fmt_tol(j.miss)) } else { "-".to_string() };
            let met = match (j.spread.is_some(), j.met) {
                (false, _) => "no",
                (true, true) => "yes",
                (true, false) => "no",
            };
            let islands = match j.seeds_needed {
                Some(k) => format!("{}/{} (needs {k})", j.seeds_met, j.seeds),
                None => format!("{}/{}", j.seeds_met, j.seeds),
            };
            let name = if a.in_loss { a.fact.to_string() } else { format!("{} (not in the loss)", a.fact) };
            rows.push([a.name(), name, mean, range, band, tol, fmt_tol(j.islands_sd), by, met.to_string(), islands]);
        }
        let width: Vec<usize> = (0..10).map(|c| rows.iter().map(|r| r[c].chars().count()).chain([head[c].len()]).max().unwrap_or(0)).collect();
        let line = |cells: &[String]| -> String {
            let mut s = String::from("  ");
            for (c, cell) in cells.iter().enumerate() {
                if [1, 3, 4].contains(&c) {
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
                let _ = writeln!(
                    out,
                    "      no island has a value, so the aim is not met: the sweep has no fact of this name (an older sweep, a run that does\n      \
                     not end in 1900, or rules that do not keep what it reads: the books, the chained cost of living and carting are the game's)"
                );
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
        let met = aims_met(&judged);
        let missed: Vec<String> = met.iter().filter(|(_, m)| !m).map(|(n, _)| n.to_string()).collect();
        let _ = writeln!(
            out,
            "\naims met: {} of {}{}",
            met.iter().filter(|(_, m)| *m).count(),
            met.len(),
            if missed.is_empty() { String::new() } else { format!(" (missed: {})", missed.join(", ")) }
        );
        let mut parts: Vec<(f64, String)> =
            judged.iter().filter(|j| j.aim.in_loss).map(|j| (if j.misses_by.is_finite() { j.misses_by * j.misses_by } else { 100.0 }, j.aim.name())).collect();
        parts.sort_by(|a, b| b.0.total_cmp(&a.0));
        let share = |x: f64| if total > 0.0 { format!("{:.0}%", 100.0 * x / total) } else { "-".to_string() };
        let _ = writeln!(
            out,
            "loss: {:.2} ({})",
            total,
            parts.iter().filter(|(x, _)| *x > 0.0).map(|(x, a)| format!("aim {a} {}", share(*x))).collect::<Vec<_>>().join(", ")
        );

        let w = BESIDE_CARD.iter().chain(WITHOUT_AIM.iter()).map(|f| f.len()).max().unwrap_or(0);
        let read = |out: &mut String, f: &str| {
            let s = setting.values.get(f).and_then(|v| spread(v));
            let text = match s {
                Some(s) if s.n < setting.seeds.len() => format!("{:>9}  [{} .. {}] ({}/{})", show(f, s.mean), show(f, s.least), show(f, s.greatest), s.n, setting.seeds.len()),
                Some(s) => format!("{:>9}  [{} .. {}]", show(f, s.mean), show(f, s.least), show(f, s.greatest)),
                None => format!("{:>9}", "-"),
            };
            let _ = writeln!(out, "  {f:<w$}  {text}");
        };
        out.push_str("\nRead beside the card (PLAN.md 1.3 names them and gives no number; the pass gives them no aim)\n");
        for f in BESIDE_CARD {
            read(&mut out, f);
        }
        out.push_str("\nStated, and not read by a number here\n");
        for u in UNREAD {
            let _ = writeln!(out, "  - {u}");
        }
        out.push_str("\nRead without an aim\n");
        for f in WITHOUT_AIM {
            read(&mut out, f);
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

    fn by<'a>(j: &'a [Judged], number: u8, part: char) -> &'a Judged {
        j.iter().find(|x| x.aim.number == number && x.aim.part == part).unwrap()
    }

    #[test]
    fn an_aim_is_met_within_its_own_tolerance_and_the_loss_counts_the_misses() {
        let v = values(&[
            // aim 3: mean 0.48 misses 0.5 by 0.02, two thirds of its tolerance of 0.03
            ("urban_share_1881_1900", &[0.46, 0.50, 0.46, 0.50]),
            // aim 5: a band; 2.4 is under it by 0.1, two thirds of 0.15
            ("real_wage_1881_1900_vs_1750", &[2.3, 2.5, 2.3, 2.5]),
            // aim 9: on the right side
            ("price_swing_iron", &[0.1, 0.12, 0.14, 0.16]),
            // aim 6: the mean 1.15 is in its band, but only two islands of four are up a tenth
            // and the aim asks twelve of sixteen, so three of four
            ("spending_per_head_chained_1890s_vs_1870s", &[1.0, 1.3, 1.0, 1.3]),
            // aim 20: no tolerance, so a mean a year over is missed
            ("years_to_half_coke_furnace", &[10.0, 12.0, 10.0, 12.0]),
            // aim 16, of the islands only: one island short
            ("share_bessemer_1880", &[0.95, 0.95, 0.95, 0.5]),
        ]);
        let j = judge(&v, None);
        let u = by(&j, 3, ' ');
        assert!(u.met && (u.misses_by - 0.02 / 0.03).abs() < 1e-12 && u.seeds_met == 2 && u.seeds == 4, "{u:?}");
        let w = by(&j, 5, ' ');
        assert!(w.met && (w.miss - 0.1).abs() < 1e-12 && w.seeds_met == 2, "{w:?}");
        let i = by(&j, 9, ' ');
        assert!(i.met && i.misses_by == 0.0 && i.seeds_met == 4, "{i:?}");
        let s = by(&j, 6, ' ');
        assert!(s.miss == 0.0 && s.seeds_needed == Some(3) && s.seeds_met == 2 && !s.met, "{s:?}");
        let c = by(&j, 20, ' ');
        assert!(!c.met && c.misses_by.is_infinite() && !c.aim.in_loss, "{c:?}");
        let b = by(&j, 16, ' ');
        assert!(!b.met && b.seeds_needed == Some(4) && b.seeds_met == 3 && b.misses_by.is_nan(), "{b:?}");
        // a fact no island has: not met, and in the loss as missed by ten
        let p = by(&j, 1, ' ');
        assert!(p.spread.is_none() && !p.met);
        // the loss: the aims with a tolerance, those with no value at ten each
        let in_loss = AIMS.iter().filter(|a| a.in_loss).count();
        let want = u.misses_by.powi(2) + w.misses_by.powi(2) + (in_loss - 4) as f64 * 100.0;
        assert!((loss(&j) - want).abs() < 1e-9, "{} against {want}", loss(&j));
        // the two parts of an aim are met together
        let met = aims_met(&j);
        assert_eq!(met.len(), AIM_COUNT);
        assert_eq!(met.iter().find(|(n, _)| *n == 3), Some(&(3, true)));

        // another sweep's spread is printed beside, and moves nothing that is judged
        let wide = values(&[("urban_share_1881_1900", &[0.40, 0.56, 0.40, 0.56])]);
        let j2 = judge(&v, Some(&wide));
        let u2 = by(&j2, 3, ' ');
        assert!(u2.islands_sd > 3.0 * u.islands_sd && u2.misses_by == u.misses_by && u2.met == u.met, "{u2:?}");
    }

    #[test]
    fn the_card_holds_the_designs_twenty_aims() {
        // twenty aims, numbered in order, the parts of one together; every one says who asks
        let numbers: Vec<u8> = AIMS.iter().map(|a| a.number).collect();
        assert!(numbers.windows(2).all(|w| w[1] == w[0] || w[1] == w[0] + 1) && numbers[0] == 1 && numbers[AIMS.len() - 1] == AIM_COUNT as u8);
        for a in AIMS {
            assert!(a.stated.contains(".md"), "{}: {}", a.fact, a.stated);
            // in the loss exactly when the mean is judged with a tolerance
            assert_eq!(a.in_loss, a.on_mean && a.tolerance > 0.0, "{}", a.name());
            assert!(a.on_mean || a.islands.is_some(), "{}", a.name());
        }
        // the critic's two changes (docs/designs/rules-pass.md, section 10.9, 5)
        let six = AIMS.iter().find(|a| a.number == 6).unwrap();
        assert_eq!((six.test, six.islands), (Test::Between(1.10, 1.40), Some((Test::AtLeast(1.10), 12))));
        let fifteen = AIMS.iter().find(|a| a.number == 15 && a.fact == "rail_km_1850").unwrap();
        assert_eq!(fifteen.islands, Some((Test::Between(1200.0, 1700.0), 14)));
        assert_eq!(BESIDE_CARD, ["share_textile_mill_1830", "rail_km_1870"]);
        assert!((Test::Between(2.5, 3.5).miss(3.7) - 0.2).abs() < 1e-12 && (Test::Between(2.5, 3.5).miss(2.0) - 0.5).abs() < 1e-12);
        assert_eq!(Test::Between(2.5, 3.5).miss(3.0), 0.0);
        assert_eq!(parse_args(&["d".to_string()]).unwrap(), (PathBuf::from("d"), None));
        assert!(parse_args(&[]).is_err() && parse_args(&["a".to_string(), "b".to_string()]).is_err());
        assert!(parse_args(&["d".to_string(), "--spread".to_string()]).is_err());
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
        let coke = card.lines().find(|l| l.contains("years_to_half_coke_furnace (not in the loss)")).unwrap();
        assert!(coke.trim_end().ends_with("1/2"), "{coke}");
        // a lone island, and fewer than sixteen
        assert!(card.contains("Scorecard: c, 1 run (seed 4)") && card.contains("fewer than 16 islands"), "{card}");
        // a fact no island has is said, and the aim is not met; aim 20 is met on the islands it has
        assert!(card.contains("pop_1900_vs_1750") && card.contains("no island has a value, so the aim is not met"), "{card}");
        assert!(card.contains("aims met: 1 of 20 (missed: 1, 2, 3,"), "{card}");
        // the two read beside the card
        assert!(card.contains("Read beside the card") && card.contains("rail_km_1870"), "{card}");
    }
}
