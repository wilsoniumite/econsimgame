//! The lab's contact sheets: for each generator, a grid of cells, each one
//! call of it with its parameters varied along the rows and the columns, and
//! a few small scenes. The viewer's `--lab` mode lays a sheet out on plain
//! ground in the game's light and writes a picture of it at each of the
//! camera's standard distances.
//!
//! A sheet is a function that returns its rows of cells, and a line in
//! `SHEETS`. A cell is a label and what one call builds on level ground
//! round the origin; the lab works out the spacing from what is built, so a
//! sheet says nothing about layout.
//!
//! ```text
//! fn my_things(pal: &Palette) -> Vec<Vec<Cell>> {
//!     grid(2, 6, |row, col| {
//!         let p = MyThing { storeys: row as u8 + 1, seed: Seed::of(col as u32) };
//!         Cell::new(format!("{} storeys\nseed {col}", row + 1), |m, site| my_thing(&mut m.solid, site, &p, pal))
//!     })
//! }
//! ```

use glam::{Vec2, Vec3};

use crate::bridge::{road_bridge, RoadBridge};
use crate::farm::{farmstead, Farmstead};
use crate::fence::fence;
use crate::hash::Seed;
use crate::houses::roof::{self, Roof};
use crate::houses::{cottage, terrace, townhouse, Cottage, Terrace, Townhouse, BACK_PLOT_WIDTH, PLOT};
use crate::palette::Palette;
use crate::scenes::{self, Cover, Market, Street, WorksYard};
use crate::site::{Model, Site};
use crate::town;
use crate::trees::{bush, hedge_tree, tree, Species, Tree};
use crate::works::{pit, scaffold, works, yard_shed, Kind, Pit, PitKind, Works, YardShed};

/// The ground a sheet stands on: the land its things stand on in the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Land {
    Pasture,
    Town,
    Yard,
    Wood,
    Heath,
}

/// One cell of a sheet: what one generator call (or one scene) builds, and
/// the label that goes under it.
pub struct Cell {
    pub label: String,
    pub model: Model,
}

impl Cell {
    /// A cell built on level ground at the origin, its front to the south
    /// (the lab then turns every cell a little, to show a side as well).
    pub fn new(label: impl Into<String>, build: impl FnOnce(&mut Model, &Site)) -> Cell {
        Cell::turned(label, 0.0, build)
    }

    /// A cell whose site is turned by `yaw`, to show a thing from another side.
    pub fn turned(label: impl Into<String>, yaw: f32, build: impl FnOnce(&mut Model, &Site)) -> Cell {
        let mut model = Model::default();
        build(&mut model, &Site::level(Vec3::ZERO, yaw));
        Cell { label: label.into(), model }
    }
}

/// `rows` rows of `cols` cells, each made by `cell(row, col)`.
pub fn grid(rows: usize, cols: usize, mut cell: impl FnMut(usize, usize) -> Cell) -> Vec<Vec<Cell>> {
    (0..rows).map(|r| (0..cols).map(|c| cell(r, c)).collect()).collect()
}

/// A contact sheet.
pub struct Sheet {
    /// what `--lab` calls it, and the start of its pictures' file names
    pub name: &'static str,
    /// a line on what it shows and what varies along its rows and columns
    pub about: &'static str,
    pub land: Land,
    /// the rows of cells, top row first
    pub build: fn(&Palette) -> Vec<Vec<Cell>>,
}

