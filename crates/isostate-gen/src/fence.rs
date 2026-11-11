//! The post-and-rail fence a family puts round its land.
//!
//! A fence is not placed by a `Site`: it is handed the line it runs along,
//! as points on the ground (the viewer finds the hedge lines round a
//! family's open land), and stands a post at each with a rail between.

use glam::Vec3;

use crate::colour::{lerp, Rgba};
use crate::mesh::MeshBuf;
use crate::palette::Palette;

/// How much of the family's colour is in a rail: the rest is weathered timber.
const FAMILY_IN_RAIL: f32 = 0.38;
/// How tall a post stands, and how high the rail runs.
const POST: f32 = 0.022;
const RAIL: f32 = 0.016;

/// A fence along `line`, a run of points on the ground: a timber post at
/// each point and a rail from post to post. It is the family's by its
/// colour, quietly: the rail is timber stained toward `family`, and only the
/// cap of each post, which is what shows from above, is painted in it.
pub fn fence(solid: &mut MeshBuf, line: &[Vec3], family: Rgba, pal: &Palette) {
    let timber = *pal.marks.fence_timber;
    let rail = lerp(timber, family, FAMILY_IN_RAIL);
    let post = lerp(timber, *pal.marks.post_shade, 0.4);
    for (i, &p) in line.iter().enumerate() {
        solid.block(p, 0.011, 0.011, POST, 0.0, post, Some(family), 0.012);
        if i > 0 {
            solid.beam(line[i - 1] + Vec3::Y * RAIL, p + Vec3::Y * RAIL, 0.0065, rail);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgba = [0.6, 0.05, 0.05, 1.0];

    fn run(n: usize) -> MeshBuf {
        let mut b = MeshBuf::default();
        let line: Vec<Vec3> = (0..n).map(|i| Vec3::new(i as f32 * 0.1, 0.01 * i as f32, 0.0)).collect();
        fence(&mut b, &line, RED, &Palette::standard());
        assert!(b.is_sound());
        b
    }

    #[test]
    fn a_post_at_every_point_and_a_rail_between() {
        // a post is four sides and a cap, a rail four sides
        assert_eq!(run(1).triangles(), 10);
        assert_eq!(run(5).triangles(), 5 * 10 + 4 * 8);
        assert!(run(0).is_empty());
        // it follows the ground: the last post stands higher than the first
        let b = run(5);
        let (lo, hi) = b.bounds().unwrap();
        assert!((hi.y - (0.04 + POST)).abs() < 1e-6 && lo.y < 0.0);
        assert!((hi.x - lo.x - 0.411).abs() < 1e-5);
    }

    #[test]
    fn it_is_the_family_s_without_shouting() {
        let pal = Palette::standard();
        let b = run(3);
        // how far a colour is from grey
        let loud = |c: Rgba| c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2]);
        let caps = b.col.iter().filter(|c| **c == RED).count() / 4;
        assert_eq!(caps, 3);
        // every other face is quieter than the family's own colour, and the rail leans toward it
        let rail = lerp(*pal.marks.fence_timber, RED, FAMILY_IN_RAIL);
        assert!(b.col.iter().filter(|c| **c != RED).all(|c| loud(*c) < loud(RED) * 0.6));
        assert!(rail[0] > rail[1] * 1.5 && loud(rail) > loud(*pal.marks.fence_timber));
    }
}
