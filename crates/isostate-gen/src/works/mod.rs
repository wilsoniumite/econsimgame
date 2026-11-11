//! The works that firms build: one generator for each kind, by the method
//! the firm works by, at any size from a workshop to a great works.
//!
//! A works is a compound in a yard of its own. Its size is its capacity over
//! the standard capacity of its method (K over K0), from about 0.4 to 100,
//! and it grows by adding parts: bays, storeys, furnaces, chimneys, kilns,
//! and a yard that widens to hold them (`footprint`). The year it was
//! founded sets its style (stone, timber and water early; brick, then iron
//! and glass), and its state says whether it is going up, working, standing
//! idle or derelict. Each kind is a function in a file here that places
//! parts from `kit` in the yard; `pit` draws the heaps and headframes of
//! the mines, which are not firms.

mod brick;
mod engine;
mod furnaces;
mod kit;
mod mills;
mod pit;
mod plain;
mod steel;
mod textile;
mod trades;
mod woodland;

pub use kit::{plume, WIND};
pub use pit::{pit, Pit, PitKind};

use glam::Vec3;

use crate::hash::Seed;
use crate::kit::Detail;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// What a works is, by the method its firm works by (the names are the
/// simulation's, from data/content.toml).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    HandLoom,
    Smithy,
    Millwright,
    /// shops and trades: the simulation's "services"
    Shops,
    CharcoalBurning,
    Bloomery,
    CharcoalFurnace,
    CokeFurnace,
    TextileMill,
    EngineWorks,
    ForgeRails,
    RollingMill,
    SteelRolling,
    Bessemer,
    CrucibleSteel,
    Brickworks,
    /// a method the generators do not know: plain sheds in a yard
    Other,
}

impl Kind {
    /// Every kind, in the order the lab shows them: the town trades, iron, steel, the rest.
    pub const ALL: [Kind; 17] = [
        Kind::HandLoom,
        Kind::Smithy,
        Kind::Millwright,
        Kind::Shops,
        Kind::CharcoalBurning,
        Kind::Bloomery,
        Kind::CharcoalFurnace,
        Kind::CokeFurnace,
        Kind::ForgeRails,
        Kind::RollingMill,
        Kind::CrucibleSteel,
        Kind::Bessemer,
        Kind::SteelRolling,
        Kind::EngineWorks,
        Kind::TextileMill,
        Kind::Brickworks,
        Kind::Other,
    ];

    /// The kind of works a method builds.
    pub fn of_method(name: &str) -> Kind {
        match name {
            "services" => Kind::Shops,
            "hand_loom" => Kind::HandLoom,
            "smithy" => Kind::Smithy,
            "millwright" => Kind::Millwright,
            "charcoal_burning" => Kind::CharcoalBurning,
            "bloomery" => Kind::Bloomery,
            "charcoal_furnace" => Kind::CharcoalFurnace,
            "coke_furnace" => Kind::CokeFurnace,
            "textile_mill" => Kind::TextileMill,
            "engine_works" => Kind::EngineWorks,
            "forge_rails" => Kind::ForgeRails,
            "rolling_mill" => Kind::RollingMill,
            "steel_rolling" => Kind::SteelRolling,
            "bessemer" => Kind::Bessemer,
            "crucible_steel" => Kind::CrucibleSteel,
            "brickworks" => Kind::Brickworks,
            _ => Kind::Other,
        }
    }