/// Every sheet the lab knows. To add one, write its function and add a line here.
pub static SHEETS: &[Sheet] = &[
    Sheet { name: "cottages", about: "A house on its own plot: one to three storeys down the rows, ten seeds across.", land: Land::Town, build: cottages },
    Sheet { name: "townhouses", about: "A house in a high street's frontage: two and three storeys down the rows, ten seeds across.", land: Land::Town, build: townhouses },
    Sheet { name: "terraces", about: "A terraced row: two and three storeys down the rows; three to six houses across, each a seed of its own.", land: Land::Town, build: terraces },
    Sheet { name: "roofs", about: "What a cottage is roofed with by when and where it was built (thatch, tile, stone slate, blue slate), and how it has weathered by the year it is seen: twelve seeds across.", land: Land::Town, build: roofs },
    Sheet { name: "street", about: "A high street with its back lanes, as the towns lay them out, built up to 0.95 in the middle and 0.3 at the ends: as it stands in 1800, and in 1880.", land: Land::Town, build: street },
    Sheet { name: "market", about: "A market place with its church and the houses round it: a small town half built up in 1780, and a big town fully in 1880.", land: Land::Town, build: market },
    Sheet { name: "landmarks", about: "What marks a town out: its church in its churchyard (a small town's, with a spire, and a big town's), station, market cross, market hall and lighthouse.", land: Land::Town, build: landmarks },
    Sheet { name: "works-trades", about: "The cottage trades: a kind in each row, tiers 1 to 4 across.", land: Land::Yard, build: works_trades },
    Sheet { name: "works-iron", about: "Ironmaking: a kind in each row, tiers 1 to 4 across.", land: Land::Yard, build: works_iron },
    Sheet { name: "works-heavy", about: "The heavy trades: a kind in each row, tiers 1 to 4 across.", land: Land::Yard, build: works_heavy },
    Sheet { name: "works-mills", about: "Mills, kilns and the rest: a kind in each row, tiers 1 to 4 across.", land: Land::Yard, build: works_mills },
    Sheet { name: "works-kinds", about: "Every kind of works at tier 2, side by side, to tell them apart by shape.", land: Land::Yard, build: works_kinds },
    Sheet { name: "works-going-up", about: "A works under scaffolding, as it is the year it opens and the year it grows a tier: four kinds across, tier 1 and tier 2.", land: Land::Yard, build: works_going_up },
    Sheet { name: "works-yards", about: "A works among the sheds of its yard, as works ground is filled.", land: Land::Yard, build: works_yards },
    Sheet { name: "yard-sheds", about: "The sheds and warehouses of a works yard: sixteen seeds.", land: Land::Yard, build: yard_sheds },
    Sheet { name: "pits", about: "Pits and their spoil heaps: coal, ironstone, stone and clay down the rows; a small, a middling and a large heap across.", land: Land::Heath, build: pits },
    Sheet { name: "farmsteads", about: "A farmstead seen from its yard: lowland and upland, arable and pasture down the rows; six seeds across.", land: Land::Pasture, build: farmsteads },
    Sheet { name: "trees", about: "Trees and bushes: broadleaf, conifer, hedgerow tree and heath bush down the rows; sizes across (a tree has no age yet).", land: Land::Pasture, build: trees },
    Sheet { name: "woods", about: "A patch of lowland wood and of upland wood, with a tree at every slot as the island has them.", land: Land::Wood, build: woods },
    Sheet { name: "heath", about: "Two patches of heath, fifteen slots each: gorse and broom at about one slot in six (and a tree at one in thirty: none here).", land: Land::Heath, build: heath },
    Sheet { name: "fences", about: "A family's fence round a field, in four colours a family might have (the banners themselves are the interface's): timber rails stained toward the colour, and post caps painted in it.", land: Land::Pasture, build: fences },
    Sheet { name: "bridges", about: "A road bridge: a lane's and a main road's down the rows; wider rivers across (water drawn flat and matt here).", land: Land::Pasture, build: bridges },
];

/// The sheet of this name, if there is one.
pub fn find(name: &str) -> Option<&'static Sheet> {
    SHEETS.iter().find(|s| s.name == name)
}

fn storeys(n: usize) -> String {
    if n == 1 { "1 storey".into() } else { format!("{n} storeys") }
}

// ------------------------------------------------------------------ houses

fn cottages(pal: &Palette) -> Vec<Vec<Cell>> {
    grid(3, 10, |row, col| {
        // standing since before 1750 and seen in 1800: thatch and tile, part weathered
        let seed = Seed::of(col as u32);
        let c = Cottage { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: row as u8 + 1, roof: Roof::of(roof::built_before(1750, seed), 1800, false, false, seed), seed };
        Cell::new(format!("{}\nseed {col}", storeys(row + 1)), |m, site| cottage(&mut m.solid, &mut m.smoke, site, &c, pal))
    })
}

