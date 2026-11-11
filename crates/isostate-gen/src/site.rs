//! Where a generator stands, and what it builds into.

use glam::{Quat, Vec3};

use crate::mesh::MeshBuf;

/// Where a thing stands on the map: the point on the ground its middle is
/// at, the way it faces, and the ground round it.
///
/// Everything is in map units: one unit is one cell of the simulation, 2 km,
/// and up is +y (with the relief exaggerated about three times, as on a
/// model). A thing's own +x runs along its front (a house's ridge, a
/// street), +z is its front, where the door is, and `yaw` turns it about
/// the upright: at a yaw of nothing the front faces south, toward the camera
/// as a game opens.
///
/// A generator writes its corners where they stand on the map, not round
/// nought to be moved afterwards, so that the same slot gives the same
/// numbers every time. `at` gives the point at an offset in the thing's own
/// frame, on the ground there, for the parts of a thing that stand apart (a
/// barn across the yard, a chimney beside a mill).
#[derive(Clone, Copy)]
pub struct Site<'a> {
    pub origin: Vec3,
    pub yaw: f32,
    ground: Option<&'a dyn Fn(f32, f32) -> f32>,
}

impl<'a> Site<'a> {
    /// A site on level ground at the height of `origin`: the lab's, and a test's.
    pub fn level(origin: Vec3, yaw: f32) -> Site<'static> {
        Site { origin, yaw, ground: None }
    }

    /// A site on the island: `ground` gives the height of the land at a map point (x, z).
    pub fn on(origin: Vec3, yaw: f32, ground: &'a dyn Fn(f32, f32) -> f32) -> Site<'a> {
        Site { origin, yaw, ground: Some(ground) }
    }

    /// The turn from the thing's own frame to the map's.
    pub fn turn(&self) -> Quat {
        Quat::from_rotation_y(self.yaw)
    }

    /// The point `dx` along the front and `dz` out from it, on the ground there.
    pub fn at(&self, dx: f32, dz: f32) -> Vec3 {
        let o = self.turn() * Vec3::new(dx, 0.0, dz);
        let (x, z) = (self.origin.x + o.x, self.origin.z + o.z);
        let y = match self.ground {
            Some(ground) => ground(x, z),
            None => self.origin.y,
        };
        Vec3::new(x, y, z)
    }

    /// The ground is level here (the lab's, a test's), so nothing need follow it.
    pub fn is_level(&self) -> bool {
        self.ground.is_none()
    }

    /// The same ground, standing somewhere else on it.
    pub fn moved(&self, origin: Vec3, yaw: f32) -> Site<'a> {
        Site { origin, yaw, ground: self.ground }
    }
}

/// What a generator builds into, kept apart because the viewer draws each
/// with a material of its own: `solid` for walls, roofs and everything
/// built; `smoke`, which lets light through and casts no shadow; `green`
/// for leaves, which turn with the seasons; `flat` for what lies on the
/// ground (roads, lanes, paving, water), which is matt and casts no shadow.
///
/// A generator takes the buffers it writes as arguments and appends to them
/// (the game hands it the one buffer all the island's houses share, so that
/// nothing is copied). In the lab and in tests a `Model` holds one of each.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Model {
    pub solid: MeshBuf,
    pub smoke: MeshBuf,
    pub green: MeshBuf,
    pub flat: MeshBuf,
}

impl Model {
    fn parts(&self) -> [&MeshBuf; 4] {
        [&self.solid, &self.smoke, &self.green, &self.flat]
    }

    pub fn is_empty(&self) -> bool {
        self.parts().into_iter().all(|b| b.is_empty())
    }

    /// Append a copy of another model's triangles, moved by `by`.
    pub fn merge_at(&mut self, other: &Model, by: Vec3) {
        for (mine, theirs) in [(&mut self.solid, &other.solid), (&mut self.smoke, &other.smoke), (&mut self.green, &other.green), (&mut self.flat, &other.flat)] {
            let from = mine.pos.len();
            mine.merge(theirs);
            for p in &mut mine.pos[from..] {
                *p = (Vec3::from_array(*p) + by).to_array();
            }
        }
    }

