//! A year's scenery: the one table of the meshes a year hands to the card.
//!
//! A year's scenery is built off the main thread and handed to the card a
//! mesh at a time (`handover.rs`). What those meshes are was written out by
//! hand in some eleven places (the architecture review's finding 2 and change
//! B2, `docs/designs/architecture.md`): the struct that carried them, a list
//! of their names, the places of each in that list, the handles kept for
//! them, the entities and materials made for them at the start, what a build
//! for the camera alone sends again, the order of the hand-over, the names in
//! a picture's caption, the fingerprints and the bench. Here they are one
//! table, `MESHES`, a row to each kind of mesh, and every one of those places
//! reads it. A new mesh is a row here and a maker (`Make`).
//!
//! Four orders of the same meshes are kept, and three of them can be seen, so
//! the table says each:
//!
//! - the table's own order, which is the order of the list everything else
//!   indexes (`list`), of `ISOSTATE_MESH_SUMS`'s line and of the bench's count;
//! - `made`, the order the meshes' handles are made and their entities spawned
//!   at the start. The meshes' asset numbers follow it, and two meshes of one
//!   material are drawn in that order, which decides the pixels where they lie
//!   at one height (a lane where it meets a road);
//! - `goes`, the order of the hand-over, which was hard won (`hand_over`).

// (until the places that list the meshes read the table, the next commits)
#![allow(dead_code)]

use isostate_gen::trees::{in_leaf, LEAF_GROUPS};

use crate::look::Look;
use crate::meshkit::Builder;
use crate::replay::Replay;
use crate::world::{Ground, YearLand};
use crate::{network, scatter};

/// One kind of mesh a year's scenery hands to the card.
pub struct YearMesh {
    /// what it is called in a scripted check (`scenery`) and in the caption of
    /// a picture of a year half arrived; a mesh to each group of trees is
    /// called by its word, its group and this ("trees 2 leaves")
    pub name: &'static str,
    /// the word it is counted under in `ISOSTATE_MESH_SUMS`'s line and the
    /// bench's count. Meshes that share a word are fingerprinted as one, in
    /// the order of the list.
    pub word: &'static str,
    /// one mesh to each group of trees (`LEAF_GROUPS`), or one
    pub each_group: bool,
    /// who makes it, and from what
    pub make: Make,
    /// what it is drawn in, and what it carries beside
    pub drawn: Drawn,
    /// whether a year built again for the camera alone (`scatter::Lod`)
    /// sends it to the card again; what it does not send stays as it is
    pub camera: bool,
    /// its place in the hand-over
    pub goes: Goes,
    /// its place among the year's meshes when their handles are made and
    /// their entities spawned at the start (`setup_world`). Meshes with the
    /// same number are made in the order of the list.
    pub made: u8,
}

/// Who makes a year's mesh. The year's makers each make several meshes at
/// once, in one pass over the island, and a row takes its own from what its
/// maker made; or it has a maker of its own.
#[derive(Clone, Copy)]
pub enum Make {
    /// one of what the year's scattered things are built into
    /// (`scatter::build_year_on`): the trees, the buildings, their smoke and
    /// what lies flat round them. Given the group, for a mesh to each group.
    Scatter(fn(&mut scatter::YearMeshes, usize) -> Builder),
    /// one of the year's lines (`network::year_nets`): the roads and the railway
    Lines(fn(&mut network::Nets) -> Builder),
    /// a maker of its own, from what the year is built from (and the group,
    /// for a mesh to each group). None of today's meshes has one; the test's
    /// does, and so will the next mesh that is not made in the same pass as
    /// another.
    #[allow(dead_code)]
    Own(fn(&Inputs, usize) -> Builder),
}

