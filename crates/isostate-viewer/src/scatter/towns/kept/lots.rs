//! The lots of a town after 1900: each is built and rebuilt by technology,
//! year and land value, from the simulation's own figures.
//!
//! On 20 November 2026 the designer said of the look after 1900: "I mean, I
//! think it should reflect technology, year, and land value no? These are
//! assets that then can be used by the sim". The assets are the building
//! forms of `isostate_gen::forms`, each saying its years, its storeys and what
//! its builders must know; here the simulation chooses among them. The rule:
//!
//! 1. **A lot is built in the form of its year.** A lot first built after
//!    1900 (`FROM`) is built as `forms::choose` answers for that year, what
//!    its town knew that year and the storeys its square's land value asks
//!    (`storeys_for`). A lot built by 1900 keeps the house it had in 1900:
//!    what a house grew into after 1900 by the older rules (a second storey,
//!    a townhouse's front) is not done any more (`Story::most_to`).
//! 2. **Land value sets the storeys; technology caps them; the year picks
//!    the style.** The storeys asked are two on ground of its town's value
//!    of 1900, rising as the land value's power three quarters (`RISE`; the
//!    record's `value`: the square's rent a unit of room over its town's rent
//!    of 1900, in the money of the day), at least one. Before the town knows the
//!    steel frame, or reinforced concrete and the lift together, no more than
//!    five. The forms hold the storeys to what each stands.
//! 3. **A square is rebuilt only when it must hold more.** The record's
//!    `dens` says how many times its room of 1900 the square holds. The
//!    standing buildings hold, on the mean of the square's lots, 1 for a
//!    house and half its storeys for a form of two storeys or more, half as
//!    much again on plots thrown together, whose gardens and yards it covers
//!    (`COVERS`). In a year
//!    `dens` stands above what they hold, the square's lots are rebuilt one
//!    group at a time in the form of that year until they hold it, and no
//!    more; a building comes down only for one that holds twice as much
//!    (`TWICE`), and no more than a twentieth of a square's plots are rebuilt
//!    in a year (`A_YEAR`). Nothing is rebuilt for fashion: a square that
//!    never grew dear looks in 2050 as it did when it was built.
//! 4. **Which go first.** The lots that hold least; among those, the
//!    frontage of a road or a market place before the back lanes (that is
//!    where the shops and the dearest ground are, and where English high
//!    streets were rebuilt first); then the oldest building; then a number of
//!    the lot's own. So a town's back lanes keep their cottages longest, and
//!    a lot rebuilt in 1925 stands until every older house of its square has
//!    gone.
//! 5. **What goes up together** (`units`). The plots of a row (the back
//!    lanes are built a row at a time, and a terrace is one building) are
//!    built and rebuilt together; on a road or round a market place up to
//!    four plots side by side are thrown together (`FRONTAGE`), all of them
//!    standing since 1900 or all built after (a plot that comes to such a
//!    group later is taken into the form it was built as). A row of one plot
//!    asks for a form on a house lot, any other group for one on lots thrown
//!    together.
//! 7. **Height follows the street** (26 November 2026). Towers go to the
//!    main streets and the town's middle, and a street's front rises
//!    together: a main street's frontage (a road through or a market place)
//!    asks storeys as if its ground were half as dear again as its square's
//!    (`MAIN_STREET`), and a lot elsewhere, outside its town's middle
//!    (`in_middle`), no more than six (`MIDDLE_STOREYS`: terraces and mansion
//!    blocks), with no form chosen that stands higher (`forms::choose_within`);
//!    the offices and the closes built over by themselves likewise. When a
//!    group is rebuilt, the rest of its front (the same street, side and
//!    stretch of `STRETCH` plots) goes up with it in the same form to the
//!    same height, wherever what stands would come down for it. So a street
//!    has a wall, and no tower stands among the cottages of its own street.
//!
//! The same record always draws the same town: all of it is worked out from
//! the record's years up to the one drawn, whichever year is drawn and
//! however it was come to. The year a lot was built still comes from the
//! houses' own rule (`Story::year_of`); the number of plots a square's houses
//! take now follows its residents over its `dens` (`Replay::footprint`), so a
//! square that holds more rises rather than spreads.

use super::*;
use isostate_gen::forms::{self, Know, Knows, Lot as FormLot, Shape, Use};
use isostate_gen::Model;

/// The last year the houses' own rules build by: a lot first built after it
/// is built as a form, and a lot built by it keeps its house until rebuilt.
pub(crate) const FROM: i32 = 1900;

/// A building is pulled down only for one that holds this many times as
/// much (rule 3): a little more room is not worth a house.
pub(crate) const TWICE: f32 = 2.0;

/// The most of a square's plots rebuilt in one year: a twentieth, what its
/// builders can manage beside everything else they build (rule 3).
pub(crate) const A_YEAR: f32 = 0.05;

/// What a building of the forms holds, in the square's room of 1900 to a
/// plot: half its storeys, and half as much again for one on plots thrown
/// together, which covers the gardens and yards behind the houses it stands
/// for as well as the houses (`COVERS`); never less than a house.
fn holds(storeys: u8, joined: bool) -> f32 {
    (storeys as f32 * 0.5 * if joined { COVERS } else { 1.0 }).max(1.0)
}

/// How much more of its ground a building on plots thrown together covers
/// than the houses did (rule 3).
pub(crate) const COVERS: f32 = 1.5;

/// How the storeys a lot asks for follow its land value: as its power
/// `RISE`, two storeys on ground of its town's value of 1900. Land four
/// times as dear asks for not eight storeys but six: a builder spends more on
/// a dearer plot, but each storey higher costs more than the last (the lift,
/// the frame, the deeper footing), so the height rises less than the value.
/// At 1 (in proportion) every rebuilt lot of a city whose ground is all four
/// or five times as dear was a tower of eight to ten storeys, a forest of
/// them from edge to edge (`look-follows` notes, the first pictures).
pub(crate) const RISE: f32 = 0.75;

/// The most storeys the town can build: five before it knows the steel frame,
/// or reinforced concrete and the lift together (a wall of brick bears no
/// more, and nobody climbs further), forty after.
pub(crate) fn most_storeys(knows: Knows) -> u8 {
    let tall = knows.has(Know::SteelFrame) || (knows.has(Know::Concrete) && knows.has(Know::Lift));
    if tall { 40 } else { 5 }
}

/// The storeys a lot's land value asks for, as far as the town's knowledge
/// lets it build (rule 2).
pub(crate) fn storeys_for(value: f32, knows: Knows) -> u8 {
    let asked = (2.0 * value.max(0.0).powf(RISE)).round().clamp(1.0, 40.0) as u8;
    asked.min(most_storeys(knows))
}

/// A form a group of plots is built as: its shape, its storeys and the year it went up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Built {
    pub shape: Shape,
    pub storeys: u8,
    pub year: i32,
}

/// A group of plots that is built and rebuilt together, as the walk sees it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Group {
    /// how many plots
    pub plots: usize,
    /// the year its first building went up
    pub built: i32,
    /// on a road or round a market place
    pub front: bool,
    /// a number of its own, 0 to 1
    pub pick: f32,
    /// plots thrown together, built as one on more than a house lot: a row,
    /// or a group of a road's or a market place's frontage however many of
    /// its plots stand yet (so that its form does not change as they come)
    pub joined: bool,
    /// an estate's whole block between four streets (`estates.rs`): built
    /// and rebuilt as a form on a whole block
    pub block: bool,
    /// its front: the stretch of street it stands on (`STRETCH`), whose
    /// groups are rebuilt together to one height (rule 7); a number among
    /// its square's
    pub wall: u32,
    /// it may rise tall: it fronts a main street (a road through or a market
    /// place), or stands in its town's middle (rule 7)
    pub tall: bool,
}

