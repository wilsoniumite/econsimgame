//! Land (see `landform.rs` and docs/rules/land.md): the island the economy sees.
//!
//! The land is made on a fine grid; the economy runs on cells of 2 km. This file
//! reduces the one to the other, filling the `World` the port's generator fills
//! (land and sea, heights, terrain, rivers and where they run, the resource
//! fields, the first town sites), and keeps the fine fields and the water beside
//! it in a `Landform` for whoever draws the island or builds on its water.
//!
//! The resource fields come from the land instead of from free noise: corn land
//! from alluvium, slope, rain and height; grass where it is wet, cool or steep;
//! woodland from rain and height; stone where hard rock is laid bare; clay in the
//! vales; coal where the seams of a coalfield come near the surface; ore in the
//! mountain belt.

use crate::landform::{
    Design, Grid, LandParams, ROCK_ALLUVIUM, ROCK_BAND, ROCK_BASEMENT, ROCK_CAP, ROCK_COAL, ROCK_SOFT, ROCK_VALE, Relief, build_relief, draw_design, fbm01,
    smoothstep,
};
use crate::landwater::{Lake, Oxbow, River, WET_MARSH, WET_PLAIN, Water, make_water};
use crate::ndimage::{self, Mode};
use crate::rng::{Rng, land_stream, random_n};
use crate::world::{HILLS, LOWLAND, MOUNTAINS, NEIGH8, SEA, World};

/// Where a river system meets the sea.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outlet {
    /// The cell of its mouth.
    pub cell: usize,
    /// The land that drains through it, square km, and its mean flow, cubic metres a second.
    pub area_km2: f32,
    pub flow: f32,
}

/// Why a cell is a place a town would grow (`Landform::site_kind`).
pub const SITE_NONE: u8 = 0;
/// The head of an estuary: the lowest place a river can be bridged, and as far as a ship can come.
pub const SITE_ESTUARY: u8 = 1;
pub const SITE_CONFLUENCE: u8 = 2;
pub const SITE_LAKE_OUTLET: u8 = 3;
/// A gap through the hills.
pub const SITE_GAP: u8 = 4;
pub const SITE_NAMES: [&str; 5] = ["", "estuary head", "confluence", "lake outlet", "gap"];

/// Everything the land mechanism knows beyond the port's `World`: the fine grid
/// and the water. `World::landform` holds one when the mechanism is on.
///
/// Fine arrays are row-major, `fh` rows of `fw` nodes, `fine` nodes to a cell's
/// side; node (i, j) has its centre at ((i + 0.5) / fine, (j + 0.5) / fine) in
/// cell units. Cell arrays are row-major like the `World`'s.
#[derive(Clone, Debug, Default)]
pub struct Landform {
    pub fine: usize,
    pub fw: usize,
    pub fh: usize,
    /// Fine: height in metres above the sea (below 0 under it; a lake's bed under a lake).
    pub height: Vec<f32>,
    /// Fine: rock at the surface (`landform::ROCK_*`).
    pub rock: Vec<u8>,
    /// Fine: 0, or the lake a node lies under plus 1 (its place in `lakes`).
    pub lake_id: Vec<u16>,
    /// Fine: mean flow through the node, cubic metres a second.
    pub flow: Vec<f32>,
    /// Fine: `landwater::WET_*` (dry, flood plain, marsh).
    pub wet: Vec<u8>,
    pub rivers: Vec<River>,
    pub oxbows: Vec<Oxbow>,
    pub lakes: Vec<Lake>,
    /// Where each river system meets the sea, the largest first.
    pub outlets: Vec<Outlet>,
    /// The way the prevailing wind blows, a unit vector in (x, y).
    pub wind: (f64, f64),
    /// How far the sea rose at the end, metres.
    pub drowned: f64,
    /// Height in metres that is 1 in `World::h`.
    pub h_ref: f64,

