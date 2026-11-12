use glam::Vec3;

use crate::colour::shade;
use crate::houses::roof::Cover;
use crate::kit::{Detail, End, Kit, Open, Opening, Range, RoofSpec, Side, Wall};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::STOREY;
use crate::site::Site;

/// A market cross on its steps, in the middle of a small town's market
/// place. From far off its shaft alone; close to it has a second step and
/// a knop under its head.
pub fn market_cross(solid: &mut MeshBuf, site: &Site, detail: Detail, pal: &Palette) {
    let m = site.origin;
    let stone = *pal.stone.market_cross;
    if detail == Detail::Far {
        solid.beam(m, m + Vec3::Y * 0.03, 0.0035, stone);
        return;
    }
    solid.block(m, 0.016, 0.016, 0.004, site.yaw, stone, Some(stone), 0.0);
    let near = detail == Detail::Near;
    let foot = if near { 0.0064 } else { 0.0 };
    if near {
        solid.block(m + Vec3::Y * 0.004, 0.0105, 0.0105, 0.0024, site.yaw, shade(stone, 0.94), Some(stone), 0.0);
    }
    solid.beam(m + Vec3::Y * foot, m + Vec3::Y * 0.03, if near { 0.0028 } else { 0.0035 }, stone);
    let q = site.turn();
    solid.beam(m + q * Vec3::new(-0.006, 0.024, 0.0), m + q * Vec3::new(0.006, 0.024, 0.0), if near { 0.0026 } else { 0.003 }, stone);
    if near {
        solid.block(m + Vec3::Y * 0.0188, 0.0044, 0.0044, 0.0016, site.yaw, shade(stone, 0.9), Some(stone), 0.0);
    }
}

/// How high the arcade under a market hall is, and the room over it: a storey.
const ARCADE: f32 = 0.0135;

/// A market hall in the middle of a big town's market place: an open arcade
/// of stone arches with a room of one storey over it, under tile.
pub fn market_hall(solid: &mut MeshBuf, site: &Site, detail: Detail, pal: &Palette) {
    let mut k = Kit::new(solid, site, pal, detail, 0.0);
    let (len, dep) = (0.044, 0.03);
    let rg = Range::rect(-len * 0.5, len * 0.5, -dep * 0.5, dep * 0.5);
    let (stone, wash) = (*pal.stone.church, *pal.walls.market_hall);
    let frame = *pal.walls.window_frame;
    let lie = Cover::Tile.lie();
    let top = ARCADE + STOREY;
    // the arcade: three arches to each long side and two to each end, the market under them in shadow
    let dark = shade(*pal.smoke.soot, 1.3);
    let arches = |n: usize, len: f32| -> Vec<Opening> { (0..n).map(|i| Opening { u: len * (i as f32 + 0.5) / n as f32, y: 0.0, w: len / n as f32 - 0.0036, h: ARCADE - 0.0026, kind: Open::Mouth { within: dark, round: true } }).collect() };
    let (long, short) = (arches(3, len), arches(2, dep));
    let piers = Wall { col: stone, plinth: None, frame, sill: stone };
    k.walls(&rg, -k.drop(&rg), ARCADE, &piers, [Side::free(&long), Side::free(&short), Side::free(&long), Side::free(&short)], true, false);
    // the room over it
    let windows = |n: usize, len: f32| -> Vec<Opening> { (0..n).map(|i| Opening { u: len * (i as f32 + 0.5) / n as f32, y: ARCADE + 0.0072, w: 0.0062, h: 0.0078, kind: Open::Sash }).collect() };
    let (long, short) = (windows(3, len), windows(2, dep));
    let room = Wall { col: wash, plinth: None, frame, sill: *pal.stone.sill };
    k.walls(&rg, ARCADE, top, &room, [Side::free(&long), Side::free(&short), Side::free(&long), Side::free(&short)], false, true);
    if !k.far() {
        // the string course where the room sits on the arcade
        for i in 0..4 {
            let l = if i % 2 == 0 { len } else { dep };
            k.strip(&rg, i, 0.0, l, ARCADE - 0.0007, ARCADE + 0.0007, 0.0006, shade(stone, 1.05));
        }
    }
    k.roof(&rg, &RoofSpec { h: top, rise: 0.014, eaves: lie.eaves, verge: lie.verge, ends: [End::Hip, End::Hip], col: *pal.roofs.tile_dark, gable: wash, thick: lie.thick });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hall_stands_on_its_arcade_and_the_cross_is_a_cross() {
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        let build = |detail: Detail| {
            let (mut cross, mut hall) = (MeshBuf::default(), MeshBuf::default());
            market_cross(&mut cross, &site, detail, &pal);
            market_hall(&mut hall, &site, detail, &pal);
            assert!(cross.is_sound() && hall.is_sound());
            (cross, hall)
        };
        let (far, play, near) = (build(Detail::Far), build(Detail::Play), build(Detail::Near));
        // a shaft; then a step, a shaft and an arm; then two steps and a knop as well
        assert_eq!((far.0.triangles(), play.0.triangles()), (8, 10 + 8 + 8));
        assert!(near.0.triangles() > play.0.triangles());
        for cross in [&far.0, &play.0, &near.0] {
            assert!((cross.bounds().unwrap().1.y - 0.03).abs() < 1e-6);
        }
        // the hall: an arcade and a storey over it under a hipped roof, with more to it at each level
        for hall in [&far.1, &play.1, &near.1] {
            assert!((hall.bounds().unwrap().1.y - (ARCADE + STOREY + 0.014 + 0.0005)).abs() < 1e-4, "{}", hall.bounds().unwrap().1.y);
        }
        assert!(far.1.triangles() < play.1.triangles() && play.1.triangles() < near.1.triangles());
        // from the play distance the arches show dark
        let dark = shade(*pal.smoke.soot, 1.3);
        assert!(play.1.col.contains(&dark) && !far.1.col.contains(&dark));
    }
}
