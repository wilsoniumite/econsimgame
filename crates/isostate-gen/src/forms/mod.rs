//! The building forms of a town, from the houses of 1750 to the towers of
//! 2050, as one catalogue the simulation can ask of.
//!
//! On 20 November 2026 the designer said of the look after 1900: "it should
//! reflect technology, year, and land value". So there is no one style to
//! choose. Every form the three designers drew on 19 November (an English
//! town grown on, a dense town of courtyard blocks, a tall town in a carpet
//! of houses) is here beside the houses as they are drawn, and each says, as
//! data (`Spec`), what it is for: the years in which it is built, the
//! storeys it stands, what its builders must know (`Know`), the lot it takes
//! (`Lot`) and whether it holds dwellings, offices or sheds (`Use`).
//!
//! One function answers the question a town will ask of a lot (`choose`):
//! given what the town knows, the year, the storeys its rent asks for and the
//! kind of lot, which form. It always answers (the nearest form when none
//! fits exactly) and the same question always gets the same answer. One
//! function draws a form (`form`), from the same kit as the houses and to
//! their scale. Nothing in the game calls either yet: the piece after this
//! one makes each lot of a town follow them.
//!
//! The houses as drawn are not changed: `form` hands a cottage, a townhouse,
//! a terrace or a villa to its own generator with the defaults that
//! generator's `new` gives, which is the house a sheet of today draws.

pub(crate) mod blocks;
pub(crate) mod detached;
pub(crate) mod flats;
pub(crate) mod mill;
pub(crate) mod offices;
pub(crate) mod semis;
pub(crate) mod sheds;
pub(crate) mod sheets;
pub(crate) mod towers;

use glam::Vec3;

use crate::colour::{shade, Rgba};
use crate::hash::Seed;
use crate::houses::roof::Roof;
use crate::houses::{self, Era, PLOT};
use crate::kit::{Detail, Kit, Open, Opening, Range, FOOT, STOREY};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::BAY;
use crate::site::{Model, Site};

// ------------------------------------------------------------------ what a form is for

/// What a building holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Use {
    /// houses and flats
    Dwellings,
    /// banks, offices and towers of offices
    Offices,
    /// what is built for goods and cars rather than people: shops in a shed, a
    /// warehouse, a car park
    Sheds,
}

impl Use {
    pub const ALL: [Use; 3] = [Use::Dwellings, Use::Offices, Use::Sheds];

    pub fn label(self) -> &'static str {
        match self {
            Use::Dwellings => "dwellings",
            Use::Offices => "offices",
            Use::Sheds => "sheds",
        }
    }
}

/// The ground a form takes. A town's ground is laid out in house lots
/// (`houses::PLOT`, 0.05 by 0.036, a house and its yard); a form of more than
/// a household takes several thrown together, or a whole block between streets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lot {
    /// one house lot
    House,
    /// two to four lots thrown together along a street
    Joined,
    /// a whole block between streets, or as much ground
    Block,
    /// the ground of a works that has closed, its building kept and turned
    /// to another use (the only form on it so far is the mill turned to flats)
    Works,
}

impl Lot {
    pub const ALL: [Lot; 4] = [Lot::House, Lot::Joined, Lot::Block, Lot::Works];

    pub fn label(self) -> &'static str {
        match self {
            Lot::House => "a house lot",
            Lot::Joined => "lots thrown together",
            Lot::Block => "a whole block",
            Lot::Works => "a works' ground",
        }
    }

    /// How far apart two kinds of lot are, for finding the nearest form: a
    /// step from a house lot to lots thrown together to a block, and a works'
    /// ground as far from every other as a block is from a house lot.
    fn apart(self, other: Lot) -> i32 {
        let at = |l: Lot| -> i32 {
            match l {
                Lot::House => 0,
                Lot::Joined => 1,
                Lot::Block | Lot::Works => 2,
            }
        };
        if self == other {
            0
        } else if self == Lot::Works || other == Lot::Works {
            2
        } else {
            (at(self) - at(other)).abs()
        }
    }
}

/// What a form's builders must know. Some of it is in the content now (the
/// game's methods, `data/game.toml`), and a town knows it when it knows the
/// method; the rest the content does not have yet, and a year stands in for
/// it (`Know::year`) until it does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Know {
    /// to lay brick and frame a floor and a roof in timber: every house of 1750
    BrickAndTimber,
    /// electric light and power in the building (the content's `coal_power_station`)
    Electricity,
    /// a frame of rolled steel that carries the walls, so that a building can
    /// rise past the five or six storeys a wall of brick will bear (the
    /// content's `steel_frame`, which the game finds about 1906)
    SteelFrame,
    /// the electric lift, without which nobody climbs past six floors (not in
    /// the content: the Otis lifts of the 1890s stand in, from 1890)
    Lift,
    /// a frame and floors of reinforced concrete, and panels cast in a works
    /// (not in the content: the content makes cement from 1850, and
    /// reinforced frames were built from about 1905, which stands in)
    Concrete,
    /// a wall hung on the frame and glazed from floor to floor: plate and then
    /// float glass in a frame of metal (not in the content: 1952 stands in,
    /// the first such walls in Britain, a few years before float glass)
    CurtainWall,
    /// the motor car and lorry in common use: garages, car parks, sheds
    /// served by road (not in the content: 1925 stands in, when cars were
    /// first built by the hundred thousand in Britain)
    MotorCar,
}