    /// Cell: mostly lake. Such a cell is water to the economy (`World::land` is false).
    pub lake: Vec<bool>,
    /// Cell: land beside a lake cell.
    pub shore: Vec<bool>,
    /// Cell: share of its area under a lake.
    pub lake_share: Vec<f32>,
    /// Cell: rain, mm a year.
    pub rain: Vec<f32>,
    /// Cell: runoff in each month of an average year, mm (`runoff[m * N + c]`, m = 0 for January).
    pub runoff: Vec<f32>,
    /// Cell: mean flow of its largest stream, cubic metres a second.
    pub discharge: Vec<f32>,
    /// Cell: the sea outlet its water reaches (a place in `outlets`), -1 for water.
    pub basin: Vec<i32>,
    /// Cell: the river its water joins first (a place in `rivers`), -1 for none.
    pub stream: Vec<i32>,
    /// Cell: water power, kW: flow times fall, summed over its streams.
    pub power: Vec<f32>,
    /// Cell: share of it that is flood plain (marsh included), and that is marsh or fen.
    pub floodplain: Vec<f32>,
    pub marsh: Vec<f32>,
    /// Cell: how wet the ground is, 0 to 1.
    pub wetness: Vec<f32>,
    /// Cell: mean slope of the ground (rise over run).
    pub slope: Vec<f32>,
    /// Cell: range of height within it, metres.
    pub relief: Vec<f32>,
    /// Cell: why a town would grow here (`SITE_*`) and how strongly, 0 to 1.
    pub site_kind: Vec<u8>,
    pub site: Vec<f32>,
}

impl Landform {
    /// Mean flow at every cell's largest stream in a year or month whose runoff
    /// is `mult[c]` times the average in each cell (1 everywhere gives
    /// `discharge` back, near enough). `down` is the world's. For a later
    /// mechanism's wet and dry years: flow is runoff gathered downstream, so a
    /// dry district upstream lowers the river all the way to the sea.
    pub fn discharge_with(&self, world: &World, mult: &[f64]) -> Vec<f64> {
        let N = world.W * world.H;
        let year: Vec<f64> = (0..N).map(|c| (0..12).map(|m| self.runoff[m * N + c] as f64).sum()).collect();
        let (wet, avg) = (self.gather(world, &(0..N).map(|c| year[c] * mult[c]).collect::<Vec<_>>()), self.gather(world, &year));
        (0..N).map(|c| if avg[c] > 0.0 { self.discharge[c] as f64 * wet[c] / avg[c] } else { 0.0 }).collect()
    }

    /// Mean flow at every cell's largest stream in month m (0 for January) of an average year.
    pub fn discharge_month(&self, world: &World, m: usize) -> Vec<f64> {
        let N = world.W * world.H;
        let year: Vec<f64> = (0..N).map(|c| (0..12).map(|k| self.runoff[k * N + c] as f64).sum()).collect();
        let month: Vec<f64> = (0..N).map(|c| 12.0 * self.runoff[m * N + c] as f64).collect();
        let (wet, avg) = (self.gather(world, &month), self.gather(world, &year));
        (0..N).map(|c| if avg[c] > 0.0 { self.discharge[c] as f64 * wet[c] / avg[c] } else { 0.0 }).collect()
    }

    /// Sum a cell quantity down the world's flow directions.
    fn gather(&self, world: &World, v: &[f64]) -> Vec<f64> {
        let N = world.W * world.H;
        let mut out: Vec<f64> = (0..N).map(|c| if world.land[c] { v[c] } else { 0.0 }).collect();
        // upstream cells first: a cell is done when every cell running into it is
        let inland = |c: usize| world.land[c] && world.down[c] >= 0 && world.land[world.down[c] as usize];
        let mut waiting = vec![0u32; N];
        for c in 0..N {
            if inland(c) {
                waiting[world.down[c] as usize] += 1;
            }
        }
        let mut ready: Vec<usize> = (0..N).filter(|&c| world.land[c] && waiting[c] == 0).collect();
        while let Some(c) = ready.pop() {
            if inland(c) {
                let d = world.down[c] as usize;
                out[d] += out[c];
                waiting[d] -= 1;
                if waiting[d] == 0 {
                    ready.push(d);
                }
            }
        }
        out
    }
}

