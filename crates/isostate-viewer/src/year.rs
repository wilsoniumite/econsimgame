//! A year's scenery: the one table of the meshes a year hands to the card,
//! and how a year is built off the main thread and put on screen
//! (`rebuild_year`, which came here from `main.rs` unchanged but for the table).
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
//! Three orders of the same meshes can be seen, in the pictures or in what is
//! printed, so the table says each:
//!
//! - the table's own order, which is the order of the list everything else
//!   indexes (`list`), of `ISOSTATE_MESH_SUMS`'s line and of the bench's count;
//! - `made`, the order the meshes' handles are made and their entities spawned
//!   at the start. The meshes' asset numbers follow it, and two meshes of one
//!   material are drawn in that order, which decides the pixels where they lie
//!   at one height (a lane where it meets a road);
//! - `goes`, the order of the hand-over, which was hard won (`hand_over`).

use bevy::prelude::*;
use bevy::tasks::{futures::check_ready, AsyncComputeTaskPool, Task};
use isostate_gen::trees::{in_leaf, LEAF_GROUPS};

use crate::built::BuiltMaterial;
use crate::look::{Look, TheLook};
use crate::meshkit::{self, Builder};
use crate::replay::Replay;
use crate::world::{Ground, YearLand};
use crate::{frames, handover, network, scatter, terrain, trains, world};
use crate::{non_empty, season_phase, timing, BareLimbs, Data, Evergreen, ForcedSeason, Handles, Leaves, Orbit, Rails, SeasonClock, Timeline, Train, TrainAssets};

/// One kind of mesh a year's scenery hands to the card.
#[derive(Clone, Copy)]
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
    #[cfg_attr(not(test), allow(dead_code))]
    Own(fn(&Inputs, usize) -> Builder),
}

/// What a year's scenery is built from: everything its makers read. (Only a
/// maker of its own is handed it whole, and today only the test's has one.)
#[allow(dead_code)]
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
///
/// A new mesh is a row at the foot, with a maker of its own
/// (`Make::Own(canals::year)`, a function from `Inputs` to a `Builder`), or a
/// mesh taken from what a shared maker already makes. Give it a word of its
/// own, so that the fingerprints of the other words stay as they were; a
/// `made` past the last, so that every handle made before it keeps its
/// number; and the place in the hand-over where it belongs with what it is
/// seen beside (put between two, the places after it move up by one). The test
/// `a_row_in_the_table_is_all_a_new_mesh_needs` does exactly this.
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

/// The materials a year's meshes are drawn in (`Drawn`), made once at the start.
pub struct Materials {
    pub built: Handle<BuiltMaterial>,
    pub flat: Handle<BuiltMaterial>,
    pub smoke: Handle<StandardMaterial>,
    pub evergreen: Handle<StandardMaterial>,
    pub leaves: Handle<StandardMaterial>,
    pub bare: Handle<StandardMaterial>,
}

/// At the start: a handle for each of a year's meshes, holding a stand-in
/// until the first year is built (`rebuild_year` fills them), and an entity to
/// draw it in its material. Both are made in the order `made` says; the
/// handles are given back in the order of the list.
pub fn spawn(commands: &mut Commands, meshes: &mut Assets<Mesh>, table: &[YearMesh], mats: &Materials) -> Vec<Handle<Mesh>> {
    let parts = list(table);
    let order = made_order(table);
    let mut handles = vec![Handle::default(); parts.len()];
    for &i in &order {
        handles[i] = meshes.add(crate::placeholder_mesh());
    }
    // (the year's meshes span the island and are never out of view, so none is ever culled,
    // and the engine need not read every corner of one to find its box in the frame it
    // goes on screen: `bevy_camera::visibility::calculate_bounds` passes these by)
    use bevy::camera::visibility::NoFrustumCulling as Whole;
    use bevy::light::NotShadowCaster;
    for &i in &order {
        let (h, group) = (handles[i].clone(), parts[i].group);
        match table[parts[i].row].drawn {
            Drawn::Evergreen => commands.spawn((Mesh3d(h), MeshMaterial3d(mats.evergreen.clone()), Evergreen, Whole)),
            Drawn::Leaves => commands.spawn((Mesh3d(h), MeshMaterial3d(mats.leaves.clone()), Leaves(group), Whole)),
            Drawn::Bare => commands.spawn((Mesh3d(h), MeshMaterial3d(mats.bare.clone()), BareLimbs(group), Visibility::Hidden, Whole)),
            Drawn::Built => commands.spawn((Mesh3d(h), MeshMaterial3d(mats.built.clone()), Whole)),
            Drawn::Smoke => commands.spawn((Mesh3d(h), MeshMaterial3d(mats.smoke.clone()), NotShadowCaster, Whole)),
            Drawn::Flat => commands.spawn((Mesh3d(h), MeshMaterial3d(mats.flat.clone()), NotShadowCaster, Whole)),
        };
    }
    handles
}

