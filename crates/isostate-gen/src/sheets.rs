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
use crate::houses::{cottage, terrace, townhouse, villa, Cottage, Detail, Era, Terrace, Townhouse, Villa, BACK_PLOT_WIDTH, PLOT};
use crate::palette::Palette;
use crate::scenes::{self, Cover, Market, Street};
use crate::site::{Model, Site};
use crate::town;
use crate::trees::{bush, hedge_tree, tree, Species, Tree};
use crate::works::site::{self, Edge, Now, Plot, Stage};
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

    /// A cell on ground that rises `fall` for each unit along the site's
    /// x, `width` wide and `depth` deep: the ground is built too, as a
    /// bank standing on the lab's level floor, so that the thing is seen
    /// sitting into it.
    pub fn on_slope(label: impl Into<String>, yaw: f32, fall: f32, width: f32, depth: f32, bank: crate::colour::Rgba, build: impl FnOnce(&mut Model, &Site)) -> Cell {
        let (hw, hd) = (width * 0.5, depth * 0.5);
        let ground = move |x: f32, _z: f32| (x + hw) * fall + 0.004;
        let mut model = Model::default();
        let top = |x: f32, z: f32| Vec3::new(x, ground(x, z), z);
        let b = &mut model.solid;
        b.quad(top(-hw, hd), top(hw, hd), top(hw, -hd), top(-hw, -hd), bank);
        b.quad(Vec3::new(-hw, 0.0, hd), Vec3::new(hw, 0.0, hd), top(hw, hd), top(-hw, hd), crate::colour::shade(bank, 0.86));
        b.quad(Vec3::new(hw, 0.0, -hd), Vec3::new(-hw, 0.0, -hd), top(-hw, -hd), top(hw, -hd), crate::colour::shade(bank, 0.8));
        b.quad(Vec3::new(hw, 0.0, hd), Vec3::new(hw, 0.0, -hd), top(hw, -hd), top(hw, hd), crate::colour::shade(bank, 0.9));
        build(&mut model, &Site::on(Vec3::new(0.0, ground(0.0, 0.0), 0.0), yaw, &ground));
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
    pub build: fn(&Palette, Detail) -> Vec<Vec<Cell>>,
}

