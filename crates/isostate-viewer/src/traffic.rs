//! What each way carries (the core's docs/rules/traffic.md), as the viewer
//! holds it and shows it: the Traffic map view's ribbons, and the lines of a
//! town's card that say which towns it trades most with.
//!
//! A run that keeps the traffic record has two layers a year: the weight over
//! each link between two neighbouring squares (a byte on a scale by ratio,
//! `isostate_core::traffic::goods_byte`) and the way most of it went (cart,
//! boat, rail or sea). A live game hands the same bytes over each year. The
//! trade between towns is the books': a recorded run's books' tables
//! (`acct_trade_*`) are summed here a pair of towns at a time with the core's
//! own `traffic::pairs`, and a live game sends the year's pairs ready summed.
//!
//! The ribbons are one mesh, laid over the ground along each link with
//! traffic from one square's meeting place to the other's, as wide as what the
//! link carried on the view's scale (`overlay::Scale`, so the three ranges of
//! the other views widen and narrow them as they tint those), in the ink of
//! the way it went. They are made again only when the year, the view or its
//! scale changes, and only while the view is shown, off the main thread
//! (`lay`), along the roads' ways the year's scenery found.

use bevy::prelude::*;
use isostate_core::traffic::{self as tf, Pair};

use crate::meshkit::{self, Builder};
use crate::overlay::{MapOverlay, Overlay, Scale};
use crate::replay::Replay;
use crate::world::{self, Ground};
use crate::Data;

/// The traffic a run keeps, `[year][cell][slot]` and `[year][cell]`; all empty
/// where the run kept none.
#[derive(Clone, Default, Debug)]
pub struct Traffic {
    /// the core's byte of the weight carried over each link
    pub goods: Vec<u8>,
    /// the way most of it went, two bits a link (`isostate_core::traffic::mode_of`)
    pub mode: Vec<u8>,
    /// each year's trade between each pair of towns, by seller and then buyer
    pub pairs: Vec<Vec<Pair>>,
}

impl Traffic {
    /// Whether the run keeps the weight on its links.
    pub fn kept(&self) -> bool {
        !self.goods.is_empty()
    }

    /// The weight a year over square `c`'s link in slot `s` in year `k`, of `n` squares.
    pub fn weight(&self, n: usize, k: usize, c: usize, s: usize) -> f32 {
        self.goods.get((k * n + c) * 4 + s).map_or(0.0, |&b| tf::byte_goods(b))
    }

    /// The way most of that weight went (`isostate_core::traffic::CART` to `SEA`).
    pub fn mode(&self, n: usize, k: usize, c: usize, s: usize) -> u8 {
        self.mode.get(k * n + c).map_or(tf::CART, |&b| tf::mode_of(b, s))
    }

    /// Write year `k` as a live game hands it over.
    pub fn take_year(&mut self, ny: usize, n: usize, k: usize, goods: &[u8], mode: &[u8]) {
        if goods.len() != n * 4 || mode.len() != n {
            return;
        }
        if self.goods.len() != ny * n * 4 {
            self.goods = vec![0; ny * n * 4];
            self.mode = vec![0; ny * n];
        }
        self.goods[k * n * 4..(k + 1) * n * 4].copy_from_slice(goods);
        self.mode[k * n..(k + 1) * n].copy_from_slice(mode);
    }

    /// Keep year `k`'s pairs.
    pub fn take_pairs(&mut self, ny: usize, k: usize, pairs: Vec<Pair>) {
        if self.pairs.len() < ny {
            self.pairs.resize(ny, Vec::new());
        }
        if let Some(p) = self.pairs.get_mut(k) {
            *p = pairs;
        }
    }
}

/// The books' flows between towns of a recorded run, summed a pair at a
/// time for each year: `off` the rows each books' year starts at (one more
/// than the years), `key` (seller, buyer, good) and `val` (quantity, worth,
/// carting) a row each, the books' first year `first` years after the run's.
/// Empty where the tables do not fit together.
pub fn pairs_of_books(off: &[i32], key: &[i32], val: &[f32], weight: &[f64], first: usize, ny: usize) -> Vec<Vec<Pair>> {
    let rows = key.len() / 3;
    if off.is_empty() || val.len() != rows * 3 || key.len() != rows * 3 {
        return Vec::new();
    }
    let mut out = vec![Vec::new(); ny];
    for b in 0..off.len() - 1 {
        let (lo, hi) = (off[b].max(0) as usize, (off[b + 1].max(0) as usize).min(rows));
        let Some(slot) = out.get_mut(first + b) else { break };
        if lo >= hi {
            continue;
        }
        let flows = (lo..hi).map(|i| (key[i * 3] as u16, key[i * 3 + 1] as u16, key[i * 3 + 2] as u16, val[i * 3], val[i * 3 + 1]));
        *slot = tf::pairs(flows, weight);
    }
    out
}