fn townhouses(pal: &Palette) -> Vec<Vec<Cell>> {
    grid(2, 10, |row, col| {
        // built in 1820 and seen in 1860
        let seed = Seed::of(col as u32);
        let t = Townhouse { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: row as u8 + 2, roof: Roof::of(1820, 1860, false, false, seed), seed };
        Cell::new(format!("{}\nseed {col}", storeys(row + 2)), |m, site| townhouse(&mut m.solid, &mut m.smoke, site, &t, pal))
    })
}

fn terraces(pal: &Palette) -> Vec<Vec<Cell>> {
    grid(2, 4, |row, col| {
        let houses = col + 3;
        // a seed for each cell, so the sheet shows every brick and both roofs; built in 1840 and seen in 1870
        let seed = Seed::of((row * 4 + col) as u32);
        let t = Terrace { houses, length: Terrace::length_of(houses, BACK_PLOT_WIDTH), depth: PLOT.1, storeys: row as u8 + 2, roof: Roof::of(1840, 1870, false, false, seed), seed };
        Cell::new(format!("{houses} houses, {}\nseed {}", storeys(row + 2), row * 4 + col), |m, site| terrace(&mut m.solid, &mut m.smoke, site, &t, pal))
    })
}

/// What a house is roofed with by when and where it was built, and how the roof weathers.
fn roofs(pal: &Palette) -> Vec<Vec<Cell>> {
    // (built, seen, in the hills, the railway had come)
    let rows = [
        (1740, 1750, false, false, "lowland, built 1740\nseen 1750"),
        (1740, 1850, false, false, "lowland, built 1740\nseen 1850"),
        (1800, 1850, false, false, "lowland, built 1800\nseen 1850"),
        (1840, 1850, false, false, "lowland, built 1840\nseen 1850"),
        (1870, 1900, false, true, "lowland, built 1870 by rail\nseen 1900"),
        (1740, 1850, true, false, "upland, built 1740\nseen 1850"),
        (1870, 1900, true, true, "upland, built 1870 by rail\nseen 1900"),
    ];
    rows.into_iter()
        .map(|(built, year, upland, by_rail, label)| {
            (0..12)
                .map(|col| {
                    // seeds of their own, so no column is all one covering; one storey, as a raised cottage loses its thatch
                    let seed = Seed::of(40 + col);
                    let c = Cottage { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: 1, roof: Roof::of(built, year, upland, by_rail, seed), seed };
                    Cell::new(if col == 0 { label.to_string() } else { format!("seed {}", 40 + col) }, |m, site| cottage(&mut m.solid, &mut m.smoke, site, &c, pal))
                })
                .collect()
        })
        .collect()
}

fn street(pal: &Palette) -> Vec<Vec<Cell>> {
    [(1800, "in 1800: separate houses on the back lanes"), (1880, "in 1880: terraces on the back lanes, and slate")]
        .into_iter()
        .map(|(year, label)| {
            let s = Street { length: 1.0, middle: 0.95, ends: 0.3, year, id: 3 };
            vec![Cell::new(label, |m, site| scenes::street(m, site, &s, pal))]
        })
        .collect()
}

fn market(pal: &Palette) -> Vec<Vec<Cell>> {
    let cell = |big: bool, density: f32, year: i32, label: &str| Cell::new(label, |m, site| scenes::market(m, site, &Market { big, density, year, id: 2 }, pal));
    vec![vec![cell(false, 0.5, 1780, "a small town in 1780, built up to 0.5"), cell(true, 1.0, 1880, "a big town in 1880, fully built up")]]
}

fn landmarks(pal: &Palette) -> Vec<Vec<Cell>> {
    // the churches are seen from the south, the tower and its gate to the left
    let church = |big: bool, spire: bool, label: &str| {
        let c = town::Church { big, spire, seed: Seed::of(big as u32 + 1) };
        Cell::turned(label, -0.75, |m, site| town::church(&mut m.solid, site, &c, pal))
    };
    vec![vec![
        church(false, false, "a small town's church"),
        church(false, true, "the same with a spire"),
        church(true, true, "a big town's church"),
        Cell::new("station", |m, site| town::station(&mut m.solid, site, pal)),
        Cell::new("market cross", |m, site| town::market_cross(&mut m.solid, site, pal)),
        Cell::new("market hall", |m, site| town::market_hall(&mut m.solid, site, pal)),
        Cell::new("lighthouse", |m, site| town::lighthouse(&mut m.solid, site, pal)),
    ]]
}