/// The most storeys a lot asks for away from the main streets and its
/// town's middle (rule 7): six, a mansion block's, whatever its land is
/// worth. Dear ground there gives middle density, terraces and blocks of
/// four to six floors; the towers go to the main streets and the middle.
pub(crate) const MIDDLE_STOREYS: u8 = 6;

/// How many plots along a street make one front, rebuilt together (rule 7):
/// eight, some 0.4 of a square, about a street block between cross streets.
pub(crate) const STRETCH: i32 = 8;

/// How far from its town's market place, in squares, a square is its
/// middle, where towers may stand off the main streets (rule 7).
pub(crate) const MIDDLE_REACH: f32 = 1.5;

/// How much dearer a main street's frontage is than its square's ground on
/// the mean (rule 7): half as dear again. The record keeps one land value a
/// square; the frontage of a road through or a market place, where the trade
/// passes, lets for more than the back lanes behind it, and builds higher.
pub(crate) const MAIN_STREET: f32 = 1.5;

/// The storeys a group asks for (rules 2 and 7): on a main street's
/// frontage as its ground is worth `MAIN_STREET` times its square's;
/// elsewhere, outside its town's middle, no more than `MIDDLE_STOREYS`.
fn asked(value: f32, knows: Knows, tall: bool, main: bool) -> u8 {
    if super::estates::bare() {
        return storeys_for(value, knows);
    }
    storeys_for(if main { value * MAIN_STREET } else { value }, knows).min(most(tall))
}

/// The most storeys a building may be built to (rule 7): as many as asked
/// on a main street or in the town's middle, `MIDDLE_STOREYS` elsewhere; a
/// form that stands no lower than that is not chosen there
/// (`forms::choose_within`).
fn most(tall: bool) -> u8 {
    if tall || super::estates::bare() { u8::MAX } else { MIDDLE_STOREYS }
}

/// One year of a square's history: the year, how densely the square is
/// built, its land value, and what its town knew.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Year {
    pub year: i32,
    pub dens: f32,
    pub value: f32,
    pub knows: Knows,
}

/// Walk a square's years after 1900 and say what each group stands as at
/// the end: None for the buildings it had by 1900, else the form it was last
/// built or rebuilt as (rules 1, 3 and 4). `years` runs from 1901 up to the
/// year drawn, in order. A pure function of what it is given.
#[cfg(test)]
pub(crate) fn walk(groups: &[Group], years: &[Year]) -> Vec<Option<Built>> {
    walk_with(groups, &vec![Vec::new(); groups.len()], years).form
}

/// The land value from which a group of a way's frontage that is rebuilt is
/// rebuilt as a whole block between its street and the lane behind
/// (`Lot::Block`): three times its town's of 1900. Below it the plots of a
/// frontage are thrown together a few at a time (`FRONTAGE`). Three is about
/// the middle of island 7's first city in 1950 (value 4.0 at its market
/// place, 2.5 to 3 a mile out), where London's and Glasgow's blocks of
/// mansion flats went up a lifetime after their terraces.
pub(crate) const BLOCK_VALUE: f32 = 3.0;

/// What the walk leaves of a square: the form each group was last built or
/// rebuilt as (`form`), and for a group a whole block was built over, the
/// group whose block it is (`by`; that group's form stands on it).
pub(crate) struct Walked {
    pub form: Vec<Option<Built>>,
    pub by: Vec<Option<usize>>,
}

/// `walk`, where a group may be rebuilt as a whole block (rule 6). For each
/// group, `partners` are the other groups whose plots lie on the ground its
/// block would take; a group with none is never a block. Each year it is
/// rebuilt on ground worth `BLOCK_VALUE` or more, a group with partners asks
/// for a form on a whole block, if every partner standing would come down for
/// it (it holds twice as much as each does) and none is a block's itself or
/// under another's; then the partners are under its block from that year, and
/// a partner built later never goes up. Else it is rebuilt as before.
pub(crate) fn walk_with(groups: &[Group], partners: &[Vec<usize>], years: &[Year]) -> Walked {
    let n = groups.len();
    let mut form: Vec<Option<Built>> = vec![None; n];
    let mut by: Vec<Option<usize>> = vec![None; n];
    let mut cap = vec![1.0f32; n];
    let mut since: Vec<i32> = groups.iter().map(|g| g.built).collect();
    let ask = |g: &Group, y: &Year, block: bool| {
        let lot = if block || g.block { FormLot::Block } else if g.joined { FormLot::Joined } else { FormLot::House };
        forms::choose_within(&forms::Ask { knows: y.knows, year: y.year, storeys: asked(y.value, y.knows, g.tall, g.front), lot, use_: Use::Dwellings, pick: g.pick }, most(g.tall))
    };
    let together = !super::estates::bare();
    for y in years {
        // the groups first built this year are built as the form of the year (and any
        // built after 1900 in a year the walk was not given, in the first it is); one on
        // ground a block has taken never goes up, and holds what the block does
        for (i, g) in groups.iter().enumerate() {
            if let Some(o) = by[i] {
                cap[i] = cap[o];
                continue;
            }
            if g.built > FROM && g.built <= y.year && form[i].is_none() {
                let a = ask(g, y, false);
                form[i] = Some(Built { shape: a.shape, storeys: a.storeys, year: y.year.max(g.built) });
                cap[i] = holds(a.storeys, g.joined);
                since[i] = y.year.max(g.built);
            }
        }
        // what the square's standing buildings hold, on the mean of its plots
        let standing: Vec<usize> = (0..n).filter(|&i| groups[i].built <= y.year).collect();
        let plots: usize = standing.iter().map(|&i| groups[i].plots).sum();
        if plots == 0 {
            continue;
        }
        let mut held: f32 = standing.iter().map(|&i| cap[i] * groups[i].plots as f32).sum::<f32>() / plots as f32;
        // rebuilt, a group at a time, until they hold what the square holds, and no more of
        // it in a year than its builders can (`A_YEAR`)
        let mut rebuilt = 0;
        while y.dens > held + 0.005 && (rebuilt as f32) < A_YEAR * plots as f32 {
            let Some(&i) = standing.iter().filter(|&&i| by[i].is_none()).min_by(|&&a, &&b| {
                cap[a].total_cmp(&cap[b]).then(groups[b].front.cmp(&groups[a].front)).then(since[a].cmp(&since[b])).then(groups[a].pick.total_cmp(&groups[b].pick))
            }) else {
                break;
            };
            // a whole block, where its ground is dear enough and all on it would come down for it
            let owner = |j: usize| by.contains(&Some(j));
            let block = y.value >= BLOCK_VALUE && !partners[i].is_empty() && {
                let more = holds(ask(&groups[i], y, true).storeys, true);
                partners[i].iter().all(|&j| by[j].is_none_or(|o| o == i) && !owner(j) && (groups[j].built > y.year || more >= TWICE * cap[j]))
            };
            let a = ask(&groups[i], y, block);
            let more = holds(a.storeys, groups[i].joined || block);
            if more < TWICE * cap[i] {
                // the year's form would not hold twice what the least of the square holds: not
                // worth pulling down for, and the square stays crowded until its ground pays
                break;
            }
            rebuilt += groups[i].plots;
            held += (more - cap[i]) * groups[i].plots as f32 / plots as f32;
            cap[i] = more;
            form[i] = Some(Built { shape: a.shape, storeys: a.storeys, year: y.year });
            since[i] = y.year;
            // the rest of its front goes up with it, in the same form to the same height,
            // wherever what stands would come down for it: a street gets a wall, not a
            // tower among cottages (rule 7)
            if together && !block && forms::spec(a.shape).lot != FormLot::Block {
                for j in 0..n {
                    let gj = &groups[j];
                    if j == i || gj.wall != groups[i].wall || gj.built > y.year || by[j].is_some() || owner(j) || gj.block {
                        continue;
                    }
                    let theirs = holds(a.storeys, true);
                    if theirs < TWICE * cap[j] {
                        continue;
                    }
                    rebuilt += gj.plots;
                    held += (theirs - cap[j]) * gj.plots as f32 / plots as f32;
                    cap[j] = theirs;
                    form[j] = Some(Built { shape: a.shape, storeys: a.storeys, year: y.year });
                    since[j] = y.year;
                }
            }
            if block {
                for &j in &partners[i] {
                    if by[j].is_none() && groups[j].built <= y.year {
                        rebuilt += groups[j].plots;
                        held += (more - cap[j]) * groups[j].plots as f32 / plots as f32;
                    }
                    by[j] = Some(i);
                    form[j] = None;
                    cap[j] = more;
                    since[j] = y.year;
                }
            }
        }
    }
    Walked { form, by }
}

