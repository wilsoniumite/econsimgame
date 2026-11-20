//! Blocks of flats over shops and an office of the new century, from the
//! "dense" direction: the forms a town takes after 1900 where it thickens
//! before it spreads. Each to the houses' scale and built from their kit.
//!
//! - **`block`**: a perimeter block of flats round a green court, four to
//!   seven storeys, shops under the flats on the faces that look onto a busy
//!   street. Its street faces are cut into sections about a plot wide, each
//!   with its own wall from the block's family, so that a block reads as a
//!   row of tall houses and not as one slab. A block too shallow for a court
//!   is one deep range: flats over shops on a few lots thrown together.
//! - **`tower`**: offices on a podium of shops; the catalogue takes the one
//!   of the new century, with its terracotta fins and planted terraces.
//!
//! The age a block was built in (`Age`) sets its style for good, as `Era`
//! does a house's: brick and stucco under steep slate with turrets on the
//! corners to 1918; washed render under hipped tile to 1945; pale brick with
//! balconies under flat roofs to 1980; render and boards with a top storey
//! set back under a green roof after.

use glam::Vec3;

use crate::colour::{pick, shade, Rgba};
use crate::hash::Seed;
use crate::kit::{
    Detail, End, Kit, Open, Opening, Over, Range, RoofSpec, Side, Wall, FOOT, STOREY,
};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;
use crate::trees::{hedge_tree, Greenery};

/// The age a block or a tower was built in: its style, kept for good.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Age {
    /// 1895 to 1918: brick and stucco, tall sashes, steep slate with dormers, a turret on each corner
    Reform,
    /// 1919 to 1945: washed render, wide casements, hipped tile, bands of stone
    Interwar,
    /// 1946 to 1979: pale brick, wide windows, balconies, flat roofs
    Postwar,
    /// 1980 on: render and boards, the top storey set back, green roofs and panels
    Renewal,
}

impl Age {
    pub fn of(year: i32) -> Age {
        match year {
            i32::MIN..=1918 => Age::Reform,
            1919..=1945 => Age::Interwar,
            1946..=1979 => Age::Postwar,
            _ => Age::Renewal,
        }
    }
}

/// How deep a wing of flats is, wall to wall: a townhouse's depth.
pub const WING: f32 = 0.028;
/// The least a court is across: less, and the block is one deep range.
pub const COURT_MIN: f32 = 0.03;
/// The depth of flats over shops on lots thrown together: one deep range.
pub const RANGE_DEPTH: f32 = 0.034;
/// The ground a courtyard block takes when nothing else is said: a block
/// between streets, with a court of 0.084 by 0.054 inside its wings.
pub const COURT_GROUND: (f32, f32) = (0.14, 0.11);
/// How far apart the windows of a block stand along its walls.
const BAY_W: f32 = 0.0135;

/// A perimeter block.
#[derive(Clone, Copy, Debug)]
pub struct Block {
    /// along its front street (x), and back from it (z), wall to wall
    pub width: f32,
    pub depth: f32,
    pub storeys: u8,
    pub age: Age,
    /// a shop under the flats on its front, right, back and left faces (seen from the front):
    /// the faces that look onto a street that carries trams or traffic
    pub shops: [bool; 4],
    /// a house stands against it on its left and on its right (an infill block): no wall drawn there
    pub neighbours: (bool, bool),
    /// it was raised a storey after 1990 (its cell's dens came to give it two more than it has):
    /// the new storey is of boards with wide windows, and a flat roof over it is green
    pub raised: bool,
    pub detail: Detail,
    pub seed: Seed,
}

impl Block {
    pub fn new(width: f32, depth: f32, storeys: u8, age: Age, seed: Seed) -> Block {
        Block {
            width,
            depth,
            storeys,
            age,
            shops: [true, false, false, false],
            neighbours: (false, false),
            raised: false,
            detail: Detail::Play,
            seed,
        }
    }
}

/// The way a range's `a` runs in the block's frame, for each quarter turn.
fn axis(turn: u8) -> (f32, f32) {
    match turn % 4 {
        0 => (1.0, 0.0),
        1 => (0.0, -1.0),
        2 => (-1.0, 0.0),
        _ => (0.0, 1.0),
    }
}

/// The part of a range from `a0` to `a1` along it.
fn part(rg: &Range, a0: f32, a1: f32) -> Range {
    let (ax, az) = axis(rg.turn);
    let m = (a0 + a1) * 0.5;
    Range {
        cx: rg.cx + ax * m,
        cz: rg.cz + az * m,
        len: a1 - a0,
        dep: rg.dep,
        turn: rg.turn,
    }
}

/// A window of storey `s` at `u` along its wall.
fn win(u: f32, s: usize, w: f32, h: f32, kind: Open) -> Opening {
    Opening {
        u,
        y: s as f32 * STOREY + 0.006,
        w,
        h,
        kind,
    }
}

fn door_at(u: f32, kind: Open) -> Opening {
    Opening {
        u,
        y: FOOT,
        w: 0.0056,
        h: 0.0128,
        kind,
    }
}

