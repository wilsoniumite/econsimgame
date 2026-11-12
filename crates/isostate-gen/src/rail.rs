//! What stands beside a railway where the ground was too steep for an
//! ordinary line: the winding engine house at the head of an inclined plane
//! (before locomotives could climb such a bank), the shed of the banking
//! engine at the foot of a steep bank (after), and the buffer stop at the end
//! of a line. The viewer finds where they stand (`isostate-viewer/src/rails.rs`).
//!
//! Each stands with its front (+z) to the track, which runs along x a little
//! way in front of it: `TRACK_CLEAR` from the front of its footprint.

use glam::Vec3;

use crate::colour::shade;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// How far in front of the footprint of a winding house or a shed the middle of the track runs.
pub const TRACK_CLEAR: f32 = 0.03;

/// The ground a winding engine house keeps: along the track, and back from it.
pub const WINDING_HOUSE: (f32, f32) = (0.09, 0.056);

/// The ground a banking engine's shed keeps: along the track, and back from it.
pub const BANKER_SHED: (f32, f32) = (0.13, 0.044);

/// The winding engine house at the head of an inclined plane: a tall stone
/// engine house with a slate roof, its boiler house beside it under a
/// lean-to with a square stack that smokes, and before it the winding drum
/// in its timber frame, the rope running from it out to the track.
pub fn winding_house(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, pal: &Palette) {
    let yaw = site.yaw;
    let (stone, slate, iron, timber) = (*pal.stone.works, *pal.slate.works, *pal.iron.winding_gear, *pal.timber.headframe);
    // the engine house: tall and narrow, its gable to the track
    let house = site.at(0.012, -0.006);
    solid.block(house, 0.034, 0.036, 0.056, yaw, stone, None, 0.03);
    solid.gable(house, 0.036, 0.034, 0.056, 0.014, yaw + std::f32::consts::FRAC_PI_2, 0.003, slate, stone);
    // a tall window to the drum, and a door
    let q = site.turn();
    let face = |x0: f32, x1: f32, y0: f32, y1: f32, col| {
        let z = 0.018 + 0.0006;
        let c = [Vec3::new(x0, y0, z), Vec3::new(x1, y0, z), Vec3::new(x1, y1, z), Vec3::new(x0, y1, z)].map(|v| house + q * v);
        (c, col)
    };
    for (c, col) in [face(-0.006, 0.006, 0.02, 0.046, *pal.glass.window), face(-0.012, -0.007, 0.0, 0.014, *pal.timber.door)] {
        solid.quad(c[0], c[1], c[2], c[3], col);
    }
    // the boiler house, lower, with the stack at its back
    let boiler = site.at(-0.024, -0.01);
    solid.block(boiler, 0.036, 0.028, 0.022, yaw, shade(stone, 0.96), None, 0.03);
    solid.gable(boiler, 0.036, 0.028, 0.022, 0.009, yaw, 0.003, slate, shade(stone, 0.96));
    let stack = site.at(-0.034, -0.018);
    solid.frustum_turned(stack, 0.0085, 0.006, 0.1, 4, std::f32::consts::FRAC_PI_4 - yaw, 0.03, *pal.brick.chimney, Some(*pal.smoke.soot));
    crate::works::plume(smoke, stack + Vec3::Y * 0.1, 0.7, 1.3, *pal.smoke.puff);
    // the drum before the house: two timber frames and the dark drum between them
    let drum = site.at(0.012, 0.02);
    for dx in [-0.012f32, 0.012] {
        let foot = drum + q * Vec3::new(dx, 0.0, 0.0);
        solid.beam(foot + q * Vec3::new(0.0, -0.004, -0.007), foot + q * Vec3::new(0.0, 0.016, 0.0), 0.003, timber);
        solid.beam(foot + q * Vec3::new(0.0, -0.004, 0.007), foot + q * Vec3::new(0.0, 0.016, 0.0), 0.003, timber);
    }
    solid.block(drum + Vec3::Y * 0.007, 0.02, 0.014, 0.014, yaw, iron, Some(shade(iron, 1.15)), 0.0);
    // the rope out to the middle of the track
    let out = site.at(0.012, WINDING_HOUSE.1 * 0.5 + TRACK_CLEAR - 0.002);
    solid.beam(drum + Vec3::Y * 0.018, out + Vec3::Y * 0.022, 0.0016, *pal.rail.rope);
}

