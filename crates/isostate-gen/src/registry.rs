//! Every generator of the island, in one list, so that each can be built at
//! each level of detail and held to the scale (`scale::faults`).
//!
//! An entry names the generator functions it stands for and builds a few
//! specimens of the thing: the sizes, ages and seeds that show what it can
//! be. The tests here build every specimen at every level and measure it,
//! and read the crate's own source for generator functions (public functions
//! that write into a `MeshBuf`, a `Greenery` or a `Model`) that no entry
//! covers: a new generator fails the tests until it is listed here. The
//! lab's `scale` sheet stands the first specimen of every entry side by side
//! at each level with its numbers under it.

use glam::Vec3;

use crate::hash::Seed;
use crate::houses::roof::Roof;
use crate::houses::{cottage, farmhouse, terrace, townhouse, villa, Cottage, Detail, Era, Farmhouse, Terrace, Townhouse, Villa, BACK_PLOT_WIDTH, PLOT};
use crate::palette::Palette;
use crate::site::{Model, Site};

/// What kind of thing a generator makes, which settles what of the scale it answers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// walls under a roof: answers to all of the scale
    Building,
    /// built, but not a building (a bridge, a fence, a pit's heaps, a
    /// market cross): its grain is held to the scale, its walls and roofs are not
    Structure,
    /// grown: held only to the grain of its level
    Plant,
}

/// One thing a generator made, for measuring and for the `scale` sheet.
pub struct Specimen {
    pub label: String,
    pub model: Model,
}

/// A generator, or a family of them that share a struct of parameters.
pub struct Entry {
    pub name: &'static str,
    pub class: Class,
    /// the generator functions this entry builds, as `module::function`
    pub covers: &'static [&'static str],
    /// a few specimens at a level of detail; the first is the one the `scale` sheet shows
    pub build: fn(Detail, &Palette) -> Vec<Specimen>,
    /// whether the thing changes with the level at all (a tree does not: the
    /// viewer leaves the island's trees as they are when the camera alone moves)
    pub levels: bool,
}

fn specimen(label: impl Into<String>, build: impl FnOnce(&mut Model, &Site)) -> Specimen {
    let mut model = Model::default();
    build(&mut model, &Site::level(Vec3::ZERO, 0.0));
    Specimen { label: label.into(), model }
}

/// The years a house of each era is taken to have been built in, for its roof.
fn year_of(era: Era) -> i32 {
    match era {
        Era::Old => 1700,
        Era::Georgian => 1780,
        Era::Brick => 1840,
        Era::Late => 1875,
    }
}

fn cottages(detail: Detail, pal: &Palette) -> Vec<Specimen> {
    let mut out = Vec::new();
    for (i, era) in Era::ALL.into_iter().enumerate() {
        for n in 1..=3u8 {
            for upland in [false, true] {
                let seed = Seed::of(5 + i as u32 * 7 + n as u32);
                let c = Cottage { era, upland, detail, ..Cottage::new(PLOT.0, PLOT.1, n, Roof::of(year_of(era), 1890, upland, false, seed), seed) };
                out.push(specimen(format!("cottage, {}, {n} storeys", era.label()), |m, site| cottage(&mut m.solid, &mut m.smoke, &mut m.green, site, &c, pal)));
            }
        }
    }
    out
}

fn townhouses(detail: Detail, pal: &Palette) -> Vec<Specimen> {
    let mut out = Vec::new();
    for (i, era) in [Era::Georgian, Era::Old, Era::Brick, Era::Late].into_iter().enumerate() {
        for n in 2..=4u8 {
            for shop in [true, false] {
                let seed = Seed::of(5 + i as u32 * 11 + n as u32);
                let t = Townhouse { era, shop, detail, ..Townhouse::new(PLOT.0, PLOT.1, n, Roof::of(year_of(era), 1890, false, false, seed), seed) };
                out.push(specimen(format!("townhouse, {}, {n} storeys", era.label()), |m, site| townhouse(&mut m.solid, &mut m.smoke, site, &t, pal)));
            }
        }
    }
    out
}

fn terraces(detail: Detail, pal: &Palette) -> Vec<Specimen> {
    let mut out = Vec::new();
    for (i, (houses, n, era, upland)) in [(4, 2, Era::Brick, false), (6, 3, Era::Brick, false), (3, 2, Era::Late, false), (5, 2, Era::Late, true)].into_iter().enumerate() {
        let seed = Seed::of(5 + i as u32);
        let t = Terrace { era, upland, detail, ..Terrace::new(houses, Terrace::length_of(houses, BACK_PLOT_WIDTH), PLOT.1, n, Roof::of(year_of(era), 1890, upland, era == Era::Late, seed), seed) };
        out.push(specimen(format!("terrace of {houses}, {}", era.label()), |m, site| terrace(&mut m.solid, &mut m.smoke, site, &t, pal)));
    }
    out
}

