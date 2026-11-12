//! The second island generator's land (the core's docs/rules/land.md) as the
//! viewer holds it: a plain copy of what the core knows beyond the cells, kept
//! beside the run in an `Arc` of its own (`Replay::land`), so that a year
//! arriving in a live game never copies it.
//!
//! A recorded run made on the new land carries it as arrays and a `land`
//! object in its header (`landcells::static_arrays`, `static_header`), and
//! replay.rs reads them, all or nothing. A live game has the core's own
//! `Landform` in its static part, and live.rs copies it here. A run made on
//! the old island has none, and is drawn as it always was.
//!
//! Words: a **node** is a point of the core's fine grid, 500 m apart at four
//! to a cell's side; node (i, j) has its centre at ((i + 0.5) / fine,
//! (j + 0.5) / fine) in cells. Heights and levels are in metres above the sea.
//!
//! What is drawn from it so far (docs/probes/dx-land-view.md, stages 0 and
//! 1): the ground, its rivers and its lakes (`ground`), and each cell's
//! meeting place on dry and level ground (`middles`).

mod ground;
mod middles;
#[cfg(test)]
mod on_islands;

pub use ground::{ground, metres, Laid, RELIEF};

use bevy::math::Vec2;

/// What a river line runs into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Sea,
    /// a lake, by its place in `NewLand::lakes`
    Lake(usize),
    /// a bigger river, by its place in `NewLand::lines`
    River(usize),
}

impl End {
    /// From the record's number: -1 the sea, -2 - L lake L, else the river joined.
    pub fn from_code(code: i32) -> End {
        match code {
            -1 => End::Sea,
            c if c <= -2 => End::Lake((-2 - c) as usize),
            c => End::River(c as usize),
        }
    }
}

/// One of the core's river lines: a polyline from where the stream first
/// carries 0.3 cubic metres a second to the sea, a lake or the river it
/// joins, with its meanders in its points (some 40 m apart).
#[derive(Clone, Debug)]
pub struct Line {
    /// in cells, upstream first
    pub pts: Vec<Vec2>,
    /// the water's width at each point, metres
    pub width: Vec<f32>,
    /// the water's level at each point, metres above the sea
    pub level: Vec<f32>,
    /// mean flow at each point, cubic metres a second
    pub flow: Vec<f32>,
    pub end: End,
}

/// A loop a river has cut off and left as still water.
#[derive(Clone, Debug)]
pub struct Oxbow {
    pub pts: Vec<Vec2>,
    /// metres
    pub width: f32,
    /// metres above the sea
    pub level: f32,
}

/// A lake. In the header of a recorded run as the core writes it.
#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Lake {
    /// the height of its water above the sea, metres
    pub level: f32,
    #[serde(default)]
    pub area_km2: f32,
    /// its greatest depth, metres
    #[serde(default)]
    pub depth: f32,
    /// its middle, in cells
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    /// where the water leaves it, in cells
    #[serde(default)]
    pub outlet: (f32, f32),
    /// mean flow out of it, cubic metres a second
    #[serde(default)]
    pub outflow: f32,
}

/// The `land` object of a recorded run's header, as far as the viewer reads it.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct LandHeader {
    /// nodes along a cell's side
    pub fine: usize,
    /// the fine grid's size in nodes
    pub fw: usize,
    pub fh: usize,
    /// how far the sea rose at the end, metres
    #[serde(default)]
    pub drowned: f32,
    #[serde(default)]
    pub lakes: Vec<Lake>,
}