/// A finished build's meshes (in the order of the list), made ready for the
/// card under their handles (`handles`, in the same order) and in the order of
/// the hand-over, with their fingerprints where the build took them. A year
/// built again for the camera alone (`whole` false) sends only the meshes
/// whose row says so, and gives back the others to be let go of; and how many
/// corners are sent.
pub fn queue(table: &[YearMesh], meshes: Vec<Builder>, handles: &[Handle<Mesh>], prints: Option<&[(usize, u64)]>, whole: bool, phase: f32) -> (Vec<crate::handover::Piece>, Vec<Builder>, usize) {
    let parts = list(table);
    let sends = |i: usize| whole || table[parts[i].row].camera;
    let corners = meshes.iter().enumerate().filter(|&(i, _)| sends(i)).map(|(_, b)| b.pos.len()).sum();
    let (mut going, mut rest) = (Vec::new(), Vec::new());
    for (i, b) in meshes.into_iter().enumerate() {
        if sends(i) {
            going.push(Some(piece(&handles[i], b, prints.map(|p| p[i]))));
        } else {
            going.push(None);
            rest.push(b);
        }
    }
    let pieces = hand_over(table, phase, whole).into_iter().filter_map(|i| going[i].take()).collect();
    (pieces, rest, corners)
}

/// Set a year built afresh (its meshes in the order of the list) against what
/// was last handed over under each handle (`HandOver::on_screen`): how many
/// corners the fresh build has, and each mesh on screen that is not the fresh
/// build's, by name, with its corners on screen and in the build.
pub fn compare(table: &[YearMesh], fresh: &[Builder], handles: &[Handle<Mesh>], on_screen: &std::collections::HashMap<AssetId<Mesh>, (usize, u64)>) -> (usize, Vec<String>) {
    let mut corners = 0;
    let mut wrong = Vec::new();
    for ((p, b), h) in list(table).into_iter().zip(fresh).zip(handles) {
        let want = print_of(b);
        let have = on_screen.get(&h.id()).copied();
        corners += want.0;
        if have != Some(want) {
            wrong.push(format!("{} ({} corners on screen, {} in the year's own)", name(table, p), have.map_or(0, |p| p.0), want.0));
        }
    }
    (corners, wrong)
}

/// The name of the year's mesh under `handle`, for a picture's caption.
pub fn named(table: &[YearMesh], handles: &[Handle<Mesh>], handle: &Handle<Mesh>) -> String {
    match handles.iter().position(|h| h.id() == handle.id()) {
        Some(i) => name(table, list(table)[i]),
        None => "?".to_string(),
    }
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

/// A year's meshes (in the order of the list) by the words of the table, in
/// the table's order: each word with its meshes, in the list's order, and the
/// row of each.
fn by_word<'a>(table: &[YearMesh], meshes: &'a [Builder]) -> Vec<(&'static str, Vec<(usize, &'a Builder)>)> {
    let mut out: Vec<(&'static str, Vec<(usize, &'a Builder)>)> = Vec::new();
    for (p, b) in list(table).into_iter().zip(meshes) {
        let word = table[p.row].word;
        match out.iter_mut().find(|(w, _)| *w == word) {
            Some((_, of)) => of.push((p.row, b)),
            None => out.push((word, vec![(p.row, b)])),
        }
    }
    out
}

/// `ISOSTATE_MESH_SUMS`'s line for a year's meshes (in the order of the
/// list): each word of the table with a fingerprint of its meshes, those that
/// share a word put together in the list's order.
pub fn sums(table: &[YearMesh], meshes: &[Builder]) -> String {
    let sums: Vec<String> = by_word(table, meshes)
        .into_iter()
        .map(|(word, of)| {
            let sum = match of.as_slice() {
                [(_, one)] => mesh_sum(one),
                _ => mesh_sum(&of.iter().fold(Builder::default(), |mut all, (_, b)| {
                    all.merge(b);
                    all
                })),
            };
            format!("{word} {sum:016x}")
        })
        .collect();
    sums.join(", ")
}

/// The bench's count of a year's vertices (`ISOSTATE_BENCH`), by the same
/// words as `sums`; where meshes of more than one row share a word, the bare
/// limbs among them are counted apart as well ("trees 3412 (of them bare
/// limbs 802)").
pub fn counts(table: &[YearMesh], meshes: &[Builder]) -> String {
    let counts: Vec<String> = by_word(table, meshes)
        .into_iter()
        .map(|(word, of)| {
            let all: usize = of.iter().map(|(_, b)| b.pos.len()).sum();
            let bare = of.iter().find(|&&(row, _)| table[row].drawn == Drawn::Bare && of.iter().any(|&(other, _)| other != row)).map(|&(row, _)| row);
            match bare {
                Some(bare) => format!("{word} {all} (of them {} {})", table[bare].name, of.iter().filter(|&&(row, _)| row == bare).map(|(_, b)| b.pos.len()).sum::<usize>()),
                None => format!("{word} {all}"),
            }
        })
        .collect();
    counts.join(", ")
}

/// The meshes a year built again for the camera alone sends, by name, as a
/// sentence says them ("houses, works and smoke").
pub fn camera_names(table: &[YearMesh]) -> String {
    let names: Vec<&str> = table.iter().filter(|m| m.camera).map(|m| m.name).collect();
    match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        Some((last, _)) => last.to_string(),
        None => "nothing".to_string(),
    }
}

// ------------------------------------------------------------------ the year, built and put on screen