impl Know {
    pub const ALL: [Know; 7] = [Know::BrickAndTimber, Know::Electricity, Know::SteelFrame, Know::Lift, Know::Concrete, Know::CurtainWall, Know::MotorCar];

    pub fn label(self) -> &'static str {
        match self {
            Know::BrickAndTimber => "brick and timber",
            Know::Electricity => "electricity",
            Know::SteelFrame => "the steel frame",
            Know::Lift => "the lift",
            Know::Concrete => "reinforced concrete",
            Know::CurtainWall => "the curtain wall",
            Know::MotorCar => "the motor car",
        }
    }

    /// The content's method a town must know for this, where the content has one.
    pub fn method(self) -> Option<&'static str> {
        match self {
            Know::BrickAndTimber => None,
            Know::Electricity => Some("coal_power_station"),
            Know::SteelFrame => Some("steel_frame"),
            Know::Lift | Know::Concrete | Know::CurtainWall | Know::MotorCar => None,
        }
    }

    /// The year that stands for it: for what the content has, the year its
    /// method is aimed at (`target` in data/game.toml); for the rest, the
    /// year that stands in until the content has it.
    pub fn year(self) -> i32 {
        match self {
            Know::BrickAndTimber => 1600,
            Know::Electricity => 1888,
            Know::SteelFrame => 1906,
            Know::Lift => 1890,
            Know::Concrete => 1905,
            Know::CurtainWall => 1952,
            Know::MotorCar => 1925,
        }
    }

    fn bit(self) -> u16 {
        1 << self as u16
    }
}

/// What a town knows, of what builders may need.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Knows(u16);

impl Knows {
    pub const NOTHING: Knows = Knows(0);

    /// Everything, as though by the year that stands for each (`Know::year`):
    /// what a town knows when the content's methods are not asked.
    pub fn by_year(year: i32) -> Knows {
        Know::ALL.iter().filter(|k| k.year() <= year).fold(Knows::NOTHING, |s, k| s.with(*k))
    }

    /// What a town knows in `year`, asking `knows_method` of what the content
    /// has (the town's own record of the methods it knows) and going by the
    /// stand-in years for the rest.
    pub fn of(year: i32, knows_method: impl Fn(&str) -> bool) -> Knows {
        Know::ALL.iter().filter(|k| k.method().map_or(k.year() <= year, &knows_method)).fold(Knows::NOTHING, |s, k| s.with(*k))
    }

    pub fn with(self, k: Know) -> Knows {
        Knows(self.0 | k.bit())
    }

    pub fn without(self, k: Know) -> Knows {
        Knows(self.0 & !k.bit())
    }

    pub fn has(self, k: Know) -> bool {
        self.0 & k.bit() != 0
    }

    pub fn all(self, list: &[Know]) -> bool {
        list.iter().all(|k| self.has(*k))
    }
}

/// Which of the three designers' directions of 19 November a form came from,
/// or whether it is one of the houses drawn before them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// the houses as they were drawn before 1900: the reference
    Today,
    /// "grown": the English mill town grown on (branch night/look-2050-grown)
    Grown,
    /// "dense": the town of courtyard blocks (branch night/look-2050-dense)
    Dense,
    /// "tall": towers in a carpet of detached houses (branch night/look-2050-tall)
    Tall,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::Today => "today",
            Source::Grown => "grown",
            Source::Dense => "dense",
            Source::Tall => "tall",
        }
    }
}

/// Every form there is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Shape {
    Cottage,
    Townhouse,
    Terrace,
    Villa,
    SemisInterwar,
    CouncilPair,
    EstatePair,
    DetachedInterwar,
    Bungalow,
    DetachedLate,
    LowFlats,
    FlatsOverShops1905,
    FlatsOverShops1930,
    FlatsOverShops1960,
    FlatsOverShops2000,
    Court1905,
    Court1930,
    Court1960,
    Court2000,
    Slab,
    PointBlock,
    FlatsTower,
    MillFlats,
    Chambers,
    OfficeBlock,
    SteppedTower,
    PodiumOffices,
    StoneTower,
    GlassTower,
    TerracottaOffices,
    RetailShed,
    DistributionShed,
    CarPark,
}