/// What a year's scenery is built from: everything its makers read.
pub struct Inputs<'a> {
    pub r: &'a Replay,
    pub g: &'a Ground,
    pub slots: &'a scatter::Slots,
    pub cen: &'a [(f32, f32)],
    /// the year, counted from the run's first
    pub k: usize,
    pub lk: &'a Look,
    /// the camera the buildings are built for
    pub lod: &'a scatter::Lod,
    /// the year's land, worked out once for the ground's textures and the scenery
    pub land: &'a YearLand,
}

/// What a year's mesh is drawn in, and what its entity carries beside.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Drawn {
    /// the material winter lies on, casting shadows: the houses and the works
    /// (churches, farm buildings and bridges among them)
    Built,
    /// the same material for what lies flat on the ground, casting no shadow:
    /// the works' yards and ponds, the farm lanes, the roads, the railway's track
    Flat,
    /// the smoke's own blended material, casting no shadow
    Smoke,
    /// what is green all year of a group of trees: always shown (`Evergreen`)
    Evergreen,
    /// a group's leaves, in a material that turns with the seasons, shown
    /// while the group is in leaf (`Leaves`)
    Leaves,
    /// a group's bare limbs, shown while it is not (`BareLimbs`)
    Bare,
}

impl Drawn {
    /// Whether a mesh drawn so shows at `phase` of the year, in group `group`.
    pub fn shows(self, phase: f32, group: usize) -> bool {
        match self {
            Drawn::Leaves => in_leaf(phase, group),
            Drawn::Bare => !in_leaf(phase, group),
            _ => true,
        }
    }
}

/// A mesh's place in the hand-over of a year.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Goes {
    /// at this place among those that go before the trees
    Before(u8),
    /// with the trees, group by group, while the season shows it; after
    /// every group while the season hides it
    WithTrees,
}

/// The meshes of a year's scenery. The trees come first, because the
/// fingerprints' line has always begun with them.
pub const MESHES: &[YearMesh] = &[
    YearMesh { name: "evergreen", word: "trees", each_group: true, make: Make::Scatter(|m, g| std::mem::take(&mut m.trees[g].evergreen)), drawn: Drawn::Evergreen, camera: false, goes: Goes::WithTrees, made: 0 },
    YearMesh { name: "leaves", word: "trees", each_group: true, make: Make::Scatter(|m, g| std::mem::take(&mut m.trees[g].leaves)), drawn: Drawn::Leaves, camera: false, goes: Goes::WithTrees, made: 0 },
    YearMesh { name: "bare limbs", word: "trees", each_group: true, make: Make::Scatter(|m, g| std::mem::take(&mut m.trees[g].bare)), drawn: Drawn::Bare, camera: false, goes: Goes::WithTrees, made: 0 },
    YearMesh { name: "houses", word: "houses", each_group: false, make: Make::Scatter(|m, _| std::mem::take(&mut m.houses)), drawn: Drawn::Built, camera: true, goes: Goes::Before(0), made: 1 },
    YearMesh { name: "works", word: "works", each_group: false, make: Make::Scatter(|m, _| std::mem::take(&mut m.works)), drawn: Drawn::Built, camera: true, goes: Goes::Before(1), made: 2 },
    YearMesh { name: "smoke", word: "smoke", each_group: false, make: Make::Scatter(|m, _| std::mem::take(&mut m.smoke)), drawn: Drawn::Smoke, camera: true, goes: Goes::Before(3), made: 3 },
    YearMesh { name: "yards and lanes", word: "lanes", each_group: false, make: Make::Scatter(|m, _| std::mem::take(&mut m.lanes)), drawn: Drawn::Flat, camera: false, goes: Goes::Before(2), made: 6 },
    YearMesh { name: "roads", word: "roads", each_group: false, make: Make::Lines(|n| std::mem::take(&mut n.roads)), drawn: Drawn::Flat, camera: false, goes: Goes::Before(4), made: 4 },
    YearMesh { name: "railway", word: "rails", each_group: false, make: Make::Lines(|n| std::mem::take(&mut n.rails)), drawn: Drawn::Flat, camera: false, goes: Goes::Before(5), made: 5 },
];

