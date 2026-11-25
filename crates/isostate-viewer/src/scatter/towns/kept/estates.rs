//! Streets laid out by one owner at one time: a square a town filled whole
//! after 1900 is drawn as an estate of the year it was filled, with streets
//! of its own (`docs/designs/ways.md`, part 5, "Streets inside towns").
//!
//! On 21 November 2026 the designer wrote: "dense towns shouldn't have
//! patches if farmland, they can have parks but we'll need to find some way
//! to make the cities properly dense"; shown the first city of 2050 on 23
//! November she found it "a bit too chaotic". What the eye saw was a filled
//! square with no streets of its own: its houses stood only along the lanes
//! behind the ways between squares, and its middle was built over close by
//! close. The rule, in the page's steps:
//!
//! 1. **Squares built by 1900 are left alone.** Only a square that joined its
//!    town after 1900 (`lots::FROM`) and was filled whole (`Replay::filled`)
//!    is an estate. Its year is the first year it was filled; the record
//!    keeps a square's year of joining in steps of two years past 1900, and a
//!    square filled whole is filled the year it is taken (or the year after).
//! 2. **Every spot of the square belongs to its nearest way**: the ways
//!    between squares (`ways`, kind 0: the roads, and the lane to the square
//!    a square grew from), as they stood in the estate's year. Its streets are
//!    laid in that way's own frame, `u` along it from its first end and `v`
//!    out from it, so a street runs on unbroken into the next square beside
//!    the same way, and two estates of different years meet in a seam.
//! 3. **The pattern is its year's and its ground's** (`Pattern`): by-law
//!    terraces to 1918, the garden suburb to 1944, the estate with a loop to
//!    1969, culs-de-sac after; and on ground worth three times its town's of
//!    1900 in that year (`lots::BLOCK_VALUE`), perimeter blocks.
//! 4. **A street once laid is never moved.** What decides it (the square's
//!    year, its value then, and the ways that stood then) is fixed from the
//!    year it is laid; a way that comes later takes nothing from it.
//! 5. **No street crosses** a park close, a works' yard, a churchyard, the
//!    railway or the water, nor runs on to ground nearer another way
//!    (`Laying::asked`): it stops at their edge.
//! 6. **Plots front every new street**, both sides; a filled square's plots
//!    are all built the year it was filled, in the forms of that year, and
//!    rebuilt afterwards only by `lots.rs`'s rule. Where its ground was worth
//!    `WALLS` times its town's of 1900 or more, the plots of a street between
//!    two cross streets are thrown together into one street wall; on cheaper
//!    ground houses stand apart as their era built them (semis in pairs,
//!    detached houses alone, the by-law years' terraces a row to a block).
//!    On blocks, a block between four streets is one building (`Lot::Block`)
//!    where it fits whole, with plots along its streets where it does not.
//! 7. **The ways' lanes give way.** The lanes and cross streets behind a way
//!    (kinds 1 to 4) are not laid on an estate's ground; where an older
//!    square's house fronts such a lane at the estate's edge it still stands
//!    as it stood.
//!
//! `ISOSTATE_STREETS=old` draws every town as before this rule, for pictures
//! side by side.

use super::*;

/// The pattern an estate's streets are laid in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Pattern {
    /// 1901 to 1918: streets every 0.12 along the way, cross streets every 0.42; terraces back to back
    ByLaw,
    /// 1919 to 1944: streets every 0.16, bowed in a long wave; cross streets every 0.55; semis with gardens
    Garden,
    /// 1945 to 1969: two distributors, culs-de-sac off the first, joined by link roads; pairs and low flats
    Loop,
    /// 1970 on: culs-de-sac off the way and off a spine behind it, no way through; detached houses
    Closes,
    /// any year, on ground worth `lots::BLOCK_VALUE` times its town's of 1900: perimeter blocks
    Blocks,
}

impl Pattern {
    pub(crate) fn of(year: i32, value: f32) -> Pattern {
        if value >= lots::BLOCK_VALUE {
            Pattern::Blocks
        } else if year <= 1918 {
            Pattern::ByLaw
        } else if year <= 1944 {
            Pattern::Garden
        } else if year <= 1969 {
            Pattern::Loop
        } else {
            Pattern::Closes
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Pattern::ByLaw => "by-law terraces",
            Pattern::Garden => "garden suburb",
            Pattern::Loop => "estate with a loop",
            Pattern::Closes => "culs-de-sac",
            Pattern::Blocks => "blocks",
        }
    }

    /// How wide its plots are along a street: a terrace house's, a semi's
    /// half with a side garden, a council pair's, a detached house's wide
    /// plot, a range of flats over shops.
    pub(crate) fn plot(self) -> f32 {
        match self {
            Pattern::ByLaw => 0.046,
            Pattern::Garden => 0.06,
            Pattern::Loop => 0.058,
            Pattern::Closes => 0.072,
            Pattern::Blocks => 0.05,
        }
    }

