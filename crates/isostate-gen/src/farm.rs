//! Farmsteads among the fields.

use glam::Vec3;

use crate::colour::{pick, shade, Rgba};
use crate::hash::Seed;
use crate::houses::roof::Cover;
use crate::houses::{farmhouse, Detail, Farmhouse};
use crate::kit::{End, Kit, Open, Opening, Over, Range, RoofSpec, Side, Wall, FOOT};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::STOREY;
use crate::site::Site;
use crate::trees::{hedge_tree, Greenery};

/// A farmstead to draw. The site is the farmhouse; the yard and the barn lie behind it (toward -z).
#[derive(Clone, Copy, Debug)]
pub struct Farmstead {
    /// it grows corn (and so has a rick and a granary) rather than keeping stock on grass
    pub arable: bool,
    /// it stands in the hills: stone and slate in place of wash, brick, tile and thatch
    pub upland: bool,
    /// how much of it to draw
    pub detail: Detail,
    pub seed: Seed,
}

/// What a farm building is built of.
#[derive(Clone, Copy)]
struct Fabric {
    wall: Wall,
    /// boarded on a frame, not walled: no slits, the posts of the frame instead
    boarded: bool,
    roof: Rgba,
    cover: Cover,
}

/// A farmstead: the house with its back to the hedge and its door to the
/// yard, the threshing barn across the yard, and on the bigger farms a range
/// down one side (a cart shed, open to the yard, where corn is grown; a
/// byre where stock is kept); a rick and a granary on staddle stones by an
/// arable farm, and a tree or two. Stone and slate in the hills, brick or
/// tarred boards, tile and thatch in the lowland. The house is a
/// `houses::farmhouse`; the bigger farms' has a second storey and a wing.
///
/// From far off the buildings are walls and roofs. At the distance the game
/// is played from the barn has its porch and great doors and the slits that
/// air the corn, the cart shed its open bays. Close to the doors are let
/// into the walls, the granary stands on its stones with steps up to its
/// door, and the rick has its thatch.
pub fn farmstead(solid: &mut MeshBuf, smoke: &mut MeshBuf, green: &mut Greenery, site: &Site, f: &Farmstead, pal: &Palette) {
    let Seed { u, v } = f.seed;
    let yaw = site.yaw;
    let (frame, sill) = (*pal.timber.door, *pal.stone.sill);
    let boarded = !f.upland && u < 0.2;
    let col = if f.upland { *pal.stone.upland_barn } else if boarded { *pal.timber.barn_tarred } else { *pal.brick.barn };
    let fabric = Fabric {
        wall: Wall { col, plinth: if boarded { Some(*pal.brick.barn) } else { None }, frame, sill },
        boarded,
        roof: if f.upland { *pal.slate.upland_barn } else { *pal.roofs.barn },
        cover: if f.upland { Cover::Slate } else { Cover::Tile },
    };
    // the yard: beaten earth between the buildings
    let yc = site.at(0.0, -0.045);
    solid.block(yc - Vec3::Y * 0.004, 0.085, 0.055, 0.0055, yaw, *pal.ground.farmyard_edge, Some(*pal.ground.farmyard), 0.01);
    // the house looks into its yard
    let big = v > 0.55;
    let house = Farmhouse { storeys: 1 + big as u8, upland: f.upland, thatch: v < 0.45, wing: big, detail: f.detail, seed: f.seed };
    farmhouse(solid, smoke, &site.moved(site.origin, yaw + std::f32::consts::PI), &house, pal);
    // the barn across the yard, long and tall, its great doors to the yard
    let doors = shade(if f.upland { *pal.timber.door } else { pick(&[pal.timber.door, pal.timber.boards], u * 3.0) }, 0.9);
    barn(solid, &site.moved(site.at(-0.005, -0.09), yaw), &fabric, doors, f.detail, f.seed, pal);
    // a cart shed or a byre closing one side of the bigger yards, its front to the yard
    let side = if u < 0.16 { 1.0 } else { -1.0 };
    if big {
        side_range(solid, &site.moved(site.at(0.052 * side, -0.046), yaw - side * std::f32::consts::FRAC_PI_2), &fabric, !f.arable, f.detail, pal);
    }
    if f.arable {
        // a rick of corn or hay, and the granary
        rick(solid, site.at(-0.06 * side, -0.03), f.detail, pal);
        granary(solid, &site.moved(site.at(-0.058 * side, -0.062), yaw), &fabric, f.detail, pal);
    }
    // a tree by the house
    if v < 0.6 {
        hedge_tree(green, &site.moved(site.at(0.05 * -side, 0.03), 0.0), 0.7 + 0.2 * u, v, yaw, pal);
    }
}