// ------------------------------------------------------------------ works

/// A row for each kind, tiers 1 to 4 across.
fn works_rows(kinds: &[Kind], pal: &Palette) -> Vec<Vec<Cell>> {
    kinds
        .iter()
        .enumerate()
        .map(|(i, &kind)| {
            (1..=4u8)
                .map(|tier| {
                    let w = Works { kind, tier, seed: (i as f32 * 4.0 + tier as f32) * 0.37 };
                    Cell::new(format!("{}\ntier {tier}", kind.label()), |m, site| works(&mut m.solid, &mut m.smoke, site, &w, pal))
                })
                .collect()
        })
        .collect()
}

fn works_trades(pal: &Palette) -> Vec<Vec<Cell>> {
    works_rows(&[Kind::HandLoom, Kind::Smithy, Kind::Millwright], pal)
}

fn works_iron(pal: &Palette) -> Vec<Vec<Cell>> {
    works_rows(&[Kind::CharcoalBurning, Kind::Bloomery, Kind::CharcoalFurnace, Kind::CokeFurnace], pal)
}

fn works_heavy(pal: &Palette) -> Vec<Vec<Cell>> {
    works_rows(&[Kind::ForgeRails, Kind::RollingMill, Kind::SteelRolling, Kind::Bessemer, Kind::EngineWorks], pal)
}

fn works_mills(pal: &Palette) -> Vec<Vec<Cell>> {
    works_rows(&[Kind::TextileMill, Kind::Brickworks, Kind::CrucibleSteel, Kind::Other], pal)
}

fn works_kinds(pal: &Palette) -> Vec<Vec<Cell>> {
    grid(4, 4, |row, col| {
        let kind = Kind::ALL[row * 4 + col];
        Cell::new(kind.label(), |m, site| works(&mut m.solid, &mut m.smoke, site, &Works { kind, tier: 2, seed: (row * 4 + col) as f32 * 0.37 }, pal))
    })
}

/// The one state a works has besides its tier: scaffolding, the year it opens and the year it grows a tier.
fn works_going_up(pal: &Palette) -> Vec<Vec<Cell>> {
    let kinds = [Kind::Smithy, Kind::CokeFurnace, Kind::TextileMill, Kind::RollingMill];
    grid(2, 4, |row, col| {
        let (kind, tier) = (kinds[col], row as u8 + 1);
        Cell::new(format!("{}\ntier {tier}, going up", kind.label()), |m, site| {
            works(&mut m.solid, &mut m.smoke, site, &Works { kind, tier, seed: col as f32 * 0.37 }, pal);
            scaffold(&mut m.solid, site, tier, pal);
        })
    })
}

fn works_yards(pal: &Palette) -> Vec<Vec<Cell>> {
    [(Kind::CokeFurnace, 3u8), (Kind::TextileMill, 3), (Kind::RollingMill, 4)]
        .into_iter()
        .enumerate()
        .map(|(i, (kind, tier))| vec![Cell::new(format!("{}, tier {tier}, in its yard", kind.label()), |m, site| scenes::works_yard(m, site, &WorksYard { kind, tier, id: i as i32 + 1 }, pal))])
        .collect()
}

fn yard_sheds(pal: &Palette) -> Vec<Vec<Cell>> {
    grid(2, 8, |row, col| {
        let n = (row * 8 + col) as u32;
        Cell::new(format!("seed {n}"), |m, site| yard_shed(&mut m.solid, &mut m.smoke, site, &YardShed { seed: Seed::of(n), smoke_seed: n as f32 }, pal))
    })
}

fn pits(pal: &Palette) -> Vec<Vec<Cell>> {
    let kinds = [(PitKind::Coal, "colliery"), (PitKind::Ironstone, "iron mine"), (PitKind::Stone, "quarry"), (PitKind::Clay, "clay pit")];
    grid(4, 3, |row, col| {
        let (kind, name) = kinds[row];
        let v = col as f32 * 0.5;
        Cell::new(format!("{name}\nheap {v}"), |m, site| pit(&mut m.solid, site, &Pit { kind, v }, pal))
    })
}