/// What each house standing in a year is drawn as.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Drawn {
    /// the house `what_stands` gives it
    House,
    /// the first plot of a group built as a form: the form, and the group's plots in `standing`
    Form(Built, usize),
    /// another plot of such a group: the form stands on it
    Covered,
}

/// How many plots of a road's or a market place's frontage standing since
/// 1900 are thrown together at the most (rule 5): four, some 200 m, a block
/// of flats over shops or two side by side.
const FRONTAGE: i32 = 4;

/// The groups of plots that are built and rebuilt together (rule 5): for
/// each, the places in `standing` (sorted street by street and along each,
/// as `find_neighbours` leaves it) of its plots, in order along the street.
fn units(standing: &[Stood]) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < standing.len() {
        let s = &standing[i].lot;
        let along = |j: usize| j < standing.len() && standing[j].lot.pl.run == s.pl.run && standing[j].lot.cell == s.cell;
        let mut j = i + 1;
        if let Some(ri) = s.pl.row {
            while along(j) && standing[j].lot.pl.row == Some(ri) {
                j += 1;
            }
        } else {
            // up to four plots side by side, from a place along the street fixed by its own
            // number, all built by 1900 or all after
            let old = standing[i].year <= FROM;
            while along(j) && standing[j].lot.pl.row.is_none() && standing[j].lot.pl.index == standing[j - 1].lot.pl.index + 1 && (standing[j].year <= FROM) == old && standing[j].lot.pl.index.div_euclid(FRONTAGE) == s.pl.index.div_euclid(FRONTAGE) {
                j += 1;
            }
        }
        out.push(i..j);
        i = j;
    }
    out
}

/// What each house standing in year `k` is drawn as: the house as before, or
/// a form (`walk`). Nothing changes in a run that does not say how densely
/// its squares are built, nor in a year to 1900. No group is a whole block
/// here (`drawn_on` lays blocks out on the ground).
#[cfg(test)]
pub(super) fn drawn(r: &Replay, k: usize, standing: &[Stood]) -> Vec<Drawn> {
    drawn_with(r, k, standing, None).drawn
}

/// How deep a whole block between a way and the lane behind it is drawn:
/// from the plots' front line to just short of the back lane, which runs
/// 0.13 behind the way's middle (`BACK_LANE`).
pub(crate) const BLOCK_DEPTH: f32 = 0.1;

/// The ground a whole block stands on: a rectangle, its middle, the way
/// along its street, the way its front faces, and its half width and depth.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Footprint {
    pub at: Vec2,
    pub along: Vec2,
    pub front: Vec2,
    pub half: (f32, f32),
}

impl Footprint {
    pub fn holds(&self, p: Vec2) -> bool {
        let d = p - self.at;
        d.dot(self.along).abs() <= self.half.0 && d.dot(self.front).abs() <= self.half.1
    }
}

/// What `drawn_on` says of a year's houses: how each is drawn, and the
/// ground of each whole block drawn, with the plots it took in (for the lanes
/// to keep out of, and for `ISOSTATE_LOTS`).
pub(super) struct Lots {
    pub drawn: Vec<Drawn>,
    pub blocks: Vec<(Footprint, usize)>,
}

/// The ground a group of plots would be rebuilt over as a whole block, if it
/// may be (rule 6): the frontage of a way between two squares, three plots or
/// more, the block's ground wholly in its own square, dry, and clear of the
/// works' yards, the churchyards, the market places, the railway (`clear`) and
/// the roads.
fn block_ground(g: &Ground, standing: &[Stood], range: &std::ops::Range<usize>, clear: &dyn Fn(f32, f32) -> bool, roads: &[(Vec2, Vec2)]) -> Option<Footprint> {
    let s = &standing[range.start];
    if s.lot.kind != 0 || s.lot.pl.row.is_some() || range.len() < 3 || s.lot.on_estate {
        return None;
    }
    let (a, z) = (&s.lot.pl, &standing[range.end - 1].lot.pl);
    let front = Vec2::new(a.yaw.sin(), a.yaw.cos());
    let run = z.p - a.p;
    let along = if run.length() > 1e-4 { run.normalize() } else { return None };
    let width = run.length() + a.width;
    let at = (a.p + z.p) * 0.5 + front * (a.depth * 0.5 - BLOCK_DEPTH * 0.5);
    let f = Footprint { at, along, front, half: (width * 0.5, BLOCK_DEPTH * 0.5) };
    let cell = s.lot.cell as usize;
    let w = g.w;
    for i in 0..5 {
        for j in 0..4 {
            let p = at + along * (width * (i as f32 / 4.0 - 0.5) * 0.96) + front * (BLOCK_DEPTH * (j as f32 / 3.0 - 0.5) * 0.8);
            let here = (p.y.max(0.0) as usize).min(g.h - 1) * w + (p.x.max(0.0) as usize).min(w - 1);
            if here != cell || clear(p.x, p.y) || world::height_at(g, p.x, p.y) < 0.05 || world::water_dist(g, p.x, p.y) < 0.03 {
                return None;
            }
            if roads.iter().any(|&(ra, rb)| super::off_line(p, ra, rb) < 0.022) {
                return None;
            }
        }
    }
    Some(f)
}

/// `drawn`, with the groups that may be rebuilt as whole blocks laid out on
/// the ground: `clear` says where a block may not stand (what keeps a house
/// out), and the roads of the year are read from the record.
pub(super) fn drawn_on(r: &Replay, g: &Ground, k: usize, standing: &[Stood], clear: &dyn Fn(f32, f32) -> bool) -> Lots {
    drawn_with(r, k, standing, (!plain()).then_some((g, clear)))
}

