//! A works as a place: planned round its flow of material, and grown.
//!
//! `works::works` draws a works of a kind and a size as one fixed
//! arrangement in a rectangular yard. This module lays a works out on the
//! plot it was given, from what the simulation says of it:
//!
//! - **the plot** (`Plot`): its outline, and what each side of it faces: a
//!   road, a railway, water, a neighbour that hands it material over the
//!   wall or takes its output;
//! - **the process** (`Process`): the method's goods in, its heat and its
//!   good out, as the simulation's content has them (`Process::standard`
//!   reads the content shipped with the repository);
//! - **how it grew** (`stages`): the year it was founded and each year it
//!   was much enlarged, with the size it had reached.
//!
//! `plan` turns those into a `Plan`: the gate and its cart yard, a counting
//! house by the gate, a store for each input that is kept under a roof, a
//! stock yard for each that lies in the open, the process buildings of every
//! stage of its growth (each in the style of its own year, drawn by the
//! kind's own function, so a furnace is still a furnace), a warehouse or a
//! stack yard for what is made by the way out, a siding with a loading bank
//! where a railway runs by, a wharf with a crane on navigable water, and the
//! tip. Material comes in at one place, goes round the yard and leaves at
//! another. A plan is built stage by stage and nothing already placed moves
//! when a stage is added, so a works keeps what it built where it built it.
//!
//! `site` draws a plan as it stands in a year (`Now`): how hard it is
//! working, what it holds, which of its parts have fallen out of use.

use std::f32::consts::FRAC_PI_2;
use std::sync::OnceLock;

use glam::Vec3;

use super::kit::{Ctx, Face, LIFT};
use super::{footprint, grain, Kind, State, Works};
use crate::colour::{lerp, shade, Rgba};
use crate::hash::{hash2, Seed};
use crate::kit::Detail;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

// ------------------------------------------------------------------ the process

/// A good, as the simulation's content has it.
#[derive(Clone, Debug, PartialEq)]
pub struct Good {
    pub name: String,
    /// its bulk in carting (the content's `weight`)
    pub weight: f32,
    /// its price at the start, by which dear goods are told from cheap
    pub price: f32,
}

impl Good {
    pub fn new(name: &str, weight: f32, price: f32) -> Good {
        Good { name: name.to_string(), weight, price }
    }
}

/// How a good is kept at a works. The rule goes by the content's numbers, so
/// a good the generators have never heard of is kept sensibly: heavy cheap
/// stuff lies in heaps in the open, heavy made things are stacked in the
/// yard, and what is light or would spoil is kept under a roof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    /// loose in heaps in the open: ore, coal, stone, clay
    Heap,
    /// stacked in the open: pigs and bars, rails, bricks, crated machines
    Stack,
    /// timber, in long piles
    Logs,
    /// under a roof: wool, cloth, charcoal, grain, tools
    Cover,
    /// nothing to keep: what is sold is work, not goods (the content's "services")
    Nothing,
}

impl Keep {
    pub fn of(g: &Good) -> Keep {
        if g.name == "services" {
            Keep::Nothing
        } else if g.name == "timber" {
            Keep::Logs
        } else if g.weight >= 1.5 && g.price < 1.5 {
            Keep::Heap
        } else if g.weight >= 0.95 && g.name != "grain" {
            Keep::Stack
        } else {
            Keep::Cover
        }
    }
}

/// The colour of a good where it lies in a yard.
fn good_colour(g: &Good, pal: &Palette) -> Rgba {
    match g.name.as_str() {
        "coal" => *pal.goods.coal,
        "iron_ore" => *pal.ground.spoil_ironstone,
        "stone" => *pal.stone.limestone,
        "clay" => *pal.ground.spoil_clay,
        "timber" => *pal.timber.logs,
        "charcoal" => *pal.ground.charcoal_clamp,
        "iron" | "steel" | "rails" => *pal.iron.plate,
        "bricks" => *pal.goods.bricks,
        "wool" => *pal.goods.bales,
        "cloth" => *pal.goods.cloth,
        _ => match Keep::of(g) {
            Keep::Heap => *pal.ground.spoil_stone,
            Keep::Logs => *pal.timber.logs,
            _ => *pal.timber.planks,
        },
    }
}

/// What a works does, as the simulation's content says of its method.
#[derive(Clone, Debug, PartialEq)]
pub struct Process {
    /// its goods in, each with how much goes to a unit of output
    pub inputs: Vec<(Good, f32)>,
    /// the heat it burns to a unit of output (the content's heat units: a unit of coal gives 2.4)
    pub heat: f32,
    /// worker-months to a unit of output
    pub labour: f32,
    /// what a unit of monthly capacity costs to build, at the goods' starting prices
    pub capital: f32,
    pub output: Good,
}

/// The content shipped with the repository, which the lab and the tests read
/// the processes from. The game hands the planner the process from the
/// simulation it is running.
const CONTENT: &str = include_str!("../../../../data/content.toml");

fn content() -> &'static toml::Table {
    static TABLE: OnceLock<toml::Table> = OnceLock::new();
    TABLE.get_or_init(|| toml::from_str(CONTENT).expect("data/content.toml is valid"))
}

fn num(v: &toml::Value) -> f32 {
    v.as_float().map(|f| f as f32).or_else(|| v.as_integer().map(|i| i as f32)).unwrap_or(0.0)
}

impl Process {
    /// A good of the shipped content, by name.
    pub fn good(name: &str) -> Good {
        let goods = content().get("meta").and_then(|m| m.get("goods")).and_then(|g| g.as_array());
        goods
            .into_iter()
            .flatten()
            .find(|g| g.get("name").and_then(|n| n.as_str()) == Some(name))
            .map(|g| Good::new(name, g.get("weight").map_or(1.0, num), g.get("price").map_or(1.0, num)))
            .unwrap_or_else(|| Good::new(name, 1.0, 1.0))
    }

    /// The process of a method of the shipped content, if it has one of that name.
    pub fn standard(method: &str) -> Option<Process> {
        let methods = content().get("method")?.as_array()?;
        let m = methods.iter().find(|m| m.get("name").and_then(|n| n.as_str()) == Some(method))?;
        let table = |key: &str| m.get(key).and_then(|t| t.as_table());
        let inputs = table("inputs").into_iter().flatten().map(|(name, amount)| (Process::good(name), num(amount))).collect();
        let capital = table("capital").into_iter().flatten().map(|(name, amount)| Process::good(name).price * num(amount)).sum();
        Some(Process { inputs, heat: m.get("heat").map_or(0.0, num), labour: m.get("labor").map_or(1.0, num), capital, output: Process::good(m.get("output")?.as_str()?) })
    }

    /// The process of a kind of works the generators know, from the shipped
    /// content; for any other, a works that makes something of nothing known.
    pub fn of(kind: Kind) -> Process {
        Process::standard(kind.method()).unwrap_or(Process { inputs: Vec::new(), heat: 0.0, labour: 1.0, capital: 10.0, output: Good::new("goods", 1.0, 3.0) })
    }

    /// The fuel it burns, with how much to a unit of output, if it burns any.
    /// A STAND-IN: the simulation buys whichever fuel gives heat cheapest in
    /// its town that month and does not record which; here it is coal from
    /// 1790 and cordwood before.
    fn fuel(&self, year: i32) -> Option<(Good, f32)> {
        if self.heat <= 0.0 {
            return None;
        }
        Some(if year >= 1790 { (Process::good("coal"), self.heat / 2.4) } else { (Process::good("timber"), self.heat) })
    }

    /// The goods that come in, fuel among them: each with its share of the bulk that comes in.
    fn incoming(&self, year: i32) -> Vec<(Good, f32)> {
        let mut all: Vec<(Good, f32)> = self.inputs.iter().filter(|(g, _)| Keep::of(g) != Keep::Nothing).cloned().collect();
        if let Some((fuel, amount)) = self.fuel(year) {
            match all.iter_mut().find(|(g, _)| g.name == fuel.name) {
                Some(have) => have.1 += amount,
                None => all.push((fuel, amount)),
            }
        }
        let bulk: f32 = all.iter().map(|(g, a)| g.weight * a).sum();
        all.into_iter().map(|(g, a)| { let share = g.weight * a / bulk.max(1e-6); (g, share) }).collect()
    }

    /// It makes waste that is tipped: it smelts or burns.
    fn tips(&self) -> bool {
        self.heat > 0.0 || self.inputs.iter().any(|(g, _)| matches!(g.name.as_str(), "iron_ore" | "coal" | "charcoal" | "clay"))
    }

    /// What is left of a unit of output to tip: the bulk of what comes in,
    /// fuel and all, less the bulk of what goes out. A furnace leaves six
    /// times its iron in slag; a mill leaves the ashes of its boilers.
    fn waste(&self, year: i32) -> f32 {
        let fuel = self.fuel(year).map_or(0.0, |(g, a)| g.weight * a);
        (self.inputs.iter().map(|(g, a)| g.weight * a).sum::<f32>() + fuel - self.output.weight).max(0.0)
    }
}

// ------------------------------------------------------------------ the plot

/// What a side of a plot faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Edge {
    /// the road (or the lane to it): the gate is here
    Road,
    /// a wall, with a field or a neighbour that is nothing to this works beyond it
    Wall,
    /// a railway runs by: a siding along this side
    Rail,
    /// water: a leat can be led from it, and where it is `navigable` a wharf stands on it
    Water { navigable: bool },
    /// a works that makes what this one uses stands beyond: material comes in over this wall
    Fed,
    /// a works that uses what this one makes stands beyond: its output goes out over this wall
    Feeds,
}

/// The ground a works is given: an outline in the works' own frame (x along
/// its front, z out toward the road, the origin about its middle) and what
/// each side of it faces. The outline is convex, and its first side is the
/// front: it runs along x at the outline's greatest z, and the gate is in
/// it. Side `i` runs from corner `i` to corner `i + 1`.
///
/// A side may have come to face what it faces after the works was founded
/// (the railway came in 1848; the forge next door was built in 1820):
/// `since` is the year for each side, and before it the side is a wall. The
/// plan places what each stage built by what its sides faced that year, so
/// a railway's coming moves nothing that stood before it.
///
/// A side that faces the railway has a siding only if the siding can be led
/// out to the line: `line` is where the line runs by the plot, as points
/// along it in the plot's frame. A side marked `Rail` with no line within
/// `JOIN` of it is a wall to the plan, and no siding is drawn that does not
/// meet the line.
#[derive(Clone, Debug, PartialEq)]
pub struct Plot {
    pub outline: Vec<(f32, f32)>,
    pub edges: Vec<Edge>,
    pub since: Vec<i32>,
    pub line: Vec<(f32, f32)>,
}

impl Plot {
    /// A plot of this outline, the road before it and walls round the rest.
    pub fn new(outline: Vec<(f32, f32)>) -> Plot {
        let n = outline.len();
        let mut edges = vec![Edge::Wall; n];
        if n > 0 {
            edges[0] = Edge::Road;
        }
        Plot { outline, edges, since: vec![i32::MIN; n], line: Vec::new() }
    }

    /// The same plot with the railway running by it along `line` (points in the plot's frame).
    pub fn with_line(mut self, line: Vec<(f32, f32)>) -> Plot {
        self.line = line;
        self
    }

    /// A line that runs straight along side `i`, `off` outside it and `past` beyond each of its ends.
    pub fn line_along(&self, i: usize, off: f32, past: f32) -> Vec<(f32, f32)> {
        let ((a, b), o) = (self.side(i), self.outward(i));
        let len = (b.0 - a.0).hypot(b.1 - a.1).max(1e-6);
        let d = ((b.0 - a.0) / len, (b.1 - a.1) / len);
        vec![(a.0 + o.0 * off - d.0 * past, a.1 + o.1 * off - d.1 * past), (b.0 + o.0 * off + d.0 * past, b.1 + o.1 * off + d.1 * past)]
    }

    /// A rectangle, `w` along its front and `d` deep, the road before it and walls round the rest.
    pub fn rect(w: f32, d: f32) -> Plot {
        let (hw, hd) = (w * 0.5, d * 0.5);
        Plot::new(vec![(-hw, hd), (hw, hd), (hw, -hd), (-hw, -hd)])
    }

    /// A rectangle whose back runs aslant: its far right corner is drawn in by `k` of its depth.
    pub fn wedge(w: f32, d: f32, k: f32) -> Plot {
        let (hw, hd) = (w * 0.5, d * 0.5);
        Plot::new(vec![(-hw, hd), (hw, hd), (hw, -hd + d * k), (-hw, -hd)])
    }

    /// A rectangle with its far left corner cut off, `k` of the way along the back and up the side.
    pub fn cut(w: f32, d: f32, k: f32) -> Plot {
        let (hw, hd) = (w * 0.5, d * 0.5);
        Plot::new(vec![(-hw, hd), (hw, hd), (hw, -hd), (-hw + w * k, -hd), (-hw, -hd + d * k)])
    }