/// What a block of an age is built of and how its windows go.
struct Style {
    glazing: Open,
    /// window width and height
    ww: f32,
    wh: f32,
}

fn style(age: Age) -> Style {
    match age {
        Age::Reform => Style {
            glazing: Open::Sash,
            ww: 0.0068,
            wh: 0.0095,
        },
        Age::Interwar => Style {
            glazing: Open::Casement,
            ww: 0.0088,
            wh: 0.0078,
        },
        Age::Postwar => Style {
            glazing: Open::Casement,
            ww: 0.0098,
            wh: 0.0082,
        },
        Age::Renewal => Style {
            glazing: Open::Casement,
            ww: 0.0084,
            wh: 0.0104,
        },
    }
}

/// The wall of one section of a block: the block's family (`fam`) and a number of the section's own (`own`).
fn section_wall(pal: &Palette, age: Age, fam: Seed, own: f32, k: f32) -> Wall {
    let (frame, sill) = (*pal.walls.window_frame, *pal.stone.sill);
    // half the sections take the block's colour, the rest one of their own
    let v = if own < 0.5 {
        fam.u
    } else {
        (own * 7.3).fract()
    };
    match age {
        Age::Reform => {
            if fam.pick(1) < 0.6 && k < 0.7 {
                Wall {
                    col: pal.brick.terrace(v),
                    plinth: Some(*pal.stone.plinth),
                    frame,
                    sill,
                }
            } else {
                Wall {
                    col: pal.walls.town_house(v),
                    plinth: Some(*pal.stone.plinth),
                    frame,
                    sill,
                }
            }
        }
        Age::Interwar => Wall {
            col: pal.walls.town_house(v),
            plinth: Some(*pal.brick.dark),
            frame,
            sill,
        },
        Age::Postwar => {
            let brick = if v < 0.5 { *pal.brick.pale } else { shade(*pal.brick.pale, 0.94) };
            Wall {
                col: if k < 0.6 {
                    brick
                } else {
                    pal.walls.town_house(v)
                },
                plinth: Some(*pal.brick.dark),
                frame,
                sill,
            }
        }
        Age::Renewal => Wall {
            col: pal.walls.town_house(v),
            plinth: Some(*pal.stone.plinth),
            frame,
            sill,
        },
    }
}

/// Green that is grown on a roof.
fn roof_green(pal: &Palette) -> Rgba {
    *pal.modern.green_roof
}

/// Felt on a flat roof.
fn felt(pal: &Palette) -> Rgba {
    *pal.modern.gravel
}

/// One wing of a block: a range, its turn putting +c to the street, and
/// whether its ends are open to a side street (the front and back wings) or
/// stand against the other wings.
struct WingOf {
    rg: Range,
    open_ends: bool,
    face: usize,
}