fn drawn_with(r: &Replay, k: usize, standing: &[Stood], ground: Option<(&Ground, &dyn Fn(f32, f32) -> bool)>) -> Lots {
    let mut out = vec![Drawn::House; standing.len()];
    let mut blocks = Vec::new();
    let now = r.y0 + k as i32;
    if !r.builds.kept() || now <= FROM {
        return Lots { drawn: out, blocks };
    }
    let units = units(standing);
    let mut by_cell: HashMap<u32, Vec<usize>> = HashMap::default();
    for (u, range) in units.iter().enumerate() {
        by_cell.entry(standing[range.start].lot.cell).or_default().push(u);
    }
    let roads: Vec<(Vec2, Vec2)> = match ground {
        Some(_) => {
            let cen = crate::network::centres(r);
            crate::network::road_segments(r, k).into_iter().map(|(a, b, _)| (Vec2::new(cen[a].0, cen[a].1), Vec2::new(cen[b].0, cen[b].1))).collect()
        }
        None => Vec::new(),
    };
    let first = (FROM + 1 - r.y0).max(0) as usize;
    let mut cells: Vec<(&u32, &Vec<usize>)> = by_cell.iter().collect();
    cells.sort_by_key(|c| *c.0);
    for (&c, list) in cells {
        let middle = in_middle(r, k, c as usize);
        // each group's front: its street, its side, and the stretch of it its first plot is on
        let mut walls: HashMap<(i32, u8, i32), u32> = HashMap::default();
        let groups: Vec<Group> = list
            .iter()
            .map(|&u| {
                let range = &units[u];
                let s = &standing[range.start];
                let block = s.lot.estate.is_some_and(|e| e.block);
                let next = walls.len() as u32;
                let wall = *walls.entry((s.lot.pl.run.0, s.lot.pl.run.1, s.lot.pl.index.div_euclid(STRETCH))).or_insert(next);
                let front = s.lot.road || s.lot.kind == 9;
                Group { plots: range.len(), built: range.clone().map(|i| standing[i].year).min().unwrap_or(s.year), front, pick: s.lot.pl.u, joined: range.len() > 1 || s.lot.pl.row.is_none() || block, block, wall, tall: front || middle }
            })
            .collect();
        // the ground each group's block would take, and the groups whose plots stand on it
        let grounds: Vec<Option<Footprint>> = list.iter().map(|&u| ground.and_then(|(g, clear)| block_ground(g, standing, &units[u], clear, &roads))).collect();
        let partners: Vec<Vec<usize>> = grounds
            .iter()
            .enumerate()
            .map(|(i, f)| f.map_or(Vec::new(), |f| (0..list.len()).filter(|&j| j != i && units[list[j]].clone().any(|p| f.holds(standing[p].lot.pl.p))).collect()))
            .collect();
        let years: Vec<Year> = (first..=k).map(|y| square_year(r, y, c as usize)).collect();
        let walked = walk_with(&groups, &partners, &years);
        for (i, &u) in list.iter().enumerate() {
            let range = &units[u];
            if walked.by[i].is_some() {
                for p in range.clone() {
                    out[p] = Drawn::Covered;
                }
            } else if let Some(b) = walked.form[i] {
                out[range.start] = Drawn::Form(b, range.len());
                for p in range.start + 1..range.end {
                    out[p] = Drawn::Covered;
                }
                if forms::spec(b.shape).lot == FormLot::Block {
                    if let Some(f) = grounds[i] {
                        let took = range.len() + walked.by.iter().zip(list).filter(|(o, _)| **o == Some(i)).map(|(_, &v)| units[v].len()).sum::<usize>();
                        blocks.push((f, took));
                    }
                }
            }
        }
    }
    Lots { drawn: out, blocks }
}

/// `ISOSTATE_LOTS=NAME`: say of the town of that name in year `k` how many
/// of its plots stand from each age (houses of 1900 or before, and forms by
/// the decade they went up), how many groups of old plots were rebuilt and
/// in which decade, and the storeys at its middle (its market place's square
/// and those next to it) and at its edge (the squares furthest from it).
/// `storeys` is what `what_stands` gives each house.
pub(super) fn tell(r: &Replay, k: usize, standing: &[Stood], storeys: &[u8], drawn: &[Drawn]) {
    let Ok(name) = std::env::var("ISOSTATE_LOTS") else { return };
    let Some(t) = r.header.towns.iter().position(|t| t.name == name) else {
        eprintln!("lots {}: no town called {name}", r.y0 + k as i32);
        return;
    };
    let node = r.header.towns[t].node;
    let off = |c: u32| ((c as usize % r.w) as f32 - (node % r.w) as f32).hypot((c as usize / r.w) as f32 - (node / r.w) as f32);
    let ours = |c: u32| r.catch.get(k * r.n + c as usize).copied() == Some(t as u8 + 1);
    let mut by_age: std::collections::BTreeMap<i32, usize> = Default::default();
    let mut rebuilt: std::collections::BTreeMap<i32, usize> = Default::default();
    // (storeys of each plot, and how far its square lies from the middle)
    let mut tall: Vec<(f32, u8)> = Vec::new();
    let mut form: Option<(Built, usize)> = None;
    for (i, s) in standing.iter().enumerate() {
        match drawn[i] {
            Drawn::Form(b, plots) => {
                form = Some((b, plots));
                if s.year <= FROM && ours(s.lot.cell) {
                    *rebuilt.entry(b.year / 10 * 10).or_default() += 1;
                }
            }
            Drawn::House => form = None,
            Drawn::Covered => {}
        }
        if !ours(s.lot.cell) {
            continue;
        }
        let (age, st) = match (drawn[i], form) {
            (Drawn::House, _) | (_, None) => (if s.year <= FROM { FROM } else { s.year / 10 * 10 }, storeys[i]),
            (_, Some((b, _))) => (b.year / 10 * 10, b.storeys),
        };
        *by_age.entry(age).or_default() += 1;
        tall.push((off(s.lot.cell), st));
    }
    let far = tall.iter().map(|t| t.0).fold(0.0, f32::max);
    let mean = |pick: &dyn Fn(f32) -> bool| {
        let v: Vec<u8> = tall.iter().filter(|t| pick(t.0)).map(|t| t.1).collect();
        (v.iter().map(|&s| s as f32).sum::<f32>() / v.len().max(1) as f32, v.iter().copied().max().unwrap_or(0), v.len())
    };
    let (mid, edge) = (mean(&|d| d < 1.5), mean(&|d| d >= far - 1.5));
    let ages: Vec<String> = by_age.iter().map(|(a, n)| if *a == FROM { format!("by 1900 {n}") } else { format!("{a}s {n}") }).collect();
    let again: Vec<String> = rebuilt.iter().map(|(a, n)| format!("{a}s {n}")).collect();
    eprintln!(
        "lots {}: {name}: {} plots; standing from each age: {}; groups of old plots rebuilt, by decade: {} ({} in all); storeys at the middle (within 1.5 squares) mean {:.1} highest {} over {} plots, at the edge (within 1.5 squares of the farthest, {far:.1} out) mean {:.1} highest {} over {} plots",
        r.y0 + k as i32, tall.len(), ages.join(", "), again.join(", "), rebuilt.values().sum::<usize>(), mid.0, mid.1, mid.2, edge.0, edge.1, edge.2
    );
}

/// Whether cell `c` is its town's middle in year `k`: within `MIDDLE_REACH`
/// squares of the market place of the town it trades in (rule 7).
pub(crate) fn in_middle(r: &Replay, k: usize, c: usize) -> bool {
    let t = (r.catch.get(k * r.n + c).copied().unwrap_or(0) as usize).wrapping_sub(1);
    r.header.towns.get(t).is_some_and(|t| ((c % r.w) as f32 - (t.node % r.w) as f32).hypot((c / r.w) as f32 - (t.node / r.w) as f32) <= MIDDLE_REACH)
}