/// What a form is for: one line of the catalogue.
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub shape: Shape,
    pub name: &'static str,
    /// a name of a few words, for a label
    pub short: &'static str,
    pub use_: Use,
    pub lot: Lot,
    /// the first and last years in which it is built
    pub years: (i32, i32),
    /// the fewest and the most storeys it stands (a hall of a storey and a
    /// half counts as two; a car park's decks count as storeys)
    pub storeys: (u8, u8),
    /// what its builders must know
    pub needs: &'static [Know],
    /// the ground it is drawn on when nothing else is said, along the street and back from it
    pub ground: (f32, f32),
    pub source: Source,
}

use Know::*;

/// The catalogue: every form, in the order the sheets show them. Where two
/// forms answer the same question (the same use, lot, year and storeys), a
/// lot's own number chooses between them (`Ask::pick`).
pub static FORMS: &[Spec] = &[
    // the houses as drawn, the reference (houses/): the years are those their own rules build them in
    Spec { shape: Shape::Cottage, name: "cottage", short: "cottage", use_: Use::Dwellings, lot: Lot::House, years: (1600, 1918), storeys: (1, 2), needs: &[BrickAndTimber], ground: (PLOT.0, PLOT.1), source: Source::Today },
    Spec { shape: Shape::Townhouse, name: "townhouse", short: "townhouse", use_: Use::Dwellings, lot: Lot::House, years: (1600, 1918), storeys: (3, 4), needs: &[BrickAndTimber], ground: (PLOT.0, PLOT.1), source: Source::Today },
    Spec { shape: Shape::Terrace, name: "terrace of the railway years", short: "terrace", use_: Use::Dwellings, lot: Lot::Joined, years: (1800, 1918), storeys: (2, 3), needs: &[BrickAndTimber], ground: (0.14, PLOT.1), source: Source::Today },
    Spec { shape: Shape::Villa, name: "villa", short: "villa", use_: Use::Dwellings, lot: Lot::Joined, years: (1800, 1918), storeys: (2, 2), needs: &[BrickAndTimber], ground: (PLOT.0, PLOT.1), source: Source::Today },
    // the suburbs
    Spec { shape: Shape::SemisInterwar, name: "semis of between the wars", short: "semis 1919", use_: Use::Dwellings, lot: Lot::Joined, years: (1919, 1944), storeys: (2, 2), needs: &[BrickAndTimber], ground: semis::PAIR_PLOT, source: Source::Grown },
    Spec { shape: Shape::CouncilPair, name: "the council's pair", short: "council pair", use_: Use::Dwellings, lot: Lot::Joined, years: (1945, 1974), storeys: (2, 2), needs: &[BrickAndTimber], ground: semis::PAIR_PLOT, source: Source::Grown },
    Spec { shape: Shape::EstatePair, name: "a builder's estate pair", short: "estate pair", use_: Use::Dwellings, lot: Lot::Joined, years: (1975, 2050), storeys: (2, 2), needs: &[BrickAndTimber, MotorCar], ground: semis::PAIR_PLOT, source: Source::Grown },
    Spec { shape: Shape::DetachedInterwar, name: "detached house of between the wars", short: "detached 1919", use_: Use::Dwellings, lot: Lot::House, years: (1919, 1945), storeys: (2, 2), needs: &[BrickAndTimber], ground: detached::SUBURB_PLOT, source: Source::Tall },
    Spec { shape: Shape::Bungalow, name: "bungalow", short: "bungalow", use_: Use::Dwellings, lot: Lot::House, years: (1946, 1975), storeys: (1, 1), needs: &[BrickAndTimber, MotorCar], ground: detached::SUBURB_PLOT, source: Source::Tall },
    Spec { shape: Shape::DetachedLate, name: "detached house of the estates", short: "detached 1976", use_: Use::Dwellings, lot: Lot::House, years: (1976, 2050), storeys: (2, 2), needs: &[BrickAndTimber, MotorCar], ground: detached::SUBURB_PLOT, source: Source::Tall },
    // flats
    Spec { shape: Shape::LowFlats, name: "the council's low flats", short: "low flats", use_: Use::Dwellings, lot: Lot::Joined, years: (1945, 1965), storeys: (3, 4), needs: &[BrickAndTimber, Concrete], ground: (0.12, flats::LOW_FLATS_DEPTH), source: Source::Grown },
    Spec { shape: Shape::FlatsOverShops1905, name: "flats over shops, 1895 to 1918", short: "shop flats 1895", use_: Use::Dwellings, lot: Lot::Joined, years: (1895, 1918), storeys: (4, 6), needs: &[BrickAndTimber], ground: (0.1, blocks::RANGE_DEPTH), source: Source::Dense },
    Spec { shape: Shape::FlatsOverShops1930, name: "flats over shops, 1919 to 1945", short: "shop flats 1919", use_: Use::Dwellings, lot: Lot::Joined, years: (1919, 1945), storeys: (4, 6), needs: &[BrickAndTimber], ground: (0.1, blocks::RANGE_DEPTH), source: Source::Dense },
    Spec { shape: Shape::FlatsOverShops1960, name: "flats over shops, 1946 to 1979", short: "shop flats 1946", use_: Use::Dwellings, lot: Lot::Joined, years: (1946, 1979), storeys: (5, 7), needs: &[BrickAndTimber, Concrete], ground: (0.1, blocks::RANGE_DEPTH), source: Source::Dense },
    Spec { shape: Shape::FlatsOverShops2000, name: "flats over shops, from 1980", short: "shop flats 1980", use_: Use::Dwellings, lot: Lot::Joined, years: (1980, 2050), storeys: (4, 8), needs: &[Concrete, Lift], ground: (0.1, blocks::RANGE_DEPTH), source: Source::Dense },
    Spec { shape: Shape::Court1905, name: "courtyard block, 1895 to 1918", short: "court block 1895", use_: Use::Dwellings, lot: Lot::Block, years: (1895, 1918), storeys: (4, 7), needs: &[BrickAndTimber], ground: blocks::COURT_GROUND, source: Source::Dense },
    Spec { shape: Shape::Court1930, name: "courtyard block, 1919 to 1945", short: "court block 1919", use_: Use::Dwellings, lot: Lot::Block, years: (1919, 1945), storeys: (4, 6), needs: &[BrickAndTimber], ground: blocks::COURT_GROUND, source: Source::Dense },
    Spec { shape: Shape::Court1960, name: "courtyard block, 1946 to 1979", short: "court block 1946", use_: Use::Dwellings, lot: Lot::Block, years: (1946, 1979), storeys: (4, 7), needs: &[BrickAndTimber, Concrete], ground: blocks::COURT_GROUND, source: Source::Dense },
    Spec { shape: Shape::Court2000, name: "courtyard block, from 1980", short: "court block 1980", use_: Use::Dwellings, lot: Lot::Block, years: (1980, 2050), storeys: (4, 8), needs: &[Concrete, Lift], ground: blocks::COURT_GROUND, source: Source::Dense },
    Spec { shape: Shape::Slab, name: "slab block", short: "slab block", use_: Use::Dwellings, lot: Lot::Block, years: (1955, 1979), storeys: (8, 13), needs: &[Concrete, Lift], ground: (0.2, 0.09), source: Source::Grown },
    Spec { shape: Shape::PointBlock, name: "point block", short: "point block", use_: Use::Dwellings, lot: Lot::Joined, years: (1957, 1985), storeys: (10, 22), needs: &[Concrete, Lift], ground: (0.1, 0.1), source: Source::Grown },
    Spec { shape: Shape::FlatsTower, name: "tower of flats", short: "flats tower", use_: Use::Dwellings, lot: Lot::Joined, years: (1986, 2050), storeys: (10, 32), needs: &[Concrete, Lift], ground: (0.1, 0.09), source: Source::Tall },
    Spec { shape: Shape::MillFlats, name: "mill turned to flats", short: "mill flats", use_: Use::Dwellings, lot: Lot::Works, years: (1980, 2050), storeys: (3, 6), needs: &[BrickAndTimber, Lift], ground: (0.2, 0.12), source: Source::Grown },
    // offices
    Spec { shape: Shape::Chambers, name: "bank and chambers", short: "bank chambers", use_: Use::Offices, lot: Lot::Joined, years: (1840, 1939), storeys: (3, 4), needs: &[BrickAndTimber], ground: (offices::BANK.0 + 0.016, offices::BANK.1 + 0.016), source: Source::Grown },
    Spec { shape: Shape::OfficeBlock, name: "office block of brick and terracotta", short: "office block", use_: Use::Offices, lot: Lot::Joined, years: (1900, 1929), storeys: (5, 12), needs: &[SteelFrame, Lift, Electricity], ground: (0.1, 0.08), source: Source::Tall },
    Spec { shape: Shape::SteppedTower, name: "stepped tower in stone", short: "stepped tower", use_: Use::Offices, lot: Lot::Joined, years: (1925, 1959), storeys: (10, 32), needs: &[SteelFrame, Lift, Electricity], ground: (0.11, 0.1), source: Source::Tall },
    Spec { shape: Shape::PodiumOffices, name: "offices on a podium of shops", short: "podium offices", use_: Use::Offices, lot: Lot::Block, years: (1955, 1989), storeys: (6, 18), needs: &[Concrete, Lift, CurtainWall], ground: (offices::OFFICE.0 + 0.02, offices::OFFICE.1 + 0.02), source: Source::Grown },
    Spec { shape: Shape::StoneTower, name: "tower in stone panels and dark glass", short: "stone tower", use_: Use::Offices, lot: Lot::Joined, years: (1990, 2009), storeys: (12, 40), needs: &[SteelFrame, Lift, CurtainWall], ground: (offices::TOWER.0 + 0.03, offices::TOWER.1 + 0.03), source: Source::Grown },
    Spec { shape: Shape::GlassTower, name: "tower in glass with white fins", short: "glass tower", use_: Use::Offices, lot: Lot::Joined, years: (2010, 2050), storeys: (16, 40), needs: &[SteelFrame, Lift, CurtainWall], ground: (offices::TOWER.0 + 0.03, offices::TOWER.1 + 0.03), source: Source::Grown },
    Spec { shape: Shape::TerracottaOffices, name: "offices with terracotta fins", short: "terracotta offices", use_: Use::Offices, lot: Lot::Joined, years: (1995, 2050), storeys: (5, 15), needs: &[Concrete, Lift], ground: (0.1, 0.075), source: Source::Dense },
    // sheds
    Spec { shape: Shape::RetailShed, name: "retail shed in its car park", short: "retail shed", use_: Use::Sheds, lot: Lot::Block, years: (1965, 2050), storeys: (1, 2), needs: &[SteelFrame, MotorCar], ground: (0.17, 0.2), source: Source::Grown },
    Spec { shape: Shape::DistributionShed, name: "distribution shed", short: "distribution shed", use_: Use::Sheds, lot: Lot::Block, years: (1970, 2050), storeys: (2, 3), needs: &[SteelFrame, MotorCar], ground: (0.32, 0.26), source: Source::Grown },
    Spec { shape: Shape::CarPark, name: "car park of decks", short: "car park", use_: Use::Sheds, lot: Lot::Joined, years: (1955, 2050), storeys: (2, 7), needs: &[Concrete, MotorCar], ground: (0.1, 0.07), source: Source::Tall },
];