/// The land of a run made by the second island generator.
#[derive(Clone, Debug)]
pub struct NewLand {
    /// the map's size in cells
    pub w: usize,
    pub h: usize,
    /// nodes along a cell's side, and the fine grid's size in nodes
    pub fine: usize,
    pub fw: usize,
    pub fh: usize,
    /// per node, row-major, `fh` rows of `fw`: metres above the sea (below 0
    /// under it; a lake's bed under a lake)
    pub height: Vec<f32>,
    /// per node: the rock at the surface (the core's `landform::ROCK_*`)
    pub rock: Vec<u8>,
    /// per node: 0, or the lake it lies under plus 1
    pub lake: Vec<u8>,
    /// per node: 0 dry, 1 flood plain, 2 marsh
    pub wet: Vec<u8>,
    /// per node: mean flow through it, cubic metres a second
    pub flow: Vec<f32>,
    pub lines: Vec<Line>,
    pub oxbows: Vec<Oxbow>,
    pub lakes: Vec<Lake>,
    /// how far the sea rose at the end, metres
    pub drowned: f32,
    /// per cell: the cell is a lake to the record (`land_lakecell`), it has a
    /// lake's shore in it, a river's tideway runs through it
    pub lake_cell: Vec<bool>,
    pub shore: Vec<bool>,
    pub estuary: Vec<bool>,
    /// where each cell's roads meet (`NewLand::middles`), worked out the first time it is asked for
    pub middles: std::sync::OnceLock<Vec<(f32, f32)>>,
}

/// The core's code for marsh in `NewLand::wet` (`landwater::WET_MARSH`).
pub const WET_MARSH: u8 = 2;

/// The arrays of a recorded run, by river or by point, that make its lines.
pub struct LineArrays {
    /// river r's points are `start[r] .. start[r + 1]`
    pub start: Vec<i32>,
    /// x and y of each point, in cells
    pub xy: Vec<f32>,
    pub width: Vec<f32>,
    pub level: Vec<f32>,
    pub flow: Vec<f32>,
    /// by river (`End::from_code`)
    pub end: Vec<i32>,
}

/// Whether the starts of a set of lines agree with their points: they begin
/// at nought, never go back, and end at the number of points there are.
/// `what` names the array for the message.
fn starts_agree(what: &str, start: &[i32], points: usize) -> Result<(), String> {
    if start.first() != Some(&0) {
        return Err(format!("its array {what} should begin at 0 and begins at {}", start.first().map_or_else(|| "nothing".to_string(), |s| s.to_string())));
    }
    if let Some(w) = start.windows(2).find(|w| w[1] < w[0]) {
        return Err(format!("its array {what} goes back from {} to {}, and a line's points come after the line before it", w[0], w[1]));
    }
    let last = *start.last().unwrap() as usize;
    if last != points {
        return Err(format!("its array {what} ends at point {last}, and there are {points} points"));
    }
    Ok(())
}

/// The river lines from a recorded run's arrays, or what does not agree
/// among them: each array's length is held to the others'.
pub fn lines_from(a: &LineArrays) -> Result<Vec<Line>, String> {
    let points = a.width.len();
    for (name, len, want) in [("river_xy", a.xy.len(), 2 * points), ("river_level", a.level.len(), points), ("river_flow", a.flow.len(), points)] {
        if len != want {
            return Err(format!("its array {name} has {len} values, where the {points} points of river_width need {want}"));
        }
    }
    starts_agree("river_start", &a.start, points)?;
    if a.end.len() + 1 != a.start.len() {
        return Err(format!("its array river_end has {} rivers, where river_start has {}", a.end.len(), a.start.len() - 1));
    }
    let rivers = a.end.len();
    if let Some(&e) = a.end.iter().find(|&&e| e >= rivers as i32) {
        return Err(format!("its array river_end says a river joins river {e}, and there are {rivers}"));
    }
    Ok((0..rivers)
        .map(|r| {
            let (i0, i1) = (a.start[r] as usize, a.start[r + 1] as usize);
            Line {
                pts: (i0..i1).map(|i| Vec2::new(a.xy[2 * i], a.xy[2 * i + 1])).collect(),
                width: a.width[i0..i1].to_vec(),
                level: a.level[i0..i1].to_vec(),
                flow: a.flow[i0..i1].to_vec(),
                end: End::from_code(a.end[r]),
            }
        })
        .collect())
}

