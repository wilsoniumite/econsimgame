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

use bevy::prelude::*;
use isostate_gen::trees::{in_leaf, LEAF_GROUPS};

use crate::built::BuiltMaterial;
use crate::look::Look;
use crate::meshkit::Builder;
use crate::replay::Replay;
use crate::world::{Ground, YearLand};
use crate::{network, scatter, BareLimbs, Evergreen, Leaves};

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
            going.push(Some(crate::piece(&handles[i], b, prints.map(|p| p[i]))));
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
        let want = crate::print_of(b);
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
                [(_, one)] => crate::mesh_sum(one),
                _ => crate::mesh_sum(&of.iter().fold(Builder::default(), |mut all, (_, b)| {
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
        assert!(sums(MESHES, &meshes).starts_with(&format!("trees {:016x}, houses {:016x}, ", crate::mesh_sum(&trees), crate::mesh_sum(&meshes[3 * LEAF_GROUPS]))));
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
        let sc = crate::build_scenery(table, &r, &g, &slots, &cen, 0, &lk, lod);
        let without = crate::build_scenery(MESHES, &r, &g, &slots, &cen, 0, &lk, lod);
        assert_eq!(sc.meshes.len(), parts.len());
        assert_eq!(sc.meshes[at], canal(&Inputs { r: &r, g: &g, slots: &slots, cen: &cen, k: 0, lk: &lk, lod: &lod, land: &crate::world::year_land(&r, &g, 0) }, 0));
        assert_eq!(sc.meshes[..at], without.meshes[..]);
        // fingerprinted and counted under its own word
        assert_eq!(sums(table, &sc.meshes), format!("{}, canals {:016x}", sums(MESHES, &without.meshes), crate::mesh_sum(&sc.meshes[at])));
        assert_eq!(counts(table, &sc.meshes), format!("{}, canals 3", counts(MESHES, &without.meshes)));
        // queued for the card under its handle after the railway, and kept back from a build
        // for the camera, to be let go of with the trees
        let prints: Vec<(usize, u64)> = sc.meshes.iter().map(crate::print_of).collect();
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
}