/// The posts of a boarded frame on a side of a range, a bay apart, and close
/// to the joints of its boards, clear of the openings in that side.
fn boarding(k: &mut Kit, rg: &Range, side: usize, y0: f32, y1: f32, ops: &[Opening], col: Rgba) {
    // (posts and joints are a hair wide: they are drawn close to and not from the play distance)
    if !k.near() {
        return;
    }
    let len = rg.face(side).len;
    let cols = ((len / (1.25 * crate::scale::BAY)).round() as usize).max(1);
    let t = if k.near() { 0.0008 } else { 0.0013 };
    let clear = |u: f32, t: f32| ops.iter().all(|o| (o.u - u).abs() > o.w * 0.5 + t);
    for c in 0..=cols {
        let u = (len * c as f32 / cols as f32).clamp(t, len - t);
        if clear(u, t) {
            k.strip(rg, side, u - t, u + t, y0, y1 - 0.0014, 0.0006, shade(col, 0.8));
        }
        if k.near() && c < cols {
            for j in 1..5 {
                let u = len * (c as f32 + j as f32 * 0.2) / cols as f32;
                if clear(u, 0.0004) {
                    k.strip(rg, side, u - 0.0002, u + 0.0002, y0 + FOOT, y1 - 0.0014, 0.0003, shade(col, 0.9));
                }
            }
        }
    }
}

/// A threshing barn: a hall a storey and a half high under a steep roof,
/// the great doors of its threshing floor in a porch on the yard side and
/// facing them at the back, slits to air the corn in every bay (a boarded
/// barn has the posts of its frame instead), a man's door at one end and a
/// pitching hole high in each gable. It is old, and its ridge has settled.
fn barn(solid: &mut MeshBuf, site: &Site, fab: &Fabric, doors: Rgba, detail: Detail, seed: Seed, pal: &Palette) {
    let mut k = Kit::new(solid, site, pal, detail, 0.0);
    let (len, dep, h) = (0.08, 0.036, 1.5 * STOREY);
    let (hl, hd) = (len * 0.5, dep * 0.5);
    let lie = fab.cover.lie();
    let rise = dep * lie.rise * 0.8;
    let rg = Range::rect(-hl, hl, -hd, hd);
    k.crooked(((seed.pick(21) - 0.5) * 0.02, 0.0), 0.0014 + 0.001 * seed.pick(22), hl + lie.verge, h, h + rise);
    let soot = *pal.smoke.soot;
    let slit = |u: f32| Opening { u, y: FOOT + 0.006, w: 0.0014, h: 0.0076, kind: Open::Mouth { within: soot, round: false } };
    let great = Open::Leaves { paint: doors, round: false };
    let cell = len / 5.0;
    let (mut front, mut back) = (Vec::new(), Vec::new());
    // (the slits are a hair wide: they are drawn close to and not from the play distance)
    if !fab.boarded && k.near() {
        for c in [0usize, 1, 3] {
            front.push(slit(cell * (c as f32 + 0.5)));
        }
        for c in [0usize, 1, 3, 4] {
            back.push(slit(cell * (c as f32 + 0.5)));
        }
    }
    // a man's door at the end of the front, and the doors of the threshing floor at the back
    front.push(Opening { u: cell * 4.5, y: FOOT, w: 0.0052, h: 0.0124, kind: Open::Door { paint: doors, over: Over::Nothing } });
    back.push(Opening { u: hl, y: FOOT, w: 0.0125, h: 0.0172, kind: great });
    let y0 = -k.drop(&rg);
    k.walls(&rg, y0, h, &fab.wall, [Side::free(&front), Side::PLAIN, Side::free(&back), Side::PLAIN], true, true);
    k.roof(&rg, &RoofSpec { h, rise, eaves: lie.eaves, verge: lie.verge, ends: [End::Gable, End::Gable], col: fab.roof, gable: fab.wall.col, thick: lie.thick });
    if fab.boarded {
        for (i, ops) in [(0, &front), (2, &back)] {
            boarding(&mut k, &rg, i, 0.0, h, ops, fab.wall.col);
        }
        for i in [1, 3] {
            boarding(&mut k, &rg, i, 0.0, h, &[], fab.wall.col);
        }
    }
    if k.far() {
        return;
    }
    // the pitching hole high in each gable, shut
    if k.near() {
        for i in [1usize, 3] {
            k.strip(&rg, i, hd - 0.0024, hd + 0.0024, h - 0.0052, h + 0.0004, 0.0007, shade(doors, 0.85));
        }
    }
    // the porch of the threshing floor: a gabled range before the middle of the front, the
    // great doors in its face, its roof running back into the barn's
    let (pw, pd) = (0.0165, 0.0075);
    let porch = Range::rect(-pw * 0.5, pw * 0.5, hd, hd + pd);
    let way_in = [Opening { u: pw * 0.5, y: FOOT, w: 0.0125, h: 0.0172, kind: great }];
    k.walls(&porch, -k.drop(&porch), h, &fab.wall, [Side::free(&way_in), Side::PLAIN.dark(true, false), Side::NONE, Side::PLAIN.dark(false, true)], true, true);
    let porch_rise = (pw * 0.5 * rise / hd).min(rise * 0.85);
    let run = hd * porch_rise / rise;
    let over = Range { cx: 0.0, cz: hd + pd * 0.5, len: pd, dep: pw, turn: 3 };
    k.roof(&over, &RoofSpec { h, rise: porch_rise, eaves: lie.eaves * 0.8, verge: lie.verge, ends: [End::Gable, End::Valley(run)], col: fab.roof, gable: fab.wall.col, thick: lie.thick });
}

