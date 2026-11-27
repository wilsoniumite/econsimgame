//! The start page's small map of a new island (start_page.rs, "Island
//! settings"): land and sea, heights, rivers and lakes, the towns of the
//! start and marks for coal, ore, stone and clay, with the land's area and
//! the people a square kilometre at the start under it.
//!
//! It is made off the main thread, by two threads of its own that each take
//! only the newest island asked for: one makes the sketch of the land
//! (`landcells::sketch_land_towns`, under a second), the other, once the
//! settings have rested for a moment, the whole island
//! (`landcells::generate_land`, two to five seconds), which then takes the
//! sketch's place. The page never waits for either: it shows the last map it
//! has, dimmed while a newer one is being made.
//!
//! Everything here but the threads is plain arithmetic on the core's `World`,
//! so the tests can draw a map without a window.

use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use isostate_core::island::Island;
use isostate_core::params::{Params, Rules};
use isostate_core::world::World;

/// The map's size in pixels: three to a 2 km cell of the 120 by 84 island.
pub const MAP_W: usize = 360;
pub const MAP_H: usize = 252;
/// A cell's area, km² (a cell is 2 km square).
pub const CELL_KM2: f64 = 4.0;

/// The island a map is asked for: everything on the page that changes its land
/// or its towns.
#[derive(Clone, Debug, PartialEq)]
pub struct MapAsk {
    pub seed: u64,
    pub old_land: bool,
    pub landfall: bool,
    /// the settings as the command line will say them (`Choices::island_said`)
    pub island: Island,
}

/// A map made.
#[derive(Clone, Debug)]
pub struct MapPicture {
    /// which ask it answers, counted up by the page
    pub turn: u64,
    /// the whole island, and not the sketch
    pub whole: bool,
    /// MAP_W by MAP_H, RGBA, 8 bits each, sRGB
    pub rgba: Vec<u8>,
    pub land_km2: f64,
    /// the people of the start
    pub people: f64,
    pub towns: usize,
    /// seconds it took to make, land and drawing
    pub took: f32,
    /// what could not be made, said plainly; the picture is then empty sea
    pub fault: Option<String>,
}

impl MapPicture {
    /// People a square kilometre of land at the start.
    pub fn density(&self) -> f64 {
        if self.land_km2 > 0.0 { self.people / self.land_km2 } else { 0.0 }
    }
}

/// The rules a new game with these choices begins on, as `Game::begin_with` lays them.
pub fn params_for(ask: &MapAsk) -> Result<Params, String> {
    let mut p = isostate_core::save::params(Rules::Game, ask.seed, crate::start_page::FAMILIES);
    if ask.old_land {
        p.land = isostate_core::landform::LandParams::default();
    }
    if ask.landfall {
        isostate_core::landfall::landfall(&mut p);
    }
    ask.island.set(&mut p)?;
    Ok(p)
}

/// The island's land, and the cells of its towns at the start: the sketch, or
/// the whole of it. The old island has no sketch and is always whole.
pub fn land_of(p: &Params, whole: bool) -> (World, Vec<usize>) {
    let mut world = if !p.land.enabled {
        isostate_core::world::generate_world(&mut isostate_core::rng::make_rngs(p.seed).world, p.W, p.H, p.n_towns, 0.58)
    } else if whole {
        isostate_core::landcells::generate_land(p.seed, p.W, p.H, p.n_towns, &p.land)
    } else {
        isostate_core::landcells::sketch_land_towns(p.seed, p.W, p.H, p.n_towns, &p.land)
    };
    if p.start.landfall {
        world.town_sites = isostate_core::landfall::site(&world).into_iter().collect();
    }
    let sites = world.town_sites.clone();
    (world, sites)
}