    /// The same plot with side `i` facing something else.
    pub fn facing(self, i: usize, edge: Edge) -> Plot {
        self.facing_from(i, edge, i32::MIN)
    }

    /// The same plot with side `i` facing something else from `year` on. (A
    /// side set to face the railway, on a plot that has not been told where
    /// the line runs, has it run straight along that side, `LINE_OFF` out:
    /// the lab's plots and the tests'.)
    pub fn facing_from(mut self, i: usize, edge: Edge, year: i32) -> Plot {
        if i < self.edges.len() {
            self.edges[i] = edge;
            self.since[i] = year;
            if edge == Edge::Rail && self.line.is_empty() {
                self.line = self.line_along(i, LINE_OFF, 0.05);
            }
        }
        self
    }

    /// What side `i` faces in `year`.
    pub fn edge(&self, i: usize, year: i32) -> Edge {
        if year >= self.since[i] { self.edges[i] } else { Edge::Wall }
    }

    /// The box round it: its least and greatest x, its least and greatest z.
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        self.outline.iter().fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |b, p| (b.0.min(p.0), b.1.max(p.0), b.2.min(p.1), b.3.max(p.1)))
    }

    pub fn area(&self) -> f32 {
        let n = self.outline.len();
        (0..n).map(|i| { let (a, b) = (self.outline[i], self.outline[(i + 1) % n]); a.0 * b.1 - b.0 * a.1 }).sum::<f32>().abs() * 0.5
    }

    fn middle(&self) -> (f32, f32) {
        let n = self.outline.len().max(1) as f32;
        let s = self.outline.iter().fold((0.0, 0.0), |s, p| (s.0 + p.0, s.1 + p.1));
        (s.0 / n, s.1 / n)
    }

    /// The two ends of side `i`.
    fn side(&self, i: usize) -> ((f32, f32), (f32, f32)) {
        (self.outline[i], self.outline[(i + 1) % self.outline.len()])
    }

    /// The way out of the plot across side `i`, as a unit vector.
    pub fn outward(&self, i: usize) -> (f32, f32) {
        let (a, b) = self.side(i);
        let (dx, dz) = (b.0 - a.0, b.1 - a.1);
        let len = dx.hypot(dz).max(1e-6);
        let n = (dz / len, -dx / len);
        let m = self.middle();
        if (a.0 - m.0) * n.0 + (a.1 - m.1) * n.1 >= 0.0 { n } else { (-n.0, -n.1) }
    }

    /// A point of side `i`, `t` of the way along it, `back` inside the plot.
    fn along(&self, i: usize, t: f32, back: f32) -> (f32, f32) {
        let ((a, b), n) = (self.side(i), self.outward(i));
        (a.0 + (b.0 - a.0) * t - n.0 * back, a.1 + (b.1 - a.1) * t - n.1 * back)
    }

    /// The side whose way out lies nearest a direction (it need not be a unit vector).
    pub fn side_toward(&self, dir: (f32, f32)) -> usize {
        (0..self.outline.len()).max_by(|&a, &b| { let (oa, ob) = (self.outward(a), self.outward(b)); (oa.0 * dir.0 + oa.1 * dir.1).total_cmp(&(ob.0 * dir.0 + ob.1 * dir.1)) }).unwrap_or(0)
    }

    /// The first side that faces what is wanted in `year`.
    fn find(&self, year: i32, want: impl Fn(Edge) -> bool) -> Option<usize> {
        (0..self.edges.len()).find(|&i| want(self.edge(i, year)))
    }

    /// The outline is sound: convex, three corners or more, a side for each,
    /// and the front along x at its greatest z.
    pub fn is_sound(&self) -> bool {
        let n = self.outline.len();
        if n < 3 || self.edges.len() != n || self.since.len() != n {
            return false;
        }
        let (_, _, _, z1) = self.bounds();
        let front = (self.outline[0].1 - z1).abs() < 1e-5 && (self.outline[1].1 - z1).abs() < 1e-5;
        let m = self.middle();
        front && (0..n).all(|i| {
            let (a, _) = self.side(i);
            let o = self.outward(i);
            self.outline.iter().all(|p| (p.0 - a.0) * o.0 + (p.1 - a.1) * o.1 <= 1e-5) && (m.0 - a.0) * o.0 + (m.1 - a.1) * o.1 < 0.0
        })
    }
}

/// How much wider and deeper than `works::footprint` the ground of a planned
/// works is: the stores, the stock yards and the lanes want room beside the
/// process buildings.
pub const GROUND: f32 = 1.15;

/// The ground a planned works of a kind wants at a size: its width along the front and its depth.
pub fn ground(kind: Kind, size: f32) -> (f32, f32) {
    let (w, d) = footprint(kind, size);
    (w * GROUND, d * GROUND)
}

// ------------------------------------------------------------------ how it grew

/// A stage of a works' growth: the year it was founded, or a year it was much enlarged,
/// and the size it had reached then (its capacity over its method's standard capacity).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stage {
    pub year: i32,
    pub size: f32,
}

/// A works is enlarged, as the plan counts it, when it has grown to this many times the size of its last stage.
pub const ENLARGED: f32 = 2.5;

/// How much of the ground a stage of growth added is its process buildings':
/// the rest goes to the stores, the yards, the lanes and the siding.
const CORE: f32 = 0.65;

/// The size the process buildings of a stage are drawn at, for a works that
/// grew from `before` to `size`: the size of a works that would stand on
/// `CORE` of the ground the stage added. A works' ground goes by its size to
/// the power 0.64 (`footprint`), so growing from `before` to `size` adds
/// `size^0.64 - before^0.64` of its kind's standard ground. Reckoned so, the
/// buildings of all a works' stages together stand on `CORE` of the ground
/// one works of its size stands on, however many stages it grew by; reckoned
/// by capacity alone they would want half as much ground again, and the last
/// stage of an old works would find no room.
pub fn stage_size(size: f32, before: f32) -> f32 {
    let ground = |s: f32| if s <= 0.0 { 0.0 } else { s.clamp(0.4, 120.0).powf(0.64) };
    (CORE * (ground(size) - ground(before)).max(0.0)).powf(1.0 / 0.64).max(0.4)
}

/// The stages of a works from its record: `history` is its size in each year
/// it stood, in order. The first is its founding; each later one is the year
/// it first stood at `ENLARGED` times the size of the stage before. (What it
/// grows between stages shows in what it holds and how hard it works.)
pub fn stages(history: &[(i32, f32)]) -> Vec<Stage> {
    let mut out: Vec<Stage> = Vec::new();
    for &(year, size) in history {
        let size = size.max(0.4);
        match out.last() {
            None => out.push(Stage { year, size }),
            Some(last) if size >= last.size * ENLARGED => out.push(Stage { year, size }),
            _ => {}
        }
    }
    out
}

// ------------------------------------------------------------------ the plan

/// What a part of a works is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// the process buildings of a stage, drawn by the kind's own function
    Core,
    /// a store for the `n`th good that comes in and is kept under a roof
    Store(usize),
    /// the warehouse for what is made
    Warehouse,
    /// the counting house, by the gate
    Office,
}

/// A building, or a group of them, placed on the plot: a rectangle in the
/// plot's frame, the way its front faces, and the stage it was built in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub role: Role,
    /// the middle of its ground, and how wide (along x) and deep (along z) that is
    pub x: f32,
    pub z: f32,
    pub w: f32,
    pub d: f32,
    /// quarter turns of its front from the plot's: 0 faces the road, 1 the plot's +x
    pub turn: u8,
    /// the stage it was built in
    pub stage: usize,
    /// the size it is drawn at
    pub size: f32,
    /// where the middle of its ground lies in the frame of what is built on
    /// it: a stage's process buildings are laid out by their kind on a yard
    /// of their own, and take of the plot only what they cover of it
    pub off: (f32, f32),
}

impl Part {
    /// The way its front faces, in the plot's frame.
    fn front(&self) -> (f32, f32) {
        [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)][self.turn as usize % 4]
    }

    /// The way its own +x runs, in the plot's frame.
    fn right(&self) -> (f32, f32) {
        [(1.0, 0.0), (0.0, -1.0), (-1.0, 0.0), (0.0, 1.0)][self.turn as usize % 4]
    }

    /// Where the origin of what is built on it stands, in the plot's frame.
    fn origin(&self) -> (f32, f32) {
        let (r, f) = (self.right(), self.front());
        (self.x - r.0 * self.off.0 - f.0 * self.off.1, self.z - r.1 * self.off.0 - f.1 * self.off.1)
    }

    /// The place before its door.
    fn door(&self) -> (f32, f32) {
        let f = self.front();
        (self.x + f.0 * (self.w * 0.5 + 0.004), self.z + f.1 * (self.d * 0.5 + 0.004))
    }
}

/// What lies in an open yard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// the `n`th good that comes in
    In(usize),
    /// what is made
    Out,
    /// what is tipped: slag, ashes, wasters
    Tip,
}

/// A piece of open ground set aside for stock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Yard {
    pub hold: Hold,
    pub x: f32,
    pub z: f32,
    pub w: f32,
    pub d: f32,
}

/// A works laid out on its plot.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub kind: Kind,
    pub process: Process,
    pub plot: Plot,
    pub stages: Vec<Stage>,
    pub seed: Seed,
    /// the buildings, in the order they were built
    pub parts: Vec<Part>,
    /// the open stock yards
    pub yards: Vec<Yard>,
    /// where the gate is along the front (x), and how wide
    pub gate: (f32, f32),
    /// the cart yard inside the gate: its middle, width and depth
    pub cart_yard: (f32, f32, f32, f32),
    /// the lane from the cart yard into the works, in a works big enough to have one: its two ends and its width
    pub lane: Option<((f32, f32), (f32, f32), f32)>,
    /// where material comes in, and where what is made goes out
    pub receive: (f32, f32),
    pub dispatch: (f32, f32),
    /// the year the plan is for: what each side of the plot faces now
    pub year: i32,
    /// the side a siding runs along, and the side a wharf stands on, each
    /// with the stretch of the side it has (from 0 to 1 along it): what
    /// stood there before the railway came is left standing
    pub siding: Option<(usize, f32, f32)>,
    pub wharf: Option<(usize, f32, f32)>,
    /// the track that leads the siding out to the line: points in the plot's
    /// frame, the first where it leaves the siding and the last on the line
    /// (a works has no siding that cannot be led out), and whether it leaves
    /// from the start of the siding's stretch or from its end (the buffer
    /// stop stands at the other)
    pub spur: Vec<(f32, f32)>,
    pub spur_at_start: bool,
    /// the goods that come in (fuel among them), each with its share of the bulk
    pub incoming: Vec<(Good, f32)>,
    /// how many parts found no room at their own size (and were built smaller, or a store not at all), and how many stages' process buildings found none
    pub squeezed: usize,
    pub dropped: usize,
    /// the ground the works has taken in so far: the part of its plot that
    /// its buildings and yards stand on, which is what is walled and what
    /// its ground is laid over. It grows with the works toward the plot's edges.
    pub enclosed: Plot,
    /// the size its stock yards are laid out for: as big as it can grow
    /// before it must build again, or as big as its plot was set aside for
    pub laid_for: f32,
}

/// The ground a siding takes along its side, a wharf along its, and the room a wall wants.
const SIDING: f32 = 0.022;
const WHARF: f32 = 0.018;
const WALL: f32 = 0.005;
/// How near its railway or its water a works must have built to have a siding or a wharf there.
const REACH: f32 = 0.08;
/// The ground a works keeps for stock, as shares of the ground a works of its
/// kind and size stands on (`footprint`): for what comes in and lies in the
/// open, for what it makes, and for its tip at the most; and the least ground
/// worth calling a yard. These are the plan's rule, stated: the simulation
/// keeps no stock at a works.
const STOCK_IN: f32 = 0.26;
const STOCK_OUT: f32 = 0.13;
const TIP: f32 = 0.11;
const YARD_LEAST: f32 = 0.00035;
/// How far outside a side the lab's railway runs (`Plot::facing_from`).
pub const LINE_OFF: f32 = 0.022;
/// How near a side the line must run for a siding along that side to be led
/// out to it (320 m), and the longest run a spur makes from the siding's end
/// to the line.
pub const JOIN: f32 = 0.16;
const SPUR: f32 = 0.36;
/// How far in from the end of its stretch a siding's track stops where the
/// spur leaves it, so that the spur has room to curve out across the open
/// side and not through the wall at the corner.
const SPUR_ROOM: f32 = 0.03;

/// Points along a line of points, no further apart than `step`, each with the way the line runs there.
fn along_line(line: &[(f32, f32)], step: f32) -> Vec<((f32, f32), (f32, f32))> {
    let mut out = Vec::new();
    for w in line.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        if len < 1e-6 {
            continue;
        }
        let dir = ((b.0 - a.0) / len, (b.1 - a.1) / len);
        let n = ((len / step).ceil() as usize).max(1);
        for i in 0..=n {
            let t = i as f32 / n as f32;
            out.push(((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t), dir));
        }
    }
    out
}

