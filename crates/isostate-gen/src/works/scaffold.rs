use glam::Vec3;

use super::scale;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// Poles and boards round a works of this tier that is going up or being
/// enlarged, with a pile of timber and a heap of bricks beside it.
pub fn scaffold(solid: &mut MeshBuf, site: &Site, tier: u8, pal: &Palette) {
    let s = scale(tier);
    let (hw, hd, h) = (0.075 * s, 0.05 * s, 0.075);
    let pole = *pal.timber.scaffold_pole;
    let board = *pal.timber.scaffold_board;
    let corners = [site.at(-hw, -hd), site.at(hw, -hd), site.at(hw, hd), site.at(-hw, hd)];
    for c in corners {
        solid.beam(c - Vec3::Y * 0.01, c + Vec3::Y * h, 0.004, pole);
    }
    // mid poles on the long sides
    for c in [site.at(0.0, -hd), site.at(0.0, hd)] {
        solid.beam(c - Vec3::Y * 0.01, c + Vec3::Y * h, 0.004, pole);
    }
    // two lifts of boards all round
    for lift in [0.4, 0.85] {
        let y = Vec3::Y * (h * lift);
        for i in 0..4 {
            solid.beam(corners[i] + y, corners[(i + 1) % 4] + y, 0.005, board);
        }
    }
    // a diagonal brace on one face
    solid.beam(corners[0] + Vec3::Y * 0.004, corners[1] + Vec3::Y * (h * 0.85), 0.003, pole);
    // a pile of timber and a heap of bricks beside it
    let yard = site.at(hw + 0.035, 0.0);
    solid.block(yard, 0.03, 0.014, 0.008, site.yaw, board, Some(board), 0.004);
    solid.frustum(site.at(hw + 0.03, hd * 0.8), 0.012, 0.004, 0.012, 5, *pal.brick.red, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scaffold_is_bigger_round_a_bigger_works() {
        let pal = Palette::standard();
        let width = |tier: u8| {
            let mut b = MeshBuf::default();
            scaffold(&mut b, &Site::level(Vec3::ZERO, 0.0), tier, &pal);
            assert!(b.is_sound());
            let (lo, hi) = b.bounds().unwrap();
            hi.x - lo.x
        };
        assert!(width(4) > width(1));
    }
}