/// The range down the side of a bigger yard, a storey high: a cart shed,
/// its bays open to the yard between piers, or (`byre`) a byre with its
/// doors and its slatted windows.
fn side_range(solid: &mut MeshBuf, site: &Site, fab: &Fabric, byre: bool, detail: Detail, pal: &Palette) {
    let mut k = Kit::new(solid, site, pal, detail, 0.0);
    let (len, dep, h) = (0.05, 0.026, STOREY);
    let rg = Range::rect(-len * 0.5, len * 0.5, -dep * 0.5, dep * 0.5);
    let lie = fab.cover.lie();
    let soot = *pal.smoke.soot;
    let door = Open::Door { paint: fab.wall.frame, over: Over::Nothing };
    let cell = len / 3.0;
    let front: Vec<Opening> = if byre {
        // a door and a slatted window to each standing
        (0..3).flat_map(|c| [Opening { u: cell * (c as f32 + 0.32), y: FOOT, w: 0.0052, h: 0.0124, kind: door }, Opening { u: cell * (c as f32 + 0.74), y: FOOT + 0.0062, w: 0.005, h: 0.0052, kind: Open::Louvre }]).collect()
    } else {
        (0..3).map(|c| Opening { u: cell * (c as f32 + 0.5), y: FOOT - 0.0026, w: cell - 0.0034, h: h - 0.0052, kind: Open::Mouth { within: soot, round: false } }).collect()
    };
    k.walls(&rg, -k.drop(&rg), h, &fab.wall, [Side::free(&front), Side::PLAIN, Side::PLAIN, Side::PLAIN], true, true);
    k.roof(&rg, &RoofSpec { h, rise: dep * lie.rise * 0.8, eaves: lie.eaves, verge: lie.verge, ends: [End::Gable, End::Gable], col: fab.roof, gable: fab.wall.col, thick: lie.thick });
    if fab.boarded {
        boarding(&mut k, &rg, 0, 0.0, h, &front, fab.wall.col);
        for i in 1..4 {
            boarding(&mut k, &rg, i, 0.0, h, &[], fab.wall.col);
        }
    }
}