/// How far a point is from a line of points.
fn off_line(line: &[(f32, f32)], p: (f32, f32)) -> f32 {
    line.windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let (abx, abz) = (b.0 - a.0, b.1 - a.1);
            let t = (((p.0 - a.0) * abx + (p.1 - a.1) * abz) / (abx * abx + abz * abz).max(1e-9)).clamp(0.0, 1.0);
            (a.0 + abx * t - p.0).hypot(a.1 + abz * t - p.1)
        })
        .fold(f32::MAX, f32::min)
}

/// The two ends of a siding's track along the stretch `f0` to `f1` of a
/// side: the end the spur leaves from first, the end with the stop second.
fn siding_track(plot: &Plot, side: usize, f0: f32, f1: f32, at_start: bool) -> ((f32, f32), (f32, f32)) {
    let (a, b) = (plot.along(side, f0 + 0.03 * (f1 - f0), 0.008), plot.along(side, f1 - 0.03 * (f1 - f0), 0.008));
    let (out, stop) = if at_start { (a, b) } else { (b, a) };
    let len = (stop.0 - out.0).hypot(stop.1 - out.1).max(1e-6);
    let room = SPUR_ROOM.min(len * 0.4);
    ((out.0 + (stop.0 - out.0) / len * room, out.1 + (stop.1 - out.1) / len * room), stop)
}

/// The track that leads a siding out to the line: from the siding's end
/// `from`, where it runs the way `dir`, in a curve to the nearest place
/// ahead on the line that it can reach at a gentle angle, where it runs in
/// with the line. Nothing if the line is not there to be met.
fn spur_to(line: &[(f32, f32)], from: (f32, f32), dir: (f32, f32)) -> Option<Vec<(f32, f32)>> {
    let mut best: Option<(f32, (f32, f32), (f32, f32))> = None;
    for (p, t) in along_line(line, 0.006) {
        let (dx, dz) = (p.0 - from.0, p.1 - from.1);
        let (along, across) = (dx * dir.0 + dz * dir.1, (dx * dir.1 - dz * dir.0).abs());
        let t = if t.0 * dir.0 + t.1 * dir.1 >= 0.0 { t } else { (-t.0, -t.1) };
        // ahead, at a turnout's angle or less, and where the line runs much the way the siding does
        if along < 0.025 || along > SPUR || across > 0.5 * along || t.0 * dir.0 + t.1 * dir.1 < 0.7 {
            continue;
        }
        if best.map_or(true, |b| along < b.0) {
            best = Some((along, p, t));
        }
    }
    let (_, to, t) = best?;
    let reach = (to.0 - from.0).hypot(to.1 - from.1) * 0.4;
    let (p1, p2) = ((from.0 + dir.0 * reach, from.1 + dir.1 * reach), (to.0 - t.0 * reach, to.1 - t.1 * reach));
    const N: usize = 8;
    Some(
        (0..=N)
            .map(|i| {
                let s = i as f32 / N as f32;
                let (a, b, c, d) = ((1.0 - s).powi(3), 3.0 * (1.0 - s).powi(2) * s, 3.0 * (1.0 - s) * s * s, s.powi(3));
                (a * from.0 + b * p1.0 + c * p2.0 + d * to.0, a * from.1 + b * p1.1 + c * p2.1 + d * to.1)
            })
            .collect(),
    )
}

/// The free ground of a plot while it is being laid out.
struct Field {
    /// the plot's sides, each moved in by what stands along it: a point is inside while `n . p <= c` for all
    planes: Vec<(f32, f32, f32)>,
    taken: Vec<(f32, f32, f32, f32)>,
    bounds: (f32, f32, f32, f32),
    step: f32,
    /// how far from the plot's sides the places tried begin
    back: f32,
}

impl Field {
    fn new(plot: &Plot) -> Field {
        Field::within(plot, WALL)
    }

    /// The whole of a plot to within `back` of its sides.
    fn within(plot: &Plot, back: f32) -> Field {
        let planes = (0..plot.outline.len())
            .map(|i| {
                let ((a, _), n) = (plot.side(i), plot.outward(i));
                (n.0, n.1, a.0 * n.0 + a.1 * n.1 - back)
            })
            .collect();
        let bounds = plot.bounds();
        let step = ((bounds.1 - bounds.0).min(bounds.3 - bounds.2) / 44.0).max(0.003);
        Field { planes, taken: Vec::new(), bounds, step, back }
    }

    /// How far inside the plot a box is: the least room between it and any side (negative when it is out).
    fn room(&self, x0: f32, z0: f32, x1: f32, z1: f32) -> f32 {
        let mut least = f32::MAX;
        for &(nx, nz, c) in &self.planes {
            let far = (if nx >= 0.0 { x1 } else { x0 }) * nx + (if nz >= 0.0 { z1 } else { z0 }) * nz;
            least = least.min(c - far);
        }
        least
    }

    fn free(&self, x0: f32, z0: f32, x1: f32, z1: f32, gap: f32) -> bool {
        self.room(x0, z0, x1, z1) >= 0.0 && !self.taken.iter().any(|t| x0 < t.2 + gap && x1 > t.0 - gap && z0 < t.3 + gap && z1 > t.1 - gap)
    }

    /// How near a box stands to what is already taken (the gap to the nearest), if anything is.
    fn nearest(&self, x0: f32, z0: f32, x1: f32, z1: f32) -> f32 {
        self.taken.iter().map(|t| (t.0 - x1).max(x0 - t.2).max(0.0).hypot((t.1 - z1).max(z0 - t.3).max(0.0))).fold(f32::MAX, f32::min)
    }

    /// The best free place for a box `w` by `d`: its middle. `cost` says how
    /// bad a place is; a place hard against the plot's edge or a neighbour is
    /// liked a little more, so that buildings line the edges and each other.
    fn place(&self, w: f32, d: f32, gap: f32, hug: f32, cost: impl Fn(f32, f32) -> f32) -> Option<(f32, f32)> {
        // (the places tried begin hard against the walls)
        let (bx0, bx1, bz0, bz1) = (self.bounds.0 + self.back, self.bounds.1 - self.back, self.bounds.2 + self.back, self.bounds.3 - self.back);
        let (nx, nz) = ((((bx1 - bx0 - w) / self.step).floor()).max(0.0) as usize, (((bz1 - bz0 - d) / self.step).floor()).max(0.0) as usize);
        let span = (bx1 - bx0).hypot(bz1 - bz0);
        if bx1 - bx0 < w || bz1 - bz0 < d {
            return None;
        }
        let mut best: Option<(f32, (f32, f32))> = None;
        // from the left and from the right, from the back and from the front, so that both walls can be stood against
        let mut xs: Vec<f32> = (0..=nx).flat_map(|i| [bx0 + i as f32 * self.step, bx1 - w - i as f32 * self.step]).collect();
        let mut zs: Vec<f32> = (0..=nz).flat_map(|j| [bz0 + j as f32 * self.step, bz1 - d - j as f32 * self.step]).collect();
        xs.dedup();
        zs.dedup();
        for &z0 in &zs {
            for &x0 in &xs {
                let (x1, z1) = (x0 + w, z0 + d);
                if !self.free(x0, z0, x1, z1, gap) {
                    continue;
                }
                let room = self.room(x0, z0, x1, z1);
                let mut c = cost((x0 + x1) * 0.5, (z0 + z1) * 0.5);
                if room < self.step * 1.5 {
                    c -= span * 0.06 * hug;
                }
                if self.nearest(x0, z0, x1, z1) < gap + self.step * 1.5 {
                    c -= span * 0.03 * hug;
                }
                if best.map_or(true, |(b, _)| c < b - 1e-6) {
                    best = Some((c, ((x0 + x1) * 0.5, (z0 + z1) * 0.5)));
                }
            }
        }
        best.map(|(_, at)| at)
    }

    /// Keep what is built from now on back from side `i` by `back`: a siding or a wharf has come to it.
    fn set_back(&mut self, plot: &Plot, i: usize, back: f32) {
        let ((a, _), n) = (plot.side(i), plot.outward(i));
        self.planes[i].2 = self.planes[i].2.min(a.0 * n.0 + a.1 * n.1 - back);
    }

    /// The longest stretch of side `i` along which a strip `back` deep is free of what is built.
    fn stretch(&self, plot: &Plot, i: usize, back: f32) -> Option<(f32, f32)> {
        const N: usize = 40;
        let free: Vec<bool> = (0..N)
            .map(|k| {
                let p = plot.along(i, (k as f32 + 0.5) / N as f32, back * 0.5);
                let h = back * 0.5;
                !self.taken.iter().any(|t| p.0 - h < t.2 && p.0 + h > t.0 && p.1 - h < t.3 && p.1 + h > t.1)
            })
            .collect();
        let (mut best, mut from) = ((0, 0), 0);
        for k in 0..=N {
            if k == N || !free[k] {
                if k - from > best.1 - best.0 {
                    best = (from, k);
                }
                from = k + 1;
            }
        }
        (best.1 > best.0).then(|| (best.0 as f32 / N as f32, best.1 as f32 / N as f32))
    }

    fn take(&mut self, x: f32, z: f32, w: f32, d: f32) {
        self.taken.push((x - w * 0.5, z - d * 0.5, x + w * 0.5, z + d * 0.5));
    }
}

/// The side of its own ground a kind leads its race in from, when a wheel
/// turns it (+1 its own +x, -1 its own -x), so that the wheel is stood to the water.
fn race_side(kind: Kind) -> f32 {
    match kind {
        Kind::CharcoalFurnace | Kind::Smithy | Kind::Millwright | Kind::HandLoom => -1.0,
        _ => 1.0,
    }
}

/// The ground a covered store or a warehouse of a works takes, by the size its stage reached.
fn store_ground(kind: Kind, size: f32, share: f32) -> (f32, f32) {
    // (a town trade's store is to the scale of its shop)
    let g = grain(size) * (kind.plot().0 / 0.13).clamp(0.6, 1.0);
    ((0.03 + 0.016 * share) * g * size.clamp(0.4, 60.0).powf(0.16), 0.024 * g)
}

/// The ground the process buildings of a stage cover, in their own frame
/// (x0, x1, z0, z1): they are built once as they will be drawn, and
/// measured, so that they take of the plot what they stand on and no more.
#[allow(clippy::too_many_arguments)]
fn core_cover(kind: Kind, process: &Process, size: f32, year: i32, water: bool, seed: Seed, pal: &Palette) -> (f32, f32, f32, f32) {
    let (mut solid, mut smoke, mut flat) = (MeshBuf::default(), MeshBuf::default(), MeshBuf::default());
    let site = Site::level(Vec3::ZERO, 0.0);
    let (fw, fd) = footprint(kind, size);
    // (at the play level, which has every wall and stack where the near level has it, at a third of the work)
    let w = Works { kind, size, founded: year, state: State::Working, water, rail: false, detail: Detail::Play, seed };
    let mut c = Ctx::part(&mut solid, &mut smoke, &mut flat, &site, &w, pal, fw * 0.5, fd * 0.5, 0.0);
    c.stock_apart = true;
    if kind == Kind::Other { super::plain::process(&mut c, process) } else { super::raise(&mut c) }
    c.finish();
    let mut cover: Option<(f32, f32, f32, f32)> = None;
    for b in [&solid, &flat] {
        if let Some((lo, hi)) = b.bounds() {
            let c = cover.get_or_insert((lo.x, hi.x, lo.z, hi.z));
            *c = (c.0.min(lo.x), c.1.max(hi.x), c.2.min(lo.z), c.3.max(hi.z));
        }
    }
    let (hw, hd) = (fw * 0.5, fd * 0.5);
    let m = 0.0045;
    match cover {
        Some(c) => ((c.0 - m).max(-hw), (c.1 + m).min(hw), (c.2 - m).max(-hd), (c.3 + m).min(hd)),
        None => (-hw, hw, -hd, hd),
    }
}