/// One of a town's partners in a year: what it sold the town and what it
/// bought from it, each its worth, weight and leading goods.
#[derive(Clone, Debug, PartialEq)]
pub struct Partner {
    pub town: usize,
    /// what the town sold the partner
    pub sold: Option<Pair>,
    /// what the town bought from the partner
    pub bought: Option<Pair>,
}

impl Partner {
    pub fn worth(&self) -> f32 {
        self.sold.map_or(0.0, |p| p.value) + self.bought.map_or(0.0, |p| p.value)
    }
}

/// Town `t`'s largest partners in year `k`, by the worth of the trade both
/// ways, at most `most` of them.
pub fn partners(r: &Replay, k: usize, t: usize, most: usize) -> Vec<Partner> {
    let Some(pairs) = r.traffic.pairs.get(k) else { return Vec::new() };
    let mut by: Vec<Partner> = Vec::new();
    for p in pairs {
        let (other, sold) = if p.from as usize == t {
            (p.to as usize, true)
        } else if p.to as usize == t {
            (p.from as usize, false)
        } else {
            continue;
        };
        let i = match by.iter().position(|q| q.town == other) {
            Some(i) => i,
            None => {
                by.push(Partner { town: other, sold: None, bought: None });
                by.len() - 1
            }
        };
        if sold {
            by[i].sold = Some(*p);
        } else {
            by[i].bought = Some(*p);
        }
    }
    by.sort_by(|a, b| b.worth().total_cmp(&a.worth()).then(a.town.cmp(&b.town)));
    by.truncate(most);
    by
}

/// How many partners a town's card names.
pub const PARTNERS: usize = 5;

fn goods_words(r: &Replay, p: &Pair) -> String {
    let names: Vec<String> = p.lead.iter().filter(|&&g| g != tf::NO_GOOD).filter_map(|&g| r.header.goods.get(g as usize)).map(|g| g.label.to_lowercase()).collect();
    names.join(", ")
}

fn worth(v: f32) -> String {
    format!("\u{00a3}{}", crate::ui::thousands(crate::overlay::round_sig(v).max(v.min(1.0))))
}

/// The lines a town's card ends with: "Trades most with, in 1900", then each
/// of its five largest partners with what goes each way. None where the run
/// kept no books, or the town traded with nobody that year.
pub fn trade_lines(r: &Replay, k: usize, t: usize) -> Option<String> {
    let ps = partners(r, k, t, PARTNERS);
    if ps.is_empty() {
        return None;
    }
    let year = r.y0 + k as i32;
    let mut s = format!("Trades most with, in {year} (the year's worth at the seller's prices; weight in the goods' units):");
    for p in &ps {
        let name = r.header.towns.get(p.town).map_or("another town", |t| t.name.as_str());
        let side = |q: &Option<Pair>, verb: &str| -> Option<String> {
            q.filter(|q| q.value >= 0.5).map(|q| format!("{verb} {} of {} ({} by weight)", worth(q.value), goods_words(r, &q), crate::ui::thousands(q.weight)))
        };
        let parts: Vec<String> = [side(&p.sold, "sells it"), side(&p.bought, "buys from it")].into_iter().flatten().collect();
        if !parts.is_empty() {
            s.push_str(&format!("\n{name}: {}.", parts.join("; ")));
        }
    }
    Some(s)
}

// ------------------------------------------------------------------ the carts

/// The weight carted along a carters' round (`trains::road_routes`) in year
/// `k`: the mean, over the links between squares it passes, of what each
/// carried by cart (nothing where the link's weight went by rail or water).
/// The links are the round's own (`TrainPath::cells`); a path that does not
/// say them is read by the squares its points lie in.
pub fn cart_weight_along(r: &Replay, k: usize, p: &crate::trains::TrainPath) -> f32 {
    let mut cells: Vec<usize> = p.cells.clone();
    for q in p.pts.iter().filter(|_| p.cells.is_empty()) {
        if q.x < 0.0 || q.z < 0.0 || q.x as usize >= r.w || q.z as usize >= r.h {
            continue;
        }
        let c = q.z as usize * r.w + q.x as usize;
        if cells.last() != Some(&c) {
            cells.push(c);
        }
    }
    let (mut sum, mut links) = (0.0f32, 0usize);
    for s in cells.windows(2) {
        if let Some((c, slot)) = tf::link(r.w, s[0], s[1]) {
            links += 1;
            if r.traffic.mode(r.n, k, c, slot) == tf::CART {
                sum += r.traffic.weight(r.n, k, c, slot);
            }
        }
    }
    if links == 0 { 0.0 } else { sum / links as f32 }
}