/// The oxbows from a recorded run's arrays, held to each other in the same way.
pub fn oxbows_from(start: &[i32], xy: &[f32], width: &[f32], level: &[f32]) -> Result<Vec<Oxbow>, String> {
    if xy.len() % 2 != 0 {
        return Err(format!("its array oxbow_xy has {} values, and a point has two", xy.len()));
    }
    starts_agree("oxbow_start", start, xy.len() / 2)?;
    for (name, len) in [("oxbow_width", width.len()), ("oxbow_level", level.len())] {
        if len + 1 != start.len() {
            return Err(format!("its array {name} has {len} values, where oxbow_start has {} oxbows", start.len() - 1));
        }
    }
    Ok((0..width.len())
        .map(|o| Oxbow { pts: (start[o] as usize..start[o + 1] as usize).map(|i| Vec2::new(xy[2 * i], xy[2 * i + 1])).collect(), width: width[o], level: level[o] })
        .collect())
}

impl NewLand {
    /// The land of a live game, from the core's own (`LiveStatic::land`).
    pub fn from_core(lf: &isostate_core::landcells::Landform, w: usize, h: usize) -> NewLand {
        use isostate_core::landwater::RiverEnd;
        NewLand {
            w,
            h,
            fine: lf.fine,
            fw: lf.fw,
            fh: lf.fh,
            height: lf.height.clone(),
            rock: lf.rock.clone(),
            // (a byte, as the record keeps it)
            lake: lf.lake_id.iter().map(|&l| l.min(255) as u8).collect(),
            wet: lf.wet.clone(),
            flow: lf.flow.clone(),
            lines: lf
                .rivers
                .iter()
                .map(|r| Line {
                    pts: r.x.iter().zip(&r.y).map(|(&x, &y)| Vec2::new(x, y)).collect(),
                    width: r.width.clone(),
                    level: r.level.clone(),
                    flow: r.flow.clone(),
                    end: match r.end {
                        RiverEnd::Sea => End::Sea,
                        RiverEnd::Lake(l) => End::Lake(l),
                        RiverEnd::River(m) => End::River(m),
                    },
                })
                .collect(),
            oxbows: lf.oxbows.iter().map(|o| Oxbow { pts: o.x.iter().zip(&o.y).map(|(&x, &y)| Vec2::new(x, y)).collect(), width: o.width, level: o.level }).collect(),
            lakes: lf.lakes.iter().map(|l| Lake { level: l.level, area_km2: l.area_km2, depth: l.depth, x: l.x, y: l.y, outlet: l.outlet, outflow: l.outflow }).collect(),
            drowned: lf.drowned as f32,
            lake_cell: lf.lake.clone(),
            shore: lf.shore.clone(),
            estuary: lf.estuary.clone(),
            middles: Default::default(),
        }
    }

    /// Where each cell's roads meet: for a land cell the nearest dry and
    /// level spot of the cell to where the old island would put it, off
    /// tidewater, lakes and marsh and back from a river (middles.rs).
    /// `terrain` is the record's class of each cell, 0 for water. It is
    /// worked out once, and is the same however often it is asked for.
    pub fn middles(&self, terrain: &[u8]) -> &[(f32, f32)] {
        self.middles.get_or_init(|| middles::place(self, terrain))
    }

    /// The node a spot in cells lies in.
    pub fn node_at(&self, x: f32, y: f32) -> usize {
        let i = ((x * self.fine as f32).max(0.0) as usize).min(self.fw - 1);
        let j = ((y * self.fine as f32).max(0.0) as usize).min(self.fh - 1);
        j * self.fw + i
    }

    /// The middle of node `n`, in cells.
    pub fn node_xy(&self, n: usize) -> Vec2 {
        Vec2::new(((n % self.fw) as f32 + 0.5) / self.fine as f32, ((n / self.fw) as f32 + 0.5) / self.fine as f32)
    }