/// A perimeter block of flats round a green court (or, too shallow for a
/// court, one deep range), on a site whose origin is the middle of the
/// block. Front to +z as every generator's.
///
/// From far off: the walls and the roofs of the four wings. At play: the
/// windows of every storey, the shops and their awnings, dormers, stacks,
/// balconies, the turrets on the corners, the court's garden and its trees.
/// Close to, the windows let into the walls with their bars and sills.
pub(crate) fn block(
    solid: &mut MeshBuf,
    flat: &mut MeshBuf,
    green: &mut Greenery,
    site: &Site,
    b: &Block,
    pal: &Palette,
) {
    let (hw, hd) = (b.width * 0.5, b.depth * 0.5);
    let n = b.storeys.clamp(3, 9) as usize;
    let st = style(b.age);
    let fam = b.seed;
    let court = b.depth >= 2.0 * WING + COURT_MIN && b.width >= 2.0 * WING + COURT_MIN;
    let renewal = b.age == Age::Renewal;
    // a block of the last age sets its top storey back from the street
    let walls_n = if renewal { n - 1 } else { n };
    // a storey added on the roof after 1990: the old walls stop at `h_old`, the roof stands on the new
    let raised = b.raised && !renewal;
    let h_old = STOREY * walls_n as f32;
    let h = h_old + if raised { STOREY } else { 0.0 };
    let boards = Wall { col: *pal.modern.cladding_timber, plinth: None, frame: *pal.walls.window_frame, sill: *pal.stone.sill };

    let wings: Vec<WingOf> = if court {
        let side_len = b.depth - 2.0 * WING;
        vec![
            WingOf {
                rg: Range {
                    cx: 0.0,
                    cz: hd - WING * 0.5,
                    len: b.width,
                    dep: WING,
                    turn: 0,
                },
                open_ends: true,
                face: 0,
            },
            WingOf {
                rg: Range {
                    cx: hw - WING * 0.5,
                    cz: 0.0,
                    len: side_len,
                    dep: WING,
                    turn: 1,
                },
                open_ends: false,
                face: 1,
            },
            WingOf {
                rg: Range {
                    cx: 0.0,
                    cz: -hd + WING * 0.5,
                    len: b.width,
                    dep: WING,
                    turn: 2,
                },
                open_ends: true,
                face: 2,
            },
            WingOf {
                rg: Range {
                    cx: -hw + WING * 0.5,
                    cz: 0.0,
                    len: side_len,
                    dep: WING,
                    turn: 3,
                },
                open_ends: false,
                face: 3,
            },
        ]
    } else {
        // one deep range, its front to the street
        vec![WingOf {
            rg: Range {
                cx: 0.0,
                cz: 0.0,
                len: b.width,
                dep: b.depth,
                turn: 0,
            },
            open_ends: true,
            face: 0,
        }]
    };

    let mut k = Kit::new(solid, site, pal, b.detail, 0.0);
    let roof_col = match b.age {
        Age::Reform => *pal.slate.town,
        Age::Interwar => {
            if fam.pick(3) < 0.5 {
                *pal.roofs.tile_red
            } else {
                *pal.roofs.tile_orange
            }
        }
        Age::Postwar => felt(pal),
        Age::Renewal => roof_green(pal),
    };

    for (wi, w) in wings.iter().enumerate() {
        let rg = w.rg;
        let shops = b.shops[w.face] && (wi == 0 || court);
        // sections a plot or so long
        let count = ((rg.len / 0.05).round() as usize).max(1);
        let mut cuts = vec![-rg.len * 0.5];
        for i in 1..count {
            let jig = (fam.pick(40 + wi as i32 * 13 + i as i32) - 0.5) * 0.012;
            cuts.push(-rg.len * 0.5 + rg.len * i as f32 / count as f32 + jig);
        }
        cuts.push(rg.len * 0.5);
        for i in 0..count {
            let (a0, a1) = (cuts[i], cuts[i + 1]);
            let sec = part(&rg, a0, a1);
            let own = fam.pick(60 + wi as i32 * 17 + i as i32);
            let wall = section_wall(
                pal,
                b.age,
                fam,
                own,
                fam.pick(70 + wi as i32 * 17 + i as i32),
            );
            let len = a1 - a0;
            let bays = ((len / BAY_W).floor() as usize).max(2);
            let mut street = Vec::new();
            let mut yard = Vec::new();
            let mut end_ops = Vec::new();
            if !k.far() {
                if shops {
                    let paint = pick(&pal.walls.door_paints, own);
                    let awning = (fam.pick(80 + i as i32) < 0.55)
                        .then(|| pick(&pal.roofs.awnings, own * 3.1));
                    street.push(Opening {
                        u: len * 0.5,
                        y: FOOT,
                        w: len * 0.78,
                        h: 0.0136,
                        kind: Open::Shop {
                            paint: if own < 0.3 {
                                *pal.walls.shop_front
                            } else {
                                paint
                            },
                            awning,
                        },
                    });
                }
                for bay in 0..bays {
                    let u = len * (bay as f32 + 0.5) / bays as f32;
                    if !shops {
                        if bay == bays / 2 {
                            street.push(door_at(
                                u,
                                Open::Door {
                                    paint: pick(&pal.walls.door_paints, own),
                                    over: if b.age == Age::Reform {
                                        Over::Fanlight
                                    } else {
                                        Over::Nothing
                                    },
                                },
                            ));
                        } else {
                            street.push(win(u, 0, st.ww, st.wh * 0.9, st.glazing));
                        }
                    }
                    for s in 1..walls_n {
                        street.push(win(
                            u,
                            s,
                            st.ww,
                            if s == walls_n - 1 && b.age == Age::Reform {
                                st.wh * 0.8
                            } else {
                                st.wh
                            },
                            st.glazing,
                        ));
                    }
                    for s in 0..walls_n {
                        if s == 0 && bay == 0 {
                            yard.push(door_at(
                                u,
                                Open::Door {
                                    paint: *pal.timber.door,
                                    over: Over::Nothing,
                                },
                            ));
                        } else {
                            yard.push(win(u, s, st.ww * 0.9, st.wh * 0.9, st.glazing));
                        }
                    }
                }
                for s in 0..walls_n {
                    end_ops.push(win(WING * 0.5, s, st.ww, st.wh, st.glazing));
                }
            }
            let first = i == 0;
            let last = i == count - 1;
            // the +a end (side 1) shows on the last section of an open wing, the -a end (side 3) on the first
            let left_party = !court && b.neighbours.0;
            let right_party = !court && b.neighbours.1;
            let s1 = if w.open_ends && last && !right_party {
                Side::with(&end_ops)
            } else {
                Side::NONE
            };
            let s3 = if w.open_ends && first && !left_party {
                Side::with(&end_ops)
            } else {
                Side::NONE
            };
            let y0 = -k.drop(&sec);
            k.walls(
                &sec,
                y0,
                h_old,
                &wall,
                [Side::with(&street), s1, Side::with(&yard), s3],
                true,
                b.age <= Age::Interwar && !raised,
            );
            if raised {
                let mut top = Vec::new();
                if !k.far() {
                    for bay in 0..bays {
                        top.push(win(len * (bay as f32 + 0.5) / bays as f32, walls_n, 0.0098, 0.0105, Open::Casement));
                    }
                }
                let ends: Vec<Opening> = Vec::new();
                let e1 = if s1.show { Side::with(&ends) } else { Side::NONE };
                let e3 = if s3.show { Side::with(&ends) } else { Side::NONE };
                k.walls(&sec, h_old - 0.0004, h, &boards, [Side::with(&top), e1, Side::with(&top), e3], false, b.age <= Age::Interwar);
            }
            // stone bands between the storeys of the older ages
            if !k.far() && b.age <= Age::Interwar {
                let band = if b.age == Age::Reform {
                    wall.sill
                } else {
                    shade(wall.col, 0.9)
                };
                k.strip(
                    &sec,
                    0,
                    0.0,
                    len,
                    STOREY - 0.0008,
                    STOREY - 0.0008 + super::thick(&k, 0.0012),
                    0.0005,
                    band,
                );
                if b.age == Age::Reform {
                    k.strip(
                        &sec,
                        0,
                        0.0,
                        len,
                        h - STOREY - 0.0006,
                        h - STOREY - 0.0006 + super::thick(&k, 0.001),
                        0.0005,
                        band,
                    );
                }
            }
            // a balcony to each flat on the street of the postwar age, and on the court after
            if !k.far() && b.age >= Age::Postwar {
                let rail = if b.age == Age::Postwar {
                    pick(&pal.modern.panels, fam.pick(90))
                } else {
                    *pal.modern.cladding_timber
                };
                for bay in 0..bays {
                    if bay % 2 == (i % 2) {
                        continue;
                    }
                    let u = len * (bay as f32 + 0.5) / bays as f32;
                    for s in 1..walls_n {
                        let side = if b.age == Age::Postwar { 0 } else { 2 };
                        balcony(&mut k, &sec, side, u, s, rail, pal);
                    }
                }
            }
        }

        // the roof over the whole wing
        match b.age {
            Age::Reform | Age::Interwar => {
                let steep = if b.age == Age::Reform { 0.95 } else { 0.62 };
                let rise = rg.dep * 0.5 * steep;
                let end = if w.open_ends {
                    End::Hip
                } else {
                    End::Valley(WING * 0.5)
                };
                let ends = if court {
                    [end, end]
                } else {
                    [
                        if b.neighbours.1 { End::Party } else { End::Hip },
                        if b.neighbours.0 { End::Party } else { End::Hip },
                    ]
                };
                let lie_thick = if b.age == Age::Reform { 0.0009 } else { 0.0012 };
                let gable = section_wall(pal, b.age, fam, 0.0, 0.0).col;
                let spec = RoofSpec {
                    h,
                    rise,
                    eaves: 0.0026,
                    verge: 0.002,
                    ends,
                    col: roof_col,
                    gable,
                    thick: lie_thick,
                };
                k.roof(&rg, &spec);
                if b.age == Age::Reform && !k.far() {
                    // dormers in the street slope, a bay apart
                    let dn = ((rg.len - 0.03) / 0.022).floor().max(0.0) as usize;
                    for d in 0..dn {
                        let a = -rg.len * 0.5 + 0.015 + 0.022 * (d as f32 + 0.5);
                        k.dormer(
                            &rg,
                            &spec,
                            a,
                            0.0056,
                            1.0,
                            0.18,
                            &section_wall(pal, b.age, fam, 0.0, 0.0),
                        );
                    }
                }
                if !k.far() {
                    // stacks on the ridge at every other section's party wall
                    let count = ((rg.len / 0.05).round() as usize).max(1);
                    for i in 1..count {
                        if i % 2 == 1 {
                            let a = -rg.len * 0.5 + rg.len * i as f32 / count as f32;
                            let (x, z) = rg.at(a, 0.0);
                            k.stack(
                                x,
                                z,
                                0.0056,
                                0.0072,
                                h + rise * 0.6,
                                h + rise + 0.006,
                                *pal.brick.chimney,
                                3,
                            );
                        }
                    }
                }
            }
            Age::Postwar => {
                // a flat roof behind a low parapet, and the lift's motor room
                flat_top(
                    &mut k,
                    &rg,
                    h,
                    0.0022,
                    if raised { roof_green(pal) } else { roof_col },
                    if raised { boards.col } else { section_wall(pal, b.age, fam, 0.0, 0.0).col },
                );
                if !k.far() && wi % 2 == 0 {
                    let (x, z) = rg.at(rg.len * 0.18, 0.0);
                    k.boxy(
                        x - 0.005,
                        x + 0.005,
                        z - 0.006,
                        z + 0.006,
                        h,
                        h + 0.009,
                        shade(felt(pal), 1.15),
                        Some(felt(pal)),
                    );
                }
            }
            Age::Renewal => {
                // the lower roof green; the top storey set back from the street, in boards, under its own green roof with panels
                flat_top(
                    &mut k,
                    &rg,
                    h,
                    0.0016,
                    roof_green(pal),
                    section_wall(pal, b.age, fam, 0.0, 0.0).col,
                );
                let back = WING * 0.36;
                let top_rg = Range {
                    dep: rg.dep - back,
                    ..part(
                        &rg,
                        -rg.len * 0.5 + if w.open_ends { 0.008 } else { 0.0 },
                        rg.len * 0.5 - if w.open_ends { 0.008 } else { 0.0 },
                    )
                };
                // move it back from the street (toward -c)
                let (cx, cz) = {
                    let r = top_rg;
                    let (x, z) = r.at(0.0, -back * 0.5);
                    (x, z)
                };
                let top_rg = Range { cx, cz, ..top_rg };
                let wall = Wall {
                    col: shade(*pal.modern.cladding_timber, 1.06),
                    plinth: None,
                    frame: *pal.walls.window_frame,
                    sill: *pal.stone.sill,
                };
                let mut ops = Vec::new();
                if !k.far() {
                    let bays = (top_rg.len / 0.016).floor().max(1.0) as usize;
                    for bay in 0..bays {
                        ops.push(Opening {
                            u: top_rg.len * (bay as f32 + 0.5) / bays as f32,
                            y: h + 0.002,
                            w: 0.009,
                            h: 0.013,
                            kind: Open::Casement,
                        });
                    }
                }
                let ends = if w.open_ends { Side::PLAIN } else { Side::NONE };
                k.walls(
                    &top_rg,
                    h,
                    h + STOREY * 0.92,
                    &wall,
                    [Side::free(&ops), ends, Side::free(&ops), ends],
                    false,
                    false,
                );
                flat_top(
                    &mut k,
                    &top_rg,
                    h + STOREY * 0.92,
                    0.0012,
                    roof_green(pal),
                    wall.col,
                );
                if !k.far() {
                    // solar panels on the top roof, tilted to the south
                    let panel = *pal.modern.solar_panel;
                    let pn = (top_rg.len / 0.012).floor() as usize;
                    for p in 0..pn {
                        let a = -top_rg.len * 0.5 + 0.012 * (p as f32 + 0.5);
                        let y = h + STOREY * 0.92 + 0.0014;
                        let c0 = -top_rg.dep * 0.25;
                        let pts = [
                            k.rp(&top_rg, a - 0.0045, y, c0 + 0.006),
                            k.rp(&top_rg, a + 0.0045, y, c0 + 0.006),
                            k.rp(&top_rg, a + 0.0045, y + 0.003, c0 - 0.002),
                            k.rp(&top_rg, a - 0.0045, y + 0.003, c0 - 0.002),
                        ];
                        k.quad(pts, panel, Vec3::Y);
                    }
                }
            }
        }
    }

    // the turrets on the street corners of a block of the first age
    if b.age == Age::Reform && court {
        for sx in [-1.0f32, 1.0] {
            let t = 0.026;
            let (x0, x1) = if sx > 0.0 {
                (hw - t + 0.002, hw + 0.002)
            } else {
                (-hw - 0.002, -hw + t - 0.002)
            };
            let rg = Range::rect(x0, x1, hd - t + 0.002, hd + 0.002);
            let wall = section_wall(pal, b.age, fam, 0.9, 0.9);
            let ops = if k.far() {
                vec![]
            } else {
                vec![win(t * 0.5, walls_n, 0.007, 0.009, Open::Sash)]
            };
            // (free: the turret's walls stand on the block's, not on the ground)
            let sides = [
                Side::free(&ops),
                if sx > 0.0 {
                    Side::free(&ops)
                } else {
                    Side::PLAIN
                },
                Side::PLAIN,
                if sx < 0.0 {
                    Side::free(&ops)
                } else {
                    Side::PLAIN
                },
            ];
            k.walls(&rg, h - STOREY * 0.5, h + STOREY, &wall, sides, false, true);
            k.roof(
                &rg,
                &RoofSpec {
                    h: h + STOREY,
                    rise: 0.02,
                    eaves: 0.002,
                    verge: 0.002,
                    ends: [End::Hip, End::Hip],
                    col: *pal.roofs.church_lead,
                    gable: wall.col,
                    thick: 0.0009,
                },
            );
        }
    }
    drop(k);

    // the court: a garden with paths and trees, and a pavement round the block's streets
    let mut g = Kit::new(flat, site, pal, Detail::Play, 0.0);
    let pave = *pal.ground.shop_pavement;
    let e = 0.006;
    g.patch(-hw - e, hw + e, hd, hd + e, 0.0012, pave);
    g.patch(-hw - e, hw + e, -hd - e, -hd, 0.0012, pave);
    g.patch(-hw - e, -hw, -hd, hd, 0.0012, pave);
    g.patch(hw, hw + e, -hd, hd, 0.0012, pave);
    if court {
        let (cx0, cx1, cz0, cz1) = (-hw + WING, hw - WING, -hd + WING, hd - WING);
        g.patch(cx0, cx1, cz0, cz1, 0.0012, *pal.ground.garden);
        let path = *pal.ground.garden_path;
        g.patch(cx0, cx1, -0.0025, 0.0025, 0.0016, path);
        g.patch(-0.0025, 0.0025, cz0, cz1, 0.0016, path);
        // trees: as many as the court holds, a few for a small court
        let room = (cx1 - cx0) * (cz1 - cz0);
        let trees = ((room / 0.0011).round() as usize).clamp(2, 14);
        for t in 0..trees {
            let (u, v) = (fam.pick(100 + t as i32), fam.pick(120 + t as i32));
            let x = cx0 + 0.008 + (cx1 - cx0 - 0.016) * u;
            let z = cz0 + 0.008 + (cz1 - cz0 - 0.016) * v;
            if x.abs() < 0.005 || z.abs() < 0.005 {
                continue;
            }
            let p = g.pt(x, 0.0, z);
            hedge_tree(
                green,
                &site.moved(p, 0.0),
                0.36 + 0.16 * fam.pick(140 + t as i32),
                fam.pick(160 + t as i32) * 3.0,
                u * 6.0,
                pal,
            );
        }
    }
}