/// The island for a seed, made by water: the `World` the economy runs on, with
/// its `landform`. Draws only from the land's own stream.
pub fn generate_land(seed: u64, W: usize, H: usize, n_towns: usize, P: &LandParams) -> World {
    let mut rng = land_stream(seed);
    let d = draw_design(&mut rng, W, H, P);
    let relief = build_relief(&d, P);
    let water = make_water(&d, P, &relief, &mut rng);
    make_world(&d, P, &relief, &water, &mut rng, n_towns)
}

/// Reduce the fine land and its water to cells.
pub fn make_world(d: &Design, P: &LandParams, relief: &Relief, wt: &Water, rng: &mut Rng, n_towns: usize) -> World {
    let (W, H) = (d.W, d.H);
    let N = W * H;
    let g = Grid { w: wt.w, h: wt.h, k: wt.k };
    let k2 = (g.k * g.k) as f64;
    let dx = g.dx();
    let node_km2 = dx * dx / 1e6;
    let rt = &wt.rt;

    // what each cell is made of
    let (mut n_dry, mut n_lake) = (vec![0usize; N], vec![0usize; N]);
    for c in 0..g.n() {
        let cell = g.cell(c);
        if wt.lake_id[c] > 0 {
            n_lake[cell] += 1;
        } else if !wt.sea[c] {
            n_dry[cell] += 1;
        }
    }
    let mut land: Vec<bool> = (0..N).map(|c| 2.0 * n_dry[c] as f64 >= k2).collect();
    // one island
    let (lab, nl) = ndimage::label(&land, H, W);
    if nl > 1 {
        let mut sizes = vec![0usize; nl + 1];
        for &l in &lab {
            sizes[l as usize] += 1;
        }
        let mut keep = 1;
        for l in 2..=nl {
            if sizes[l] > sizes[keep] {
                keep = l;
            }
        }
        land = lab.iter().map(|&l| l as usize == keep).collect();
    }
    let mut lake: Vec<bool> = (0..N).map(|c| !land[c] && n_lake[c] as f64 * 2.0 > k2 - n_dry[c] as f64).collect();
    // water the open sea does not reach (boats go by all eight neighbours) and
    // that is not a lake is an inlet narrower than a cell: land, as in the port
    {
        let mut open = vec![false; N];
        let mut stack = vec![0usize];
        open[0] = true;
        while let Some(c) = stack.pop() {
            for (dy, dx) in NEIGH8 {
                let (y, x) = ((c / W) as isize + dy, (c % W) as isize + dx);
                if y >= 0 && y < H as isize && x >= 0 && x < W as isize {
                    let nc = y as usize * W + x as usize;
                    if !open[nc] && !land[nc] && !lake[nc] {
                        open[nc] = true;
                        stack.push(nc);
                    }
                }
            }
        }
        for c in 0..N {
            if !land[c] && !lake[c] && !open[c] {
                land[c] = true;
            }
        }
        // a lake the sea touches is a lagoon: sea
        for c in 0..N {
            if lake[c] && n_lake[c] == 0 {
                lake[c] = false;
            }
        }
    }

    // each cell's ground: heights, slope, rock, wet ground
    let off = g.offsets();
    let (mut zsum, mut zmin, mut zmax) = (vec![0.0f64; N], vec![f64::INFINITY; N], vec![f64::NEG_INFINITY; N]);
    let (mut slope, mut sed, mut plain, mut marsh, mut twi) = (vec![0.0f64; N], vec![0.0f64; N], vec![0.0f64; N], vec![0.0f64; N], vec![0.0f64; N]);
    let (mut stone, mut clay, mut coal, mut power) = (vec![0.0f64; N], vec![0.0f64; N], vec![0.0f64; N], vec![0.0f64; N]);
    let mut soil = vec![0.0f64; N];
    let mut main = vec![usize::MAX; N];
    for &c in &rt.order {
        let c = c as usize;
        let cell = g.cell(c);
        // the cell's largest stream, a lake's through-flow included: each cell's
        // is then smaller than that of the cell it runs into, so `down` has no loops
        if main[cell] == usize::MAX || wt.flow[c] > wt.flow[main[cell]] {
            main[cell] = c;
        }
        if wt.lake_id[c] > 0 {
            continue;
        }
        let z = wt.z[c].max(0.0);
        zsum[cell] += z;
        zmin[cell] = zmin[cell].min(z);
        zmax[cell] = zmax[cell].max(z);
        // the ground's slope: the steepest fall to a neighbour
        let mut s = 0.0f64;
        for k in 0..8 {
            let nb = (c as isize + off[k]) as usize;
            let zn = if wt.sea[nb] { 0.0 } else if wt.lake_id[nb] > 0 { rt.fill[nb] } else { wt.z[nb] };
            s = s.max((wt.z[c] - zn) / (dx * crate::landform::N8[k].2));
        }
        slope[cell] += s;
        sed[cell] += relief.sed[c];
        if wt.wet[c] == WET_MARSH {
            marsh[cell] += 1.0;
            plain[cell] += 1.0;
        } else if wt.wet[c] == WET_PLAIN {
            plain[cell] += 1.0;
        }
        twi[cell] += (rt.area[c] * dx / s.max(0.002)).ln();
        let bare = smoothstep(0.04, 0.25, s);
        let rock = if relief.sed[c] > 1.0 { ROCK_ALLUVIUM } else { relief.rock[c] };
        let (st, cl, so) = match rock {
            ROCK_BASEMENT => (0.3, 0.05, 0.45),
            ROCK_CAP => (0.3, 0.05, 0.4),
            ROCK_BAND => (0.2, 0.12, 0.65),
            ROCK_COAL => (0.1, 0.35, 0.6),
            ROCK_SOFT => (0.02, 0.5, 0.8),
            ROCK_VALE => (0.02, 0.55, 0.85),
            _ => (0.0, 0.65, 1.0),
        };
        stone[cell] += (st * (0.5 + 1.8 * bare)).min(1.0);
        clay[cell] += cl * (1.0 - smoothstep(0.03, 0.15, s)) * (1.0 - smoothstep(150.0, 400.0, z));
        soil[cell] += so;
        // coal: the seam lies deeper towards the middle of its field; it is won
        // where the land has been worn down nearly to it, along the valley sides
        let cw = relief.coal[c] as f64;
        if cw > 0.18 {
            let seam = 30.0 + 110.0 * ((cw - 0.18) / 0.6).min(1.0);
            let gap = seam - relief.worn[c];
            let near = smoothstep(-70.0, -40.0, gap) * (1.0 - smoothstep(25.0, 45.0, gap));
            coal[cell] += cw * (0.35 + 0.65 * near);
        }
        let r = rt.rcv[c] as usize;
        if wt.flow[c] >= 0.05 {
            let fall = rt.fill[c] - if wt.sea[r] { 0.0 } else { rt.fill[r] };
            power[cell] += 9.81 * wt.flow[c] * fall.max(0.0);
        }
    }

    let mut h = vec![0.0f64; N];
    let mut terrain = vec![SEA; N];
    let mut lf = Landform {
        fine: g.k,
        fw: g.w,
        fh: g.h,
        height: wt.z.iter().map(|&v| v as f32).collect(),
        rock: (0..g.n()).map(|c| if relief.sed[c] > 1.0 && !wt.sea[c] { ROCK_ALLUVIUM } else { relief.rock[c] }).collect(),
        lake_id: wt.lake_id.clone(),
        flow: wt.flow.iter().map(|&v| v as f32).collect(),
        wet: wt.wet.clone(),
        rivers: wt.rivers.clone(),
        oxbows: wt.oxbows.clone(),
        lakes: wt.lakes.clone(),
        outlets: Vec::new(),
        wind: d.wind,
        drowned: wt.drowned,
        h_ref: P.h_ref,
        lake: lake.clone(),
        shore: vec![false; N],
        lake_share: (0..N).map(|c| (n_lake[c] as f64 / k2) as f32).collect(),
        rain: wt.rain.iter().map(|&v| v as f32).collect(),
        runoff: wt.runoff.clone(),
        discharge: vec![0.0; N],
        basin: vec![-1; N],
        stream: vec![-1; N],
        power: vec![0.0; N],
        floodplain: vec![0.0; N],
        marsh: vec![0.0; N],
        wetness: vec![0.0; N],
        slope: vec![0.0; N],
        relief: vec![0.0; N],
        site_kind: vec![SITE_NONE; N],
        site: vec![0.0; N],
    };
    for c in 0..N {
        let nd = n_dry[c].max(1) as f64;
        if land[c] {
            let zc = zsum[c] / nd;
            let rel = if zmax[c] > zmin[c] { zmax[c] - zmin[c] } else { 0.0 };
            h[c] = (zc / P.h_ref).clamp(0.0, 1.0);
            terrain[c] = if zc > 380.0 || rel > 260.0 {
                MOUNTAINS
            } else if zc > 90.0 || rel > 50.0 {
                HILLS
            } else {
                LOWLAND
            };
            lf.slope[c] = (slope[c] / nd) as f32;
            lf.relief[c] = rel as f32;
            lf.floodplain[c] = (plain[c] / k2) as f32;
            lf.marsh[c] = (marsh[c] / k2) as f32;
            lf.power[c] = power[c] as f32;
            let wet = 0.5 * ((twi[c] / nd - 8.0) / 5.0).clamp(0.0, 1.0)
                + 0.3 * smoothstep(600.0, 1800.0, wt.rain[c])
                + 0.4 * marsh[c] / k2
                + 0.2 * lf.lake_share[c] as f64;
            lf.wetness[c] = wet.clamp(0.0, 1.0) as f32;
        } else if lake[c] {
            // a lake cell stands at its water's level
            let mut lev = 0.0f64;
            for j in 0..g.k {
                for i in 0..g.k {
                    let nd = ((c / W) * g.k + j) * g.w + (c % W) * g.k + i;
                    if wt.lake_id[nd] > 0 {
                        lev = lev.max(wt.lakes[wt.lake_id[nd] as usize - 1].level as f64);
                    }
                }
            }
            h[c] = (lev / P.h_ref).clamp(0.0, 1.0);
        }
    }

    // where the water of each cell goes: follow its largest stream until it leaves the cell
    let mut down = vec![-1i64; N];
    let mut acc = vec![0.0f64; N];
    let mut gradient = vec![0.0f64; N];
    // the node where each node's water meets the sea, and the river it joins
    let mut mouth = vec![u32::MAX; g.n()];
    let mut joins = vec![-1i32; g.n()];
    for &c in &rt.order {
        let c = c as usize;
        let r = rt.rcv[c] as usize;
        mouth[c] = if wt.sea[r] { c as u32 } else { mouth[r] };
        joins[c] = if wt.owner[c] >= 0 {
            wt.owner[c]
        } else if wt.sea[r] {
            -1
        } else {
            joins[r]
        };
    }
    let mut mouth_of: Vec<u32> = Vec::new();
    for c in 0..N {
        if !land[c] || main[c] == usize::MAX {
            continue;
        }
        let m = main[c];
        acc[c] = rt.area[m] / k2;
        lf.discharge[c] = wt.flow[m] as f32;
        lf.stream[c] = joins[m];
        mouth_of.push(mouth[m]);
        let mut cur = m;
        let mut steps = 0;
        loop {
            let r = rt.rcv[cur] as usize;
            if steps < g.k {
                gradient[c] = (rt.fill[m] - if wt.sea[r] { 0.0 } else { rt.fill[r] }) / ((steps + 1) as f64 * dx);
                steps += 1;
            }
            if wt.sea[r] {
                // the stream reaches the sea: it runs to the nearest water cell,
                // as the port's coastal cells do (or nowhere, on an inlet
                // narrower than a cell)
                let (ry, rx) = (r / g.w, r % g.w);
                let mut best = f64::INFINITY;
                for (dy, dx8) in NEIGH8 {
                    let (y, x) = ((c / W) as isize + dy, (c % W) as isize + dx8);
                    let nc = y as usize * W + x as usize;
                    if !land[nc] {
                        let dist = ((y as f64 + 0.5) * g.k as f64 - ry as f64).hypot((x as f64 + 0.5) * g.k as f64 - rx as f64);
                        if dist < best {
                            best = dist;
                            down[c] = nc as i64;
                        }
                    }
                }
                break;
            }
            if g.cell(r) != c {
                down[c] = g.cell(r) as i64;
                break;
            }
            cur = r;
        }
    }
    mouth_of.sort_unstable();
    mouth_of.dedup();
    mouth_of.sort_by(|&a, &b| rt.area[b as usize].total_cmp(&rt.area[a as usize]).then(a.cmp(&b)));
    lf.outlets = mouth_of
        .iter()
        .map(|&m| Outlet { cell: g.cell(m as usize), area_km2: (rt.area[m as usize] * node_km2) as f32, flow: wt.flow[m as usize] as f32 })
        .collect();
    for c in 0..N {
        if land[c] && main[c] != usize::MAX {
            let m = mouth[main[c]];
            lf.basin[c] = mouth_of.iter().position(|&x| x == m).map_or(-1, |p| p as i32);
        }
    }

    let near = |mask: &dyn Fn(usize) -> bool, c: usize| -> bool {
        NEIGH8.iter().any(|&(dy, dx8)| {
            let (y, x) = ((c / W) as isize + dy, (c % W) as isize + dx8);
            y >= 0 && y < H as isize && x >= 0 && x < W as isize && mask(y as usize * W + x as usize)
        })
    };
    let coast: Vec<bool> = (0..N).map(|c| land[c] && near(&|n| !land[n] && !lake[n], c)).collect();
    lf.shore = (0..N).map(|c| land[c] && near(&|n| lake[n], c)).collect();
    // a river is a cell with flow enough; boats use it where it is big and slack,
    // and a cell with a good part under a lake
    let river: Vec<bool> = (0..N).map(|c| land[c] && (lf.discharge[c] as f64 >= P.river_flow || lf.lake_share[c] >= 0.25)).collect();
    let navigable: Vec<bool> =
        (0..N).map(|c| land[c] && ((lf.discharge[c] as f64 >= P.nav_flow && gradient[c] <= P.nav_slope) || lf.lake_share[c] >= 0.25)).collect();

    // the resource fields
    let jitter: Vec<f64> = (0..N).map(|c| fbm01((c % W) as f64 / 4.0, (c / W) as f64 / 4.0, d.seeds[2] ^ 0x717, 2, 0.5)).collect();
    let grain = |c: usize, s: u64| 0.85 + 0.3 * fbm01((c % W) as f64 / 6.0, (c / W) as f64 / 6.0, d.seeds[3] ^ s, 3, 0.5);
    let mut world = World { W, H, ..World::default() };
    world.fertility = vec![0.0; N];
    world.pasture = vec![0.0; N];
    world.forest_pot = vec![0.0; N];
    world.rock = vec![0.0; N];
    world.clay = vec![0.0; N];
    world.ore = vec![0.0; N];
    world.coal = vec![0.0; N];
    for c in 0..N {
        if !land[c] {
            continue;
        }
        let nd = n_dry[c].max(1) as f64;
        let (zc, sl, rain) = (zsum[c] / nd, slope[c] / nd, wt.rain[c]);
        let (fp, ma) = (plain[c] / k2, marsh[c] / k2);
        let (x, y) = ((c % W) as f64 + 0.5, (c / W) as f64 + 0.5);
        // corn land: low, level, neither sodden nor parched, on a kindly soil;
        // best of all on the flood plain's silt, where it is not marsh
        let low = 1.0 - smoothstep(110.0, 480.0, zc);
        let level = 1.0 - smoothstep(0.025, 0.2, sl);
        let weather = 1.0 - 0.5 * smoothstep(1000.0, 2200.0, rain) - 0.2 * (1.0 - smoothstep(420.0, 600.0, rain));
        let silt = (0.5 * (sed[c] / nd / 3.0).min(1.0) + 0.5 * fp) * (1.0 - ma);
        let f = (0.06 + 0.82 * low * level * weather * (soil[c] / nd) * grain(c, 1) + 0.2 * silt) * (1.0 - 0.6 * ma);
        world.fertility[c] = f.clamp(0.0, 1.0);
        // grass: where it is wet, cool or steep, and on the water meadows; thin on the tops
        let p = 0.4 + 0.2 * smoothstep(650.0, 1500.0, rain) + 0.24 * smoothstep(50.0, 280.0, zc) + 0.12 * smoothstep(0.03, 0.15, sl) + 0.1 * fp
            - 0.5 * smoothstep(420.0, 800.0, zc)
            - 0.2 * ma;
        world.pasture[c] = (p * grain(c, 2)).clamp(0.0, 1.0);
        // woodland: rain and height; none above the tree line, little in the fens
        let t = 0.56 + 0.22 * smoothstep(550.0, 1200.0, rain) + 0.14 * smoothstep(40.0, 250.0, zc) - 0.62 * smoothstep(380.0, 760.0, zc) - 0.3 * ma;
        world.forest_pot[c] = (t * grain(c, 3)).clamp(0.0, 1.0);
        world.rock[c] = (2.2 * stone[c] / nd * grain(c, 4)).clamp(0.0, 1.0);
        world.clay[c] = (0.95 * clay[c] / nd * grain(c, 5)).clamp(0.0, 1.0);
        let co = 2.8 * coal[c] / nd;
        world.coal[c] = if co > 0.18 { co.min(1.0) } else { 0.0 };
        // ore bodies in the old rocks of the belt; ironstone in the coal measures
        let mut o = 0.0f64;
        for b in &d.ore {
            o = o.max(b.k * b.at(x, y));
        }
        let o = if o > 0.15 { o * (0.6 + 0.4 * smoothstep(40.0, 200.0, zc)) } else { 0.0 };
        world.ore[c] = o.max(0.4 * world.coal[c]);
    }

    world.land = land;
    world.h = h;
    world.terrain = terrain;
    world.down = down;
    world.acc = acc;
    world.river = river;
    world.navigable = navigable;
    world.coast = coast;
    world.jitter = jitter;

    // town sites: where towns really grew
    let mut up_rivers = vec![0u8; N];
    for c in 0..N {
        let dn = world.down[c];
        if world.land[c] && world.river[c] && dn >= 0 && world.land[dn as usize] {
            up_rivers[dn as usize] += 1;
        }
    }
    for c in 0..N {
        if !world.land[c] {
            continue;
        }
        let dn = world.down[c];
        let (mut kind, mut s) = (SITE_NONE, 0.0f64);
        if world.river[c] && dn >= 0 && !world.land[dn as usize] && !lake[dn as usize] {
            kind = SITE_ESTUARY;
            s = 0.6 + 0.4 * (lf.discharge[c] as f64 / P.nav_flow).min(1.0);
        }
        if up_rivers[c] >= 2 && world.navigable[c] && s < 0.5 {
            kind = SITE_CONFLUENCE;
            s = 0.5;
        }
        // a gap: lower than the ground a few cells off on two opposite sides,
        // level with the ground along the other axis
        let (y, x) = ((c / W) as isize, (c % W) as isize);
        if s < 0.5 && x >= 4 && y >= 4 && x + 4 < W as isize && y + 4 < H as isize {
            let hm = |dy: isize, dx8: isize| world.h[((y + dy) * W as isize + x + dx8) as usize] * P.h_ref;
            let here = world.h[c] * P.h_ref;
            for (dy, dx8) in [(0isize, 1isize), (1, 0), (1, 1), (1, -1)] {
                let walls = hm(3 * dy, 3 * dx8).min(hm(-3 * dy, -3 * dx8)) - here;
                let along = hm(3 * dx8, -3 * dy).max(hm(-3 * dx8, 3 * dy)) - here;
                if walls > 110.0 && along < 45.0 && world.terrain[c] != MOUNTAINS {
                    kind = SITE_GAP;
                    s = 0.5;
                }
            }
        }
        lf.site_kind[c] = kind;
        lf.site[c] = s as f32;
    }
    for lk in &wt.lakes {
        if lk.area_km2 >= 3.0 {
            let c = (lk.outlet.1 as usize).min(H - 1) * W + (lk.outlet.0 as usize).min(W - 1);
            if world.land[c] && lf.site[c] < 0.8 {
                lf.site_kind[c] = SITE_LAKE_OUTLET;
                lf.site[c] = 0.8;
            }
        }
    }
    world.town_sites = pick_sites(&world, &lf, rng, n_towns);
    // woods are already thinned around old settlements in 1750, as in the port
    let mut cleared = vec![0.0f64; N];
    for &c in &world.town_sites {
        for i in 0..N {
            let dist = ((i / W) as f64 - (c / W) as f64).hypot((i % W) as f64 - (c % W) as f64);
            cleared[i] = cleared[i].max((-dist / 9.0).exp());
        }
    }
    world.forest0 = (0..N).map(|i| world.forest_pot[i] * (1.0 - 0.85 * cleared[i])).collect();
    world.landform = Some(Box::new(lf));
    world
}

