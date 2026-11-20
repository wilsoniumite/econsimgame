//! How a map view's colours are spread over its figures: where the palest and
//! the darkest tint stand. Three ways, chosen on the legend card and kept for
//! each view while the viewer is open:
//!
//! - **This year**: the year's own least and greatest.
//! - **Thirty years**: the least and greatest of the last thirty years, which
//!   the two ends glide toward: each year an end goes only `PACE` of the way
//!   from where it stood to where the window would put it, so that one wild
//!   year coming into the window, or leaving it thirty years on, cannot make
//!   the colours jump.
//! - **Whole run**: the least and greatest of every year played so far (of the
//!   whole record, in a recorded run).
//!
//! The least and the greatest are never one odd town's or one odd square's:
//! they are taken a little inside the extremes, at the `LOW` and `HIGH`
//! centiles of what the view shows, and what lies beyond is drawn in the
//! end's tint (the legend says "and below" and "and above").
//!
//! The ends are reckoned in the scale's own measure: the figure itself for an
//! even scale, and its logarithm for a scale by ratio, so that rent gliding
//! from 10 to 1,000 passes 100 halfway, as its tints do.

/// How the ends of a view's scale are taken.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Range {
    Year,
    Thirty,
    Whole,
}

impl Range {
    pub const ALL: [Range; 3] = [Range::Year, Range::Thirty, Range::Whole];

    /// The word on its chip.
    pub fn chip(self) -> &'static str {
        match self {
            Range::Year => "This year",
            Range::Thirty => "30 years",
            Range::Whole => "Whole run",
        }
    }

    /// Its word after a map view on the command line and in scripts (`rent+year`).
    pub fn word(self) -> &'static str {
        match self {
            Range::Year => "year",
            Range::Thirty => "30",
            Range::Whole => "all",
        }
    }

    pub fn parse(w: &str) -> Option<Range> {
        Range::ALL.into_iter().find(|r| r.word().eq_ignore_ascii_case(w))
    }
}

/// The centiles the ends are taken at, of the squares or towns a view shows:
/// one market in twenty at either end is left out of setting them. Chosen by
/// looking at the price of coal on island 7 (build notes, "The centiles").
pub const LOW: f32 = 0.05;
pub const HIGH: f32 = 0.95;
/// The years the second setting looks back over, this one among them.
pub const WINDOW: usize = 30;
/// The part of the way an end goes in a year toward where the window puts it
/// (build notes, "The pace").
pub const PACE: f32 = 0.2;

/// A view's two ends, in its scale's own measure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ends {
    pub lo: f32,
    pub hi: f32,
}

/// Under this many towns or squares the ends are the least and the greatest:
/// there are too few to tell an odd one.
pub const FEW: usize = 5;

/// The least and the greatest of a sorted list once the `LOW` share at its
/// bottom and the `HIGH` share at its top are left out, at least one at each
/// end: so one odd town of twenty, or of five, never sets an end. A list of
/// fewer than `FEW` keeps its ends.
pub fn trimmed(sorted: &[f32]) -> Option<(f32, f32)> {
    let n = sorted.len();
    if n == 0 {
        return None;
    }
    if n < FEW {
        return Some((sorted[0], sorted[n - 1]));
    }
    // (a hair under, so that a twentieth of twenty is one and not two by rounding)
    let below = ((n as f32 * LOW - 1e-3).ceil() as usize).max(1);
    let above = ((n as f32 * (1.0 - HIGH) - 1e-3).ceil() as usize).max(1);
    Some((sorted[below.min(n - 1)], sorted[(n - 1).saturating_sub(above)]))
}

/// One year's ends from what it shows (in the scale's measure), or None if it
/// shows nothing. `pinned` holds the low end where the view's nature sets it
/// (nought kilometres to market, a deposit of nothing).
pub fn year_ends(mut vals: Vec<f32>, pinned: Option<f32>) -> Option<Ends> {
    vals.retain(|v| v.is_finite());
    vals.sort_by(f32::total_cmp);
    let (lo, hi) = trimmed(&vals)?;
    let lo = pinned.unwrap_or(lo);
    Some(Ends { lo, hi: hi.max(lo) })
}