/// Cell `c` in year `k` as the walk reads it.
fn square_year(r: &Replay, k: usize, c: usize) -> Year {
    let year = r.y0 + k as i32;
    let b = &r.builds;
    let town = (r.catch.get(k * r.n + c).copied().unwrap_or(0) as usize).wrapping_sub(1);
    let knows = Knows::of(year, |method| {
        let tech = match method {
            "steel_frame" => b.steel,
            "coal_power_station" => b.power,
            _ => None,
        };
        // (where the run does not say, the year that stands for it)
        b.knows(k, town, tech).unwrap_or_else(|| Know::ALL.iter().any(|k| k.method() == Some(method) && k.year() <= year))
    });
    Year { year, dens: b.dens(r.n, k, c), value: b.value(r.n, k, c).unwrap_or(1.0), knows }
}

/// The hands a storey of an office takes, on ground of no more than its
/// town's value of 1900. An office works of the content's usual size (1,000
/// a month, 330 hands) stands two storeys; at its largest (some 4,600 a
/// month, 1,530 hands) eight.
pub(crate) const HANDS_A_STOREY: f32 = 200.0;

/// An office works (the content's method `offices`) after 1900: the
/// building it stands in in year `k`, from the forms, or None where it is
/// drawn as the site planner draws every works. It is built the first year
/// it has hands, in that year's form for offices, its storeys its hands over
/// `HANDS_A_STOREY` times the square root of its square's land value (on
/// dear ground the same floors stand on less of it), at least two, held as
/// a lot's are by what its town knows. It is built again, in the form of
/// the year, only when it has come to want half as many storeys again as it
/// stands: so it does not change with every hand it takes on or lets go.
pub(crate) fn office(r: &Replay, f: usize, k: usize) -> Option<Built> {
    let now = r.y0 + k as i32;
    if !r.builds.kept() || now <= FROM {
        return None;
    }
    let firm = r.header.firms.get(f)?;
    let method = r.header.methods.get(firm.method)?;
    if method.name != "offices" {
        return None;
    }
    let labor = method.labor.unwrap_or(0.33);
    let pick = world::hash2(f as i32, 3, 44);
    let mut built: Option<Built> = None;
    let start = (firm.born.max(FROM + 1) - r.y0).max(0) as usize;
    for y in start..=k {
        let hands = replay::firm_k(r, f, y) * labor;
        if hands <= 0.0 {
            continue;
        }
        let sq = square_year(r, y, firm.cell);
        let floors = hands / HANDS_A_STOREY * sq.value.max(1.0).sqrt();
        let asked = (floors.round().min(40.0) as u8).max(2).min(most_storeys(sq.knows));
        // (outside its town's middle no higher than a mansion block, rule 7)
        let cap = most(in_middle(r, y, firm.cell));
        let asked = asked.min(cap);
        if built.is_some_and(|b| (asked as f32) <= b.storeys as f32 * 1.5) {
            continue;
        }
        let a = forms::choose_within(&forms::Ask { knows: sq.knows, year: sq.year, storeys: asked, lot: FormLot::Joined, use_: Use::Offices, pick }, cap);
        if built.is_none_or(|b| a.storeys > b.storeys) {
            built = Some(Built { shape: a.shape, storeys: a.storeys, year: sq.year });
        }
    }
    built
}

/// `ISOSTATE_LOOK=plain`: draw a town after 1900 as it was drawn before the
/// look of a filled square (21 November 2026): houses by residents over
/// `dens`, no filled squares or parks, no whole blocks, and office buildings
/// turned as the planner turned their yards. For setting before beside after
/// in one program.
pub(crate) fn plain() -> bool {
    crate::replay::look_plain()
}

/// The way a building at `p` faces: its front to the nearest road within
/// half a square, square to it; `yaw` (the planner's) where none is near.
/// A yaw's front is (sin, cos), as a plot's is.
pub(crate) fn facing(p: Vec2, yaw: f32, roads: &[(Vec2, Vec2)]) -> f32 {
    if plain() {
        return yaw;
    }
    let near = roads.iter().map(|&(a, b)| (super::off_line(p, a, b), a, b)).filter(|n| n.0 < 0.5).min_by(|x, y| x.0.total_cmp(&y.0));
    let Some((_, a, b)) = near else { return yaw };
    let d = (b - a).normalize_or_zero();
    let mut n = Vec2::new(-d.y, d.x);
    let t = ((p - a).dot(b - a) / a.distance_squared(b).max(1e-9)).clamp(0.0, 1.0);
    if n.dot(a.lerp(b, t) - p) < 0.0 {
        n = -n;
    }
    n.x.atan2(n.y)
}

/// The tall buildings of the offices drawn this year, for `ISOSTATE_LOTS`:
/// where each stands, its yaw, its storeys and its square.
static TALL: std::sync::Mutex<Vec<(Vec2, f32, u8, u32)>> = std::sync::Mutex::new(Vec::new());

/// Note an office building for `ISOSTATE_LOTS` (nothing is kept without it).
pub(crate) fn note_tall(p: Vec2, yaw: f32, storeys: u8, cell: u32) {
    if std::env::var_os("ISOSTATE_LOTS").is_some() {
        TALL.lock().unwrap().push((p, yaw, storeys, cell));
    }
}

/// How many of `tall` (where each stands and its yaw) stand square to a
/// street: their front, or their side, within ten degrees of facing the
/// nearest of `streets` within 0.15 of a square. Of those with no street that
/// near, none counts.
pub(crate) fn square_to_streets(tall: &[(Vec2, f32)], streets: &[(Vec2, Vec2)]) -> usize {
    tall.iter()
        .filter(|&&(p, yaw)| {
            let near = streets.iter().map(|&(a, b)| (super::off_line(p, a, b), a, b)).filter(|n| n.0 < 0.15).min_by(|x, y| x.0.total_cmp(&y.0));
            near.is_some_and(|(_, a, b)| {
                let along = Vec2::new(yaw.sin(), yaw.cos()).dot((b - a).normalize_or_zero()).abs();
                along < 0.174 || along > 0.985
            })
        })
        .count()
}

