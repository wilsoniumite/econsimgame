use super::{Kind, Works};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::smoking_chimney;
use crate::site::Site;

/// Ironmaking: a bloomery's low stone stack, a charcoal blast furnace's tall
/// one, or a row of coke furnaces (one for each tier) with their chimneys and
/// a bank of coke ovens. Each has a casting shed in front.
pub fn furnace(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) {
    let (yaw, s, tier) = (site.yaw, w.scale(), w.tier as f32);
    let brick = *pal.brick.red;
    let coke = w.kind == Kind::CokeFurnace;
    let n = if coke { tier as usize } else { 1 };
    let fh = if w.kind == Kind::Bloomery { 0.04 } else { 0.075 };
    for i in 0..n {
        solid.frustum(site.at(-0.03 + i as f32 * 0.045, -0.015), 0.022 * s, 0.014 * s, fh, 4, *pal.stone.works, Some(*pal.smoke.soot));
    }
    solid.block(site.at(0.02, 0.025), 0.07 * s, 0.04, 0.025, yaw, if coke { brick } else { *pal.timber.boards }, None, 0.03);
    solid.gable(site.at(0.02, 0.025), 0.07 * s, 0.04, 0.025, 0.018, yaw, 0.003, *pal.slate.works, brick);
    if coke {
        for i in 0..tier as usize {
            smoking_chimney(solid, smoke, pal, site.at(0.06, -0.03 + i as f32 * 0.02), 0.13 + 0.02 * tier, 0.008, w.seed + i as f32);
        }
        solid.block(site.at(-0.02, 0.06), 0.11 * s, 0.022, 0.014, yaw, *pal.brick.dark, Some(*pal.roofs.coke_ovens), 0.02);
    }
}