    /// Turn everything about the upright through the origin, the way a
    /// site's `yaw` turns what is built on it. (For the lab, which shows a
    /// sheet's things from another side; the game hands a generator its
    /// facing in the site.)
    pub fn turn(&mut self, yaw: f32) {
        let t = glam::Affine3A::from_rotation_y(yaw);
        for b in [&mut self.solid, &mut self.smoke, &mut self.green, &mut self.flat] {
            b.transform(t);
        }
    }

    /// The box round everything in the model, if there is anything.
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        self.parts().into_iter().filter_map(|b| b.bounds()).reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
    }

    pub fn is_sound(&self) -> bool {
        self.parts().into_iter().all(|b| b.is_sound())
    }

    pub fn triangles(&self) -> usize {
        self.parts().into_iter().map(|b| b.triangles()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_model_merges_each_kind_to_its_own_and_moves_it() {
        let red = [1.0, 0.0, 0.0, 1.0];
        let mut a = Model::default();
        a.solid.tri(Vec3::ZERO, Vec3::X, Vec3::Z, red);
        a.flat.tri(Vec3::ZERO, Vec3::X, Vec3::Z, red);
        let mut sheet = Model::default();
        sheet.merge_at(&a, Vec3::new(1.0, 0.0, 0.0));
        sheet.merge_at(&a, Vec3::new(3.0, 0.0, 0.0));
        assert_eq!((sheet.solid.triangles(), sheet.flat.triangles(), sheet.green.triangles()), (2, 2, 0));
        assert_eq!(sheet.bounds().unwrap(), (Vec3::new(1.0, 0.0, 0.0), Vec3::new(4.0, 0.0, 1.0)));
        assert!(sheet.is_sound() && !sheet.is_empty() && Model::default().is_empty());
        // the first copy was not moved again by the second
        assert_eq!(sheet.solid.pos[0], [1.0, 0.0, 0.0]);
    }

    #[test]
    fn turning_a_model_is_building_it_on_a_turned_site() {
        let pal = crate::Palette::standard();
        let c = crate::houses::Cottage { plot_width: 0.05, plot_depth: 0.036, storeys: 2, seed: crate::Seed::of(3) };
        let build = |yaw: f32| {
            let mut m = Model::default();
            crate::houses::cottage(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, yaw), &c, &pal);
            m
        };
        let (mut turned, built) = (build(0.0), build(0.6));
        turned.turn(0.6);
        assert_eq!(turned.solid.idx, built.solid.idx);
        assert_eq!(turned.solid.col, built.solid.col);
        let near = |a: &[[f32; 3]], b: &[[f32; 3]]| a.iter().zip(b).all(|(p, q)| (Vec3::from_array(*p) - Vec3::from_array(*q)).length() < 1e-5);
        assert!(near(&turned.solid.pos, &built.solid.pos) && near(&turned.solid.nrm, &built.solid.nrm));
        assert!(near(&turned.smoke.pos, &built.smoke.pos));
    }

    #[test]
    fn a_level_site_keeps_its_height() {
        let s = Site::level(Vec3::new(3.0, 0.5, 4.0), 0.0);
        assert_eq!(s.at(0.1, 0.2), Vec3::new(3.1, 0.5, 4.2));
    }

    #[test]
    fn a_site_on_the_island_asks_the_ground() {
        let slope = |x: f32, _z: f32| x * 0.5;
        let s = Site::on(Vec3::new(2.0, 1.0, 0.0), 0.0, &slope);
        assert_eq!(s.at(1.0, 0.0), Vec3::new(3.0, 1.5, 0.0));
        // the same ground somewhere else
        assert_eq!(s.moved(Vec3::new(4.0, 2.0, 0.0), 0.0).at(0.0, 0.0), Vec3::new(4.0, 2.0, 0.0));
    }

    #[test]
    fn the_front_is_plus_z_and_yaw_turns_it_about_the_upright() {
        // a quarter turn takes the front (+z) round to +x
        let s = Site::level(Vec3::ZERO, std::f32::consts::FRAC_PI_2);
        let front = s.at(0.0, 1.0);
        assert!((front - Vec3::X).length() < 1e-6, "{front}");
        let along = s.at(1.0, 0.0);
        assert!((along - Vec3::NEG_Z).length() < 1e-6, "{along}");
    }
}