/// `ISOSTATE_LOTS=NAME`: of the town's buildings of five storeys or more in
/// year `k` (the forms on its lots, `forms`, and its offices, noted as they
/// were drawn), how many stand square to a street (`square_to_streets`), the
/// lanes drawn and the roads together; and how many whole blocks stand, the
/// plots they took in, and the forms on them. Returns the offices and the
/// closes built over by themselves that were noted (`note_tall`).
pub(super) fn tell_square(r: &Replay, k: usize, forms_tall: &[(Vec2, f32, u8, u32)], lanes: &[(Vec<Vec2>, f32)], blocks: &[(Footprint, usize)], shapes: &[Shape]) -> Vec<(Vec2, f32, u8, u32)> {
    let Ok(name) = std::env::var("ISOSTATE_LOTS") else { return Vec::new() };
    let Some(t) = r.header.towns.iter().position(|t| t.name == name) else { return Vec::new() };
    let ours = |c: u32| r.catch.get(k * r.n + c as usize).copied() == Some(t as u8 + 1);
    let cen = crate::network::centres(r);
    let mut streets: Vec<(Vec2, Vec2)> = crate::network::road_segments(r, k).into_iter().map(|(a, b, _)| (Vec2::new(cen[a].0, cen[a].1), Vec2::new(cen[b].0, cen[b].1))).collect();
    for (line, _) in lanes {
        streets.extend(line.windows(2).map(|w| (w[0], w[1])));
    }
    let offices: Vec<(Vec2, f32, u8, u32)> = std::mem::take(&mut *TALL.lock().unwrap());
    let pick = |v: &[(Vec2, f32, u8, u32)]| -> Vec<(Vec2, f32)> { v.iter().filter(|b| b.2 >= 5 && ours(b.3)).map(|b| (b.0, b.1)).collect() };
    let (on_lots, of_offices) = (pick(forms_tall), pick(&offices));
    let mut kinds: std::collections::BTreeMap<&str, usize> = Default::default();
    for s in shapes {
        *kinds.entry(forms::spec(*s).short).or_default() += 1;
    }
    let kinds: Vec<String> = kinds.iter().map(|(k, n)| format!("{k} {n}")).collect();
    eprintln!(
        "lots {}: {name}: buildings of five storeys or more square to a street: on its lots {} of {}, offices and infill {} of {}, together {} of {}; whole blocks {} taking in {} plots ({})",
        r.y0 + k as i32,
        square_to_streets(&on_lots, &streets),
        on_lots.len(),
        square_to_streets(&of_offices, &streets),
        of_offices.len(),
        square_to_streets(&on_lots, &streets) + square_to_streets(&of_offices, &streets),
        on_lots.len() + of_offices.len(),
        blocks.len(),
        blocks.iter().map(|b| b.1).sum::<usize>(),
        kinds.join(", ")
    );
    // the squares of its parks nearest its market place, for pointing a camera at one
    let node = r.header.towns[t].node;
    let mut parks: Vec<(f32, usize)> = (0..r.n).filter(|&c| ours(c as u32) && r.fill_of(k, c).is_some_and(|f| f.1 > 0.0)).map(|c| (((c % r.w) as f32 - (node % r.w) as f32).hypot((c / r.w) as f32 - (node / r.w) as f32), c)).collect();
    parks.sort_by(|a, b| a.0.total_cmp(&b.0));
    let near: Vec<String> = parks.iter().take(6).map(|&(_, c)| format!("{},{}", c % r.w, c / r.w)).collect();
    eprintln!("lots {}: {name}: {} squares with a park; the nearest its market place: {}", r.y0 + k as i32, parks.len(), near.join("; "));
    offices
}

/// Draw an office works' building at its place on the map, its front to its
/// street (`facing`).
pub(crate) fn draw_office(g: &Ground, at: (f32, f32, f32), b: &Built, f: usize, lod: &Lod, lk: &Look, solid: &mut Builder, smoke: &mut Builder) {
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let seed = Seed::new(world::hash2(f as i32, 1, 44), world::hash2(f as i32, 2, 44));
    let spec = forms::spec(b.shape);
    let form = forms::Form { shape: b.shape, storeys: b.storeys, built: b.year, ground: spec.ground, detail: lod.at(at.0, at.1), seed };
    let mut m = Model { solid: std::mem::take(solid), smoke: std::mem::take(smoke), green: Default::default(), flat: Builder::default() };
    forms::form(&mut m, &Site::on(ground(g, at.0, at.1), at.2, &height), &form, lk);
    let Model { solid: mut s, smoke: sm, flat, .. } = m;
    s.append(flat);
    *solid = s;
    *smoke = sm;
}

/// How far from the middle of a close of a filled square the nearest house,
/// form or street may stand for the close to be built over by itself
/// (`infill`): about half a close across.
const INFILL_CLEAR: f32 = 0.055;

/// A filled square's ground that no street's plots reach: the closes of its
/// order (all but its park) with no house, form or street within
/// `INFILL_CLEAR` of their middle, clear of the works' yards, the
/// churchyards, the market places and the railway (`clear`) and dry. The
/// town built over the whole of the square when it took it, so each such
/// close is built over by itself, in the form of the year its square was
/// first filled, its town's knowledge that year and the storeys its land
/// value then asked, on lots thrown together, its front to the nearest road.
/// It is worked out afresh each year from the record, as the lots are: the
/// same record draws the same buildings, and a square filled once is not
/// built over again. Returns the ground of each that stands.
#[allow(clippy::too_many_arguments)]
pub(super) fn infill(r: &Replay, g: &Ground, k: usize, town: &world::TownGround, standing: &[Stood], blocks: &[(Footprint, usize)], lanes: &[(Vec<Vec2>, f32)], clear: &dyn Fn(f32, f32) -> bool, lk: &Look, into: &mut Into) -> Vec<Footprint> {
    let mut out = Vec::new();
    if plain() || !r.builds.kept() || r.y0 + k as i32 <= FROM {
        return out;
    }
    // what stands, in buckets a tenth of a square across: the plots, and the streets as points
    let bucket = |p: Vec2| ((p.x * 10.0).floor() as i32, (p.y * 10.0).floor() as i32);
    let mut near: HashMap<(i32, i32), Vec<Vec2>> = HashMap::default();
    for s in standing {
        near.entry(bucket(s.lot.pl.p)).or_default().push(s.lot.pl.p);
    }
    for (line, _) in lanes {
        for w in line.windows(2) {
            let steps = ((w[0].distance(w[1]) / 0.02).ceil() as usize).max(1);
            for i in 0..=steps {
                let p = w[0].lerp(w[1], i as f32 / steps as f32);
                near.entry(bucket(p)).or_default().push(p);
            }
        }
    }
    let taken = |near: &HashMap<(i32, i32), Vec<Vec2>>, p: Vec2| {
        let (bx, bz) = bucket(p);
        (-1..=1).any(|dz| (-1..=1).any(|dx| near.get(&(bx + dx, bz + dz)).is_some_and(|v| v.iter().any(|q| q.distance(p) < INFILL_CLEAR))))
    };
    let cen = crate::network::centres(r);
    let roads: Vec<(Vec2, Vec2)> = crate::network::road_segments(r, k).into_iter().map(|(a, b, _)| (Vec2::new(cen[a].0, cen[a].1), Vec2::new(cen[b].0, cen[b].1))).collect();
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let mut cells: Vec<(&u32, &world::TownCell)> = town.cells.iter().filter(|(_, c)| c.filled).collect();
    cells.sort_by_key(|c| *c.0);
    for (&c, cell) in cells {
        // the year it was first filled, and what was built there then
        let mut y = k;
        while y > 0 && r.filled(y - 1, c as usize) && r.y0 + y as i32 - 1 > FROM {
            y -= 1;
        }
        let sq = square_year(r, y, c as usize);
        let middle = in_middle(r, y, c as usize);
        for (n, &(parcel, sub, area)) in cell.order.iter().enumerate().take(cell.built as usize) {
            if area <= 0.0 {
                continue;
            }
            let at = world::closes_of(g, parcel)[sub as usize].at;
            if taken(&near, at) || blocks.iter().any(|(f, _)| f.holds(at)) || roads.iter().any(|&(a, b)| super::off_line(at, a, b) < 0.045) {
                continue;
            }
            let yaw = facing(at, 0.0, &roads);
            let front = Vec2::new(yaw.sin(), yaw.cos());
            let side = Vec2::new(front.y, -front.x);
            let pick = world::hash2(c as i32, n as i32, 45);
            // (no close fronts a main street: tall only in its town's middle, rule 7)
            let a = forms::choose_within(&forms::Ask { knows: sq.knows, year: sq.year, storeys: asked(sq.value, sq.knows, middle, false), lot: FormLot::Joined, use_: Use::Dwellings, pick }, most(middle));
            let spec = forms::spec(a.shape);
            let ground = (spec.ground.0.min(0.09), spec.ground.1.min(0.07));
            // all of it on dry ground clear of what keeps a house out
            let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (0.0, 0.0)].map(|(u, v)| at + side * (u * ground.0 * 0.5) + front * (v * ground.1 * 0.5));
            if corners.iter().any(|q| clear(q.x, q.y) || world::height_at(g, q.x, q.y) < 0.05 || world::water_dist(g, q.x, q.y) < 0.03) {
                continue;
            }
            let seed = Seed::new(world::hash2(c as i32, n as i32, 46), world::hash2(c as i32, n as i32, 47));
            let f = forms::Form { shape: a.shape, storeys: a.storeys, built: sq.year, ground, detail: into.lod.at(at.x, at.y), seed };
            let leaves = leaf_group(seed.pick(43));
            let mut m = Model { solid: std::mem::take(into.houses), smoke: std::mem::take(into.smoke), green: std::mem::take(&mut into.trees[leaves]), flat: Builder::default() };
            forms::form(&mut m, &Site::on(super::ground(g, at.x, at.y), yaw, &height), &f, lk);
            let Model { mut solid, smoke, green, flat } = m;
            solid.append(flat);
            *into.houses = solid;
            *into.smoke = smoke;
            into.trees[leaves] = green;
            // (and it is taken: the next close keeps its distance from it)
            near.entry(bucket(at)).or_default().push(at);
            note_tall(at, yaw, a.storeys, c);
            out.push(Footprint { at, along: side, front, half: (ground.0 * 0.5, ground.1 * 0.5) });
        }
    }
    out
}

