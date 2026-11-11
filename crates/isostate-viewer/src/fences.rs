//! Where the families' fences run: along the hedge lines round their open
//! land.
//!
//! The simulation knows who owns each square of the map. The land is drawn
//! in parcels (the fields of fields.rs, three cuts down), and every parcel
//! belongs to the square its own point lies in, which is also what settles
//! its land class: so farm, pasture and wood change at hedge lines, and a
//! family's land is taken to end at them too. A fence runs along each edge
//! of a parcel of the family's open land (fields, pasture, wood and heath:
//! not town ground, works yards or pits) where the parcel beyond is not the
//! family's, a hair inside the hedge.
//!
//! The parcels of a square and what lies beyond each stretch of their edges
//! never change, so they are worked out once for each square that is ever
//! owned and kept; a year's fences are then a walk over the owned squares.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use bevy::math::{Vec2, Vec3};
use bevy::prelude::Resource;
use isostate_gen::fence::fence;

use crate::fields;
use crate::look::Look;
use crate::meshkit::Builder;
use crate::replay::Replay;
use crate::world::{self, Ground};

/// One side of a parcel: where the posts would stand along it, a hair
/// inside its edge (none where the ground is wet or the sea), and for each
/// stretch between two posts the square the parcel beyond it belongs to.
struct Side {
    posts: Vec<Option<Vec3>>,
    beyond: Vec<u32>,
}

/// A parcel of land: what settles its class, and its four sides in order round it.
struct Parcel {
    wcell: usize,
    sub: f32,
    sides: [Side; 4],
}

/// The parcels of each square that has been asked about.
#[derive(Resource, Default)]
pub struct Lines {
    squares: HashMap<usize, Arc<Vec<Parcel>>>,
}

/// How far inside its parcel's edge a fence stands, on the map.
const INSET: f32 = 0.012;
/// The longest stretch between two posts.
const STRETCH: f32 = 0.12;

/// The map point of a point of block space: `fields::to_block` undone, warp
/// and all, to within `near` of the true point.
fn to_map(q: Vec2, near: f32) -> Vec2 {
    let mut p = fields::from_block(q);
    // the warp is gentle, so each turn brings the point several times nearer
    for _ in 0..10 {
        let step = fields::from_block(q - fields::to_block(p));
        p += step;
        if step.length_squared() < near * near {
            break;
        }
    }
    p
}

fn square_of(g: &Ground, p: Vec2) -> usize {
    (p.y.floor().clamp(0.0, g.h as f32 - 1.0) as usize) * g.w + p.x.floor().clamp(0.0, g.w as f32 - 1.0) as usize
}