/// A flat roof over a range at `h`, behind a parapet `up` high in the wall's colour.
fn flat_top(k: &mut Kit, rg: &Range, h: f32, up: f32, top: Rgba, wall: Rgba) {
    let up = super::thick(k, up);
    let (hl, hd) = (rg.len * 0.5, rg.dep * 0.5);
    let c = [
        k.rp(rg, -hl, h + up * 0.4, hd),
        k.rp(rg, hl, h + up * 0.4, hd),
        k.rp(rg, hl, h + up * 0.4, -hd),
        k.rp(rg, -hl, h + up * 0.4, -hd),
    ];
    k.quad(c, top, Vec3::Y);
    if k.far() {
        return;
    }
    // the parapet: a band round the edge
    let cap = shade(wall, 1.04);
    for (i, (a0, c0, a1, c1)) in [
        (-hl, hd, hl, hd),
        (hl, hd, hl, -hd),
        (hl, -hd, -hl, -hd),
        (-hl, -hd, -hl, hd),
    ]
    .into_iter()
    .enumerate()
    {
        let out = k.rp(rg, (a0 + a1) * 0.5, h, (c0 + c1) * 0.5) - k.rp(rg, 0.0, h, 0.0);
        let p = [
            k.rp(rg, a0, h - 0.0004, c0),
            k.rp(rg, a1, h - 0.0004, c1),
            k.rp(rg, a1, h + up, c1),
            k.rp(rg, a0, h + up, c0),
        ];
        k.quad(
            p,
            shade(wall, [1.0, 0.93, 0.86, 0.95][i]),
            Vec3::new(out.x, 0.0, out.z).normalize_or_zero(),
        );
        // its coping
        let inset = super::thick(k, 0.0012);
        let shrink = |a: f32, c: f32| {
            (
                a - a.signum() * inset * (a.abs() > 1e-6) as i32 as f32,
                c - c.signum() * inset,
            )
        };
        let (ia0, ic0) = shrink(a0, c0);
        let (ia1, ic1) = shrink(a1, c1);
        let q = [
            k.rp(rg, a0, h + up, c0),
            k.rp(rg, a1, h + up, c1),
            k.rp(rg, ia1, h + up, ic1),
            k.rp(rg, ia0, h + up, ic0),
        ];
        k.quad(q, cap, Vec3::Y);
    }
}

