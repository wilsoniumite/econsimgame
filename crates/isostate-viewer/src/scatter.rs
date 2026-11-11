//! Life on the land: trees, town houses, farmsteads, mines and the works
//! that firms build.
//!
//! The economy only knows shares and densities per 2 km cell. Here every
//! possible tree or house has a fixed spot and a fixed threshold, and it is
//! drawn once its cell's share or density passes that threshold. Growth is
//! therefore smooth and stable: nothing reshuffles from one year to the next,
//! towns thicken in place, and woods thin out tree by tree.
//!
//! This file decides where things stand and which of them stand this year.
//! What each looks like is a generator's business: those are plain functions
//! in the isostate-gen crate, called from here with the spot, the slot's
//! fixed numbers and the look's palette.

use bevy::math::Vec3;

pub mod towns;

use isostate_gen::bridge::{road_bridge, RoadBridge};
use isostate_gen::farm::{farmstead, Farmstead};
use isostate_gen::trees::{bush, hedge_tree, tree, Species, Tree};
use isostate_gen::works::{pit, scaffold, works, yard_shed, Kind, Pit, PitKind, Works, YardShed};
use isostate_gen::{Seed, Site};

use crate::look::Look;
use crate::meshkit::{shade, Builder};
use crate::replay::{self, Replay};
use crate::world::{self, Ground};

#[derive(Clone, Copy)]
pub struct Slot {
    pub x: f32,
    pub z: f32,
    /// the warped cell whose mix decides what grows here
    pub wcell: u32,
    /// patch-noise value at the spot, 0..255, so the slot agrees with the painted ground
    pub sub: f32,
    /// fixed thresholds and variety
    pub u: f32,
    pub v: f32,
    pub yaw: f32,
}

#[derive(Clone)]
pub struct Slots {
    pub trees: Vec<Slot>,
    pub houses: Vec<Slot>,
    pub farms: Vec<Slot>,
    pub mines: Vec<Slot>,
    /// stable building plot for every firm
    pub firm_pos: Vec<(f32, f32, f32)>,
    /// how many plots each cell has handed out
    pub taken: std::collections::HashMap<usize, usize>,
    /// spots on the hedges between fields, for hedgerow trees
    pub hedge: Vec<Slot>,
    /// the year the fields of each hedge spot and each farm spot are enclosed
    /// (0 if they always were), as there are no hedges round open fields and the
    /// farms stand in the village until enclosure
    pub hedge_open: Vec<i32>,
    pub farm_open: Vec<i32>,
}

fn grid_slots(r: &Replay, g: &Ground, n: usize, seed: i32, jitter: f32) -> Vec<Slot> {
    let mut out = Vec::new();
    for c in 0..r.n {
        if r.terrain[c] == 0 {
            continue;
        }
        let (cx, cy) = ((c % r.w) as f32, (c / r.w) as f32);
        for j in 0..n {
            for i in 0..n {
                let id = (c * n * n + j * n + i) as i32;
                let x = cx + (i as f32 + 0.5 + jitter * (world::hash2(id, 1, seed) - 0.5)) / n as f32;
                let z = cy + (j as f32 + 0.5 + jitter * (world::hash2(id, 2, seed) - 0.5)) / n as f32;
                // nothing on the beach or in a river
                if world::height_at(g, x, z) < 0.05 || world::water_dist(g, x, z) < 0.025 {
                    continue;
                }
                let (wc, sub) = world::class_key(g, x, z);
                out.push(Slot {
                    x,
                    z,
                    wcell: wc as u32,
                    sub,
                    u: world::hash2(id, 3, seed),
                    v: world::hash2(id, 4, seed),
                    yaw: world::hash2(id, 5, seed) * std::f32::consts::TAU,
                });
            }
        }
    }
    out
}