/// The line of the catalogue for a form.
pub fn spec(shape: Shape) -> &'static Spec {
    FORMS.iter().find(|s| s.shape == shape).expect("every shape has a line in FORMS")
}

// ------------------------------------------------------------------ the question

/// The question a town asks of a lot.
#[derive(Clone, Copy, Debug)]
pub struct Ask {
    pub knows: Knows,
    pub year: i32,
    /// the storeys the lot's rent asks for
    pub storeys: u8,
    pub lot: Lot,
    pub use_: Use,
    /// a number of the lot's own, 0 to 1, that chooses among forms that answer
    /// equally well (so that one street of the 1930s is not all of one form)
    pub pick: f32,
}

/// The answer: the form, the storeys to build it to (what was asked, within
/// what the form stands), and whether it fits the question exactly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Answer {
    pub shape: Shape,
    pub storeys: u8,
    /// false when no form fits the question and this is the nearest
    pub fits: bool,
}

/// How far a form is from a question: nothing if it answers it exactly. A
/// year outside its years costs half a point each, a storey outside its
/// storeys a point, a step of lot four, and another use a hundred. (So a
/// form a decade out of its time is as near as one five storeys off.)
fn distance(s: &Spec, a: &Ask) -> f32 {
    let years = if a.year < s.years.0 { s.years.0 - a.year } else { (a.year - s.years.1).max(0) };
    let storeys = if a.storeys < s.storeys.0 { s.storeys.0 - a.storeys } else { a.storeys.saturating_sub(s.storeys.1) };
    0.5 * years as f32 + storeys as f32 + 4.0 * a.lot.apart(s.lot) as f32 + if a.use_ == s.use_ { 0.0 } else { 100.0 }
}