/// Every sheet the lab knows. To add one, write its function and add a line here.
pub static SHEETS: &[Sheet] = &[
    Sheet { name: "cottages", about: "A house on its own plot, standing when the story opens and seen in 1800: one to three storeys down the rows, ten seeds across.", land: Land::Town, build: cottages },
    Sheet { name: "cottages-eras", about: "Cottages by when and where they were built: before 1720, to 1815, to 1860 and after in the lowland, then the hills early and late; eight seeds across, of one storey and of two by turns.", land: Land::Town, build: cottages_eras },
    Sheet { name: "townhouses", about: "A house in a high street's frontage, built about 1780: two, three and four storeys down the rows, ten seeds across.", land: Land::Town, build: townhouses },
    Sheet { name: "townhouses-eras", about: "Townhouses of three storeys by when and where they were built: before 1720, to 1815, to 1860 and after in the lowland, then the hills; eight seeds across, every other one a shop.", land: Land::Town, build: townhouses_eras },
    Sheet { name: "terraces", about: "A terraced row: two and three storeys built about 1840, then two storeys built after 1860 and in the hills; three to six houses across, each a seed of its own.", land: Land::Town, build: terraces },
    Sheet { name: "terraces-slope", about: "A terraced row on ground that falls along the street: nearly level, a gentle hill and a steep one; four and six houses.", land: Land::Town, build: terraces_slope },
    Sheet { name: "villas", about: "A villa in its garden: built before 1815, to 1860 and after, and in the hills; six seeds across.", land: Land::Town, build: villas },
    Sheet { name: "house-backs", about: "What is behind a house: cottages, townhouses, a villa and terraces seen from their yards and gardens.", land: Land::Town, build: house_backs },
    Sheet { name: "house-levels", about: "The three levels a house is drawn at, as the camera's distance chooses: far, play and near across; a cottage, a townhouse, a villa, a terrace and a farmstead down the rows.", land: Land::Town, build: house_levels },
    Sheet { name: "roofs", about: "What a cottage is roofed with by when and where it was built (thatch, tile, stone slate, blue slate), and how it has weathered by the year it is seen: twelve seeds across.", land: Land::Town, build: roofs },
    Sheet { name: "street", about: "A high street with its back lanes, as the towns lay them out, built up to 0.95 in the middle and 0.3 at the ends: as it stands in 1800, and in 1880. Each picture at the level the game draws from its distance.", land: Land::Town, build: street },
    Sheet { name: "street-play", about: "The same street with its houses at the play level in every picture, to see that level close up.", land: Land::Town, build: street_play },
    Sheet { name: "street-far", about: "The same street with its houses at the far level in every picture, to see that level close up.", land: Land::Town, build: street_far },
    Sheet { name: "market", about: "A market place with its church and the houses round it: a small town half built up in 1780, and a big town fully in 1880.", land: Land::Town, build: market },
    Sheet { name: "market-play", about: "The same market places at the play level in every picture, to see that level close up.", land: Land::Town, build: market_play },
    Sheet { name: "landmarks", about: "What marks a town out: its church in its churchyard (a small town's, with a spire, and a big town's), station, market cross, market hall and lighthouse.", land: Land::Town, build: landmarks },
    Sheet { name: "railway", about: "What stands by a railway: a town's station, the winding engine house at the head of an inclined plane (before 1850), the banking engine's shed at the foot of a steep bank (after), and the buffer stop at the end of a line; each beside a length of track.", land: Land::Pasture, build: railway },
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
    Sheet { name: "sites-kinds", about: "Every kind of works laid out on its plot (works::site): ten times the standard size in 1880, founded in 1790 and grown since, so each shows the stages of its growth, its stores and stock yards, and its way in and out by the gate.", land: Land::Pasture, build: sites_kinds },
    Sheet { name: "sites-small", about: "Every kind of works laid out on its plot at the standard size in 1800, founded in 1770: what most of a small town's works are.", land: Land::Pasture, build: sites_small },
    Sheet { name: "site-history", about: "One powered mill through its own history on one plot by a stream: going up in 1760, at work, enlarged in 1795, in 1820 and (the railway having come in 1848) in 1870; then the same works shrunk to a fifth for four years and for twelve, standing idle, and derelict.", land: Land::Pasture, build: site_history },
    Sheet { name: "site-history-iron", about: "One coke ironworks through its own history on one plot: going up in 1790, at work, enlarged in 1815, in 1840 and (the railway having come in 1848) in 1865; then shrunk for four years and for twelve, standing idle, and derelict.", land: Land::Pasture, build: site_history_iron },
    Sheet { name: "site-ways", about: "The ways in and out of a works (a rolling mill, then a brickworks, of ten times the standard size): by the gate alone; with a railway along its back; on navigable water; with both; with a works that feeds it on its left and one it feeds on its right; and how busy it is and what it holds, from slack and empty to hard at work and full.", land: Land::Pasture, build: site_ways },
    Sheet { name: "site-unknown", about: "Works of trades the generators have never heard of, each planned from its process alone (works::site, plain::process): a brewhouse, a glasshouse, a gasworks, a saw pit, a lime kiln, a potbank, a paper mill and a mason's yard, as the deeper production tree's overlay had them on 12 November 2026 (examples only: the game hands the planner the run's own processes). At the standard size in 1830 (top row) and at eight times it in 1885.", land: Land::Pasture, build: site_unknown },
    Sheet { name: "site-slope", about: "Works on sloping land: a coke ironworks, a powered mill and a brickworks, each of eight times the standard size, on land that falls away to the left by 6 in 100 (top row) and by 12 in 100, which is as steep as a works is given while ground is not short. The yard is level, at the height of the highest land under its plot, and is held up on its low side by a retaining wall.", land: Land::Pasture, build: site_slope },
    Sheet { name: "site-plots", about: "One works on plots of three shapes (square, with its back aslant, with a corner cut off), deep and shallow: a coke ironworks, a powered mill and a works of a kind the generators do not know, each of twelve times the standard size.", land: Land::Pasture, build: site_plots },
    Sheet { name: "works-district", about: "A district of works that stand together along a road with the railway behind: coke furnaces hand pig iron over the wall to a forge and the forge its bars to a rolling mill, with a brickworks, an engine works and a smithy beside them; as it stood in 1800, in 1850 and in 1890.", land: Land::Pasture, build: works_district },
    Sheet { name: "site-loom", about: "Handloom weavers laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_loom },
    Sheet { name: "site-smithy", about: "Smithies laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_smithy },
    Sheet { name: "site-millwright", about: "Millwrights laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_millwright },
    Sheet { name: "site-shops", about: "Shops and trades laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_shops },
    Sheet { name: "site-charcoal", about: "Charcoal burning laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_charcoal },
    Sheet { name: "site-bloomery", about: "Bloomeries laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_bloomery },
    Sheet { name: "site-charcoal-furnace", about: "Charcoal blast furnaces laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_charcoal_furnace },
    Sheet { name: "site-coke-furnace", about: "Coke blast furnaces laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_coke_furnace },
    Sheet { name: "site-forge", about: "Forges making wrought rails laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_forge },
    Sheet { name: "site-rolling", about: "Rolling mills laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_rolling },
    Sheet { name: "site-rail-mill", about: "Steel rail mills laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_rail_mill },
    Sheet { name: "site-crucible", about: "Crucible steel laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_crucible },
    Sheet { name: "site-converter", about: "Converter shops laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_converter },
    Sheet { name: "site-engine", about: "Engine works laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_engine },
    Sheet { name: "site-mill", about: "Powered mills laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_mill },
    Sheet { name: "site-brick", about: "Brickworks laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_brick },
    Sheet { name: "site-other", about: "A works of a kind the generators do not know laid out on their plots (works::site), as they stand in 1890: founded in 1760 on a square plot by a stream (top row), in 1820 on a plot with its back aslant along a railway, and in 1870 on a plot with a corner cut off; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Pasture, build: site_other },
    Sheet { name: "works-other", about: "A works of a kind the generators do not know: built in 1760 (top row) and in 1870; sizes 0.4, 1, 3, 10, 30 and 100 across.", land: Land::Town, build: works_other },
    Sheet { name: "pits", about: "Pits and their spoil heaps: coal, ironstone, stone and clay down the rows; a small, a middling and a large heap across.", land: Land::Heath, build: pits },
    Sheet { name: "farmsteads", about: "A farmstead seen from its yard: lowland and upland, arable and pasture down the rows; six seeds across.", land: Land::Pasture, build: farmsteads },
    Sheet { name: "trees", about: "Trees and bushes: broadleaf, conifer, hedgerow tree and heath bush down the rows; sizes across (a tree has no age yet). With --season 0 the broadleaves are bare; 0.8 is autumn.", land: Land::Pasture, build: trees },
    Sheet { name: "woods", about: "A patch of lowland wood and of upland wood, with a tree at every slot as the island has them. With --season 0 the broadleaves are bare; 0.8 is autumn.", land: Land::Wood, build: woods },
    Sheet { name: "heath", about: "Two patches of heath, fifteen slots each: gorse and broom at about one slot in six (and a tree at one in thirty: none here).", land: Land::Heath, build: heath },
    Sheet { name: "fences", about: "A family's fence round a field, in four colours a family might have (the banners themselves are the interface's): timber rails stained toward the colour, and post caps painted in it.", land: Land::Pasture, build: fences },
    Sheet { name: "bridges", about: "A road bridge: a lane's and a main road's down the rows; wider rivers across (water drawn flat and matt here).", land: Land::Pasture, build: bridges },
    Sheet { name: "scale", about: "One of every generator in the registry at each level of detail: far, play and near across. Under each: its triangles; its finest feature in thousandths of a unit; triangles to a bay of a storey of wall and to a bay by a bay of roof; its longest bare wall in bays.", land: Land::Pasture, build: scale },
];

/// The sheet of this name, if there is one.
pub fn find(name: &str) -> Option<&'static Sheet> {
    SHEETS.iter().find(|s| s.name == name)
}

fn storeys(n: usize) -> String {
    if n == 1 { "1 storey".into() } else { format!("{n} storeys") }
}

// ------------------------------------------------------------------ houses

fn cottages(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    grid(3, 10, |row, col| {
        // standing since before 1750 and seen in 1800: thatch and tile, part weathered; the oldest are framed
        let seed = Seed::of(col as u32);
        let built = roof::built_before(1750, seed);
        let c = Cottage { detail, era: Era::of(built), ..Cottage::new(PLOT.0, PLOT.1, row as u8 + 1, Roof::of(built, 1800, false, false, seed), seed) };
        Cell::new(format!("{}, {built}\nseed {col}", storeys(row + 1)), |m, site| cottage(&mut m.solid, &mut m.smoke, &mut m.green, site, &c, pal))
    })
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

/// The rows of a sheet of eras: each era in the lowland, then the hills early and late.
const ERA_ROWS: [(Era, bool, &str); 6] =
    [(Era::Old, false, "before 1720"), (Era::Georgian, false, "1720 to 1815"), (Era::Brick, false, "1815 to 1860"), (Era::Late, false, "after 1860"), (Era::Georgian, true, "hills, to 1815"), (Era::Late, true, "hills, after 1860")];

fn cottages_eras(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    grid(ERA_ROWS.len(), 8, |row, col| {
        let (era, upland, name) = ERA_ROWS[row];
        let seed = Seed::of(60 + (row * 8 + col) as u32);
        let n = 1 + (col % 2) as u8;
        let c = Cottage { detail, era, upland, ..Cottage::new(PLOT.0, PLOT.1, n, Roof::of(year_of(era), 1890, upland, era == Era::Late, seed), seed) };
        Cell::new(format!("{name}\n{}", storeys(n as usize)), |m, site| cottage(&mut m.solid, &mut m.smoke, &mut m.green, site, &c, pal))
    })
}

fn townhouses(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    grid(3, 10, |row, col| {
        // built in 1780 and seen in 1860
        let seed = Seed::of(col as u32);
        let t = Townhouse { detail, ..Townhouse::new(PLOT.0, PLOT.1, row as u8 + 2, Roof::of(1780, 1860, false, false, seed), seed) };
        Cell::new(format!("{}\nseed {col}", storeys(row + 2)), |m, site| townhouse(&mut m.solid, &mut m.smoke, site, &t, pal))
    })
}

fn townhouses_eras(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    grid(ERA_ROWS.len(), 8, |row, col| {
        let (era, upland, name) = ERA_ROWS[row];
        let seed = Seed::of(120 + (row * 8 + col) as u32);
        let shop = col % 2 == 1;
        let t = Townhouse { detail, era, upland, shop, ..Townhouse::new(PLOT.0, PLOT.1, 3, Roof::of(year_of(era), 1890, upland, era == Era::Late, seed), seed) };
        Cell::new(format!("{name}\n{}", if shop { "a shop" } else { "a house" }), |m, site| townhouse(&mut m.solid, &mut m.smoke, site, &t, pal))
    })
}

/// A terraced row of `houses` houses.
fn row_of(houses: usize, n: u8, era: Era, upland: bool, seed: Seed) -> Terrace {
    Terrace { era, upland, ..Terrace::new(houses, Terrace::length_of(houses, BACK_PLOT_WIDTH), PLOT.1, n, Roof::of(year_of(era), 1890, upland, era == Era::Late, seed), seed) }
}

fn terraces(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let rows = [(2, Era::Brick, false, "1840"), (3, Era::Brick, false, "1840"), (2, Era::Late, false, "1875"), (2, Era::Late, true, "the hills")];
    grid(4, 4, |row, col| {
        let (n, era, upland, name) = rows[row];
        let houses = col + 3;
        // a seed for each cell, so the sheet shows every brick and both roofs
        let seed = Seed::of((row * 4 + col) as u32);
        let t = Terrace { detail, ..row_of(houses, n, era, upland, seed) };
        Cell::new(format!("{houses} houses, {}\n{name}, seed {}", storeys(n as usize), row * 4 + col), |m, site| terrace(&mut m.solid, &mut m.smoke, site, &t, pal))
    })
}

fn terraces_slope(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let falls = [(0.01, "nearly level"), (0.06, "a gentle hill"), (0.14, "a steep hill")];
    grid(3, 2, |row, col| {
        let (fall, name) = falls[row];
        let houses = [4, 6][col];
        let seed = Seed::of(30 + (row * 2 + col) as u32);
        let t = Terrace { detail, ..row_of(houses, 2, Era::Brick, false, seed) };
        Cell::on_slope(format!("{houses} houses on {name}\nfalling {fall} to the unit"), 0.0, fall, t.length + 0.03, 0.09, *pal.ground.street_earth, |m, site| terrace(&mut m.solid, &mut m.smoke, site, &t, pal))
    })
}

fn villas(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let rows = [(Era::Georgian, false, "to 1815"), (Era::Brick, false, "1815 to 1860"), (Era::Late, false, "after 1860"), (Era::Late, true, "hills, after 1860")];
    grid(4, 6, |row, col| {
        let (era, upland, name) = rows[row];
        let seed = Seed::of(200 + (row * 6 + col) as u32);
        let v = Villa { detail, era, upland, ..Villa::new(PLOT.0, PLOT.1, Roof::of(year_of(era).max(1830), 1890, upland, true, seed), seed) };
        Cell::new(format!("{name}\nseed {}", 200 + row * 6 + col), |m, site| villa(&mut m.solid, &mut m.smoke, &mut m.green, site, &v, pal))
    })
}

/// A cottage, a townhouse, a villa, a terrace and a farmstead, each built into a model at a level of detail.
fn one_of_each(detail: Detail, n: u32, pal: &Palette) -> [(&'static str, Model); 5] {
    let seed = Seed::of(n);
    let level = Site::level(Vec3::ZERO, 0.0);
    let mut out = [("a cottage", Model::default()), ("a townhouse", Model::default()), ("a villa", Model::default()), ("a terrace", Model::default()), ("a farmstead", Model::default())];
    let c = Cottage { era: Era::Old, detail, ..Cottage::new(PLOT.0, PLOT.1, 2, Roof::of(1700, 1800, false, false, seed), seed) };
    cottage(&mut out[0].1.solid, &mut Default::default(), &mut out[0].1.green, &level, &c, pal);
    let t = Townhouse { shop: true, detail, ..Townhouse::new(PLOT.0, PLOT.1, 3, Roof::of(1780, 1860, false, false, seed), seed) };
    townhouse(&mut out[1].1.solid, &mut Default::default(), &level, &t, pal);
    let v = Villa { era: Era::Late, detail, ..Villa::new(PLOT.0, PLOT.1, Roof::of(1875, 1890, false, true, seed), seed) };
    villa(&mut out[2].1.solid, &mut Default::default(), &mut out[2].1.green, &level, &v, pal);
    let r = Terrace { detail, ..row_of(4, 2, Era::Brick, false, seed) };
    terrace(&mut out[3].1.solid, &mut Default::default(), &level, &r, pal);
    let f = Farmstead { arable: true, upland: false, detail, seed: Seed::new(0.4, 0.7) };
    farmstead(&mut out[4].1.solid, &mut Default::default(), &mut out[4].1.green, &Site::level(Vec3::ZERO, std::f32::consts::PI), &f, pal);
    out
}

fn house_levels(pal: &Palette, _detail: Detail) -> Vec<Vec<Cell>> {
    let levels: Vec<[(&'static str, Model); 5]> = Detail::ALL.iter().map(|&d| one_of_each(d, 5, pal)).collect();
    (0..5)
        .map(|kind| {
            Detail::ALL
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    let (name, model) = &levels[i][kind];
                    Cell { label: format!("{name}: {}\n{} triangles", d.label(), model.solid.triangles()), model: model.clone() }
                })
                .collect()
        })
        .collect()
}

fn house_backs(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let back = std::f32::consts::PI;
    let mut rows: Vec<Vec<Cell>> = Vec::new();
    rows.push(
        (0..6)
            .map(|col| {
                let seed = Seed::of(col);
                let built = roof::built_before(1750, seed);
                let c = Cottage { detail, era: Era::of(built), ..Cottage::new(PLOT.0, PLOT.1, 1 + (col % 2) as u8, Roof::of(built, 1800, false, false, seed), seed) };
                Cell::turned(format!("a cottage of {built}\nseed {col}"), back, |m, site| cottage(&mut m.solid, &mut m.smoke, &mut m.green, site, &c, pal))
            })
            .collect(),
    );
    rows.push(
        (0..6)
            .map(|col| {
                let seed = Seed::of(col);
                if col == 5 {
                    let v = Villa { detail, era: Era::Late, ..Villa::new(PLOT.0, PLOT.1, Roof::of(1875, 1890, false, true, seed), seed) };
                    return Cell::turned("a villa\nafter 1860", back, |m, site| villa(&mut m.solid, &mut m.smoke, &mut m.green, site, &v, pal));
                }
                let t = Townhouse { detail, neighbours: (true, true), ..Townhouse::new(PLOT.0, PLOT.1, 2 + (col % 2) as u8, Roof::of(1780, 1860, false, false, seed), seed) };
                Cell::turned(format!("a townhouse\nseed {col}"), back, |m, site| townhouse(&mut m.solid, &mut m.smoke, site, &t, pal))
            })
            .collect(),
    );
    rows.push(
        [(4, 2), (6, 2), (5, 3)]
            .into_iter()
            .enumerate()
            .map(|(i, (houses, n))| {
                let t = Terrace { detail, ..row_of(houses, n, Era::Brick, false, Seed::of(i as u32 + 3)) };
                Cell::turned(format!("a terrace of {houses}\n{}", storeys(n as usize)), back, |m, site| terrace(&mut m.solid, &mut m.smoke, site, &t, pal))
            })
            .collect(),
    );
    rows
}

/// What a house is roofed with by when and where it was built, and how the roof weathers.
fn roofs(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
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
                    let c = Cottage { detail, era: Era::of(built), upland, ..Cottage::new(PLOT.0, PLOT.1, 1, Roof::of(built, year, upland, by_rail, seed), seed) };
                    Cell::new(if col == 0 { label.to_string() } else { format!("seed {}", 40 + col) }, |m, site| cottage(&mut m.solid, &mut m.smoke, &mut m.green, site, &c, pal))
                })
                .collect()
        })
        .collect()
}

fn street_at(detail: Detail, pal: &Palette) -> Vec<Vec<Cell>> {
    [(1800, "in 1800: separate houses on the back lanes"), (1880, "in 1880: terraces on the back lanes, and slate")]
        .into_iter()
        .map(|(year, label)| {
            let s = Street { length: 1.0, middle: 0.95, ends: 0.3, year, id: 3, detail };
            vec![Cell::new(label, |m, site| scenes::street(m, site, &s, pal))]
        })
        .collect()
}

fn street(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    street_at(detail, pal)
}

fn street_play(pal: &Palette, _detail: Detail) -> Vec<Vec<Cell>> {
    street_at(Detail::Play, pal)
}

fn street_far(pal: &Palette, _detail: Detail) -> Vec<Vec<Cell>> {
    street_at(Detail::Far, pal)
}

fn market_at(detail: Detail, pal: &Palette) -> Vec<Vec<Cell>> {
    let cell = |big: bool, density: f32, year: i32, label: &str| Cell::new(label, |m, site| scenes::market(m, site, &Market { big, density, year, id: 2, detail }, pal));
    vec![vec![cell(false, 0.5, 1780, "a small town in 1780, built up to 0.5"), cell(true, 1.0, 1880, "a big town in 1880, fully built up")]]
}

fn market(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    market_at(detail, pal)
}

fn market_play(pal: &Palette, _detail: Detail) -> Vec<Vec<Cell>> {
    market_at(Detail::Play, pal)
}

fn landmarks(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    // the churches are seen from the south, the tower and its gate to the left
    let church = |big: bool, spire: bool, label: &str| {
        let c = town::Church { big, spire, seed: Seed::of(big as u32 + 1) };
        Cell::turned(label, -0.75, |m, site| town::church(&mut m.solid, site, &c, detail, pal))
    };
    vec![vec![
        church(false, false, "a small town's church"),
        church(false, true, "the same with a spire"),
        church(true, true, "a big town's church"),
        Cell::new("station", |m, site| town::station(&mut m.solid, site, detail, pal)),
        Cell::new("market cross", |m, site| town::market_cross(&mut m.solid, site, detail, pal)),
        Cell::new("market hall", |m, site| town::market_hall(&mut m.solid, site, detail, pal)),
        Cell::new("lighthouse", |m, site| town::lighthouse(&mut m.solid, site, detail, pal)),
    ]]
}

/// A length of track along a cell, `dz` in front of the middle of what stands there.
fn track(m: &mut Model, site: &Site, pal: &Palette, half: f32, dz: f32) {
    let (a, b) = (site.at(-half, dz) + Vec3::Y * 0.004, site.at(half, dz) + Vec3::Y * 0.004);
    m.flat.ribbon(&[a, b], 0.04, *pal.rail.ballast);
    m.flat.ribbon(&[a + Vec3::Y * 0.004, b + Vec3::Y * 0.004], 0.016, *pal.rail.rails);
}

fn railway(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    use crate::rail;
    vec![vec![
        Cell::new("station", |m, site| {
            town::station(&mut m.solid, site, detail, pal);
            track(m, site, pal, 0.1, 0.085);
        }),
        Cell::new("winding engine
at an incline's head", |m, site| {
            rail::winding_house(&mut m.solid, &mut m.smoke, site, detail, pal);
            track(m, site, pal, 0.1, rail::WINDING_HOUSE.1 * 0.5 + rail::TRACK_CLEAR);
        }),
        Cell::new("banking engine's shed
at a bank's foot", |m, site| {
            rail::banker_shed(&mut m.solid, site, detail, pal);
            track(m, site, pal, 0.1, rail::BANKER_SHED.1 * 0.5 + rail::TRACK_CLEAR);
        }),
        Cell::new("buffer stop", |m, site| {
            rail::buffer_stop(&mut m.solid, site, detail, pal);
            let (a, b) = (site.at(0.0, 0.0) + Vec3::Y * 0.004, site.at(-0.1, 0.0) + Vec3::Y * 0.004);
            m.flat.ribbon(&[b, a], 0.04, *pal.rail.ballast);
            m.flat.ribbon(&[b + Vec3::Y * 0.004, a + Vec3::Y * 0.004], 0.016, *pal.rail.rails);
        }),
    ]]
}

// ------------------------------------------------------------------ works

/// The sizes a works is shown at: from a workshop to a great works.
const SIZES: [f32; 6] = [0.4, 1.0, 3.0, 10.0, 30.0, 100.0];

/// A works in a cell.
fn works_cell(label: String, w: Works, detail: Detail, pal: &Palette) -> Cell {
    let w = Works { detail, ..w };
    Cell::new(label, |m, site| works(&mut m.solid, &mut m.smoke, &mut m.flat, site, &w, pal))
}

/// One kind at every size: built early and by water, then built late and by rail.
fn works_sizes(kind: Kind, pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    [(1760, true), (1870, false)]
        .into_iter()
        .map(|(founded, water)| {
            SIZES
                .iter()
                .enumerate()
                .map(|(i, &size)| {
                    let w = Works { founded, water, rail: !water, seed: Seed::of(kind as u32 * 16 + i as u32), ..Works::new(kind, size) };
                    works_cell(format!("{}, {founded}\nsize {size}", kind.label()), w, detail, pal)
                })
                .collect()
        })
        .collect()
}

fn works_loom(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::HandLoom, pal, detail)
}

fn works_smithy(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Smithy, pal, detail)
}

fn works_millwright(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Millwright, pal, detail)
}

fn works_shops(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Shops, pal, detail)
}

fn works_charcoal(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::CharcoalBurning, pal, detail)
}

fn works_bloomery(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Bloomery, pal, detail)
}

fn works_charcoal_furnace(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::CharcoalFurnace, pal, detail)
}

fn works_coke_furnace(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::CokeFurnace, pal, detail)
}

fn works_forge(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::ForgeRails, pal, detail)
}

fn works_rolling(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::RollingMill, pal, detail)
}

fn works_rail_mill(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::SteelRolling, pal, detail)
}

fn works_crucible(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::CrucibleSteel, pal, detail)
}