    /// What a spot stands on, in a word or two, with its height: for a report.
    pub fn stands_on(&self, x: f32, y: f32) -> String {
        let n = self.node_at(x, y);
        let what = if self.lake[n] > 0 {
            "a lake"
        } else if self.height[n] <= 0.0 {
            "tidewater"
        } else if self.wet[n] == WET_MARSH {
            "marsh"
        } else if self.wet[n] > 0 {
            "flood plain"
        } else {
            "dry ground"
        };
        format!("{what} at {:.0} m", self.height[n])
    }
}

/// `ISOSTATE_LAND_REPORT=1`: say what the run's land holds, what the ground
/// drawn from it came to, and where its lakes, tablelands, heights and towns
/// are (for pointing a camera at them), and stop. True if it was asked for.
pub fn maybe_report(r: &crate::replay::Replay, g: &crate::world::Ground, cen: &[(f32, f32)]) -> bool {
    if std::env::var_os("ISOSTATE_LAND_REPORT").is_none_or(|v| v.is_empty()) {
        return false;
    }
    let (Some(l), Some(laid)) = (r.land.as_deref(), g.laid.as_ref()) else {
        println!("land report: this run was made on the old island and has none of the new land's arrays");
        return true;
    };
    report_ground(g, laid);
    report_middles(r, l, cen);
    let nodes = l.fw * l.fh;
    let dry: Vec<usize> = (0..nodes).filter(|&n| l.height[n] > 0.0 && l.lake[n] == 0).collect();
    let share = |count: usize| 100.0 * count as f32 / dry.len().max(1) as f32;
    let top = (0..nodes).max_by(|&a, &b| l.height[a].total_cmp(&l.height[b])).unwrap_or(0);
    println!(
        "land report: {} by {} nodes at {} to a cell's side on a map of {} by {} cells; {} of them dry land, {} under a lake; the sea rose {:.1} m at the end",
        l.fw,
        l.fh,
        l.fine,
        l.w,
        l.h,
        dry.len(),
        l.lake.iter().filter(|&&k| k > 0).count(),
        l.drowned
    );
    println!(
        "land report: of the dry nodes {:.1}% are flood plain and {:.1}% marsh; the highest stands {:.0} m, at {:.1},{:.1}; the greatest flow through a node is {:.0} cubic m a second",
        share(dry.iter().filter(|&&n| l.wet[n] == 1).count()),
        share(dry.iter().filter(|&&n| l.wet[n] == WET_MARSH).count()),
        l.height[top],
        l.node_xy(top).x,
        l.node_xy(top).y,
        l.flow.iter().copied().fold(0.0, f32::max)
    );
    // the rock at the surface, and the middle of each kind's nodes: the cap rock is the tablelands
    let names = isostate_core::landform::ROCK_NAMES;
    for (k, name) in names.iter().enumerate() {
        let of: Vec<usize> = dry.iter().copied().filter(|&n| l.rock[n] as usize == k).collect();
        if of.is_empty() {
            continue;
        }
        let mid = of.iter().map(|&n| l.node_xy(n)).sum::<Vec2>() / of.len() as f32;
        println!("land report: rock: {name} at the surface of {:.1}% of the dry nodes, about {:.1},{:.1}", share(of.len()), mid.x, mid.y);
    }
    let km = |pts: &[Vec2]| pts.windows(2).map(|p| p[0].distance(p[1])).sum::<f32>() * 2.0;
    let points: usize = l.lines.iter().map(|ln| ln.pts.len()).sum();
    let ends = |e: fn(&End) -> bool| l.lines.iter().filter(|ln| e(&ln.end)).count();
    println!(
        "land report: {} river lines of {points} points, {:.0} km; {} run to the sea, {} into a lake, {} into another river; widths {:.0} to {:.0} m; {} oxbows of {} points, {:.0} to {:.0} m wide, the highest {:.0} m up",
        l.lines.len(),
        l.lines.iter().map(|ln| km(&ln.pts)).sum::<f32>(),
        ends(|e| *e == End::Sea),
        ends(|e| matches!(e, End::Lake(_))),
        ends(|e| matches!(e, End::River(_))),
        l.lines.iter().flat_map(|ln| ln.width.iter().copied()).fold(f32::MAX, f32::min),
        l.lines.iter().flat_map(|ln| ln.width.iter().copied()).fold(0.0, f32::max),
        l.oxbows.len(),
        l.oxbows.iter().map(|o| o.pts.len()).sum::<usize>(),
        l.oxbows.iter().map(|o| o.width).fold(f32::MAX, f32::min),
        l.oxbows.iter().map(|o| o.width).fold(0.0, f32::max),
        l.oxbows.iter().map(|o| o.level).fold(0.0, f32::max)
    );
    // the greatest river, where it is widest: its lowest reach before the tide or the lake
    if let Some(big) = l.lines.iter().max_by(|a, b| a.flow.last().unwrap_or(&0.0).total_cmp(b.flow.last().unwrap_or(&0.0))).filter(|ln| ln.pts.len() > 2) {
        let at = big.pts[big.pts.len() * 3 / 4];
        println!(
            "land report: the greatest river carries {:.0} cubic m a second where it ends, {:.0} m wide, its water {:.1} m up at its head; three quarters of the way down it is at {:.1},{:.1}",
            big.flow.last().unwrap(),
            big.width.last().unwrap(),
            big.level[0],
            at.x,
            at.y
        );
    }
    for (i, lake) in l.lakes.iter().enumerate() {
        println!(
            "land report: lake {i}: {:.1} square km at {:.1} m, {:.0} m deep, its middle at {:.1},{:.1}, its outlet at {:.1},{:.1} with {:.1} cubic m a second",
            lake.area_km2, lake.level, lake.depth, lake.x, lake.y, lake.outlet.0, lake.outlet.1, lake.outflow
        );
    }
    let count = |v: &[bool]| v.iter().filter(|&&b| b).count();
    println!("land report: cells: {} are a lake to the record, {} have a lake's shore, {} a river's tideway", count(&l.lake_cell), count(&l.shore), count(&l.estuary));
    true
}