/// The least of the lows and the greatest of the highs of some years.
fn widest<'a>(years: impl Iterator<Item = &'a Option<Ends>>) -> Option<Ends> {
    years.flatten().fold(None, |acc: Option<Ends>, e| Some(match acc {
        None => *e,
        Some(a) => Ends { lo: a.lo.min(e.lo), hi: a.hi.max(e.hi) },
    }))
}

/// Where the thirty years' window would put the ends in year `j`.
pub fn window(years: &[Option<Ends>], j: usize) -> Option<Ends> {
    let j = j.min(years.len().checked_sub(1)?);
    widest(years[(j + 1).saturating_sub(WINDOW)..=j].iter())
}

/// The ends gliding toward the window's, from the first year to year `k`:
/// each year they go `PACE` of the way from where they stood. A year that
/// shows nothing leaves them where they were.
pub fn glide(years: &[Option<Ends>], k: usize) -> Option<Ends> {
    let mut now: Option<Ends> = None;
    for j in 0..=k.min(years.len().saturating_sub(1)) {
        if years.is_empty() {
            break;
        }
        let Some(to) = window(years, j) else { continue };
        now = Some(match now {
            None => to,
            Some(e) => Ends { lo: e.lo + PACE * (to.lo - e.lo), hi: e.hi + PACE * (to.hi - e.hi) },
        });
    }
    now
}

/// The ends of year `k` under a setting, from each year's own ends (`years`,
/// as many as there are years played so far).
pub fn ends(range: Range, years: &[Option<Ends>], k: usize) -> Option<Ends> {
    match range {
        Range::Year => years.get(k).copied().flatten(),
        Range::Thirty => glide(years, k),
        Range::Whole => widest(years.iter()),
    }
}