fn works_converter(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Bessemer, pal, detail)
}

fn works_engine(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::EngineWorks, pal, detail)
}

fn works_mill(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::TextileMill, pal, detail)
}

fn works_brick(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Brickworks, pal, detail)
}

fn works_other(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_sizes(Kind::Other, pal, detail)
}

/// Every kind at one size and one founding year, in rows of `across`.
fn works_all(size: f32, founded: i32, across: usize, pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    Kind::ALL
        .chunks(across)
        .enumerate()
        .map(|(row, kinds)| kinds.iter().enumerate().map(|(col, &kind)| works_cell(kind.label().into(), Works { founded, seed: Seed::of((row * across + col) as u32), ..Works::new(kind, size) }, detail, pal)).collect())
        .collect()
}

fn works_kinds(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_all(3.0, 1820, 6, pal, detail)
}

fn works_great(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_all(30.0, 1850, 3, pal, detail)
}

/// Every kind at one size, built early and then built late, in rows of nine.
fn works_early_and_late(size: f32, pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let mut rows = Vec::new();
    for founded in [1760, 1870] {
        for (row, kinds) in Kind::ALL.chunks(9).enumerate() {
            rows.push(kinds.iter().enumerate().map(|(i, &kind)| works_cell(format!("{}\n{founded}", kind.label()), Works { founded, seed: Seed::of((row * 9 + i) as u32), ..Works::new(kind, size) }, detail, pal)).collect());
        }
    }
    rows
}