// ------------------------------------------------------------------ the country

fn farmsteads(pal: &Palette) -> Vec<Vec<Cell>> {
    let kinds = [(false, true, "lowland arable"), (false, false, "lowland pasture"), (true, true, "upland arable"), (true, false, "upland pasture")];
    grid(4, 6, |row, col| {
        let (upland, arable, name) = kinds[row];
        let f = Farmstead { arable, upland, seed: Seed::of(col as u32) };
        // turned about, so the yard and the barn are on the camera's side of the house
        Cell::turned(format!("{name}\nseed {col}"), std::f32::consts::PI, |m, site| farmstead(&mut m.solid, &mut m.green, site, &f, pal))
    })
}

fn trees(pal: &Palette) -> Vec<Vec<Cell>> {
    let sizes = [0.6, 0.75, 0.9, 1.05, 1.25];
    let mut rows = Vec::new();
    for (species, name) in [(Species::Broadleaf, "broadleaf"), (Species::Conifer, "conifer")] {
        rows.push(
            sizes
                .iter()
                .enumerate()
                .map(|(i, &size)| Cell::new(format!("{name}\nsize {size}"), |m, site| tree(&mut m.green, site, &Tree { species, size, tint: i as f32 / 5.0, turn: i as f32 * 1.3 }, pal)))
                .collect(),
        );
    }
    // a hedgerow tree is drawn from 0.55 to 0.85
    rows.push([0.55, 0.65, 0.75, 0.85].iter().enumerate().map(|(i, &size)| Cell::new(format!("hedgerow tree\nsize {size}"), |m, site| hedge_tree(&mut m.green, site, size, i as f32 / 4.0, i as f32 * 1.3, pal))).collect());
    rows.push(
        [(0.0, "gorse\nsmall"), (0.49, "gorse\nlarge"), (0.5, "broom\nsmall"), (1.0, "broom\nlarge")]
            .iter()
            .enumerate()
            .map(|(i, &(v, name))| Cell::new(name, |m, site| bush(&mut m.green, site, v, i as f32 * 1.3, pal)))
            .collect(),
    );
    rows
}

fn woods(pal: &Palette) -> Vec<Vec<Cell>> {
    [(Cover::LowlandWood, "a lowland wood"), (Cover::UplandWood, "an upland wood")]
        .into_iter()
        .enumerate()
        .map(|(i, (what, name))| vec![Cell::new(name, |m, site| scenes::cover(m, site, what, i as i32, pal))])
        .collect()
}

fn heath(pal: &Palette) -> Vec<Vec<Cell>> {
    // fifteen slots to a patch: two or three bushes, and a tree on one patch in two (neither of these)
    vec![[2, 7].into_iter().map(|id| Cell::new(format!("heath, patch {id}"), |m, site| scenes::cover(m, site, Cover::Heath, id, pal))).collect()]
}

fn fences(pal: &Palette) -> Vec<Vec<Cell>> {
    // stand-ins for the families' banners, which are the interface's colours and not in the palette
    let families = [(*pal.walls.lighthouse_red, "a red family's"), (*pal.vehicles.engine, "a green family's"), (*pal.slate.house, "a blue family's"), (*pal.marks.finial, "a gold family's")];
    vec![families
        .into_iter()
        .map(|(colour, name)| {
            Cell::new(format!("{name} fence\nround a field"), |m, site| {
                // a field a quarter of a square by a sixth, a post every 0.09 or so
                let (w, d) = (0.27, 0.18);
                let mut line = Vec::new();
                for (from, to, posts) in [((-w, -d), (w, -d), 6), ((w, -d), (w, d), 4), ((w, d), (-w, d), 6), ((-w, d), (-w, -d), 4)] {
                    for i in 0..posts {
                        let t = i as f32 / posts as f32;
                        line.push(site.at(0.5 * (from.0 + (to.0 - from.0) * t), 0.5 * (from.1 + (to.1 - from.1) * t)));
                    }
                }
                line.push(line[0]);
                fence(&mut m.solid, &line, colour, pal);
            })
        })
        .collect()]
}