/// The first towns: the port's rule (a fertile country round about, navigable
/// water, the coast, well apart) with the places towns really grew counted in:
/// the head of an estuary, a confluence, a lake's outlet, a gap through the hills.
fn pick_sites(world: &World, lf: &Landform, rng: &mut Rng, n: usize) -> Vec<usize> {
    let (H, W) = (world.H, world.W);
    let N = H * W;
    let fert_sum = ndimage::uniform_filter(&world.fertility, H, W, 11, Mode::Reflect);
    let near_nav = ndimage::binary_dilation(&world.navigable, H, W, false);
    let noise = random_n(rng, N);
    let mut score: Vec<f64> = (0..N)
        .map(|c| fert_sum[c] + 0.35 * (near_nav[c] as u8 as f64) + 0.25 * (world.coast[c] as u8 as f64) + 0.4 * lf.site[c] as f64 + 0.08 * noise[c])
        .collect();
    for c in 0..N {
        let (y, x) = (c / W, c % W);
        if !world.land[c] || world.terrain[c] == MOUNTAINS || y < 3 || y >= H - 3 || x < 3 || x >= W - 3 {
            score[c] = f64::NEG_INFINITY;
        }
    }
    let mut sites = Vec::new();
    for _ in 0..n {
        let mut c = 0;
        for i in 1..N {
            if score[i] > score[c] {
                c = i;
            }
        }
        if !score[c].is_finite() {
            break;
        }
        sites.push(c);
        for i in 0..N {
            if ((i / W) as f64 - (c / W) as f64).hypot((i % W) as f64 - (c % W) as f64) < 17.0 {
                score[i] = f64::NEG_INFINITY;
            }
        }
    }
    sites
}