/// Whether a year's build takes a fingerprint of each of its meshes, to be
/// kept as the record of what was handed to the renderer. The year's meshes
/// keep no copy of themselves on the main thread once the renderer has taken
/// them: nothing in the game reads one back, and a year is some ten million
/// corners (400 MB), so the renderer takes the mesh itself and not a copy of
/// it, and the old year's buffers are not freed on the main thread. That is
/// so in a scripted run too, which must time what the game does. A script
/// with a `scenery` step, which checks what is on screen against the year
/// built afresh (`check_scenery`), asks for the fingerprints instead: they
/// are taken off the main thread, by the build, and cost a frame nothing.
pub static PRINT_YEAR_MESHES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A year's mesh of one kind on its way to the renderer under `handle`.
fn piece(handle: &Handle<Mesh>, b: meshkit::Builder, print: Option<(usize, u64)>) -> handover::Piece {
    // what the card will hold of it: a corner is its place, its normal and its colour
    // (40 bytes), and each index four more; nothing at all is the stand-in's one triangle
    let bytes = if b.is_empty() { 3 * 40 } else { b.pos.len() * 40 + b.idx.len() * 4 };
    handover::Piece { handle: handle.clone(), mesh: non_empty(b), bytes, print }
}

/// A mesh buffer's corners and its fingerprint, as `non_empty` would put it on screen.
fn print_of(b: &meshkit::Builder) -> (usize, u64) {
    if b.is_empty() { (3, 0) } else { (b.pos.len(), mesh_sum(b)) }
}

/// Whether the trains of two years run on the same lines.
fn same_lines(a: &[trains::TrainPath], b: &[trains::TrainPath]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.pts == b.pts)
}

/// A year's scenery: the ground's land mix, the meshes and the railway paths.
pub struct Scenery {
    k: usize,
    /// the camera its houses were built for
    lod: scatter::Lod,
    cells: Vec<u8>,
    /// (task C, the land) the year's wood and heath (`world::YearLand::field`)
    field: Vec<u8>,
    /// the year's meshes, one to each of the table's list (`list`)
    meshes: Vec<meshkit::Builder>,
    rails: Vec<trains::TrainPath>,
    /// each mesh's corners and fingerprint in the order of the list, where a
    /// scripted run checks what is on screen (`PRINT_YEAR_MESHES`)
    prints: Option<Vec<(usize, u64)>>,
}

/// The year's scenery being built off the main thread, so a new year never
/// stops the picture. One build at a time; the year on screen keeps its
/// scenery until the next is ready, and the ground changes with it.
#[derive(Resource, Default)]
pub struct Builds {
    /// the year being built, and which state of the run it is built from (`Shown::run`)
    pub task: Option<(usize, u32, Task<Scenery>)>,
}

/// What `rebuild_year` remembers of the scenery on screen (a resource, so
/// that a scripted run can say what was put on screen and what it cost).
#[derive(Resource, Default)]
pub struct Shown {
    /// the camera its houses were built for
    lod: Option<scatter::Lod>,
    /// how many years the run holds: what scenery is built from changes only when a year
    /// arrives in a live game (the months between write their figures and their news into
    /// the run, and nothing a year's scenery reads)
    run: u32,
    /// the year, and the state of the run, that everything on screen was last built from
    whole: Option<(usize, u32)>,
    /// the year the clock stood in when `rebuild_year` last looked
    clock: Option<usize>,
    /// how many times a whole year began to go on screen, and how many times the houses,
    /// the works and their smoke alone, for the camera (the frame recorder's W and C)
    pub swaps: (u32, u32),
    /// the last time scenery went on screen: whether it was for the camera alone, how
    /// many corners were queued for the renderer, and how long the queueing took on the
    /// main thread, in ms. (What the frames cost while it went is the frame recorder's
    /// to say: `frames::swap_longest`.)
    last: Option<(bool, usize, f32)>,
}

/// Build a year's scenery: the meshes `table` lists (the game's is `MESHES`).
#[allow(clippy::too_many_arguments)]
fn build_scenery(table: &[YearMesh], r: &Replay, g: &Ground, slots: &scatter::Slots, cen: &[(f32, f32)], k: usize, lk: &Look, lod: scatter::Lod) -> Scenery {
    let t0 = std::time::Instant::now();
    // (task C, the land) one reckoning of the year's land for the ground's textures and the scenery
    let land = world::timed("year land", || world::year_land(r, g, k));
    let cells = terrain::year_parcels(r, g, k, &land);
    let mut ym = scatter::build_year_on(r, g, slots, cen, k, lk, &lod, &land);
    let t1 = t0.elapsed().as_secs_f32();
    let mut nets = network::year_nets(r, g, cen, k, lk);
    let rails = trains::rail_paths(r, g, cen, k);
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        eprintln!("year {}: scatter {:.0} ms, networks {:.0} ms (off the main thread); {} vertices of houses for a camera {:.1} off", r.y0 + k as i32, t1 * 1e3, (t0.elapsed().as_secs_f32() - t1) * 1e3, ym.houses.pos.len(), lod.dist);
    }
    // each mesh the table lists, taken from what the makers made, or made by its own maker
    let meshes = take(table, &mut ym, &mut nets, &Inputs { r, g, slots, cen, k, lk, lod: &lod, land: &land });
    if std::env::var_os("ISOSTATE_MESH_SUMS").is_some() {
        // a fingerprint of the year's geometry: two builds of the viewer that print the
        // same line for a year build exactly the same scenery for it, to the last bit
        eprintln!("year {} meshes: {}", r.y0 + k as i32, sums(table, &meshes));
        // (`ISOSTATE_MESH_SUMS=each` adds a line for every mesh the year hands to the card, by
        // its own name: its corners, its indices and its fingerprint)
        if std::env::var("ISOSTATE_MESH_SUMS").is_ok_and(|v| v == "each") {
            for (p, b) in list(table).into_iter().zip(&meshes) {
                eprintln!("year {} mesh {}: {} corners, {} indices, {:016x}", r.y0 + k as i32, name(table, p), b.pos.len(), b.idx.len(), mesh_sum(b));
            }
        }
    }
    // (a scripted run that checks what is on screen: each mesh's fingerprint, taken here,
    // off the main thread, and kept as the record of what the renderer was handed)
    let prints = PRINT_YEAR_MESHES.load(std::sync::atomic::Ordering::Relaxed).then(|| meshes.iter().map(print_of).collect());
    Scenery { k, lod, cells, field: land.field, meshes, rails, prints }
}

