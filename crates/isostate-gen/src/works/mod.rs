//! The works that firms build: one generator for each kind, by the method
//! the firm works by, at one of four sizes.
//!
//! A works is drawn at a tier, 1 for a small one up to 4, which steps up each
//! time its capacity doubles (`tier`). Each kind is a file of its own here.
//! Round them stand the scaffolding of a works going up or growing
//! (`scaffold`), the sheds of a works yard (`yard_shed`) and the heaps and
//! headframes of the pits (`pit`).

mod brickworks;
mod charcoal;
mod crucible;
mod furnace;
mod heavy;
mod loom;
mod pit;
mod plain;
mod scaffold;
mod textile_mill;
mod workshop;
mod yard;

pub use pit::{pit, Pit, PitKind};
pub use scaffold::scaffold;
pub use yard::{yard_shed, YardShed};

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
    /// a method the generators do not know: a plain brick shed
    Other,
}

impl Kind {
    /// Every kind, in the order the lab shows them: cottage trades, iron, steel, the rest.
    pub const ALL: [Kind; 16] = [
        Kind::HandLoom,
        Kind::Smithy,
        Kind::Millwright,
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

    /// The kind of works a method builds; nothing for services, which have no works.
    pub fn of_method(name: &str) -> Option<Kind> {
        Some(match name {
            "services" => return None,
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
        })
    }

    /// The method's name in the simulation.
    pub fn method(self) -> &'static str {
        match self {
            Kind::HandLoom => "hand_loom",
            Kind::Smithy => "smithy",
            Kind::Millwright => "millwright",
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
}

/// A works to draw.
#[derive(Clone, Copy, Debug)]
pub struct Works {
    pub kind: Kind,
    /// its size class, 1 to 4
    pub tier: u8,
    /// a number of the works' own, which sets how its smoke drifts
    pub seed: f32,
}

impl Works {
    /// How much bigger than a tier-1 works the main building is drawn.
    pub fn scale(&self) -> f32 {
        scale(self.tier)
    }
}

/// The size class a works is drawn at: 1 for a small one, up to 4. `ratio` is
/// its capacity over the standard capacity of its method, and the class steps
/// up at each doubling.
pub fn tier(ratio: f32) -> u8 {
    (ratio.max(0.05).log2() + 1.5).floor().clamp(1.0, 4.0) as u8
}

/// How much bigger than a tier-1 works a works of this tier is drawn.
pub fn scale(tier: u8) -> f32 {
    0.8 + 0.2 * tier as f32
}

/// Draw a works on its plot: the site's origin is the middle of the plot.
pub fn works(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) {
    match w.kind {
        Kind::HandLoom => loom::hand_loom(solid, site, pal),
        Kind::Smithy | Kind::Millwright => workshop::workshop(solid, smoke, site, w, pal),
        Kind::CharcoalBurning => charcoal::charcoal_burning(solid, site, w, pal),
        Kind::Bloomery | Kind::CharcoalFurnace | Kind::CokeFurnace => furnace::furnace(solid, smoke, site, w, pal),
        Kind::TextileMill => textile_mill::textile_mill(solid, smoke, site, w, pal),
        Kind::EngineWorks | Kind::ForgeRails | Kind::RollingMill | Kind::SteelRolling | Kind::Bessemer => heavy::heavy(solid, smoke, site, w, pal),
        Kind::CrucibleSteel => crucible::crucible_steel(solid, site, pal),
        Kind::Brickworks => brickworks::brickworks(solid, site, w, pal),
        Kind::Other => plain::plain(solid, site, w, pal),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::Model;
    use glam::Vec3;

    fn build(kind: Kind, tier: u8) -> Model {
        let mut m = Model::default();
        works(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), &Works { kind, tier, seed: 1.0 }, &Palette::standard());
        m
    }

    #[test]
    fn the_tier_steps_up_at_each_doubling() {
        assert_eq!(tier(0.0), 1);
        assert_eq!(tier(1.0), 1);
        assert_eq!(tier(1.41), 1);
        assert_eq!(tier(1.42), 2);
        assert_eq!(tier(2.82), 2);
        assert_eq!(tier(2.84), 3);
        assert_eq!(tier(5.67), 4);
        assert_eq!(tier(1000.0), 4);
        assert_eq!(scale(1), 1.0);
    }

    #[test]
    fn every_method_has_a_kind_and_back() {
        for k in Kind::ALL {
            if k != Kind::Other {
                assert_eq!(Kind::of_method(k.method()), Some(k));
            }
        }
        assert_eq!(Kind::of_method("services"), None);
        assert_eq!(Kind::of_method("something new"), Some(Kind::Other));
    }

    #[test]
    fn every_kind_builds_at_every_tier_and_stays_on_its_plot() {
        for kind in Kind::ALL {
            for tier in 1..=4 {
                let m = build(kind, tier);
                assert!(!m.solid.is_empty() && m.is_sound(), "{kind:?} at tier {tier}");
                let (lo, hi) = m.solid.bounds().unwrap();
                // plots are a third of a cell apart (0.33); the biggest works reach a quarter of a cell across
                assert!(hi.x - lo.x < 0.26 && hi.z - lo.z < 0.26, "{kind:?} at tier {tier} is {} by {}", hi.x - lo.x, hi.z - lo.z);
                assert!(hi.y < 0.3, "{kind:?} at tier {tier} is {} tall", hi.y);
            }
        }
    }

    #[test]
    fn the_kinds_that_burn_coal_smoke() {
        for kind in [Kind::Smithy, Kind::Millwright, Kind::CokeFurnace, Kind::TextileMill, Kind::EngineWorks, Kind::ForgeRails, Kind::RollingMill, Kind::SteelRolling, Kind::Bessemer] {
            assert!(!build(kind, 2).smoke.is_empty(), "{kind:?}");
        }
        for kind in [Kind::HandLoom, Kind::CharcoalBurning, Kind::Bloomery, Kind::CharcoalFurnace, Kind::CrucibleSteel, Kind::Brickworks, Kind::Other] {
            assert!(build(kind, 2).smoke.is_empty(), "{kind:?}");
        }
    }

    #[test]
    fn most_kinds_grow_with_their_tier() {
        // a handloom weaver's cottage and a crucible shop are the same at any size
        for kind in Kind::ALL {
            let (small, big) = (build(kind, 1), build(kind, 4));
            let size = |m: &Model| {
                let (lo, hi) = m.solid.bounds().unwrap();
                (hi - lo).length()
            };
            if matches!(kind, Kind::HandLoom | Kind::CrucibleSteel) {
                assert_eq!(small, big, "{kind:?}");
            } else {
                assert!(size(&big) > size(&small), "{kind:?} does not grow");
            }
        }
    }
}
