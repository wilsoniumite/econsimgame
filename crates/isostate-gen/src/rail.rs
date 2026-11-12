//! What stands beside a railway where the ground was too steep for an
//! ordinary line: the winding engine house at the head of an inclined plane
//! (before locomotives could climb such a bank), the shed of the banking
//! engine at the foot of a steep bank (after), and the buffer stop at the end
//! of a line. The viewer finds where they stand (`isostate-viewer/src/rails.rs`).
//!
//! Each stands with its front (+z) to the track, which runs along x a little
//! way in front of it: `TRACK_CLEAR` from the front of its footprint. Their
//! walls, roofs and openings are the kit's, at the scale every building
//! answers to (`scale.rs`), and each is drawn at the level the camera asks.

use glam::Vec3;

use crate::colour::{shade, Rgba};
use crate::houses::roof::Cover;
use crate::kit::{Detail, End, Kit, Open, Opening, Over, Range, RoofSpec, Side, Wall, FOOT};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::STOREY;
use crate::site::Site;

/// How far in front of the footprint of a winding house or a shed the middle of the track runs.
pub const TRACK_CLEAR: f32 = 0.03;

/// The ground a winding engine house keeps: along the track, and back from it.
pub const WINDING_HOUSE: (f32, f32) = (0.09, 0.056);

/// The ground a banking engine's shed keeps: along the track, and back from it.
pub const BANKER_SHED: (f32, f32) = (0.13, 0.044);

/// How far the walls of what stands by the line reach below the rails: the
/// viewer stands each on ground within a little of the rails' height.
const SINK: f32 = 0.03;
/// How high the sill of a window stands above the ground.
const SILL: f32 = FOOT + 0.0058;

/// What a wall of a colour is made of: a plinth a shade darker, painted frames, stone sills.
fn fabric(col: Rgba, pal: &Palette) -> Wall {
    Wall { col, plinth: Some(shade(col, 0.84)), frame: *pal.walls.window_frame, sill: *pal.stone.sill }
}

/// A window of small panes, each about 0.003 square, for a hole of a size.
fn light(u: f32, y: f32, w: f32, h: f32, round: bool) -> Opening {
    let (cols, rows) = (((w / 0.0032).round() as u8).clamp(1, 12), ((h / 0.003).round() as u8).clamp(1, 12));
    Opening { u, y, w, h, kind: Open::Paned { cols, rows, round, lit: false } }
}