/// A granary: a small boarded store raised on staddle stones out of the
/// reach of rats, a door in its end and steps up to it.
fn granary(solid: &mut MeshBuf, site: &Site, fab: &Fabric, detail: Detail, pal: &Palette) {
    let mut k = Kit::new(solid, site, pal, detail, 0.0);
    let (len, dep, floor, h) = (0.016, 0.0125, 0.0042, 0.0042 + 0.6 * STOREY);
    let (hl, hd) = (len * 0.5, dep * 0.5);
    let rg = Range::rect(-hl, hl, -hd, hd);
    let boards = *pal.timber.boards;
    let wall = Wall { col: boards, plinth: None, frame: fab.wall.frame, sill: fab.wall.sill };
    let lie = fab.cover.lie();
    let door = [Opening { u: hd, y: floor + 0.0006, w: 0.0046, h: 0.0092, kind: Open::Door { paint: fab.wall.frame, over: Over::Nothing } }];
    let stone = *pal.stone.sill;
    if !k.near() {
        // from the play distance and beyond a box and its roof
        k.walls(&rg, -k.drop(&rg), h, &wall, [Side::PLAIN; 4], true, true);
    } else {
        // the store on its stones, the dark under it
        k.walls(&rg, floor, h, &wall, [Side::PLAIN, Side::free(&door), Side::PLAIN, Side::PLAIN], false, true);
        k.quad([k.pt(-hl, floor, hd), k.pt(hl, floor, hd), k.pt(hl, floor, -hd), k.pt(-hl, floor, -hd)], shade(*pal.smoke.soot, 1.4), -Vec3::Y);
        if k.near() {
            for i in 0..4 {
                boarding(&mut k, &rg, i, floor, h, if i == 1 { &door[..] } else { &[][..] }, boards);
            }
            for (x, z) in [(-1.0f32, -1.0f32), (0.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (0.0, 1.0), (1.0, 1.0)] {
                let p = k.pt(x * (hl - 0.0016), k.ground(x * hl, z * hd), z * (hd - 0.0016));
                k.solid.frustum_turned(p, 0.0011, 0.0007, floor - 0.0009, 5, 0.3, 0.003, stone, None);
                k.solid.frustum_turned(p + Vec3::Y * (floor - 0.0009), 0.0017, 0.0017, 0.0009, 5, 0.3, 0.0, shade(stone, 0.92), Some(stone));
            }
            // the steps up to the door
            k.boxy_on(hl + 0.0002, hl + 0.0026, -0.0026, 0.0026, floor * 0.5, shade(stone, 0.9), Some(stone));
            k.boxy_on(hl + 0.0026, hl + 0.005, -0.0026, 0.0026, floor * 0.25, shade(stone, 0.9), Some(stone));
        }
    }
    k.roof(&rg, &RoofSpec { h, rise: dep * lie.rise * 0.8, eaves: lie.eaves * 0.8, verge: lie.verge, ends: [End::Gable, End::Gable], col: fab.roof, gable: boards, thick: lie.thick });
}

/// A rick of corn or hay: round, under a cap of thatch. From far off a cone.
fn rick(solid: &mut MeshBuf, at: Vec3, detail: Detail, pal: &Palette) {
    let straw = *pal.thatch.rick;
    match detail {
        Detail::Far => solid.frustum(at, 0.013, 0.0, 0.024, 5, straw, None),
        Detail::Play => solid.frustum(at, 0.013, 0.0, 0.024, 6, straw, None),
        Detail::Near => {
            let sides = 8;
            solid.frustum_turned(at, 0.0115, 0.0128, 0.011, sides, 0.2, 0.02, shade(straw, 0.92), None);
            solid.frustum_turned(at + Vec3::Y * 0.0105, 0.0142, 0.0, 0.0135, sides, 0.2, 0.0, straw, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::Model;

    fn at(arable: bool, upland: bool, detail: Detail, seed: Seed) -> Model {
        let mut m = Model::default();
        farmstead(&mut m.solid, &mut m.smoke, &mut m.green, &Site::level(Vec3::ZERO, 0.0), &Farmstead { arable, upland, detail, seed }, &Palette::standard());
        assert!(m.is_sound());
        m
    }

    fn build(arable: bool, upland: bool, seed: Seed) -> Model {
        at(arable, upland, Detail::Near, seed)
    }

    #[test]
    fn the_yard_and_the_barn_lie_behind_the_house() {
        for n in 0..30 {
            for detail in Detail::ALL {
                let m = at(true, false, detail, Seed::of(n));
                let (lo, hi) = m.solid.bounds().unwrap();
                // the house is at the origin; everything else is toward -z
                assert!(lo.z < -0.1 && hi.z < 0.03, "seed {n}: from {} to {}", lo.z, hi.z);
                assert!(hi.x - lo.x < 0.2);
            }
        }
    }

    #[test]
    fn hill_farms_are_stone_and_slate() {
        let pal = Palette::standard();
        let seed = Seed::new(0.5, 0.3);
        let (low, high) = (build(false, false, seed), build(false, true, seed));
        assert!(high.solid.col.contains(&*pal.slate.upland_farm) && high.solid.col.contains(&*pal.slate.upland_barn));
        assert!(!low.solid.col.contains(&*pal.slate.upland_farm));
        assert!(low.solid.col.contains(&*pal.thatch.farmhouse));
    }

    #[test]
    fn an_arable_farm_has_a_rick_and_a_granary() {
        let seed = Seed::new(0.5, 0.3);
        for detail in Detail::ALL {
            let (corn, grass) = (at(true, false, detail, seed), at(false, false, detail, seed));
            // a cone and a box under a roof at the least
            assert!(corn.solid.triangles() >= grass.solid.triangles() + 5 + 14, "{detail:?}");
            // they stand to one side of the yard, beyond where a farm of stock has anything
            let reach = |m: &Model| m.solid.pos.iter().filter(|p| p[2] < -0.02 && p[2] > -0.07).map(|p| p[0].abs()).fold(0.0f32, f32::max);
            assert!(reach(&corn) > reach(&grass) + 0.01, "{detail:?}");
        }
    }

    #[test]
    fn the_bigger_farms_have_a_second_storey_and_a_side_range_and_the_smaller_a_tree() {
        let (small, big) = (build(false, false, Seed::new(0.5, 0.3)), build(false, false, Seed::new(0.5, 0.7)));
        assert!(big.solid.triangles() > small.solid.triangles());
        let house_top = |m: &Model| m.solid.pos.iter().filter(|p| p[2] > -0.02).map(|p| p[1]).fold(f32::MIN, f32::max);
        assert!(house_top(&big) > house_top(&small) + 0.01);
        assert!(!small.green.leaves.is_empty() && !small.green.bare.is_empty());
        assert!(big.green.is_empty());
    }

    #[test]
    fn far_off_a_farmstead_is_a_few_plain_buildings() {
        for n in 0..20 {
            let (far, play, near) = (at(true, false, Detail::Far, Seed::of(n)), at(true, false, Detail::Play, Seed::of(n)), at(true, false, Detail::Near, Seed::of(n)));
            assert!(far.solid.triangles() <= 110 && far.smoke.is_empty(), "seed {n}: {}", far.solid.triangles());
            assert!(far.solid.triangles() < play.solid.triangles() && play.solid.triangles() < near.solid.triangles());
        }
    }

    #[test]
    fn a_barn_is_a_storey_and_a_half_to_its_eaves_under_a_steep_roof() {
        let pal = Palette::standard();
        let fab = Fabric { wall: Wall { col: *pal.brick.barn, plinth: None, frame: *pal.timber.door, sill: *pal.stone.sill }, boarded: false, roof: *pal.roofs.barn, cover: Cover::Tile };
        for detail in Detail::ALL {
            let mut b = MeshBuf::default();
            barn(&mut b, &Site::level(Vec3::ZERO, 0.0), &fab, *pal.timber.door, detail, Seed::of(1), &pal);
            assert!(b.is_sound());
            let (lo, hi) = b.bounds().unwrap();
            // its walls reach below the ground for a slope; its ridge stands above its eaves by its pitch
            assert!(lo.y < 0.0 && (hi.y - (1.5 * STOREY + 0.036 * 0.58 * 0.8)).abs() < 0.002, "{detail:?}: from {} to {}", lo.y, hi.y);
            // the porch stands before it at the play and the near levels
            assert_eq!(hi.z > 0.0225, detail != Detail::Far);
        }
    }
}