fn works_small(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_early_and_late(0.4, pal, detail)
}

fn works_standard(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_early_and_late(1.0, pal, detail)
}

/// The states of a works down the rows, a kind in each column.
fn works_in_states(kinds: &[Kind], size: f32, pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    State::ALL
        .iter()
        .map(|&state| kinds.iter().enumerate().map(|(i, &kind)| works_cell(format!("{}\n{}", kind.label(), state.label()), Works { state, founded: 1840, seed: Seed::of(i as u32), ..Works::new(kind, size) }, detail, pal)).collect())
        .collect()
}

fn works_states(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_in_states(&[Kind::Smithy, Kind::CokeFurnace, Kind::TextileMill, Kind::RollingMill, Kind::Brickworks, Kind::CrucibleSteel], 3.0, pal, detail)
}

fn works_states_great(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    works_in_states(&[Kind::TextileMill, Kind::CokeFurnace, Kind::EngineWorks], 30.0, pal, detail)
}

// ------------------------------------------------------------------ works laid out on their plots

/// What a plot's sides face, drawn beside it so that a sheet shows why a
/// works is laid out as it is: the road before it, a railway, water.
fn beside(m: &mut Model, s: &Site, plot: &Plot, year: i32, pal: &Palette) {
    let n = plot.outline.len();
    let mid = plot.outline.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0 / n as f32, a.1 + p.1 / n as f32));
    for i in 0..n {
        let (a, b) = (plot.outline[i], plot.outline[(i + 1) % n]);
        let (dx, dz) = (b.0 - a.0, b.1 - a.1);
        let len = dx.hypot(dz).max(1e-5);
        let mut o = (dz / len, -dx / len);
        if (a.0 - mid.0) * o.0 + (a.1 - mid.1) * o.1 < 0.0 {
            o = (-o.0, -o.1);
        }
        let line = |off: f32, lift: f32| [s.at(a.0 + o.0 * off - dx / len * 0.02, a.1 + o.1 * off - dz / len * 0.02) + Vec3::Y * lift, s.at(b.0 + o.0 * off + dx / len * 0.02, b.1 + o.1 * off + dz / len * 0.02) + Vec3::Y * lift];
        match plot.edge(i, year) {
            Edge::Road => m.flat.ribbon(&line(0.016, 0.003), 0.026, *pal.ground.road_gravel),
            Edge::Water { navigable } => m.flat.ribbon(&line(if navigable { 0.04 } else { 0.014 }, 0.003), if navigable { 0.075 } else { 0.022 }, *pal.water.river),
            _ => {}
        }
    }
    // the railway where the plot says it runs, once it has come: a siding is led out to it
    if (0..n).any(|i| plot.edge(i, year) == Edge::Rail) && plot.line.len() >= 2 {
        let along = |lift: f32| -> Vec<Vec3> { plot.line.iter().map(|p| s.at(p.0, p.1) + Vec3::Y * lift).collect() };
        m.flat.ribbon(&along(0.003), 0.03, *pal.rail.ballast);
        m.flat.ribbon(&along(0.005), 0.012, *pal.rail.rails);
    }
}

