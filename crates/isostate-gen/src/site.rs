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

    /// The same ground, standing somewhere else on it.
    pub fn moved(&self, origin: Vec3, yaw: f32) -> Site<'a> {
        Site { origin, yaw, ground: self.ground }
    }
}

/// What a generator builds into, kept apart because the viewer draws each
/// with a material of its own: `solid` for walls, roofs and everything
/// built; `smoke`, which lets light through and casts no shadow; `green`
/// for leaves, which turn with the seasons.
///
/// A generator takes the buffers it writes as arguments and appends to them
/// (the game hands it the one buffer all the island's houses share, so that
/// nothing is copied). In the lab and in tests a `Model` holds one of each.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Model {
    pub solid: MeshBuf,
    pub smoke: MeshBuf,
    pub green: MeshBuf,
}

impl Model {
    pub fn is_empty(&self) -> bool {
        self.solid.is_empty() && self.smoke.is_empty() && self.green.is_empty()
    }

    /// Append another model's triangles, each kind to its own.
    pub fn append(&mut self, other: Model) {
        self.solid.append(other.solid);
        self.smoke.append(other.smoke);
        self.green.append(other.green);
    }

    /// Move everything in the model.
    pub fn translate(&mut self, by: Vec3) {
        self.solid.translate(by);
        self.smoke.translate(by);
        self.green.translate(by);
    }

    /// The box round everything in the model, if there is anything.
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        [&self.solid, &self.smoke, &self.green].into_iter().filter_map(|b| b.bounds()).reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
    }

    /// The box round what is built, leaving the smoke out.
    pub fn bounds_without_smoke(&self) -> Option<(Vec3, Vec3)> {
        [&self.solid, &self.green].into_iter().filter_map(|b| b.bounds()).reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
    }

    pub fn is_sound(&self) -> bool {
        self.solid.is_sound() && self.smoke.is_sound() && self.green.is_sound()
    }

    pub fn triangles(&self) -> usize {
        self.solid.triangles() + self.smoke.triangles() + self.green.triangles()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