/// Lay a works out on its plot. `stages` is how it grew (`stages`): the plan
/// of a works with a stage more is the plan without it and what that stage
/// built, so nothing that stands moves when a works is enlarged on the same plot.
///
/// `year` is the year the plan is drawn for: the open yards, the siding and
/// the wharf go by what the plot's sides face then, while each stage's
/// buildings were placed by what they faced when it was built.
pub fn plan(kind: Kind, process: &Process, stages: &[Stage], plot: &Plot, year: i32, seed: Seed, pal: &Palette) -> Plan {
    assert!(plot.is_sound(), "a plot must be convex with its front along x: {plot:?}");
    let whole = plot;
    // a workshop's plot has no room for a siding or a quay: its railway and its water are walls to it
    let small = { let b = plot.bounds(); (b.1 - b.0).min(b.3 - b.2) < 0.12 };
    // and a side faces the railway only where a siding can be led out to the line: where the
    // line runs within `JOIN` of some point of it
    let joins = |i: usize| plot.line.len() >= 2 && [0.1f32, 0.3, 0.5, 0.7, 0.9].into_iter().any(|t| off_line(&plot.line, plot.along(i, t, 0.0)) <= JOIN);
    let demoted = Plot {
        edges: plot
            .edges
            .iter()
            .enumerate()
            .map(|(i, &e)| match e {
                Edge::Rail if small || !joins(i) => Edge::Wall,
                Edge::Water { navigable: true } if small => Edge::Water { navigable: false },
                e => e,
            })
            .collect(),
        ..plot.clone()
    };
    let plot = &demoted;
    let stages: Vec<Stage> = if stages.is_empty() { vec![Stage { year: 1800, size: 1.0 }] } else { stages.to_vec() };
    let founded = stages[0].year;
    let (bx0, bx1, bz0, bz1) = plot.bounds();
    let (pw, pd) = (bx1 - bx0, bz1 - bz0);
    let span = pw.hypot(pd);
    let mut field = Field::new(plot);
    let incoming = process.incoming(founded);

    // the gate, off the middle of the front, and the cart yard inside it
    let (fa, fb) = plot.side(0);
    let gw = (0.022 * grain(stages[0].size)).min((fb.0 - fa.0).abs() * 0.4);
    let gx = fa.0 + (fb.0 - fa.0) * (0.36 + 0.28 * seed.pick(40));
    let cy = (gw + 0.012, (pd * 0.14).clamp(0.005, 0.034));
    let cart_yard = (gx, bz1 - WALL - cy.1 * 0.5, cy.0, cy.1);
    field.take(cart_yard.0, cart_yard.1, cart_yard.2, cart_yard.3);
    let gate = (gx, bz1 - WALL - cy.1);
    // the ground beside the gate is kept for the counting house from the first
    let office_ground = if pw < 0.12 {
        None
    } else {
        let g = grain(1.5);
        let (lw, ld) = (0.03 * g, 0.021 * g);
        [1.0f32, -1.0].into_iter().map(|side| (gx + side * (cy.0 * 0.5 + 0.003 + lw * 0.5), bz1 - WALL - ld * 0.5, lw, ld)).find(|o| field.free(o.0 - o.2 * 0.5, o.1 - o.3 * 0.5, o.0 + o.2 * 0.5, o.1 + o.3 * 0.5, 0.0))
    };
    // (which of the boxes taken is the office's: the wall goes round it once it is built)
    let office_box = office_ground.map(|o| {
        field.take(o.0, o.1, o.2, o.3);
        field.taken.len() - 1
    });
    // a lane on into a works with ground enough for buildings either side of it
    let lane = (pw >= 0.3 && pd >= 0.2).then(|| {
        let (lw, len) = (0.012, pd * 0.5);
        let (a, b) = ((gx, cart_yard.1 - cy.1 * 0.5), (gx, cart_yard.1 - cy.1 * 0.5 - len));
        field.take(gx, (a.1 + b.1) * 0.5, lw, len);
        (a, b, lw)
    });
    // (the boxes taken so far are ground kept, not ground built on: the cart yard, the office's, the lane's)
    let reserved = field.taken.len();
    // the box round what stands so far (the cart yard and every building and yard), and how far
    // a box would reach out of it: a works takes in new ground only when it has none left within
    // its walls, so its buildings stand close and its yards lie in the bays between them
    let built_box = |field: &Field| field.taken.iter().enumerate().filter(|(k, _)| *k == 0 || *k >= reserved).fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |b, (_, t)| (b.0.min(t.0), b.1.max(t.2), b.2.min(t.1), b.3.max(t.3)));
    let out_of = |b: (f32, f32, f32, f32), x: f32, z: f32, w: f32, d: f32| (b.0 - (x - w * 0.5)).max(0.0) + ((x + w * 0.5) - b.1).max(0.0) + (b.2 - (z - d * 0.5)).max(0.0) + ((z + d * 0.5) - b.3).max(0.0);

    // where material comes in and where what is made goes out, in a year: over a neighbour's
    // wall, by the siding, at the wharf, or through the gate, in to its one side and out at its other
    let by = |side: usize, t: f32, back: f32| plot.along(side, t, back);
    let aside = gw * 0.5 + 0.03;
    // a works has a siding or a wharf of its own once it has built out to within reach of the
    // line or the water: a small works on ground set aside for a great one carts to the station
    // (and a works that takes in and sends out nothing with any weight to it, a shop, has neither)
    let carries = !incoming.is_empty() || Keep::of(&process.output) != Keep::Nothing;
    let reaches = |side: usize, parts: &[Part]| {
        let ((a, _), n) = (plot.side(side), plot.outward(side));
        carries && parts.iter().any(|p| (a.0 - p.x) * n.0 + (a.1 - p.z) * n.1 - (p.w * n.0.abs() + p.d * n.1.abs()) * 0.5 <= REACH)
    };
    let ways = |year: i32, parts: &[Part]| -> ((f32, f32), (f32, f32)) {
        let siding = plot.find(year, |e| e == Edge::Rail).filter(|s| reaches(*s, parts));
        let wharf = plot.find(year, |e| e == Edge::Water { navigable: true }).filter(|s| reaches(*s, parts));
        let fed = plot.find(year, |e| e == Edge::Fed);
        let feeds = plot.find(year, |e| e == Edge::Feeds);
        let receive = fed.map(|s| by(s, 0.5, WALL)).or(siding.map(|s| by(s, 0.3, SIDING))).or(wharf.map(|s| by(s, 0.35, WHARF))).unwrap_or((gx - aside, bz1 - 0.02));
        let dispatch = feeds.map(|s| by(s, 0.5, WALL)).or(siding.map(|s| by(s, 0.72, SIDING))).or(wharf.map(|s| by(s, 0.7, WHARF))).unwrap_or((gx + aside, bz1 - 0.02));
        (receive, dispatch)
    };
    let stream = plot.find(founded, |e| matches!(e, Edge::Water { .. }));
    // what is built once a siding or a wharf has come keeps back from its side
    let set_back = |field: &mut Field, year: i32| {
        for i in 0..plot.edges.len() {
            match plot.edge(i, year) {
                Edge::Rail => field.set_back(plot, i, SIDING),
                Edge::Water { navigable: true } => field.set_back(plot, i, WHARF),
                _ => {}
            }
        }
    };

    let dist = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).hypot(a.1 - b.1);
    let mut parts: Vec<Part> = Vec::new();
    let (mut squeezed, mut dropped) = (0, 0);
    let mut last_core: Option<(f32, f32)> = None;
    let mut stored = 0.0f32;

    for (i, st) in stages.iter().enumerate() {
        let before = if i == 0 { 0.0 } else { stages[i - 1].size };
        set_back(&mut field, st.year);
        // ---- the process buildings of the stage
        let wheel = i == 0 && st.year < 1790 && stream.is_some();
        let by_water = i == 0 && stream.is_some();
        let wanted = stage_size(st.size, before);
        // the best free place for a group of process buildings of a size, as the `n`th part of
        // the works, by the group before it: the part if there is room, and its ground in its own frame
        let try_core = |field: &Field, n: usize, size: f32, last_core: Option<(f32, f32)>| -> (Option<Part>, (f32, f32), (f32, f32)) {
            let cover = core_cover(kind, process, size, st.year, by_water, part_seed(seed, n), pal);
            let (lw, ld) = (cover.1 - cover.0, cover.3 - cover.2);
            let off = ((cover.0 + cover.1) * 0.5, (cover.2 + cover.3) * 0.5);
            let mut best: Option<(f32, Part)> = None;
            let standing = built_box(field);
            // (a works enlarged once the line runs by it builds toward the line: how far a box is from that side)
            let line_side = plot.find(st.year, |e| e == Edge::Rail);
            let to_line = |x: f32, z: f32, w: f32, d: f32| line_side.map_or(0.0, |s| { let ((a, _), n) = (plot.side(s), plot.outward(s)); ((a.0 - x) * n.0 + (a.1 - z) * n.1 - (w * n.0.abs() + d * n.1.abs()) * 0.5 - SIDING).max(0.0) });
            for turn in 0..4u8 {
                let (w, d) = if turn % 2 == 0 { (lw, ld) } else { (ld, lw) };
                let part = |x: f32, z: f32| Part { role: Role::Core, x, z, w, d, turn, stage: i, size, off };
                let (front, right) = (part(0.0, 0.0).front(), part(0.0, 0.0).right());
                let cost = |x: f32, z: f32| {
                    let mut c = match (last_core, wheel) {
                        // the first stands just in from the gate; a wheel stands to the water
                        (None, true) => 0.5 * dist((x, z), gate) + 1.2 * { let o = plot.outward(stream.unwrap()); let a = plot.side(stream.unwrap()).0; ((a.0 - x) * o.0 + (a.1 - z) * o.1).abs() },
                        (None, false) => dist((x, z), (gx - (cy.0 * 0.5 + 0.004 + w * 0.5), bz1 - WALL - d * 0.5)),
                        // an enlargement stands by what it enlarges, and no further from the gate than it need
                        (Some(at), _) => dist((x, z), at) + 0.6 * dist((x, z), gate) + 0.7 * out_of(standing, x, z, w, d) + 0.9 * to_line(x, z, w, d),
                    };
                    // its front to the gate's side of the yard, not to the wall behind it
                    c -= span * 0.05 * (front.0 * (gate.0 - x) + front.1 * (gate.1 + 0.02 - z)).signum().max(0.0);
                    if wheel {
                        let o = plot.outward(stream.unwrap());
                        c -= span * 0.2 * ((right.0 * o.0 + right.1 * o.1) * race_side(kind)).max(0.0);
                    }
                    c
                };
                if let Some((x, z)) = field.place(w, d, 0.004, 1.0, &cost) {
                    let c = cost(x, z);
                    if best.as_ref().map_or(true, |(b, _)| c < *b - 1e-6) {
                        best = Some((c, part(x, z)));
                    }
                }
            }
            (best.map(|b| b.1), (lw, ld), off)
        };
        // As one group where the ground has room for it. Where the ground that is left is
        // broken up (an old works, its plot half built over), as two, three or four groups
        // that come to the same capacity between them: an enlargement built as two ranges.
        // And only then smaller than the stage was, by small steps, so that it is built as
        // near its size as its ground has room for.
        let mut groups: Vec<Part> = Vec::new();
        for n in 1..=4usize {
            let each = wanted / n as f32;
            if n > 1 && each < 0.5 {
                break;
            }
            let mark = field.taken.len();
            let mut by = last_core;
            for _ in 0..n {
                let (Some(p), _, _) = try_core(&field, parts.len() + groups.len(), each, by) else { break };
                field.take(p.x, p.z, p.w, p.d);
                by = Some((p.x, p.z));
                groups.push(p);
            }
            if groups.len() == n {
                break;
            }
            field.taken.truncate(mark);
            groups.clear();
        }
        if groups.is_empty() {
            let mut size = wanted;
            loop {
                size = (size * 0.8).max(0.4);
                let (found, (lw, ld), off) = try_core(&field, parts.len(), size, last_core);
                if let Some(p) = found {
                    field.take(p.x, p.z, p.w, p.d);
                    squeezed += (p.size < wanted * 0.99) as usize;
                    groups.push(p);
                    break;
                }
                if size <= 0.4 {
                    // no room even for the smallest: the first works of a plot stands in the middle
                    // of it all the same, over its own cart yard
                    if i == 0 {
                        let bare = Field::within(plot, 0.0005);
                        let at = bare.place(lw, ld, 0.0, 1.0, |x, z| dist((x, z), gate)).unwrap_or(plot.middle());
                        field.take(at.0, at.1, lw, ld);
                        groups.push(Part { role: Role::Core, x: at.0, z: at.1, w: lw, d: ld, turn: 0, stage: i, size, off });
                    }
                    break;
                }
            }
        }
        match groups.last() {
            Some(p) => last_core = Some((p.x, p.z)),
            None => dropped += 1,
        }
        parts.extend(groups);

        let (receive, dispatch) = ways(st.year, &parts);

        // ---- the counting house, once there are hands enough to want a clerk
        if st.size >= 1.5 && before < 1.5 {
            // (a works on a plot too small for one keeps its books in the shop)
            if let Some((x, z, w, d)) = office_ground {
                parts.push(Part { role: Role::Office, x, z, w, d, turn: 0, stage: i, size: st.size, off: (0.0, 0.0) });
            }
        }

        // ---- stores and a warehouse, at the founding and again each time it has grown fourfold since the last
        if st.size >= 0.8 && (stored == 0.0 || st.size >= stored * 4.0) {
            stored = st.size;
            let mut wants: Vec<(Role, f32, (f32, f32))> = incoming.iter().enumerate().filter(|(_, (g, _))| Keep::of(g) == Keep::Cover).map(|(n, (_, share))| (Role::Store(n), *share, receive)).collect();
            if Keep::of(&process.output) == Keep::Cover {
                wants.push((Role::Warehouse, 1.0, dispatch));
            }
            for (role, share, by) in wants {
                let (mut lw, mut ld) = store_ground(kind, st.size - before * 0.5, share);
                let mut placed = false;
                let standing = built_box(&field);
                for _ in 0..6 {
                    let mut best: Option<(f32, Part)> = None;
                    for turn in 0..4u8 {
                        let (w, d) = if turn % 2 == 0 { (lw, ld) } else { (ld, lw) };
                        let part = |x: f32, z: f32| Part { role, x, z, w, d, turn, stage: i, size: st.size, off: (0.0, 0.0) };
                        let front = part(0.0, 0.0).front();
                        let cost = |x: f32, z: f32| dist((x, z), by) + 0.4 * dist((x, z), gate) + 0.7 * out_of(standing, x, z, w, d) - span * 0.05 * (front.0 * (gate.0 - x) + front.1 * (gate.1 - z)).signum().max(0.0);
                        if let Some((x, z)) = field.place(w, d, 0.003, 3.0, &cost) {
                            let c = cost(x, z);
                            if best.as_ref().map_or(true, |(b, _)| c < *b - 1e-6) {
                                best = Some((c, part(x, z)));
                            }
                        }
                    }
                    if let Some((_, p)) = best {
                        field.take(p.x, p.z, p.w, p.d);
                        parts.push(p);
                        placed = true;
                        break;
                    }
                    (lw, ld) = ((lw * 0.78).max(0.018), (ld * 0.88).max(0.014));
                }
                // (a works with no room for a store keeps its stock in the shop)
                if !placed {
                    squeezed += 1;
                }
            }
        }
    }

    // ---- the siding and the wharf, along what of their sides is free
    set_back(&mut field, year);
    let (receive, dispatch) = ways(year, &parts);
    let run = |s: &(usize, f32, f32)| { let (a, b) = plot.side(s.0); (s.2 - s.1) * (b.0 - a.0).hypot(b.1 - a.1) };
    let siding = plot.find(year, |e| e == Edge::Rail).filter(|s| reaches(*s, &parts)).and_then(|i| field.stretch(plot, i, SIDING).map(|(a, b)| (i, a, b))).filter(|s| run(s) >= 0.06);
    // the siding is led out to the line from whichever of its ends has the shorter way, and a
    // siding that cannot be led out is not laid
    let spur = siding.and_then(|(i, f0, f1)| {
        [true, false]
            .into_iter()
            .filter_map(|at_start| {
                let (out, stop) = siding_track(plot, i, f0, f1, at_start);
                let len = (out.0 - stop.0).hypot(out.1 - stop.1).max(1e-6);
                spur_to(&plot.line, out, ((out.0 - stop.0) / len, (out.1 - stop.1) / len)).map(|pts| (pts, at_start))
            })
            .min_by(|a, b| { let run = |p: &Vec<(f32, f32)>| (p[0].0 - p[p.len() - 1].0).hypot(p[0].1 - p[p.len() - 1].1); run(&a.0).total_cmp(&run(&b.0)) })
    });
    let siding = siding.filter(|_| spur.is_some());
    let (spur, spur_at_start) = spur.unwrap_or((Vec::new(), true));
    // (with no siding after all, what comes and goes does so by the gate)
    let (receive, dispatch) = if siding.is_none() && plot.find(year, |e| e == Edge::Rail).is_some_and(|s| reaches(s, &parts)) {
        let (fed, feeds) = (plot.find(year, |e| e == Edge::Fed), plot.find(year, |e| e == Edge::Feeds));
        let wharf = plot.find(year, |e| e == Edge::Water { navigable: true }).filter(|s| reaches(*s, &parts));
        (fed.map(|s| by(s, 0.5, WALL)).or(wharf.map(|s| by(s, 0.35, WHARF))).unwrap_or((gx - aside, bz1 - 0.02)), feeds.map(|s| by(s, 0.5, WALL)).or(wharf.map(|s| by(s, 0.7, WHARF))).unwrap_or((gx + aside, bz1 - 0.02)))
    } else {
        (receive, dispatch)
    };
    let wharf = plot.find(year, |e| e == Edge::Water { navigable: true }).filter(|s| reaches(*s, &parts)).and_then(|i| field.stretch(plot, i, WHARF).map(|(a, b)| (i, a, b))).filter(|s| run(s) >= 0.03);
    let built = field.taken.len();
    let mut strips: Vec<(f32, f32, f32, f32)> = Vec::new();
    for (side, back) in [(siding, SIDING), (wharf, WHARF)] {
        if let Some((i, f0, f1)) = side {
            // (the yards keep off them: length by length, so that a side that runs aslant keeps
            // its strip and not the whole corner of the plot)
            const N: usize = 8;
            for k in 0..N {
                let (a, b) = (plot.along(i, f0 + (f1 - f0) * k as f32 / N as f32, back * 0.5), plot.along(i, f0 + (f1 - f0) * (k + 1) as f32 / N as f32, back * 0.5));
                strips.push(((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5, (a.0 - b.0).abs() + back, (a.1 - b.1).abs() + back));
            }
        }
    }

    // ---- the open yards, in what ground the buildings left: stock by the way in, what is
    // made by the way out, the tip at the corner of the works furthest from the gate. (Stock
    // is not built, so a yard may be moved when the works is enlarged.)
    //
    // How much ground each is given is a RULE OF THE PLAN'S, not a fact of the simulation,
    // which keeps no stock at a works: of the ground a works of its kind and size stands on
    // (`footprint`), `STOCK_IN` for what comes in and lies in the open, shared among the goods
    // by the bulk of each; `STOCK_OUT` for what is made; and for the tip up to `TIP`, by how
    // much the process leaves. A yard lies in one piece where there is room for it and in up
    // to four where there is not, in the bays between the buildings before any new ground.
    //
    // The size they are laid out for is the size the works can reach before it must build
    // again (`ENLARGED` times its last stage), or the size its plot was set aside for if that
    // is less. So a works that grows between two stages, which builds nothing, has its yards
    // fuller year by year (`yard` draws as much of them in use as `Now::size` asks).
    let total = stages.last().unwrap().size;
    let (sw, sd) = kind.plot();
    let room = (whole.area() / (GROUND * GROUND * sw * sd)).powf(1.0 / 0.64);
    let laid_for = (total * ENLARGED).min(room.max(total));
    let standard = sw * sd * laid_for.clamp(0.4, 120.0).powf(0.64);
    let mut yards: Vec<Yard> = Vec::new();
    let mut wants: Vec<(Hold, f32, (f32, f32))> = Vec::new();
    // (the tip is given its ground first: the far corner is its own)
    if process.tips() && total >= 1.0 {
        // the corner of what is built so far that is furthest from the gate
        let b = field.taken[reserved.min(built - 1)..built].iter().fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |b, t| (b.0.min(t.0), b.1.max(t.2), b.2.min(t.1), b.3.max(t.3)));
        let far = [(b.0, b.2), (b.1, b.2), (b.0, b.3), (b.1, b.3)].into_iter().fold((gate, -1.0f32), |best, p| { let d = dist(p, gate); if d > best.1 { (p, d) } else { best } }).0;
        // (as big as what the process leaves: a furnace's slag bank, a mill's ash heap)
        wants.push((Hold::Tip, TIP * standard * (process.waste(founded) / 3.0).clamp(0.15, 1.0), far));
    }
    // (then what is made, which is the less ground and says what the works is; then what comes in)
    if matches!(Keep::of(&process.output), Keep::Heap | Keep::Stack | Keep::Logs) {
        wants.push((Hold::Out, STOCK_OUT * standard, dispatch));
    }
    wants.extend(incoming.iter().enumerate().filter(|(_, (g, _))| matches!(Keep::of(g), Keep::Heap | Keep::Stack | Keep::Logs)).map(|(n, (_, share))| (Hold::In(n), STOCK_IN * standard * share.max(0.3), receive)));
    for (x, z, w, d) in strips {
        field.take(x, z, w, d);
    }
    // Each yard in turn is given a piece: the most of what it still wants that finds room. So
    // where ground is short every good has some, and none has it all. A tip is one bank, or
    // two; a stock yard is in four pieces at the most, and no piece is less than a seventh of
    // the yard (scraps of stock about a works are clutter).
    let mut state: Vec<(f32, f32, usize, bool)> = wants.iter().map(|w| (w.1.max(YARD_LEAST), w.1.max(YARD_LEAST), 0, false)).collect();
    while state.iter().any(|s| !s.3) {
        for (k, &(hold, area, by)) in wants.iter().enumerate() {
            let (mut left, mut piece, mut pieces, done) = state[k];
            if done {
                continue;
            }
            let area = area.max(YARD_LEAST);
            let (most, least) = if hold == Hold::Tip { (2, area * 0.3) } else { (4, (area * 0.15).max(YARD_LEAST)) };
            let standing = built_box(&field);
            let mut found: Option<Yard> = None;
            while found.is_none() && piece >= least {
                let mut best: Option<(f32, Yard)> = None;
                // (a tip is a bank along the wall, and is where the works ends; stock lies square, within it)
                let aspects: &[f32] = if hold == Hold::Tip { &[3.0, 0.33, 1.8, 0.55, 1.0] } else { &[1.0, 1.8, 0.55, 3.0, 0.33] };
                for &aspect in aspects {
                    let (w, d) = ((piece * aspect).sqrt(), (piece / aspect).sqrt());
                    // (the tip at the far corner of what stands, where the wall will go round it:
                    // not off across ground the works has no other use for yet)
                    let cost = |x: f32, z: f32| if hold == Hold::Tip { dist((x, z), by) } else { dist((x, z), by) + 0.3 * dist((x, z), gate) + 1.2 * out_of(standing, x, z, w, d) };
                    if let Some((x, z)) = field.place(w, d, 0.003, 1.0, cost) {
                        let c = cost(x, z) + if hold == Hold::Tip { 0.0 } else { (aspect.max(1.0 / aspect) - 1.0) * span * 0.08 };
                        if best.as_ref().map_or(true, |(b, _)| c < *b - 1e-6) {
                            best = Some((c, Yard { hold, x, z, w, d }));
                        }
                    }
                }
                match best {
                    Some((_, y)) => found = Some(y),
                    None => piece *= 0.6,
                }
            }
            state[k] = match found {
                Some(y) => {
                    field.take(y.x, y.z, y.w, y.d);
                    yards.push(y);
                    left -= piece;
                    pieces += 1;
                    // (what is left over of a yard is not worth a piece of its own below a fifth of it)
                    (left, piece.min(left), pieces, pieces >= most || left < area * 0.2)
                }
                None => (left, piece, pieces, true),
            };
        }
    }

    // ---- the ground taken in so far: the box round all of it, with a little room, within the plot
    let office_built = parts.iter().any(|p| p.role == Role::Office);
    let b = field.taken.iter().enumerate().filter(|(k, _)| *k == 0 || *k >= reserved || (office_built && Some(*k) == office_box)).fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |b, (_, t)| (b.0.min(t.0), b.1.max(t.2), b.2.min(t.1), b.3.max(t.3)));
    let m = WALL + 0.006;
    let mut outline = plot.outline.clone();
    for (nx, nz, lim) in [(1.0f32, 0.0f32, b.1 + m), (-1.0, 0.0, -(b.0 - m)), (0.0, -1.0, -(b.2 - m))] {
        outline = clip(&outline, nx, nz, lim);
    }
    let enclosed = enclose(whole, outline);

    Plan { kind, process: process.clone(), plot: whole.clone(), stages, seed, parts, yards, gate: (gx, gw), cart_yard, lane, receive, dispatch, year, siding, wharf, spur, spur_at_start, incoming, squeezed, dropped, enclosed, laid_for }
}