fn villas(detail: Detail, pal: &Palette) -> Vec<Specimen> {
    let mut out = Vec::new();
    for (i, (era, upland)) in [(Era::Late, false), (Era::Georgian, false), (Era::Brick, false), (Era::Late, true)].into_iter().enumerate() {
        for s in 0..3 {
            let seed = Seed::of(5 + i as u32 * 3 + s);
            let v = Villa { era, upland, detail, ..Villa::new(PLOT.0, PLOT.1, Roof::of(year_of(era).max(1830), 1890, upland, true, seed), seed) };
            out.push(specimen(format!("villa, {}", era.label()), |m, site| villa(&mut m.solid, &mut m.smoke, &mut m.green, site, &v, pal)));
        }
    }
    out
}

fn farmhouses(detail: Detail, pal: &Palette) -> Vec<Specimen> {
    let mut out = Vec::new();
    for (i, (storeys, upland, thatch, wing)) in [(2, false, false, true), (1, false, true, false), (1, true, false, false), (2, true, false, true)].into_iter().enumerate() {
        let f = Farmhouse { storeys, upland, thatch, wing, detail, seed: Seed::of(5 + i as u32) };
        out.push(specimen(format!("farmhouse, {storeys} storeys"), |m, site| farmhouse(&mut m.solid, &mut m.smoke, site, &f, pal)));
    }
    out
}

/// Every generator. To add one: write its specimens and add a line here.
pub static REGISTRY: &[Entry] = &[
    Entry { name: "cottage", class: Class::Building, covers: &["houses/cottage::cottage"], build: cottages, levels: true },
    Entry { name: "townhouse", class: Class::Building, covers: &["houses/townhouse::townhouse"], build: townhouses, levels: true },
    Entry { name: "terrace", class: Class::Building, covers: &["houses/terrace::terrace"], build: terraces, levels: true },
    Entry { name: "villa", class: Class::Building, covers: &["houses/villa::villa"], build: villas, levels: true },
    Entry { name: "farmhouse", class: Class::Building, covers: &["houses/farmhouse::farmhouse"], build: farmhouses, levels: true },
];