/// The building plots of a cell, in the order firms take them: a 3 x 3 grid,
/// less the market square of a town and anything in the sea or on the beach.
fn cell_plots(r: &Replay, g: &Ground, cell: usize) -> Vec<(f32, f32)> {
    let is_town = r.header.towns.iter().any(|t| t.node == cell);
    let (cx, cy) = ((cell % r.w) as f32, (cell / r.w) as f32);
    let mut plots: Vec<(f32, f32)> = Vec::new();
    for j in 0..3 {
        for i in 0..3 {
            if is_town && i == 1 && j == 1 {
                continue; // leave the market square free
            }
            let (px, pz) = ((i as f32 + 0.5) / 3.0, (j as f32 + 0.5) / 3.0);
            if world::height_at(g, cx + px, cy + pz) >= 0.07 {
                plots.push((px, pz)); // never build in the sea or on the beach
            }
        }
    }
    if plots.is_empty() {
        // a coastal sliver: use its highest point
        let mut best = (0.5, 0.5, f32::MIN);
        for j in 0..7 {
            for i in 0..7 {
                let (px, pz) = ((i as f32 + 0.5) / 7.0, (j as f32 + 0.5) / 7.0);
                let h = world::height_at(g, cx + px, cy + pz);
                if h > best.2 {
                    best = (px, pz, h);
                }
            }
        }
        plots.push((best.0, best.1));
    }
    plots.sort_by(|a, b| {
        let ka = world::hash2((a.0 * 7.0) as i32, (a.1 * 7.0) as i32, cell as i32);
        let kb = world::hash2((b.0 * 7.0) as i32, (b.1 * 7.0) as i32, cell as i32);
        ka.partial_cmp(&kb).unwrap()
    });
    plots
}

/// Where firm `f` stands: the `n`th firm to build in its cell takes the `n`th plot,
/// and later rounds squeeze in beside the first.
fn firm_plot(r: &Replay, g: &Ground, plots: &[(f32, f32)], f: usize, n: usize) -> (f32, f32, f32) {
    let cell = r.header.firms[f].cell;
    let (cx, cy) = ((cell % r.w) as f32, (cell / r.w) as f32);
    let (px, pz) = plots[n % plots.len()];
    let round = (n / plots.len()) as f32;
    let mut jx = (world::hash2(f as i32, 7, 3) - 0.5) * 0.12 + round * 0.07;
    let mut jz = (world::hash2(f as i32, 8, 3) - 0.5) * 0.12;
    if world::height_at(g, cx + px + jx, cy + pz + jz) < 0.07 {
        jx = 0.0;
        jz = 0.0;
    }
    let yaw = (world::hash2(f as i32, 9, 3) * 4.0).floor() * std::f32::consts::FRAC_PI_2 + (world::hash2(f as i32, 10, 3) - 0.5) * 0.3;
    (cx + px + jx, cy + pz + jz, yaw)
}

/// Plots for firms the slots have not seen yet (a live game adds firms as it goes).
/// They must arrive in the order they were founded.
pub fn place_new_firms(r: &Replay, g: &Ground, slots: &mut Slots) {
    let start = slots.firm_pos.len();
    let mut order: Vec<usize> = (start..r.header.firms.len()).collect();
    order.sort_by_key(|&f| (r.header.firms[f].born, f));
    slots.firm_pos.resize(r.header.firms.len(), (0.0, 0.0, 0.0));
    for f in order {
        let cell = r.header.firms[f].cell;
        let n = slots.taken.entry(cell).or_insert(0);
        let plots = cell_plots(r, g, cell);
        slots.firm_pos[f] = firm_plot(r, g, &plots, f, *n);
        *n += 1;
    }
}

pub fn make_slots(r: &Replay, g: &Ground) -> Slots {
    world::timed("slots", || make_slots_inner(r, g))
}

fn make_slots_inner(r: &Replay, g: &Ground) -> Slots {
    let mut slots = Slots {
        trees: grid_slots(r, g, 6, 11, 0.9),
        houses: grid_slots(r, g, 7, 12, 0.7),
        farms: Vec::new(),
        mines: grid_slots(r, g, 2, 14, 0.7),
        firm_pos: Vec::new(),
        taken: Default::default(),
        hedge: Vec::new(),
        hedge_open: Vec::new(),
        farm_open: Vec::new(),
    };
    (slots.hedge, slots.hedge_open) = hedge_slots(r, g).into_iter().unzip();
    slots.farms = grid_slots(r, g, 2, 13, 0.5).into_iter().map(|s| farm_spot(g, s)).collect();
    slots.farm_open = slots.farms.iter().map(|s| g.open_until[world::own_cell(g, s.x, s.z)]).collect();
    // firms get plots in their cell, first come first served
    place_new_firms(r, g, &mut slots);
    crate::plan::maybe_write(r, g, &slots);
    crate::plan::maybe_bench(r, g, &slots, &crate::network::centres(r));
    slots
}