    /// How far apart the cross streets are along the way, and the streets
    /// along it out from it: the faces of a street wall end there.
    fn every(self) -> (f32, f32) {
        match self {
            Pattern::ByLaw => (0.42, 0.12),
            Pattern::Garden => (0.55, 0.16),
            Pattern::Loop => (LOOP_CULS, 0.26),
            Pattern::Closes => (0.22, 0.26),
            Pattern::Blocks => (0.2, 0.13),
        }
    }
}

/// How far the garden suburb's streets along the way bow out and back, in a
/// wave of 1.1 along it.
const GARDEN_WAVE: f32 = 0.05;

/// How far apart the culs-de-sac off a loop's first distributor are: a third
/// of the way between its link roads (0.55), so that a link road stands where
/// a cul-de-sac would, and the faces of a distributor end at both.
const LOOP_CULS: f32 = 0.55 / 3.0;

/// The land value (times its town's of 1900, in the year an estate was laid
/// out) from which its frontages are thrown together into street walls: half
/// as dear again. Below it houses stand apart as their era built them. On
/// island 7 the first city's estates were laid out on ground worth 1.0 to
/// 4.6 times its town's of 1900 (the notes of `town-streets`).
pub(crate) const WALLS: f32 = 1.5;

/// A square laid out as one owner's estate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Estate {
    /// the year (counted from the start) it was laid out, and the calendar year
    pub since: u32,
    pub year: i32,
    pub pattern: Pattern,
    /// its land value that year, over its town's of 1900
    pub value: f32,
    /// its frontages are street walls (`WALLS`)
    pub walls: bool,
}

/// `ISOSTATE_STREETS=old` (or `ISOSTATE_LOOK=plain`): no estate is laid out,
/// and every town is drawn as before this rule.
pub(crate) fn old() -> bool {
    #[cfg(test)]
    if let Some(old) = OLD_IN_TEST.with(|o| o.get()) {
        return old;
    }
    static OLD: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *OLD.get_or_init(|| std::env::var("ISOSTATE_STREETS").as_deref() == Ok("old")) || lots::plain()
}

#[cfg(test)]
thread_local! {
    /// a test's own setting of `old`, for laying the same record out both ways
    pub(crate) static OLD_IN_TEST: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// The estates of year `k`: every square of town ground that joined its town
/// after 1900 and has been filled whole in some year since, with the first
/// such year. It reads the record's years up to `k` only, so a square is an
/// estate from the year it is filled on, and the same in every later year.
pub(super) fn of_year(r: &Replay, k: usize, town: &world::TownGround) -> HashMap<u32, Estate> {
    let mut out: HashMap<u32, Estate> = HashMap::default();
    if old() || !r.builds.kept() {
        return out;
    }
    let first = ((lots::FROM + 1 - r.y0).max(0) as usize).min(k + 1);
    for &c in town.cells.keys() {
        let Some(joined) = r.joined(k, c as usize) else { continue };
        if joined <= lots::FROM {
            continue;
        }
        let from = (joined_in(r, k, c as usize) as usize).max(first);
        if let Some(y) = (from..=k).find(|&y| r.filled(y, c as usize)) {
            let value = r.builds.value(r.n, y, c as usize).unwrap_or(1.0);
            let year = r.y0 + y as i32;
            out.insert(c, Estate { since: y as u32, year, pattern: Pattern::of(year, value), value, walls: value >= WALLS });
        }
    }
    out
}

/// What an estate's street knows of its estate and of the way it is laid from.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Laid {
    pub cell: u32,
    pub pattern: Pattern,
    pub walls: bool,
    pub year: i32,
    /// the way's first end, the way along it, and out from it on this street's side
    pub o: Vec2,
    pub u: Vec2,
    pub n: Vec2,
    /// a cul-de-sac or its stub, with a turning circle at its far end
    pub circle: bool,
    /// a back street along the line where the ground of this street's way
    /// meets that of this other way (step 4), laid where the two are near
    /// enough equally far
    pub between: Option<i32>,
}

impl Laid {
    /// Where a spot lies in the way's frame: along it, and out from it on the street's side.
    pub fn frame(&self, p: Vec2) -> (f32, f32) {
        ((p - self.o).dot(self.u), (p - self.o).dot(self.n))
    }