/// Which form a lot is built as. Only forms whose builders' knowledge the
/// town has are considered (if it knows none of them, all are); of those, the
/// nearest to the question (`distance`), and among the equally near, the one
/// the lot's `pick` falls on. The same question always gets the same answer.
pub fn choose(a: &Ask) -> Answer {
    choose_within(a, u8::MAX)
}

/// `choose`, among the forms that can be built to `most` storeys or fewer,
/// and built to no more than that: where a town keeps its towers to its main
/// streets, a lot elsewhere asks for six storeys and must not get a tower of
/// ten because the nearest form of its year stands no lower. (If no form
/// stands that low, every form is in the running, as `choose` has it.)
pub fn choose_within(a: &Ask, most: u8) -> Answer {
    let low = |s: &&Spec| s.storeys.0 <= most;
    let known: Vec<&Spec> = FORMS.iter().filter(|s| a.knows.all(s.needs)).filter(low).collect();
    let known = if known.is_empty() && most != u8::MAX { FORMS.iter().filter(|s| a.knows.all(s.needs)).collect() } else { known };
    let pool: Vec<&Spec> = if known.is_empty() { FORMS.iter().collect() } else { known };
    let best = pool.iter().map(|s| distance(s, a)).fold(f32::INFINITY, f32::min);
    let near: Vec<&&Spec> = pool.iter().filter(|s| distance(s, a) <= best + 1e-6).collect();
    let i = ((a.pick.clamp(0.0, 1.0) * near.len() as f32) as usize).min(near.len() - 1);
    let s = near[i];
    Answer { shape: s.shape, storeys: a.storeys.clamp(s.storeys.0, s.storeys.1).min(most.max(s.storeys.0)), fits: best == 0.0 }
}