/// Spots on the hedges: a grid of candidates in every land cell, each moved
/// onto the nearest edge of its field. The spot keeps the class of the field it
/// started in, so a tree only grows on the hedges of farm and pasture.
fn hedge_slots(r: &Replay, g: &Ground) -> Vec<(Slot, i32)> {
    use rayon::prelude::*;
    let n = 7usize;
    (0..r.n)
        .into_par_iter()
        .flat_map_iter(|c| {
            let mut out = Vec::new();
            if r.terrain[c] == 0 {
                return out;
            }
            let (cx, cy) = ((c % r.w) as f32, (c / r.w) as f32);
            for j in 0..n {
                for i in 0..n {
                    let id = (c * n * n + j * n + i) as i32;
                    let x = cx + (i as f32 + 0.5 + 0.9 * (world::hash2(id, 1, 17) - 0.5)) / n as f32;
                    let z = cy + (j as f32 + 0.5 + 0.9 * (world::hash2(id, 2, 17) - 0.5)) / n as f32;
                    if world::height_at(g, x, z) < 0.05 {
                        continue;
                    }
                    let f = world::field_at(g, x, z);
                    if f.edge > 0.08 {
                        continue;
                    }
                    // step onto the edge, twice, as the gentle warp of the fields
                    // makes the first step fall a little short
                    let (mut hx, mut hz) = (x + f.toward.x * f.edge, z + f.toward.y * f.edge);
                    let f2 = world::field_at(g, hx - f.toward.x * 0.004, hz - f.toward.y * 0.004);
                    if f2.cut.id == f.cut.id && f2.block.i == f.block.i && f2.block.j == f.block.j {
                        hx += f2.toward.x * (f2.edge - 0.004).max(0.0);
                        hz += f2.toward.y * (f2.edge - 0.004).max(0.0);
                    }
                    if world::height_at(g, hx, hz) < 0.05 || world::water_dist(g, hx, hz) < 0.03 {
                        continue;
                    }
                    // trees stand thicker in some districts than others
                    let u = world::hash2(id, 3, 17);
                    if u > 0.05 + 0.12 * (0.5 + 0.5 * world::vnoise(hx * 0.45, hz * 0.45, 181)) {
                        continue;
                    }
                    let (wc, sub) = world::class_key(g, x, z);
                    let slot = Slot {
                        x: hx,
                        z: hz,
                        wcell: wc as u32,
                        sub,
                        u,
                        v: world::hash2(id, 4, 17),
                        yaw: world::hash2(id, 5, 17) * std::f32::consts::TAU,
                    };
                    out.push((slot, g.open_until[world::own_cell(g, x, z)]));
                }
            }
            out
        })
        .collect()
}

/// A farmstead stands just inside a hedge of its field, square to it.
fn farm_spot(g: &Ground, s: Slot) -> Slot {
    let f = world::field_at(g, s.x, s.z);
    let mut out = s;
    if f.edge > 0.045 {
        out.x += f.toward.x * (f.edge - 0.04);
        out.z += f.toward.y * (f.edge - 0.04);
    }
    // the house's back to the hedge, its front to the yard
    out.yaw = (-f.toward.y).atan2(f.toward.x) + std::f32::consts::FRAC_PI_2;
    out
}

fn ground(g: &Ground, x: f32, z: f32) -> Vec3 {
    Vec3::new(x, world::height_at(g, x, z), z)
}

// ------------------------------------------------------------------ works

/// The size class a works is drawn at: 1 for a small one, up to 4.
fn tier(r: &Replay, f: usize, k: usize) -> u8 {
    let m = &r.header.methods[r.header.firms[f].method];
    isostate_gen::works::tier(replay::firm_k(r, f, k) / m.k0.max(1e-6))
}

// ------------------------------------------------------------------ the year's meshes

pub struct YearMeshes {
    pub trees: Builder,
    pub houses: Builder,
    pub works: Builder,
    pub smoke: Builder,
    /// farm lanes, flat on the ground
    pub lanes: Builder,
}

pub fn build_year(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, lk: &Look) -> YearMeshes {
    world::timed("year scatter", || build_year_inner(r, g, slots, cen, k, lk))
}