/// A planned works in a cell, with what its plot's sides face drawn beside it.
fn site_cell(label: String, plan: &site::Plan, now: &Now, pal: &Palette) -> Cell {
    Cell::new(label, |m, s| {
        site::site(&mut m.solid, &mut m.smoke, &mut m.flat, s, plan, now, pal);
        beside(m, s, &plan.plot, now.year, pal);
    })
}

/// A kind of works founded and grown to a size by a year, on a plot.
fn site_of(label: String, kind: Kind, founded: i32, year: i32, size: f32, plot: Plot, seed: u32, detail: Detail, pal: &Palette) -> Cell {
    let (plan, now) = site::specimen(kind, founded, year, size, &plot, Seed::of(seed), pal);
    site_cell(label, &plan, &Now { detail, ..now }, pal)
}

fn sites_at(pal: &Palette, detail: Detail, founded: i32, year: i32, size: f32) -> Vec<Vec<Cell>> {
    Kind::ALL
        .chunks(6)
        .map(|row| {
            row.iter()
                .map(|&kind| {
                    let (w, d) = site::ground(kind, size);
                    site_of(format!("{}\nsize {size} in {year}", kind.label()), kind, founded, year, size, Plot::rect(w, d), kind as u32 + 3, detail, pal)
                })
                .collect()
        })
        .collect()
}

fn sites_kinds(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    sites_at(pal, detail, 1790, 1880, 10.0)
}

fn sites_small(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    sites_at(pal, detail, 1770, 1800, 1.0)
}

/// One kind at every size, as it stands in 1890: founded early on a square
/// plot by a stream, in 1820 on a plot with its back aslant along a railway,
/// and late on a plot with a corner cut off.
fn site_sizes(kind: Kind, pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    [1760, 1820, 1870]
        .into_iter()
        .map(|founded| {
            SIZES
                .iter()
                .enumerate()
                .map(|(i, &size)| {
                    let (w, d) = site::ground(kind, size);
                    let (plot, on) = match founded {
                        1760 => (Plot::rect(w, d).facing(1, Edge::Water { navigable: false }), "by a stream"),
                        1820 => (Plot::wedge(w * 1.15, d * 1.15, 0.25).facing(2, Edge::Rail), "by a railway"),
                        _ => (Plot::cut(w * 1.05, d * 1.05, 0.35), "a corner cut"),
                    };
                    site_of(format!("{}, {founded}, {on}\nsize {size}", kind.label()), kind, founded, 1890, size, plot, kind as u32 * 16 + i as u32, detail, pal)
                })
                .collect()
        })
        .collect()
}