/// The winding engine house at the head of an inclined plane: a tall stone
/// engine house under slate, its gable and its great window to the track;
/// its boiler house beside it, a storey high, with a square stack that
/// smokes; and before it the winding drum in its timber frame, the rope
/// running from it out to the track.
///
/// From far off the two buildings and the stack. At the distance the game
/// is played from, the tall round-headed windows of the engine house, the
/// doors, the drum in its frame and the rope. Close to the windows are let
/// into the walls with their panes, the stack has its head, and the drum
/// its cheeks and the ties of its frame.
pub fn winding_house(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, detail: Detail, pal: &Palette) {
    let yaw = site.yaw;
    let q = site.turn();
    let (far, near) = (detail == Detail::Far, detail == Detail::Near);
    let (stone, slate, iron, timber) = (*pal.stone.works, *pal.slate.works, *pal.iron.winding_gear, *pal.timber.headframe);
    let lie = Cover::Slate.lie();
    {
        let mut k = Kit::new(solid, site, pal, detail, 0.0);
        // the engine house: one tall space for the engine's beam, narrow, its gable to the track
        let (h, rise) = (0.056, 0.014);
        let house = Range::rect_across(-0.005, 0.029, -0.024, 0.012);
        let wall = fabric(stone, pal);
        let tall = |u: f32| light(u, 0.014, 0.0085, 0.03, true);
        let door = Opening { u: 0.0055, y: FOOT, w: 0.0052, h: 0.0126, kind: Open::Door { paint: *pal.timber.door, over: Over::Nothing } };
        // its sides in the kit's order: the wall away from the boilers, the back, the wall the
        // boiler house stands against (lit above that roof), and the gable to the track
        let right = [door, tall(0.0165), tall(0.0285)];
        let back = [tall(0.010), tall(0.024)];
        let left = [light(0.008, 0.033, 0.0066, 0.014, true), light(0.022, 0.033, 0.0066, 0.014, true)];
        let front = [light(0.017, 0.016, 0.011, 0.031, true)];
        k.walls(&house, -SINK, h, &wall, [Side::free(&right), Side::free(&back), Side::free(&left), Side::free(&front)], true, true);
        k.roof(&house, &RoofSpec { h, rise, eaves: lie.eaves, verge: lie.verge, ends: [End::Gable, End::Gable], col: slate, gable: stone, thick: lie.thick });
        // the boiler house, a storey high, against the engine house's side
        let boilers = Range::rect(-0.042, -0.005, -0.024, 0.004);
        let wall = fabric(shade(stone, 0.96), pal);
        let pane = |u: f32| light(u, SILL, 0.0066, 0.0086, false);
        let front = [Opening { u: 0.0095, y: 0.0, w: 0.009, h: 0.0135, kind: Open::Leaves { paint: *pal.timber.door, round: false } }, pane(0.021), pane(0.031)];
        let back = [pane(0.008), pane(0.019)];
        let end = [pane(0.019)];
        k.walls(&boilers, -SINK, STOREY, &wall, [Side::free(&front), Side::NONE, Side::free(&back), Side::free(&end)], true, true);
        k.roof(&boilers, &RoofSpec { h: STOREY, rise: 0.009, eaves: lie.eaves, verge: lie.verge, ends: [End::Party, End::Gable], col: slate, gable: wall.col, thick: lie.thick });
    }
    // the stack, through the boiler house's roof at its back
    let stack = site.at(-0.034, -0.018);
    let (brick, turn) = (*pal.brick.chimney, std::f32::consts::FRAC_PI_4 - yaw);
    solid.frustum_turned(stack, 0.0085, 0.006, 0.1, 4, turn, SINK, brick, Some(*pal.smoke.soot));
    if near {
        // the oversailing courses of its head
        solid.frustum_turned(stack + Vec3::Y * 0.0935, 0.0072, 0.0072, 0.0045, 4, turn, 0.0, shade(brick, 0.9), None);
    }
    crate::works::plume(smoke, stack + Vec3::Y * 0.1, 0.7, 1.3, *pal.smoke.puff);
    if far {
        return;
    }
    // the drum before the house: two timber frames and the dark drum between them
    let drum = site.at(0.012, 0.02);
    for dx in [-0.012f32, 0.012] {
        let foot = drum + q * Vec3::new(dx, 0.0, 0.0);
        let head = foot + q * Vec3::new(0.0, 0.016, 0.0);
        solid.beam(foot + q * Vec3::new(0.0, -0.004, -0.007), head, 0.003, timber);
        solid.beam(foot + q * Vec3::new(0.0, -0.004, 0.007), head, 0.003, timber);
        if near {
            // the tie between the two legs of a frame
            solid.beam(foot + q * Vec3::new(0.0, 0.006, -0.0045), foot + q * Vec3::new(0.0, 0.006, 0.0045), 0.0014, shade(timber, 0.9));
        }
    }
    solid.block(drum + Vec3::Y * 0.007, 0.02, 0.014, 0.014, yaw, iron, Some(shade(iron, 1.15)), 0.0);
    if near {
        // the cheeks of the drum, which keep the rope on it
        for dx in [-0.0094f32, 0.0094] {
            solid.block(drum + q * Vec3::new(dx, 0.0055, 0.0), 0.0012, 0.017, 0.017, yaw, shade(iron, 0.8), Some(iron), 0.0);
        }
    }
    // the rope out to the middle of the track
    let out = site.at(0.012, WINDING_HOUSE.1 * 0.5 + TRACK_CLEAR - 0.002);
    solid.beam(drum + Vec3::Y * 0.018, out + Vec3::Y * 0.022, if near { 0.0009 } else { 0.0016 }, *pal.rail.rope);
}