fn build_year_inner(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, lk: &Look) -> YearMeshes {
    let mut trees = Builder::default();
    let mut houses = Builder::default();
    let mut works_b = Builder::default();
    let mut smoke = Builder::default();

    // the generators ask this for the ground under the parts of a thing that stand apart
    let height = |x: f32, z: f32| world::height_at(g, x, z);

    // firm plots in use this year, so houses keep clear of them
    let mut plots: std::collections::HashMap<usize, Vec<(f32, f32)>> = Default::default();
    for (f, firm) in r.header.firms.iter().enumerate() {
        if replay::firm_k(r, f, k) <= 0.0 {
            continue;
        }
        // services have no works
        let Some(kind) = Kind::of_method(&r.header.methods[firm.method].name) else { continue };
        let p = slots.firm_pos[f];
        plots.entry(firm.cell).or_default().push((p.0, p.1));
        let site = Site::on(ground(g, p.0, p.1), p.2, &height);
        works(&mut works_b, &mut smoke, &site, &Works { kind, tier: tier(r, f, k), seed: f as f32 * 0.37 }, lk);
        // scaffolding the first year a works stands, and the year it grows a size
        let opened = k > 0 && replay::firm_k(r, f, k - 1) <= 0.0 && firm.born > r.y0;
        let grew = k > 0 && replay::firm_k(r, f, k - 1) > 0.0 && tier(r, f, k) > tier(r, f, k - 1);
        if opened || grew {
            scaffold(&mut works_b, &site, tier(r, f, k), lk);
        }
    }
    // the railway's embankments, viaducts, cuttings, portals and bridges, and a
    // bridge wherever a road crosses a river
    works_b.append(crate::rails::structures(&crate::rails::year(r, g, cen, k), g, lk));
    for (c, m, lv) in crate::network::road_segments(r, k) {
        let (a, b) = (bevy::math::Vec2::new(cen[c].0, cen[c].1), bevy::math::Vec2::new(cen[m].0, cen[m].1));
        for (at, _, half, level) in crate::rivers::crossings(&g.rivers, &g.water, a, b) {
            road_bridge(&mut works_b, &RoadBridge { at, along: (b - a).normalize_or(bevy::math::Vec2::X), half, level, main: lv == 3 }, &height, lk);
        }
        causeway(&mut works_b, lk, g, a, b, lv == 3);
    }
    towns::landmarks(r, g, cen, k, lk, &mut houses, &mut plots);
    let near_plot = |x: f32, z: f32| -> bool {
        let c = (z.floor() as usize).min(r.h - 1) * r.w + (x.floor() as usize).min(r.w - 1);
        plots.get(&c).map(|v| v.iter().any(|&(px, pz)| (px - x).abs() < 0.085 && (pz - z).abs() < 0.085)).unwrap_or(false)
    };
    // the town houses are placed afresh each year, so they also keep out of the rivers
    let near_plot_or_water = |x: f32, z: f32| -> bool { near_plot(x, z) || world::water_dist(g, x, z) < 0.03 };

    for s in &slots.trees {
        let cls = world::pick_class(r, k, s.wcell as usize, s.sub);
        let terr = r.terrain[s.wcell as usize];
        let (grow, scale) = match cls {
            world::WOOD => (true, 0.85 + 0.4 * s.v),
            // the odd tree out in a field; most stand in the hedges
            world::FARM | world::PASTURE => (s.u < 0.012, 0.75 + 0.3 * s.v),
            world::HEATH => {
                // gorse and broom: low bushes, and the odd tree
                if s.u < 0.16 && !near_plot(s.x, s.z) {
                    bush(&mut trees, &Site::level(ground(g, s.x, s.z), 0.0), s.v, s.yaw, lk);
                }
                (s.u < if terr >= 3 { 0.02 } else { 0.035 }, 0.6 + 0.3 * s.v)
            }
            _ => (false, 1.0),
        };
        if !grow || near_plot(s.x, s.z) {
            continue;
        }
        let p = ground(g, s.x, s.z);
        let conifer = terr >= 2 && s.v > 0.35 || s.v > 0.88;
        let species = if conifer { Species::Conifer } else { Species::Broadleaf };
        tree(&mut trees, &Site::level(p, 0.0), &Tree { species, size: scale, tint: s.u, turn: s.yaw }, lk);
    }

    // yards, sheds and warehouses around the works
    for s in &slots.houses {
        let c = s.wcell as usize;
        if world::pick_class(r, k, c, s.sub) != world::WORKS || s.u >= 0.5 || near_plot(s.x, s.z) {
            continue;
        }
        let p = ground(g, s.x, s.z);
        let yaw = (s.yaw / std::f32::consts::FRAC_PI_2).round() * std::f32::consts::FRAC_PI_2;
        yard_shed(&mut houses, &mut smoke, &Site::level(p, yaw), &YardShed { seed: Seed::new(s.u, s.v), smoke_seed: s.yaw }, lk);
    }

    towns::houses(r, g, slots, cen, k, lk, &near_plot_or_water, &mut houses, &mut smoke);

    // hedgerow trees, once the fields are enclosed, and never in a road
    let year = r.y0 + k as i32;
    let roads = RoadIndex::new(r, cen, k);
    for (s, &open) in slots.hedge.iter().zip(&slots.hedge_open) {
        if year < open {
            continue;
        }
        let cls = world::pick_class(r, k, s.wcell as usize, s.sub);
        if !(cls == world::FARM || cls == world::PASTURE) || near_plot(s.x, s.z) || roads.nearest(s.x, s.z, 0.03).is_some() {
            continue;
        }
        let p = ground(g, s.x, s.z);
        hedge_tree(&mut trees, &Site::level(p, 0.0), 0.55 + 0.3 * s.v, s.u * 7.3, s.yaw, lk);
    }

    // farmsteads among the fields, each with a lane to the nearest road
    let mut lanes = Builder::default();
    for (s, &open) in slots.farms.iter().zip(&slots.farm_open) {
        let c = s.wcell as usize;
        let off = k * r.n + c;
        let main = r.mix_a[off];
        if year < open || !(main == world::FARM || main == world::PASTURE) || s.u > 0.32 || near_plot(s.x, s.z) {
            continue;
        }
        let cls = world::pick_class(r, k, c, s.sub);
        if !(cls == world::FARM || cls == world::PASTURE) || roads.nearest(s.x, s.z, 0.075).is_some() {
            continue;
        }
        // stone and slate in the hills
        let upland = r.terrain[c] >= 2;
        let site = Site::on(ground(g, s.x, s.z), s.yaw, &height);
        farmstead(&mut houses, &mut trees, &site, &Farmstead { arable: main == world::FARM, upland, seed: Seed::new(s.u, s.v) }, lk);
        let q = bevy::math::Quat::from_rotation_y(s.yaw);
        let gate = Vec3::new(s.x, 0.0, s.z) + q * Vec3::new(0.0, 0.0, -0.05);
        if let Some(to) = roads.nearest(gate.x, gate.z, 0.55) {
            lane(&mut lanes, lk, g, (gate.x, gate.z), to, s.v);
        }
    }

    for s in &slots.mines {
        let c = s.wcell as usize;
        let cls = world::pick_class(r, k, c, s.sub);
        let kind = match cls {
            world::COALMINE => PitKind::Coal,
            world::IRONMINE => PitKind::Ironstone,
            world::QUARRY => PitKind::Stone,
            world::CLAYPIT => PitKind::Clay,
            _ => continue,
        };
        if s.u > 0.7 || near_plot(s.x, s.z) {
            continue;
        }
        pit(&mut works_b, &Site::level(ground(g, s.x, s.z), s.yaw), &Pit { kind, v: s.v }, lk);
    }

    YearMeshes { trees, houses, works: works_b, smoke, lanes }
}

