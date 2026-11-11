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
use crate::hash::Seed;
use crate::houses::{cottage, terrace, townhouse, Cottage, Terrace, Townhouse, BACK_PLOT_WIDTH, PLOT};
use crate::palette::Palette;
use crate::scenes::{self, Cover, Market, Street};
use crate::site::{Model, Site};
use crate::town;
use crate::trees::{bush, hedge_tree, tree, Species, Tree};
use crate::works::{pit, works, Kind, Pit, PitKind, State, Works};

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
    Sheet { name: "street", about: "A high street with its back lanes, as the towns lay them out, built up to 0.95 in the middle and 0.3 at the ends: with back lanes built before 1815, and after.", land: Land::Town, build: street },
    Sheet { name: "market", about: "A market place with its church and the houses round it: a small town half built up, and a big town fully.", land: Land::Town, build: market },
    Sheet { name: "landmarks", about: "What marks a town out: its church (and a big town's), station, market cross, market hall and lighthouse.", land: Land::Town, build: landmarks },
    Sheet { name: "works-kinds", about: "Every kind of works at three times the standard size, built about 1820, side by side: to tell them apart by shape.", land: Land::Pasture, build: works_kinds },
    Sheet { name: "works-small", about: "Every kind of works at its smallest, 0.4 of the standard size, as most stand in the game today: built in 1760, then built in 1870.", land: Land::Yard, build: works_small },
    Sheet { name: "works-standard", about: "Every kind of works at the standard size: built in 1760, then built in 1870.", land: Land::Yard, build: works_standard },
    Sheet { name: "works-great", about: "Every kind of works at thirty times the standard size, built about 1850.", land: Land::Pasture, build: works_great },
    Sheet { name: "works-states", about: "The states of a works down the rows: going up, being enlarged, working, standing idle, derelict. Six kinds across, at three times the standard size.", land: Land::Pasture, build: works_states },
    Sheet { name: "works-states-great", about: "The states of a great works down the rows: going up, being enlarged, working, standing idle, derelict. Three kinds across, at thirty times the standard size.", land: Land::Pasture, build: works_states_great },
    Sheet { name: "works-loom", about: "Handloom weavers: built in 1760 by water (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Town, build: works_loom },
    Sheet { name: "works-smithy", about: "Smithies: built in 1760 by water (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Town, build: works_smithy },
    Sheet { name: "works-millwright", about: "Millwrights: built in 1760 by water (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Town, build: works_millwright },
    Sheet { name: "works-shops", about: "Shops and trades: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Town, build: works_shops },
    Sheet { name: "works-charcoal", about: "Charcoal burning: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Wood, build: works_charcoal },
    Sheet { name: "works-bloomery", about: "Bloomeries: built in 1760 by water (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: works_bloomery },
    Sheet { name: "works-charcoal-furnace", about: "Charcoal blast furnaces: built in 1760 by water (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: works_charcoal_furnace },
    Sheet { name: "works-coke-furnace", about: "Coke blast furnaces: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: works_coke_furnace },
    Sheet { name: "works-forge", about: "Forges making wrought rails: built in 1760 by water (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: works_forge },
    Sheet { name: "works-rolling", about: "Rolling mills: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: works_rolling },
    Sheet { name: "works-rail-mill", about: "Steel rail mills: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: works_rail_mill },
    Sheet { name: "works-crucible", about: "Crucible steel: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Town, build: works_crucible },
    Sheet { name: "works-converter", about: "Converter shops: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: works_converter },
    Sheet { name: "works-engine", about: "Engine works: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Town, build: works_engine },
    Sheet { name: "works-mill", about: "Powered mills: built in 1760 by water (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: works_mill },
    Sheet { name: "works-brick", about: "Brickworks: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: works_brick },
    Sheet { name: "works-other", about: "A works of a kind the generators do not know: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Town, build: works_other },
    Sheet { name: "pits", about: "Pits and their spoil heaps: coal, ironstone, stone and clay down the rows; a small, a middling and a large heap across.", land: Land::Heath, build: pits },
    Sheet { name: "farmsteads", about: "A farmstead seen from its yard: lowland and upland, arable and pasture down the rows; six seeds across.", land: Land::Pasture, build: farmsteads },
    Sheet { name: "trees", about: "Trees and bushes: broadleaf, conifer, hedgerow tree and heath bush down the rows; sizes across (a tree has no age yet).", land: Land::Pasture, build: trees },
    Sheet { name: "woods", about: "A patch of lowland wood and of upland wood, with a tree at every slot as the island has them.", land: Land::Wood, build: woods },
    Sheet { name: "heath", about: "Two patches of heath, fifteen slots each: gorse and broom at about one slot in six (and a tree at one in thirty: none here).", land: Land::Heath, build: heath },
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
        let c = Cottage { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: row as u8 + 1, seed: Seed::of(col as u32) };
        Cell::new(format!("{}\nseed {col}", storeys(row + 1)), |m, site| cottage(&mut m.solid, &mut m.smoke, site, &c, pal))
    })
}

fn townhouses(pal: &Palette) -> Vec<Vec<Cell>> {
    grid(2, 10, |row, col| {
        let t = Townhouse { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: row as u8 + 2, seed: Seed::of(col as u32) };
        Cell::new(format!("{}\nseed {col}", storeys(row + 2)), |m, site| townhouse(&mut m.solid, &mut m.smoke, site, &t, pal))
    })
}

fn terraces(pal: &Palette) -> Vec<Vec<Cell>> {
    grid(2, 4, |row, col| {
        let houses = col + 3;
        // a seed for each cell, so the sheet shows every brick and both roofs
        let t = Terrace { houses, length: Terrace::length_of(houses, BACK_PLOT_WIDTH), depth: PLOT.1, storeys: row as u8 + 2, seed: Seed::of((row * 4 + col) as u32) };
        Cell::new(format!("{houses} houses, {}\nseed {}", storeys(row + 2), row * 4 + col), |m, site| terrace(&mut m.solid, &mut m.smoke, site, &t, pal))
    })
}

fn street(pal: &Palette) -> Vec<Vec<Cell>> {
    [false, true]
        .into_iter()
        .map(|terraces| {
            let s = Street { length: 1.0, middle: 0.95, ends: 0.3, terraces, id: 3 };
            let label = if terraces { "back lanes built from 1815: terraces" } else { "back lanes built before 1815: separate houses" };
            vec![Cell::new(label, |m, site| scenes::street(m, site, &s, pal))]
        })
        .collect()
}

fn market(pal: &Palette) -> Vec<Vec<Cell>> {
    let cell = |big: bool, density: f32, label: &str| Cell::new(label, |m, site| scenes::market(m, site, &Market { big, density, id: 2 }, pal));
    vec![vec![cell(false, 0.5, "a small town, built up to 0.5"), cell(true, 1.0, "a big town, fully built up")]]
}

fn landmarks(pal: &Palette) -> Vec<Vec<Cell>> {
    vec![vec![
        Cell::new("church", |m, site| town::church(&mut m.solid, site, false, pal)),
        Cell::new("a big town's church", |m, site| town::church(&mut m.solid, site, true, pal)),
        Cell::new("station", |m, site| town::station(&mut m.solid, site, pal)),
        Cell::new("market cross", |m, site| town::market_cross(&mut m.solid, site, pal)),
        Cell::new("market hall", |m, site| town::market_hall(&mut m.solid, site, pal)),
        Cell::new("lighthouse", |m, site| town::lighthouse(&mut m.solid, site, pal)),
    ]]
}

// ------------------------------------------------------------------ works

/// The sizes a works is shown at: from a workshop to a great works.
const SIZES: [f32; 6] = [0.4, 1.0, 3.0, 10.0, 30.0, 100.0];

/// A works in a cell.
fn works_cell(label: String, w: Works, pal: &Palette) -> Cell {
    Cell::new(label, |m, site| works(&mut m.solid, &mut m.smoke, &mut m.flat, site, &w, pal))
}

/// One kind at every size: built early and by water, then built late and by rail.
fn works_sizes(kind: Kind, pal: &Palette) -> Vec<Vec<Cell>> {
    [(1760, true), (1870, false)]
        .into_iter()
        .map(|(founded, water)| {
            SIZES
                .iter()
                .enumerate()
                .map(|(i, &size)| {
                    let w = Works { founded, water, rail: !water, seed: Seed::of(kind as u32 * 16 + i as u32), ..Works::new(kind, size) };
                    works_cell(format!("{}, {founded}\nsize {size}", kind.label()), w, pal)
                })
                .collect()
        })
        .collect()
}

fn works_loom(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::HandLoom, pal)
}

fn works_smithy(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Smithy, pal)
}

fn works_millwright(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Millwright, pal)
}

fn works_shops(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Shops, pal)
}

fn works_charcoal(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::CharcoalBurning, pal)
}

fn works_bloomery(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Bloomery, pal)
}

fn works_charcoal_furnace(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::CharcoalFurnace, pal)
}

fn works_coke_furnace(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::CokeFurnace, pal)
}

fn works_forge(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::ForgeRails, pal)
}

fn works_rolling(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::RollingMill, pal)
}

fn works_rail_mill(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::SteelRolling, pal)
}

fn works_crucible(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::CrucibleSteel, pal)
}

fn works_converter(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Bessemer, pal)
}

fn works_engine(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::EngineWorks, pal)
}