/// The plot a works has taken in of its whole plot: `outline` is a part of
/// the plot's, cut by straight lines; a side of it that lies along a side of
/// the plot faces what that side faces, and a new side is a wall.
fn enclose(plot: &Plot, outline: Vec<(f32, f32)>) -> Plot {
    // drop corners that fall together, and begin at the front's left end
    let mut pts: Vec<(f32, f32)> = Vec::new();
    for p in outline {
        if pts.last().map_or(true, |q: &(f32, f32)| (q.0 - p.0).hypot(q.1 - p.1) > 1e-4) {
            pts.push(p);
        }
    }
    if pts.len() > 1 && (pts[0].0 - pts[pts.len() - 1].0).hypot(pts[0].1 - pts[pts.len() - 1].1) <= 1e-4 {
        pts.pop();
    }
    if pts.len() < 3 {
        return plot.clone();
    }
    let n = pts.len();
    let z1 = pts.iter().map(|p| p.1).fold(f32::MIN, f32::max);
    let Some(first) = (0..n).find(|&i| (pts[i].1 - z1).abs() < 1e-4 && (pts[(i + 1) % n].1 - z1).abs() < 1e-4) else { return plot.clone() };
    pts.rotate_left(first);
    let mut out = Plot::new(pts).with_line(plot.line.clone());
    for i in 0..n {
        let (a, b) = out.side(i);
        for j in 0..plot.outline.len() {
            let ((pa, _), o) = (plot.side(j), plot.outward(j));
            let off = |p: (f32, f32)| ((p.0 - pa.0) * o.0 + (p.1 - pa.1) * o.1).abs();
            if off(a) < 1e-4 && off(b) < 1e-4 {
                out.edges[i] = plot.edges[j];
                out.since[i] = plot.since[j];
            }
        }
    }
    if out.is_sound() { out } else { plot.clone() }
}