/// How many carts each round has, from what is carted along it: one on every
/// round (so that a road the railway took the freight from is not empty),
/// two on the busier rounds and three on the busiest, by their rank among
/// the year's rounds that carry anything by cart: the lower 40 per cent one,
/// the next 35 two, the top quarter three: 1.85 a round where there is
/// carting, against 1.35 drawn by chance before, and they stand where it is.
pub fn carts_by_weight(w: &[f32]) -> Vec<usize> {
    let mut busy: Vec<usize> = (0..w.len()).filter(|&i| w[i] > 0.0).collect();
    busy.sort_by(|&a, &b| w[a].total_cmp(&w[b]).then(a.cmp(&b)));
    let mut out = vec![1; w.len()];
    let n = busy.len() as f32;
    for (rank, &i) in busy.iter().enumerate() {
        let at = rank as f32 / n.max(1.0);
        out[i] = if at >= 0.75 { 3 } else if at >= 0.4 { 2 } else { 1 };
    }
    out
}

// ------------------------------------------------------------------ the ribbons

/// The narrowest and the widest ribbon, in squares (a turnpike's ribbon is
/// 0.034), seen over the whole island (`FAR`). Between them a ribbon widens
/// as the square of where its weight stands on the scale, so that the few
/// busiest ways stand out from the many middling ones, as main lines do on a
/// map of a country's traffic. Nearer, every ribbon is narrower (`nearer`),
/// so that a town is not under them.
pub const NARROW: f32 = 0.018;
pub const WIDE: f32 = 0.42;
/// The camera's distance the widths are for, in squares: the whole island.
const FAR: f32 = 134.0;
/// Ways carrying less than this part of the scale's low end are not drawn at all.
const FAINT: f32 = 1.0 / 30.0;

/// How much narrower the ribbons are drawn with the camera `dist` squares
/// off: by the distance to the power 0.6, in steps of half a doubling (so
/// that they are laid again only a few times in a zoom), never wider than
/// over the whole island. Returns the step and the factor.
pub fn nearer(dist: f32) -> (i32, f32) {
    let step = (2.0 * (dist.max(1.0) / FAR).log2()).round().clamp(-14.0, 0.0) as i32;
    (step, 2f32.powf(0.6 * step as f32 / 2.0))
}
/// How far above the ground a ribbon lies: over the roads' ribbons.
const LIFT: f32 = 0.022;
/// How often along a link a ribbon takes the ground's height, in squares.
const STEP: f32 = 0.2;

/// The ink of each way a load goes: a cart's in brown, a boat's in river
/// blue, a train's in coal black, a ship's in a paler sea blue.
pub const INKS: [u32; 4] = [0x8a4f2a, 0x2f6d93, 0x2e2b2a, 0x6f93b0];
/// The paper the narrowest ribbons fade toward.
const PAPER: u32 = 0xe6dccb;

/// The ink `t` of the way from `a` to `b`, mixed as the eye sees them (in sRGB).
fn mix(a: u32, b: u32, t: f32) -> [f32; 4] {
    let ch = |h: u32, s: u32| ((h >> s) & 0xff) as f32;
    let f = |s: u32| ((ch(a, s) + (ch(b, s) - ch(a, s)) * t.clamp(0.0, 1.0)).round() as u32) << s;
    meshkit::hex(f(16) | f(8) | f(0))
}

/// The ribbon's middle line from `a` to `b`, at the ground's height (never below the sea's) plus `LIFT`.
fn line(g: &Ground, a: (f32, f32), b: (f32, f32)) -> Vec<Vec3> {
    let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    let n = ((len / STEP).ceil() as usize).max(1);
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            Vec3::new(x, world::height_at(g, x, z).max(0.0) + LIFT, z)
        })
        .collect()
}