/// A balcony standing out from side `side` of a range at `u`, storey `s`: a slab and a rail.
fn balcony(k: &mut Kit, rg: &Range, side: usize, u: f32, s: usize, rail: Rgba, pal: &Palette) {
    let (hl, hd) = (rg.len * 0.5, rg.dep * 0.5);
    let c = if side == 0 { 1.0 } else { -1.0 };
    // `a` along the range for a point `u` along side 0 (which runs +a) or side 2 (which runs -a)
    let a = if side == 0 { -hl + u } else { hl - u };
    let y = s as f32 * STOREY;
    let (w, out) = (0.0095, 0.0042);
    let slab = shade(*pal.stone.sill, 0.95);
    let p0 = |k: &Kit, da: f32, dc: f32, dy: f32| k.rp(rg, a + da, y + dy, c * (hd + dc));
    let outward = (p0(k, 0.0, 1.0, 0.0) - p0(k, 0.0, 0.0, 0.0)).normalize_or_zero();
    // the slab's top and its edge
    let q = [
        p0(k, -w * 0.5, out, 0.0),
        p0(k, w * 0.5, out, 0.0),
        p0(k, w * 0.5, 0.0, 0.0),
        p0(k, -w * 0.5, 0.0, 0.0),
    ];
    k.quad(q, slab, Vec3::Y);
    let q = [
        p0(k, -w * 0.5, out, -super::thick(k, 0.0012)),
        p0(k, w * 0.5, out, -super::thick(k, 0.0012)),
        p0(k, w * 0.5, out, 0.0),
        p0(k, -w * 0.5, out, 0.0),
    ];
    k.quad(q, shade(slab, 0.8), outward);
    // the rail: a solid front a little under a storey's third
    let q = [
        p0(k, -w * 0.5, out, 0.0),
        p0(k, w * 0.5, out, 0.0),
        p0(k, w * 0.5, out, 0.0052),
        p0(k, -w * 0.5, out, 0.0052),
    ];
    k.quad(q, rail, outward);
    if !k.near() {
        return;
    }
    for sgn in [-1.0f32, 1.0] {
        let q = [
            p0(k, sgn * w * 0.5, 0.0, 0.0),
            p0(k, sgn * w * 0.5, out, 0.0),
            p0(k, sgn * w * 0.5, out, 0.0052),
            p0(k, sgn * w * 0.5, 0.0, 0.0052),
        ];
        let along = (p0(k, 1.0, 0.0, 0.0) - p0(k, 0.0, 0.0, 0.0)).normalize_or_zero() * sgn;
        k.quad(q, shade(rail, 0.9), along);
    }
}