/// The houses: the entries the scale was read off, which the others are held to.
pub const REFERENCE: [&str; 5] = ["cottage", "townhouse", "terrace", "villa", "farmhouse"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::{self, measure};

    #[test]
    fn every_generator_keeps_to_the_scale_at_every_level() {
        let pal = Palette::standard();
        let mut wrong = Vec::new();
        for e in REGISTRY {
            let mut last: Option<Vec<usize>> = None;
            for detail in Detail::ALL {
                let specimens = (e.build)(detail, &pal);
                assert!(!specimens.is_empty(), "{} has no specimens", e.name);
                for s in &specimens {
                    assert!(s.model.is_sound() && !s.model.is_empty(), "{}: {} at the {} level", e.name, s.label, detail.label());
                    let m = measure(&s.model.solid, &pal);
                    let faults = match e.class {
                        Class::Building => scale::faults(&m, detail),
                        // what is not a building answers for its grain alone
                        _ => scale::faults(&scale::Measure { wall_area: 0.0, roof_area: 0.0, blank: 0.0, storey: None, ..m }, detail),
                    };
                    for f in faults {
                        wrong.push(format!("{}: {} at the {} level: {f}", e.name, s.label, detail.label()));
                    }
                }
                // each level draws no less than the one before it, and the same things
                let now: Vec<usize> = specimens.iter().map(|s| s.model.solid.triangles()).collect();
                if let Some(was) = &last {
                    assert_eq!(was.len(), now.len(), "{} has other specimens at the {} level", e.name, detail.label());
                    for (i, s) in specimens.iter().enumerate() {
                        assert!(now[i] >= was[i], "{}: {} has less to it at the {} level than at the one before", e.name, s.label, detail.label());
                        assert!(e.levels || now[i] == was[i], "{}: {} says it has no levels and differs by level", e.name, s.label);
                    }
                }
                last = Some(now);
            }
        }
        assert!(wrong.is_empty(), "{} off the scale:\n{}", wrong.len(), wrong.join("\n"));
    }

    #[test]
    fn the_houses_are_the_reference_and_in_the_registry() {
        for name in REFERENCE {
            assert!(REGISTRY.iter().any(|e| e.name == name && e.class == Class::Building), "{name}");
        }
        for (i, e) in REGISTRY.iter().enumerate() {
            assert!(REGISTRY[..i].iter().all(|o| o.name != e.name), "two entries are called {}", e.name);
            assert!(!e.covers.is_empty(), "{} covers no generator", e.name);
        }
    }

    /// The public generator functions of the crate, read from its source: every
    /// free `pub fn` that is handed a buffer to write into, as `file::function`.
    fn generators_in_the_source() -> Vec<String> {
        fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, root, out);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let module = path.strip_prefix(root).unwrap().with_extension("").to_string_lossy().replace('\\', "/");
                let text = std::fs::read_to_string(&path).unwrap();
                let lines: Vec<&str> = text.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    // a free function: `pub fn` at the start of a line
                    let Some(rest) = line.strip_prefix("pub fn ") else { continue };
                    let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                    // its parameters may run over several lines
                    let mut sig = String::new();
                    for l in &lines[i..(i + 12).min(lines.len())] {
                        sig.push_str(l);
                        if l.contains('{') {
                            break;
                        }
                    }
                    if ["&mut MeshBuf", "&mut Greenery", "&mut Model"].iter().any(|b| sig.contains(b)) {
                        out.push(format!("{module}::{name}"));
                    }
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut out = Vec::new();
        walk(&root, &root, &mut out);
        out.sort();
        out
    }

    /// Public functions that write into a buffer and are not generators of a thing.
    const NOT_GENERATORS: &[(&str, &str)] = &[
        ("works/kit::plume", "smoke, which every flue shares"),
        ("scenes::street", "a lab scene of houses, each of which is listed"),
        ("scenes::market", "a lab scene of houses and a church, each of which is listed"),
        ("scenes::cover", "a lab scene of trees and bushes, each of which is listed"),
    ];

    /// Work in progress: the generators not yet brought to the scale. To be emptied and removed.
    const NOT_YET: &[&str] = &[
        "bridge::road_bridge", "farm::farmstead", "fence::fence", "parts::gabled", "town/church::church", "town/lighthouse::lighthouse", "town/market::market_cross", "town/market::market_hall",
        "town/station::station", "trees::bush", "trees::hedge_tree", "trees::tree", "works/mod::works", "works/pit::pit",
    ];

    #[test]
    fn a_generator_that_is_not_in_the_registry_fails() {
        let found = generators_in_the_source();
        assert!(found.len() >= 10, "the source was not read: {found:?}");
        let covered: Vec<&str> = REGISTRY.iter().flat_map(|e| e.covers.iter().copied()).collect();
        let mut missing = Vec::new();
        for g in &found {
            if !covered.contains(&g.as_str()) && !NOT_GENERATORS.iter().any(|(n, _)| n == g) && !NOT_YET.contains(&g.as_str()) {
                missing.push(g.clone());
            }
        }
        assert!(missing.is_empty(), "these generators are not in registry::REGISTRY, so nothing holds them to the scale: {missing:?}");
        // and nothing is listed that is not there
        for c in covered.iter().chain(NOT_GENERATORS.iter().map(|(n, _)| n)) {
            assert!(found.iter().any(|g| g == c), "{c} is listed and is not a generator in the source");
        }
    }

    /// Prints what every specimen measures at every level
    /// (`cargo test -p isostate-gen measures -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn measures() {
        let pal = Palette::standard();
        for e in REGISTRY {
            for detail in Detail::ALL {
                let (mut lo, mut hi) = (scale::Measure { smallest: f32::MAX, wall: f32::MAX, roof: f32::MAX, ..Default::default() }, scale::Measure::default());
                for s in (e.build)(detail, &pal) {
                    let m = measure(&s.model.solid, &pal);
                    if std::env::var_os("EACH").is_some() {
                        println!("    {:<40} {:>5} tris, finest {:.5}, wall {:>5.1} over {:>5.1}, roof {:>5.2} over {:>5.1}, bare {:.1}, storey {:?}", s.label, m.triangles, m.smallest, m.wall, m.wall_area, m.roof, m.roof_area, m.blank, m.storey);
                    }
                    lo.smallest = lo.smallest.min(m.smallest);
                    hi.smallest = hi.smallest.max(m.smallest);
                    lo.wall = lo.wall.min(m.wall);
                    hi.wall = hi.wall.max(m.wall);
                    lo.roof = lo.roof.min(m.roof);
                    hi.roof = hi.roof.max(m.roof);
                    hi.blank = hi.blank.max(m.blank);
                    hi.triangles = hi.triangles.max(m.triangles);
                }
                println!("measure {:<18} {:<5} finest {:.5} to {:.5}, wall {:>5.1} to {:>5.1}, roof {:>5.2} to {:>5.2}, bare up to {:.1}, up to {} triangles", e.name, detail.label(), lo.smallest, hi.smallest, lo.wall, hi.wall, lo.roof, hi.roof, hi.blank, hi.triangles);
            }
        }
    }
}