/// The shed of a banking engine at the foot of a steep bank: a long brick
/// shed under slate, as high as an engine needs, with the arched doorway
/// its road leaves by in one gable, a smoke vent along its ridge, a water
/// tank on a stone base and a coal stage beside it.
///
/// From far off the shed, the tank and the coal. At the distance the game
/// is played from, its round-headed windows, the doorway and the vent.
/// Close to the windows have their panes and the tank its straps.
pub fn banker_shed(solid: &mut MeshBuf, site: &Site, detail: Detail, pal: &Palette) {
    let yaw = site.yaw;
    let (far, near) = (detail == Detail::Far, detail == Detail::Near);
    let (brick, slate) = (*pal.brick.shed, *pal.slate.works);
    let lie = Cover::Slate.lie();
    let (w, d, h, rise) = (0.078, 0.03, 0.03, 0.012);
    let (cx, cz) = (0.012, -0.004);
    {
        let mut k = Kit::new(solid, site, pal, detail, 0.0);
        let rg = Range::rect(cx - w * 0.5, cx + w * 0.5, cz - d * 0.5, cz + d * 0.5);
        let wall = fabric(brick, pal);
        // four bays of tall windows down each side, the engine's doorway in the gable at +x
        let long: Vec<Opening> = (0..4).map(|i| light(w * (i as f32 + 0.5) / 4.0, SILL, 0.0066, 0.0168, true)).collect();
        let way = [Opening { u: d * 0.5, y: 0.0, w: 0.018, h: 0.026, kind: Open::Mouth { within: *pal.rail.tunnel_mouth, round: true } }];
        let end = [light(d * 0.5, SILL, 0.0066, 0.0168, true)];
        k.walls(&rg, -SINK, h, &wall, [Side::free(&long), Side::free(&way), Side::free(&long), Side::free(&end)], true, true);
        k.roof(&rg, &RoofSpec { h, rise, eaves: lie.eaves, verge: lie.verge, ends: [End::Gable, End::Gable], col: slate, gable: brick, thick: lie.thick });
    }
    if !far {
        // the vent along the ridge
        solid.block(site.at(cx, cz) + Vec3::Y * (h + 0.0105), w * 0.6, 0.006, 0.004, yaw, shade(slate, 0.8), Some(shade(slate, 1.1)), 0.0);
    }
    // the water tank on its base, and the coal stage
    let tank = site.at(-0.046, 0.004);
    let plate = *pal.iron.plate;
    solid.block(tank, 0.016, 0.016, 0.026, yaw, *pal.stone.works, None, SINK);
    solid.block(tank + Vec3::Y * 0.026, 0.022, 0.022, 0.012, yaw, plate, Some(*pal.water.river), 0.0);
    if near {
        // the straps round the tank
        for y in [0.0285, 0.0345] {
            solid.block(tank + Vec3::Y * y, 0.0228, 0.0228, 0.001, yaw, shade(plate, 0.75), None, 0.0);
        }
    }
    let coal = site.at(-0.05, -0.01);
    solid.frustum_turned(coal, 0.011, 0.004, 0.009, 5, 0.4, 0.01, *pal.goods.coal, Some(shade(*pal.goods.coal, 1.1)));
}