/// A low causeway of stone wherever a road runs over water (the drowned mouth
/// of a river, a creek), so the road does not lie on the sea.
fn causeway(b: &mut Builder, lk: &Look, g: &Ground, a: bevy::math::Vec2, c: bevy::math::Vec2, main: bool) {
    let len = a.distance(c);
    let n = ((len / 0.02).ceil() as usize).max(2);
    let wet = |t: f32| {
        let p = a + (c - a) * t;
        world::height_at(g, p.x, p.y) < 0.004
    };
    let stone = *lk.bridges.causeway;
    let width = if main { 0.046 } else { 0.034 };
    let dir = (c - a).normalize_or(bevy::math::Vec2::X);
    let side = Vec3::new(-dir.y, 0.0, dir.x) * (width * 0.5);
    let mut i = 0;
    while i < n {
        if !wet(i as f32 / n as f32) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < n && wet(j as f32 / n as f32) {
            j += 1;
        }
        // from one dry end to the other, a little past each
        let (t0, t1) = (((i as f32 - 1.0) / n as f32).max(0.0), ((j as f32 + 0.5) / n as f32).min(1.0));
        let (p0, p1) = (a + (c - a) * t0, a + (c - a) * t1);
        let top = 0.016;
        let (q0, q1) = (Vec3::new(p0.x, top, p0.y), Vec3::new(p1.x, top, p1.y));
        let foot = Vec3::Y * (top + 0.03);
        b.quad(q0 + side, q1 + side, q1 - side, q0 - side, shade(stone, 1.06));
        b.quad(q0 + side - foot, q1 + side - foot, q1 + side, q0 + side, stone);
        b.quad(q1 - side - foot, q0 - side - foot, q0 - side, q1 - side, shade(stone, 0.85));
        i = j;
    }
}