/// The ground as drawn: its heights and slopes, the rivers and their beds, the lakes, the marsh.
fn report_ground(g: &crate::world::Ground, laid: &Laid) {
    let pct = |v: &mut Vec<f32>, q: f32| -> f32 {
        if v.is_empty() {
            return f32::NAN;
        }
        v.sort_by(|a, b| a.total_cmp(b));
        v[((v.len() - 1) as f32 * q).round() as usize]
    };
    let step = 1.0 / crate::world::VRES as f32;
    let (mut hs, mut ss) = (Vec::new(), Vec::new());
    // how far the rivers' beds lowered the ground, in metres, where they did
    let (mut bedded, mut sum, mut most) = (0usize, 0.0f64, 0.0f32);
    let (mut dry, mut marsh) = (0usize, 0.0f32);
    for j in 1..g.gh - 1 {
        for i in 1..g.gw - 1 {
            let k = j * g.gw + i;
            if g.hgt[k] > 0.022 {
                hs.push(g.hgt[k]);
                ss.push((g.hgt[k + 1] - g.hgt[k - 1]).hypot(g.hgt[k + g.gw] - g.hgt[k - g.gw]) / (2.0 * step));
                dry += 1;
                marsh += laid.marsh[k];
            }
            let cut = laid.metres[k] - metres(g.hgt[k]);
            if cut > 0.05 {
                bedded += 1;
                sum += cut as f64;
                most = most.max(cut);
            }
        }
    }
    println!(
        "land report: the land as drawn, relief {RELIEF} times: heights in cells, 10% below {:.3}, half {:.3}, 90% {:.3}, 99% {:.3}, top {:.3} (which is {:.0} m); slope as drawn, half below {:.3}, 90% {:.3}, 99% {:.3}, steepest {:.2}; steeper than 1.15, where the shader paints rock, {:.2}%",
        pct(&mut hs, 0.1),
        pct(&mut hs, 0.5),
        pct(&mut hs, 0.9),
        pct(&mut hs, 0.99),
        pct(&mut hs, 1.0),
        metres(pct(&mut hs, 1.0)),
        pct(&mut ss, 0.5),
        pct(&mut ss, 0.9),
        pct(&mut ss, 0.99),
        pct(&mut ss, 1.0),
        100.0 * ss.iter().filter(|&&s| s > 1.15).count() as f32 / ss.len().max(1) as f32
    );
    let km: f32 = g.rivers.iter().map(|rv| rv.pts.windows(2).map(|p| p[0].distance(p[1])).sum::<f32>()).sum::<f32>() * 2.0;
    println!(
        "land report: {} rivers drawn (from {} cubic m a second), {} points, {km:.0} km, the widest {:.0} m; their beds lowered {bedded} vertices of {}, by {:.1} m on average and {:.0} m at the most",
        g.rivers.len(),
        ground::MIN_FLOW,
        g.rivers.iter().map(|rv| rv.pts.len()).sum::<usize>(),
        g.rivers.iter().flat_map(|rv| rv.half.iter().copied()).fold(0.0, f32::max) * 2.0 * ground::CELL_M,
        g.gw * g.gh,
        sum / bedded.max(1) as f64,
        most
    );
    println!(
        "land report: the lakes' sheets lie over {} squares of the mesh ({:.1} square km); marsh is {:.1}% of the dry vertices; {} cells the record calls water hold dry shore",
        laid.sheet.iter().filter(|&&s| s > 0).count(),
        laid.sheet.iter().filter(|&&s| s > 0).count() as f32 * (2.0 * step) * (2.0 * step),
        100.0 * marsh / dry.max(1) as f32,
        laid.shore_cell.iter().filter(|&&s| s).count()
    );
}