fn works_mill(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::TextileMill, pal)
}

fn works_brick(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Brickworks, pal)
}

fn works_other(pal: &Palette) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Other, pal)
}

/// Every kind at one size and one founding year, in rows of `across`.
fn works_all(size: f32, founded: i32, across: usize, pal: &Palette) -> Vec<Vec<Cell>> {
    Kind::ALL
        .chunks(across)
        .enumerate()
        .map(|(row, kinds)| kinds.iter().enumerate().map(|(col, &kind)| works_cell(kind.label().into(), Works { founded, seed: Seed::of((row * across + col) as u32), ..Works::new(kind, size) }, pal)).collect())
        .collect()
}

fn works_kinds(pal: &Palette) -> Vec<Vec<Cell>> {
    works_all(3.0, 1820, 6, pal)
}

fn works_great(pal: &Palette) -> Vec<Vec<Cell>> {
    works_all(30.0, 1850, 3, pal)
}

/// Every kind at one size, built early and then built late, in rows of nine.
fn works_early_and_late(size: f32, pal: &Palette) -> Vec<Vec<Cell>> {
    let mut rows = Vec::new();
    for founded in [1760, 1870] {
        for (row, kinds) in Kind::ALL.chunks(9).enumerate() {
            rows.push(kinds.iter().enumerate().map(|(i, &kind)| works_cell(format!("{}\n{founded}", kind.label()), Works { founded, seed: Seed::of((row * 9 + i) as u32), ..Works::new(kind, size) }, pal)).collect());
        }
    }
    rows
}