/// The links of year `k` drawn on the scale: (square, slot, weight, mode),
/// heaviest last so that they lie on top.
///
/// A link carted along between two squares of made road is drawn only where
/// the map draws the road (`network::drawn_links`), so that no ribbon runs
/// where there is no road under it: what the year's carting took over a
/// link the carts had not yet worn is left out (on island 7, 6.7 per cent
/// of the carting's weight by length in 1850, 0.5 in 1900, 0.1 in 2050), and
/// what is drawn follows the road's own way (`network::route`). Carting over
/// tracks and open ground, which the map has never drawn as a road, is
/// drawn as before.
pub fn drawn_links(r: &Replay, worn: &crate::network::Worn, k: usize, sc: &Scale) -> Vec<(usize, usize, f32, u8)> {
    let mut out = Vec::new();
    if !r.traffic.kept() {
        return out;
    }
    let roads: std::collections::HashSet<(usize, usize)> = crate::network::drawn_links(r, worn, k).into_iter().map(|(c, m, _)| (c, m)).collect();
    // (with every road drawn, `ISOSTATE_WORN=off`, every link is drawn, as before)
    let made = |c: usize| crate::network::by_wear() && r.trans[k * r.n + c] & 3 >= 2;
    for c in 0..r.n {
        for s in 0..4 {
            let w = r.traffic.weight(r.n, k, c, s);
            if w <= 0.0 || w < sc.lo * FAINT {
                continue;
            }
            if let Some(d) = tf::other_end(r.w, r.h, c, s) {
                let mode = r.traffic.mode(r.n, k, c, s);
                if mode == tf::CART && made(c) && made(d) && !roads.contains(&(c, d)) {
                    continue;
                }
                out.push((c, s, w, mode));
            }
        }
    }
    out.sort_by(|a, b| a.2.total_cmp(&b.2));
    out
}

/// The ribbons of year `k` on scale `sc`, `near` times as wide as over the whole island.
pub fn ribbons(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, sc: &Scale, near: f32) -> Builder {
    let mut b = Builder::default();
    let made = |c: usize| crate::network::by_wear() && r.trans[k * r.n + c] & 3 >= 2;
    for (i, (c, s, w, m)) in drawn_links(r, &g.worn, k, sc).into_iter().enumerate() {
        let Some(d) = tf::other_end(r.w, r.h, c, s) else { continue };
        let t = sc.at(w);
        let width = (NARROW + (WIDE - NARROW) * t * t) * near;
        let col = mix(PAPER, INKS[m as usize & 3], 0.35 + 0.65 * t);
        // carting on a made road follows the road's own course (its curve, or round a meander or a
        // lake, as `network::course` draws it)
        let mut pts = if m == tf::CART && made(c) && made(d) {
            let way = crate::network::course(r, g, cen, c, d, k);
            way.pts.windows(2).enumerate().flat_map(|(j, p)| {
                let seg = line(g, (p[0].x, p[0].y), (p[1].x, p[1].y));
                // (each length after the first begins where the last ended)
                seg.into_iter().skip(if j == 0 { 0 } else { 1 })
            }).collect()
        } else {
            line(g, cen[c], cen[d])
        };
        // a hair higher for each one laid, so the heavier lie over the lighter where they meet
        let up = 1e-6 * i as f32;
        pts.iter_mut().for_each(|p| p.y += up);
        b.ribbon(&pts, width, col);
        // highways (the core's docs/rules/highways.md): a link between two squares of
        // trunk road (the record's 64) has a dark line down its middle, a bypass's (128) a red one
        let (tc, td) = (r.trans[k * r.n + c], r.trans[k * r.n + d]);
        if tc & 64 != 0 && td & 64 != 0 {
            let bypass = tc & 128 != 0 || td & 128 != 0;
            pts.iter_mut().for_each(|p| p.y += 1e-4);
            b.ribbon(&pts, 0.3 * width, meshkit::hex(if bypass { 0xb3261e } else { 0x262626 }));
        }
    }
    b
}

/// The ribbons' entity.
#[derive(Component)]
pub(crate) struct Ribbons;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup).add_systems(Update, lay.after(crate::overlay::paint));
}

pub(crate) fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mat = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, ..default() });
    commands.spawn((Mesh3d(meshes.add(meshkit::into_mesh(Builder::default()))), MeshMaterial3d(mat), Transform::default(), Visibility::Hidden, bevy::light::NotShadowCaster, Ribbons));
}