// ------------------------------------------------------------------ the works in a year

/// A planned works as it stands in a year.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Now {
    pub year: i32,
    /// its size this year: less than its last stage's when it has shrunk
    pub size: f32,
    /// what the whole works is doing. `GoingUp` is its first year; `Extending` is not used
    /// (a stage built this year is under scaffolding by itself).
    pub state: State,
    /// how hard it is working: its output over its capacity, 0 to 1
    pub busy: f32,
    /// how full its stock yards and stores are of what comes in, and its warehouse or stack yard of what it makes, 0 to 1
    pub stock_in: f32,
    pub stock_out: f32,
    /// how many years it has stood below its greatest size (0 while it stands at it)
    pub shrunk_for: i32,
    pub detail: Detail,
}

impl Now {
    /// A works in full work in `year`, at the size of its plan's last stage, drawn in full.
    pub fn working(plan: &Plan, year: i32) -> Now {
        Now { year, size: plan.stages.last().map_or(1.0, |s| s.size), state: State::Working, busy: 0.8, stock_in: 0.7, stock_out: 0.5, shrunk_for: 0, detail: Detail::Near }
    }
}

/// What a part is doing this year, from what the works is doing: a stage
/// built this year is going up; a stage the works has shrunk out of stands
/// idle, and derelict once it has stood so for eight years.
fn part_state(plan: &Plan, now: &Now, stage: usize) -> State {
    match now.state {
        State::GoingUp | State::Idle | State::Derelict => now.state,
        _ => {
            let st = plan.stages[stage];
            if stage > 0 && st.year == now.year {
                State::GoingUp
            } else if stage > 0 && now.size < plan.stages[stage - 1].size * 1.2 && now.shrunk_for > 0 {
                if now.shrunk_for >= 8 { State::Derelict } else { State::Idle }
            } else {
                State::Working
            }
        }
    }
}

/// How weathered a part built in `built` is in `year`: a step for each forty years, up to two.
fn worn(built: i32, year: i32) -> f32 {
    (((year - built) / 40).clamp(0, 2)) as f32
}

/// Draw a planned works: the site's origin is the origin of the plot's
/// frame, and the plot's front (+z) is turned by the site's yaw. Buildings
/// go to `solid`, smoke to `smoke`, and the plot's ground, its lanes, its
/// siding's ballast and its quay to `flat`.
pub fn site(solid: &mut MeshBuf, smoke: &mut MeshBuf, flat: &mut MeshBuf, site: &Site, plan: &Plan, now: &Now, pal: &Palette) {
    build(solid, smoke, flat, site, plan, now, pal);
}

/// Where a flag on the works would fly from: the top of its highest gable.
/// The same point at every level of detail.
pub fn flag_point(site: &Site, plan: &Plan, now: &Now, pal: &Palette) -> Vec3 {
    let (mut solid, mut smoke, mut flat) = (MeshBuf::default(), MeshBuf::default(), MeshBuf::default());
    build(&mut solid, &mut smoke, &mut flat, site, plan, &Now { detail: Detail::Far, ..*now }, pal).0.unwrap_or(site.origin + Vec3::Y * 0.03)
}

/// The seed of the `n`th part of a works.
fn part_seed(seed: Seed, n: usize) -> Seed {
    Seed::new(hash2(n as i32, (seed.u * 65_521.0) as i32, 211), hash2(n as i32, (seed.v * 65_521.0) as i32, 223))
}

/// Build a planned works. It gives back the top of its highest gable and
/// how many of their own numbers its parts took (the same at every level).
fn build(solid: &mut MeshBuf, smoke: &mut MeshBuf, flat: &mut MeshBuf, site: &Site, plan: &Plan, now: &Now, pal: &Palette) -> (Option<Vec3>, i32) {
    // A yard is level. The works stands at the height of its site's origin, whatever the
    // land does under it: the viewer hands in the height of the highest land under the plot
    // (`scatter::Slots::firm_level`), so the yard is nowhere below the land, and where the
    // land falls away from it the yard is built up behind a retaining wall (`terrace`).
    let land = site;
    let bench = Site::level(site.origin, site.yaw);
    let site = &bench;
    let founded = plan.stages[0].year;
    let water = plan.plot.find(founded, |e| matches!(e, Edge::Water { .. })).is_some();
    let mut peak: Option<Vec3> = None;
    let mut taken = 0;
    let (mut asked_ground, mut asked_wall) = (None, None);
    let note = |c: &Ctx, peak: &mut Option<Vec3>| {
        if let Some(p) = c.peak {
            if peak.map_or(true, |q| p.y > q.y) {
                *peak = Some(p);
            }
        }
    };
    for (n, part) in plan.parts.iter().enumerate() {
        let st = plan.stages[part.stage];
        let state = part_state(plan, now, part.stage);
        let o = part.origin();
        let psite = site.moved(site.at(o.0, o.1), site.yaw + part.turn as f32 * FRAC_PI_2);
        let (lw, ld) = if part.role == Role::Core { footprint(plan.kind, part.size) } else if part.turn % 2 == 0 { (part.w, part.d) } else { (part.d, part.w) };
        let w = Works { kind: plan.kind, size: part.size, founded: st.year, state, water: water && part.stage == 0, rail: plan.siding.is_some(), detail: now.detail, seed: part_seed(plan.seed, n) };
        let mut c = Ctx::part(&mut *solid, &mut *smoke, &mut *flat, &psite, &w, pal, lw * 0.5, ld * 0.5, worn(st.year, now.year));
        match part.role {
            Role::Core => {
                // (what lies loose about a planned works, its stock and its tip, is laid out by the plan)
                c.stock_apart = true;
                if plan.kind == Kind::Other {
                    super::plain::process(&mut c, &plan.process);
                } else {
                    super::raise(&mut c);
                }
                super::scaffolding(&mut c);
                if asked_ground.is_none() {
                    (asked_ground, asked_wall) = (c.asked_ground, c.asked_wall);
                }
            }
            Role::Store(i) => store(&mut c, &plan.incoming[i].0, now.stock_in, false),
            Role::Warehouse => store(&mut c, &plan.process.output, now.stock_out, true),
            Role::Office => office(&mut c),
        }
        c.finish();
        note(&c, &mut peak);
        taken += c.taken();
    }

    // the plot itself: its ground, the wall round it, the gate, the lanes, and what lies in its yards
    let (bx0, bx1, bz0, bz1) = plan.plot.bounds();
    let total = plan.stages.last().unwrap().size;
    let w = Works { kind: plan.kind, size: total, founded, state: now.state, water, rail: plan.siding.is_some(), detail: now.detail, seed: plan.seed };
    let mut c = Ctx::part(&mut *solid, &mut *smoke, &mut *flat, site, &w, pal, (bx1 - bx0) * 0.5, (bz1 - bz0) * 0.5, worn(founded, now.year));
    let base = asked_ground.unwrap_or(if plan.process.heat > 0.0 { *pal.ground.yard_cinders } else { *pal.ground.yard_setts });
    let idle_parts = plan.parts.iter().filter(|p| matches!(part_state(plan, now, p.stage), State::Idle | State::Derelict)).count();
    let weedy = if now.state == State::Derelict { 0.65 } else { 0.3 * idle_parts as f32 / plan.parts.len().max(1) as f32 };
    let ground = lerp(base, *pal.ground.yard_weeds, weedy);
    ground_within(&mut c, &plan.enclosed.outline, ground);
    boundary(&mut c, plan, asked_wall);
    lanes(&mut c, plan, ground);
    for (n, y) in plan.yards.iter().enumerate() {
        yard(&mut c, plan, now, y, n, ground);
    }
    if let Some(side) = plan.siding {
        siding(&mut c, plan, now, side, land);
    }
    if let Some(side) = plan.wharf {
        wharf(&mut c, plan, now, side);
    }
    // weeds round the parts that have fallen out of use
    if now.state != State::Derelict {
        for p in plan.parts.iter().filter(|p| matches!(part_state(plan, now, p.stage), State::Idle | State::Derelict)) {
            c.patch(p.x, p.z, p.w * 0.5, p.d * 0.5, 7, 0, lerp(ground, *pal.ground.yard_weeds, 0.7));
        }
    }
    c.finish();
    taken += c.taken();
    if !land.is_level() {
        let wall = asked_wall.map_or(*pal.stone.yard_wall, |w| w.0);
        terrace(solid, land, site, plan, wall, lerp(ground, *pal.ground.lane_dust, 0.45));
    }
    (peak, taken)
}

/// How far the yard must stand above the land before a retaining wall or a way up to the gate is built.
const SHOWS: f32 = 0.002;
/// How long the way up to the gate of a works that stands above its road is.
const RAMP: f32 = 0.028;

/// What holds a level yard up where the land falls away from it: a retaining
/// wall under the edge of the ground the works has taken in, from the yard's
/// level down to the land, wherever the land lies lower; and where the gate
/// stands above the land before it, a way up to it. Drawn at every level of
/// detail: from far off it is what keeps a works on a hillside from floating.
fn terrace(solid: &mut MeshBuf, land: &Site, bench: &Site, plan: &Plan, wall: Rgba, way: Rgba) {
    let top = bench.origin.y + LIFT;
    let out = &plan.enclosed;
    let foot = Vec3::Y * 0.006;
    for i in 0..out.outline.len() {
        let ((a, b), o) = (out.side(i), out.outward(i));
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        let n = ((len / 0.03).ceil() as usize).max(1);
        let facing = bench.turn() * Vec3::new(o.0, 0.0, o.1);
        let at = |k: usize| { let t = k as f32 / n as f32; land.at(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t) };
        for k in 0..n {
            let (p, q) = (at(k), at(k + 1));
            if bench.origin.y - p.y.min(q.y) < SHOWS {
                continue;
            }
            // (darker at its foot, as every wall is)
            let (tp, tq) = (Vec3::new(p.x, top, p.z), Vec3::new(q.x, top, q.z));
            solid.quad_toward([tp, tq, q - foot, p - foot], [wall, wall, shade(wall, 0.84), shade(wall, 0.84)], facing);
        }
    }
    // the way up to the gate, from the land a little before it
    let ((gx, gw), z1) = (plan.gate, plan.plot.bounds().3);
    let (fa, fb) = (land.at(gx - gw * 0.5, z1 + RAMP), land.at(gx + gw * 0.5, z1 + RAMP));
    if bench.origin.y - fa.y.min(fb.y) >= SHOWS {
        let lift = Vec3::Y * 0.003;
        let (a, b) = (bench.at(gx - gw * 0.5, z1) + Vec3::Y * LIFT, bench.at(gx + gw * 0.5, z1) + Vec3::Y * LIFT);
        solid.quad_toward([a, b, fb + lift, fa + lift], [way; 4], Vec3::Y);
        // its two cheeks, down to the land under the gate posts
        let right = bench.turn() * Vec3::X;
        for (head, toe, x, side) in [(a, fa + lift, gx - gw * 0.5, -1.0f32), (b, fb + lift, gx + gw * 0.5, 1.0)] {
            let under = land.at(x, z1) - foot;
            solid.tri_toward([head, toe - foot, under], [wall, shade(wall, 0.78), shade(wall, 0.78)], right * side);
            solid.tri_toward([head, toe, toe - foot], [wall, wall, shade(wall, 0.78)], right * side);
        }
    }
}

// ------------------------------------------------------------------ the parts the plan adds

/// A store or a warehouse: a range with a cart door and, when it is of more
/// than one floor, a hoist; bales or cases stand before its door as it is full.
fn store(c: &mut Ctx, good: &Good, full: f32, warehouse: bool) {
    let (hw, hd, pal) = (c.hw, c.hd, c.pal);
    let (wall, roof) = (c.masonry(false), c.roofing());
    let apron = 0.007;
    let (len, dep) = (2.0 * hw - 0.003, 2.0 * hd - 0.003 - apron);
    let storeys = if warehouse { 2 + (c.w.size >= 6.0) as usize + (c.w.size >= 30.0) as usize } else { 1 + (c.w.size >= 4.0) as usize };
    let h = c.storeys(storeys);
    let s = c.shed_pitched(0.0, -apron * 0.5, len, dep, h, 0.3, false, wall, roof);
    let door = (0.0085f32).min(len * 0.2);
    let u = -len * 0.5 + len * if warehouse { 0.3 } else { 0.5 };
    c.doors(&s, Face::Front, u - door, u + door, 0.0, 0.0135, false, *pal.timber.door);
    if storeys > 1 {
        c.hoist(&s, Face::Front, len * 0.5 - 0.011, 1);
    }
    if c.stocked() {
        let n = ((len - 2.0 * door - 0.01) / 0.011).floor().max(0.0) as usize;
        let shown = (n as f32 * full.clamp(0.0, 1.0)).round() as usize;
        let col = good_colour(good, pal);
        for i in 0..shown {
            // (the door stays clear)
            let x = -len * 0.5 + 0.007 + i as f32 * 0.011;
            if (x - u).abs() > door + 0.005 {
                c.pile(x, hd - apron * 0.5 - 0.0012, 0.008, 0.0048, 0.0042, false, col);
            }
        }
    }
}