// ------------------------------------------------------------------ drawing a form

/// One form to draw.
#[derive(Clone, Copy, Debug)]
pub struct Form {
    pub shape: Shape,
    /// the storeys it is built to: held within what the form stands
    pub storeys: u8,
    /// the year it was built: held within the form's years, and setting its
    /// style within them where it has more than one (a house's era, its roof)
    pub built: i32,
    /// the ground it stands on, along the street and back from it: forms that
    /// fill their lot take it (a block, a tower, a shed); the rest keep their own size
    pub ground: (f32, f32),
    pub detail: Detail,
    pub seed: Seed,
}

impl Form {
    /// A form on the ground its line of the catalogue gives, at the play level.
    pub fn new(shape: Shape, storeys: u8, built: i32, seed: Seed) -> Form {
        Form { shape, storeys, built, ground: spec(shape).ground, detail: Detail::Play, seed }
    }
}

/// Draw a form on its site, its front to +z, the middle of its ground at
/// the site's origin.
pub fn form(m: &mut Model, site: &Site, f: &Form, pal: &Palette) {
    let s = spec(f.shape);
    let n = f.storeys.clamp(s.storeys.0, s.storeys.1);
    let built = f.built.clamp(s.years.0, s.years.1);
    let (detail, seed) = (f.detail, f.seed);
    let (w, d) = f.ground;
    let roof = Roof::of(built, built, false, false, seed);
    match f.shape {
        Shape::Cottage => {
            let c = houses::Cottage { era: Era::of(built), detail, ..houses::Cottage::new(PLOT.0, PLOT.1, n, roof, seed) };
            houses::cottage(&mut m.solid, &mut m.smoke, &mut m.green, site, &c, pal);
        }
        Shape::Townhouse => {
            let t = houses::Townhouse { era: Era::of(built), detail, ..houses::Townhouse::new(PLOT.0, PLOT.1, n, roof, seed) };
            houses::townhouse(&mut m.solid, &mut m.smoke, site, &t, pal);
        }
        Shape::Terrace => {
            let count = ((w / houses::BACK_PLOT_WIDTH).floor() as usize).clamp(3, 8);
            let t = houses::Terrace { era: Era::of(built), detail, ..houses::Terrace::new(count, houses::Terrace::length_of(count, houses::BACK_PLOT_WIDTH), PLOT.1, n, roof, seed) };
            houses::terrace(&mut m.solid, &mut m.smoke, site, &t, pal);
        }
        Shape::Villa => {
            let v = houses::Villa { era: Era::of(built), detail, ..houses::Villa::new(PLOT.0, PLOT.1, Roof::of(built.max(1830), built.max(1830), false, true, seed), seed) };
            houses::villa(&mut m.solid, &mut m.smoke, &mut m.green, site, &v, pal);
        }
        Shape::SemisInterwar | Shape::CouncilPair | Shape::EstatePair => {
            let garage = f.shape == Shape::CouncilPair && built >= 1958 && seed.pick(77) < 0.5;
            semis::semis(&mut m.solid, &mut m.green, site, &semis::Semis { built, garage, detail, seed }, pal);
        }
        Shape::DetachedInterwar | Shape::Bungalow | Shape::DetachedLate => {
            let cars = if built < 1930 { 0 } else if built < 1976 { 1 } else { 1 + (seed.pick(78) < 0.5) as u8 };
            detached::detached(m, site, &detached::Detached { plot_w: w, plot_d: d, built, cars, detail, seed }, pal);
        }
        Shape::LowFlats => {
            let bays = ((w / (BAY * 1.04)).floor() as usize).max(4);
            flats::low_flats(&mut m.solid, site, &flats::LowFlats { bays, storeys: n, detail, seed }, pal);
        }
        Shape::FlatsOverShops1905 | Shape::FlatsOverShops1930 | Shape::FlatsOverShops1960 | Shape::FlatsOverShops2000 => {
            let b = blocks::Block { shops: [true, false, false, false], detail, ..blocks::Block::new(w, d.min(blocks::RANGE_DEPTH), n, blocks::Age::of(built), seed) };
            blocks::block(&mut m.solid, &mut m.flat, &mut m.green, site, &b, pal);
        }
        Shape::Court1905 | Shape::Court1930 | Shape::Court1960 | Shape::Court2000 => {
            let shops = [true, seed.pick(79) < 0.4, false, seed.pick(80) < 0.4];
            let b = blocks::Block { shops, detail, ..blocks::Block::new(w, d, n, blocks::Age::of(built), seed) };
            blocks::block(&mut m.solid, &mut m.flat, &mut m.green, site, &b, pal);
        }
        Shape::Slab => {
            let bays = ((w * 0.9 / BAY).floor() as usize).max(8);
            flats::slab(&mut m.solid, site, &flats::Slab { bays, storeys: n, detail, seed }, pal);
        }
        Shape::PointBlock => flats::point_block(&mut m.solid, site, &flats::PointBlock { storeys: n, detail, seed }, pal),
        Shape::FlatsTower => towers::flats(m, site, &towers::Tower { w, d, storeys: n, built, detail, seed }, pal),
        Shape::MillFlats => {
            let bays = ((w * 0.7 / (BAY * 1.1)).floor() as usize).max(6);
            mill::mill_flats(&mut m.solid, &mut m.flat, &mut m.green, site, &mill::MillFlats { bays, storeys: n, turned: Some(built), closed: false, detail, seed }, pal);
        }
        Shape::Chambers => offices::bank(&mut m.solid, site, &offices::Bank { detail, seed }, pal),
        Shape::OfficeBlock => towers::office_block(m, site, &towers::Tower { w, d, storeys: n, built, detail, seed }, pal),
        Shape::SteppedTower => towers::stepped(m, site, &towers::Tower { w, d, storeys: n, built, detail, seed }, pal),
        Shape::PodiumOffices => offices::office(&mut m.solid, site, &offices::Office { storeys: n.saturating_sub(2).max(2), detail, seed }, pal),
        Shape::StoneTower | Shape::GlassTower => offices::tower(&mut m.solid, &mut m.green, site, &offices::Tower { storeys: n, built, detail, seed }, pal),
        Shape::TerracottaOffices => {
            let t = blocks::Tower { width: w, depth: d, storeys: n.saturating_sub(2).max(3), age: blocks::Age::Renewal, detail, seed };
            blocks::tower(&mut m.solid, &mut m.green, site, &t, pal);
        }
        Shape::RetailShed => sheds::retail_shed(&mut m.solid, &mut m.flat, &mut m.green, site, &sheds::RetailShed { size: 1.0, detail, seed }, pal),
        Shape::DistributionShed => sheds::dist_shed(&mut m.solid, &mut m.flat, site, &sheds::DistShed { size: 1.0, detail, seed }, pal),
        Shape::CarPark => sheds::deck(m, site, &sheds::Deck { w, d, decks: n - 1, full: 0.4 + 0.5 * seed.pick(81), detail, seed }, pal),
    }
}

