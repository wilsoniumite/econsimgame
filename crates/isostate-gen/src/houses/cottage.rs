use glam::{Quat, Vec3};

use crate::hash::Seed;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::{chimney, gabled, smoking_chimney, windows, STOREY};
use crate::site::Site;

/// A house standing on its own plot.
#[derive(Clone, Copy, Debug)]
pub struct Cottage {
    /// the plot it stands on; the house is a little narrower, so the gaps show
    pub plot_width: f32,
    pub plot_depth: f32,
    /// one to three
    pub storeys: u8,
    pub seed: Seed,
}

/// Each house adds a storey at its own density, so a street grows a house at a time.
pub fn storeys(density: f32, seed: Seed) -> u8 {
    1 + (density >= 0.35 + 0.35 * seed.v) as u8 + (density >= 0.8 + 0.15 * seed.u) as u8
}

/// A cottage: washed walls under a steep roof of tile or slate, a little
/// askew on its plot, some with a chimney at one end and a few of those smoking.
pub fn cottage(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, c: &Cottage, pal: &Palette) {
    let p = site.origin;
    let Seed { u, v } = c.seed;
    let storeys = c.storeys as f32;
    let w = c.plot_width * (0.68 + 0.22 * v);
    let d = c.plot_depth * (0.9 + 0.2 * u);
    let yaw = site.yaw + (v - 0.5) * 0.08;
    let wall = pal.walls.town_house(v);
    let roof = pal.house_roof(u);
    gabled(solid, p, w, d, storeys, yaw, wall, roof);
    windows(solid, pal, p, w, d, storeys, yaw, 1, true);
    if u < 0.3 {
        let cp = p + Quat::from_rotation_y(yaw) * Vec3::new(w * 0.3, 0.0, 0.0);
        if v > 0.8 {
            smoking_chimney(solid, smoke, pal, cp, STOREY * storeys + 0.03, 0.004, u * 7.0);
        } else {
            chimney(solid, pal, cp, STOREY * storeys + 0.03, 0.004);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::houses::PLOT;
    use crate::site::Model;

    fn build(storeys: u8, seed: Seed) -> Model {
        let mut m = Model::default();
        cottage(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), &Cottage { plot_width: PLOT.0, plot_depth: PLOT.1, storeys, seed }, &Palette::standard());
        m
    }

    #[test]
    fn a_storey_is_added_at_a_density_of_the_house_s_own() {
        let s = Seed::new(0.5, 0.5);
        assert_eq!(storeys(0.0, s), 1);
        assert_eq!(storeys(0.52, s), 1);
        assert_eq!(storeys(0.53, s), 2);
        assert_eq!(storeys(0.88, s), 3);
        // two houses on the same ground need not stand at the same step
        assert_ne!(storeys(0.5, Seed::new(0.5, 0.1)), storeys(0.5, Seed::new(0.5, 0.9)));
    }

    #[test]
    fn it_stays_on_its_plot_and_grows_with_its_storeys() {
        for n in 0..40 {
            let seed = Seed::of(n);
            let mut last = 0.0;
            for storeys in 1..=3 {
                let m = build(storeys, seed);
                assert!(m.is_sound());
                let (lo, hi) = m.solid.bounds().unwrap();
                // the eaves of the widest, turned a little, overhang the plot by a hair, no more
                assert!(hi.x - lo.x <= PLOT.0 + 0.006, "seed {n}: {} wide", hi.x - lo.x);
                assert!(hi.z - lo.z <= PLOT.1 * 1.1 + 0.012, "seed {n}: {} deep", hi.z - lo.z);
                assert!(hi.y > last, "seed {n}: no taller at {storeys} storeys");
                last = hi.y;
            }
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_house() {
        assert_eq!(build(2, Seed::of(3)), build(2, Seed::of(3)));
        assert_ne!(build(2, Seed::of(3)), build(2, Seed::of(4)));
    }

    #[test]
    fn some_have_chimneys_and_a_few_of_those_smoke() {
        let (mut stacks, mut smoking) = (0, 0);
        for n in 0..400 {
            let m = build(1, Seed::of(n));
            // walls, roof, windows and door come to 22 triangles; a chimney adds its own
            stacks += (m.solid.triangles() > 22) as usize;
            smoking += !m.smoke.is_empty() as usize;
        }
        assert!((80..160).contains(&stacks), "{stacks} of 400 have a chimney");
        assert!((8..50).contains(&smoking), "{smoking} of 400 smoke");
    }
}