    /// The method's name in the simulation.
    pub fn method(self) -> &'static str {
        match self {
            Kind::HandLoom => "hand_loom",
            Kind::Smithy => "smithy",
            Kind::Millwright => "millwright",
            Kind::Shops => "services",
            Kind::CharcoalBurning => "charcoal_burning",
            Kind::Bloomery => "bloomery",
            Kind::CharcoalFurnace => "charcoal_furnace",
            Kind::CokeFurnace => "coke_furnace",
            Kind::TextileMill => "textile_mill",
            Kind::EngineWorks => "engine_works",
            Kind::ForgeRails => "forge_rails",
            Kind::RollingMill => "rolling_mill",
            Kind::SteelRolling => "steel_rolling",
            Kind::Bessemer => "bessemer",
            Kind::CrucibleSteel => "crucible_steel",
            Kind::Brickworks => "brickworks",
            Kind::Other => "other",
        }
    }

    /// What the game calls it, in plain words.
    pub fn label(self) -> &'static str {
        match self {
            Kind::HandLoom => "handloom weaver",
            Kind::Smithy => "smithy",
            Kind::Millwright => "millwright",
            Kind::Shops => "shops and trades",
            Kind::CharcoalBurning => "charcoal burning",
            Kind::Bloomery => "bloomery",
            Kind::CharcoalFurnace => "charcoal furnace",
            Kind::CokeFurnace => "coke furnace",
            Kind::TextileMill => "powered mill",
            Kind::EngineWorks => "engine works",
            Kind::ForgeRails => "forged rails",
            Kind::RollingMill => "rolling mill",
            Kind::SteelRolling => "steel rail mill",
            Kind::Bessemer => "converter",
            Kind::CrucibleSteel => "crucible steel",
            Kind::Brickworks => "brickworks",
            Kind::Other => "any other works",
        }
    }

    /// The ground a works of this kind and of the standard size takes: its
    /// width along the front and its depth, in map units.
    fn plot(self) -> (f32, f32) {
        match self {
            Kind::HandLoom => (0.075, 0.06),
            Kind::Smithy => (0.08, 0.065),
            Kind::Millwright => (0.10, 0.08),
            Kind::Shops => (0.07, 0.055),
            Kind::CharcoalBurning => (0.11, 0.09),
            Kind::Bloomery => (0.085, 0.07),
            Kind::CharcoalFurnace => (0.13, 0.10),
            Kind::CokeFurnace => (0.15, 0.11),
            Kind::TextileMill => (0.13, 0.10),
            Kind::EngineWorks => (0.13, 0.10),
            Kind::ForgeRails => (0.13, 0.09),
            Kind::RollingMill => (0.14, 0.10),
            Kind::SteelRolling => (0.165, 0.10),
            Kind::Bessemer => (0.14, 0.10),
            Kind::CrucibleSteel => (0.10, 0.08),
            Kind::Brickworks => (0.13, 0.10),
            Kind::Other => (0.10, 0.075),
        }
    }
}

/// What a works is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// newly founded and not yet at work: walls part of the way up under
    /// scaffolding, no roofs, no smoke
    GoingUp,
    /// at work, and being enlarged this year: scaffolding round its newest part
    Extending,
    /// at work
    Working,
    /// standing whole with its fires out and its gate shut
    Idle,
    /// closed for good: roofs falling in, chimneys broken, weeds in the yard
    Derelict,
}

impl State {
    pub const ALL: [State; 5] = [State::GoingUp, State::Extending, State::Working, State::Idle, State::Derelict];

    pub fn label(self) -> &'static str {
        match self {
            State::GoingUp => "going up",
            State::Extending => "being enlarged",
            State::Working => "working",
            State::Idle => "standing idle",
            State::Derelict => "derelict",
        }
    }
}

/// A works to draw.
#[derive(Clone, Copy, Debug)]
pub struct Works {
    pub kind: Kind,
    /// its size: its capacity over the standard capacity of its method (K
    /// over K0), from a workshop of 0.4 to a great works of 100. The game
    /// hands it a size from a ladder (`rung`, `rung_size`) so that a works
    /// changes in steps that can be seen.
    pub size: f32,
    /// the year it was founded, which sets its style
    pub founded: i32,
    pub state: State,
    /// a river runs by the site (an early works then takes its power from a wheel)
    pub water: bool,
    /// a railway runs by the site
    pub rail: bool,
    /// how much of it to draw (`Detail`): the viewer chooses by where the
    /// camera is, as it does for a house. Far: walls, roofs and the tall
    /// chimneys. Play: openings as flat panels, stacks, the yard's wall and
    /// what stands in it. Near: everything. The yard's ground and water
    /// (what goes into `flat`) and where the flag flies are the same at all three.
    pub detail: Detail,
    /// the works' own two numbers, which settle every choice it makes for itself
    pub seed: Seed,
}