/// Make the map for `ask`, on whatever thread calls it.
pub fn make(ask: &MapAsk, turn: u64, whole: bool) -> MapPicture {
    let t0 = Instant::now();
    let empty = |fault: String| MapPicture { turn, whole, rgba: vec![0; MAP_W * MAP_H * 4], land_km2: 0.0, people: 0.0, towns: 0, took: 0.0, fault: Some(fault) };
    let p = match params_for(ask) {
        Ok(p) => p,
        Err(e) => return empty(e),
    };
    let (world, sites) = land_of(&p, whole || !p.land.enabled);
    let rgba = draw(&world, &sites);
    let land = world.land.iter().filter(|&&l| l).count() as f64 * CELL_KM2;
    MapPicture { turn, whole: whole || !p.land.enabled, rgba, land_km2: land, people: isostate_core::landfall::people(&p), towns: sites.len(), took: t0.elapsed().as_secs_f32(), fault: None }
}

// ------------------------------------------------------------------ the drawing

type Rgb = [f32; 3];

const SEA_DEEP: Rgb = [0.33, 0.50, 0.62];
const SEA_SHALLOW: Rgb = [0.56, 0.73, 0.79];
const LAKE: Rgb = [0.44, 0.65, 0.80];
const RIVER: Rgb = [0.27, 0.47, 0.70];
const WOODS: Rgb = [0.33, 0.50, 0.29];
/// The marks of the four riches, as the legend under the map shows them.
pub const COAL: Rgb = [0.10, 0.10, 0.11];
pub const ORE: Rgb = [0.74, 0.26, 0.15];
pub const STONE: Rgb = [0.40, 0.38, 0.62];
pub const CLAY: Rgb = [0.90, 0.60, 0.25];
const TOWN: Rgb = [0.22, 0.13, 0.08];

/// Where a mark counts a cell as rich in it: coal wherever a coalfield is,
/// ore where the ore bodies are (not the ironstone of the coal measures, which
/// coal's mark already shows), stone and clay where they are thick.
pub const STONE_MARK: f64 = 0.9;
pub const CLAY_MARK: f64 = 0.54;
pub const ORE_MARK: f64 = 0.15;

/// The land's colour by height, metres: lowland green, upland tan, high
/// ground brown and grey, snow on the tops.
fn by_height(z: f32) -> Rgb {
    const STOPS: [(f32, Rgb); 6] = [(0.0, [0.60, 0.71, 0.44]), (150.0, [0.69, 0.75, 0.49]), (400.0, [0.78, 0.73, 0.53]), (700.0, [0.67, 0.58, 0.46]), (1000.0, [0.68, 0.64, 0.61]), (1300.0, [0.96, 0.96, 0.96])];
    if z <= STOPS[0].0 {
        return STOPS[0].1;
    }
    for w in STOPS.windows(2) {
        if z <= w[1].0 {
            let t = (z - w[0].0) / (w[1].0 - w[0].0);
            return mix(w[0].1, w[1].1, t);
        }
    }
    STOPS[5].1
}

fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// The map of `world` with its towns at `sites`, MAP_W by MAP_H, RGBA.
pub fn draw(world: &World, sites: &[usize]) -> Vec<u8> {
    let (cols, rows) = (world.W, world.H);
    let (sx, sy) = (MAP_W as f32 / cols as f32, MAP_H as f32 / rows as f32);
    let cell_of = |px: usize, py: usize| ((((py as f32 + 0.5) / sy) as usize).min(rows - 1)) * cols + (((px as f32 + 0.5) / sx) as usize).min(cols - 1);
    let lf = world.landform.as_deref();
    // the height of every pixel, metres (below 0 at sea), and whether it is under a lake
    let mut z = vec![0f32; MAP_W * MAP_H];
    let mut lake = vec![false; MAP_W * MAP_H];
    for py in 0..MAP_H {
        for px in 0..MAP_W {
            let i = py * MAP_W + px;
            let (xc, yc) = ((px as f32 + 0.5) / sx, (py as f32 + 0.5) / sy);
            match lf {
                Some(lf) => {
                    let k = lf.fine as f32;
                    let (fx, fy) = ((xc * k - 0.5).clamp(0.0, (lf.fw - 1) as f32), (yc * k - 0.5).clamp(0.0, (lf.fh - 1) as f32));
                    let (x0, y0) = (fx as usize, fy as usize);
                    let (x1, y1) = ((x0 + 1).min(lf.fw - 1), (y0 + 1).min(lf.fh - 1));
                    let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
                    let at = |x: usize, y: usize| lf.height[y * lf.fw + x];
                    z[i] = (at(x0, y0) * (1.0 - tx) + at(x1, y0) * tx) * (1.0 - ty) + (at(x0, y1) * (1.0 - tx) + at(x1, y1) * tx) * ty;
                    lake[i] = lf.lake_id[(fy.round() as usize).min(lf.fh - 1) * lf.fw + (fx.round() as usize).min(lf.fw - 1)] > 0;
                }
                None => {
                    let c = cell_of(px, py);
                    z[i] = if world.land[c] { 5.0 + world.h[c] as f32 * 1200.0 } else { -60.0 };
                }
            }
        }
    }
    let wet = |i: usize| z[i] <= 0.0 && !lake[i];
    let mut rgb = vec![[0f32; 3]; MAP_W * MAP_H];
    // the metres a pixel spans, and the relief shown three times over, so that a small map reads
    let span = 2000.0 / sx;
    for py in 0..MAP_H {
        for px in 0..MAP_W {
            let i = py * MAP_W + px;
            if lake[i] {
                rgb[i] = LAKE;
                continue;
            }
            if wet(i) {
                rgb[i] = mix(SEA_SHALLOW, SEA_DEEP, (-z[i] / 160.0).clamp(0.0, 1.0));
                continue;
            }
            let c = cell_of(px, py);
            let mut col = by_height(z[i]);
            col = mix(col, WOODS, 0.28 * world.forest_pot.get(c).copied().unwrap_or(0.0) as f32 * (1.0 - (z[i] / 900.0).clamp(0.0, 1.0)));
            // light from the north-west
            let zl = z[py * MAP_W + px.saturating_sub(1)].max(0.0);
            let zr = z[py * MAP_W + (px + 1).min(MAP_W - 1)].max(0.0);
            let zu = z[py.saturating_sub(1) * MAP_W + px].max(0.0);
            let zd = z[(py + 1).min(MAP_H - 1) * MAP_W + px].max(0.0);
            let (nx, ny) = (-3.0 * (zr - zl) / (2.0 * span), -3.0 * (zd - zu) / (2.0 * span));
            let lambert = (-0.55 * nx - 0.55 * ny + 0.63) / (nx * nx + ny * ny + 1.0).sqrt();
            let shade = (1.0 + 1.3 * (lambert - 0.63)).clamp(0.62, 1.22);
            col = [col[0] * shade, col[1] * shade, col[2] * shade];
            // the coast, a darker line
            let near_sea = [(px.wrapping_sub(1), py), (px + 1, py), (px, py.wrapping_sub(1)), (px, py + 1)].iter().any(|&(x, y)| x < MAP_W && y < MAP_H && wet(y * MAP_W + x));
            if near_sea {
                col = [col[0] * 0.72, col[1] * 0.72, col[2] * 0.72];
            }
            rgb[i] = col;
        }
    }
    // rivers: the lines of the land, or on the old island each river cell to the next
    let mut canvas = Canvas { rgb, sx, sy };
    match lf {
        Some(lf) => {
            for r in &lf.rivers {
                for k in 1..r.x.len() {
                    let w = (0.45 + 0.3 * (r.flow[k] / 15.0).sqrt()).min(1.6);
                    canvas.line((r.x[k - 1], r.y[k - 1]), (r.x[k], r.y[k]), w, RIVER, 0.95);
                }
            }
        }
        None => {
            for c in 0..cols * rows {
                let d = world.down[c];
                if world.land[c] && world.river[c] && d >= 0 {
                    let d = d as usize;
                    let at = |c: usize| ((c % cols) as f32 + 0.5, (c / cols) as f32 + 0.5);
                    canvas.line(at(c), at(d), 0.6, RIVER, 0.9);
                }
            }
        }
    }
    // the riches, a dot in a corner of each cell rich in one
    for c in 0..cols * rows {
        if !world.land[c] {
            continue;
        }
        let (x, y) = ((c % cols) as f32, (c / cols) as f32);
        let ore_bodies = world.ore[c] > ORE_MARK && world.ore[c] > 0.4 * world.coal[c] + 1e-9;
        for (on, (ox, oy), col) in [(world.coal[c] > 0.0, (0.3, 0.3), COAL), (ore_bodies, (0.7, 0.3), ORE), (world.rock[c] >= STONE_MARK, (0.3, 0.7), STONE), (world.clay[c] >= CLAY_MARK, (0.7, 0.7), CLAY)] {
            if on {
                canvas.dot((x + ox, y + oy), 0.62, col, 1.0);
            }
        }
    }
    // the towns, a dark dot in a pale ring
    for &c in sites {
        let at = ((c % cols) as f32 + 0.5, (c / cols) as f32 + 0.5);
        canvas.dot(at, 4.0, [0.98, 0.96, 0.90], 1.0);
        canvas.dot(at, 2.8, TOWN, 1.0);
    }
    canvas.rgb.iter().flat_map(|c| [to_u8(c[0]), to_u8(c[1]), to_u8(c[2]), 255]).collect()
}

fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

struct Canvas {
    rgb: Vec<Rgb>,
    sx: f32,
    sy: f32,
}

impl Canvas {
    /// A round mark of radius `r` pixels at `at` (cell units), its edge soft.
    fn dot(&mut self, at: (f32, f32), r: f32, col: Rgb, alpha: f32) {
        let (cx, cy) = (at.0 * self.sx, at.1 * self.sy);
        let (x0, x1) = ((cx - r - 1.0).floor().max(0.0) as usize, ((cx + r + 1.0).ceil() as usize).min(MAP_W - 1));
        let (y0, y1) = ((cy - r - 1.0).floor().max(0.0) as usize, ((cy + r + 1.0).ceil() as usize).min(MAP_H - 1));
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let a = (r + 0.5 - d).clamp(0.0, 1.0) * alpha;
                if a > 0.0 {
                    let p = &mut self.rgb[y * MAP_W + x];
                    *p = mix(*p, col, a);
                }
            }
        }
    }

    /// A line from `a` to `b` (cell units), `w` pixels across.
    fn line(&mut self, a: (f32, f32), b: (f32, f32), w: f32, col: Rgb, alpha: f32) {
        let (ax, ay, bx, by) = (a.0 * self.sx, a.1 * self.sy, b.0 * self.sx, b.1 * self.sy);
        let r = w * 0.5;
        let (x0, x1) = ((ax.min(bx) - r - 1.0).floor().max(0.0) as usize, ((ax.max(bx) + r + 1.0).ceil() as usize).min(MAP_W - 1));
        let (y0, y1) = ((ay.min(by) - r - 1.0).floor().max(0.0) as usize, ((ay.max(by) + r + 1.0).ceil() as usize).min(MAP_H - 1));
        let (dx, dy) = (bx - ax, by - ay);
        let len2 = (dx * dx + dy * dy).max(1e-6);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let t = (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0);
                let d = ((px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt();
                let a = (r + 0.5 - d).clamp(0.0, 1.0) * alpha;
                if a > 0.0 {
                    let p = &mut self.rgb[y * MAP_W + x];
                    // (segments meet: a pixel already as blue as this one is not made bluer)
                    let m = mix(*p, col, a);
                    if (m[2] - m[0]) > (p[2] - p[0]) {
                        *p = m;
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ the two threads

/// The two threads and what they send back.
pub struct Maker {
    sketch: Sender<(u64, MapAsk)>,
    whole: Sender<(u64, MapAsk)>,
    done: Mutex<Receiver<MapPicture>>,
}

/// How long the settings must rest before the whole island is made. While
/// somebody clicks + and - the sketches alone are made: a whole island made
/// beside the page lengthens its frames by a sixth on this machine, as any
/// one busy thread does (crates/isostate-viewer/README.md, "Island settings").
/// Putting the thread in the background (Windows' `THREAD_MODE_BACKGROUND_BEGIN`)
/// was tried and made no difference.
const REST: Duration = Duration::from_millis(1000);

impl Maker {
    pub fn start() -> Maker {
        let (done_tx, done) = channel::<MapPicture>();
        let mut senders = Vec::new();
        for whole in [false, true] {
            let (tx, rx) = channel::<(u64, MapAsk)>();
            senders.push(tx);
            let done_tx = done_tx.clone();
            let spawned = std::thread::Builder::new().name(if whole { "island map, whole" } else { "island map, sketch" }.into()).spawn(move || loop {
                let Ok(mut job) = rx.recv() else { return };
                if whole {
                    // only once the settings rest: a click every half second makes sketches alone
                    loop {
                        match rx.recv_timeout(REST) {
                            Ok(j) => job = j,
                            Err(RecvTimeoutError::Timeout) => break,
                            Err(RecvTimeoutError::Disconnected) => return,
                        }
                    }
                }
                while let Ok(j) = rx.try_recv() {
                    job = j;
                }
                // (the old island has one map, which the sketch's thread makes)
                if whole && job.1.old_land {
                    continue;
                }
                if done_tx.send(make(&job.1, job.0, whole)).is_err() {
                    return;
                }
            });
            if let Err(e) = spawned {
                eprintln!("the start page could not start a thread for its map: {e}");
            }
        }
        let whole = senders.pop().expect("two senders");
        let sketch = senders.pop().expect("two senders");
        Maker { sketch, whole, done: Mutex::new(done) }
    }

    /// Ask for the map of `ask`, numbered `turn`.
    pub fn ask(&self, turn: u64, ask: &MapAsk) {
        let _ = self.sketch.send((turn, ask.clone()));
        let _ = self.whole.send((turn, ask.clone()));
    }

    /// The maps made since last asked, oldest first.
    pub fn take(&self) -> Vec<MapPicture> {
        self.done.lock().map(|rx| rx.try_iter().collect()).unwrap_or_default()
    }
}

/// Of the map on show (its turn and whether whole) and one just made, whether
/// the new one takes its place: a newer ask always, the same ask's whole
/// island over its sketch, and never an older one.
pub fn replaces(shown: Option<(u64, bool)>, new: (u64, bool)) -> bool {
    match shown {
        None => true,
        Some((turn, whole)) => new.0 > turn || (new.0 == turn && new.1 && !whole),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_is_drawn_for_each_start_and_says_its_land() {
        let ask = MapAsk { seed: 7, old_land: false, landfall: false, island: Island::default() };
        let m = make(&ask, 1, false);
        assert!(m.fault.is_none(), "{:?}", m.fault);
        assert_eq!(m.rgba.len(), MAP_W * MAP_H * 4);
        assert_eq!(m.towns, 6);
        // island 7: about 23,000 km² of land, 135,000 people, near six a km²
        assert!((15_000.0..35_000.0).contains(&m.land_km2), "{}", m.land_km2);
        assert!((3.0..10.0).contains(&m.density()), "{}", m.density());
        // sea and land both drawn: some pixels blue, some green
        let px: Vec<&[u8]> = m.rgba.chunks(4).collect();
        assert!(px.iter().any(|p| p[2] > p[1] && p[2] > p[0]));
        assert!(px.iter().any(|p| p[1] > p[2]));
        // landfall: one town, 5,000 people
        let lf = make(&MapAsk { landfall: true, ..ask.clone() }, 2, false);
        assert_eq!((lf.towns, lf.people), (1, 5_000.0));
        // more people, the same land
        let crowded = make(&MapAsk { island: Island::parse("people=540000,towns=12").unwrap(), ..ask.clone() }, 3, false);
        assert_eq!(crowded.towns, 12);
        assert_eq!(crowded.land_km2, m.land_km2);
        assert!((crowded.density() / m.density() - 4.0).abs() < 1e-9);
        // the old island has a map too
        let old = make(&MapAsk { old_land: true, ..ask.clone() }, 4, false);
        assert!(old.fault.is_none() && old.whole && old.towns == 6, "{:?}", old.fault);
        // a setting no game can have is said, not drawn
        let bad = make(&MapAsk { island: Island::parse("rain=10").unwrap(), ..ask }, 5, false);
        assert!(bad.fault.unwrap().contains("outside its range"));
    }

    /// The maps at the settings' usual values and the ends of their ranges,
    /// each timed, sketch and whole, and each written three times as large to
    /// `ISOSTATE_MAP_OUT` (a folder) to be looked at:
    ///
    ///     ISOSTATE_MAP_OUT=out/maps cargo test -p isostate-viewer --bin isostate-viewer -- maps_at_the_ends --ignored --nocapture
    #[test]
    #[ignore]
    fn maps_at_the_ends_of_the_ranges() {
        let out = std::env::var("ISOSTATE_MAP_OUT").ok().map(std::path::PathBuf::from);
        let mut asks = vec![("usual".to_string(), Island::default())];
        for (i, s) in isostate_core::island::SETTINGS.iter().enumerate() {
            let (least, most) = isostate_core::island::range_of(i, false);
            for v in [least, most] {
                let mut isl = Island::default();
                *isl.slot(i) = Some(v);
                asks.push((format!("{}={v}", s.name), isl));
            }
        }
        for (name, island) in asks {
            let ask = MapAsk { seed: 7, old_land: false, landfall: false, island };
            let (sketch, whole) = (make(&ask, 1, false), make(&ask, 1, true));
            println!("{name:<16} sketch {:.2} s, whole {:.2} s; land {} km², {} towns, {:.1} /km²", sketch.took, whole.took, whole.land_km2, whole.towns, whole.density());
            if name == "usual" {
                // how the riches lie, to set the marks by
                let (world, _) = land_of(&params_for(&ask).unwrap(), true);
                let land: Vec<usize> = (0..world.land.len()).filter(|&c| world.land[c]).collect();
                for (what, v) in [("rock", &world.rock), ("clay", &world.clay), ("ore", &world.ore), ("coal", &world.coal)] {
                    let mut x: Vec<f64> = land.iter().map(|&c| v[c]).collect();
                    x.sort_by(f64::total_cmp);
                    let q = |p: f64| x[((x.len() - 1) as f64 * p) as usize];
                    println!("  {what}: 50% {:.2}, 75% {:.2}, 90% {:.2}, 95% {:.2}, 99% {:.2}", q(0.5), q(0.75), q(0.9), q(0.95), q(0.99));
                }
            }
            if let Some(dir) = &out {
                std::fs::create_dir_all(dir).unwrap();
                for (pic, which) in [(&sketch, "sketch"), (&whole, "whole")] {
                    let k = 3;
                    let mut big = vec![0u8; MAP_W * k * MAP_H * k * 4];
                    for y in 0..MAP_H * k {
                        for x in 0..MAP_W * k {
                            let (s, d) = (((y / k) * MAP_W + x / k) * 4, (y * MAP_W * k + x) * 4);
                            big[d..d + 4].copy_from_slice(&pic.rgba[s..s + 4]);
                        }
                    }
                    let extent = bevy::render::render_resource::Extent3d { width: (MAP_W * k) as u32, height: (MAP_H * k) as u32, depth_or_array_layers: 1 };
                    let img = bevy::image::Image::new(extent, bevy::render::render_resource::TextureDimension::D2, big, bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default());
                    img.try_into_dynamic().unwrap().save(dir.join(format!("{name}-{which}.png"))).unwrap();
                }
            }
        }
    }

    #[test]
    fn a_newer_map_replaces_an_older_and_the_whole_island_its_sketch() {
        assert!(replaces(None, (1, false)));
        assert!(replaces(Some((1, false)), (1, true)));
        assert!(replaces(Some((1, true)), (2, false)));
        assert!(!replaces(Some((2, false)), (1, true)));
        assert!(!replaces(Some((2, true)), (2, false)));
        assert!(!replaces(Some((2, true)), (2, true)));
    }
}