/// Round numbers between `lo` and `hi` (the figures themselves, not their
/// measure) for the legend's labels: three to six of them, on steps of 1, 2,
/// 2.5 or 5 of a power of ten, or for a scale by ratio on 1, 2 and 5 of each
/// power of ten where there is room for them.
pub fn ticks(lo: f32, hi: f32, log: bool) -> Vec<f32> {
    if !(hi > lo) || !lo.is_finite() || !hi.is_finite() {
        return Vec::new();
    }
    if log && lo > 0.0 && hi / lo >= 4.0 {
        for mantissas in [&[1.0, 2.0, 5.0][..], &[1.0][..]] {
            let mut out = Vec::new();
            let mut e = lo.log10().floor() as i32;
            while 10f32.powi(e) <= hi * 1.0001 {
                for &m in mantissas {
                    let v = m * 10f32.powi(e);
                    if v >= lo * 0.9999 && v <= hi * 1.0001 {
                        out.push(v);
                    }
                }
                e += 1;
            }
            if out.len() <= 6 {
                if out.len() >= 2 {
                    return out;
                }
                break;
            }
            if mantissas.len() == 1 {
                // more than six powers of ten: every other one
                let step = out.len().div_ceil(6);
                return out.into_iter().step_by(step).collect();
            }
        }
    }
    let span = hi - lo;
    let mag = 10f32.powf((span / 4.0).log10().floor());
    // three to five if there is a step that gives them, else two to six
    for wanted in [3..=5, 2..=6] {
        for m in [0.5, 1.0, 2.0, 2.5, 5.0, 10.0, 20.0] {
            let step = m * mag;
            let first = (lo / step - 1e-4).ceil() as i64;
            let last = (hi / step + 1e-4).floor() as i64;
            if wanted.contains(&(last - first + 1)) {
                return (first..=last).map(|i| i as f32 * step).collect();
            }
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(lo: f32, hi: f32) -> Option<Ends> {
        Some(Ends { lo, hi })
    }

    #[test]
    fn one_odd_town_never_sets_an_end() {
        // five towns, one of them wild: the ends are the second and the fourth
        assert_eq!(trimmed(&[1.0, 2.0, 3.0, 4.0, 100.0]), Some((2.0, 4.0)));
        // too few to tell: the least and the greatest
        assert_eq!(trimmed(&[1.0, 2.0, 100.0]), Some((1.0, 100.0)));
        assert_eq!(trimmed(&[]), None);
        // one odd town out of twenty does not set the end
        let mut towns: Vec<f32> = (0..19).map(|i| 1.0 + i as f32 * 0.01).collect();
        towns.push(9.0);
        let ends = year_ends(towns, None).unwrap();
        assert!((ends.hi - 1.18).abs() < 1e-5 && (ends.lo - 1.01).abs() < 1e-5, "{ends:?}");
        // of six thousand squares, the twentieth part at either end is left out
        let squares: Vec<f32> = (0..6000).map(|i| i as f32).collect();
        assert_eq!(trimmed(&squares), Some((300.0, 5699.0)));
        // the low end can be pinned by the view's nature
        assert_eq!(year_ends(vec![3.0, 4.0], Some(0.0)).unwrap().lo, 0.0);
        assert_eq!(year_ends(vec![f32::NAN], None), None);
    }

    #[test]
    fn the_three_settings_take_their_own_years() {
        let years = [e(1.0, 2.0), None, e(0.5, 3.0), e(1.0, 2.0)];
        assert_eq!(ends(Range::Year, &years, 0), e(1.0, 2.0));
        assert_eq!(ends(Range::Year, &years, 1), None);
        assert_eq!(ends(Range::Whole, &years, 0), e(0.5, 3.0));
        // thirty years: the window holds the wild year 2, and the ends go a part of the way toward it
        let g = ends(Range::Thirty, &years, 2).unwrap();
        assert!((g.lo - (1.0 - PACE * 0.5)).abs() < 1e-6 && (g.hi - (2.0 + PACE)).abs() < 1e-6, "{g:?}");
    }

    /// The pass the brief asks for: in the thirty years' setting neither end
    /// moves more than `PACE` of the way to the window's end in any year, for
    /// a run of years with wild ones coming into the window and leaving it.
    #[test]
    fn in_thirty_years_neither_end_moves_more_than_the_pace_in_a_year() {
        let mut years = Vec::new();
        let mut x = 7u32;
        for j in 0..200 {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let wobble = (x >> 16) as f32 / 65_536.0;
            // a calm run, a wild year every 37, and a stretch of nothing
            let (lo, hi) = if j % 37 == 11 { (-5.0, 40.0) } else { (1.0 + wobble, 3.0 + wobble) };
            years.push(if (90..95).contains(&j) { None } else { e(lo, hi) });
        }
        let mut before = glide(&years, 0).unwrap();
        let mut biggest = 0.0f32;
        for k in 1..years.len() {
            let now = glide(&years, k).unwrap();
            match window(&years, k) {
                Some(to) => {
                    for (b, n, t) in [(before.lo, now.lo, to.lo), (before.hi, now.hi, to.hi)] {
                        let way = (t - b).abs();
                        assert!((n - b).abs() <= PACE * way + 1e-4, "year {k}: an end moved {} of a way of {way}", (n - b).abs());
                        if way > 0.0 {
                            biggest = biggest.max((n - b).abs() / way);
                        }
                    }
                }
                None => assert_eq!(now, before, "year {k} shows nothing and the ends moved"),
            }
            before = now;
        }
        assert!((biggest - PACE).abs() < 1e-3, "the ends never went their whole pace: {biggest}");
    }

    #[test]
    fn ticks_are_round_and_inside_the_ends() {
        let near = |a: Vec<f32>, b: &[f32]| a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1e-4 * y.abs().max(1.0));
        assert!(near(ticks(0.83, 1.46, false), &[1.0, 1.2, 1.4]));
        assert!(near(ticks(0.0, 25.0, false), &[0.0, 10.0, 20.0]));
        assert!(near(ticks(1650.0, 1900.0, false), &[1700.0, 1800.0, 1900.0]));
        // by ratio: 1, 2 and 5 of each power of ten while they fit, else the powers alone
        assert!(near(ticks(3.0, 120.0, true), &[5.0, 10.0, 20.0, 50.0, 100.0]));
        assert!(near(ticks(0.5, 2.0e5, true), &[1.0, 10.0, 100.0, 1000.0, 10_000.0, 100_000.0]));
        // a ratio too narrow for its powers goes by even steps
        assert!(near(ticks(300.0, 900.0, true), &[400.0, 600.0, 800.0]));
        assert!(ticks(3.0, 3.0, false).is_empty());
    }
}
