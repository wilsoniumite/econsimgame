//! The houses of the suburbs: a pair of semi-detached houses, in the style
//! of the year their ground was laid out.

use crate::colour::{pick, shade};
use super::{door, pane_x, pane_z, win, Tones};
use crate::hash::Seed;
use crate::houses::{Detail, BACK_ROOM, SET_BACK};
use crate::kit::{End, Kit, Open, Range, RoofSpec, Side, Wall, FOOT, STOREY};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;
use crate::trees::{hedge_tree, Greenery};

/// The plot a pair stands on: wider than a terrace's two, and deeper.
pub const PAIR_PLOT: (f32, f32) = (0.094, 0.044);

/// What a suburb built, by the year its ground was laid out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Style {
    /// 1919 to 1944: roughcast over brick, two-storey bays under gables, hipped tile, a hedge of privet
    Interwar,
    /// 1945 to 1974: the council's plain brick pair, a canopy over the doors, an open lawn
    Council,
    /// 1975 on: the builder's brick box, a shallow gable, an integral garage and a drive
    Estate,
}

impl Style {
    pub fn of(built: i32) -> Style {
        match built {
            i32::MIN..=1944 => Style::Interwar,
            1945..=1974 => Style::Council,
            _ => Style::Estate,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Semis {
    /// the year the pair was built (its ground laid out): the style follows
    pub built: i32,
    /// a garage added at its side: where the households keep a car (the
    /// simulation does not know cars yet: see the notes)
    pub garage: bool,
    pub detail: Detail,
    pub seed: Seed,
}

/// A pair of houses under one roof, their party wall on the site's middle,
/// fronts to +z, on a plot `PAIR_PLOT` with its front garden before it and
/// its back gardens behind.
pub(crate) fn semis(solid: &mut MeshBuf, green: &mut Greenery, site: &Site, s: &Semis, pal: &Palette) {
    let t = Tones::of(pal);
    let r = |n: i32| s.seed.pick(n);
    let style = Style::of(s.built);
    let mut k = Kit::new(solid, site, pal, s.detail, 0.0);
    let (hw, zb, zf) = (0.0355, -0.016, 0.013);
    let h = 2.0 * STOREY;
    let body = Range::rect(-hw, hw, zb, zf);
    let y0 = -k.drop(&body);
    let frame = *pal.walls.window_frame;
    let sill = *pal.stone.sill;
    let brick = match style {
        Style::Interwar => pal.brick.terrace(0.6 + 0.4 * r(1)),
        Style::Council => pal.brick.terrace(r(1)),
        Style::Estate => shade(pal.brick.terrace(0.2 * r(1)), 0.94),
    };
    let rendered = style == Style::Interwar && r(2) < 0.8;
    let upper_col = if rendered { if r(3) < 0.6 { t.render } else { *pal.walls.cream } } else { brick };
    let lower = Wall { col: brick, plinth: None, frame, sill };
    let upper = Wall { col: upper_col, plinth: None, frame, sill };
    let paints = [pick(&pal.walls.door_paints, r(4)), pick(&pal.walls.door_paints, r(5))];
    let kind = Open::Casement;
    let wide = if style == Style::Estate { 0.0095 } else { 0.0078 };
    // each house: a bay (or a wide window) at its outer side, the door by the party wall
    let (bay_u, door_u, bay_w) = (0.0235, 0.0075, 0.0135);
    let mut fg = Vec::new();
    let mut fu = Vec::new();
    let mut bg = Vec::new();
    let mut bu = Vec::new();
    let mut ends_g = Vec::new();
    let mut ends_u = Vec::new();
    if !k.far() {
        for side in [-1.0f32, 1.0] {
            let at = |x: f32| x + hw;
            let (xb, xd) = (side * bay_u, side * door_u);
            if style != Style::Interwar {
                fg.push(win(at(xb), 0, wide, kind));
                fu.push(win(at(xb), 1, wide, kind));
            }
            fg.push(door(at(xd), paints[(side > 0.0) as usize]));
            fu.push(win(at(xd), 1, 0.0052, kind));
            // behind: the kitchen's window and door, two above
            let back = |x: f32| hw - x;
            bg.push(win(back(side * 0.026), 0, 0.0078, kind));
            bg.push(door(back(side * 0.012), shade(*pal.timber.door, 1.1)));
            bu.push(win(back(side * 0.026), 1, 0.0072, kind));
            bu.push(win(back(side * 0.011), 1, 0.0052, kind));
        }
        let dep = zf - zb;
        ends_g.push(win(dep * 0.35, 0, 0.0056, kind));
        ends_u.push(win(dep * 0.6, 1, 0.0052, kind));
        if style == Style::Estate && !s.garage {
            ends_g.push(door(dep * 0.75, shade(paints[0], 0.95)));
        }
    }
    k.walls(&body, y0, STOREY, &lower, [Side::with(&fg), Side::with(&ends_g), Side::with(&bg), Side::with(&ends_g)], true, false);
    k.walls(&body, STOREY, h, &upper, [Side::with(&fu), Side::with(&ends_u), Side::with(&bu), Side::with(&ends_u)], false, true);
    if !k.far() && rendered {
        // the line where the roughcast meets the brick
        for i in [0usize, 1, 2, 3] {
            let len = body.face(i).len;
            k.strip(&body, i, 0.0, len, STOREY - 0.0006, STOREY - 0.0006 + super::thick(&k, 0.001), 0.0005, shade(upper_col, 0.86));
        }
    }

    // the roof: hipped tile over the pair, or a shallow gable for the estate's box
    let roof_col = match style {
        Style::Interwar => if r(6) < 0.6 { *pal.roofs.tile_red } else { *pal.roofs.tile_dark },
        Style::Council => if r(6) < 0.5 { *pal.roofs.tile_dark } else { *pal.slate.house },
        Style::Estate => shade(*pal.roofs.tile_dark, 0.82),
    };
    let dep = zf - zb;
    let (rise, ends) = match style {
        Style::Interwar => (dep * 0.5 * 1.05, [End::Hip, End::Hip]),
        Style::Council => (dep * 0.5 * 0.8, if r(7) < 0.5 { [End::Hip, End::Hip] } else { [End::Gable, End::Gable] }),
        Style::Estate => (dep * 0.5 * 0.55, [End::Gable, End::Gable]),
    };
    k.roof(&body, &RoofSpec { h, rise, eaves: 0.0032, verge: 0.0024, ends, col: roof_col, gable: upper_col, thick: 0.0012 });
    let stack_col = if style == Style::Estate { brick } else { *pal.brick.chimney };
    if style != Style::Estate || r(8) < 0.3 {
        k.stack(0.0, (zb + zf) * 0.5 - 0.002, 0.009, 0.0058, h + rise * 0.6, h + rise + 0.0055, stack_col, 4);
    }

    if !k.far() {
        for side in [-1.0f32, 1.0] {
            let xb = side * bay_u;
            let (x0, x1) = (xb - bay_w * 0.5, xb + bay_w * 0.5);
            if style == Style::Interwar {
                // the bay, through both storeys, under a gable of its own with its timbers
                let out = 0.0042;
                k.boxy(x0, x1, zf, zf + out, y0, STOREY, brick, None);
                k.boxy(x0, x1, zf, zf + out, STOREY, h, upper_col, None);
                let glass = *pal.glass.window;
                for (ya, yb) in [(FOOT + 0.0052, STOREY - 0.0035), (STOREY + 0.0055, h - 0.0042)] {
                    pane_z(&mut k, x0 + 0.0011, x1 - 0.0011, zf + out + 0.0006, ya, yb, glass, 1.0);
                    pane_x(&mut k, x0 - 0.0006, zf + 0.0008, zf + out - 0.0008, ya, yb, shade(glass, 0.92), -1.0);
                    pane_x(&mut k, x1 + 0.0006, zf + 0.0008, zf + out - 0.0008, ya, yb, shade(glass, 0.86), 1.0);
                    if k.near() {
                        // the white frames' mullions
                        for m in 1..3 {
                            let x = x0 + (x1 - x0) * m as f32 / 3.0;
                            pane_z(&mut k, x - 0.0004, x + 0.0004, zf + out + 0.0009, ya, yb, frame, 1.0);
                        }
                    }
                }
                let gab = Range::rect_across(x0 - 0.0006, x1 + 0.0006, zf - 0.006, zf + out);
                k.roof(&gab, &RoofSpec { h, rise: (x1 - x0) * 0.5 * 1.1, eaves: 0.0026, verge: 0.0022, ends: [End::Party, End::Gable], col: roof_col, gable: upper_col, thick: 0.0012 });
                if k.near() {
                    // the gable's mock timbers
                    let oak = *pal.timber.house_frame;
                    let ridge = h + (x1 - x0) * 0.5 * 1.1;
                    pane_z(&mut k, xb - 0.0005, xb + 0.0005, zf + out + 0.0005, h, ridge - 0.0018, oak, 1.0);
                    pane_z(&mut k, x0 + 0.001, x1 - 0.001, zf + out + 0.0005, h + 0.0004, h + 0.0012, oak, 1.0);
                }
            }
            // a hood over the door
            let xd = side * door_u;
            let (ph, reach) = (FOOT + 0.0142, if style == Style::Estate { 0.003 } else { 0.0045 });
            let canopy = if style == Style::Interwar { roof_col } else { t.concrete };
            k.boxy(xd - 0.0048, xd + 0.0048, zf, zf + reach, ph, ph + 0.0014, canopy, Some(shade(canopy, 1.04)));
        }
    }

    // a garage at each side where the households keep a car: a flat roof, an up-and-over door
    if s.garage || style == Style::Estate {
        for side in [-1.0f32, 1.0] {
            let (gx0, gx1) = if side > 0.0 { (hw + 0.0004, hw + 0.0118) } else { (-hw - 0.0118, -hw - 0.0004) };
            let (gz0, gz1) = (zb + 0.004, zf + if style == Style::Estate { 0.0 } else { -0.002 });
            let gh = 0.0128;
            let gw = Wall { col: brick, plinth: None, frame, sill };
            let grg = Range::rect(gx0, gx1, gz0, gz1);
            let leaf = if style == Style::Estate { t.cladding_white } else { shade(paints[(side > 0.0) as usize], 1.1) };
            let gd = [crate::kit::Opening { u: (gx1 - gx0) * 0.5, y: 0.0, w: 0.0086, h: 0.0098, kind: Open::Leaves { paint: leaf, round: false } }];
            let front = if k.far() { Side::PLAIN } else { Side::with(&gd) };
            let inner = Side::NONE;
            let sides = if side > 0.0 { [front, Side::PLAIN, Side::PLAIN, inner] } else { [front, inner, Side::PLAIN, Side::PLAIN] };
            k.walls(&grg, y0, gh, &gw, sides, true, false);
            super::flat_roof(&mut k, gx0, gx1, gz0, gz1, gh, 0.0012, shade(brick, 0.9), t.felt);
        }
    }

    // gardens: a tree behind, lawns before and behind, and close to a hedge, paths and drives
    let rear = -PAIR_PLOT.1 * 0.5 - BACK_ROOM;
    let front = PAIR_PLOT.1 * 0.5 + SET_BACK - 0.0012;
    let px = PAIR_PLOT.0 * 0.5 - 0.0008;
    if r(9) < 0.7 {
        let x = if r(10) < 0.5 { -0.03 } else { 0.03 };
        let p = k.pt(x, 0.0, rear + 0.008);
        hedge_tree(green, &site.moved(p, 0.0), 0.26 + 0.1 * r(11), r(12) * 3.0, r(13) * 6.0, pal);
    }
    if k.far() {
        return;
    }
    k.patch(-px, px, rear, zb - 0.0005, 0.0034, *pal.ground.garden);
    k.patch(-px, px, zf + 0.0045, front, 0.0034, *pal.ground.garden);
    if !k.near() {
        return;
    }
    let drive = if style == Style::Interwar { *pal.ground.garden_path } else { t.tarmac_light };
    for side in [-1.0f32, 1.0] {
        let xd = side * door_u;
        k.patch(xd - 0.0018, xd + 0.0018, zf + 0.004, front, 0.004, *pal.ground.garden_path);
        let (d0, d1) = if side > 0.0 { (hw + 0.001, hw + 0.0112) } else { (-hw - 0.0112, -hw - 0.001) };
        k.patch(d0, d1, zf + 0.001, front, 0.0042, drive);
        // the fence between the back gardens
        k.low_wall((0.0, zb - 0.001), (0.0, rear + 0.001), 0.0042, 0.0009, *pal.timber.fence);
    }
    if style == Style::Interwar {
        let hedge = *pal.foliage.hedge;
        for (a, b) in [(-hw, -door_u - 0.003), (-door_u + 0.003, door_u - 0.003), (door_u + 0.003, hw)] {
            if b - a > 0.002 {
                k.low_wall((a, front), (b, front), 0.0052, 0.0026, hedge);
            }
        }
    }
}