    /// The row a plot of an estate street goes up in (`Plot::row`), from
    /// where it stands and its place along its street: the face of the
    /// street between two cross streets, and within it, on cheap ground,
    /// pairs of plots (semis) or single plots (detached houses).
    pub fn row(&self, kind: u8, p: Vec2, index: i32) -> i32 {
        if self.between.is_some() {
            // a back street runs across its ways' faces: walls of four plots, or pairs, or single plots
            let own = index.rem_euclid(1 << 20);
            return match self.pattern {
                _ if self.walls => own / 4,
                Pattern::ByLaw | Pattern::Blocks => own / 4,
                Pattern::Garden | Pattern::Loop => own / 2,
                Pattern::Closes => own,
            };
        }
        let (u, v) = self.frame(p);
        let (along, out) = self.pattern.every();
        // the faces of a street along the way end at its cross streets, which stand at the
        // middle of each `along`; those of a street out from it at the streets along it (the
        // loop's distributors from 0.2 out, the garden suburb's in their wave)
        let face = if kind == 6 {
            (u / along - 0.5).floor() as i32
        } else {
            match self.pattern {
                Pattern::Loop => ((v - 0.2) / out).floor() as i32,
                Pattern::Garden => ((v - GARDEN_WAVE * (u * std::f32::consts::TAU / 1.1).sin()) / out).floor() as i32,
                _ => (v / out).floor() as i32,
            }
        };
        let face = face.clamp(-4000, 4000) * 512;
        let own = index.rem_euclid(512);
        match self.pattern {
            _ if self.walls => face,
            Pattern::ByLaw | Pattern::Blocks => face,
            Pattern::Garden | Pattern::Loop => face + own / 2,
            Pattern::Closes => face + own,
        }
    }
}

/// A block between four streets of an estate on blocks, which a whole block
/// of the forms may stand on (`Lot::Block`): its middle, the way its front
/// faces (to the way), its width along the way and its depth, its cell and
/// the way it belongs to.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BlockGround {
    /// its number, as a street's (`id_of`): its plot's street
    pub id: i32,
    pub at: Vec2,
    pub front: Vec2,
    pub width: f32,
    pub depth: f32,
    pub cell: u32,
    pub main: i32,
    pub since: u32,
    pub year: i32,
}

/// The streets to be numbered from here up: clear of every way's and lane's
/// number (`ways`: a link's 64 numbers from (cell x 4 + direction) x 64,
/// some 2.6 million on island 7's map of 10,080 cells).
const FIRST_ID: i32 = 1 << 22;

/// An estate street's number: its square, the way's place among those its
/// square is laid from (fewer than 8), and the street's own place (fewer
/// than 1,024), all fixed by what fixes the street. (Kept under 2^27, so that
/// sixteen times it, a plot's own number in `lots::draw`, is still a number.)
fn id_of(cell: u32, slot: usize, own: usize) -> i32 {
    FIRST_ID + ((cell as i32 * 8 + slot.min(7) as i32) * 1024 + own.min(1023) as i32)
}

