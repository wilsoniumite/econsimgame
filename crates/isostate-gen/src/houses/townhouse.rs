use glam::{Quat, Vec3};

use crate::hash::Seed;
use crate::houses::roof::Roof;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::{chimney, smoking_chimney, windows, STOREY};
use crate::site::Site;

/// A house in a continuous frontage.
#[derive(Clone, Copy, Debug)]
pub struct Townhouse {
    pub plot_width: f32,
    pub plot_depth: f32,
    /// two or three
    pub storeys: u8,
    /// what its plot's house was roofed with, and how weathered that is now
    /// (a townhouse is a rebuilding, so it is never thatched)
    pub roof: Roof,
    pub seed: Seed,
}

/// A townhouse has two storeys, and a third once its ground is dense enough.
pub fn storeys(density: f32, seed: Seed) -> u8 {
    2 + (density >= 0.62 + 0.35 * seed.u) as u8
}

/// A townhouse: the full width of its plot, two or three storeys, its own
/// colour and roof, so a high street reads as a row of separate houses.
pub fn townhouse(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, t: &Townhouse, pal: &Palette) {
    let p = site.origin;
    let Seed { u, v } = t.seed;
    let storeys = t.storeys as f32;
    let w = t.plot_width * 0.99;
    let d = t.plot_depth * (0.95 + 0.15 * u);
    let wall = if v < 0.25 { pal.brick.terrace(u) } else { pal.walls.town_house(v) };
    let roof = t.roof.rebuilt().colour(pal, t.seed);
    let h = STOREY * storeys;
    solid.block(p, w, d, h, site.yaw, wall, None, 0.03);
    solid.gable(p, w, d, h, d * (0.4 + 0.2 * u), site.yaw, 0.004, roof, wall);
    windows(solid, pal, p, w, d, storeys, site.yaw, 1, true);
    let cp = p + Quat::from_rotation_y(site.yaw) * Vec3::new(w * 0.42, 0.0, 0.0);
    if u < 0.12 {
        smoking_chimney(solid, smoke, pal, cp, h + d * 0.3, 0.004, v * 7.0);
    } else {
        chimney(solid, pal, cp, h + d * 0.3, 0.004);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::houses::PLOT;
    use crate::site::Model;

    #[test]
    fn it_fills_the_width_of_its_plot() {
        let pal = Palette::standard();
        for n in 0..40 {
            let mut m = Model::default();
            let roof = Roof::of(1750, 1850, false, false, Seed::of(n));
            townhouse(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), &Townhouse { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: 2, roof, seed: Seed::of(n) }, &pal);
            assert!(m.is_sound());
            // whatever its plot's cottage was roofed with, a townhouse is not thatched
            assert!(!m.solid.col.contains(&*pal.thatch.cottage) && !m.solid.col.contains(&*pal.thatch.weathered));
            let (lo, hi) = m.solid.bounds().unwrap();
            // wall to wall is the plot, and the eaves reach a little over the neighbours'
            assert!((hi.x - lo.x - (PLOT.0 * 0.99 + 0.008)).abs() < 1e-5, "{}", hi.x - lo.x);
        }
    }

    #[test]
    fn two_storeys_then_three() {
        let s = Seed::new(0.2, 0.5);
        assert_eq!(storeys(0.3, s), 2);
        assert_eq!(storeys(0.7, s), 3);
    }
}