/// A fingerprint of a mesh buffer: every bit of every corner, normal, colour and index.
pub fn mesh_sum(b: &meshkit::Builder) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut eat = |v: u32| {
        for byte in v.to_le_bytes() {
            h = (h ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for p in b.pos.iter().chain(&b.nrm) {
        p.iter().for_each(|v| eat(v.to_bits()));
    }
    for c in &b.col {
        c.iter().for_each(|v| eat(v.to_bits()));
    }
    b.idx.iter().for_each(|&i| eat(i));
    h
}

#[allow(clippy::too_many_arguments)]
pub fn rebuild_year(
    mut commands: Commands,
    mut tl: ResMut<Timeline>,
    data: Res<Data>,
    look: Res<TheLook>,
    handles: Res<Handles>,
    ta: Res<TrainAssets>,
    mut rails: ResMut<Rails>,
    mut builds: ResMut<Builds>,
    orbit: Res<Orbit>,
    mut shown: ResMut<Shown>,
    mut hand: ResMut<handover::HandOver>,
    season: Res<SeasonClock>,
    forced: Res<ForcedSeason>,
    old_trains: Query<Entity, With<Train>>,
    mut images: ResMut<Assets<Image>>,
) {
    let _frames = frames::span(frames::Sys::Scenery);
    // (task D) the state of the run that scenery is built from is the years it holds. It was
    // counted by every change to the run, and since the clock tells the month a month's
    // figures change it twelve times a year: a year built again for the camera then never
    // counted as the camera's alone, and went on screen whole
    let run = data.r.filled as u32;
    if shown.run != run {
        shown.run = run;
        // the frame a year arrives in has work enough of its own. `game::pump`, which writes the
        // year in, runs before this in every frame (`year_systems`), so it is this very frame.
        // (Whether the meshes on their way are of the run as it was is asked when the last has
        // gone: `HandOver::finished`.)
        hand.hold();
    }
    // so has the frame the clock's year changes in
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    if shown.clock != Some(k) {
        shown.clock = Some(k);
        hand.hold();
    }
    // the houses are built with as much as the camera can see of them: when it has moved or
    // zoomed past what the scenery on screen was built for, the year is built again
    let lod = scatter::Lod::of_camera((orbit.target.x, orbit.target.z), orbit.dist);
    if tl.built.is_some() && shown.lod.is_some_and(|was| was.stale(&lod)) {
        tl.built = None;
    }
    // a finished build is queued for the renderer, which is handed it a mesh at a time
    // (handover.rs): the year is on screen, and `tl.built` says so, when the last has gone
    if let Some((run, sc)) = builds.task.as_mut().and_then(|(_, run, t)| check_ready(t).map(|sc| (*run, sc))) {
        builds.task = None;
        let began = std::time::Instant::now();
        shown.lod = Some(sc.lod);
        // a year built again for the camera alone, from the run as it was, differs only in its
        // houses, its works and their smoke (a garden's tree is drawn at every level): the rest
        // stays as it is on screen, the trees are not sent to the card again (they are most of
        // the year's corners), and the trains run on from where they are
        let for_camera = shown.whole == Some((sc.k, run));
        shown.whole = Some((sc.k, run));
        // (task B, one scale of detail: the works, the pits and the bridges take their level
        // from the camera as the houses do, so their mesh goes on screen again with the houses';
        // their yards' ground, in `lanes`, is the same at every level and stays. Which meshes
        // go again for the camera is the table's to say: `YearMesh::camera`). They are
        // queued in the hand-over's order (`hand_over`); of each group of trees, what is
        // green all year and whichever of its leaves and its bare limbs the season has out
        // show, and the other is hidden until the season turns
        let phase = season_phase(&season, forced.0);
        let (pieces, rest, corners) = queue(MESHES, sc.meshes, &handles.meshes, sc.prints.as_deref(), !for_camera, phase);
        if for_camera {
            // the rest of what was built is not wanted. It is 300 MB of trees and more, and
            // letting go of it here cost the frame 10 ms: it is let go of off the main thread
            let rest = (rest, sc.cells, sc.field, sc.rails);
            AsyncComputeTaskPool::get().spawn(async move { drop(rest) }).detach();
        } else {
            // the ground's patchwork changes with the first mesh
            if let Some(mut img) = images.get_mut(&handles.year_cells) {
                img.data = Some(sc.cells);
            }
            if let Some(mut img) = images.get_mut(&handles.year_field) {
                img.data = Some(sc.field);
            }
            // trains: one for every eight units of line, up to three, spread along it. While
            // the lines are as they were the trains run on from where they are: a new year
            // does not send every train back to where it started, nor does a year built
            // again for the camera after the run has moved on (in a live game the next year
            // arrives a little before the clock reaches it, and a camera build begun in that
            // while is not the camera's alone: the works may have been laid out again)
            if !same_lines(&rails.0, &sc.rails) {
                for e in &old_trains {
                    commands.entity(e).despawn();
                }
                rails.0 = sc.rails;
                // (ISOSTATE_TRAINS_AT, for pictures: a train at each spot named, and every train held still)
                let pins = trains::pinned(&rails.0);
                for (pi, p) in rails.0.iter().enumerate() {
                    let count = (1 + (p.len / 8.0) as usize).min(3);
                    for i in 0..count {
                        let pin = pins.as_ref().and_then(|v| v.iter().find(|q| q.0 == pi)).filter(|_| i == 0);
                        let s = pin.map_or(p.len * (i as f32 + 0.5) / count as f32, |q| q.1);
                        let (pos, yaw) = p.at(s);
                        let dir = if (pi + i) % 2 == 0 { 1.0 } else { -1.0 };
                        commands.spawn((
                            Mesh3d(ta.mesh.clone()),
                            MeshMaterial3d(ta.mat.clone()),
                            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw + if dir < 0.0 { std::f32::consts::PI } else { 0.0 })),
                            Train { path: pi, s, dir, speed: if pins.is_some() { 0.0 } else { 0.5 + 0.3 * world::hash2(pi as i32, i as i32, 3) } },
                        ));
                    }
                }
            }
        }
        hand.fill(sc.k, run, pieces);
        let ms = began.elapsed().as_secs_f32() * 1e3;
        if for_camera { shown.swaps.1 += 1 } else { shown.swaps.0 += 1 }
        shown.last = Some((for_camera, corners, ms));
        if timing() {
            let what = if for_camera { format!("its {} alone, for the camera", camera_names(MESHES)) } else { "the whole year".to_string() };
            eprintln!("year {} built: {what} ({corners} corners) queued for the renderer in {ms:.1} ms on the main thread", data.r.y0 + sc.k as i32);
        }
    }
    // the year on screen, if it is neither shown nor being built; a build for another year
    // (the clock moved on) finishes first and is shown on the way. And none is begun while
    // the last is still going to the renderer: a year is shown whole or not at all, so when
    // the years come faster than they can be shown some are skipped and none is left half sent
    if tl.built != Some(k) && builds.task.is_none() && hand.is_empty() {
        let (r, g, slots, cen, lk) = (data.r.clone(), data.g.clone(), data.slots.clone(), data.cen.clone(), look.0.clone());
        let task = AsyncComputeTaskPool::get().spawn(async move { build_scenery(MESHES, &r, &g, &slots, &cen, k, &lk, lod) });
        builds.task = Some((k, shown.run, task));
    }
}