/// The streets of year `k`'s estates, and the blocks of those on blocks.
/// `mains` are the year's ways between squares (`ways`, kind 0): each one's
/// ends, its number and its year (counted from the start).
pub(super) fn streets(r: &Replay, estates: &HashMap<u32, Estate>, mains: &[(Vec2, Vec2, i32, u32)]) -> (Vec<Way>, Vec<BlockGround>) {
    let mut out: Vec<Way> = Vec::new();
    let mut blocks: Vec<BlockGround> = Vec::new();
    let mut cells: Vec<(&u32, &Estate)> = estates.iter().collect();
    cells.sort_by_key(|c| *c.0);
    // the ways by the square of the map they come within reach of
    let mut by_cell: HashMap<(i32, i32), Vec<usize>> = HashMap::default();
    for (i, &(a, b, ..)) in mains.iter().enumerate() {
        let reach = 1.6;
        let (lo, hi) = (a.min(b) - Vec2::splat(reach), a.max(b) + Vec2::splat(reach));
        for cy in lo.y.floor() as i32..=hi.y.floor() as i32 {
            for cx in lo.x.floor() as i32..=hi.x.floor() as i32 {
                if off_line(Vec2::new(cx as f32 + 0.5, cy as f32 + 0.5), a, b) < reach {
                    by_cell.entry((cx, cy)).or_default().push(i);
                }
            }
        }
    }
    const SAMPLES: usize = 8;
    for (&c, e) in cells {
        let (cx, cy) = ((c as usize % r.w) as i32, (c as usize / r.w) as i32);
        // the ways that stood the year it was laid out
        let near: Vec<usize> = by_cell.get(&(cx, cy)).map_or(Vec::new(), |v| v.iter().copied().filter(|&i| mains[i].3 <= e.since).collect());
        if near.is_empty() {
            continue;
        }
        // which way each spot of the square is nearest, and how far along and out from it the
        // spots of each way lie: (way, side) -> least and most along, most out
        let mut reach: std::collections::BTreeMap<(i32, i8), (usize, f32, f32, f32)> = Default::default();
        let mut owner: Vec<usize> = Vec::with_capacity((SAMPLES + 1) * (SAMPLES + 1));
        for sy in 0..=SAMPLES {
            for sx in 0..=SAMPLES {
                let p = Vec2::new(cx as f32 + sx as f32 / SAMPLES as f32, cy as f32 + sy as f32 / SAMPLES as f32);
                let Some(&i) = near.iter().min_by(|&&i, &&j| off_line(p, mains[i].0, mains[i].1).total_cmp(&off_line(p, mains[j].0, mains[j].1)).then(mains[i].2.cmp(&mains[j].2))) else { continue };
                owner.push(i);
                let (a, b, id, ..) = mains[i];
                let u = (b - a).normalize_or_zero();
                let n = Vec2::new(-u.y, u.x);
                let (along, out) = ((p - a).dot(u), (p - a).dot(n));
                let side = if out >= 0.0 { 1 } else { -1 };
                let got = reach.entry((id, side)).or_insert((i, f32::MAX, f32::MIN, 0.0));
                got.1 = got.1.min(along);
                got.2 = got.2.max(along);
                got.3 = got.3.max(out.abs());
            }
        }
        // the ways in the order of their numbers: each one's place among them is its slot
        let mut slots: Vec<i32> = reach.keys().map(|k| k.0).collect();
        slots.dedup();
        for (&(id, side), &(i, u0, u1, v1)) in &reach {
            let slot = slots.iter().position(|&s| s == id).unwrap_or(0);
            let (a, b, ..) = mains[i];
            let u = (b - a).normalize_or_zero();
            let n = Vec2::new(-u.y, u.x) * side as f32;
            // a little past the last spot each way, so that the street reaches the square's edge
            let pad = 1.0 / SAMPLES as f32;
            let (u0, u1, v1) = (u0 - pad, u1 + pad, v1 + pad);
            let laid = Laid { cell: c, pattern: e.pattern, walls: e.walls, year: e.year, o: a, u, n, circle: false, between: None };
            let mut own = if side > 0 { 0 } else { 512 };
            let at = |along: f32, out: f32| a + u * along + n * out;
            // a street starts at the edge of the widest the way comes to be, so that a road
            // widened later moves no street
            let edge = WIDEST_WAY * 0.5 + 0.004;
            let mut push = |p: Vec2, q: Vec2, w: f32, kind: u8, circle: bool| {
                out.push(Way { a: p, b: q, width: w, id: id_of(c, slot, own), road: false, kind, main: id, since: e.since, estate: Some(Laid { circle, ..laid }) });
                own += 1;
            };
            // streets along the way every `every`, from `first` out, straight or in a wave
            let along_way = |first: f32, every: f32, w: f32, wave: f32, push: &mut dyn FnMut(Vec2, Vec2, f32, u8, bool)| {
                let mut v = first;
                while v <= v1 {
                    if wave > 0.0 {
                        // bowed by `wave` in a wave of 1.1 along the way, the bends at the way's own places
                        let step = 1.1 / 8.0;
                        let mut s = (u0 / step).floor();
                        while s * step < u1 {
                            let bow = |t: f32| v + wave * (t * std::f32::consts::TAU / 1.1).sin();
                            let (t0, t1) = (s * step, (s + 1.0) * step);
                            push(at(t0, bow(t0)), at(t1, bow(t1)), w, 6, false);
                            s += 1.0;
                        }
                    } else {
                        push(at(u0, v), at(u1, v), w, 6, false);
                    }
                    v += every;
                }
            };
            // streets out from the way every `every` along it, the first at the middle of one,
            // from `from` to `to` out
            let across = |every: f32, offset: f32, from: f32, to: f32, w: f32, circle: bool, push: &mut dyn FnMut(Vec2, Vec2, f32, u8, bool)| {
                let mut j = ((u0 - offset) / every).ceil();
                while j * every + offset <= u1 {
                    let t = j * every + offset;
                    push(at(t, from), at(t, to), w, 7, circle);
                    j += 1.0;
                }
            };
            match e.pattern {
                Pattern::ByLaw => {
                    along_way(0.12, 0.12, 0.016, 0.0, &mut push);
                    across(0.42, 0.21, edge, v1, 0.016, false, &mut push);
                }
                Pattern::Blocks => {
                    along_way(0.13, 0.13, 0.018, 0.0, &mut push);
                    across(0.2, 0.1, edge, v1, 0.018, false, &mut push);
                    // each block between two streets along the way and two across, from the
                    // first street along it out
                    let hw = 0.009 + 0.006;
                    // (numbered after the streets of this side, which are fewer than two hundred)
                    let mut number = if side > 0 { 200 } else { 712 };
                    let mut row = 1.0;
                    while (row + 1.0) * 0.13 <= v1 {
                        let mut j = ((u0 - 0.1) / 0.2).ceil();
                        while (j + 1.0) * 0.2 + 0.1 <= u1 {
                            let (t, v) = ((j + 1.0) * 0.2, (row + 0.5) * 0.13);
                            blocks.push(BlockGround { id: id_of(c, slot, number), at: at(t, v), front: -n, width: 0.2 - 2.0 * hw, depth: 0.13 - 2.0 * hw, cell: c, main: id, since: e.since, year: e.year });
                            number += 1;
                            j += 1.0;
                        }
                        row += 1.0;
                    }
                }
                Pattern::Garden => {
                    along_way(0.16, 0.16, 0.018, GARDEN_WAVE, &mut push);
                    across(0.55, 0.275, edge, v1, 0.018, false, &mut push);
                }
                Pattern::Loop => {
                    // two distributors, then again further out; culs-de-sac off the first of each
                    // pair; link roads from the way to the last
                    let mut m = 0.0;
                    let mut last = 0.0f32;
                    while 0.2 + 0.52 * m <= v1 {
                        let (d1, d2) = (0.2 + 0.52 * m, 0.46 + 0.52 * m);
                        push(at(u0, d1), at(u1, d1), 0.022, 6, false);
                        across(LOOP_CULS, LOOP_CULS * 0.5, d1 + 0.011, d1 + 0.13, 0.016, true, &mut push);
                        last = d1;
                        if d2 <= v1 {
                            push(at(u0, d2), at(u1, d2), 0.022, 6, false);
                            last = d2;
                        }
                        m += 1.0;
                    }
                    across(0.55, 0.275, edge, last.max(0.2), 0.016, false, &mut push);
                }
                Pattern::Closes => {
                    // culs-de-sac off the way, each with a stub to one side at its end
                    let (every, long) = (0.22, 0.24);
                    let mut j = ((u0 - every * 0.5) / every).ceil();
                    while j * every + every * 0.5 <= u1 {
                        let t = j * every + every * 0.5;
                        push(at(t, edge), at(t, long), 0.014, 7, true);
                        let to = if (j as i32).rem_euclid(2) == 0 { 0.07 } else { -0.07 };
                        push(at(t, long - 0.04), at(t + to, long - 0.04), 0.012, 7, true);
                        j += 1.0;
                    }
                    // and behind them a spine every 0.75 with culs-de-sac off both sides
                    let mut spine = 0.52;
                    while spine - 0.2 <= v1 {
                        push(at(u0, spine), at(u1, spine), 0.016, 6, false);
                        across(every, every * 0.5, spine - 0.011, spine - 0.2, 0.014, true, &mut push);
                        across(every, every * 0.5, spine + 0.011, spine + 0.2, 0.014, true, &mut push);
                        spine += 0.75;
                    }
                }
            }
        }
        // step 4: where the ground of two ways that share no meeting place comes together, a
        // back street along the line between them closes the blocks of both. (Between two ways
        // that meet at a junction nothing is laid on that line: it would be a spoke from every
        // crossroads.) The line is the one through the spots between two samples of the square
        // nearest the two, a straight street as long as they reach and a little more.
        let meet = |i: usize, j: usize| {
            let (a, b) = ((mains[i].0, mains[i].1), (mains[j].0, mains[j].1));
            [a.0, a.1].iter().any(|p| p.distance(b.0) < 1e-3 || p.distance(b.1) < 1e-3)
        };
        let mut seams: std::collections::BTreeMap<(i32, i32), (usize, Vec<Vec2>)> = Default::default();
        if owner.len() == (SAMPLES + 1) * (SAMPLES + 1) {
            for sy in 0..=SAMPLES {
                for sx in 0..=SAMPLES {
                    let here = owner[sy * (SAMPLES + 1) + sx];
                    for (nx, ny) in [(sx + 1, sy), (sx, sy + 1)] {
                        if nx > SAMPLES || ny > SAMPLES {
                            continue;
                        }
                        let there = owner[ny * (SAMPLES + 1) + nx];
                        if here == there || meet(here, there) {
                            continue;
                        }
                        let (lo, hi) = if mains[here].2 < mains[there].2 { (here, there) } else { (there, here) };
                        let mid = Vec2::new(cx as f32 + (sx + nx) as f32 * 0.5 / SAMPLES as f32, cy as f32 + (sy + ny) as f32 * 0.5 / SAMPLES as f32);
                        seams.entry((mains[lo].2, mains[hi].2)).or_insert((lo, Vec::new())).1.push(mid);
                    }
                }
            }
        }
        for (q, (&(id, other), (i, points))) in seams.iter().enumerate() {
            if points.len() < 3 {
                continue;
            }
            // the line through them: their middle and the way they spread most
            let mid = points.iter().copied().sum::<Vec2>() / points.len() as f32;
            let (mut xx, mut xy, mut yy) = (0.0f32, 0.0f32, 0.0f32);
            for p in points {
                let d = *p - mid;
                (xx, xy, yy) = (xx + d.x * d.x, xy + d.x * d.y, yy + d.y * d.y);
            }
            let d = Vec2::from_angle(0.5 * (2.0 * xy).atan2(xx - yy));
            let (t0, t1) = points.iter().map(|p| (*p - mid).dot(d)).fold((f32::MAX, f32::MIN), |(lo, hi), t| (lo.min(t), hi.max(t)));
            let pad = 1.0 / SAMPLES as f32;
            let (a, b, ..) = mains[*i];
            let u = (b - a).normalize_or_zero();
            let laid = Laid { cell: c, pattern: e.pattern, walls: e.walls, year: e.year, o: a, u, n: Vec2::new(-u.y, u.x), circle: false, between: Some(other) };
            out.push(Way { a: mid + d * (t0 - pad), b: mid + d * (t1 + pad), width: 0.016, id: id_of(c, slot_of(&reach, id), 1000 + q.min(23)), road: false, kind: 6, main: id, since: e.since, estate: Some(laid) });
        }
    }
    (out, blocks)
}

