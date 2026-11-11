use crate::colour::{pick, shade};
use crate::hash::Seed;
use crate::houses::cottage::back_door;
use crate::houses::kit::{Detail, End, Kit, Open, Opening, Over, Range, RoofSpec, Side, Wall, FOOT};
use crate::houses::roof::Cover;
use crate::houses::{door_at, hearth_smoke, window_at};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::STOREY;
use crate::site::Site;

/// The house of a farmstead. The site is the middle of it; its front is +z.
#[derive(Clone, Copy, Debug)]
pub struct Farmhouse {
    /// one or two
    pub storeys: u8,
    /// it stands in the hills: stone and slate in place of wash, tile and thatch
    pub upland: bool,
    /// a lowland house under thatch rather than tile
    pub thatch: bool,
    /// a bigger farm's: a wing stands forward at one end
    pub wing: bool,
    /// how much of it to draw
    pub detail: Detail,
    pub seed: Seed,
}

/// How wide and how deep a farmhouse is.
pub const FARMHOUSE: (f32, f32) = (0.05, 0.028);

/// A farmhouse: a long low range, washed in the lowland under thatch or
/// tile, stone under slate in the hills, a stack at each end, a porch to
/// its door, and an outshut along the back under a catslide (a dairy, a
/// wash-house). A bigger farm's has a wing standing forward at one end, its
/// roof running back into the house's in valleys. It is old, and a little
/// crooked.
pub fn farmhouse(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, f: &Farmhouse, pal: &Palette) {
    let Seed { u, v } = f.seed;
    let r = |n: i32| f.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, f.detail, 0.0);
    let (w, d) = FARMHOUSE;
    let (hw, zf, zb) = (w * 0.5, d * 0.5, -d * 0.5);
    let n = f.storeys.clamp(1, 2) as usize;
    let (frame, sill) = (*pal.walls.window_frame, *pal.stone.sill);
    let wall = if f.upland { Wall { col: pick(&pal.stone.upland_farm, v), plinth: None, frame, sill } } else { Wall { col: pal.walls.farmhouse(v), plinth: Some(*pal.stone.plinth), frame, sill } };
    let (colour, cover) = if f.upland { (*pal.slate.upland_farm, Cover::Slate) } else if f.thatch { (*pal.thatch.farmhouse, Cover::Thatch) } else { (*pal.roofs.tile_farm, Cover::Tile) };
    let lie = cover.lie();
    let h = STOREY * n as f32 * if f.thatch { 0.94 } else { 1.0 };
    let rise = d * lie.rise;
    let body = Range::rect(-hw, hw, zb, zf);
    let y0 = -k.drop(&body);
    if !f.upland {
        k.crooked(((r(9) - 0.5) * 0.03, (r(10) - 0.5) * 0.02), 0.0018 * if f.thatch { 1.4 } else { 1.0 }, hw + lie.verge, h, h + rise);
    }
    let glazing = if f.thatch { Open::Casement } else { Open::Sash };
    // the wing stands forward of one end; the door is in the middle of what is left
    let wing_right = u < 0.5;
    let (wing_w, wing_d) = (0.019, 0.011);
    let (wx0, wx1) = if wing_right { (hw - wing_w, hw) } else { (-hw, -hw + wing_w) };
    let before_wing = |x: f32| f.wing && x > wx0 && x < wx1;
    let outshut = !k.far() && r(6) < 0.7;
    let (ox0, ox1) = if r(7) < 0.5 { (-hw, -hw + w * 0.62) } else { (hw - w * 0.62, hw) };
    let under = |x: f32| outshut && x > ox0 && x < ox1;
    let mut front = Vec::new();
    let mut back = Vec::new();
    let door_bay = if f.wing { if wing_right { 0 } else { 2 } } else { 1 };
    if !k.far() {
        for b in 0..3 {
            let at = w * (b as f32 + 0.5) / 3.0;
            let (x, bx) = (-hw + at, hw - at);
            if !before_wing(x) {
                if b == door_bay {
                    front.push(door_at(at, Open::Door { paint: pick(&pal.walls.door_paints, r(4)), over: Over::Nothing }));
                } else {
                    front.push(window_at(at, 0, 0.0086, 0.006, 0.0078, glazing));
                }
                for s in 1..n {
                    front.push(window_at(at, s, 0.0074, 0.006, 0.0076, glazing));
                }
            }
            for s in 1..n {
                back.push(window_at(at, s, 0.0074, 0.0056, 0.0076, glazing));
            }
            if !under(bx) {
                back.push(if b == 1 { door_at(at, back_door(pal)) } else { window_at(at, 0, 0.0086, 0.0056, 0.0078, glazing) });
            }
        }
    }
    k.walls(&body, y0, h, &wall, [Side::with(&front), Side::PLAIN, Side::with(&back), Side::PLAIN], true, true);
    let end = if f.thatch && !k.far() && r(8) < 0.6 { End::HalfHip } else { End::Gable };
    let spec = RoofSpec { h, rise, eaves: lie.eaves, verge: lie.verge, ends: [end, end], col: colour, gable: wall.col, thick: lie.thick };
    k.roof(&body, &spec);

    if f.wing {
        // its gable to the front, its ridge running back to meet the house's front slope
        let rg = Range::rect(wx0, wx1, zf, zf + wing_d);
        let ops: Vec<Opening> = if k.far() { Vec::new() } else { (0..n).map(|s| window_at(wing_w * 0.5, s, 0.0082, 0.006, 0.0078, glazing)).collect() };
        let (right, left) = if wing_right { (Side::PLAIN, Side::PLAIN.dark(true, false)) } else { (Side::PLAIN.dark(false, true), Side::PLAIN) };
        k.walls(&rg, -k.drop(&rg), h, &wall, [Side::with(&ops), right, Side::NONE, left], true, true);
        let wing_rise = (wing_w * 0.5 * spec.pitch(&body)).min(rise * 0.9);
        let run = d * 0.5 * wing_rise / rise;
        let roof = Range { cx: (wx0 + wx1) * 0.5, cz: zf + wing_d * 0.5, len: wing_d, dep: wing_w, turn: 3 };
        k.roof(&roof, &RoofSpec { h, rise: wing_rise, eaves: lie.eaves * 0.8, verge: lie.verge, ends: [End::Gable, End::Valley(run)], col: colour, gable: wall.col, thick: lie.thick });
    }
    if k.far() {
        return;
    }

    if outshut {
        let depth = 0.0082;
        let rg = Range { cx: (ox0 + ox1) * 0.5, cz: zb - depth * 0.5, len: ox1 - ox0, dep: depth, turn: 2 };
        let (high, low) = if n == 1 { (h, (h - depth * spec.pitch(&body) * 0.8).max(0.0105)) } else { (STOREY + 0.0045, 0.0125) };
        let ops = [Opening { u: (ox1 - ox0) * 0.3, y: FOOT, w: 0.0046, h: (low - FOOT - 0.0036).min(0.012), kind: back_door(pal) }, Opening { u: (ox1 - ox0) * 0.7, y: FOOT + 0.0035, w: 0.0046, h: (low - FOOT - 0.0072).max(0.002), kind: glazing }];
        k.lean_to(&rg, low, high, &wall, shade(colour, 0.82), lie.eaves * 0.8, Side::with(&ops), [true, true]);
    }
    // the porch: a little gabled range before the door
    let door_x = -hw + w * (door_bay as f32 + 0.5) / 3.0;
    let (px, pz) = (0.0042, 0.0046);
    let porch = Range::rect(door_x - px, door_x + px, zf, zf + pz);
    let way_in = [Opening { u: px, y: FOOT, w: 0.005, h: 0.012, kind: Open::Arch }];
    k.walls(&porch, -k.drop(&porch), FOOT + 0.0142, &wall, [Side::with(&way_in), Side::PLAIN, Side::NONE, Side::PLAIN], true, false);
    let porch_roof = Range { cx: door_x, cz: zf + pz * 0.5, len: pz, dep: px * 2.0, turn: 3 };
    k.roof(&porch_roof, &RoofSpec { h: FOOT + 0.0142, rise: px * 0.9, eaves: 0.0012, verge: 0.0014, ends: [End::Gable, End::Party], col: colour, gable: wall.col, thick: lie.thick * 0.6 });

    // a stack at each end, astride the ridge, and the kitchen's is alight more often than not
    let stack = if f.upland { shade(wall.col, 0.9) } else { *pal.brick.chimney };
    let inset = if end == End::HalfHip { 0.011 } else { 0.0046 };
    let top = k.stack(hw - inset, 0.0, 0.0058, 0.0056, h + rise * 0.45, h + rise + 0.0058, stack, 2);
    k.stack(-hw + inset, 0.0, 0.0056, 0.0054, h + rise * 0.45, h + rise + 0.005, stack, 1);
    if r(11) < 0.55 {
        hearth_smoke(smoke, top, f.seed, pal);
    }
    if k.near() {
        // a butt at the back corner, and wood stacked against the end wall
        if !under(hw - 0.003) {
            k.butt(hw - 0.003, zb - 0.0026);
        }
        k.woodpile(-hw - 0.0032, -d * 0.3, 0.003, d * 0.5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::Model;
    use glam::Vec3;

    fn built(f: &Farmhouse) -> Model {
        let mut m = Model::default();
        farmhouse(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), f, &Palette::standard());
        assert!(m.is_sound());
        m
    }

    #[test]
    fn it_keeps_to_its_ground_at_every_level() {
        for n in 0..40 {
            let seed = Seed::of(n);
            let mut last = 0;
            for detail in Detail::ALL {
                let f = Farmhouse { storeys: 1 + (n % 2) as u8, upland: n % 3 == 0, thatch: n % 4 == 1, wing: n % 5 < 2, detail, seed };
                let m = built(&f);
                let (lo, hi) = m.solid.bounds().unwrap();
                assert!(lo.x > -0.034 && hi.x < 0.034, "seed {n}: from {} to {}", lo.x, hi.x);
                // the outshut behind and the wing before
                assert!(lo.z > -0.027 && hi.z < 0.03, "seed {n} {detail:?}: from {} to {}", lo.z, hi.z);
                assert!(m.solid.triangles() > last);
                last = m.solid.triangles();
            }
        }
    }

    #[test]
    fn a_wing_s_ridge_runs_back_into_the_house_s_roof() {
        let pal = Palette::standard();
        let seed = Seed::new(0.2, 0.3);
        let (plain, winged) = (built(&Farmhouse { storeys: 2, upland: true, thatch: false, wing: false, detail: Detail::Far, seed }), built(&Farmhouse { storeys: 2, upland: true, thatch: false, wing: true, detail: Detail::Far, seed }));
        // three more walls, two slopes, a gable and the two valleys
        assert_eq!(winged.solid.triangles(), plain.solid.triangles() + 3 * 2 + 2 * 2 + 1 + 2);
        // the wing's roof reaches behind the front wall, onto the house's own slope
        let slate = *pal.slate.upland_farm;
        let behind = winged.solid.pos.iter().zip(&winged.solid.col).filter(|(p, c)| **c == slate && p[0] > 0.01 && p[2] > 0.0 && p[2] < FARMHOUSE.1 * 0.5 - 0.001).count();
        assert!(behind > plain.solid.pos.iter().zip(&plain.solid.col).filter(|(p, c)| **c == slate && p[0] > 0.01 && p[2] > 0.0 && p[2] < FARMHOUSE.1 * 0.5 - 0.001).count());
    }

    #[test]
    fn hill_farmhouses_are_stone_and_slate_and_lowland_ones_washed() {
        let pal = Palette::standard();
        let seed = Seed::new(0.5, 0.3);
        let (low, high) = (built(&Farmhouse { storeys: 1, upland: false, thatch: true, wing: false, detail: Detail::Play, seed }), built(&Farmhouse { storeys: 1, upland: true, thatch: false, wing: false, detail: Detail::Play, seed }));
        assert!(high.solid.col.contains(&*pal.slate.upland_farm) && !low.solid.col.contains(&*pal.slate.upland_farm));
        assert!(low.solid.col.contains(&*pal.thatch.farmhouse));
    }
}
