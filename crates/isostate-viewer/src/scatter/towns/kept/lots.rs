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
pub(crate) fn walk(groups: &[Group], years: &[Year]) -> Vec<Option<Built>> {
    let n = groups.len();
    let mut form: Vec<Option<Built>> = vec![None; n];
    let mut cap = vec![1.0f32; n];
    let mut since: Vec<i32> = groups.iter().map(|g| g.built).collect();
    let ask = |g: &Group, y: &Year| {
        let lot = if g.joined { FormLot::Joined } else { FormLot::House };
        forms::choose(&forms::Ask { knows: y.knows, year: y.year, storeys: storeys_for(y.value, y.knows), lot, use_: Use::Dwellings, pick: g.pick })
    };
    for y in years {
        // the groups first built this year are built as the form of the year (and any
        // built after 1900 in a year the walk was not given, in the first it is)
        for (i, g) in groups.iter().enumerate() {
            if g.built > FROM && g.built <= y.year && form[i].is_none() {
                let a = ask(g, y);
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
            let Some(&i) = standing.iter().min_by(|&&a, &&b| {
                cap[a].total_cmp(&cap[b]).then(groups[b].front.cmp(&groups[a].front)).then(since[a].cmp(&since[b])).then(groups[a].pick.total_cmp(&groups[b].pick))
            }) else {
                break;
            };
            let a = ask(&groups[i], y);
            let more = holds(a.storeys, groups[i].joined);
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
        }
    }
    form
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
/// its squares are built, nor in a year to 1900.
pub(super) fn drawn(r: &Replay, k: usize, standing: &[Stood]) -> Vec<Drawn> {
    let mut out = vec![Drawn::House; standing.len()];
    let now = r.y0 + k as i32;
    if !r.builds.kept() || now <= FROM {
        return out;
    }
    let units = units(standing);
    let mut by_cell: HashMap<u32, Vec<usize>> = HashMap::default();
    for (u, range) in units.iter().enumerate() {
        by_cell.entry(standing[range.start].lot.cell).or_default().push(u);
    }
    let first = (FROM + 1 - r.y0).max(0) as usize;
    for (&c, list) in &by_cell {
        let groups: Vec<Group> = list
            .iter()
            .map(|&u| {
                let range = &units[u];
                let s = &standing[range.start];
                Group { plots: range.len(), built: range.clone().map(|i| standing[i].year).min().unwrap_or(s.year), front: s.lot.road || s.lot.kind == 9, pick: s.lot.pl.u, joined: range.len() > 1 || s.lot.pl.row.is_none() }
            })
            .collect();
        let years: Vec<Year> = (first..=k).map(|y| square_year(r, y, c as usize)).collect();
        for (&u, built) in list.iter().zip(walk(&groups, &years)) {
            if let Some(b) = built {
                let range = &units[u];
                out[range.start] = Drawn::Form(b, range.len());
                for i in range.start + 1..range.end {
                    out[i] = Drawn::Covered;
                }
            }
        }
    }
    out
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
        if built.is_some_and(|b| (asked as f32) <= b.storeys as f32 * 1.5) {
            continue;
        }
        let a = forms::choose(&forms::Ask { knows: sq.knows, year: sq.year, storeys: asked, lot: FormLot::Joined, use_: Use::Offices, pick });
        if built.is_none_or(|b| a.storeys > b.storeys) {
            built = Some(Built { shape: a.shape, storeys: a.storeys, year: sq.year });
        }
    }
    built
}

/// Draw an office works' building at its place on the map, its front to its
/// yard's gate as the planner turned it.
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

/// Draw a group of plots built as a form: as many of the form as its
/// frontage holds, side by side, each with its front on the street's line.
pub(super) fn draw(g: &Ground, standing: &[Stood], first: usize, plots: usize, b: &Built, lk: &Look, into: &mut Into) {
    let (a, z) = (&standing[first].lot.pl, &standing[first + plots - 1].lot.pl);
    let spec = forms::spec(b.shape);
    let dir = Vec2::new(a.yaw.sin(), a.yaw.cos());
    let (mid, run) = ((a.p + z.p) * 0.5, z.p - a.p);
    let along = if run.length() > 1e-4 { run.normalize() } else { Vec2::new(dir.y, -dir.x) };
    let width = run.length() + a.width;
    let count = ((width / spec.ground.0).round() as usize).max(1);
    let depth = spec.ground.1;
    // its front on the plots' front line
    let back = dir * ((depth - a.depth) * 0.5).max(0.0);
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let id = a.run.0 * 4 + a.run.1 as i32;
    for i in 0..count {
        let at = mid + along * (((i as f32 + 0.5) / count as f32 - 0.5) * width) - back;
        let seed = Seed::new(world::hash2(id, a.index + i as i32, 41), world::hash2(id, a.index + i as i32, 42));
        let f = forms::Form { shape: b.shape, storeys: b.storeys, built: b.year, ground: (width / count as f32 * 0.96, depth), detail: into.lod.at(at.x, at.y), seed };
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
        let mut g: Vec<Group> = (0..10).map(|i| Group { plots: 2, built: 1820 + i, front: true, pick: i as f32 / 10.0, joined: true }).collect();
        g.extend((0..4).map(|i| Group { plots: 5, built: 1850 + i, front: false, pick: 0.05 + i as f32 / 4.0, joined: true }));
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
        let g = [Group { plots: 2, built: 1930, front: false, pick: 0.3, joined: true }, Group { plots: 1, built: 1930, front: false, pick: 0.3, joined: false }, Group { plots: 2, built: 1965, front: true, pick: 0.3, joined: true }];
        let built = walk(&g, &years(1901, 1970, |_| 1.0, |y| if y < 1960 { 1.0 } else { 4.0 }));
        assert_eq!(built[0].map(|b| b.shape), Some(Shape::SemisInterwar));
        assert_eq!(built[1].map(|b| b.shape), Some(Shape::DetachedInterwar));
        let flats = built[2].unwrap();
        assert!(flats.storeys >= 5 && forms::spec(flats.shape).use_ == Use::Dwellings && flats.year == 1965, "{flats:?}");
    }
}
