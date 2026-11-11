use glam::{Quat, Vec3};

use crate::hash::Seed;
use crate::houses::roof::Roof;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::{smoking_chimney, windows, STOREY};
use crate::site::Site;

/// A terraced row. It stands with its middle on the site, along the site's x.
#[derive(Clone, Copy, Debug)]
pub struct Terrace {
    /// how many houses are in the row
    pub houses: usize,
    /// from end wall to end wall
    pub length: f32,
    pub depth: f32,
    /// two or three
    pub storeys: u8,
    /// what the row was roofed with when it went up, and how weathered that is now
    pub roof: Roof,
    /// the row's own numbers: its first plot's
    pub seed: Seed,
}

impl Terrace {
    /// The length of a row of `houses` plots, each `unit` wide, as the towns lay them out.
    pub fn length_of(houses: usize, unit: f32) -> f32 {
        (houses as f32 - 1.0) * unit + unit * 0.96
    }
}

/// A row is raised a storey when its own ground is dense enough.
pub fn storeys(density: f32, seed: Seed) -> u8 {
    if density >= 0.78 + 0.2 * seed.v { 3 } else { 2 }
}

/// A terraced row: one long block and roof over several plots, in brick, with a
/// chimney stack on every party wall.
pub fn terrace(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, t: &Terrace, pal: &Palette) {
    let p = site.origin;
    let yaw = site.yaw;
    let Seed { u, v } = t.seed;
    let (len, storeys) = (t.length, t.storeys as f32);
    let wall = pal.brick.terrace(v);
    let roof = t.roof.rebuilt().colour(pal, t.seed);
    // terraces have lower roofs than cottages
    let h = STOREY * storeys;
    solid.block(p, len, t.depth, h, yaw, wall, None, 0.03);
    solid.gable(p, len, t.depth, h, t.depth * 0.36, yaw, 0.004, roof, wall);
    windows(solid, pal, p, len, t.depth, storeys, yaw, t.houses, true);
    let q = Quat::from_rotation_y(yaw);
    let top = h + t.depth * 0.36 * 0.55;
    for i in 0..=t.houses {
        let x = -len * 0.5 + len * i as f32 / t.houses as f32;
        let cp = p + q * Vec3::new(x.clamp(-len * 0.5 + 0.004, len * 0.5 - 0.004), 0.0, 0.0);
        if i == t.houses / 2 && u < 0.25 {
            smoking_chimney(solid, smoke, pal, cp, top + 0.012, 0.0045, v * 9.0);
        } else {
            solid.block(cp + Vec3::Y * top, 0.008, 0.006, 0.01, yaw, *pal.brick.dark, Some(*pal.smoke.stack_top), 0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::houses::{BACK_PLOT_WIDTH, PLOT};
    use crate::site::Model;

    fn build(houses: usize, storeys: u8, seed: Seed) -> Model {
        let mut m = Model::default();
        let t = Terrace { houses, length: Terrace::length_of(houses, BACK_PLOT_WIDTH), depth: PLOT.1, storeys, roof: Roof::of(1840, 1870, false, false, seed), seed };
        terrace(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), &t, &Palette::standard());
        m
    }

    #[test]
    fn a_row_is_as_long_as_its_houses() {
        for houses in 2..=6 {
            let m = build(houses, 2, Seed::of(1));
            assert!(m.is_sound());
            let (lo, hi) = m.solid.bounds().unwrap();
            let want = Terrace::length_of(houses, BACK_PLOT_WIDTH) + 0.008;
            assert!((hi.x - lo.x - want).abs() < 1e-5, "{houses} houses: {} long, wanted {want}", hi.x - lo.x);
        }
    }

    #[test]
    fn every_house_has_a_door_and_every_party_wall_a_stack() {
        let pal = Palette::standard();
        // a seed whose row has no smoking chimney, so every stack is a block
        let seed = Seed::new(0.5, 0.5);
        for houses in 2..=6 {
            let m = build(houses, 2, seed);
            let doors = m.solid.col.iter().filter(|c| **c == *pal.timber.door).count() / 4;
            assert_eq!(doors, houses);
            let stack_tops = m.solid.col.iter().filter(|c| **c == *pal.smoke.stack_top).count() / 4;
            assert_eq!(stack_tops, houses + 1);
        }
    }

    #[test]
    fn two_storeys_then_three() {
        let s = Seed::new(0.5, 0.5);
        assert_eq!(storeys(0.8, s), 2);
        assert_eq!(storeys(0.9, s), 3);
        assert!(build(4, 3, s).solid.bounds().unwrap().1.y > build(4, 2, s).solid.bounds().unwrap().1.y);
    }
}