/// A works through its own history: the stages it was built in, on one plot.
fn history_of(kind: Kind, stages: &[Stage], plot: Plot, pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let process = site::Process::of(kind);
    let seed = Seed::of(kind as u32 + 40);
    let upto = |year: i32| -> Vec<Stage> { stages.iter().copied().filter(|s| s.year <= year).collect() };
    let cell = |label: &str, year: i32, change: &dyn Fn(Now) -> Now| {
        let plan = site::plan(kind, &process, &upto(year), &plot, year, seed, pal);
        let now = change(Now { detail, ..Now::working(&plan, year) });
        site_cell(format!("{label}\n{year}"), &plan, &now, pal)
    };
    let first = stages[0].year;
    let last = stages.last().unwrap();
    let mut growing = vec![cell("going up", first, &|n| Now { state: State::GoingUp, ..n }), cell("at work", first + 12, &|n| n)];
    for st in &stages[1..] {
        growing.push(cell("being enlarged", st.year, &|n| n));
    }
    growing.push(cell("at work", last.year + 15, &|n| n));
    let end = last.year + 25;
    let low = stages[stages.len().saturating_sub(3)].size;
    let ending = vec![
        cell("shrunk these four years", end, &|n| Now { size: low, shrunk_for: 4, busy: 0.4, stock_in: 0.3, stock_out: 0.3, ..n }),
        cell("shrunk these twelve", end + 8, &|n| Now { size: low, shrunk_for: 12, busy: 0.4, stock_in: 0.3, stock_out: 0.2, ..n }),
        cell("standing idle", end + 9, &|n| Now { state: State::Idle, busy: 0.0, ..n }),
        cell("derelict", end + 20, &|n| Now { state: State::Derelict, busy: 0.0, ..n }),
    ];
    vec![growing, ending]
}

fn site_history(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let (w, d) = site::ground(Kind::TextileMill, 24.0);
    let plot = Plot::rect(w, d).facing(1, Edge::Water { navigable: false }).facing_from(2, Edge::Rail, 1848);
    history_of(Kind::TextileMill, &[Stage { year: 1760, size: 0.6 }, Stage { year: 1795, size: 2.0 }, Stage { year: 1820, size: 7.0 }, Stage { year: 1870, size: 24.0 }], plot, pal, detail)
}

fn site_history_iron(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let (w, d) = site::ground(Kind::CokeFurnace, 20.0);
    let plot = Plot::wedge(w, d, 0.25).facing_from(2, Edge::Rail, 1848);
    history_of(Kind::CokeFurnace, &[Stage { year: 1790, size: 0.8 }, Stage { year: 1815, size: 2.4 }, Stage { year: 1840, size: 7.0 }, Stage { year: 1865, size: 20.0 }], plot, pal, detail)
}

fn site_ways(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    [Kind::RollingMill, Kind::Brickworks]
        .into_iter()
        .map(|kind| {
            let (w, d) = site::ground(kind, 10.0);
            let plot = || Plot::rect(w, d);
            let nav = Edge::Water { navigable: true };
            let cell = |label: &str, plot: Plot, change: &dyn Fn(Now) -> Now| {
                let (plan, now) = site::specimen(kind, 1800, 1885, 10.0, &plot, Seed::of(kind as u32 + 9), pal);
                site_cell(format!("{}\n{label}", kind.label()), &plan, &change(Now { detail, ..now }), pal)
            };
            vec![
                cell("by the gate", plot(), &|n| n),
                cell("a railway behind", plot().facing(2, Edge::Rail), &|n| n),
                cell("on navigable water", plot().facing(3, nav), &|n| n),
                cell("railway and water", plot().facing(2, Edge::Rail).facing(1, nav), &|n| n),
                cell("fed on its left, feeds on its right", plot().facing(3, Edge::Fed).facing(1, Edge::Feeds), &|n| n),
                cell("slack and empty", plot().facing(2, Edge::Rail), &|n| Now { busy: 0.15, stock_in: 0.1, stock_out: 0.1, ..n }),
                cell("hard at work and full", plot().facing(2, Edge::Rail), &|n| Now { busy: 1.0, stock_in: 1.0, stock_out: 1.0, ..n }),
            ]
        })
        .collect()
}

fn site_plots(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    [Kind::CokeFurnace, Kind::TextileMill, Kind::Other]
        .into_iter()
        .map(|kind| {
            let (w, d) = site::ground(kind, 12.0);
            let shapes: [(&str, Plot); 6] = [
                ("square", Plot::rect(w, d)),
                ("its back aslant", Plot::wedge(w * 1.1, d * 1.1, 0.4)),
                ("a corner cut off", Plot::cut(w * 1.08, d * 1.08, 0.45)),
                ("wide and shallow", Plot::rect(w * 1.5, d / 1.5)),
                ("narrow and deep", Plot::rect(w / 1.45, d * 1.45)),
                ("shallow, its back aslant", Plot::wedge(w * 1.55, d / 1.3, 0.45)),
            ];
            shapes.into_iter().enumerate().map(|(i, (name, plot))| site_of(format!("{}\n{name}", kind.label()), kind, 1795, 1885, 12.0, plot, kind as u32 * 8 + i as u32, detail, pal)).collect()
        })
        .collect()
}