/// The cells' meeting places: how many were moved off where the old island
/// would have put them, why and how far; and each town's, before and after.
fn report_middles(r: &crate::replay::Replay, l: &NewLand, cen: &[(f32, f32)]) {
    let rivers = middles::Rivers::of(l);
    let words = |f: &middles::Footing| format!("{} falling 1 in {:.0}", f.words(), 1.0 / f.slope.max(1e-4));
    let (mut land, mut moved, mut far, mut furthest) = (0usize, 0usize, 0.0f32, 0.0f32);
    // what was wrong with where the old island would have put them, by its kind (`Footing::tier`;
    // nothing wrong in kind is ground too steep), and what is still wrong with where they are
    let (mut why, mut left) = ([0usize; 6], [0usize; 6]);
    for c in 0..r.n {
        if r.terrain[c] == 0 {
            continue;
        }
        land += 1;
        let (old, new) = (middles::jittered(c, r.w), cen[c]);
        left[middles::footing(l, &rivers, new.0, new.1).tier() as usize] += 1;
        if old != new {
            moved += 1;
            let d = Vec2::new(old.0, old.1).distance(Vec2::new(new.0, new.1));
            far += d;
            furthest = furthest.max(d);
            why[middles::footing(l, &rivers, old.0, old.1).tier() as usize] += 1;
        }
    }
    println!(
        "land report: of {land} land cells' meeting places {moved} were moved: {} off water, {} off ground too low to build on, {} back from the water's edge, {} off marsh, {} back from a river, {} off steep ground; {:.0} m on average and {:.0} m at the furthest",
        why[5],
        why[4],
        why[3],
        why[2],
        why[1],
        why[0],
        far / moved.max(1) as f32 * ground::CELL_M,
        furthest * ground::CELL_M
    );
    println!(
        "land report: where they stand now: {} on dry ground back from the water, {} by a river, {} on marsh, {} at the water's edge, {} on ground too low to build on, {} on water (their cells having no better)",
        left[0], left[1], left[2], left[3], left[4], left[5]
    );
    for t in &r.header.towns {
        let c = t.node.min(r.n - 1);
        let (old, new) = (middles::jittered(c, r.w), cen[c]);
        let mut marks = Vec::new();
        if l.estuary[c] {
            marks.push("at a tideway");
        }
        if l.shore[c] {
            marks.push("on a lake's shore");
        }
        println!(
            "land report: town {} (founded {}), cell {},{}{}: its middle at {:.2},{:.2} stands on {} ({}); the old island would have put it at {:.2},{:.2}, on {}",
            t.name,
            t.founded,
            c % r.w,
            c / r.w,
            if marks.is_empty() { String::new() } else { format!(", {}", marks.join(" and ")) },
            new.0,
            new.1,
            words(&middles::footing(l, &rivers, new.0, new.1)),
            l.stands_on(new.0, new.1),
            old.0,
            old.1,
            words(&middles::footing(l, &rivers, old.0, old.1))
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arrays() -> LineArrays {
        // two rivers: three points to the sea, and two that join the first
        LineArrays {
            start: vec![0, 3, 5],
            xy: vec![1.0, 1.0, 1.5, 1.0, 2.0, 1.0, 1.5, 2.0, 1.5, 1.0],
            width: vec![6.0, 8.0, 12.0, 6.0, 7.0],
            level: vec![30.0, 12.0, 0.0, 20.0, 12.0],
            flow: vec![0.3, 1.0, 2.0, 0.3, 0.5],
            end: vec![-1, 0],
        }
    }

    #[test]
    fn the_lines_are_cut_from_the_arrays_where_their_starts_say() {
        let lines = lines_from(&arrays()).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!((lines[0].pts.len(), lines[1].pts.len()), (3, 2));
        assert_eq!((lines[0].end, lines[1].end), (End::Sea, End::River(0)));
        assert_eq!(lines[1].pts[0], Vec2::new(1.5, 2.0));
        assert_eq!(lines[1].level, [20.0, 12.0]);
        assert_eq!(End::from_code(-4), End::Lake(2));
    }

    #[test]
    fn lines_whose_starts_and_ends_do_not_agree_are_refused_by_name() {
        let refused = |edit: &dyn Fn(&mut LineArrays)| {
            let mut a = arrays();
            edit(&mut a);
            lines_from(&a).err().expect("lines that do not agree were read")
        };
        // the last start is the number of points
        assert_eq!(refused(&|a| a.start[2] = 4), "its array river_start ends at point 4, and there are 5 points");
        assert_eq!(refused(&|a| a.start[0] = 1), "its array river_start should begin at 0 and begins at 1");
        assert!(refused(&|a| a.start[1] = 7).starts_with("its array river_start goes back from 7 to 5"));
        // an end for every river, and no river joins one that is not there
        assert_eq!(refused(&|a| a.end.truncate(1)), "its array river_end has 1 rivers, where river_start has 2");
        assert_eq!(refused(&|a| a.end[1] = 2), "its array river_end says a river joins river 2, and there are 2");
        // a value for every point
        assert_eq!(refused(&|a| a.xy.truncate(8)), "its array river_xy has 8 values, where the 5 points of river_width need 10");
        assert_eq!(refused(&|a| a.level.truncate(4)), "its array river_level has 4 values, where the 5 points of river_width need 5");
        assert_eq!(refused(&|a| a.flow.push(0.0)), "its array river_flow has 6 values, where the 5 points of river_width need 5");
    }

    #[test]
    fn oxbows_are_held_to_their_starts_too() {
        let two = oxbows_from(&[0, 2, 5], &[0.0; 10], &[20.0, 30.0], &[4.0, 5.0]).unwrap();
        assert_eq!((two.len(), two[0].pts.len(), two[1].pts.len(), two[1].width), (2, 2, 3, 30.0));
        assert_eq!(oxbows_from(&[0], &[], &[], &[]).unwrap().len(), 0);
        assert_eq!(oxbows_from(&[0, 2, 4], &[0.0; 10], &[20.0, 30.0], &[4.0, 5.0]).err().unwrap(), "its array oxbow_start ends at point 4, and there are 5 points");
        assert_eq!(oxbows_from(&[0, 2, 5], &[0.0; 10], &[20.0], &[4.0, 5.0]).err().unwrap(), "its array oxbow_width has 1 values, where oxbow_start has 2 oxbows");
    }
}