/// The counting house by the gate: a small house of two floors with a door and a chimney.
fn office(c: &mut Ctx) {
    let (hw, hd, pal) = (c.hw, c.hd, c.pal);
    let (wall, roof) = (c.masonry(false), c.roofing());
    let (len, dep) = (2.0 * hw - 0.004, 2.0 * hd - 0.004);
    let s = c.shed_pitched(0.0, 0.0, len, dep, c.storeys(2), 0.5, false, wall, roof);
    c.door(&s, Face::Front, 0.0);
    c.stack(-len * 0.5 + 0.003, 0.0, c.storeys(2) + dep * 0.3 + 0.006, 0.0045, c.tone(*pal.brick.chimney), 0.25);
}

/// The plot's own ground: one fan over its outline, since the yard is level.
fn ground_within(c: &mut Ctx, outline: &[(f32, f32)], col: Rgba) {
    if outline.len() < 3 {
        return;
    }
    let up = Vec3::Y * LIFT;
    let m = outline.iter().fold((0.0, 0.0), |s, p| (s.0 + p.0 / outline.len() as f32, s.1 + p.1 / outline.len() as f32));
    let mid = c.at(m.0, m.1) + up;
    for i in 0..outline.len() {
        let (a, b) = (outline[i], outline[(i + 1) % outline.len()]);
        let (pa, pb) = (c.at(a.0, a.1) + up, c.at(b.0, b.1) + up);
        // whichever way round the outline runs, the ground faces up
        if (pa - mid).cross(pb - mid).y >= 0.0 { c.flat.tri(mid, pa, pb, col) } else { c.flat.tri(mid, pb, pa, col) }
    }
}