/// An office tower on a podium of shops.
#[derive(Clone, Copy, Debug)]
pub struct Tower {
    /// the podium's width and depth
    pub width: f32,
    pub depth: f32,
    /// storeys of the tower over its podium of two
    pub storeys: u8,
    pub age: Age,
    pub detail: Detail,
    pub seed: Seed,
}

/// An office tower: a podium of two storeys with shops on its streets and a
/// tower over it. Of the interwar age it steps back twice and ends in a
/// copper cap; of the postwar age it is one slab with bands of windows; of
/// the last age it has terracotta fins, green terraces where it steps
/// back, and trees on its roof.
pub(crate) fn tower(
    solid: &mut MeshBuf,
    green: &mut Greenery,
    site: &Site,
    t: &Tower,
    pal: &Palette,
) {
    let (hw, hd) = (t.width * 0.5, t.depth * 0.5);
    let n = t.storeys.max(3) as usize;
    let mut k = Kit::new(solid, site, pal, t.detail, 0.0);
    let podium_h = STOREY * 2.0;
    let stone = *pal.modern.portland;
    let pod_wall = Wall {
        col: shade(stone, 0.96),
        plinth: Some(*pal.stone.plinth),
        frame: *pal.walls.window_frame,
        sill: *pal.stone.sill,
    };
    // the podium
    let pod = Range::rect(-hw, hw, -hd, hd);
    let mut front = Vec::new();
    let mut ends = Vec::new();
    if !k.far() {
        let shops = (t.width / 0.032).floor().max(1.0) as usize;
        for s in 0..shops {
            let u = t.width * (s as f32 + 0.5) / shops as f32;
            front.push(Opening {
                u,
                y: FOOT,
                w: t.width / shops as f32 * 0.8,
                h: 0.0136,
                kind: Open::Shop {
                    paint: pick(&pal.walls.door_paints, t.seed.pick(s as i32)),
                    awning: (s % 2 == 0)
                        .then(|| pick(&pal.roofs.awnings, t.seed.pick(10 + s as i32))),
                },
            });
        }
        let bays = (t.width / BAY_W).floor() as usize;
        for b in 0..bays {
            front.push(win(
                t.width * (b as f32 + 0.5) / bays as f32,
                1,
                0.0095,
                0.0095,
                Open::Casement,
            ));
        }
        let bays = (t.depth / BAY_W).floor() as usize;
        for b in 0..bays {
            ends.push(win(
                t.depth * (b as f32 + 0.5) / bays as f32,
                1,
                0.0095,
                0.0095,
                Open::Casement,
            ));
            ends.push(win(
                t.depth * (b as f32 + 0.5) / bays as f32,
                0,
                0.0095,
                0.0095,
                Open::Casement,
            ));
        }
    }
    let y0 = -k.drop(&pod);
    k.walls(
        &pod,
        y0,
        podium_h,
        &pod_wall,
        [
            Side::with(&front),
            Side::with(&ends),
            Side::with(&front),
            Side::with(&ends),
        ],
        true,
        false,
    );
    let terrace = if t.age == Age::Renewal {
        roof_green(pal)
    } else {
        felt(pal)
    };
    flat_top(&mut k, &pod, podium_h, 0.002, terrace, pod_wall.col);

    // the tower: tiers, each set back from the one below
    let (tw, td) = (t.width * 0.72, t.depth * 0.7);
    let tiers: Vec<(f32, usize)> = match t.age {
        Age::Reform | Age::Interwar => vec![
            (1.0, (n as f32 * 0.62) as usize),
            (0.82, (n as f32 * 0.86) as usize),
            (0.62, n),
        ],
        Age::Postwar => vec![(1.0, n)],
        Age::Renewal => vec![(1.0, (n as f32 * 0.7) as usize), (0.8, n)],
    };
    let wall_col = match t.age {
        Age::Reform | Age::Interwar => *pal.modern.portland,
        Age::Postwar => *pal.brick.pale,
        Age::Renewal => shade(*pal.modern.terracotta, 0.96),
    };
    let wall = Wall {
        col: wall_col,
        plinth: None,
        frame: *pal.walls.window_frame,
        sill: *pal.stone.sill,
    };
    let mut base = 0usize;
    let mut last_rg = pod;
    for (ti, &(k_size, top)) in tiers.iter().enumerate() {
        let (w, d) = (tw * k_size, td * k_size);
        let rg = Range::rect(-w * 0.5, w * 0.5, -d * 0.5, d * 0.5);
        let (yb, yt) = (
            podium_h + STOREY * base as f32,
            podium_h + STOREY * top as f32,
        );
        let mut long = Vec::new();
        let mut short = Vec::new();
        if !k.far() {
            let (ww, wh, kind, pitch) = match t.age {
                Age::Reform | Age::Interwar => (0.0062, 0.0105, Open::Sash, 0.0118),
                Age::Postwar => (0.0118, 0.0092, Open::Casement, 0.0135),
                Age::Renewal => (0.0098, 0.0128, Open::Casement, 0.0125),
            };
            for (len, ops) in [(w, &mut long), (d, &mut short)] {
                let bays = (len / pitch).floor().max(1.0) as usize;
                for s in base..top {
                    for b in 0..bays {
                        let u = len * (b as f32 + 0.5) / bays as f32;
                        ops.push(Opening {
                            u,
                            y: podium_h + STOREY * s as f32 + 0.0045,
                            w: ww,
                            h: wh,
                            kind,
                        });
                    }
                }
            }
        }
        // (openings are placed in the tower's own heights: `free` walls cut round them where they are)
        k.walls(
            &rg,
            yb - 0.001,
            yt,
            &wall,
            [
                Side::free(&long),
                Side::free(&short),
                Side::free(&long),
                Side::free(&short),
            ],
            false,
            false,
        );
        // fins of terracotta between the windows, and piers of stone on the interwar tower
        if !k.far() && (t.age == Age::Renewal || t.age <= Age::Interwar) {
            let fin = if t.age == Age::Renewal {
                *pal.modern.terracotta
            } else {
                shade(wall_col, 1.06)
            };
            for side in 0..4 {
                let len = if side % 2 == 0 { w } else { d };
                let pitch = if t.age == Age::Renewal {
                    0.0125
                } else {
                    0.0118
                };
                let bays = (len / pitch).floor().max(1.0) as usize;
                for b in 0..=bays {
                    let u = len * b as f32 / bays as f32;
                    k.strip(
                        &rg,
                        side,
                        u.clamp(0.0011, len - 0.0011) - 0.0011,
                        u.clamp(0.0011, len - 0.0011) + 0.0011,
                        yb,
                        yt,
                        0.0014,
                        fin,
                    );
                }
            }
        }
        let terrace = if t.age == Age::Renewal {
            roof_green(pal)
        } else {
            felt(pal)
        };
        flat_top(&mut k, &rg, yt, 0.0024, terrace, wall_col);
        if t.age == Age::Renewal && ti + 1 < tiers.len() {
            // trees on the terrace where it steps back
            let p = k.pt(w * 0.5 - 0.006, yt, d * 0.5 - 0.006);
            hedge_tree(
                green,
                &site.moved(p, 0.0),
                0.22,
                t.seed.pick(200 + ti as i32),
                1.0,
                pal,
            );
            let p = k.pt(-w * 0.5 + 0.006, yt, d * 0.5 - 0.006);
            hedge_tree(
                green,
                &site.moved(p, 0.0),
                0.2,
                t.seed.pick(220 + ti as i32),
                2.0,
                pal,
            );
        }
        base = top;
        last_rg = rg;
    }
    let roof_y = podium_h + STOREY * n as f32;
    match t.age {
        Age::Reform | Age::Interwar => {
            // a copper cap
            k.roof(
                &last_rg,
                &RoofSpec {
                    h: roof_y + 0.002,
                    rise: last_rg.dep * 0.6,
                    eaves: 0.0015,
                    verge: 0.0015,
                    ends: [End::Hip, End::Hip],
                    col: *pal.modern.copper,
                    gable: wall_col,
                    thick: 0.0009,
                },
            );
        }
        Age::Postwar => {
            if !k.far() {
                let (w, d) = (last_rg.len * 0.4, last_rg.dep * 0.5);
                k.boxy(
                    -w * 0.5,
                    w * 0.5,
                    -d * 0.5,
                    d * 0.5,
                    roof_y,
                    roof_y + 0.012,
                    shade(wall_col, 0.92),
                    Some(felt(pal)),
                );
            }
        }
        Age::Renewal => {
            let p = k.pt(0.0, roof_y, 0.0);
            hedge_tree(green, &site.moved(p, 0.0), 0.26, t.seed.pick(240), 0.5, pal);
            let p = k.pt(last_rg.len * 0.3, roof_y, -last_rg.dep * 0.2);
            hedge_tree(green, &site.moved(p, 0.0), 0.22, t.seed.pick(241), 2.5, pal);
        }
    }
}