/// Lay the ribbons again when the view, the year or the scale changes; hide
/// them when another view or the plain map is shown. Nothing is worked out
/// while the view is not shown.
///
/// The ribbons are of the year whose roads are on screen (`carts::Roads::year`),
/// which is the clock's but for the second or two a new year's scenery takes
/// to be built: so they lie on the roads drawn, and every course they follow
/// was found with that year's scenery, off the main thread
/// (`road_course::Courses::find_year`), and is only read here. They are laid
/// off the main thread too (20 ms or more for a year, which the frame the
/// view opened or the year turned in once waited for), and put on screen
/// when laid; until then the ribbons laid last stay, if they are of the same
/// year, and none are shown if not.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lay(ov: Res<MapOverlay>, roads: Res<crate::carts::Roads>, data: Res<Data>, orbit: Res<crate::Orbit>, mut meshes: ResMut<Assets<Mesh>>, mut q: Query<(&Mesh3d, &mut Visibility), With<Ribbons>>, hand: Option<Res<crate::handover::HandOver>>, mut laying: Local<Laying>) {
    let _frames = crate::frames::span(crate::frames::Sys::MapView);
    let Ok((mesh, mut vis)) = q.single_mut() else { return };
    let shown = ov.kind == Some(Overlay::Traffic) && data.r.traffic.kept();
    let (Some(sc), Some(k)) = (ov.scale.filter(|_| shown), roads.year()) else {
        vis.set_if_neq(Visibility::Hidden);
        (laying.asked, laying.task) = (None, None);
        return;
    };
    let r = &data.r;
    let k = k.min(r.ny - 1);
    let (step, near) = nearer(orbit.dist);
    let key = (k, r.filled, [sc.lo.to_bits(), sc.hi.to_bits()], step);
    if laying.asked != Some(key) {
        laying.asked = Some(key);
        let (r, g, cen) = (data.r.clone(), data.g.clone(), data.cen.clone());
        let laid = async move { meshkit::into_mesh(world::timed("year traffic", || ribbons(&r, &g, &cen, k, &sc, near))) };
        if hand.is_some_and(|h| !h.in_pieces()) {
            // (pictures, which take a year whole and must not be taken before the ribbons are
            // laid, lay them here, as before)
            if let Some(mut old) = meshes.get_mut(&mesh.0) {
                *old = bevy::tasks::block_on(laid);
            }
            (laying.task, laying.drawn) = (None, Some(key));
        } else {
            laying.task = Some((key, bevy::tasks::AsyncComputeTaskPool::get().spawn(laid)));
        }
    }
    let asked = laying.asked;
    let ready = laying.task.as_mut().filter(|(at, _)| Some(*at) == asked).and_then(|(at, t)| bevy::tasks::futures::check_ready(t).map(|m| (*at, m)));
    if let Some((at, m)) = ready {
        laying.task = None;
        if let Some(mut old) = meshes.get_mut(&mesh.0) {
            *old = m;
        }
        laying.drawn = Some(at);
    }
    vis.set_if_neq(if laying.drawn.is_some_and(|d| d.0 == k) { Visibility::Inherited } else { Visibility::Hidden });
}

/// What `lay` keeps from frame to frame: the ribbons asked for (their year,
/// the run's years, the scale and the camera's step), those being laid, and
/// those on screen.
#[derive(Default)]
pub(crate) struct Laying {
    asked: Option<(usize, usize, [u32; 2], i32)>,
    task: Option<((usize, usize, [u32; 2], i32), bevy::tasks::Task<Mesh>)>,
    drawn: Option<(usize, usize, [u32; 2], i32)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_busiest_rounds_have_the_most_carts_and_every_round_has_one() {
        let w = [0.0, 5.0, 1.0, 9.0, 3.0, 0.0, 7.0, 2.0];
        let n = carts_by_weight(&w);
        assert_eq!(n, vec![1, 2, 1, 3, 1, 1, 2, 1]);
        assert!(carts_by_weight(&[]).is_empty());
    }

    #[test]
    fn the_books_flows_are_summed_a_pair_at_a_time_in_their_years() {
        // two books' years starting the run's second year: three flows and then one
        let off = [0, 3, 4];
        let key = [0, 1, 0, 0, 1, 1, 1, 0, 0, 2, 0, 1];
        let val = [10.0, 5.0, 0.1, 2.0, 30.0, 0.2, 4.0, 2.0, 0.0, 1.0, 1.0, 0.0];
        let p = pairs_of_books(&off, &key, &val, &[1.0, 3.0], 1, 4);
        assert_eq!(p.len(), 4);
        assert!(p[0].is_empty() && p[3].is_empty());
        assert_eq!(p[1].len(), 2);
        assert_eq!((p[1][0].from, p[1][0].to, p[1][0].value, p[1][0].weight, p[1][0].lead[0]), (0, 1, 35.0, 16.0, 1));
        assert_eq!((p[2][0].from, p[2][0].to), (2, 0));
        // a table that does not fit together gives nothing
        assert!(pairs_of_books(&off, &key[..5], &val, &[1.0], 0, 4).is_empty());
    }
}