/// The buffer stop at the end of a line: two posts and a beam across the
/// track, which comes to it along x from behind (the stop faces -x). From
/// far off the beam on its posts; nearer, the stays behind them.
pub fn buffer_stop(solid: &mut MeshBuf, site: &Site, detail: Detail, pal: &Palette) {
    let q = site.turn();
    let p = site.origin;
    let timber = *pal.timber.headframe;
    for dz in [-0.009f32, 0.009] {
        solid.beam(p + q * Vec3::new(0.0, -0.004, dz), p + q * Vec3::new(0.0, 0.014, dz), 0.004, timber);
        if detail != Detail::Far {
            solid.beam(p + q * Vec3::new(0.012, -0.004, dz), p + q * Vec3::new(0.001, 0.011, dz), 0.003, timber);
        }
    }
    solid.beam(p + q * Vec3::new(0.0, 0.011, -0.013), p + q * Vec3::new(0.0, 0.011, 0.013), 0.005, *pal.rail.buffer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::Model;

    /// The things here keep to their ground (smoke is free to drift).
    fn within(solid: &MeshBuf, ground: (f32, f32)) {
        let (lo, hi) = solid.bounds().unwrap();
        // the rope of a winding house reaches out to the track, and no further
        assert!(lo.x >= -ground.0 * 0.5 - 1e-4 && hi.x <= ground.0 * 0.5 + 1e-4, "{lo} {hi}");
        assert!(lo.z >= -ground.1 * 0.5 - 1e-4 && hi.z <= ground.1 * 0.5 + TRACK_CLEAR + 1e-4, "{lo} {hi}");
    }

    #[test]
    fn a_winding_house_is_sound_and_its_stack_is_the_tallest_of_it() {
        let pal = Palette::standard();
        for detail in Detail::ALL {
            let mut m = Model::default();
            winding_house(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), detail, &pal);
            assert!(m.is_sound());
            within(&m.solid, WINDING_HOUSE);
            let top = m.solid.bounds().unwrap().1.y;
            assert!((top - 0.1).abs() < 1e-5, "{detail:?}: {top}");
            // it smokes from the top of the stack
            assert!(m.smoke.bounds().unwrap().0.y >= 0.1 - 1e-5);
            // and is the same each time
            let mut again = Model::default();
            winding_house(&mut again.solid, &mut again.smoke, &Site::level(Vec3::ZERO, 0.0), detail, &pal);
            assert_eq!(m.solid.pos, again.solid.pos);
        }
    }

    #[test]
    fn a_banking_shed_is_sound_and_keeps_its_ground() {
        let pal = Palette::standard();
        for detail in Detail::ALL {
            let mut b = MeshBuf::default();
            banker_shed(&mut b, &Site::level(Vec3::ZERO, 0.0), detail, &pal);
            assert!(b.is_sound());
            within(&b, BANKER_SHED);
            // lower than a winding house's stack, longer than it is deep
            let (lo, hi) = b.bounds().unwrap();
            assert!(hi.y < 0.06 && hi.x - lo.x > 2.0 * (hi.z - lo.z), "{detail:?}");
        }
    }

    #[test]
    fn a_buffer_stop_stands_across_the_track() {
        for detail in Detail::ALL {
            let mut b = MeshBuf::default();
            buffer_stop(&mut b, &Site::level(Vec3::ZERO, 0.0), detail, &Palette::standard());
            assert!(b.is_sound());
            let (lo, hi) = b.bounds().unwrap();
            assert!(hi.z - lo.z > 0.02 && hi.y < 0.02 && hi.y > 0.01, "{detail:?}");
        }
    }

    #[test]
    fn what_stands_by_the_line_is_the_same_thing_at_every_level_and_smokes_the_same() {
        let pal = Palette::standard();
        let build = |detail| {
            let mut m = Model::default();
            winding_house(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.4), detail, &pal);
            m
        };
        let (far, play, near) = (build(Detail::Far), build(Detail::Play), build(Detail::Near));
        assert!(far.solid.triangles() < play.solid.triangles() && play.solid.triangles() < near.solid.triangles());
        assert_eq!(far.smoke.pos, play.smoke.pos);
        assert_eq!(play.smoke.pos, near.smoke.pos);
    }
}