/// The shed of a banking engine at the foot of a steep bank: a long brick
/// shed under slate with a tall door in the gable its road leaves by, a
/// smoke vent along its ridge, a water tank on a stone base and a coal stage
/// beside it.
pub fn banker_shed(solid: &mut MeshBuf, site: &Site, pal: &Palette) {
    let yaw = site.yaw;
    let q = site.turn();
    let (brick, slate) = (*pal.brick.shed, *pal.slate.works);
    let shed = site.at(0.012, -0.004);
    let (w, d, h) = (0.078, 0.03, 0.03);
    solid.block(shed, w, d, h, yaw, brick, None, 0.03);
    solid.gable(shed, w, d, h, 0.012, yaw, 0.003, slate, brick);
    // the vent along the ridge
    solid.block(shed + Vec3::Y * (h + 0.0105), w * 0.6, 0.006, 0.004, yaw, shade(slate, 0.8), Some(shade(slate, 1.1)), 0.0);
    // the door in the gable at +x, tall enough for an engine
    let x = w * 0.5 + 0.0006;
    let c = [Vec3::new(x, 0.0, 0.009), Vec3::new(x, 0.0, -0.009), Vec3::new(x, 0.026, -0.009), Vec3::new(x, 0.026, 0.009)].map(|v| shed + q * v);
    solid.quad(c[0], c[1], c[2], c[3], *pal.rail.tunnel_mouth);
    // windows along the front
    for i in 0..4 {
        let cx = -w * 0.5 + w * (i as f32 + 0.5) / 4.0;
        let z = d * 0.5 + 0.0006;
        let c = [Vec3::new(cx - 0.004, 0.01, z), Vec3::new(cx + 0.004, 0.01, z), Vec3::new(cx + 0.004, 0.022, z), Vec3::new(cx - 0.004, 0.022, z)].map(|v| shed + q * v);
        solid.quad(c[0], c[1], c[2], c[3], *pal.glass.window);
    }
    // the water tank on its base, and the coal stage
    let tank = site.at(-0.046, 0.004);
    solid.block(tank, 0.016, 0.016, 0.026, yaw, *pal.stone.works, None, 0.03);
    solid.block(tank + Vec3::Y * 0.026, 0.022, 0.022, 0.012, yaw, *pal.iron.plate, Some(*pal.water.river), 0.0);
    let coal = site.at(-0.05, -0.01);
    solid.frustum_turned(coal, 0.011, 0.004, 0.009, 5, 0.4, 0.01, *pal.goods.coal, Some(shade(*pal.goods.coal, 1.1)));
}

/// The buffer stop at the end of a line: two posts and a beam across the
/// track, which comes to it along x from behind (the stop faces -x).
pub fn buffer_stop(solid: &mut MeshBuf, site: &Site, pal: &Palette) {
    let q = site.turn();
    let p = site.origin;
    let timber = *pal.timber.headframe;
    for dz in [-0.009f32, 0.009] {
        solid.beam(p + q * Vec3::new(0.0, -0.004, dz), p + q * Vec3::new(0.0, 0.014, dz), 0.004, timber);
        solid.beam(p + q * Vec3::new(0.012, -0.004, dz), p + q * Vec3::new(0.001, 0.011, dz), 0.003, timber);
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
        let mut m = Model::default();
        winding_house(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), &pal);
        assert!(m.is_sound());
        within(&m.solid, WINDING_HOUSE);
        let top = m.solid.bounds().unwrap().1.y;
        assert!((top - 0.1).abs() < 1e-5, "{top}");
        // it smokes from the top of the stack
        assert!(m.smoke.bounds().unwrap().0.y >= 0.1 - 1e-5);
        // and is the same each time
        let mut again = Model::default();
        winding_house(&mut again.solid, &mut again.smoke, &Site::level(Vec3::ZERO, 0.0), &pal);
        assert_eq!(m.solid.pos, again.solid.pos);
    }

    #[test]
    fn a_banking_shed_is_sound_and_keeps_its_ground() {
        let pal = Palette::standard();
        let mut b = MeshBuf::default();
        banker_shed(&mut b, &Site::level(Vec3::ZERO, 0.0), &pal);
        assert!(b.is_sound());
        within(&b, BANKER_SHED);
        // lower than a winding house's stack, longer than it is deep
        let (lo, hi) = b.bounds().unwrap();
        assert!(hi.y < 0.06 && hi.x - lo.x > 2.0 * (hi.z - lo.z));
    }

    #[test]
    fn a_buffer_stop_stands_across_the_track() {
        let mut b = MeshBuf::default();
        buffer_stop(&mut b, &Site::level(Vec3::ZERO, 0.0), &Palette::standard());
        assert!(b.is_sound());
        let (lo, hi) = b.bounds().unwrap();
        assert!(hi.z - lo.z > 0.02 && hi.y < 0.02 && hi.y > 0.01);
    }
}