impl Works {
    /// A working works of the middle of the period, away from water and rail, drawn in full.
    pub fn new(kind: Kind, size: f32) -> Works {
        Works { kind, size, founded: 1820, state: State::Working, water: false, rail: false, detail: Detail::Near, seed: Seed::of(0) }
    }
}

/// The sizes a works is drawn at lie on a ladder, two rungs to a doubling,
/// so that it grows in steps that can be seen (and goes into scaffolding
/// for each). `phase`, from 0 to 1, is the works' own, and sets where its
/// rungs fall, so that the works of a town do not all step in the same
/// year. This is the rung a works of `ratio` (K over K0) stands on.
pub fn rung(ratio: f32, phase: f32) -> i32 {
    (ratio.max(0.4).log2() * 2.0 + phase).floor() as i32
}

/// The size a works on a rung is drawn at: the middle of the rung.
pub fn rung_size(rung: i32, phase: f32) -> f32 {
    2f32.powf((rung as f32 - phase + 0.5) * 0.5)
}

/// The ground a works takes: how wide its yard is along its front and how
/// deep, in map units. It grows with the size a little faster than the
/// buildings do, so that a great works is many buildings and not one large one.
pub fn footprint(kind: Kind, size: f32) -> (f32, f32) {
    let (w, d) = kind.plot();
    let k = size.clamp(0.4, 120.0).powf(0.32);
    (w * k, d * k)
}

/// How much bigger than at the standard size a works' buildings are drawn.
fn grain(size: f32) -> f32 {
    size.clamp(0.4, 120.0).powf(0.10)
}

/// Draw a works in its yard: the site's origin is the middle of the yard,
/// the gate is on its +z side. Buildings go to `solid`, smoke to `smoke`,
/// and the yard's ground (and any water in it) to `flat`.
pub fn works(solid: &mut MeshBuf, smoke: &mut MeshBuf, flat: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) {
    build(solid, smoke, flat, site, w, pal);
}

/// Where a flag on the works would fly from: the top of its highest gable
/// (or tower), on the map. It is the same point at every level of detail,
/// so it is found on the works as it is drawn from far off, which is the
/// least to build.
pub fn flag_point(site: &Site, w: &Works, pal: &Palette) -> Vec3 {
    let (mut solid, mut smoke, mut flat) = (MeshBuf::default(), MeshBuf::default(), MeshBuf::default());
    build(&mut solid, &mut smoke, &mut flat, site, &Works { detail: Detail::Far, ..*w }, pal).0.unwrap_or(site.origin + Vec3::Y * 0.03)
}