fn bridges(pal: &Palette) -> Vec<Vec<Cell>> {
    let halves = [0.012, 0.025, 0.04, 0.06, 0.1];
    grid(2, 5, |row, col| {
        let (main, half) = (row == 1, halves[col]);
        let b = RoadBridge { at: Vec2::ZERO, along: Vec2::X, half, level: 0.002, main };
        let label = format!("{}\nriver {} wide", if main { "main road" } else { "lane" }, half * 2.0);
        Cell::new(label, |m, _| {
            // the river under it and the road up to it, flat on the lab's level ground
            let (reach, y) = (0.07, Vec3::Y * 0.002);
            m.flat.ribbon(&[Vec3::new(0.0, 0.0, -reach) + y, Vec3::new(0.0, 0.0, reach) + y], half * 2.0, *pal.water.river);
            let (road, width) = if main { (*pal.ground.road_gravel, 0.034) } else { (*pal.ground.lane_dust, 0.022) };
            for side in [-1.0f32, 1.0] {
                let (a, c) = (side * b.span() * 0.5, side * (b.span() * 0.5 + 0.05));
                m.flat.ribbon(&[Vec3::new(a, 0.01, 0.0), Vec3::new(c, 0.01, 0.0)], width, road);
            }
            road_bridge(&mut m.solid, &b, &|_, _| 0.0, pal);
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sheet_builds_and_every_cell_has_something_sound_in_it() {
        let pal = Palette::standard();
        for sheet in SHEETS {
            let rows = (sheet.build)(&pal);
            assert!(!rows.is_empty(), "{} has no rows", sheet.name);
            for (r, row) in rows.iter().enumerate() {
                assert!(!row.is_empty(), "{} row {r} is empty", sheet.name);
                for (c, cell) in row.iter().enumerate() {
                    assert!(!cell.model.is_empty(), "{} cell {r},{c} ({}) is empty", sheet.name, cell.label);
                    assert!(cell.model.is_sound(), "{} cell {r},{c} ({}) has a bad number or index", sheet.name, cell.label);
                    assert!(!cell.label.is_empty() && cell.label.lines().all(|l| l.chars().count() <= 48) && cell.label.lines().count() <= 2, "{} cell {r},{c}: the label is {:?}", sheet.name, cell.label);
                }
            }
        }
    }

    #[test]
    fn a_cell_fits_the_closest_view() {
        // the camera at its closest sees about 1.2 by 0.9 of a map cell; a cell
        // wider or deeper than that would be cut off in the close picture
        let pal = Palette::standard();
        for sheet in SHEETS {
            for row in (sheet.build)(&pal) {
                for cell in row {
                    let (lo, hi) = cell.model.bounds().unwrap();
                    assert!(hi.x - lo.x <= 1.15, "{}: {} is {} wide", sheet.name, cell.label, hi.x - lo.x);
                    assert!((hi.z - lo.z) + (hi.y - lo.y) * 0.8 <= 0.8, "{}: {} is {} deep and {} tall", sheet.name, cell.label, hi.z - lo.z, hi.y - lo.y);
                }
            }
        }
    }

    #[test]
    fn sheet_names_are_plain_and_each_is_its_own() {
        for (i, s) in SHEETS.iter().enumerate() {
            assert!(s.name.chars().all(|c| c.is_ascii_lowercase() || c == '-'), "{}", s.name);
            assert!(s.name != "all" && s.name != "list", "{} is a word the lab keeps for itself", s.name);
            assert!(SHEETS[..i].iter().all(|t| t.name != s.name), "two sheets are called {}", s.name);
            assert!(find(s.name).is_some());
            assert!(s.about.ends_with('.'));
        }
        assert!(find("no such sheet").is_none());
    }

    #[test]
    fn a_sheet_is_the_same_each_time_it_is_built() {
        let pal = Palette::standard();
        for sheet in SHEETS {
            let (a, b) = ((sheet.build)(&pal), (sheet.build)(&pal));
            assert!(a.iter().flatten().zip(b.iter().flatten()).all(|(x, y)| x.model == y.model && x.label == y.label), "{}", sheet.name);
        }
    }
}