/// Eight processes that are not in the shipped content, for showing what the
/// planner makes of a trade nobody has drawn: (name, goods in with weight,
/// price and amount, heat, labour, cost of plant, good out with weight and price).
pub(crate) type Unknown = (&'static str, &'static [(&'static str, f32, f32, f32)], f32, f32, f32, (&'static str, f32, f32));
pub(crate) const UNKNOWN: [Unknown; 8] = [
    ("brewhouse", &[("grain", 1.0, 1.0, 1.2)], 0.4, 0.1, 6.6, ("beer", 2.0, 2.0)),
    ("glasshouse", &[("stone", 2.2, 0.6, 1.0), ("chemicals", 0.9, 4.5, 0.3)], 6.0, 1.2, 39.6, ("glass", 1.0, 9.0)),
    ("gasworks", &[("coal", 1.6, 0.9, 1.2), ("lime", 1.5, 2.0, 0.04)], 0.0, 0.06, 32.2, ("gas", 20.0, 2.4)),
    ("saw pit", &[("timber", 1.6, 0.8, 1.4)], 0.0, 0.28, 2.0, ("lumber", 1.2, 2.2)),
    ("lime kiln", &[("stone", 2.2, 0.6, 1.6)], 1.3, 0.08, 5.1, ("lime", 1.5, 2.0)),
    ("potbank", &[("clay", 2.0, 0.4, 1.3)], 2.5, 0.5, 14.0, ("pottery", 1.0, 3.4)),
    ("paper mill", &[("cloth", 0.3, 5.0, 0.2)], 0.5, 1.0, 12.8, ("paper", 0.5, 4.6)),
    ("masons", &[("stone", 2.2, 0.6, 1.1), ("lumber", 1.2, 2.2, 0.2), ("lime", 1.5, 2.0, 0.1)], 0.0, 0.25, 2.8, ("building", 12.0, 2.2)),
];

fn site_unknown(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    [(1.0f32, 1790, 1830), (8.0, 1800, 1885)]
        .into_iter()
        .map(|(size, founded, year)| {
            UNKNOWN
                .iter()
                .enumerate()
                .map(|(n, &(name, inputs, heat, labour, capital, out))| {
                    let process = site::Process { inputs: inputs.iter().map(|&(g, w, p, a)| (site::Good::new(g, w, p), a)).collect(), heat, labour, capital, output: site::Good::new(out.0, out.1, out.2) };
                    let (w, d) = site::ground(Kind::Other, size);
                    let record = site::history(founded, year, size);
                    let plan = site::plan(Kind::Other, &process, &site::stages(&record), &Plot::rect(w, d), year, Seed::of(n as u32 + 90), pal);
                    let now = Now { size, detail, ..Now::working(&plan, year) };
                    site_cell(format!("{name}
size {size} in {year}"), &plan, &now, pal)
                })
                .collect()
        })
        .collect()
}

fn site_slope(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    [0.06f32, 0.12]
        .into_iter()
        .map(|fall| {
            [Kind::CokeFurnace, Kind::TextileMill, Kind::Brickworks]
                .into_iter()
                .map(|kind| {
                    let (w, d) = site::ground(kind, 8.0);
                    let (plan, now) = site::specimen(kind, 1800, 1885, 8.0, &Plot::rect(w, d), Seed::of(kind as u32 + 21), pal);
                    let now = Now { detail, ..now };
                    Cell::on_slope(format!("{}
falling {fall} to the unit", kind.label()), 0.0, fall, w + 0.1, d + 0.1, *pal.ground.furnace_bank, |m, s| {
                        // (the yard at the height of the highest land under the plot, as the game hands it)
                        let top = s.height(s.origin.x + w * 0.5, s.origin.z);
                        site::site(&mut m.solid, &mut m.smoke, &mut m.flat, &s.moved(Vec3::new(s.origin.x, top, s.origin.z), s.yaw), &plan, &now, pal);
                    })
                })
                .collect()
        })
        .collect()
}

/// A district of works along a road, the railway (from 1845) behind them:
/// each plot's sides say which neighbour hands it material or takes its output.
fn district(year: i32, detail: Detail, pal: &Palette) -> Cell {
    // (kind, founded, size in 1890): the chain of iron first, in the order material passes along it
    let works: [(Kind, i32, f32); 6] = [(Kind::CokeFurnace, 1785, 8.0), (Kind::ForgeRails, 1805, 3.5), (Kind::RollingMill, 1830, 5.0), (Kind::Brickworks, 1775, 2.5), (Kind::EngineWorks, 1840, 3.5), (Kind::Smithy, 1765, 1.5)];
    let depth = works.iter().map(|&(kind, _, size)| site::ground(kind, size).1).fold(0.0, f32::max);
    let mut placed = Vec::new();
    let mut x = 0.0;
    for (n, &(kind, founded, size)) in works.iter().enumerate() {
        let (w, d) = site::ground(kind, size);
        // every plot is as deep as the deepest, and as much narrower as keeps its ground
        let w = w * d / depth;
        placed.push((n, kind, founded, size, x + w * 0.5, w));
        x += w + if n == 2 { 0.02 } else { 0.0 };
    }
    let total = x;
    Cell::new(format!("a works district\n{year}"), |m, s| {
        for &(n, kind, founded, size, cx, w) in &placed {
            if founded > year {
                continue;
            }
            // (the district's one line, as it runs by this plot: every siding is led out to it)
            let line = vec![(-cx - 0.01, -depth * 0.5 - site::LINE_OFF), (total - cx + 0.01, -depth * 0.5 - site::LINE_OFF)];
            let mut plot = Plot::rect(w, depth).with_line(line).facing_from(2, Edge::Rail, 1845);
            // the chain: the furnaces feed the forge on their right, the forge the rolling mill
            if n < 2 {
                plot = plot.facing_from(1, Edge::Feeds, works[n + 1].1);
            }
            if n == 1 || n == 2 {
                plot = plot.facing_from(3, Edge::Fed, founded);
            }
            let record: Vec<(i32, f32)> = site::history(founded, 1890, size).into_iter().filter(|h| h.0 <= year).collect();
            let plan = site::plan(kind, &site::Process::of(kind), &site::stages(&record), &plot, year, Seed::of(n as u32 + 70), pal);
            let now = Now { size: record.last().unwrap().1, state: if year == founded { State::GoingUp } else { State::Working }, detail, ..Now::working(&plan, year) };
            let at = s.moved(s.at(cx - total * 0.5, 0.0), s.yaw);
            site::site(&mut m.solid, &mut m.smoke, &mut m.flat, &at, &plan, &now, pal);
        }
        let (a, b) = (s.at(-total * 0.5 - 0.01, depth * 0.5 + 0.018) + Vec3::Y * 0.003, s.at(total * 0.5 + 0.01, depth * 0.5 + 0.018) + Vec3::Y * 0.003);
        m.flat.ribbon(&[a, b], 0.028, *pal.ground.road_gravel);
        if year >= 1845 {
            let (a, b) = (s.at(-total * 0.5 - 0.01, -depth * 0.5 - site::LINE_OFF) + Vec3::Y * 0.003, s.at(total * 0.5 + 0.01, -depth * 0.5 - site::LINE_OFF) + Vec3::Y * 0.003);
            m.flat.ribbon(&[a, b], 0.03, *pal.rail.ballast);
            m.flat.ribbon(&[a + Vec3::Y * 0.002, b + Vec3::Y * 0.002], 0.012, *pal.rail.rails);
        }
    })
}

fn works_district(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    vec![vec![district(1800, detail, pal)], vec![district(1850, detail, pal)], vec![district(1890, detail, pal)]]
}

fn site_loom(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::HandLoom, pal, detail)
}

fn site_smithy(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::Smithy, pal, detail)
}

fn site_millwright(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::Millwright, pal, detail)
}

fn site_shops(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::Shops, pal, detail)
}

fn site_charcoal(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::CharcoalBurning, pal, detail)
}

fn site_bloomery(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::Bloomery, pal, detail)
}

fn site_charcoal_furnace(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::CharcoalFurnace, pal, detail)
}

fn site_coke_furnace(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::CokeFurnace, pal, detail)
}

fn site_forge(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::ForgeRails, pal, detail)
}

fn site_rolling(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::RollingMill, pal, detail)
}

fn site_rail_mill(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::SteelRolling, pal, detail)
}

fn site_crucible(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::CrucibleSteel, pal, detail)
}

fn site_converter(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::Bessemer, pal, detail)
}

fn site_engine(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::EngineWorks, pal, detail)
}

fn site_mill(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::TextileMill, pal, detail)
}

fn site_brick(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::Brickworks, pal, detail)
}

fn site_other(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    site_sizes(Kind::Other, pal, detail)
}

fn pits(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let kinds = [(PitKind::Coal, "colliery"), (PitKind::Ironstone, "iron mine"), (PitKind::Stone, "quarry"), (PitKind::Clay, "clay pit")];
    grid(4, 3, |row, col| {
        let (kind, name) = kinds[row];
        let v = col as f32 * 0.5;
        Cell::new(format!("{name}\nheap {v}"), |m, site| pit(&mut m.solid, site, &Pit { kind, v, detail }, pal))
    })
}