/// The parcels whose own point lies in square `c`.
fn parcels_of(g: &Ground, c: usize) -> Vec<Parcel> {
    let (cx, cy) = ((c % g.w) as f32, (c / g.w) as f32);
    let mut seen: HashSet<(i32, i32, u32)> = HashSet::new();
    let mut out = Vec::new();
    // a parcel is never narrower than a ninth of a square, and holds its own point,
    // so a grid of twenty each way finds every one that belongs here
    let n = 20;
    for j in 0..n {
        for i in 0..n {
            let p = Vec2::new(cx + (i as f32 + 0.5) / n as f32, cy + (j as f32 + 0.5) / n as f32);
            let (bl, pc, centre) = fields::parcel(p);
            if square_of(g, centre) != c || !seen.insert((bl.i, bl.j, pc.id)) {
                continue;
            }
            let (wcell, sub) = world::class_key(g, p.x, p.y);
            let at = |uv: Vec2, near: f32| to_map(fields::bilinear(bl.a, bl.b, bl.c, bl.d, uv), near);
            // the inset, and the step outward to ask what lies beyond, in the block's own (u, v)
            let lu = 0.5 * ((bl.b - bl.a).length() + (bl.c - bl.d).length()) * fields::BLOCK;
            let lv = 0.5 * ((bl.d - bl.a).length() + (bl.c - bl.b).length()) * fields::BLOCK;
            let d = Vec2::new(INSET / lu.max(1e-3), INSET / lv.max(1e-3));
            let (lo, hi) = (pc.lo, pc.hi);
            // the corners in order round the parcel, and the way out across the side that starts at each
            let corners = [Vec2::new(lo.x, lo.y), Vec2::new(hi.x, lo.y), Vec2::new(hi.x, hi.y), Vec2::new(lo.x, hi.y)];
            let outward = [Vec2::new(0.0, -1.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0), Vec2::new(-1.0, 0.0)];
            let inset = |uv: Vec2| uv.clamp(lo + d, hi - d);
            // a post stands on dry ground, clear of the rivers
            let stand = |p: Vec2| {
                let h = world::height_at(g, p.x, p.y);
                (h >= 0.03 && world::water_dist(g, p.x, p.y) >= 0.02).then(|| Vec3::new(p.x, h, p.y))
            };
            let sides = std::array::from_fn(|s| {
                let (from, to) = (corners[s], corners[(s + 1) % 4]);
                let len = (fields::bilinear(bl.a, bl.b, bl.c, bl.d, to) - fields::bilinear(bl.a, bl.b, bl.c, bl.d, from)).length() * fields::BLOCK;
                let parts = ((len / STRETCH).ceil() as usize).max(1);
                let posts = (0..=parts).map(|k| stand(at(inset(from.lerp(to, k as f32 / parts as f32)), 2e-4))).collect();
                let beyond = (0..parts)
                    .map(|k| {
                        // well inside the parcel beyond, so it need not be found so finely
                        let q = at(from.lerp(to, (k as f32 + 0.5) / parts as f32) + outward[s] * d * 2.5, 4e-3);
                        world::own_cell(g, q.x, q.y) as u32
                    })
                    .collect();
                Side { posts, beyond }
            });
            out.push(Parcel { wcell, sub, sides });
        }
    }
    out
}

/// The year's fences: round every family's open land, in a quiet colour of its own.
pub fn family_fences(b: &mut Builder, r: &Replay, g: &Ground, k: usize, lk: &Look, lines: &mut Lines, banner: &dyn Fn(usize) -> [f32; 4]) {
    if r.owner.is_empty() {
        return;
    }
    let owner = |c: usize| r.owner[k * r.n + c];
    let mut run: Vec<Vec3> = Vec::new();
    for c in 0..r.n {
        let o = owner(c);
        if o < 0 {
            continue;
        }
        let parcels = lines.squares.entry(c).or_insert_with(|| Arc::new(parcels_of(g, c))).clone();
        let colour = banner(o as usize);
        for parcel in parcels.iter() {
            if !matches!(world::pick_class(r, k, parcel.wcell, parcel.sub), world::FARM | world::PASTURE | world::WOOD | world::HEATH) {
                continue;
            }
            for side in &parcel.sides {
                for (i, &beyond) in side.beyond.iter().enumerate() {
                    // a stretch of fence where the land beyond is not the family's and both posts can stand
                    let ends = if owner(beyond as usize) == o { None } else { side.posts[i].zip(side.posts[i + 1]) };
                    match ends {
                        Some((a, e)) => {
                            if run.is_empty() {
                                run.push(a);
                            }
                            run.push(e);
                        }
                        None => {
                            fence(b, &run, colour, lk);
                            run.clear();
                        }
                    }
                }
            }
            // the sides run on into one another round the parcel
            fence(b, &run, colour, lk);
            run.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_way_back_from_block_space_is_exact() {
        for (x, y) in [(3.2f32, 7.9f32), (40.5, 12.25), (0.1, 0.1), (63.7, 55.3)] {
            let p = Vec2::new(x, y);
            let back = to_map(fields::to_block(p), 1e-5);
            assert!(back.distance(p) < 1e-4, "{p} came back as {back}");
        }
    }
}