/// One mesh of a year: the row of the table it is of, and its group of trees
/// (0 for a mesh that is one of a kind).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub row: usize,
    pub group: usize,
}

/// A year's meshes in the one order everything else indexes: the table's,
/// where a run of rows that are each to a group of trees goes group by group
/// (the three of group 0, then the three of group 1, and so on).
pub fn list(table: &[YearMesh]) -> Vec<Part> {
    let mut out = Vec::new();
    let mut row = 0;
    while row < table.len() {
        let end = if table[row].each_group { (row..table.len()).find(|&i| !table[i].each_group).unwrap_or(table.len()) } else { row + 1 };
        let groups = if table[row].each_group { LEAF_GROUPS } else { 1 };
        for group in 0..groups {
            out.extend((row..end).map(|row| Part { row, group }));
        }
        row = end;
    }
    out
}

/// A mesh's name, as the scripted checks and a picture's caption say it.
pub fn name(table: &[YearMesh], p: Part) -> String {
    let row = &table[p.row];
    if row.each_group { format!("{} {} {}", row.word, p.group, row.name) } else { row.name.to_string() }
}

/// The list's places in the order the handles are made and the entities
/// spawned at the start.
pub fn made_order(table: &[YearMesh]) -> Vec<usize> {
    let parts = list(table);
    let mut order: Vec<usize> = (0..parts.len()).collect();
    order.sort_by_key(|&i| table[parts[i].row].made);
    order
}

/// The order a year's meshes go to the renderer in, as places in the list.
/// `whole` is false for a year built again for the camera alone, which sends
/// only the meshes that say so. While they go the picture holds two years at
/// once, so what belongs together goes together, and what the eye is on goes
/// first: the houses, with the ground's patchwork, which changes in the same
/// frame; the works and the yards they stand in; their smoke; the roads; the
/// railway, whose trains are on their new lines from the first frame; then the
/// trees that show, group by group; and last the trees the season has hidden.
/// What lies flat is a few megabytes and holds the trees up by a frame. It was
/// tried last, as the first brief had it, and for as long as the trees took to
/// go the new works stood on bare ground with no yard, the new roads were
/// missing, and a train ran where no track was drawn (docs/probes/dx-smooth.md,
/// section 12, "Stage one, as built", says what the pictures of a year half
/// arrived showed).
pub fn hand_over(table: &[YearMesh], phase: f32, whole: bool) -> Vec<usize> {
    let parts = list(table);
    let mut order: Vec<usize> = (0..parts.len()).filter(|&i| whole || table[parts[i].row].camera).collect();
    order.sort_by_key(|&i| {
        let (row, group) = (&table[parts[i].row], parts[i].group);
        match row.goes {
            Goes::Before(n) => (0, n),
            Goes::WithTrees if row.drawn.shows(phase, group) => (1, 0),
            Goes::WithTrees => (2, 0),
        }
    });
    order
}