/// A way's place among the ways a square is laid from, in the order of their numbers (`id_of`).
fn slot_of(reach: &std::collections::BTreeMap<(i32, i8), (usize, f32, f32, f32)>, id: i32) -> usize {
    let mut ids: Vec<i32> = reach.keys().map(|k| k.0).collect();
    ids.dedup();
    ids.iter().position(|&s| s == id).unwrap_or(0)
}

/// `ISOSTATE_LOTS=NAME`: say of the town of that name in year `k` how its
/// streets stand (`town-streets` notes, the outcome): how many of its filled
/// squares have streets of their own, its estates by pattern and by the year
/// they were laid out, the length of its streets by where they came from and
/// by the decade they were laid, the share of its buildings that front a
/// street, and on its dearest squares the share of street front that is built.
#[allow(clippy::too_many_arguments)]
pub(super) fn tell(r: &Replay, g: &Ground, k: usize, land: &world::YearLand, estates: &HashMap<u32, Estate>, laid: &[(u32, Source, i32, f32)], lanes: &[(Vec<Vec2>, f32)], lane_of: &[LaneOf], standing: &[Stood], drawn: &[lots::Drawn], rects: &[lots::Footprint], infill: &[lots::Footprint], clear: &dyn Fn(f32, f32) -> bool) {
    let Ok(name) = std::env::var("ISOSTATE_LOTS") else { return };
    let Some(t) = r.header.towns.iter().position(|t| t.name == name) else { return };
    let town = &land.town;
    let year = r.y0 + k as i32;
    let cell_of = |p: Vec2| ((p.y.floor().max(0.0) as usize).min(r.h - 1) * r.w + (p.x.floor().max(0.0) as usize).min(r.w - 1)) as u32;
    let ours = |c: u32| r.catch.get(k * r.n + c as usize).copied() == Some(t as u8 + 1);
    // the filled squares, and those with an estate's street drawn on them
    let mut with_streets: std::collections::BTreeSet<u32> = Default::default();
    for ((line, _), of) in lanes.iter().zip(lane_of) {
        if of.estate.is_some() && !of.circle {
            with_streets.extend(line.iter().map(|&p| cell_of(p)).filter(|&c| estates.contains_key(&c)));
        }
    }
    let filled: Vec<u32> = town.cells.iter().filter(|&(&c, cell)| cell.filled && ours(c)).map(|(&c, _)| c).collect();
    let after = filled.iter().filter(|&&c| r.joined(k, c as usize).is_some_and(|j| j > lots::FROM)).count();
    let own = filled.iter().filter(|c| with_streets.contains(c)).count();
    let mut by_pattern: std::collections::BTreeMap<String, usize> = Default::default();
    let mut by_decade: std::collections::BTreeMap<String, usize> = Default::default();
    let (mut lo, mut hi, mut sum, mut late) = (f32::MAX, 0.0f32, 0.0f32, 0);
    let ours_e: Vec<(&u32, &Estate)> = estates.iter().filter(|&(&c, _)| ours(c)).collect();
    for &(&c, e) in &ours_e {
        *by_pattern.entry(e.pattern.name().to_string()).or_default() += 1;
        *by_decade.entry(format!("{}s", e.year / 10 * 10)).or_default() += 1;
        (lo, hi, sum) = (lo.min(e.value), hi.max(e.value), sum + e.value);
        late += r.joined(k, c as usize).is_some_and(|j| e.year > j + 2) as usize;
    }
    let list = |m: &std::collections::BTreeMap<String, usize>| m.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", ");
    eprintln!(
        "streets {year}: {name}: {} filled squares ({after} of them joined after 1900), {own} with streets of their own; {} estates: {}; laid out {}; their ground worth {:.2} to {:.2} times its town's of 1900 the year they were laid out (mean {:.2}), {} of them walls; {late} filled more than two years after they joined",
        filled.len(),
        ours_e.len(),
        list(&by_pattern),
        list(&by_decade),
        if ours_e.is_empty() { 0.0 } else { lo },
        hi,
        sum / ours_e.len().max(1) as f32,
        ours_e.iter().filter(|e| e.1.walls).count(),
    );
    // how much of the dry ground of its filled squares is works' ground (parcels the land gives
    // to works), and how much of that is a works' yard as drawn, or a churchyard or the railway
    // (what keeps a house out): sampled 24 times by 24 a square
    let (mut dry, mut works, mut held, mut town_held) = (0usize, 0usize, 0usize, 0usize);
    for &c in &filled {
        let (cx, cy) = ((c as usize % r.w) as f32, (c as usize / r.w) as f32);
        for j in 0..24 {
            for i in 0..24 {
                let (x, z) = (cx + (i as f32 + 0.5) / 24.0, cy + (j as f32 + 0.5) / 24.0);
                if world::height_at(g, x, z) < 0.05 {
                    continue;
                }
                dry += 1;
                let (parcel, _) = world::close_at(g, x, z);
                let in_yard = clear(x, z);
                if land.class.get(parcel as usize) == Some(&world::WORKS) {
                    works += 1;
                    held += in_yard as usize;
                } else {
                    town_held += in_yard as usize;
                }
            }
        }
    }
    eprintln!(
        "streets {year}: {name}: of the dry ground of its filled squares, {:.0} per cent is works' ground ({:.0} per cent in a yard, churchyard or the railway as drawn, {:.0} per cent plain); of the rest {:.0} per cent lies in a yard, churchyard or the railway",
        100.0 * works as f32 / dry.max(1) as f32,
        100.0 * held as f32 / dry.max(1) as f32,
        100.0 * (works - held) as f32 / dry.max(1) as f32,
        100.0 * town_held as f32 / dry.max(1) as f32
    );
    // each estate, for pointing a camera at one
    let mut each: Vec<(u32, &Estate)> = ours_e.iter().map(|&(&c, e)| (c, e)).collect();
    each.sort_by_key(|e| e.0);
    let each: Vec<String> = each.iter().map(|(c, e)| format!("{},{} {} {:.2}{}", *c as usize % r.w, *c as usize / r.w, e.year, e.value, match e.pattern {
        Pattern::ByLaw => " by-law",
        Pattern::Garden => " garden",
        Pattern::Loop => " loop",
        Pattern::Closes => " closes",
        Pattern::Blocks => " blocks",
    })).collect();
    eprintln!("streets {year}: {name}: the estates: {}", each.join("; "));
    // the streets laid, by where they came from and the decade they were laid (km: a square is
    // 2 km): the roads where they run through its ground, the lanes and the estates' streets as
    // they are drawn (a lane only where houses front it)
    let mut by_source: std::collections::BTreeMap<Source, (f32, std::collections::BTreeMap<i32, f32>)> = Default::default();
    let mut add = |source: Source, since: i32, len: f32| {
        let e = by_source.entry(source).or_default();
        e.0 += len * 2.0;
        *e.1.entry(since.max(1750) / 10 * 10).or_default() += len * 2.0;
    };
    for &(_, source, since, len) in laid.iter().filter(|l| ours(l.0)) {
        add(source, since, len);
    }
    for ((line, _), of) in lanes.iter().zip(lane_of).filter(|(_, of)| !of.circle) {
        for w in line.windows(2) {
            if ours(cell_of((w[0] + w[1]) * 0.5)) {
                add(of.source, of.year, w[0].distance(w[1]));
            }
        }
    }
    for (source, (km, decades)) in &by_source {
        let d: Vec<String> = decades.iter().filter(|&(_, &km)| km >= 0.05).map(|(d, km)| format!("{d}s {km:.1}")).collect();
        eprintln!("streets {year}: {name}: {} {km:.1} km, by the decade laid: {}", source.name(), d.join(", "));
    }
    // every street's line, and the roads', for asking what is near a spot
    let cen = crate::network::centres(r);
    let mut segs: Vec<(Vec2, Vec2, f32)> = crate::network::road_segments(r, k).into_iter().map(|(a, b, lv)| (Vec2::new(cen[a].0, cen[a].1), Vec2::new(cen[b].0, cen[b].1), if lv == 3 { 0.017 } else { 0.011 })).collect();
    for ((line, w), of) in lanes.iter().zip(lane_of) {
        if !of.circle {
            segs.extend(line.windows(2).map(|p| (p[0], p[1], w * 0.5)));
        }
    }
    let mut grid: HashMap<(i32, i32), Vec<u32>> = HashMap::default();
    for (i, &(a, b, _)) in segs.iter().enumerate() {
        let (lo, hi) = ((a.min(b) - Vec2::splat(0.06)) * 10.0, (a.max(b) + Vec2::splat(0.06)) * 10.0);
        for y in lo.y.floor() as i32..=hi.y.floor() as i32 {
            for x in lo.x.floor() as i32..=hi.x.floor() as i32 {
                grid.entry((x, y)).or_default().push(i as u32);
            }
        }
    }
    // how far a spot is from the edge of the nearest street, leaving one out
    let edge_off = |p: Vec2, but: Option<usize>| -> f32 {
        grid.get(&((p.x * 10.0).floor() as i32, (p.y * 10.0).floor() as i32)).map_or(1.0, |v| v.iter().filter(|&&i| Some(i as usize) != but).map(|&i| off_line(p, segs[i as usize].0, segs[i as usize].1) - segs[i as usize].2).fold(1.0, f32::min))
    };
    // the buildings, and those that front a street: the middle of the front within 0.03 of a street's edge
    let fronts = |f: &lots::Footprint| edge_off(f.at + f.front * f.half.1, None) < 0.03;
    let mine: Vec<&lots::Footprint> = rects.iter().filter(|f| ours(cell_of(f.at))).collect();
    let mine_infill: Vec<&lots::Footprint> = infill.iter().filter(|f| ours(cell_of(f.at))).collect();
    let (built, built_front) = (mine.len(), mine.iter().filter(|f| fronts(f)).count());
    let (filled_in, filled_front) = (mine_infill.len(), mine_infill.iter().filter(|f| fronts(f)).count());
    let on_estates = mine_infill.iter().filter(|f| estates.contains_key(&cell_of(f.at))).count();
    let blocks = standing.iter().zip(drawn).filter(|(s, d)| s.lot.kind == 8 && matches!(d, lots::Drawn::Form(..)) && ours(s.lot.cell)).count();
    eprintln!(
        "streets {year}: {name}: {} buildings ({built} on plots and blocks, {blocks} of them estates' whole blocks; {filled_in} built over closes by themselves, {on_estates} of those on estates), {} of them fronting a street ({:.0} per cent; on plots {:.0}, closes {:.0})",
        built + filled_in,
        built_front + filled_front,
        100.0 * (built_front + filled_front) as f32 / (built + filled_in).max(1) as f32,
        100.0 * built_front as f32 / built.max(1) as f32,
        100.0 * filled_front as f32 / filled_in.max(1) as f32
    );
    // the street front of the dearest squares: every 0.01 along each street drawn there, on
    // each side just behind the footway, whether a building stands; not counted where another
    // street meets it, nor over a park, open ground, the water, or a works' yard, a churchyard or
    // the railway (what keeps a house out)
    let all: Vec<&lots::Footprint> = rects.iter().chain(infill).collect();
    let mut by_bucket: HashMap<(i32, i32), Vec<u32>> = HashMap::default();
    for (i, f) in all.iter().enumerate() {
        let reach = f.half.0.max(f.half.1);
        let (lo, hi) = ((f.at - Vec2::splat(reach)) * 10.0, (f.at + Vec2::splat(reach)) * 10.0);
        for y in lo.y.floor() as i32..=hi.y.floor() as i32 {
            for x in lo.x.floor() as i32..=hi.x.floor() as i32 {
                by_bucket.entry((x, y)).or_default().push(i as u32);
            }
        }
    }
    let stands = |p: Vec2| by_bucket.get(&((p.x * 10.0).floor() as i32, (p.y * 10.0).floor() as i32)).is_some_and(|v| v.iter().any(|&i| all[i as usize].holds(p)));
    let mut closes = world::CloseLookup::new(g);
    let mut front = |pick: &dyn Fn(u32) -> bool| -> (usize, usize, usize) {
        let (mut sides, mut built) = (0, 0);
        let squares = town.cells.keys().filter(|&&c| ours(c) && pick(c)).count();
        for (i, &(a, b, half)) in segs.iter().enumerate() {
            let len = a.distance(b);
            if len < 1e-4 {
                continue;
            }
            let (u, steps) = ((b - a) / len, (len / 0.01).ceil() as usize);
            let n = Vec2::new(-u.y, u.x);
            for j in 0..steps {
                let p = a + u * (len * (j as f32 + 0.5) / steps as f32);
                if !pick(cell_of(p)) || !ours(cell_of(p)) {
                    continue;
                }
                for side in [-1.0f32, 1.0] {
                    let q = p + n * side * (half + 0.018);
                    if edge_off(q, Some(i)) < 0.004 || world::water_dist(g, q.x, q.y) < 0.03 || clear(q.x, q.y) {
                        continue;
                    }
                    let (parcel, sub) = closes.at(q.x, q.y);
                    if town.park_at(parcel, sub) || !town.built_up(parcel, sub) {
                        continue;
                    }
                    sides += 1;
                    built += stands(q) as usize;
                }
            }
        }
        (squares, sides, built)
    };
    let blocks_ground = |c: u32| estates.get(&c).is_some_and(|e| e.pattern == Pattern::Blocks);
    let dear = |c: u32| r.builds.value(r.n, k, c as usize).is_some_and(|v| v >= lots::BLOCK_VALUE);
    let walls = |c: u32| estates.get(&c).is_some_and(|e| e.walls);
    let dear_estates = |c: u32| estates.contains_key(&c) && dear(c);
    for (what, (squares, sides, built)) in [
        ("estates on blocks", front(&blocks_ground)),
        ("estates whose fronts are walls", front(&walls)),
        ("estates worth three times their town's of 1900 this year", front(&dear_estates)),
        ("squares worth three times their town's of 1900 this year", front(&dear)),
    ] {
        eprintln!("streets {year}: {name}: street front built on its {what} ({squares} squares): {:.0} per cent of {:.1} km of street side", 100.0 * built as f32 / sides.max(1) as f32, sides as f32 * 0.01 * 2.0);
    }
}