// ------------------------------------------------------------------ what the forms share

/// The colours of the twentieth century, by token, as the forms of the
/// "grown" direction name them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Tones {
    pub concrete: Rgba,
    /// concrete in shadow: a recess, a column's face, a lift tower's
    pub concrete_dark: Rgba,
    pub render: Rgba,
    /// a curtain wall of 1965
    pub glass_office: Rgba,
    /// a curtain wall of the new century
    pub glass_tower: Rgba,
    pub spandrel: Rgba,
    pub tarmac: Rgba,
    pub tarmac_light: Rgba,
    pub line: Rgba,
    pub verge: Rgba,
    pub cladding_blue: Rgba,
    pub cladding_green: Rgba,
    pub cladding_white: Rgba,
    pub shed_roof: Rgba,
    /// the bright panels of the 1960s and a retailer's band
    pub accents: [Rgba; 4],
    pub felt: Rgba,
}

impl Tones {
    pub fn of(pal: &Palette) -> Tones {
        let m = &pal.modern;
        let panel = |i: usize| *m.panels[i % m.panels.len()];
        Tones {
            concrete: *m.concrete,
            concrete_dark: shade(*m.concrete, 0.82),
            render: *m.render,
            glass_office: *m.curtain_glass,
            glass_tower: *m.tower_glass,
            spandrel: *m.spandrel,
            tarmac: *pal.ground.tarmac,
            tarmac_light: *pal.ground.tarmac_worn,
            line: *pal.ground.road_line,
            verge: *pal.ground.garden,
            cladding_blue: *m.cladding_blue,
            cladding_green: *m.cladding_green,
            cladding_white: *m.cladding_white,
            shed_roof: *m.shed_roof,
            accents: [panel(0), panel(1), panel(2), panel(3)],
            felt: *m.felt,
        }
    }
}

/// The least a thing may be across from the play distance (`scale::LEVELS`
/// has 0.0012 for the finest; a rectangle that thin is finer still across its
/// diagonal, and a square end of a rail 0.0018 across is 0.0013 corner to corner): a rail, a slab's edge, a parapet that is thinner close to is
/// drawn this thick from there.
pub(crate) const THIN: f32 = 0.0018;

/// As thick as `near` close to, and no thinner than `THIN` from further off.
pub(crate) fn thick(k: &Kit, near: f32) -> f32 {
    if k.near() { near } else { near.max(THIN) }
}

