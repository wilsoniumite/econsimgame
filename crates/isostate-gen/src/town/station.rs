use glam::Vec3;

use crate::colour::shade;
use crate::houses::roof::Cover;
use crate::kit::{Detail, End, Kit, Open, Opening, Range, RoofSpec, Side, Wall, FOOT};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::STOREY;
use crate::site::Site;

/// How long and how deep the station's building is, and how high its walls:
/// a booking hall, one tall space, a storey and a half to the eaves.
const BUILDING: (f32, f32, f32) = (0.09, 0.035, 1.5 * STOREY);
/// How high the clock turret stands through the roof.
const TURRET: f32 = 0.06;

/// A railway station: a long low building under tile with a clock turret
/// through its roof, and a platform under a canopy along its front (+z).
///
/// From far off the building, the turret, the platform and the canopy over
/// it. At the distance the game is played from the round-headed windows of
/// the booking hall, its doors, the clock's faces and the canopy's posts.
/// Close to the windows are let into the wall with their panes, and the
/// canopy has its valance.
pub fn station(solid: &mut MeshBuf, site: &Site, detail: Detail, pal: &Palette) {
    let (p, yaw) = (site.origin, site.yaw);
    let q = site.turn();
    let (brick, roof, canopy) = (*pal.walls.station, *pal.roofs.tile_dark, *pal.roofs.station_canopy);
    let (len, dep, h) = BUILDING;
    let wall = Wall { col: brick, plinth: Some(shade(brick, 0.84)), frame: *pal.walls.window_frame, sill: *pal.stone.sill };
    let lie = Cover::Tile.lie();
    let rise = 0.018;
    {
        let mut k = Kit::new(solid, site, pal, detail, 0.0);
        let rg = Range::rect(-len * 0.5, len * 0.5, -dep * 0.5, dep * 0.5);
        // five bays: tall round-headed windows, and the doors to the platform and to the town in the middle one
        let cell = len / 5.0;
        let light = |u: f32| Opening { u, y: FOOT + 0.0058, w: 0.0066, h: h - FOOT - 0.0058 - 0.0052, kind: Open::Paned { cols: 2, rows: 6, round: true, lit: false } };
        let doors = Opening { u: len * 0.5, y: FOOT, w: 0.0088, h: 0.0162, kind: Open::Leaves { paint: *pal.timber.door, round: true } };
        let long: Vec<Opening> = (0..5).map(|c| if c == 2 { doors } else { light(cell * (c as f32 + 0.5)) }).collect();
        let end = [light(dep * 0.5)];
        k.walls(&rg, -k.drop(&rg), h, &wall, [Side::free(&long), Side::free(&end), Side::free(&long), Side::free(&end)], true, true);
        k.roof(&rg, &RoofSpec { h, rise, eaves: lie.eaves, verge: lie.verge, ends: [End::Gable, End::Gable], col: roof, gable: brick, thick: lie.thick });
        // the clock turret, through the middle of the roof
        let t = 0.008;
        let turret = Range::rect(-t, t, -t, t);
        k.walls(&turret, h, TURRET, &wall, [Side::PLAIN; 4], false, true);
        if !k.far() {
            // a clock's face to the platform and one to the town
            let face = *pal.walls.window_frame;
            for side in [0, 2] {
                k.strip(&turret, side, t - 0.0034, t + 0.0034, TURRET - 0.0098, TURRET - 0.003, 0.0007, face);
                if k.near() {
                    k.strip(&turret, side, t - 0.0003, t + 0.0003, TURRET - 0.0064, TURRET - 0.0038, 0.0012, *pal.smoke.soot);
                }
            }
        }
    }
    solid.frustum_turned(p + Vec3::Y * (TURRET - 0.0014), 0.0135, 0.0, 0.022, 4, std::f32::consts::FRAC_PI_4 - yaw, 0.0, roof, None);
    // the platform and its canopy
    let c = p + q * Vec3::new(0.0, 0.0, 0.035);
    solid.block(c, 0.12, 0.03, 0.002, yaw, *pal.ground.station_platform, None, 0.0);
    solid.block(c + Vec3::Y * 0.022, 0.13, 0.034, 0.004, yaw, canopy, Some(canopy), 0.0);
    if detail == Detail::Far {
        return;
    }
    // the posts the canopy stands on, along the platform's edge, and close to its valance
    let near = detail == Detail::Near;
    let iron = *pal.iron.plate;
    let posts = if near { 7 } else { 4 };
    for i in 0..posts {
        let x = -0.058 + 0.116 * i as f32 / (posts - 1) as f32;
        let foot = c + q * Vec3::new(x, 0.002, 0.012);
        solid.beam(foot, foot + Vec3::Y * 0.02, if near { 0.0014 } else { 0.0022 }, iron);
    }
    if near {
        let edge = shade(canopy, 0.82);
        let at = |x: f32, y: f32| c + q * Vec3::new(x, y, 0.0172);
        solid.quad_toward([at(-0.065, 0.0196), at(0.065, 0.0196), at(0.065, 0.022), at(-0.065, 0.022)], [edge; 4], q * Vec3::Z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_platform_is_along_the_front() {
        for detail in Detail::ALL {
            let mut b = MeshBuf::default();
            station(&mut b, &Site::level(Vec3::ZERO, 0.0), detail, &Palette::standard());
            assert!(b.is_sound());
            let (lo, hi) = b.bounds().unwrap();
            assert!(hi.z > 0.05 && lo.z > -0.03, "{detail:?}");
            // the turret's roof is the highest thing
            assert!((hi.y - (TURRET - 0.0014 + 0.022)).abs() < 1e-6, "{detail:?}: {}", hi.y);
        }
    }
}