fn works_small(pal: &Palette) -> Vec<Vec<Cell>> {
    works_early_and_late(0.4, pal)
}

fn works_standard(pal: &Palette) -> Vec<Vec<Cell>> {
    works_early_and_late(1.0, pal)
}

/// The states of a works down the rows, a kind in each column.
fn works_in_states(kinds: &[Kind], size: f32, pal: &Palette) -> Vec<Vec<Cell>> {
    State::ALL
        .iter()
        .map(|&state| kinds.iter().enumerate().map(|(i, &kind)| works_cell(format!("{}\n{}", kind.label(), state.label()), Works { state, founded: 1840, seed: Seed::of(i as u32), ..Works::new(kind, size) }, pal)).collect())
        .collect()
}

fn works_states(pal: &Palette) -> Vec<Vec<Cell>> {
    works_in_states(&[Kind::Smithy, Kind::CokeFurnace, Kind::TextileMill, Kind::RollingMill, Kind::Brickworks, Kind::CrucibleSteel], 3.0, pal)
}

fn works_states_great(pal: &Palette) -> Vec<Vec<Cell>> {
    works_in_states(&[Kind::TextileMill, Kind::CokeFurnace, Kind::EngineWorks], 30.0, pal)
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

fn bridges(pal: &Palette) -> Vec<Vec<Cell>> {
    let halves = [0.012, 0.025, 0.04, 0.06];
    grid(2, 4, |row, col| {
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
                    // (a great works is deeper: the lab stands back until the whole of it shows)
                    let room = if sheet.name.starts_with("works") { 1.0 } else { 0.8 };
                    assert!((hi.z - lo.z) + (hi.y - lo.y) * 0.8 <= room, "{}: {} is {} deep and {} tall", sheet.name, cell.label, hi.z - lo.z, hi.y - lo.y);
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