/// Draw a group of plots built as a form: as many of the form as its
/// frontage holds, side by side, each with its front on the street's line.
/// Returns the ground each stands on.
pub(super) fn draw(g: &Ground, standing: &[Stood], first: usize, plots: usize, b: &Built, lk: &Look, into: &mut Into) -> Vec<Footprint> {
    let mut out = Vec::new();
    let (a, z) = (&standing[first].lot.pl, &standing[first + plots - 1].lot.pl);
    let spec = forms::spec(b.shape);
    let dir = Vec2::new(a.yaw.sin(), a.yaw.cos());
    let (mid, run) = ((a.p + z.p) * 0.5, z.p - a.p);
    let along = if run.length() > 1e-4 { run.normalize() } else { Vec2::new(dir.y, -dir.x) };
    let width = run.length() + a.width;
    // a whole block is one building over the whole of its frontage, back to the lane behind;
    // an estate's whole block over the whole of its block between four streets
    let block = spec.lot == FormLot::Block;
    let count = if block { 1 } else { ((width / spec.ground.0).round() as usize).max(1) };
    let depth = if standing[first].lot.estate.is_some_and(|e| e.block) { a.depth } else if block { BLOCK_DEPTH } else { spec.ground.1 };
    // its front on the plots' front line
    let back = dir * ((depth - a.depth) * 0.5).max(0.0);
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let id = a.run.0.wrapping_mul(4) + a.run.1 as i32;
    for i in 0..count {
        let at = mid + along * (((i as f32 + 0.5) / count as f32 - 0.5) * width) - back;
        let seed = Seed::new(world::hash2(id, a.index + i as i32, 41), world::hash2(id, a.index + i as i32, 42));
        let f = forms::Form { shape: b.shape, storeys: b.storeys, built: b.year, ground: (width / count as f32 * 0.96, depth), detail: into.lod.at(at.x, at.y), seed };
        out.push(Footprint { at, along, front: dir, half: (width / count as f32 * 0.48, depth * 0.5) });
        let leaves = leaf_group(seed.pick(43));
        let mut m = Model { solid: std::mem::take(into.houses), smoke: std::mem::take(into.smoke), green: std::mem::take(&mut into.trees[leaves]), flat: Builder::default() };
        forms::form(&mut m, &Site::on(ground(g, at.x, at.y), a.yaw, &height), &f, lk);
        let Model { mut solid, smoke, green, flat } = m;
        // (what lies flat, a car park's or a yard's ground, goes in with the houses, as the lanes do)
        solid.append(flat);
        *into.houses = solid;
        *into.smoke = smoke;
        into.trees[leaves] = green;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn years(from: i32, to: i32, dens: impl Fn(i32) -> f32, value: impl Fn(i32) -> f32) -> Vec<Year> {
        (from..=to).map(|y| Year { year: y, dens: dens(y), value: value(y), knows: Knows::by_year(y) }).collect()
    }

    /// A square of forty plots built by 1900: ten pairs on its road, and four
    /// rows of five behind.
    fn old_square() -> Vec<Group> {
        let mut g: Vec<Group> = (0..10).map(|i| Group { plots: 2, built: 1820 + i, front: true, pick: i as f32 / 10.0, joined: true, block: false, wall: i as u32, tall: true }).collect();
        g.extend((0..4).map(|i| Group { plots: 5, built: 1850 + i, front: false, pick: 0.05 + i as f32 / 4.0, joined: true, block: false, wall: 10 + i as u32, tall: true }));
        g
    }

    /// A square whose ground never grew dear keeps the houses it had in 1900,
    /// in every year to 2050, however its land value moves below the dear.
    #[test]
    fn a_square_that_never_grew_dear_keeps_its_houses() {
        let g = old_square();
        for to in [1901, 1950, 2050] {
            let built = walk(&g, &years(1901, to, |_| 1.0, |y| 0.6 + 0.4 * ((y as f32) * 0.3).sin()));
            assert!(built.iter().all(|b| b.is_none()), "{to}: {built:?}");
        }
    }

    /// A square that grows dear is rebuilt only as far as it must hold more,
    /// the road's frontage first, in the forms of the years it grew in, and
    /// what was built is the same whichever later year the walk stops at.
    #[test]
    fn a_dear_square_is_rebuilt_just_enough_and_stays_rebuilt() {
        let g = old_square();
        let dens = |y: i32| (1.0 + (y - 1910).max(0) as f32 * 0.02).min(2.2);
        // (its land value about the square of its dens, as the core's dens is the square root
        // of the value it moves toward: as in island 7's first city, whose middle stood at dens
        // 2.02 and value 4.0 in 1950)
        let value = |y: i32| dens(y) * dens(y) * 1.05;
        let all = walk(&g, &years(1901, 2050, dens, value));
        let plots: usize = g.iter().map(|g| g.plots).sum();
        let held: f32 = g.iter().zip(&all).map(|(g, b)| b.map_or(1.0, |b| holds(b.storeys, g.joined)) * g.plots as f32).sum::<f32>() / plots as f32;
        // it holds what the square holds, or what is left standing could not be pulled down
        // for anything holding twice as much on the dearest ground it comes to
        let dearest = holds(storeys_for(value(2050), Knows::by_year(2050)), true);
        let least = g.iter().zip(&all).map(|(g, b)| b.map_or(1.0, |b| holds(b.storeys, g.joined))).fold(f32::MAX, f32::min);
        assert!(held >= 2.2 - 0.005 || least * TWICE > dearest, "it holds {held}, the least {least}");
        assert!(held > 2.0, "it holds {held}");
        // just enough: undo the last rebuilding and it would not hold it
        let last = all.iter().flatten().map(|b| b.year).max().unwrap();
        let before: f32 = g.iter().zip(&all).map(|(g, b)| b.filter(|b| b.year < last).map_or(1.0, |b| holds(b.storeys, g.joined)) * g.plots as f32).sum::<f32>() / plots as f32;
        assert!(before < dens(last) + 0.005, "{before} held before {last}");
        // some of the old stands, the frontage went first, and the forms are of several ages
        assert!(all.iter().any(|b| b.is_none()));
        let front_first = g.iter().zip(&all).filter(|(g, b)| !g.front && b.is_some()).count() == 0 || g.iter().zip(&all).all(|(g, b)| !g.front || b.is_some());
        assert!(front_first, "{all:?}");
        let mut ages: Vec<i32> = all.iter().flatten().map(|b| b.year / 10).collect();
        ages.sort();
        ages.dedup();
        assert!(ages.len() >= 3, "{all:?}");
        // the same record draws the same square: stopped in 1960, the walk gives what the
        // whole walk had built by 1960
        let to_1960 = walk(&g, &years(1901, 1960, dens, value));
        for (a, b) in to_1960.iter().zip(&all) {
            match (a, b) {
                (Some(a), Some(b)) if b.year <= 1960 => assert_eq!(a, b),
                (Some(a), Some(b)) => assert!(a.year <= 1960 && b.year > 1960),
                (None, Some(b)) => assert!(b.year > 1960),
                (Some(_), None) => panic!("rebuilt by 1960 and not after"),
                (None, None) => {}
            }
        }
    }

    /// Lots thrown together as the land passes its marks: on ground worth
    /// less than `BLOCK_VALUE` a way's frontage is rebuilt a few plots at a
    /// time; on dearer ground as a whole block, which takes in the plots on
    /// its ground behind it, and those plots stand under it from then on.
    #[test]
    fn a_dear_frontage_is_rebuilt_as_a_whole_block() {
        // a way's frontage of four plots, and two rows of the back lane on the block's ground
        let g = [
            Group { plots: 4, built: 1850, front: true, pick: 0.2, joined: true, block: false, wall: 0, tall: true },
            Group { plots: 4, built: 1860, front: false, pick: 0.4, joined: true, block: false, wall: 1, tall: true },
            Group { plots: 3, built: 1990, front: false, pick: 0.6, joined: true, block: false, wall: 2, tall: true },
        ];
        let partners = vec![vec![1, 2], Vec::new(), Vec::new()];
        let dens = |y: i32| if y < 1950 { 1.0 } else { 3.0 };
        // below the mark: the frontage alone, on lots thrown together
        let cheap = walk_with(&g, &partners, &years(1901, 1960, dens, |_| BLOCK_VALUE * 0.8));
        assert!(cheap.by.iter().all(|b| b.is_none()), "{:?}", cheap.by);
        assert_eq!(cheap.form[0].map(|b| forms::spec(b.shape).lot), Some(FormLot::Joined));
        // past it: a whole block, over the back lane's rows, and the row built later never goes up
        let dear = walk_with(&g, &partners, &years(1901, 2000, dens, |_| BLOCK_VALUE * 1.5));
        let b = dear.form[0].unwrap();
        assert_eq!(forms::spec(b.shape).lot, FormLot::Block, "{b:?}");
        assert_eq!(b.year, 1950);
        assert_eq!(dear.by, vec![None, Some(0), Some(0)]);
        assert!(dear.form[1].is_none() && dear.form[2].is_none());
        // and the same record draws the same block, whichever year the walk stops at
        let to_1960 = walk_with(&g, &partners, &years(1901, 1960, dens, |_| BLOCK_VALUE * 1.5));
        assert_eq!((to_1960.form[0], &to_1960.by[..2]), (Some(b), &dear.by[..2]));
    }

    /// Height follows the street (rule 7): on ground dear enough for towers,
    /// the main street's frontage rises tall and the back streets no higher
    /// than a mansion block; and a front goes up together, every group of it
    /// to the same form and height, so that no tall building stands alone
    /// among the houses of its own street.
    #[test]
    fn towers_keep_to_the_main_street_and_a_front_rises_together() {
        // a main street's frontage of three fronts of three pairs, and a back street of three
        // fronts of three rows, all houses of the 1890s
        let mut g: Vec<Group> = Vec::new();
        for f in 0..3u32 {
            for i in 0..3 {
                g.push(Group { plots: 2, built: 1890, front: true, pick: (f * 3 + i) as f32 / 9.0, joined: true, block: false, wall: f, tall: true });
                g.push(Group { plots: 4, built: 1891, front: false, pick: (f * 3 + i) as f32 / 9.0, joined: true, block: false, wall: 10 + f, tall: false });
            }
        }
        // ground worth five times its town's of 1900 from 1990, and a square that must hold
        // three times its room of 1900
        let built = walk(&g, &years(1901, 2050, |y| if y < 1990 { 1.0 } else { 3.0 }, |y| if y < 1990 { 1.0 } else { 5.0 }));
        assert!(built.iter().any(|b| b.is_some()), "{built:?}");
        for (gr, b) in g.iter().zip(&built) {
            if let (false, Some(b)) = (gr.tall, b) {
                assert!(b.storeys <= MIDDLE_STOREYS, "a back street rose to {b:?}");
            }
        }
        // the main street's fronts rose tall, as its ground is dearer than the square's mean
        let tallest = g.iter().zip(&built).filter(|(gr, _)| gr.front).filter_map(|(_, b)| *b).map(|b| b.storeys).max().unwrap_or(0);
        assert!(tallest > MIDDLE_STOREYS, "the main street rose only to {tallest}");
        // every front that was rebuilt was rebuilt whole, to one form and height
        for w in [0, 1, 2, 10, 11, 12] {
            let mine: Vec<Option<Built>> = g.iter().zip(&built).filter(|(gr, _)| gr.wall == w).map(|(_, b)| *b).collect();
            if mine.iter().any(|b| b.is_some()) {
                assert!(mine.iter().all(|b| *b == mine[0]), "front {w}: {mine:?}");
            }
        }
    }

    /// Land value sets the storeys and technology caps them: the same dear
    /// ground builds no higher than five storeys before the town knows a
    /// frame, and higher after.
    #[test]
    fn technology_caps_the_storeys() {
        assert_eq!(storeys_for(1.0, Knows::by_year(1950)), 2);
        assert_eq!(storeys_for(4.0, Knows::by_year(1950)), 6);
        assert_eq!(storeys_for(8.0, Knows::by_year(1950)), 10);
        assert_eq!(storeys_for(8.0, Knows::by_year(1880)), 5);
        assert_eq!(storeys_for(0.2, Knows::by_year(1950)), 1);
        assert_eq!(storeys_for(8.0, Knows::by_year(1950).without(Know::SteelFrame).without(Know::Concrete)), 5);
    }

    /// Lots first built after 1900 are built in the form of their year: a
    /// pair on cheap ground in 1930 is a pair of semis, a lot alone a
    /// detached house, and in 1965 on dear ground flats.
    #[test]
    fn a_new_lot_is_built_in_the_form_of_its_year() {
        let g = [Group { plots: 2, built: 1930, front: false, pick: 0.3, joined: true, block: false, wall: 0, tall: true }, Group { plots: 1, built: 1930, front: false, pick: 0.3, joined: false, block: false, wall: 1, tall: true }, Group { plots: 2, built: 1965, front: true, pick: 0.3, joined: true, block: false, wall: 2, tall: true }];
        let built = walk(&g, &years(1901, 1970, |_| 1.0, |y| if y < 1960 { 1.0 } else { 4.0 }));
        assert_eq!(built[0].map(|b| b.shape), Some(Shape::SemisInterwar));
        assert_eq!(built[1].map(|b| b.shape), Some(Shape::DetachedInterwar));
        let flats = built[2].unwrap();
        assert!(flats.storeys >= 5 && forms::spec(flats.shape).use_ == Use::Dwellings && flats.year == 1965, "{flats:?}");
    }
}