/// Build a works. It gives back the top of its highest gable and how many
/// of its own numbers it took (the same at every level of detail).
fn build(solid: &mut MeshBuf, smoke: &mut MeshBuf, flat: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) -> (Option<Vec3>, i32) {
    let mut c = kit::Ctx::new(solid, smoke, flat, site, w, pal);
    match w.kind {
        Kind::HandLoom => trades::hand_loom(&mut c),
        Kind::Smithy => trades::smithy(&mut c),
        Kind::Millwright => trades::millwright(&mut c),
        Kind::Shops => trades::shops(&mut c),
        Kind::CharcoalBurning => woodland::charcoal_burning(&mut c),
        Kind::Bloomery => woodland::bloomery(&mut c),
        Kind::CharcoalFurnace => furnaces::charcoal_furnace(&mut c),
        Kind::CokeFurnace => furnaces::coke_furnace(&mut c),
        Kind::ForgeRails => mills::forge(&mut c),
        Kind::RollingMill => mills::rolling_mill(&mut c),
        Kind::SteelRolling => mills::steel_rail_mill(&mut c),
        Kind::CrucibleSteel => steel::crucible(&mut c),
        Kind::Bessemer => steel::converter(&mut c),
        Kind::EngineWorks => engine::engine_works(&mut c),
        Kind::TextileMill => textile::mill(&mut c),
        Kind::Brickworks => brick::brickworks(&mut c),
        Kind::Other => plain::plain(&mut c),
    }
    match w.state {
        State::GoingUp => {
            let n = c.sheds.len().min(3);
            for i in 0..n {
                let s = c.sheds[i];
                c.scaffold(&s);
            }
            c.builders_yard();
        }
        State::Extending => {
            if let Some(&s) = c.sheds.last() {
                c.scaffold(&s);
            }
            c.builders_yard();
        }
        _ => {}
    }
    c.finish();
    (c.peak, c.taken())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::Model;

    /// The sizes the lab shows, and one beyond each end.
    const SIZES: [f32; 8] = [0.1, 0.4, 1.0, 3.0, 10.0, 30.0, 100.0, 400.0];

    fn build_at(w: &Works, yaw: f32) -> Model {
        let mut m = Model::default();
        works(&mut m.solid, &mut m.smoke, &mut m.flat, &Site::level(Vec3::ZERO, yaw), w, &Palette::standard());
        m
    }

    fn all(mut each: impl FnMut(Works)) {
        for kind in Kind::ALL {
            for size in SIZES {
                for founded in [1760, 1820, 1870] {
                    for water in [false, true] {
                        each(Works { founded, water, seed: Seed::of(kind as u32 * 7 + founded as u32), ..Works::new(kind, size) });
                    }
                }
            }
        }
    }

    #[test]
    fn every_method_has_a_kind_and_back() {
        for k in Kind::ALL {
            if k != Kind::Other {
                assert_eq!(Kind::of_method(k.method()), k);
            }
        }
        assert_eq!(Kind::of_method("something new"), Kind::Other);
    }

    #[test]
    fn the_rungs_are_two_to_a_doubling_and_each_works_has_its_own() {
        assert_eq!(rung(1.0, 0.0), 0);
        assert_eq!(rung(1.4, 0.0), 0);
        assert_eq!(rung(1.42, 0.0), 1);
        assert_eq!(rung(2.0, 0.0), 2);
        assert_eq!(rung(100.0, 0.0), 13);
        // the smallest works drawn is two fifths of the standard size
        assert_eq!(rung(0.0, 0.0), rung(0.4, 0.0));
        // a works of another phase steps at another size
        assert_eq!(rung(1.3, 0.5), 1);
        for (r, phase) in [(0.4, 0.0), (1.0, 0.3), (7.0, 0.9), (60.0, 0.5)] {
            let size = rung_size(rung(r, phase), phase);
            assert!(size / r < 1.2 && r / size < 1.2, "{r} is drawn at {size}");
            assert_eq!(rung(size, phase), rung(r, phase));
        }
    }

    #[test]
    fn every_kind_builds_at_every_size_in_every_state() {
        all(|w| {
            for state in State::ALL {
                let w = Works { state, ..w };
                let m = build_at(&w, 0.0);
                assert!(!m.solid.is_empty() && m.is_sound(), "{w:?}");
                assert_eq!(m, build_at(&w, 0.0), "{w:?} is not the same twice");
            }
        });
    }

    #[test]
    fn the_footprint_holds_the_works() {
        all(|w| {
            for state in State::ALL {
                let w = Works { state, ..w };
                let m = build_at(&w, 0.0);
                let (fw, fd) = footprint(w.kind, w.size);
                for b in [&m.solid, &m.flat] {
                    let Some((lo, hi)) = b.bounds() else { continue };
                    // eaves may overhang by a hair
                    let e = 0.004;
                    assert!(lo.x >= -fw * 0.5 - e && hi.x <= fw * 0.5 + e && lo.z >= -fd * 0.5 - e && hi.z <= fd * 0.5 + e, "{w:?}: {lo} to {hi} in a yard {fw} by {fd}");
                }
                let top = m.solid.bounds().unwrap().1.y;
                assert!(top < 0.45, "{w:?} is {top} tall");
            }
        });
    }

    #[test]
    fn the_footprint_grows_with_the_size_and_stays_within_a_cell() {
        for kind in Kind::ALL {
            let mut last = 0.0;
            for size in SIZES {
                let (w, d) = footprint(kind, size);
                assert!(w * d >= last, "{kind:?} at {size}");
                assert!(w < 0.95 && d < 0.7, "{kind:?} at {size} takes {w} by {d}");
                last = w * d;
            }
            // a works of a hundred times the size takes about twenty times the ground
            let (a, b) = (footprint(kind, 1.0), footprint(kind, 100.0));
            assert!((b.0 * b.1) / (a.0 * a.1) > 15.0);
        }
    }

    #[test]
    fn a_works_grows_with_its_size() {
        // more of it, and more ground under it, at each of the lab's sizes
        for kind in Kind::ALL {
            for founded in [1760, 1820, 1870] {
                let mut last = (0, 0.0);
                for size in [0.4, 1.0, 3.0, 10.0, 30.0, 100.0] {
                    let m = build_at(&Works { founded, ..Works::new(kind, size) }, 0.0);
                    let (lo, hi) = m.solid.bounds().unwrap();
                    let spread = (hi.x - lo.x) * (hi.z - lo.z);
                    // (a stack or two of goods may be away, so not to the last triangle)
                    assert!(m.solid.triangles() * 10 >= last.0 * 9, "{kind:?} founded {founded} has less to it at {size}: {} triangles after {}", m.solid.triangles(), last.0);
                    assert!(spread > last.1, "{kind:?} founded {founded} does not spread at {size}");
                    last = (m.solid.triangles(), spread);
                }
            }
        }
    }

    #[test]
    fn only_a_working_works_smokes() {
        all(|w| {
            for state in [State::GoingUp, State::Idle, State::Derelict] {
                assert!(build_at(&Works { state, ..w }, 0.0).smoke.is_empty(), "{w:?} smokes while {state:?}");
            }
        });
        // and every kind that has a fire smokes when it works
        for kind in Kind::ALL {
            let smokes = !build_at(&Works::new(kind, 3.0), 0.0).smoke.is_empty();
            assert_eq!(smokes, !matches!(kind, Kind::Other), "{kind:?}");
        }
    }

    #[test]
    fn a_bigger_works_makes_more_smoke() {
        for kind in [Kind::CokeFurnace, Kind::TextileMill, Kind::RollingMill, Kind::Bessemer, Kind::CrucibleSteel, Kind::Brickworks] {
            // more plumes, or a bigger one
            let smoke = |size: f32| {
                let m = build_at(&Works::new(kind, size), 0.0);
                let (lo, hi) = m.smoke.bounds().unwrap();
                m.smoke.triangles() as f32 * (hi - lo).length()
            };
            assert!(smoke(30.0) > smoke(1.0), "{kind:?}");
        }
    }

    #[test]
    fn the_state_shows() {
        // each state is a different picture of the same works
        for kind in Kind::ALL {
            let w = Works::new(kind, 3.0);
            let built: Vec<Model> = State::ALL.iter().map(|&state| build_at(&Works { state, ..w }, 0.0)).collect();
            for i in 0..built.len() {
                for j in 0..i {
                    assert!(built[i] != built[j], "{kind:?}: {:?} looks like {:?}", State::ALL[i], State::ALL[j]);
                }
            }
        }
    }

    #[test]
    fn the_flag_flies_from_the_top_of_the_works() {
        all(|w| {
            let site = Site::level(Vec3::ZERO, 0.3);
            let p = flag_point(&site, &w, &Palette::standard());
            let (fw, fd) = footprint(w.kind, w.size);
            assert!(p.y > 0.008 && p.y < 0.45, "{w:?}: the flag is at {p}");
            assert!(p.x.hypot(p.z) <= fw.hypot(fd) * 0.5 + 0.01, "{w:?}: the flag at {p} is off the yard");
        });
    }

    #[test]
    fn it_is_the_same_works_at_every_level_of_detail() {
        // the camera's distance changes how much is drawn, never what stands where: the
        // yard's ground and water, the flag's place, the plumes and the works' own numbers
        // are the same at all three levels, and each level draws more than the one before
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.3);
        all(|w| {
            for state in [State::Working, State::GoingUp, State::Derelict] {
                let built: Vec<(Model, Option<Vec3>, i32)> = Detail::ALL
                    .iter()
                    .map(|&detail| {
                        let mut m = Model::default();
                        let (peak, taken) = build(&mut m.solid, &mut m.smoke, &mut m.flat, &site, &Works { state, detail, ..w }, &pal);
                        (m, peak, taken)
                    })
                    .collect();
                let (far, play, near) = (&built[0], &built[1], &built[2]);
                for other in [play, near] {
                    assert!(far.0.flat == other.0.flat, "{w:?} {state:?}: the yard's ground differs by level");
                    assert!(far.0.smoke == other.0.smoke, "{w:?} {state:?}: the smoke differs by level");
                    assert_eq!(far.1, other.1, "{w:?} {state:?}: the flag's place differs by level");
                    assert_eq!(far.2, other.2, "{w:?} {state:?}: the works took other numbers of its own at another level");
                }
                assert!(far.0.solid.triangles() <= play.0.solid.triangles() && play.0.solid.triangles() <= near.0.solid.triangles(), "{w:?} {state:?}");
            }
        });
        // from far off a great mill is a few hundred triangles, close to some thousands
        let tris = |detail| build_at(&Works { detail, ..Works::new(Kind::TextileMill, 30.0) }, 0.0).solid.triangles();
        assert!(tris(Detail::Far) * 5 < tris(Detail::Play) && tris(Detail::Play) * 3 < tris(Detail::Near), "{} {} {}", tris(Detail::Far), tris(Detail::Play), tris(Detail::Near));
    }

    #[test]
    fn a_mill_that_a_wheel_turns_has_its_pond() {
        let water = *Palette::standard().water.river;
        let wet = |w: &Works| build_at(w, 0.0).flat.col.iter().filter(|c| **c == water).count();
        let mill = Works { founded: 1760, water: true, ..Works::new(Kind::TextileMill, 10.0) };
        // the smallest has its race and no room for a pond
        assert!(wet(&mill) > wet(&Works { size: 0.4, ..mill }) && wet(&Works { size: 0.4, ..mill }) > 0);
        // and there is no water in the yard of a mill away from the river, or of one an engine turns
        assert_eq!(wet(&Works { water: false, ..mill }), 0);
        assert_eq!(wet(&Works { founded: 1870, ..mill }), 0);
    }

    #[test]
    fn the_site_turns_the_yard() {
        let w = Works::new(Kind::CokeFurnace, 10.0);
        let (mut turned, built) = (build_at(&w, 0.0), build_at(&w, 0.7));
        turned.turn(0.7);
        let near = |a: &[[f32; 3]], b: &[[f32; 3]]| a.len() == b.len() && a.iter().zip(b).all(|(p, q)| (Vec3::from_array(*p) - Vec3::from_array(*q)).length() < 1e-4);
        assert!(near(&turned.flat.pos, &built.flat.pos));
    }
}