/// Openings along a wall `len` long, one to each bay of `pitch` or a little
/// more, in every storey from `s0` to `s1`: `w` wide and `h` tall, their foot
/// `sill` above the floor.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rows(len: f32, s0: usize, s1: usize, w: f32, h: f32, sill: f32, kind: Open, pitch: f32) -> Vec<Opening> {
    let bays = ((len / pitch).floor() as usize).max(2);
    let mut v = Vec::with_capacity(bays * s1.saturating_sub(s0));
    for s in s0..s1 {
        for b in 0..bays {
            v.push(Opening { u: len * (b as f32 + 0.5) / bays as f32, y: s as f32 * STOREY + sill, w, h, kind });
        }
    }
    v
}

/// A window of storey `s` at `u` along its wall, `w` wide: the houses' sill and height.
pub(crate) fn win(u: f32, s: usize, w: f32, kind: Open) -> Opening {
    Opening { u, y: s as f32 * STOREY + 0.0064, w, h: 0.0076, kind }
}

/// A door on the ground floor at `u`.
pub(crate) fn door(u: f32, paint: Rgba) -> Opening {
    Opening { u, y: FOOT, w: 0.0052, h: 0.0126, kind: Open::Door { paint, over: crate::kit::Over::Nothing } }
}

/// A flat roof on a rectangle at `h`: its felt and, from the play distance,
/// a parapet `p` high in `coping` round it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn flat_roof(k: &mut Kit, x0: f32, x1: f32, z0: f32, z1: f32, h: f32, p: f32, coping: Rgba, felt: Rgba) {
    k.quad([k.pt(x0, h + 0.0004, z1), k.pt(x1, h + 0.0004, z1), k.pt(x1, h + 0.0004, z0), k.pt(x0, h + 0.0004, z0)], felt, Vec3::Y);
    if k.far() || p <= 0.0 {
        return;
    }
    // (from the play distance no thinner than that level may draw)
    let (t, p) = if k.near() { (0.0012, p) } else { (THIN, p.max(THIN)) };
    let top = Some(shade(coping, 1.08));
    k.boxy(x0, x1, z1 - t, z1, h, h + p, coping, top);
    k.boxy(x0, x1, z0, z0 + t, h, h + p, coping, top);
    k.boxy(x0, x0 + t, z0 + t, z1 - t, h, h + p, coping, top);
    k.boxy(x1 - t, x1, z0 + t, z1 - t, h, h + p, coping, top);
}

/// A pane of glass (or any flat panel) on the front (+z) of something, at `z`, from x0 to x1 and y0 to y1.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pane_z(k: &mut Kit, x0: f32, x1: f32, z: f32, y0: f32, y1: f32, col: Rgba, facing: f32) {
    let p = [k.pt(x0, y0, z), k.pt(x1, y0, z), k.pt(x1, y1, z), k.pt(x0, y1, z)];
    let d = k.dir(0.0, 0.0, facing);
    k.quad(p, col, d);
}

/// A pane on a side (+x or -x) at `x`, from z0 to z1.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pane_x(k: &mut Kit, x: f32, z0: f32, z1: f32, y0: f32, y1: f32, col: Rgba, facing: f32) {
    let p = [k.pt(x, y0, z0), k.pt(x, y0, z1), k.pt(x, y1, z1), k.pt(x, y1, z0)];
    let d = k.dir(facing, 0.0, 0.0);
    k.quad(p, col, d);
}

/// Bands round the faces of a rectangular block, `out` proud of it: a
/// ribbon of windows, a spandrel, a cornice.
#[allow(clippy::too_many_arguments)]
pub(crate) fn band_round(k: &mut Kit, rg: &Range, y0: f32, y1: f32, out: f32, col: Rgba, sides: [bool; 4]) {
    for (i, on) in sides.iter().enumerate() {
        if *on {
            let len = rg.face(i).len;
            k.strip(rg, i, 0.0, len, y0, y1, out, col);
        }
    }
}

/// A motor car's colour, by a number of its own.
pub(crate) fn paintwork(pal: &Palette, v: f32) -> Rgba {
    crate::colour::pick(&pal.vehicles.cars, v)
}

/// The turn `MeshBuf::block` wants for a thing whose length runs along `dir`.
fn yaw_of(dir: Vec3) -> f32 {
    (-dir.z).atan2(dir.x)
}

/// A car at `p`, its length along `dir`: a body and a cabin (a car is a fifth of a cottage long).
pub(crate) fn car(b: &mut MeshBuf, p: Vec3, dir: Vec3, col: Rgba, pal: &Palette) {
    let yaw = yaw_of(dir);
    b.block(p, 0.0105, 0.0052, 0.0026, yaw, col, Some(shade(col, 1.1)), 0.0002);
    b.block(p - dir * 0.0008 + Vec3::Y * 0.0026, 0.0056, 0.0046, 0.0019, yaw, *pal.glass.window, Some(col), 0.0);
}

#[cfg(test)]
mod tests;