/// Take a year's meshes from what its makers made, in the order of the list,
/// running the makers of their own.
pub fn take(table: &[YearMesh], ym: &mut scatter::YearMeshes, nets: &mut network::Nets, inputs: &Inputs) -> Vec<Builder> {
    list(table)
        .into_iter()
        .map(|p| match table[p.row].make {
            Make::Scatter(f) => f(ym, p.group),
            Make::Lines(f) => f(nets),
            Make::Own(f) => f(inputs, p.group),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(order: &[usize]) -> Vec<String> {
        let parts = list(MESHES);
        order.iter().map(|&i| name(MESHES, parts[i])).collect()
    }

    fn trees(groups: impl Iterator<Item = usize>, parts: &[&str]) -> Vec<String> {
        groups.flat_map(|g| parts.iter().map(move |p| format!("trees {g} {p}"))).collect()
    }

    #[test]
    fn the_table_agrees_with_the_lists_it_replaces() {
        let empty = || scatter::YearMeshes { trees: Default::default(), houses: Builder::default(), works: Builder::default(), smoke: Builder::default(), lanes: Builder::default() };
        let (ym, nets) = (empty(), network::Nets { roads: Builder::default(), rails: Builder::default() });
        let old: Vec<String> = crate::year_parts(&ym, &nets).into_iter().map(|(n, _)| n).collect();
        let parts = list(MESHES);
        let new: Vec<String> = parts.iter().map(|&p| name(MESHES, p)).collect();
        // the same meshes by name, each once
        let (mut a, mut b) = (old.clone(), new.clone());
        a.sort();
        b.sort();
        assert_eq!(a, b);
        b.dedup();
        assert_eq!(a.len(), b.len());
        // the index constants point at the names they always did
        for (at, n) in [(crate::HOUSES, "houses"), (crate::WORKS, "works"), (crate::SMOKE, "smoke"), (crate::LANES, "yards and lanes"), (crate::ROADS, "roads"), (crate::RAILS, "railway"), (crate::TREES, "trees 0 evergreen")] {
            assert_eq!(old[at], n);
        }
        // the handles are made and the entities spawned as `setup_world` makes them: each
        // group of trees, then the houses, the works, the smoke, the roads, the railway and
        // the yards and lanes last
        let mut made = trees(0..LEAF_GROUPS, &["evergreen", "leaves", "bare limbs"]);
        made.extend(["houses", "works", "smoke", "roads", "railway", "yards and lanes"].map(String::from));
        assert_eq!(names(&made_order(MESHES)), made);
        // the hand-over is `hand_over_order`'s, at every phase of the year
        let mut assets = bevy::asset::Assets::<bevy::mesh::Mesh>::default();
        let handles: Vec<_> = old.iter().map(|_| assets.add(crate::placeholder_mesh())).collect();
        let piece = |i: usize| crate::handover::Piece { handle: handles[i].clone(), mesh: crate::placeholder_mesh(), bytes: 0, print: None };
        let of = |pieces: Vec<crate::handover::Piece>| -> Vec<String> { pieces.iter().map(|p| old[handles.iter().position(|h| h.id() == p.handle.id()).unwrap()].clone()).collect() };
        for i in 0..96 {
            let phase = i as f32 / 96.0;
            let (mut showing, mut hidden) = (Vec::new(), Vec::new());
            for group in 0..LEAF_GROUPS {
                let at = crate::TREES + 3 * group;
                showing.push(piece(at));
                let (out, away) = if in_leaf(phase, group) { (at + 1, at + 2) } else { (at + 2, at + 1) };
                showing.push(piece(out));
                hidden.push(piece(away));
            }
            let built = [piece(crate::HOUSES), piece(crate::WORKS), piece(crate::SMOKE)];
            let flat = [piece(crate::LANES), piece(crate::ROADS), piece(crate::RAILS)];
            assert_eq!(names(&hand_over(MESHES, phase, true)), of(crate::hand_over_order(built, flat, showing, hidden)), "phase {phase}");
            // and a year built again for the camera sends its houses, works and smoke, in that order
            assert_eq!(names(&hand_over(MESHES, phase, false)), ["houses", "works", "smoke"]);
        }
        // the fingerprints' line and the bench's count go by these words, in this order
        let mut words: Vec<&str> = MESHES.iter().map(|m| m.word).collect();
        words.dedup();
        assert_eq!(words, ["trees", "houses", "works", "smoke", "lanes", "roads", "rails"]);
        // each is drawn in what `setup_world` drew it in
        let drawn = |n: &str| MESHES.iter().find(|m| m.name == n).unwrap().drawn;
        assert_eq!(["houses", "works", "smoke", "yards and lanes", "roads", "railway", "evergreen", "leaves", "bare limbs"].map(drawn), [Drawn::Built, Drawn::Built, Drawn::Smoke, Drawn::Flat, Drawn::Flat, Drawn::Flat, Drawn::Evergreen, Drawn::Leaves, Drawn::Bare]);
    }
}