/// The year's road segments, bucketed by cell, for finding the nearest road.
struct RoadIndex {
    w: usize,
    h: usize,
    cells: std::collections::HashMap<usize, Vec<((f32, f32), (f32, f32))>>,
}

impl RoadIndex {
    fn new(r: &Replay, cen: &[(f32, f32)], k: usize) -> RoadIndex {
        let mut cells: std::collections::HashMap<usize, Vec<((f32, f32), (f32, f32))>> = Default::default();
        for (c, m, _) in crate::network::road_segments(r, k) {
            let seg = (cen[c], cen[m]);
            cells.entry(c).or_default().push(seg);
            cells.entry(m).or_default().push(seg);
        }
        RoadIndex { w: r.w, h: r.h, cells }
    }

    /// The nearest point on a road within `reach`, if any.
    fn nearest(&self, x: f32, z: f32, reach: f32) -> Option<(f32, f32)> {
        let (cx, cy) = (x.floor() as isize, z.floor() as isize);
        let mut best: Option<(f32, (f32, f32))> = None;
        for dy in -1..=1isize {
            for dx in -1..=1isize {
                let (xx, yy) = (cx + dx, cy + dy);
                if xx < 0 || yy < 0 || xx as usize >= self.w || yy as usize >= self.h {
                    continue;
                }
                let Some(v) = self.cells.get(&(yy as usize * self.w + xx as usize)) else { continue };
                for &(a, b) in v {
                    let (abx, abz) = (b.0 - a.0, b.1 - a.1);
                    let t = (((x - a.0) * abx + (z - a.1) * abz) / (abx * abx + abz * abz).max(1e-9)).clamp(0.0, 1.0);
                    let (px, pz) = (a.0 + abx * t, a.1 + abz * t);
                    let d = ((px - x).powi(2) + (pz - z).powi(2)).sqrt();
                    if d < reach && best.map_or(true, |b| d < b.0) {
                        best = Some((d, (px, pz)));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }
}

/// A farm lane: a pale track from the yard gate to the road, bending a little.
fn lane(b: &mut Builder, lk: &Look, g: &Ground, from: (f32, f32), to: (f32, f32), v: f32) {
    let (dx, dz) = (to.0 - from.0, to.1 - from.1);
    let len = (dx * dx + dz * dz).sqrt();
    if len < 0.03 {
        return;
    }
    // a lane does not ford a river
    if !crate::rivers::crossings(&g.rivers, &g.water, bevy::math::Vec2::new(from.0, from.1), bevy::math::Vec2::new(to.0, to.1)).is_empty() {
        return;
    }
    let bend = (v - 0.5) * 0.3 * len;
    let (mx, mz) = (from.0 + dx * 0.5 - dz / len * bend, from.1 + dz * 0.5 + dx / len * bend);
    let n = ((len / 0.03).ceil() as usize).max(2);
    let pts: Vec<Vec3> = (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let (x, z) = (u * u * from.0 + 2.0 * u * t * mx + t * t * to.0, u * u * from.1 + 2.0 * u * t * mz + t * t * to.1);
            Vec3::new(x, world::height_at(g, x, z).max(0.0) + 0.006, z)
        })
        .collect();
    b.ribbon(&pts, 0.011, *lk.ground.farm_lane);
}