/// The scenery shows the year on the clock (screenshots wait for it).
pub fn scenery_ready(tl: &Timeline, data: &Data) -> bool {
    tl.built == Some((tl.t.floor() as usize).min(data.r.ny - 1))
}

/// For a scripted run's `print`: what scenery is on screen, how it got there
/// and what the last change of it cost. The longest frame is counted from the
/// last time this was asked, by the frame recorder (frames.rs), which is the
/// one measure of a frame there is: its frame is from one frame's start on the
/// main thread to the next's, so it holds the wait for the renderer too.
pub fn scenery_line(world: &mut World) -> String {
    let orbit = world.resource::<Orbit>();
    let cam = scatter::Lod::of_camera((orbit.target.x, orbit.target.z), orbit.dist);
    let y0 = world.resource::<Data>().r.y0;
    let built = world.resource::<Timeline>().built;
    let longest = frames::take_longest(world);
    let after = frames::swap_longest(world);
    let shown = world.resource::<Shown>();
    let year = built.map_or("none".to_string(), |k| (y0 + k as i32).to_string());
    let houses = match shown.lod {
        Some(lod) => format!("houses for a camera {:.1} off at {:.1},{:.1}{}", lod.dist, lod.focus.0, lod.focus.1, if lod.stale(&cam) { ", which the camera has left" } else { "" }),
        None => "no houses yet".to_string(),
    };
    let last = match (shown.last, after) {
        (Some((for_camera, corners, ms)), after) => format!(
            "the last was {} of {corners} corners, queued for the renderer in {ms:.1} ms{}",
            if for_camera { "houses alone" } else { "a whole year" },
            after.map_or(String::new(), |a| format!(", and the longest frame while the renderer took it in was {a:.0} ms"))
        ),
        (None, _) => "nothing yet".to_string(),
    };
    let waiting = match world.resource::<handover::HandOver>().waiting() {
        (0, _) => String::new(),
        (n, bytes) => format!("; {n} meshes ({:.0} MB) still on their way to the renderer", bytes as f32 / 1e6),
    };
    format!(
        "scenery of {year}, {houses}; put on screen {} times whole and {} times as houses alone; {last}{waiting}; longest frame since last asked {}",
        shown.swaps.0,
        shown.swaps.1,
        longest.map_or("not measured".to_string(), |l| format!("{l:.0} ms"))
    )
}