// ------------------------------------------------------------------ the country

fn farmsteads(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let kinds = [(false, true, "lowland arable"), (false, false, "lowland pasture"), (true, true, "upland arable"), (true, false, "upland pasture")];
    grid(4, 6, |row, col| {
        let (upland, arable, name) = kinds[row];
        let f = Farmstead { arable, upland, detail, seed: Seed::of(col as u32) };
        // turned about, so the yard and the barn are on the camera's side of the house
        Cell::turned(format!("{name}\nseed {col}"), std::f32::consts::PI, |m, site| farmstead(&mut m.solid, &mut m.smoke, &mut m.green, site, &f, pal))
    })
}

fn trees(pal: &Palette, _detail: Detail) -> Vec<Vec<Cell>> {
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

fn woods(pal: &Palette, _detail: Detail) -> Vec<Vec<Cell>> {
    [(Cover::LowlandWood, "a lowland wood"), (Cover::UplandWood, "an upland wood")]
        .into_iter()
        .enumerate()
        .map(|(i, (what, name))| vec![Cell::new(name, |m, site| scenes::cover(m, site, what, i as i32, pal))])
        .collect()
}

fn heath(pal: &Palette, _detail: Detail) -> Vec<Vec<Cell>> {
    // fifteen slots to a patch: two or three bushes, and a tree on one patch in two (neither of these)
    vec![[2, 7].into_iter().map(|id| Cell::new(format!("heath, patch {id}"), |m, site| scenes::cover(m, site, Cover::Heath, id, pal))).collect()]
}

fn fences(pal: &Palette, _detail: Detail) -> Vec<Vec<Cell>> {
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

/// One of every generator at each level, with what it measures against the scale.
fn scale(pal: &Palette, _detail: Detail) -> Vec<Vec<Cell>> {
    use crate::registry::{Class, REGISTRY};
    REGISTRY
        .iter()
        .map(|e| {
            Detail::ALL
                .iter()
                .map(|&d| {
                    let s = (e.build)(d, pal).swap_remove(0);
                    // what is built of it, or for a plant what is grown
                    let mut b = s.model.solid.clone();
                    if e.class == Class::Plant {
                        for g in [&s.model.green.evergreen, &s.model.green.leaves, &s.model.green.bare] {
                            b.merge(g);
                        }
                    }
                    let m = crate::scale::measure(&b, &s.tally, pal);
                    Cell { label: format!("{}, {}: {} triangles
{}", e.name, d.label(), m.triangles, crate::scale::words(&m)), model: s.model }
                })
                .collect()
        })
        .collect()
}

fn bridges(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let halves = [0.012, 0.025, 0.04, 0.06, 0.1];
    grid(2, 5, |row, col| {
        let (main, half) = (row == 1, halves[col]);
        let b = RoadBridge { at: Vec2::ZERO, along: Vec2::X, half, level: 0.002, main, detail };
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
            let rows = (sheet.build)(&pal, Detail::Near);
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
    fn every_face_that_can_be_seen_has_a_facing() {
        // a quad takes its facing from its first corner; begun at a corner where two of its
        // points meet, it has none, and is drawn unlit: a dark wedge on a pale thing
        let pal = Palette::standard();
        for sheet in SHEETS {
            for cell in (sheet.build)(&pal, Detail::Near).into_iter().flatten() {
                let m = &cell.model;
                for b in [&m.solid, &m.smoke, &m.green.evergreen, &m.green.leaves, &m.green.bare, &m.flat] {
                    for t in b.idx.chunks(3) {
                        let p = [0, 1, 2].map(|i| glam::Vec3::from_array(b.pos[t[i] as usize]));
                        let area = (p[1] - p[0]).cross(p[2] - p[0]).length();
                        let facing = glam::Vec3::from_array(b.nrm[t[0] as usize]).length();
                        assert!(area < 1e-9 || facing > 0.5, "{}: {} has a face of {area} with no facing at {}", sheet.name, cell.label, p[0]);
                    }
                }
            }
        }
    }

    #[test]
    fn every_corner_of_what_is_built_carries_a_mark_for_winter() {
        // (winter, task D) the alpha of a corner in `solid` and `flat` says what winter does
        // there, and comes with the palette's token: a generator that mixed two colours by
        // hand, or wrote an alpha of its own, would have something between the marks, and
        // the viewer would frost a roof or freeze a wall
        use crate::colour::settles::{FROST, ICE, NOTHING, SNOW};
        let pal = Palette::standard();
        // (at every level a building is drawn at, since each level builds its own corners)
        for sheet in SHEETS {
            for detail in Detail::ALL {
                for cell in (sheet.build)(&pal, detail).into_iter().flatten() {
                    for (name, b) in [("solid", &cell.model.solid), ("flat", &cell.model.flat)] {
                        if let Some(c) = b.col.iter().find(|c| ![SNOW, FROST, ICE, NOTHING].contains(&c[3])) {
                            panic!("{}: {} at {detail:?} has a corner in `{name}` whose alpha is {}, which is no mark for winter", sheet.name, cell.label, c[3]);
                        }
                    }
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
            for row in (sheet.build)(&pal, Detail::Near) {
                for cell in row {
                    let (lo, hi) = cell.model.bounds().unwrap();
                    assert!(hi.x - lo.x <= 1.15, "{}: {} is {} wide", sheet.name, cell.label, hi.x - lo.x);
                    // (a great works is deeper: the lab stands back until the whole of it shows)
                    let room = if sheet.name.starts_with("works") || sheet.name.starts_with("site") { 1.0 } else { 0.8 };
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

    /// Prints a fingerprint of every cell of every sheet: run it before and after a change
    /// that should leave some sheets as they were, and compare the lines
    /// (`cargo test -p isostate-gen fingerprints -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn fingerprints() {
        let pal = Palette::standard();
        for sheet in SHEETS {
            let mut h = 0xcbf2_9ce4_8422_2325u64;
            let mut eat = |v: u32| {
                for byte in v.to_le_bytes() {
                    h = (h ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
                }
            };
            let mut tris = 0;
            for cell in (sheet.build)(&pal, Detail::Near).into_iter().flatten() {
                let m = &cell.model;
                for b in [&m.solid, &m.smoke, &m.green.evergreen, &m.green.leaves, &m.green.bare, &m.flat] {
                    tris += b.triangles();
                    b.pos.iter().chain(&b.nrm).flatten().for_each(|v| eat(v.to_bits()));
                    b.col.iter().flatten().for_each(|v| eat(v.to_bits()));
                    b.idx.iter().for_each(|&i| eat(i));
                }
            }
            println!("fingerprint {:<24} {h:016x} {tris} triangles", sheet.name);
        }
    }

    #[test]
    fn a_sheet_is_the_same_each_time_it_is_built() {
        let pal = Palette::standard();
        for sheet in SHEETS {
            let (a, b) = ((sheet.build)(&pal, Detail::Near), (sheet.build)(&pal, Detail::Near));
            assert!(a.iter().flatten().zip(b.iter().flatten()).all(|(x, y)| x.model == y.model && x.label == y.label), "{}", sheet.name);
        }
    }
}