/// The part of a convex outline where `n . p <= lim`.
fn clip(poly: &[(f32, f32)], nx: f32, nz: f32, lim: f32) -> Vec<(f32, f32)> {
    let mut out = Vec::with_capacity(poly.len() + 2);
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let (da, db) = (a.0 * nx + a.1 * nz - lim, b.0 * nx + b.1 * nz - lim);
        if da <= 0.0 {
            out.push(a);
        }
        if (da < 0.0 && db > 0.0) || (da > 0.0 && db < 0.0) {
            let t = da / (da - db);
            out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    out
}

/// The wall round the plot, with the gate in its front: none along a siding
/// (the track comes in there) or a wharf, and a way through where a
/// neighbour hands material over.
fn boundary(c: &mut Ctx, plan: &Plan, asked: Option<(Rgba, f32)>) {
    let Some((col, h)) = asked else { return };
    if c.going_up() {
        return;
    }
    let col = c.tone(col);
    let masonry = h >= 0.0065;
    let t = if masonry { 0.0034 } else { 0.0016 };
    let plot = &plan.enclosed;
    let n = plot.outline.len();
    let back = t * 0.5 + 0.0002;
    let (gx, gw) = plan.gate;
    for i in 0..n {
        let len = { let (a, b) = plot.side(i); (b.0 - a.0).hypot(b.1 - a.1) };
        if len < 1e-4 {
            continue;
        }
        let at = |f: f32| plot.along(i, f, back);
        // the gaps in this side, as stretches of it
        let gap: Option<(f32, f32)> = match plot.edge(i, plan.year) {
            // (the track comes in along a siding's side, and a quay is open to the water)
            Edge::Rail if plan.siding.is_some() => Some((0.0, 1.0)),
            Edge::Water { navigable: true } if plan.wharf.is_some() => Some((0.0, 1.0)),
            Edge::Road if i == 0 => {
                let (a, b) = plot.side(0);
                let f = (gx - a.0) / (b.0 - a.0);
                let half = gw * 0.5 / len;
                Some((f - half, f + half))
            }
            Edge::Fed | Edge::Feeds => Some((0.5 - 0.012 / len, 0.5 + 0.012 / len)),
            _ => None,
        };
        let runs: Vec<(f32, f32)> = match gap {
            None => vec![(0.0, 1.0)],
            Some((g0, g1)) => [(0.0, g0.max(0.0)), (g1.min(1.0), 1.0)].into_iter().filter(|r| r.1 - r.0 > 1e-3).collect(),
        };
        for &(f0, f1) in &runs {
            c.wall_run(at(f0), at(f1), h, t, col, masonry);
        }
        if c.far() {
            continue;
        }
        if let Some((g0, g1)) = gap.filter(|_| !matches!(plot.edge(i, plan.year), Edge::Rail | Edge::Water { .. })) {
            // piers either side of a way through; the gate shut across it when the works stands idle
            if c.w.size >= 1.0 && !c.derelict() {
                for f in [g0, g1] {
                    let p = at(f.clamp(0.0, 1.0));
                    c.pier(p.0, p.1, t * 1.9, h * 1.45, col);
                }
            }
            if c.w.state == State::Idle {
                c.wall_run(at(g0), at(g1), h * 0.9, t * 0.5, c.tone(*c.pal.timber.door), false);
            }
        }
        if masonry && c.near() && !c.derelict() {
            let p = at(0.0);
            c.pier(p.0, p.1, t * 1.5, h * 1.15, col);
        }
    }
}

/// The ways about the works: from the gate over the cart yard, along the
/// lane, and to every door; where heavy stuff is moved in a works of some
/// size, a tramway from where it comes in to the process buildings.
fn lanes(c: &mut Ctx, plan: &Plan, ground: Rgba) {
    let pal = c.pal;
    let trodden = lerp(ground, *pal.ground.lane_dust, 0.45);
    let up = Vec3::Y * (LIFT + 0.0012);
    let ribbon = |c: &mut Ctx, a: (f32, f32), b: (f32, f32), w: f32, col: Rgba, lift: f32| {
        c.flat.ribbon(&[c.at(a.0, a.1) + up + Vec3::Y * lift, c.at(b.0, b.1) + up + Vec3::Y * lift], w, col);
    };
    let (cx, cz, cw, cd) = plan.cart_yard;
    let (_, _, _, bz1) = plan.plot.bounds();
    ribbon(c, (cx, bz1), (cx, cz - cd * 0.5), cw, trodden, 0.0);
    let hub = plan.lane.map(|(a, b, w)| {
        // (as far as the furthest door that is reached from it)
        let end = plan.parts.iter().map(|p| p.door().1).fold(a.1, f32::min).max(b.1);
        ribbon(c, a, (b.0, end), w, trodden, 0.0);
        (a, (b.0, end))
    });
    // to every door, from the nearest point of the lane (or from the cart yard)
    for p in &plan.parts {
        let d = p.door();
        let from = match hub {
            Some((a, b)) => (a.0, d.1.clamp(b.1.min(a.1), a.1.max(b.1))),
            None => (cx, cz - cd * 0.5),
        };
        ribbon(c, from, d, 0.0055, trodden, 0.0);
    }
    // the tramway, in a works of some size where heavy stuff is moved: from where it comes in
    // to the lane, along it, and off it to each stage's process buildings
    let heavy = plan.incoming.iter().any(|(g, _)| Keep::of(g) == Keep::Heap);
    let total = plan.stages.last().unwrap().size;
    if let (true, Some((a, b))) = (heavy && total >= 3.0 && plan.stages.last().unwrap().year >= 1800, hub) {
        let rails = *pal.rail.rails;
        let pair = |c: &mut Ctx, from: (f32, f32), to: (f32, f32)| {
            let (dx, dz) = (to.0 - from.0, to.1 - from.1);
            let len = dx.hypot(dz);
            if len < 0.004 {
                return;
            }
            let o = (-dz / len * 0.0022, dx / len * 0.0022);
            for s in [-1.0f32, 1.0] {
                ribbon(c, (from.0 + o.0 * s, from.1 + o.1 * s), (to.0 + o.0 * s, to.1 + o.1 * s), 0.0011, rails, 0.0012);
            }
        };
        let on_lane = |p: (f32, f32)| (a.0, p.1.clamp(b.1.min(a.1), a.1.max(b.1)));
        let (mut lo, mut hi) = (on_lane(plan.receive).1, on_lane(plan.receive).1);
        pair(c, plan.receive, on_lane(plan.receive));
        for p in plan.parts.iter().filter(|p| p.role == Role::Core) {
            let d = p.door();
            pair(c, on_lane(d), d);
            (lo, hi) = (lo.min(on_lane(d).1), hi.max(on_lane(d).1));
        }
        // and to each yard of what comes in loose, where the way from the lane is clear of buildings
        for y in &plan.yards {
            if !matches!(y.hold, Hold::In(i) if Keep::of(&plan.incoming[i].0) == Keep::Heap) {
                continue;
            }
            let from = on_lane((y.x, y.z));
            let to = (from.0.clamp(y.x - y.w * 0.5, y.x + y.w * 0.5), y.z);
            let (x0, x1) = (from.0.min(to.0), from.0.max(to.0));
            if !plan.parts.iter().any(|p| x0 < p.x + p.w * 0.5 && x1 > p.x - p.w * 0.5 && (p.z - from.1).abs() < p.d * 0.5 + 0.003) {
                pair(c, from, to);
                (lo, hi) = (lo.min(from.1), hi.max(from.1));
            }
        }
        pair(c, (a.0, lo), (a.0, hi));
    }
}

/// A rectangle of ground within the plot, a little above the yard's own: the floor of a stock yard.
fn floor(c: &mut Ctx, (x0, z0, x1, z1): (f32, f32, f32, f32), col: Rgba) {
    let up = Vec3::Y * (LIFT + 0.0006);
    c.flat.quad(c.at(x0, z1) + up, c.at(x1, z1) + up, c.at(x1, z0) + up, c.at(x0, z0) + up, col);
}

/// What lies in an open yard, as full as the works' stock says, on ground
/// stained by it: ore-red, coal-black, the grey of slag. (The floor is the
/// same at every level and in every state, so an empty yard still reads as
/// the place where the ore lay.)
fn yard(c: &mut Ctx, plan: &Plan, now: &Now, y: &Yard, n: usize, ground: Rgba) {
    let pal = c.pal;
    let (good, full) = match y.hold {
        Hold::In(i) => (Some(&plan.incoming[i].0), now.stock_in),
        Hold::Out => (Some(&plan.process.output), now.stock_out),
        Hold::Tip => (None, 1.0),
    };
    // (as much of the yard in use as its size now asks, of the size the yard was laid out for)
    let in_use = (now.size / plan.laid_for.max(0.4)).clamp(0.15, 1.0).powf(0.64);
    let full = if c.w.state == State::GoingUp || c.derelict() { 0.0 } else { full.clamp(0.0, 1.0) * in_use };
    let tip = if plan.process.inputs.iter().any(|(g, _)| g.name == "iron_ore") { *pal.ground.slag } else { *pal.ground.spoil_coal };
    let stain = good.map_or(tip, |g| good_colour(g, pal));
    floor(c, (y.x - y.w * 0.5, y.z - y.d * 0.5, y.x + y.w * 0.5, y.z + y.d * 0.5), lerp(ground, stain, 0.3));
    match (good.map(Keep::of), good) {
        (None, _) => {
            // the tip is a bank of heaps along its ground, one beside the next, and grows with
            // the years the works has stood: more of them, and higher
            let age = ((now.year - plan.stages[0].year) as f32 / 40.0).clamp(0.2, 1.0);
            let (long, short, along_x) = if y.w >= y.d { (y.w, y.d, true) } else { (y.d, y.w, false) };
            let r = short * 0.5;
            let count = ((long / (r * 1.5)).round() as usize).clamp(1, 6);
            let shown = ((count as f32 * age).ceil() as usize).clamp(1, count);
            let col = c.tone(tip);
            for k in 0..shown {
                // (from end to end of its ground, and no heap over the end)
                let t = if count == 1 { 0.0 } else { (long - 2.0 * r) * (k as f32 / (count - 1) as f32 - 0.5) };
                let (x, z) = if along_x { (y.x + t, y.z) } else { (y.x, y.z + t) };
                // (the newest of them is the one being tipped on)
                let k_last = if k + 1 == shown && shown > 1 { 0.7 } else { 1.0 };
                c.heap(x, z, r * (0.8 + 0.2 * age) * k_last, r * (0.42 + 0.3 * age) * k_last, col);
            }
        }
        (Some(Keep::Heap), Some(g)) => {
            let col = good_colour(g, pal);
            // heaps shoulder to shoulder, in ridges: as many as the yard is full
            let r = (y.w.min(y.d) * 0.5).min(0.026 * c.g);
            let (cols, rows) = (((y.w / (1.7 * r)).floor() as usize).max(1), ((y.d / (1.7 * r)).floor() as usize).max(1));
            let shown = ((cols * rows) as f32 * full).ceil() as usize;
            let reach = |n: usize, k: usize, span: f32| if n == 1 { 0.0 } else { (span - 2.0 * r) * (k as f32 / (n - 1) as f32 - 0.5) };
            for k in 0..(cols * rows).min(shown) {
                let (i, j) = (k % cols, k / cols);
                let (x, z) = (y.x + reach(cols, i, y.w), y.z + reach(rows, j, y.d));
                // the last heap is the one being drawn on
                let k_last = if k + 1 == shown { 0.45 + 0.55 * (full * (cols * rows) as f32).fract().max(0.3) } else { 1.0 };
                c.heap(x, z, r * 0.97 * k_last, r * 0.62 * k_last, col);
            }
        }
        (Some(keep), Some(g)) => {
            // stacks in rows (timber in long piles): never more than seven to a row or six
            // rows, so a great yard has greater stacks and not hundreds of them
            let col = good_colour(g, pal);
            let logs = keep == Keep::Logs;
            let (pw, pd, h) = if logs { (0.03f32, 0.008f32, 0.006f32) } else { (0.012, 0.0078, 0.0056) };
            let (pw, pd) = (pw.max(y.w / 7.0 - 0.003), pd.max(y.d / 6.0 - 0.003));
            let (cols, rows) = (((y.w / (pw + 0.003)).floor() as usize).max(1), ((y.d / (pd + 0.003)).floor() as usize).max(1));
            let shown = ((cols * rows) as f32 * full).ceil() as usize;
            let (cw, rd) = (y.w / cols as f32, y.d / rows as f32);
            let high = |k: usize| h * (pd / 0.0078).min(1.6) * (0.65 + 0.35 * hash2(k as i32, n as i32, (plan.seed.u * 9973.0) as i32));
            if c.near() {
                for k in 0..(cols * rows).min(shown) {
                    let (i, j) = (k % cols, k / cols);
                    let (x, z) = (y.x - y.w * 0.5 + cw * (i as f32 + 0.5), y.z - y.d * 0.5 + rd * (j as f32 + 0.5));
                    c.pile(x, z, pw.min(cw - 0.002), pd.min(rd - 0.002), high(k), false, col);
                }
            } else {
                // from the play distance a row of stacks is one long stack
                for j in 0..rows {
                    let in_row = shown.saturating_sub(j * cols).min(cols);
                    if in_row > 0 {
                        let len = cw * in_row as f32 - (cw - pw.min(cw - 0.002));
                        c.pile(y.x - y.w * 0.5 + cw * in_row as f32 * 0.5, y.z - y.d * 0.5 + rd * (j as f32 + 0.5), len, pd.min(rd - 0.002), high(j * cols) * 0.9, false, col);
                    }
                }
            }
        }
        _ => {}
    }
}

/// A wagon on a siding: at `at` in the plot, turned to run along (dx, dz), loaded with `load`.
fn wagon(c: &mut Ctx, at: (f32, f32), dir: (f32, f32), body: Rgba, load: Option<Rgba>) {
    if c.far() {
        return;
    }
    let pal = c.pal;
    let yaw = c.site.yaw + dir.0.atan2(dir.1) - FRAC_PI_2;
    let p = c.at(at.0, at.1) + Vec3::Y * (LIFT + 0.0034);
    let (len, wide, high) = (0.018, 0.0085, 0.0062);
    c.solid.block(p, len, wide, high, yaw, body, Some(load.unwrap_or(shade(body, 0.6))), 0.0);
    if c.near() {
        // its wheels, and its load heaped above its sides
        let q = glam::Quat::from_rotation_y(yaw);
        for (dx, dz) in [(-0.3f32, -0.5f32), (0.3, -0.5), (-0.3, 0.5), (0.3, 0.5)] {
            let w = c.at(at.0, at.1) + Vec3::Y * LIFT + q * Vec3::new(dx * len, 0.0, dz * (wide + 0.0006));
            c.solid.block(w, 0.0038, 0.0008, 0.0038, yaw, *pal.vehicles.cart_wheels, None, 0.0);
        }
        if let Some(load) = load {
            c.solid.frustum_turned(p + Vec3::Y * high, wide * 0.62, wide * 0.2, 0.0022, 4, std::f32::consts::FRAC_PI_4 - yaw, 0.0, load, Some(shade(load, 1.08)));
        }
    }
}

/// The siding along a side of the plot: its ballast and rails, a loading
/// bank beside it, a buffer stop at its end, and wagons standing on it, more
/// of them as the works is bigger and busier. (The wagons are a STAND-IN for
/// the traffic the simulation would record: so many as the works' size and
/// busyness suggest.)
fn siding(c: &mut Ctx, plan: &Plan, now: &Now, (side, f0, f1): (usize, f32, f32), land: &Site) {
    let pal = c.pal;
    let plot = &plan.plot;
    // from the end the spur leaves by to the end with the stop
    let (a, b) = siding_track(plot, side, f0, f1, plan.spur_at_start);
    let len = (b.0 - a.0).hypot(b.1 - a.1);
    if len < 0.03 {
        return;
    }
    let dir = ((b.0 - a.0) / len, (b.1 - a.1) / len);
    let pt = |f: f32, off: f32| (a.0 + dir.0 * len * f - dir.1 * off, a.1 + dir.1 * len * f + dir.0 * off);
    let line = |c: &Ctx, off: f32, lift: f32| -> Vec<Vec3> { [0.0f32, 1.0].into_iter().map(|f| { let p = pt(f, off); c.at(p.0, p.1) + Vec3::Y * (LIFT + lift) }).collect() };
    let ballast = line(c, 0.0, 0.0012);
    c.flat.ribbon(&ballast, 0.013, *pal.rail.ballast);
    for off in [-0.0034f32, 0.0034] {
        let rail = line(c, off, 0.0024);
        c.flat.ribbon(&rail, 0.0013, *pal.rail.rails);
    }
    // the spur that leads it out to the line: the same track, in its curve, running from
    // the yard's level to the land's where it meets the line (the viewer gives a works a
    // siding only where those two are near enough for a track to run between them)
    if plan.spur.len() >= 2 {
        let last = plan.spur[plan.spur.len() - 1];
        let fall = land.at(last.0, last.1).y - c.site.origin.y;
        let steps = (plan.spur.len() - 1) as f32;
        let at = |c: &Ctx, i: usize, p: (f32, f32), lift: f32| c.at(p.0, p.1) + Vec3::Y * (LIFT + lift + fall * i as f32 / steps);
        let bed: Vec<Vec3> = plan.spur.iter().enumerate().map(|(i, &p)| at(c, i, p, 0.0012)).collect();
        c.flat.ribbon(&bed, 0.013, *pal.rail.ballast);
        for off in [-0.0034f32, 0.0034] {
            let rail: Vec<Vec3> = (0..plan.spur.len())
                .map(|i| {
                    let (p, q) = (plan.spur[i.saturating_sub(1)], plan.spur[(i + 1).min(plan.spur.len() - 1)]);
                    let l = (q.0 - p.0).hypot(q.1 - p.1).max(1e-6);
                    at(c, i, (plan.spur[i].0 - (q.1 - p.1) / l * off, plan.spur[i].1 + (q.0 - p.0) / l * off), 0.0024)
                })
                .collect();
            c.flat.ribbon(&rail, 0.0013, *pal.rail.rails);
        }
    }
    if c.going_up() {
        return;
    }
    // which way is into the yard from the track
    let o = plot.outward(side);
    let inward = if (-dir.1) * o.0 + dir.0 * o.1 < 0.0 { 1.0 } else { -1.0 };
    // the loading bank, on the yard's side of the track
    let stone = c.tone(*pal.rail.stone);
    let (f0, f1) = (0.34, 0.66);
    let mid = pt((f0 + f1) * 0.5, inward * 0.0104);
    let yaw = c.site.yaw + dir.0.atan2(dir.1) - FRAC_PI_2;
    let p = c.at(mid.0, mid.1);
    c.solid.block(p, len * (f1 - f0), 0.0052, LIFT + 0.0034, yaw, shade(stone, 0.9), Some(stone), 0.01);
    // the stop at the far end: two posts and a beam across the rails
    if !c.far() {
        let timber = c.tone(*pal.timber.headframe);
        for off in [-0.0036f32, 0.0036] {
            let q = pt(1.0, off);
            let foot = c.at(q.0, q.1);
            c.solid.beam(foot, foot + Vec3::Y * (LIFT + 0.0062), 0.0016, timber);
        }
        let (qa, qb) = (pt(1.0, -0.0046), pt(1.0, 0.0046));
        c.solid.beam(c.at(qa.0, qa.1) + Vec3::Y * (LIFT + 0.0052), c.at(qb.0, qb.1) + Vec3::Y * (LIFT + 0.0052), 0.0018, *pal.rail.buffer);
    }
    // the wagons
    if !c.busy() {
        return;
    }
    let room = ((len * 0.88) / 0.0205).floor() as usize;
    let wanted = (1.3 * (now.size.max(0.4) * now.busy.clamp(0.1, 1.0)).powf(0.45)).ceil() as usize;
    let count = wanted.clamp(1, room.max(1));
    let load_in = plan.incoming.iter().max_by(|a, b| a.1.total_cmp(&b.1)).map(|(g, _)| good_colour(g, pal));
    let load_out = (Keep::of(&plan.process.output) != Keep::Nothing).then(|| good_colour(&plan.process.output, pal));
    for k in 0..count {
        // those that brought stuff in stand toward the way in, those to take it away toward the way out
        let coming = k % 2 == 0;
        let slot = k / 2;
        let f = if coming { 0.07 + slot as f32 * 0.0205 / len } else { 0.9 - slot as f32 * 0.0205 / len };
        let body = *pal.vehicles.wagons[(k + (plan.seed.u * 7.0) as usize) % pal.vehicles.wagons.len()];
        wagon(c, pt(f, 0.0), dir, body, if coming { load_in } else { load_out });
    }
}

/// The wharf on a navigable side: a stone quay, a crane, and what waits on the quay.
fn wharf(c: &mut Ctx, plan: &Plan, now: &Now, (side, f0, f1): (usize, f32, f32)) {
    let pal = c.pal;
    let plot = &plan.plot;
    let (a, b) = (plot.along(side, f0 + 0.01 * (f1 - f0), WHARF * 0.5), plot.along(side, f1 - 0.01 * (f1 - f0), WHARF * 0.5));
    let len = (b.0 - a.0).hypot(b.1 - a.1);
    if len < 0.03 {
        return;
    }
    let dir = ((b.0 - a.0) / len, (b.1 - a.1) / len);
    let pt = |f: f32, off: f32| (a.0 + dir.0 * len * f - dir.1 * off, a.1 + dir.1 * len * f + dir.0 * off);
    let line: Vec<Vec3> = [0.0f32, 1.0].into_iter().map(|f| { let p = pt(f, 0.0); c.at(p.0, p.1) + Vec3::Y * (LIFT + 0.0012) }).collect();
    c.flat.ribbon(&line, WHARF - 0.002, lerp(*pal.stone.yard_wall, *pal.ground.yard_setts, 0.5));
    if c.going_up() || c.derelict() {
        return;
    }
    let crane = pt(0.5, 0.0);
    c.sheer_legs(crane.0, crane.1, (0.026 * c.g).min(WHARF * 1.1));
    if c.busy() && Keep::of(&plan.process.output) != Keep::Nothing {
        let col = good_colour(&plan.process.output, pal);
        let shown = (4.0 * now.stock_out.clamp(0.0, 1.0)).ceil() as usize;
        for k in 0..shown {
            let p = pt(0.62 + k as f32 * 0.016 / len, 0.0);
            c.pile(p.0, p.1, 0.011, 0.0075, 0.005, false, col);
        }
    }
}

// ------------------------------------------------------------------ for the lab and the tests

/// A likely record for a works founded in `founded` that stands at `size` in
/// `by`: founded at half the standard size (or at `size`, if that is
/// smaller) and grown evenly, by ratio, year after year. For the lab's
/// sheets and the tests; the game hands the planner the record itself.
pub fn history(founded: i32, by: i32, size: f32) -> Vec<(i32, f32)> {
    let first = size.min(0.5).max(0.4);
    let years = (by - founded).max(1);
    (0..=years).map(|i| (founded + i, first * (size.max(0.4) / first).powf(i as f32 / years as f32))).collect()
}

/// A works of a kind as it stands in `year`, founded in `founded` and grown
/// to `size`, on a plot: its plan and the year, for the lab's sheets.
pub fn specimen(kind: Kind, founded: i32, year: i32, size: f32, plot: &Plot, seed: Seed, pal: &Palette) -> (Plan, Now) {
    let record = history(founded, year, size);
    let plan = plan(kind, &Process::of(kind), &stages(&record), plot, year, seed, pal);
    // (at the size it has reached, which is more than its last stage's when it has grown since)
    let now = Now { size: record.last().map_or(size, |h| h.1), state: if year == founded { State::GoingUp } else { State::Working }, ..Now::working(&plan, year) };
    (plan, now)
}

#[cfg(test)]
#[path = "site_tests.rs"]
mod tests;