/// For a scripted run's `scenery`: build the year on the clock afresh, from
/// the run as it stands and for the camera the houses on screen were built
/// for, and say whether every mesh on screen is that build's. A swap that
/// left something behind (the works of last year under this year's houses,
/// say) shows here. Err names the meshes that differ.
///
/// The renderer alone keeps a year's meshes, in a scripted run as in the game,
/// so there is no mesh on the main thread to read back. What is compared is
/// the record kept at the hand-over: the corners and the fingerprint of the
/// mesh last assigned under each handle (`handover::feed`), taken by the build
/// that made it, against the fresh build's.
pub fn check_scenery(world: &mut World) -> Result<String, String> {
    // (the build below is the check's, not the game's: the frame it is in is not the renderer's)
    frames::aside(world);
    let Some(lod) = world.resource::<Shown>().lod else { return Err("no scenery has been built yet".into()) };
    let (data, tl, look) = (world.resource::<Data>(), world.resource::<Timeline>(), world.resource::<TheLook>());
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    if tl.built != Some(k) {
        return Err(format!("the scenery on screen is of {:?}, not of the year on the clock ({k}): settle first", tl.built));
    }
    let year = data.r.y0 + k as i32;
    let fresh = build_scenery(MESHES, &data.r, &data.g, &data.slots, &data.cen, k, &look.0, lod);
    let (handles, hand) = (world.resource::<Handles>(), world.resource::<handover::HandOver>());
    let (corners, wrong) = compare(MESHES, &fresh.meshes, &handles.meshes, &hand.on_screen);
    if wrong.is_empty() {
        Ok(format!("all {corners} corners on screen are {year}'s own, built afresh for the same camera"))
    } else {
        Err(format!("on screen for {year} but not that year's own: {}", wrong.join(", ")))
    }
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

    /// The hand-over as it was written out before the table (`hand_over_order`
    /// in `main.rs` until 13 November 2026): the houses, the works, the yards
    /// and lanes, the smoke, the roads, the railway; then group by group what
    /// is green all year and whichever of the leaves and the bare limbs the
    /// season shows; then, group by group, the other.
    fn todays_hand_over(phase: f32) -> Vec<String> {
        let mut want: Vec<String> = ["houses", "works", "yards and lanes", "smoke", "roads", "railway"].map(String::from).to_vec();
        for g in 0..LEAF_GROUPS {
            want.push(format!("trees {g} evergreen"));
            want.push(format!("trees {g} {}", if in_leaf(phase, g) { "leaves" } else { "bare limbs" }));
        }
        for g in 0..LEAF_GROUPS {
            want.push(format!("trees {g} {}", if in_leaf(phase, g) { "bare limbs" } else { "leaves" }));
        }
        want
    }

    /// What a queue holds, by name.
    fn queued(table: &[YearMesh], handles: &[Handle<Mesh>], pieces: &[crate::handover::Piece]) -> Vec<String> {
        pieces.iter().map(|p| named(table, handles, &p.handle)).collect()
    }

    /// Meshes with something in them: one triangle each, told apart by where it stands.
    fn meshes_of(table: &[YearMesh]) -> Vec<Builder> {
        (0..list(table).len())
            .map(|i| {
                let mut b = Builder::default();
                let x = i as f32;
                b.tri(Vec3::new(x, 0.0, 0.0), Vec3::new(x, 0.0, 0.1), Vec3::new(x + 0.1, 0.0, 0.0), [0.5, 0.5, 0.5, 1.0]);
                b
            })
            .collect()
    }

    #[test]
    fn the_table_keeps_the_orders_the_lists_had() {
        // every mesh by name, each once
        let mut all: Vec<String> = list(MESHES).into_iter().map(|p| name(MESHES, p)).collect();
        all.sort();
        let mut want = trees(0..LEAF_GROUPS, &["evergreen", "leaves", "bare limbs"]);
        want.extend(["houses", "works", "smoke", "yards and lanes", "roads", "railway"].map(String::from));
        want.sort();
        assert_eq!(all, want);
        // the handles are made and the entities spawned as `setup_world` made them: each
        // group of trees, then the houses, the works, the smoke, the roads, the railway and
        // the yards and lanes last
        let mut made = trees(0..LEAF_GROUPS, &["evergreen", "leaves", "bare limbs"]);
        made.extend(["houses", "works", "smoke", "roads", "railway", "yards and lanes"].map(String::from));
        assert_eq!(names(&made_order(MESHES)), made);
        // the hand-over is as it was, at every phase of the year, and so is the queue
        let mut assets = Assets::<Mesh>::default();
        let handles: Vec<_> = list(MESHES).iter().map(|_| assets.add(crate::placeholder_mesh())).collect();
        for i in 0..96 {
            let phase = i as f32 / 96.0;
            assert_eq!(names(&hand_over(MESHES, phase, true)), todays_hand_over(phase), "phase {phase}");
            let (pieces, rest, corners) = queue(MESHES, meshes_of(MESHES), &handles, None, true, phase);
            assert_eq!(queued(MESHES, &handles, &pieces), todays_hand_over(phase), "phase {phase}");
            assert_eq!((rest.len(), corners), (0, 3 * handles.len()));
            // and a year built again for the camera sends its houses, works and smoke, in that
            // order, and gives back the rest to be let go of
            assert_eq!(names(&hand_over(MESHES, phase, false)), ["houses", "works", "smoke"]);
            let (pieces, rest, corners) = queue(MESHES, meshes_of(MESHES), &handles, None, false, phase);
            assert_eq!(queued(MESHES, &handles, &pieces), ["houses", "works", "smoke"]);
            assert_eq!((rest.len(), corners), (handles.len() - 3, 9));
        }
        assert_eq!(camera_names(MESHES), "houses, works and smoke");
        // the fingerprints' line and the bench's count go by these words, in this order
        let mut words: Vec<&str> = MESHES.iter().map(|m| m.word).collect();
        words.dedup();
        assert_eq!(words, ["trees", "houses", "works", "smoke", "lanes", "roads", "rails"]);
        let line = counts(MESHES, &meshes_of(MESHES));
        assert_eq!(line, format!("trees {} (of them bare limbs {}), houses 3, works 3, smoke 3, lanes 3, roads 3, rails 3", 9 * LEAF_GROUPS, 3 * LEAF_GROUPS));
        // the trees are fingerprinted as one mesh, group by group
        let meshes = meshes_of(MESHES);
        let trees: Builder = meshes[..3 * LEAF_GROUPS].iter().fold(Builder::default(), |mut all, b| {
            all.merge(b);
            all
        });
        assert!(sums(MESHES, &meshes).starts_with(&format!("trees {:016x}, houses {:016x}, ", mesh_sum(&trees), mesh_sum(&meshes[3 * LEAF_GROUPS]))));
        // each is drawn in what `setup_world` drew it in
        let drawn = |n: &str| MESHES.iter().find(|m| m.name == n).unwrap().drawn;
        assert_eq!(["houses", "works", "smoke", "yards and lanes", "roads", "railway", "evergreen", "leaves", "bare limbs"].map(drawn), [Drawn::Built, Drawn::Built, Drawn::Smoke, Drawn::Flat, Drawn::Flat, Drawn::Flat, Drawn::Evergreen, Drawn::Leaves, Drawn::Bare]);
    }

    /// Spawn a table's meshes into a world of their own: for each entity in the
    /// order it was spawned, the mesh's name, what it is drawn in and what it
    /// carries beside; and whether the meshes' handles were made in that order.
    pub(super) fn spawned(table: &[YearMesh]) -> (Vec<String>, bool, Vec<Handle<Mesh>>) {
        use bevy::ecs::world::CommandQueue;
        let mut world = World::new();
        let (mut meshes, mut plain, mut built) = (Assets::<Mesh>::default(), Assets::<StandardMaterial>::default(), Assets::<BuiltMaterial>::default());
        let lk = Look::standard();
        let mats = Materials {
            built: built.add(crate::built::material(&lk, 0.5, 0.4)),
            flat: built.add(crate::built::material(&lk, 0.5, 0.9)),
            smoke: plain.add(StandardMaterial::default()),
            evergreen: plain.add(StandardMaterial::default()),
            leaves: plain.add(StandardMaterial::default()),
            bare: plain.add(StandardMaterial::default()),
        };
        let mut queue = CommandQueue::default();
        let handles = spawn(&mut Commands::new(&mut queue, &world), &mut meshes, table, &mats);
        queue.apply(&mut world);
        let parts = list(table);
        let mut q = world.query::<(Entity, &Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>, Option<&MeshMaterial3d<BuiltMaterial>>, Has<bevy::light::NotShadowCaster>, Has<Evergreen>, Option<&Leaves>, Option<&BareLimbs>, Option<&Visibility>)>();
        let mut rows: Vec<_> = q.iter(&world).collect();
        rows.sort_by_key(|r| r.0.index());
        let mut said = Vec::new();
        let mut ids = Vec::new();
        for (_, mesh, plain, built, no_shadow, evergreen, leaves, bare, vis) in rows {
            let at = handles.iter().position(|h| h.id() == mesh.0.id()).unwrap();
            ids.push(mesh.0.id());
            let id = |h: &Handle<StandardMaterial>| plain.is_some_and(|m| m.0.id() == h.id());
            let mat = match built.map(|m| m.0.id()) {
                Some(b) if b == mats.built.id() => "built",
                Some(b) if b == mats.flat.id() => "flat",
                Some(_) => "?",
                None if id(&mats.smoke) => "smoke",
                None if id(&mats.evergreen) => "evergreen",
                None if id(&mats.leaves) => "leaves",
                None if id(&mats.bare) => "bare",
                None => "?",
            };
            let mut s = format!("{} in {mat}", name(table, parts[at]));
            if no_shadow { s += ", no shadow" }
            if evergreen { s += ", Evergreen" }
            if let Some(l) = leaves { s += &format!(", Leaves({})", l.0) }
            if let Some(b) = bare { s += &format!(", BareLimbs({})", b.0) }
            if vis == Some(&Visibility::Hidden) { s += ", hidden" }
            said.push(s);
        }
        let in_order = ids.windows(2).all(|w| w[0] < w[1]);
        (said, in_order, handles)
    }

    #[test]
    fn the_meshes_are_made_and_spawned_as_setup_world_made_them() {
        let (said, in_order, handles) = spawned(MESHES);
        // as `setup_world` spawned them before the table: each group of trees, then the houses
        // and the works in the material winter lies on, the smoke, and what lies flat (the
        // roads, the railway and the yards and lanes last), casting no shadow
        let mut want: Vec<String> = (0..LEAF_GROUPS).flat_map(|g| [format!("trees {g} evergreen in evergreen, Evergreen"), format!("trees {g} leaves in leaves, Leaves({g})"), format!("trees {g} bare limbs in bare, BareLimbs({g}), hidden")]).collect();
        want.extend(["houses in built", "works in built", "smoke in smoke, no shadow", "roads in flat, no shadow", "railway in flat, no shadow", "yards and lanes in flat, no shadow"].map(String::from));
        assert_eq!(said, want);
        // and their handles were made in the same order, so the meshes' numbers are as they were
        assert!(in_order);
        assert_eq!(handles.len(), list(MESHES).len());
    }

    /// The maker of a mesh that exists only in this test: a short reach of
    /// canal, as one triangle, in the middle of the island and further north
    /// each year.
    fn canal(at: &Inputs, _group: usize) -> Builder {
        let mut b = Builder::default();
        let (x, z) = (at.r.w as f32 * 0.5, at.r.h as f32 * 0.5 - at.k as f32);
        b.tri(Vec3::new(x, 0.1, z), Vec3::new(x, 0.1, z + 0.2), Vec3::new(x + 0.2, 0.1, z), [0.3, 0.4, 0.5, 1.0]);
        b
    }

    #[test]
    fn a_row_in_the_table_is_all_a_new_mesh_needs() {
        // the table with one more row: canals, made by a maker of their own, lying flat, left as
        // they are when the year is built again for the camera, and going to the card after the
        // railway; their handle is made after every other
        let mut table = MESHES.to_vec();
        table.push(YearMesh { name: "canals", word: "canals", each_group: false, make: Make::Own(canal), drawn: Drawn::Flat, camera: false, goes: Goes::Before(6), made: 7 });
        let table = &table[..];
        let parts = list(table);
        let at = parts.len() - 1;
        assert_eq!((name(table, parts[at]), parts.len()), ("canals".to_string(), list(MESHES).len() + 1));
        // at the start: a handle and an entity in the yards' material, casting no shadow, after
        // every other, which are made and spawned as they were
        let (said, in_order, handles) = spawned(table);
        assert!(in_order);
        assert_eq!(said.last().unwrap(), "canals in flat, no shadow");
        assert_eq!(said[..at], spawned(MESHES).0[..]);
        // each year: built by its maker from what the year is built from, beside the rest,
        // which are built as they are without it
        let r = crate::world::tests::island();
        let g = crate::world::build_ground(&r);
        let (slots, cen, lk) = (scatter::make_slots(&r, &g), network::centres(&r), Look::standard());
        let lod = scatter::Lod::of_camera((1.0, 0.5), 14.0);
        let sc = build_scenery(table, &r, &g, &slots, &cen, 0, &lk, lod);
        let without = build_scenery(MESHES, &r, &g, &slots, &cen, 0, &lk, lod);
        assert_eq!(sc.meshes.len(), parts.len());
        assert_eq!(sc.meshes[at], canal(&Inputs { r: &r, g: &g, slots: &slots, cen: &cen, k: 0, lk: &lk, lod: &lod, land: &crate::world::year_land(&r, &g, 0) }, 0));
        assert_eq!(sc.meshes[..at], without.meshes[..]);
        // fingerprinted and counted under its own word
        assert_eq!(sums(table, &sc.meshes), format!("{}, canals {:016x}", sums(MESHES, &without.meshes), mesh_sum(&sc.meshes[at])));
        assert_eq!(counts(table, &sc.meshes), format!("{}, canals 3", counts(MESHES, &without.meshes)));
        // queued for the card under its handle after the railway, and kept back from a build
        // for the camera, to be let go of with the trees
        let prints: Vec<(usize, u64)> = sc.meshes.iter().map(print_of).collect();
        let (pieces, _, corners) = queue(table, sc.meshes.clone(), &handles, Some(&prints), true, 0.5);
        let mut want = todays_hand_over(0.5);
        want.insert(6, "canals".to_string());
        assert_eq!(queued(table, &handles, &pieces), want);
        assert_eq!(corners, sc.meshes.iter().map(|b| b.pos.len()).sum::<usize>());
        let (camera, rest, _) = queue(table, sc.meshes.clone(), &handles, None, false, 0.5);
        assert_eq!(queued(table, &handles, &camera), ["houses", "works", "smoke"]);
        assert_eq!(rest.last(), Some(&sc.meshes[at]));
        // the scripted check finds it on screen as it was handed over, and names it when it is not
        let mut on_screen: std::collections::HashMap<_, _> = pieces.iter().map(|p| (p.handle.id(), p.print.unwrap())).collect();
        assert_eq!(compare(table, &sc.meshes, &handles, &on_screen), (prints.iter().map(|p| p.0).sum(), vec![]));
        on_screen.insert(handles[at].id(), (3, 0));
        assert_eq!(compare(table, &sc.meshes, &handles, &on_screen).1, ["canals (3 corners on screen, 3 in the year's own)"]);
        // and a picture of a year half arrived names it in its caption
        assert_eq!(named(table, &handles, &handles[at]), "canals");
    }

    #[test]
    fn the_trains_keep_running_while_their_lines_are_as_they_were() {
        let line = |pts: &[(f32, f32)]| trains::TrainPath { pts: pts.iter().map(|&(x, z)| Vec3::new(x, 0.0, z)).collect(), cum: vec![0.0; pts.len()], len: 1.0 };
        let old = [line(&[(0.0, 0.0), (4.0, 0.0)]), line(&[(4.0, 0.0), (4.0, 6.0)])];
        // the same lines built again (another year, or the same year for another camera)
        assert!(same_lines(&old, &[line(&[(0.0, 0.0), (4.0, 0.0)]), line(&[(4.0, 0.0), (4.0, 6.0)])]));
        // a line more, a line fewer, a line laid elsewhere
        assert!(!same_lines(&old, &old[..1]));
        assert!(!same_lines(&old[..1], &old));
        assert!(!same_lines(&old, &[line(&[(0.0, 0.0), (4.0, 0.0)]), line(&[(4.0, 0.0), (4.0, 7.0)])]));
        assert!(same_lines(&[], &[]));
    }
}
