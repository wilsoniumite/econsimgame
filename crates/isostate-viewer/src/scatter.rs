//! Life on the land: trees, town houses, farmsteads, mines and the works
//! that firms build.
//!
//! The economy only knows shares and densities per 2 km cell. Here every
//! possible tree or house has a fixed spot and a fixed threshold, and it is
//! drawn once its cell's share or density passes that threshold. Growth is
//! therefore smooth and stable: nothing reshuffles from one year to the next,
//! towns thicken in place, and woods thin out tree by tree.

use bevy::math::Vec3;

use crate::meshkit::{hex, lerp, shade, Builder};
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

pub struct Slots {
    pub trees: Vec<Slot>,
    pub houses: Vec<Slot>,
    pub farms: Vec<Slot>,
    pub mines: Vec<Slot>,
    /// stable building plot for every firm
    pub firm_pos: Vec<(f32, f32, f32)>,
    /// how many plots each cell has handed out
    pub taken: std::collections::HashMap<usize, usize>,
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
                if world::height_at(g, x, z) < 0.05 {
                    continue;
                }
                let (wx, wy) = world::warp(x, z);
                let wc = (wy.floor().clamp(0.0, r.h as f32 - 1.0) as usize) * r.w + wx.floor().clamp(0.0, r.w as f32 - 1.0) as usize;
                out.push(Slot {
                    x,
                    z,
                    wcell: wc as u32,
                    sub: world::sub_uniform(g, x, z) * 255.0,
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
    let mut slots = Slots {
        trees: grid_slots(r, g, 6, 11, 0.9),
        houses: grid_slots(r, g, 7, 12, 0.7),
        farms: grid_slots(r, g, 1, 13, 0.6),
        mines: grid_slots(r, g, 2, 14, 0.7),
        firm_pos: Vec::new(),
        taken: Default::default(),
    };
    // firms get plots in their cell, first come first served
    place_new_firms(r, g, &mut slots);
    slots
}

fn ground(g: &Ground, x: f32, z: f32) -> Vec3 {
    Vec3::new(x, world::height_at(g, x, z), z)
}

// ------------------------------------------------------------------ trees

fn tree(b: &mut Builder, p: Vec3, s: f32, conifer: bool, v: f32, seed: f32) {
    let trunk = hex(0x7a5536);
    if conifer {
        let greens = [0x2f6a49, 0x3a7a51, 0x2c5f43];
        let g = hex(greens[(v * 3.0) as usize % 3]);
        b.frustum(p, 0.007 * s, 0.005 * s, 0.025 * s, 4, trunk, None);
        b.frustum(p + Vec3::Y * 0.02 * s, 0.048 * s, 0.0, 0.085 * s, 6, g, None);
        b.frustum(p + Vec3::Y * 0.062 * s, 0.035 * s, 0.0, 0.065 * s, 6, shade(g, 1.08), None);
    } else {
        let greens = [0x5e9a45, 0x6aa84f, 0x4f8a3d, 0x79b25a, 0x8ab85c];
        let g = hex(greens[(v * 5.0) as usize % 5]);
        b.frustum(p, 0.008 * s, 0.006 * s, 0.032 * s, 4, trunk, None);
        b.ball(p + Vec3::Y * 0.062 * s, 0.06 * s, 0.05 * s, 6, 3, g, seed);
    }
}

// ------------------------------------------------------------------ houses

const WALLS: [u32; 6] = [0xf6deb4, 0xfaf0dc, 0xf4d38f, 0xf0bfa3, 0xdfe2bf, 0xf6deb4];
const ROOFS: [u32; 5] = [0xc6604a, 0x5b6b86, 0xc6604a, 0x6e8f5c, 0xb5523f];

fn house(b: &mut Builder, p: Vec3, w: f32, d: f32, storeys: f32, yaw: f32, wall: [f32; 4], roof: [f32; 4]) {
    let h = 0.024 * storeys;
    b.block(p, w, d, h, yaw, wall, None, 0.03);
    b.gable(p, w, d, h, d * 0.55, yaw, 0.004, roof, wall);
}

fn chimney(b: &mut Builder, p: Vec3, h: f32, r: f32) {
    b.frustum(p, r, r * 0.8, h, 6, hex(0xa9553f), Some(hex(0x3a3330)));
}

/// A chimney with a few puffs of smoke drifting downwind.
fn smoking_chimney(b: &mut Builder, smoke: &mut Builder, p: Vec3, h: f32, r: f32, seed: f32) {
    chimney(b, p, h, r);
    let puff = hex(0xeeece8);
    for i in 0..3 {
        let t = i as f32;
        let c = p + Vec3::new(0.012 + 0.022 * t + 0.004 * seed.sin(), h + 0.018 + 0.016 * t, 0.006 * (seed * 3.0 + t).sin());
        smoke.ball(c, 0.011 + 0.006 * t, 0.009 + 0.005 * t, 6, 3, shade(puff, 1.0 - 0.04 * t), seed + t);
    }
}

fn church(b: &mut Builder, p: Vec3, yaw: f32, big: bool) {
    let stone = hex(0xddd5c4);
    let slate = hex(0x55617a);
    let k = if big { 1.35 } else { 1.0 };
    let q = bevy::math::Quat::from_rotation_y(yaw);
    b.block(p, 0.085 * k, 0.04 * k, 0.034 * k, yaw, stone, None, 0.03);
    b.gable(p, 0.085 * k, 0.04 * k, 0.034 * k, 0.026 * k, yaw, 0.003, slate, stone);
    let tower = p + q * Vec3::new(-0.05 * k, 0.0, 0.0);
    b.block(tower, 0.028 * k, 0.028 * k, 0.075 * k, yaw, stone, Some(stone), 0.03);
    b.frustum(tower + Vec3::Y * 0.075 * k, 0.021 * k, 0.0, 0.07 * k, 4, slate, None);
}

fn station(b: &mut Builder, p: Vec3, yaw: f32) {
    let q = bevy::math::Quat::from_rotation_y(yaw);
    b.block(p, 0.09, 0.035, 0.03, yaw, hex(0xf1e3c6), None, 0.03);
    b.gable(p, 0.09, 0.035, 0.03, 0.018, yaw, 0.004, hex(0xb5523f), hex(0xf1e3c6));
    // platform canopy
    let c = p + q * Vec3::new(0.0, 0.0, 0.035);
    b.block(c, 0.12, 0.03, 0.002, yaw, hex(0x6b5a4e), None, 0.0);
    b.block(c + Vec3::Y * 0.022, 0.13, 0.034, 0.004, yaw, hex(0x7a8a6a), Some(hex(0x7a8a6a)), 0.0);
    // clock turret
    b.block(p + q * Vec3::new(0.0, 0.0, 0.0), 0.016, 0.016, 0.06, yaw, hex(0xf1e3c6), Some(hex(0xf1e3c6)), 0.0);
    b.frustum(p + Vec3::Y * 0.06, 0.013, 0.0, 0.022, 4, hex(0xb5523f), None);
}

fn lighthouse(b: &mut Builder, p: Vec3) {
    let white = hex(0xf7f5ef);
    let red = hex(0xc94b3c);
    for i in 0..4 {
        let y = i as f32 * 0.024;
        let r0 = 0.016 - 0.0012 * i as f32;
        b.frustum(p + Vec3::Y * y, r0, r0 - 0.0012, 0.024, 8, if i % 2 == 0 { white } else { red }, None);
    }
    b.frustum(p + Vec3::Y * 0.096, 0.012, 0.012, 0.014, 8, hex(0xffe58a), None);
    b.frustum(p + Vec3::Y * 0.11, 0.014, 0.0, 0.014, 8, red, None);
}

// ------------------------------------------------------------------ works

fn works(b: &mut Builder, smoke: &mut Builder, r: &Replay, f: usize, k: usize, pos: (f32, f32, f32), g: &Ground) {
    let firm = &r.header.firms[f];
    let m = &r.header.methods[firm.method];
    let kk = replay::firm_k(r, f, k);
    let ratio = (kk / m.k0.max(1e-6)).max(0.05);
    let tier = (ratio.log2() + 1.5).floor().clamp(1.0, 4.0);
    let s = 0.8 + 0.2 * tier;
    let seed = f as f32 * 0.37;
    let p = ground(g, pos.0, pos.1);
    let yaw = pos.2;
    let q = bevy::math::Quat::from_rotation_y(yaw);
    let at = |x: f32, z: f32| {
        let o = q * Vec3::new(x, 0.0, z);
        ground(g, pos.0 + o.x, pos.1 + o.z)
    };
    let brick = hex(0xb5634c);
    let dark = hex(0x8f4b3b);
    let stone = hex(0xbdb4a5);
    let slate = hex(0x4a5468);
    let timber = hex(0x9a6b45);
    let name = m.name.as_str();
    match name {
        "services" => {}
        "hand_loom" => house(b, p, 0.06, 0.04, 1.0, yaw, hex(0xf0e2c4), hex(0xd6b263)),
        "smithy" | "millwright" => {
            b.block(p, 0.06 * s, 0.045 * s, 0.026, yaw, if name == "smithy" { stone } else { timber }, None, 0.03);
            b.gable(p, 0.06 * s, 0.045 * s, 0.026, 0.022, yaw, 0.004, slate, stone);
            smoking_chimney(b, smoke, at(0.02 * s, 0.0), 0.06, 0.006, seed);
        }
        "charcoal_burning" => {
            for i in 0..(tier as usize + 1) {
                let o = at((i as f32 - tier * 0.5) * 0.035, (i % 2) as f32 * 0.02);
                b.frustum(o, 0.022, 0.008, 0.02, 7, hex(0x4a3f36), Some(hex(0x3a322b)));
            }
        }
        "bloomery" | "charcoal_furnace" | "coke_furnace" => {
            let n = if name == "coke_furnace" { tier as usize } else { 1 };
            let fh = if name == "bloomery" { 0.04 } else { 0.075 };
            for i in 0..n {
                b.frustum(at(-0.03 + i as f32 * 0.045, -0.015), 0.022 * s, 0.014 * s, fh, 4, stone, Some(hex(0x3a3330)));
            }
            b.block(at(0.02, 0.025), 0.07 * s, 0.04, 0.025, yaw, if name == "coke_furnace" { brick } else { timber }, None, 0.03);
            b.gable(at(0.02, 0.025), 0.07 * s, 0.04, 0.025, 0.018, yaw, 0.003, slate, brick);
            if name == "coke_furnace" {
                for i in 0..tier as usize {
                    smoking_chimney(b, smoke, at(0.06, -0.03 + i as f32 * 0.02), 0.13 + 0.02 * tier, 0.008, seed + i as f32);
                }
                b.block(at(-0.02, 0.06), 0.11 * s, 0.022, 0.014, yaw, dark, Some(hex(0x5a4a42)), 0.02);
            }
        }
        "textile_mill" => {
            let len = 0.08 + 0.025 * tier;
            b.block(p, len, 0.04, 0.075, yaw, brick, None, 0.03);
            // rows of windows
            for row in 0..3 {
                let y = 0.016 + row as f32 * 0.022;
                let wp = p + q * Vec3::new(0.0, y, 0.0201);
                b.block(wp - Vec3::Y * 0.0, len * 0.9, 0.001, 0.008, yaw, hex(0xf3e2b0), None, 0.0);
            }
            b.gable(p, len, 0.04, 0.075, 0.016, yaw, 0.003, slate, brick);
            smoking_chimney(b, smoke, at(len * 0.5 + 0.015, 0.0), 0.16 + 0.02 * tier, 0.009, seed);
        }
        "engine_works" | "forge_rails" | "rolling_mill" | "steel_rolling" | "bessemer" => {
            let len = 0.09 + 0.03 * tier;
            b.block(p, len, 0.06, 0.04, yaw, if name == "engine_works" { brick } else { dark }, None, 0.03);
            b.gable(p, len, 0.06, 0.04, 0.025, yaw, 0.004, slate, brick);
            for i in 0..(tier as usize).max(1) {
                smoking_chimney(b, smoke, at(-len * 0.4 + i as f32 * 0.03, -0.045), 0.12 + 0.02 * tier, 0.008, seed + i as f32);
            }
            if name == "bessemer" {
                for i in 0..2 {
                    b.ball(at(len * 0.5 + 0.02, -0.02 + i as f32 * 0.035) + Vec3::Y * 0.02, 0.014, 0.018, 7, 3, hex(0x5c5a58), i as f32);
                }
            }
        }
        "crucible_steel" => {
            b.block(p, 0.07, 0.045, 0.03, yaw, stone, None, 0.03);
            b.gable(p, 0.07, 0.045, 0.03, 0.02, yaw, 0.003, slate, stone);
            b.frustum(at(0.05, 0.0), 0.016, 0.006, 0.07, 6, brick, Some(hex(0x3a3330)));
        }
        "brickworks" => {
            for i in 0..(tier as usize + 1).min(4) {
                let o = at(-0.03 + i as f32 * 0.03, -0.02);
                b.frustum(o, 0.016, 0.019, 0.02, 7, brick, None);
                b.frustum(o + Vec3::Y * 0.02, 0.019, 0.006, 0.03, 7, brick, Some(hex(0x3a3330)));
            }
            b.block(at(0.0, 0.03), 0.1, 0.025, 0.012, yaw, timber, Some(hex(0x6e5a48)), 0.02);
        }
        _ => {
            b.block(p, 0.06 * s, 0.045, 0.03, yaw, brick, None, 0.03);
            b.gable(p, 0.06 * s, 0.045, 0.03, 0.02, yaw, 0.003, slate, brick);
        }
    }
}

// ------------------------------------------------------------------ the year's meshes

pub struct YearMeshes {
    pub trees: Builder,
    pub houses: Builder,
    pub works: Builder,
    pub smoke: Builder,
}

pub fn build_year(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize) -> YearMeshes {
    let mut trees = Builder::default();
    let mut houses = Builder::default();
    let mut works_b = Builder::default();
    let mut smoke = Builder::default();

    // firm plots in use this year, so houses keep clear of them
    let mut plots: std::collections::HashMap<usize, Vec<(f32, f32)>> = Default::default();
    for (f, firm) in r.header.firms.iter().enumerate() {
        if replay::firm_k(r, f, k) <= 0.0 || r.header.methods[firm.method].name == "services" {
            continue;
        }
        let p = slots.firm_pos[f];
        plots.entry(firm.cell).or_default().push((p.0, p.1));
        works(&mut works_b, &mut smoke, r, f, k, p, g);
    }
    // each town: a church on its market square, a station once the railway arrives,
    // and a lighthouse if it is a harbour
    let nt = r.n_towns.min(r.header.towns.len());
    for t in 0..nt {
        if r.town_active[k * r.n_towns + t] == 0 {
            continue;
        }
        let node = r.header.towns[t].node;
        let (x, z) = cen[node];
        let yaw = (world::hash2(t as i32, 1, 5) * 4.0).floor() * std::f32::consts::FRAC_PI_2;
        let big = r.town_urban[k * r.n_towns + t] > 12000.0;
        church(&mut houses, ground(g, x, z), yaw, big);
        plots.entry(node).or_default().push((x, z));
        if r.trans[k * r.n + node] & 4 != 0 {
            let q = bevy::math::Quat::from_rotation_y(yaw);
            let o = q * Vec3::new(0.0, 0.0, 0.2);
            station(&mut houses, ground(g, x + o.x, z + o.z), yaw);
            plots.entry(node).or_default().push((x + o.x, z + o.z));
        }
        let (cx, cy) = ((node % r.w) as isize, (node / r.w) as isize);
        let mut sea: Option<(f32, f32)> = None;
        for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            let (nx, ny) = (cx + dx, cy + dy);
            if nx >= 0 && ny >= 0 && (nx as usize) < r.w && (ny as usize) < r.h && r.terrain[ny as usize * r.w + nx as usize] == 0 {
                sea = Some((dx as f32, dy as f32));
                break;
            }
        }
        if let Some((dx, dy)) = sea {
            // walk toward the sea until the ground is nearly at the waterline
            let (mut lx, mut lz) = (x, z);
            for _ in 0..40 {
                let (nx, nz) = (lx + dx * 0.025, lz + dy * 0.025);
                if world::height_at(g, nx, nz) < 0.03 {
                    break;
                }
                lx = nx;
                lz = nz;
            }
            lighthouse(&mut houses, ground(g, lx, lz));
        }
    }
    let near_plot = |x: f32, z: f32| -> bool {
        let c = (z.floor() as usize).min(r.h - 1) * r.w + (x.floor() as usize).min(r.w - 1);
        plots.get(&c).map(|v| v.iter().any(|&(px, pz)| (px - x).abs() < 0.085 && (pz - z).abs() < 0.085)).unwrap_or(false)
    };

    for s in &slots.trees {
        let cls = world::pick_class(r, k, s.wcell as usize, s.sub);
        let terr = r.terrain[s.wcell as usize];
        let (grow, scale) = match cls {
            world::WOOD => (true, 0.85 + 0.4 * s.v),
            world::FARM | world::PASTURE => (s.u < 0.05, 0.75 + 0.3 * s.v),
            world::HEATH => {
                // gorse and broom: low bushes, and the odd tree
                if s.u < 0.16 && !near_plot(s.x, s.z) {
                    let p = ground(g, s.x, s.z);
                    let col = if s.v < 0.5 { hex(0x9aa548) } else { hex(0x6b7f3e) };
                    trees.ball(p + Vec3::Y * 0.012, 0.022 + 0.01 * s.v, 0.014, 5, 2, col, s.yaw);
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
        tree(&mut trees, p, scale, conifer, s.u, s.yaw);
    }

    for s in &slots.houses {
        let c = s.wcell as usize;
        let cls = world::pick_class(r, k, c, s.sub);
        if cls == world::WORKS && s.u < 0.5 && !near_plot(s.x, s.z) {
            // yards, sheds and warehouses around the works
            let p = ground(g, s.x, s.z);
            let yaw = (s.yaw / std::f32::consts::FRAC_PI_2).round() * std::f32::consts::FRAC_PI_2;
            let wall = hex([0xb5634c, 0x9b5a48, 0xa9a196, 0x8f4b3b][(s.v * 4.0) as usize % 4]);
            let (w, d, h) = (0.06 + 0.04 * s.v, 0.045 + 0.015 * s.u, 0.026 + 0.018 * s.v);
            houses.block(p, w, d, h, yaw, wall, None, 0.03);
            houses.gable(p, w, d, h, d * 0.35, yaw, 0.003, hex(0x4a5468), wall);
            if s.u < 0.12 {
                let cp = p + bevy::math::Quat::from_rotation_y(yaw) * Vec3::new(w * 0.35, 0.0, 0.0);
                smoking_chimney(&mut houses, &mut smoke, cp, h + 0.07, 0.006, s.yaw);
            }
            continue;
        }
        if cls != world::URBAN || near_plot(s.x, s.z) {
            continue;
        }
        let off = k * r.n + c;
        let dense = if r.use_[off] == 10 { r.inten[off] as f32 / 255.0 } else { 0.45 };
        if s.u > 0.25 + 0.7 * dense {
            continue;
        }
        let p = ground(g, s.x, s.z);
        let yaw = (s.yaw / std::f32::consts::FRAC_PI_2).round() * std::f32::consts::FRAC_PI_2 + (s.v - 0.5) * 0.12;
        let storeys = 1.0 + ((dense > 0.6 && s.v > 0.4) as u8 as f32) + ((dense > 0.85 && s.v > 0.75) as u8 as f32);
        let wall = hex(WALLS[(s.v * 6.0) as usize % 6]);
        let roof = hex(ROOFS[(s.u * 13.0) as usize % 5]);
        let (w, d) = (0.045 + 0.02 * s.v, 0.035 + 0.008 * s.u);
        house(&mut houses, p, w, d, storeys, yaw, wall, roof);
        if s.u < 0.2 {
            let cp = p + bevy::math::Quat::from_rotation_y(yaw) * Vec3::new(w * 0.3, 0.0, 0.0);
            if s.v > 0.8 {
                smoking_chimney(&mut houses, &mut smoke, cp, 0.024 * storeys + 0.03, 0.004, s.yaw);
            } else {
                chimney(&mut houses, cp, 0.024 * storeys + 0.03, 0.004);
            }
        }
    }

    for s in &slots.farms {
        let c = s.wcell as usize;
        let off = k * r.n + c;
        let main = r.mix_a[off];
        if !(main == world::FARM || main == world::PASTURE) || s.u > 0.6 || near_plot(s.x, s.z) {
            continue;
        }
        let cls = world::pick_class(r, k, c, s.sub);
        if !(cls == world::FARM || cls == world::PASTURE) {
            continue;
        }
        let p = ground(g, s.x, s.z);
        let q = bevy::math::Quat::from_rotation_y(s.yaw);
        let wall = hex(WALLS[(s.v * 5.0) as usize % 5]);
        house(&mut houses, p, 0.05, 0.036, 1.0 + (s.v > 0.6) as u8 as f32, s.yaw, wall, hex(if s.v < 0.5 { 0xd6b263 } else { 0xc6604a }));
        let barn = p + q * Vec3::new(0.0, 0.0, 0.06);
        let bp = ground(g, barn.x, barn.z);
        house(&mut houses, bp, 0.07, 0.04, 1.2, s.yaw, hex(0xa8553e), hex(0x6b5a4e));
        if main == world::FARM {
            let hay = p + q * Vec3::new(0.055, 0.0, -0.02);
            let hp = ground(g, hay.x, hay.z);
            houses_haystack(&mut houses, hp);
        }
    }

    for s in &slots.mines {
        let c = s.wcell as usize;
        let cls = world::pick_class(r, k, c, s.sub);
        let heap = match cls {
            world::COALMINE => hex(0x3c3936),
            world::IRONMINE => hex(0x8e5a44),
            world::QUARRY => hex(0xb9b2a5),
            world::CLAYPIT => hex(0xb07a55),
            _ => continue,
        };
        if s.u > 0.7 || near_plot(s.x, s.z) {
            continue;
        }
        let p = ground(g, s.x, s.z);
        let q = bevy::math::Quat::from_rotation_y(s.yaw);
        works_b.frustum(p + q * Vec3::new(0.04, 0.0, 0.0), 0.05 + 0.02 * s.v, 0.006, 0.045 + 0.02 * s.v, 7, heap, None);
        if cls == world::COALMINE || cls == world::IRONMINE {
            // a headframe: two legs, a stay and a wheel housing
            let wood = hex(0x6b4a32);
            let top = p + Vec3::Y * 0.085;
            works_b.beam(p + q * Vec3::new(-0.015, 0.0, -0.012), top, 0.006, wood);
            works_b.beam(p + q * Vec3::new(-0.015, 0.0, 0.012), top, 0.006, wood);
            works_b.beam(p + q * Vec3::new(0.03, 0.0, 0.0), top, 0.006, wood);
            works_b.block(top - Vec3::Y * 0.006, 0.012, 0.012, 0.012, s.yaw, hex(0x3d3a38), Some(hex(0x3d3a38)), 0.0);
            works_b.block(p + q * Vec3::new(-0.04, 0.0, 0.0), 0.03, 0.025, 0.02, s.yaw, hex(0xb5634c), Some(hex(0x4a5468)), 0.02);
        }
    }

    YearMeshes { trees, houses, works: works_b, smoke }
}

fn houses_haystack(b: &mut Builder, p: Vec3) {
    let hay = hex(0xe0c068);
    b.frustum(p, 0.012, 0.012, 0.012, 7, hay, None);
    b.frustum(p + Vec3::Y * 0.012, 0.012, 0.0, 0.012, 7, lerp(hay, hex(0xc9a24e), 0.5), None);
}
