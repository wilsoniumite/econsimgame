//! Tasks: hands and machines in the works. Not in the Python
//! (`docs/rules/tasks.md`).
//!
//! Every hour of a method's work is of one of five classes (hauling, tending,
//! clerking, judging, and presence, which no machine may take), in shares the
//! game's content gives each method (`data/tasks.toml`, read into
//! `Content::tasks`). Within a class the tasks run from the easiest for a
//! machine to the hardest: at task `z` a hand is worth `worth(z)` machine
//! units, spread evenly in the logarithm between two knots (or bent by more).
//!
//! A machine for a task (`[[use]]` in the file) is found island-wide in a
//! year's turn, and from then on, each month, in each town:
//!
//! - **What a unit costs.** `c = ((hurdle + 1/life) / 12 * plant + run +
//!   minders * wage) / (quality * dial)`, the plant and the run at the town's
//!   prices: the return a works must see and the wear on the machine's plant,
//!   its fuel, and the hands who mind it, over the units one machine does.
//! - **The town's target.** The share of the class where the town's wage over
//!   `c` falls on the class's schedule, kept between the use's `from` and
//!   `reach`: every task at which a hand is worth fewer units than the wage
//!   buys is cheaper done by machine.
//! - **The works.** A new works means to hold its town's target. A standing
//!   works moves toward it by `1 / (12 * adopt_years)` of the gap a month,
//!   times how well last month's order for its machines was filled, and falls
//!   no faster than its machines wear out (`f_auto`). The machines' plant,
//!   sized to that and to its capacity, is ordered through a table of its own
//!   (`f_order`), a quarter of what is missing a month, paid from its cash and
//!   by its backers when it is short (`firms::back_firm`).
//! - **A machine works once it stands.** What the plant in place can do this
//!   month is a second share (`f_held`, never above `f_auto`); its hands a
//!   unit are the port's times `sum over classes of share * (1 - held +
//!   minders of the machines held)`, and the machines' run goods join its
//!   inputs a unit, for the share held. A new works holds nothing until its
//!   plant comes.
//! - **A machine short of its run good** idles, in the measure it is short,
//!   and nothing else does: the works' recipe is held to its own inputs, the
//!   hands do the idle machines' part at the labour it took before them, and
//!   the run goods are paid for what was used (`Running`, and the execute step
//!   of `econ::step_month`). The works hires its hands for last month's fill.
//! - **Sites and reviews.** `firms::evaluate_sites`, `firms::review` and
//!   `firms::entry` see the works at its town's target: the machines' plant
//!   for a unit of capacity is part of what the capacity costs, and the run
//!   goods and minders come off the margin.
//!
//! Nothing acts where no works holds any part of any class, so a run in which
//! no machine is found (every run to 1900) is the same run to the byte, and
//! with the switch off the state stays empty.
//!
//! **On the land** (`tasks.land`): where a use names a land sector, each
//! town's sector means to hold a share of each class (`land_auto`), moving
//! toward the town's target for it as a works' does, and holds what its plant
//! in place can do (`land_held`); its units pay their share for
//! work to a mixture of hands and machine units: work wanted is `lam * (Cc /
//! cL)^(1/alpha)` at `cL = hands * w + cost` (`Mix`, `land_month`), and the
//! units buy their machines' run goods and plant out of their revenue, the
//! return on the plant staying in the rent. The land's machines of
//! `machines.rs` stand beside these as they are. For sweeps, a ceiling on
//! what the ground yields (`yield_cap`, from 1950) and a control that holds
//! every use's quality still (`freeze_from`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::mat::Mat;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

/// The classes of task and their machines, embedded at compile time.
pub const TASKS_TOML: &str = include_str!("../../../data/tasks.toml");

/// The number of classes, and their names in the order every table keeps them.
pub const K: usize = 5;
pub const CLASSES: [&str; K] = ["haul", "tend", "clerk", "judge", "presence"];
pub const HAUL: usize = 0;
pub const TEND: usize = 1;
pub const CLERK: usize = 2;
pub const JUDGE: usize = 3;
pub const PRESENCE: usize = 4;

/// The land's sectors as the port's content names them: a use may name one
/// of these even on a content that lacks it.
const KNOWN_SECTORS: [&str; 7] = ["farm", "pasture", "forestry", "quarry", "claypit", "ironmine", "coalmine"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TasksParams {
    /// Machines for tasks are found, priced and bought by the works.
    pub enabled: bool,
    /// Years over which a standing works closes the gap to its town's target
    /// (a twelfth of this many years' part of the gap a month).
    pub adopt_years: f64,
    /// Multiplies every machine's quality: for sweeps.
    pub dial: f64,
    /// The land's sectors that a use names are in its reach: a land unit pays
    /// its share for work to a mixture of hands and machine units ("On the
    /// land"). Left out of the params' file while off, so that a run with it
    /// off writes the first half's file.
    #[serde(skip_serializing_if = "is_false")]
    pub land: bool,
    /// For sweeps: a unit's output may not pass this many times what the same
    /// ground yielded in its best year before 1950 ("The ceiling on yield").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yield_cap: Option<f64>,
    /// For the control: every use's quality is held still from this year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeze_from: Option<i64>,
    /// Machines that reckon take clerks' tasks (`docs/rules/computing.md`):
    /// the uses of the file with `switch = "computing"` may be found. Left out
    /// of the params' file while off, so that a run with it off writes the
    /// file it wrote before the switch was.
    #[serde(skip_serializing_if = "is_false")]
    pub computing: bool,
    /// Models take judging tasks (`docs/rules/computing.md`, "Models"): the
    /// uses of the file with `switch = "models"` may be found. Left out of the
    /// params' file while off, as `computing` is.
    #[serde(skip_serializing_if = "is_false")]
    pub models: bool,
    /// Office machines take the easy quarter of clerks' tasks from 1901
    /// (`docs/rules/tasks.md`, "Office machines"): the uses of the file with
    /// `switch = "office_machines"` may be found. Left out of the params' file
    /// while off, as `computing` is.
    #[serde(skip_serializing_if = "is_false")]
    pub office_machines: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Default for TasksParams {
    /// Off: what the port runs with.
    fn default() -> TasksParams {
        TasksParams { enabled: false, adopt_years: 8.0, dial: 1.0, land: false, yield_cap: None, freeze_from: None, computing: false, models: false, office_machines: false }
    }
}

impl TasksParams {
    /// The game's.
    pub fn game() -> TasksParams {
        TasksParams { enabled: true, land: true, computing: true, models: true, office_machines: true, ..TasksParams::default() }
    }

    /// Whether the switch of this name, which a `[[use]]` may name (`switch`),
    /// is on. Its names are [`USE_SWITCHES`].
    pub fn switch_on(&self, name: &str) -> bool {
        match name {
            "computing" => self.computing,
            "models" => self.models,
            "office_machines" => self.office_machines,
            _ => false,
        }
    }
}

/// The switches of `TasksParams` a `[[use]]` of the file may stand behind
/// (`switch = "computing"`): a use behind one that is off is never found.
pub const USE_SWITCHES: [&str; 3] = ["computing", "models", "office_machines"];

/// The year from which the ceiling on yield is in force, and before which the
/// best years it reads are kept.
pub const CEILING_FROM: i64 = 1950;

// ---------------------------------------------------------------- the content

/// One machine for a task, read from a `[[use]]` of the file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Use {
    pub name: String,
    /// Its class, and the part of it it may hold.
    pub class: usize,
    pub from: f64,
    pub reach: f64,
    /// Used in the works.
    pub on_works: bool,
    /// Used on each of the land's sectors, by its place in the content (read
    /// only under `land`).
    pub on_sector: Vec<bool>,
    /// Goods bought once for one machine, and a month to keep it working.
    pub plant: Vec<(usize, f64)>,
    pub run: Vec<(usize, f64)>,
    /// Worker-months a month of minding one machine.
    pub minders: f64,
    /// Years a machine stands.
    pub life: f64,
    /// Quality grows by `pace` a year from the year found, to `cap`.
    pub pace: f64,
    pub cap: f64,
    /// Nothing is found before this year.
    pub found_after: i64,
    /// Found at the first year's turn when the island has made this much of
    /// each good, counted from the start year.
    pub discover: Vec<(usize, f64)>,
    /// `discover` is read per head (`discover_per_head`): what the island has
    /// made of each good, over its people at the year's end.
    pub discover_per_head: bool,
    /// Multiplies everything one machine takes.
    pub scale: f64,
    /// The use this one takes the place of (`replaces`): where this one is
    /// laid, the use named no longer reaches the places this one is used.
    pub replaces: Option<String>,
    /// The switch of `TasksParams` it stands behind (`switch`, one of
    /// [`USE_SWITCHES`]): while that is off it is never found, and has no
    /// cost, no part, no order and no plant. None: it stands behind none.
    pub switch: Option<String>,
}

impl Use {
    /// Whether the use may act under `P`: it stands behind no switch, or
    /// behind one that is on.
    pub fn allowed(&self, P: &Params) -> bool {
        self.switch.as_deref().is_none_or(|s| P.tasks.switch_on(s))
    }

    /// A use with no plant is rented (models, `docs/rules/computing.md`): its
    /// whole cost is its run goods, bought each month. It has nothing to wait
    /// for, so what is meant is held from the first month; it orders nothing,
    /// adds nothing to a book, wears nothing (its `life` is not read), and may
    /// be dropped as fast as a works likes.
    pub fn rented(&self) -> bool {
        self.plant.is_empty()
    }
}

/// The classes of task of a content (`Content::tasks`): empty in the port's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TaskTable {
    /// Each open class's schedule as knots (task, machine units a hand is
    /// worth there); empty for a closed class.
    pub spread: [Vec<(f64, f64)>; K],
    pub closed: [bool; K],
    /// Each method's labour as shares of the classes, by its place in the
    /// content; all nothing for a method the file does not name.
    pub method: Vec<[f64; K]>,
    /// Each land sector's, and the carters' (read from the second half on).
    pub sector: Vec<[f64; K]>,
    pub carting: [f64; K],
    pub uses: Vec<Use>,
    /// Every good some use's plant names, in the content's order: the columns
    /// of the works' machine orders.
    pub plant_goods: Vec<usize>,
}

impl TaskTable {
    pub fn is_empty(&self) -> bool {
        self.method.is_empty()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    tasks: RawHead,
    #[serde(default)]
    kind: Vec<RawKind>,
    #[serde(default)]
    sector: Vec<RawSector>,
    #[serde(default, rename = "use")]
    uses: Vec<RawUse>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHead {
    classes: Vec<String>,
    closed: Vec<String>,
    spread: BTreeMap<String, Vec<[f64; 2]>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawKind {
    #[allow(dead_code)]
    name: String,
    tasks: [f64; K],
    methods: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSector {
    names: Vec<String>,
    tasks: [f64; K],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawUse {
    name: String,
    class: String,
    on: Vec<String>,
    from: f64,
    reach: f64,
    plant: BTreeMap<String, f64>,
    run: BTreeMap<String, f64>,
    minders: f64,
    life: f64,
    pace: f64,
    cap: f64,
    found_after: i64,
    discover: BTreeMap<String, f64>,
    /// The thresholds of `discover` are per head of the island's people.
    #[serde(default)]
    discover_per_head: bool,
    scale: f64,
    /// Where this use is laid, the use of this name no longer reaches the
    /// places this one is `on`.
    #[serde(default)]
    replaces: Option<String>,
    /// The switch of `TasksParams` the use stands behind, by name.
    #[serde(default)]
    switch: Option<String>,
}

/// The table of `text` for content `C`: methods and sectors by name (a name the
/// content lacks, as a method of a later stage of the tree, is passed over),
/// and the uses by their goods: a use that names a good the content lacks (a
/// good of a later stage of the tree) is not laid, as the overlay drops an
/// input of a later stage. A use laid with `replaces` takes the places it is
/// `on` from the use it names.
pub fn table_from(C: &Content, text: &str) -> Result<TaskTable, String> {
    let raw: RawFile = toml::from_str(text).map_err(|e| format!("tasks: {e}"))?;
    if raw.tasks.classes != CLASSES {
        return Err(format!("tasks: the classes must be {CLASSES:?}"));
    }
    let class = |n: &str| CLASSES.iter().position(|c| *c == n).ok_or_else(|| format!("tasks: no class {n:?}"));
    let mut t = TaskTable { method: vec![[0.0; K]; C.M], sector: vec![[0.0; K]; C.NS], ..TaskTable::default() };
    for n in &raw.tasks.closed {
        t.closed[class(n)?] = true;
    }
    for (n, knots) in &raw.tasks.spread {
        let k = class(n)?;
        if t.closed[k] {
            return Err(format!("tasks: the class {n:?} is closed and has no spread"));
        }
        if knots.len() < 2 || knots.windows(2).any(|w| !(w[1][0] > w[0][0] && w[1][1] >= w[0][1])) || knots.iter().any(|k| !(k[1] > 0.0)) {
            return Err(format!("tasks: the spread of {n:?} must be two knots or more, rising, every worth above nothing"));
        }
        if knots[0][0] != 0.0 || knots[knots.len() - 1][0] != 1.0 {
            return Err(format!("tasks: the spread of {n:?} must run from task 0 to task 1"));
        }
        t.spread[k] = knots.iter().map(|k| (k[0], k[1])).collect();
    }
    let sums = |what: &str, s: &[f64; K]| -> Result<(), String> {
        let total: f64 = s.iter().sum();
        if (total - 1.0).abs() > 1e-9 || s.iter().any(|&v| v < 0.0) {
            return Err(format!("tasks: the shares of {what} add up to {total}, not one"));
        }
        Ok(())
    };
    let mut named: BTreeMap<&str, ()> = BTreeMap::new();
    for kd in &raw.kind {
        sums(&kd.name, &kd.tasks)?;
        for m in &kd.methods {
            if named.insert(m.as_str(), ()).is_some() {
                return Err(format!("tasks: the method {m:?} is in two kinds"));
            }
            if let Some(i) = C.method_name.iter().position(|n| n == m) {
                t.method[i] = kd.tasks;
            }
        }
    }
    for s in &raw.sector {
        sums(&s.names.join(", "), &s.tasks)?;
        for n in &s.names {
            if n == "carting" {
                t.carting = s.tasks;
            } else if let Some(i) = C.sector_name.iter().position(|x| x == n) {
                t.sector[i] = s.tasks;
            }
        }
    }
    // a use's goods by name; None where the content lacks one of them
    let goods = |m: &BTreeMap<String, f64>| -> Option<Vec<(usize, f64)>> { m.iter().map(|(n, &v)| C.good_index.get(n).map(|&g| (g, v))).collect() };
    let mut replaces: Vec<(usize, String)> = Vec::new();
    for u in &raw.uses {
        if let Some(r) = &u.replaces {
            if r == &u.name || !raw.uses.iter().any(|v| &v.name == r) {
                return Err(format!("tasks: {:?} replaces {r:?}, which is not another use of the file", u.name));
            }
        }
        if let Some(s) = &u.switch {
            if !USE_SWITCHES.contains(&s.as_str()) {
                return Err(format!("tasks: {:?} stands behind the switch {s:?}, which is not a switch of the rule (one of {USE_SWITCHES:?})", u.name));
            }
        }
        let (Some(plant), Some(run), Some(discover)) = (goods(&u.plant), goods(&u.run), goods(&u.discover)) else {
            continue;
        };
        let k = class(&u.class)?;
        if t.closed[k] {
            return Err(format!("tasks: {:?} works in {:?}, which is closed to machines", u.name, u.class));
        }
        if !(0.0 <= u.from && u.from < u.reach && u.reach <= 1.0) || !(u.life > 0.0) || !(u.scale > 0.0) || !(u.cap >= 1.0) {
            return Err(format!("tasks: {:?} needs 0 <= from < reach <= 1, a life and a scale above nothing and a cap of one or more", u.name));
        }
        // where it is used: the works, a land sector the content has (one it lacks
        // is passed over, as a method of a later stage is), or the carters, whom
        // no machine reaches yet and who are passed over too
        let mut on_sector = vec![false; C.NS];
        for o in &u.on {
            if let Some(i) = C.sector_name.iter().position(|x| x == o) {
                on_sector[i] = true;
            } else if o != "works" && o != "carting" && !KNOWN_SECTORS.contains(&o.as_str()) {
                return Err(format!("tasks: {:?} is used on {o:?}, which is neither the works, the carters nor a land sector", u.name));
            }
        }
        t.uses.push(Use {
            name: u.name.clone(),
            class: k,
            from: u.from,
            reach: u.reach,
            on_works: u.on.iter().any(|o| o == "works"),
            on_sector,
            plant,
            run,
            minders: u.minders,
            life: u.life,
            pace: u.pace,
            cap: u.cap,
            found_after: u.found_after,
            discover,
            discover_per_head: u.discover_per_head,
            scale: u.scale,
            replaces: u.replaces.clone(),
            switch: u.switch.clone(),
        });
        if let Some(r) = &u.replaces {
            replaces.push((t.uses.len() - 1, r.clone()));
        }
    }
    // the use replaced no longer reaches the places its replacement is on
    // (nothing where it is not laid itself)
    for (by, name) in replaces {
        if let Some(v) = t.uses.iter().position(|x| x.name == name) {
            let (on_works, on_sector) = (t.uses[by].on_works, t.uses[by].on_sector.clone());
            let gone = &mut t.uses[v];
            gone.on_works &= !on_works;
            for (b, &o) in gone.on_sector.iter_mut().zip(&on_sector) {
                *b &= !o;
            }
        }
    }
    let mut pg: Vec<usize> = t.uses.iter().flat_map(|u| u.plant.iter().map(|&(g, _)| g)).collect();
    pg.sort_unstable();
    pg.dedup();
    t.plant_goods = pg;
    Ok(t)
}

/// The game's table, from the file built in, laid on `C` (`content::game_content`,
/// and again on the content the tree's overlay makes from it).
pub fn fill(C: &mut Content) {
    C.tasks = table_from(C, TASKS_TOML).expect("embedded data/tasks.toml is valid");
}

// ---------------------------------------------------------------- the schedule

/// Machine units a hand is worth at task `z` of a class with these knots.
pub fn worth(kn: &[(f64, f64)], z: f64) -> f64 {
    let z = z.clamp(0.0, 1.0);
    for w in kn.windows(2) {
        let ((z0, v0), (z1, v1)) = (w[0], w[1]);
        if z <= z1 {
            return v0 * (v1 / v0).powf((z - z0) / (z1 - z0));
        }
    }
    kn[kn.len() - 1].1
}

/// Machine units that hold the tasks from `a` to `b` for an hour of the
/// class: the schedule summed over that part. For two knots `lo` and `hi`,
/// `lo * ((hi/lo)^b - (hi/lo)^a) / ln(hi/lo)`.
pub fn units(kn: &[(f64, f64)], a: f64, b: f64) -> f64 {
    let (a, b) = (a.clamp(0.0, 1.0), b.clamp(0.0, 1.0));
    if b <= a {
        return 0.0;
    }
    let mut sum = 0.0;
    for w in kn.windows(2) {
        let ((z0, v0), (z1, v1)) = (w[0], w[1]);
        let (s, e) = (a.max(z0), b.min(z1));
        if e <= s {
            continue;
        }
        let (d, r) = (z1 - z0, (v1 / v0).ln());
        sum += if r.abs() < 1e-12 { v0 * (e - s) } else { v0 * d / r * ((r * (e - z0) / d).exp() - (r * (s - z0) / d).exp()) };
    }
    sum
}

/// The share of the class at which a hand is worth `ratio` units: every task
/// below it is cheaper done by a machine whose unit costs a wage over `ratio`.
pub fn share_at(kn: &[(f64, f64)], ratio: f64) -> f64 {
    if !(ratio > kn[0].1) {
        return 0.0;
    }
    for w in kn.windows(2) {
        let ((z0, v0), (z1, v1)) = (w[0], w[1]);
        if ratio <= v1 {
            return if v1 > v0 { z0 + (z1 - z0) * (ratio / v0).ln() / (v1 / v0).ln() } else { z1 };
        }
    }
    1.0
}

/// The top of a part beginning at `a` that holds `u` units an hour.
fn top_for_units(kn: &[(f64, f64)], a: f64, u: f64) -> f64 {
    let (mut lo, mut hi) = (a, 1.0);
    if units(kn, a, hi) <= u {
        return hi;
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if units(kn, a, mid) < u {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

// ---------------------------------------------------------------- the state

/// One year's machines in the works, for the facts and the gauges. Hours are
/// worker-months a month, the year's mean.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TasksYear {
    pub year: i64,
    /// Each class's work in the works (by hand or by machine), the part of it
    /// machines in place did, and the part the works meant them to do (the
    /// difference waits on machines not yet delivered).
    pub work: [f64; K],
    pub held: [f64; K],
    pub aim: [f64; K],
    /// Hands at each class's tasks, and minding the machines.
    pub hands: [f64; K],
    pub minders: f64,
    /// Work-weighted means: each town's target, the wage over the cheapest
    /// unit's cost (the waterline), and the wage over the machine's cost at
    /// the last task each works holds (the margin's gap, 1 at rest).
    pub target: [f64; K],
    pub waterline: [f64; K],
    pub margin_gap: [f64; K],
    /// The machines' book at the year's end, and what wore out in the year.
    pub book: f64,
    pub wear: f64,
    /// By town: each class's work and the part machines did, and the wage.
    pub town_work: Vec<[f64; K]>,
    pub town_held: Vec<[f64; K]>,
    pub town_wage: Vec<f64>,
    /// The land, by sector: each class's work, the part machines in place
    /// did, and the part meant (empty unless `land`).
    pub land_work: Vec<[f64; K]>,
    pub land_held: Vec<[f64; K]>,
    pub land_aim: Vec<[f64; K]>,
    /// The return on the land's machines over the year (`hurdle` a year on
    /// their plant at each month's prices), and their plant's worth at the
    /// year's end.
    pub land_return: f64,
    pub land_book: f64,
    /// The year's output at the ceiling on yield, and all the land's, at the
    /// towns' prices (while `yield_cap` is set).
    pub capped_value: f64,
    pub land_value: f64,
    /// By good: what the machines in place ran on in the works and on the land
    /// over the year (the year's total, in the good's units, as the books'
    /// `made` is).
    pub run_used: Vec<f64>,
    /// By use: the share of its class it would hold, weighted by each town's
    /// work in the class in the works, over the towns and months in which it
    /// would hold any (NaN where it held none anywhere): what a town that can
    /// run it gives it; and the part of the class's work done in those towns
    /// and months.
    pub use_run: Vec<[f64; 2]>,
    /// The farms over the year, for the facts of the land's twentieth century
    /// (docs/rules/machines.md, "Nitrogen and breeding"): their output and
    /// hands a month, and the output of the farms that keep nitrogen and
    /// breeding (each unit's output times the share of its town's farms whose
    /// improvement stands).
    pub farm_out: f64,
    pub farm_hands: f64,
    pub nitrogen_out: f64,
    pub breeding_out: f64,
}

/// The year so far.
#[derive(Clone, Debug, Default, PartialEq)]
struct Acc {
    months: f64,
    work: [f64; K],
    held: [f64; K],
    aim: [f64; K],
    hands: [f64; K],
    minders: f64,
    target: [[f64; 2]; K],
    water: [[f64; 2]; K],
    gap: [[f64; 2]; K],
    wear: f64,
    town_work: Vec<[f64; K]>,
    town_held: Vec<[f64; K]>,
    town_wage: Vec<f64>,
    land_work: Vec<[f64; K]>,
    land_held: Vec<[f64; K]>,
    land_aim: Vec<[f64; K]>,
    land_return: f64,
    capped_value: f64,
    land_value: f64,
    run_used: Vec<f64>,
    use_run: Vec<[f64; 2]>,
    /// The farms: output, hands, and output with nitrogen and with breeding.
    farm: [f64; 4],
}

/// What the rule keeps (`st.tasks`). Empty while it is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TasksState {
    /// Each use's year found (-1 until then) and quality.
    pub found: Vec<i64>,
    pub quality: Vec<f64>,
    /// By town and use: a unit's cost this month, and the share of its class
    /// it would hold (NaN where it is not known or not used in works).
    pub cost: Vec<Vec<f64>>,
    pub share: Vec<Vec<f64>>,
    /// By town and class: the share machines would hold at this month's prices.
    pub target: Vec<[f64; K]>,
    /// By works: the share of each class it means to hold (moving toward its
    /// town's target, and what its order for machines is sized to), and their
    /// book.
    pub f_auto: Vec<[f64; K]>,
    pub f_mach: Vec<f64>,
    /// By works: the share of each class its machines in place can do this
    /// month, never above `f_auto` ("A machine works once it stands"; set in
    /// `works_month` from `f_plant`). The month's hands and run goods read it.
    pub f_held: Vec<[f64; K]>,
    /// By works and plant good (`TaskTable::plant_goods`): the machines'
    /// plant in place, and what is on order for it. A table of its own: the
    /// works' plant order is wiped when no capacity is on order.
    pub f_plant: Vec<Vec<f64>>,
    pub f_order: Vec<Vec<f64>>,
    /// By works: the share of last month's machine order that was delivered,
    /// and its hands a unit over what they would be by hand.
    pub f_fill: Vec<f64>,
    pub f_factor: Vec<f64>,
    /// By works: how well its machines' run goods were filled last month (1
    /// where it has none), which its hands for this month are hired on ("A
    /// machine short of its run good").
    pub f_run_fill: Vec<f64>,
    /// By town and class, this month: the uses that would hold it, cheapest
    /// first, each with the part it holds (`parts`).
    parts: Vec<[Vec<(usize, f64, f64)>; K]>,
    // ---- the land ("On the land"; empty unless `land`)
    /// By town and land sector: the share of each class machines would hold
    /// (from the uses that reach the sector), the uses that would hold it, the
    /// share the sector means to hold (what its order for plant is sized to),
    /// and the share its plant in place can do this month (never above
    /// `land_auto`; set in `land_month`).
    pub land_target: Vec<Vec<[f64; K]>>,
    land_parts: Vec<Vec<[Vec<(usize, f64, f64)>; K]>>,
    pub land_auto: Vec<Vec<[f64; K]>>,
    pub land_held: Vec<Vec<[f64; K]>>,
    /// By town, land sector and plant good (`TaskTable::plant_goods`): the
    /// machines' plant in place, and the share of last month's order for it
    /// that was delivered.
    pub land_plant: Vec<Vec<Vec<f64>>>,
    pub land_fill: Vec<Vec<f64>>,
    /// By town and land sector: how well the machines' run goods were filled
    /// last month (1 where they have none), which the month's hands are hired
    /// on ("A machine short of its run good").
    pub land_run_fill: Vec<Vec<f64>>,
    /// This month's work (worker-months a month, by hand or by machine) by
    /// town and sector (`t * NS + s`), for the year's sums.
    pub land_work: Vec<f64>,
    // ---- the ceiling on yield (empty unless `yield_cap` is set)
    /// By sector and cell (`s * N + c`): the best year's output a month for
    /// each share of the cell, of the years before `CEILING_FROM`; this year's
    /// output and shares so far; and whether the unit stood at the ceiling
    /// last month.
    pub best: Vec<f64>,
    yield_acc: Vec<[f64; 2]>,
    pub at_ceiling: Vec<bool>,
    /// The year now running, set at each year's turn (0 before the first).
    pub year_now: i64,
    /// By use, for a use of the land alone: the share of its class the towns
    /// would give it at the end of the year it was found, each town weighted
    /// by the work of the sectors it reaches (NaN until then).
    pub land_share_found: Vec<f64>,
    /// Every year's record.
    pub years: Vec<TasksYear>,
    acc: Acc,
}

fn active(st: &State, P: &Params) -> bool {
    P.tasks.enabled && st.tasks.found.iter().any(|&y| y >= 0)
}

/// Every works' tables as long as the works' table is used.
fn fit(st: &mut State, C: &Content) {
    let n = st.F_count;
    let np = C.tasks.plant_goods.len();
    let s = &mut st.tasks;
    if s.f_auto.len() < n {
        s.f_auto.resize(n, [0.0; K]);
        s.f_held.resize(n, [0.0; K]);
        s.f_mach.resize(n, 0.0);
        s.f_plant.resize(n, vec![0.0; np]);
        s.f_order.resize(n, vec![0.0; np]);
        s.f_fill.resize(n, 1.0);
        s.f_factor.resize(n, 1.0);
        s.f_run_fill.resize(n, 1.0);
    }
}

/// A unit's effective quality: what one machine does, over everything it
/// takes scaled.
fn qeff(st: &State, C: &Content, P: &Params, u: usize) -> f64 {
    st.tasks.quality[u] * P.tasks.dial / C.tasks.uses[u].scale
}

/// The uses town `t` would put to class `k`, cheapest first, each with the
/// part it holds: the cheapest from its `from` to its share, the next what
/// lies above that up to its own, and so on. A use that holds nothing is
/// listed with a part of no width, so that a works holding more than the
/// town would can say whose machines those are.
fn parts(st: &State, t: usize, k: usize) -> &[(usize, f64, f64)] {
    st.tasks.parts.get(t).map_or(&[], |p| &p[k][..])
}

/// `parts`, worked out from this month's costs and shares (`end_month`), of
/// the uses `used` admits: those used in works, or on a land sector.
fn work_out_parts(st: &State, C: &Content, t: usize, k: usize, used: &dyn Fn(&Use) -> bool) -> Vec<(usize, f64, f64)> {
    let s = &st.tasks;
    let mut us: Vec<usize> = (0..C.tasks.uses.len()).filter(|&u| C.tasks.uses[u].class == k && used(&C.tasks.uses[u]) && s.cost.get(t).is_some_and(|c| c[u].is_finite())).collect();
    us.sort_by(|&a, &b| s.cost[t][a].total_cmp(&s.cost[t][b]));
    let mut top = 0.0f64;
    us.into_iter()
        .map(|u| {
            let a = top.max(C.tasks.uses[u].from);
            let b = s.share[t][u].max(a);
            top = top.max(b);
            (u, a, b)
        })
        .collect()
}

/// What machines do for an hour of a method's labour when they hold `auto`
/// of each class in town `t`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Adjust {
    /// Hands an hour, over an hour by hand: the work left to hands and the
    /// minders.
    pub factor: f64,
    /// Goods a month to run the machines, and their plant, for an hour of
    /// the method's labour (a worker-month a month).
    pub run: Vec<(usize, f64)>,
    pub plant: Vec<(usize, f64)>,
    /// The part of each class held.
    pub held: [f64; K],
}

/// `Adjust` for method `m` in town `t` with `auto` held; None where nothing is
/// held, so that the caller leaves its sums as they were.
pub fn adjust(st: &State, C: &Content, P: &Params, t: usize, m: usize, auto: &[f64; K]) -> Option<Adjust> {
    let sh = C.tasks.method.get(m)?;
    let (mach, held) = hold(st, C, P, sh, auto, &|k| parts(st, t, k))?;
    let tb = &C.tasks;
    let mut factor: f64 = (0..K).map(|k| sh[k] * (1.0 - held[k])).sum();
    let mut run: BTreeMap<usize, f64> = BTreeMap::new();
    let mut plant: BTreeMap<usize, f64> = BTreeMap::new();
    for (u, &n) in mach.iter().enumerate().filter(|(_, n)| **n > 0.0) {
        let us = &tb.uses[u];
        factor += n * us.minders;
        for &(g, v) in &us.run {
            *run.entry(g).or_default() += n * v;
        }
        for &(g, v) in &us.plant {
            *plant.entry(g).or_default() += n * v;
        }
    }
    Some(Adjust { factor, run: run.into_iter().collect(), plant: plant.into_iter().collect(), held })
}

/// The use whose machines hold what a works holds above its town's top of
/// class `ps` (cheapest first), where it holds `x`: the dearest use, as it
/// always has been, unless `x` is beyond its reach; then the cheapest whose
/// reach takes in `x`, or else the one that reaches highest. Some only where
/// that is not the dearest. (Office machines reach a quarter of clerking: once
/// office computing is cheaper they are the dearest use, and a works a little
/// above its town's top would otherwise hold its hardest clerking with them,
/// at some forty times the plant.) A class with one use is as it was.
fn above_top(tb: &TaskTable, ps: &[(usize, f64, f64)], x: f64) -> Option<usize> {
    let &(last, _, top) = ps.last()?;
    if x <= top || tb.uses[last].reach >= x {
        return None;
    }
    let u = ps.iter().find(|p| tb.uses[p.0].reach >= x).or_else(|| ps.iter().max_by(|a, b| tb.uses[a.0].reach.total_cmp(&tb.uses[b.0].reach)))?.0;
    (u != last).then_some(u)
}

/// The machines of each use for an hour of work whose classes are `sh`, and
/// the part of each class they hold, when `auto` of each class is held and
/// `ps_of(k)` are the uses that would hold class `k`; None where nothing is
/// held. (The works' and the land's alike.)
fn hold<'a>(st: &State, C: &Content, P: &Params, sh: &[f64; K], auto: &[f64; K], ps_of: &dyn Fn(usize) -> &'a [(usize, f64, f64)]) -> Option<(Vec<f64>, [f64; K])> {
    if auto.iter().all(|&x| x <= 0.0) || sh.iter().all(|&s| s == 0.0) {
        return None;
    }
    let tb = &C.tasks;
    let mut mach = vec![0.0; tb.uses.len()]; // machines of each use for an hour of labour
    let mut held = [0.0; K];
    for k in 0..K {
        let x = auto[k];
        if x <= 0.0 || sh[k] == 0.0 {
            continue;
        }
        let ps = ps_of(k);
        let n = ps.len();
        let over = above_top(tb, ps, x);
        for (j, &(u, a, b)) in ps.iter().enumerate() {
            // the dearest use holds whatever the works holds above the town's top,
            // unless that is beyond its reach (`above_top`)
            let b = if j + 1 == n && over.is_none() { b.max(x) } else { b };
            let e = b.min(x);
            if e > a {
                held[k] += e - a;
                mach[u] += sh[k] * units(&tb.spread[k], a, e) / qeff(st, C, P, u);
            }
        }
        if let (Some(u), Some(&(_, _, top))) = (over, ps.last()) {
            held[k] += x - top;
            mach[u] += sh[k] * units(&tb.spread[k], top, x) / qeff(st, C, P, u);
        }
    }
    if held.iter().all(|&h| h == 0.0) {
        return None;
    }
    Some((mach, held))
}

/// The share of each class the plant in place (`plant`, by plant good) can do
/// for `hours` of work whose classes are `sh` (worker-months a month by hand),
/// where `auto` is what is meant to be held and `ps_of(k)` are the uses that
/// would hold class `k` ("A machine works once it stands"). The parts are
/// taken as `hold` takes them, class by class and the cheapest use first:
/// the machines of a use in place are the least, over its plant goods, of the
/// plant not yet given to a part below over the plant one machine takes; a
/// part they cannot fill is held to the top those machines reach
/// (`top_for_units`), and nothing above it. Never above `auto`; nothing at
/// all where `auto` is nothing, so a run with no use found reads nothing.
/// Machines whose run goods are short this month still stand, and are held;
/// they idle in the execute step ("A machine short of its run good"). A use
/// with no plant (rented) takes none: the least over its empty set of plant
/// goods is no limit, so its part is held whole from the first month.
#[allow(clippy::too_many_arguments)]
fn held_by_plant<'a>(st: &State, C: &Content, P: &Params, sh: &[f64; K], auto: &[f64; K], hours: f64, plant: &[f64], ps_of: &dyn Fn(usize) -> &'a [(usize, f64, f64)]) -> [f64; K] {
    let mut out = [0.0; K];
    if !(hours > 0.0) || auto.iter().all(|&x| x <= 0.0) {
        return out;
    }
    let tb = &C.tasks;
    let col = |g: usize| tb.plant_goods.iter().position(|&p| p == g);
    let mut left: Vec<f64> = tb.plant_goods.iter().enumerate().map(|(j, _)| plant.get(j).copied().unwrap_or(0.0).max(0.0)).collect();
    for k in 0..K {
        let x = auto[k];
        if x <= 0.0 || sh[k] == 0.0 {
            continue;
        }
        let ps = ps_of(k);
        let n = ps.len();
        // as `hold`: the dearest use holds whatever is meant above the town's top,
        // unless that is beyond its reach (`above_top`)
        let over = above_top(tb, ps, x);
        let mut spans: Vec<(usize, f64, f64)> = ps.iter().enumerate().map(|(j, &(u, a, b))| (u, a, if j + 1 == n && over.is_none() { b.max(x) } else { b })).collect();
        if let (Some(u), Some(&(_, _, top))) = (over, ps.last()) {
            spans.push((u, top, x));
        }
        for (u, a, b) in spans {
            let e = b.min(x);
            if e <= a {
                continue;
            }
            let us = &tb.uses[u];
            // machines for one machine unit an hour of the class, and for the part
            let per = hours * sh[k] / qeff(st, C, P, u);
            let want = per * units(&tb.spread[k], a, e);
            // the least over the plant goods it takes: infinite (no limit) where it takes none
            let have = us.plant.iter().filter(|p| p.1 > 0.0).filter_map(|&(g, v)| col(g).map(|c| left[c] / v)).fold(f64::INFINITY, f64::min);
            let got = have.min(want).max(0.0);
            for &(g, v) in &us.plant {
                if let Some(c) = col(g) {
                    left[c] = (left[c] - got * v).max(0.0);
                }
            }
            if got >= want {
                out[k] = e;
            } else {
                if got > 0.0 {
                    out[k] = top_for_units(&tb.spread[k], a, got / per).min(e);
                }
                break;
            }
        }
    }
    out
}

/// `adjust` for method `m` at town `t`'s target: what a new works there would
/// be built with (`firms::evaluate_sites`, `entry`, `review`). None while the
/// rule is off or nothing would be held.
pub fn at_target(st: &State, C: &Content, P: &Params, t: usize, m: usize) -> Option<Adjust> {
    if !active(st, P) {
        return None;
    }
    let target = *st.tasks.target.get(t)?;
    adjust(st, C, P, t, m, &target)
}

/// The machines' plant for a unit of capacity of a method whose hands a unit
/// by hand are `l`, at prices `price`.
pub fn plant_cost(a: &Adjust, l: f64, price: &dyn Fn(usize) -> f64) -> f64 {
    a.plant.iter().map(|&(g, v)| l * v * price(g)).sum()
}

// ---------------------------------------------------------------- the month

/// The works at work this month (`econ::step_month`, after their recipes are
/// made). First, the share its machines in place can do (`f_held`, from
/// `f_plant` at its capacity and its hands a unit by hand). Where they hold
/// some part of a class, its hands a unit `l` are cut to the work left and
/// the minders, and the machines' run goods join its inputs a unit `a`; and
/// its order for their plant is what its capacity needs at the share it means
/// to hold (`f_auto`) less what it has. `fi` are the works, `cap` their
/// capacity.
pub fn works_month(st: &mut State, C: &Content, P: &Params, fi: &[usize], cap: &[f64], a: &mut Mat, l: &mut [f64]) -> Vec<Running> {
    if !active(st, P) {
        return Vec::new();
    }
    fit(st, C);
    let pg = C.tasks.plant_goods.clone();
    let mut running = vec![Running::default(); fi.len()];
    for (i, &f) in fi.iter().enumerate() {
        let (t, m) = (st.f_town[f], st.f_method[f]);
        let auto = st.tasks.f_auto[f];
        let base = l[i];
        // what stands: the machines in place, at this month's capacity and hands a unit
        let sh = C.tasks.method.get(m).copied().unwrap_or([0.0; K]);
        let held = held_by_plant(st, C, P, &sh, &auto, cap[i] * base, &st.tasks.f_plant[f], &|k| parts(st, t, k));
        let now = adjust(st, C, P, t, m, &held);
        let aim = adjust(st, C, P, t, m, &auto);
        let s = &mut st.tasks;
        s.f_held[f] = held;
        match now {
            Some(adj) => {
                let r = &mut running[i];
                r.share = (0..K).map(|k| sh[k] * adj.held[k]).sum();
                r.factor = adj.factor;
                r.planned = s.f_run_fill[f];
                // hands a unit: the work left to hands and the minders, and the
                // machines' part that last month's run goods left idle, which
                // hands do at the labour it took before the machines
                let factor = adj.factor + r.share * (1.0 - r.planned);
                l[i] = base * factor;
                s.f_factor[f] = factor;
                for &(g, v) in &adj.run {
                    a[(i, g)] += base * v;
                    r.run.push((g, base * v));
                }
            }
            None => s.f_factor[f] = 1.0,
        }
        // the order: the machines it means to have
        match aim {
            Some(adj) => {
                for (j, &g) in pg.iter().enumerate() {
                    let need = adj.plant.iter().find(|p| p.0 == g).map_or(0.0, |p| cap[i] * base * p.1);
                    s.f_order[f][j] = (need - s.f_plant[f][j]).max(0.0);
                }
            }
            None => s.f_order[f].iter_mut().for_each(|v| *v = 0.0),
        }
    }
    running
}

/// What a works' machines in place ask of this month (`works_month`), for the
/// execute step of `econ::step_month` ("A machine short of its run good").
/// The default, with no run goods, is a works whose machines hold nothing.
#[derive(Clone, Debug, Default)]
pub struct Running {
    /// The share of the works' hours by hand that its machines in place do:
    /// the sum over classes of the class's share times the part held.
    pub share: f64,
    /// Its hands a unit over its hands a unit by hand, with every machine
    /// running: the work left to hands, and the minders.
    pub factor: f64,
    /// The machines' run goods a unit of output, as `works_month` added them
    /// to the works' inputs.
    pub run: Vec<(usize, f64)>,
    /// The fill of the run goods its hands were hired for: last month's.
    pub planned: f64,
}

impl Running {
    /// What the machines' run goods add to the works' inputs of good `g`.
    pub fn of(&self, g: usize) -> f64 {
        self.run.iter().find(|r| r.0 == g).map_or(0.0, |r| r.1)
    }

    /// The share of what its recipe allows that the works makes when its
    /// machines' run goods are filled `fr`. The machines stand idle in the
    /// measure their run goods are short, and the hands do their part at the
    /// labour it took before them: a unit then takes `factor + share * (1 -
    /// fr)` of its hands by hand. The works hired its hands for last month's
    /// fill, so it makes all its recipe allows where the fill is no worse than
    /// that, and as much as those hands can where it is worse.
    pub fn output(&self, fr: f64) -> f64 {
        let hired = self.factor + self.share * (1.0 - self.planned);
        let now = self.factor + self.share * (1.0 - fr);
        if now > 0.0 { (hired / now).min(1.0) } else { 1.0 }
    }
}

/// The month's requests for machines' plant: a quarter of each works' order.
/// A works whose cash will not pay for it is backed (`firms::back_firm`).
pub struct Orders {
    pub works: Vec<usize>,
    pub towns: Vec<usize>,
    /// (rows, G): goods asked for.
    pub req: Mat,
}

pub fn orders(st: &mut State, C: &Content, P: &Params) -> Option<Orders> {
    if !active(st, P) {
        return None;
    }
    let pg = C.tasks.plant_goods.clone();
    let rows: Vec<usize> = (0..st.F_count.min(st.tasks.f_order.len())).filter(|&f| st.f_alive[f] && st.tasks.f_order[f].iter().any(|&v| v > 0.0)).collect();
    if rows.is_empty() {
        return None;
    }
    let mut req = Mat::zeros(rows.len(), C.G);
    let mut towns = Vec::with_capacity(rows.len());
    for (r, &f) in rows.iter().enumerate() {
        let t = st.f_town[f];
        towns.push(t);
        let mut value = 0.0;
        for (j, &g) in pg.iter().enumerate() {
            req[(r, g)] = 0.25 * st.tasks.f_order[f][j];
            value += req[(r, g)] * st.price[(t, g)];
        }
        let short = value - st.f_cash[f].max(0.0);
        if short > 0.0 {
            st.f_cash[f] += crate::firms::back_firm(st, C, P, f, short);
        }
    }
    Some(Orders { works: rows, towns, req })
}

/// What the market gave each order (`got`, row by row, as the works' plant
/// orders are served): the works pay for it, and it goes into their machines
/// and their book. Returns what each paid.
pub fn deliver(st: &mut State, C: &Content, o: &Orders, got: &Mat) -> Vec<f64> {
    let pg = &C.tasks.plant_goods;
    let mut pay = vec![0.0; o.works.len()];
    for (r, &f) in o.works.iter().enumerate() {
        let t = o.towns[r];
        let (mut asked, mut paid) = (0.0, 0.0);
        for (j, &g) in pg.iter().enumerate() {
            let x = got[(r, g)];
            asked += o.req[(r, g)] * st.price[(t, g)];
            paid += x * st.price[(t, g)];
            st.tasks.f_plant[f][j] += x;
            st.tasks.f_order[f][j] = (st.tasks.f_order[f][j] - x).max(0.0);
        }
        st.f_cash[f] -= paid;
        st.tasks.f_mach[f] += paid;
        st.tasks.f_fill[f] = if asked > 1e-12 { (paid / asked).clamp(0.0, 1.0) } else { 1.0 };
        pay[r] = paid;
    }
    pay
}

/// What a works keeps back from its dividend for its machines on order: the
/// order at its town's prices (`works::kept_for_order`).
pub fn order_value(st: &State, C: &Content, f: usize) -> f64 {
    let Some(o) = st.tasks.f_order.get(f) else { return 0.0 };
    let t = st.f_town[f];
    C.tasks.plant_goods.iter().enumerate().map(|(j, &g)| o[j] * st.price[(t, g)]).sum()
}

/// A works founded (`firms::create_firm`): it means to hold its town's target,
/// and holds nothing until its machines' plant comes ("A machine works once it
/// stands"): its first months are worked by hand at its recipe, but for the
/// part a rented use holds (one with no plant, as models), which it holds from
/// its first month.
pub fn opened(st: &mut State, C: &Content, P: &Params, f: usize) {
    if !active(st, P) {
        return;
    }
    fit(st, C);
    let t = st.f_town[f];
    let target = st.tasks.target.get(t).copied().unwrap_or([0.0; K]);
    let s = &mut st.tasks;
    s.f_auto[f] = target;
    s.f_held[f] = [0.0; K];
    s.f_mach[f] = 0.0;
    s.f_plant[f].iter_mut().for_each(|v| *v = 0.0);
    s.f_order[f].iter_mut().for_each(|v| *v = 0.0);
    s.f_fill[f] = 1.0;
    s.f_factor[f] = 1.0;
    s.f_run_fill[f] = 1.0;
}

// ---------------------------------------------------------------- the land

/// Machines reach the land: the rule and its land switch are both on.
pub fn land_on(P: &Params) -> bool {
    P.tasks.enabled && P.tasks.land
}

/// What machines do for an hour of a land sector's work in a town, where they
/// hold some part of a class (`mix_at`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mix {
    /// Hands an hour of work: the work left to hands, and the minders.
    pub hands: f64,
    /// What the machine units cost an hour of work beside their minders: the
    /// return a works must see and the wear on their plant, and their run
    /// goods, at the prices given.
    pub cost: f64,
    /// Goods a month to run them, and their plant, for an hour of work.
    pub run: Vec<(usize, f64)>,
    pub plant: Vec<(usize, f64)>,
    /// The part of an hour's work the machines do.
    pub machine_work: f64,
    /// The fill of the machines' run goods the hands were hired for (last
    /// month's), whose shortfall `hands` already counts as work by hand (1
    /// outside the month's hiring).
    pub planned: f64,
}

impl Mix {
    /// The share of the work hired that is done when the machines' run goods
    /// are filled `fr` (`Running::output`, for the land): the machines stand
    /// idle in the measure their run goods are short and the hands do their
    /// part, so the work hired is all done where the fill is no worse than the
    /// one the hands were hired for, and what those hands can do where it is
    /// worse.
    pub fn done(&self, fr: f64) -> f64 {
        let now = self.hands + self.machine_work * (self.planned - fr);
        if now > 0.0 { (self.hands / now).min(1.0) } else { 1.0 }
    }

    /// The cost of an hour's work done the cheapest way at wage `w`: the
    /// hands' and the minders' pay, and the machine units' cost.
    pub fn cost_of_work(&self, w: f64) -> f64 {
        self.hands * w + self.cost
    }
}

/// The mixture on sector `s` of town `t` at the part it means to hold
/// (`land_auto`), at prices `price`; None where that is nothing. (What the
/// sector's bids and a prospect's review read: `land_cost`.)
pub fn mix_at(st: &State, C: &Content, P: &Params, t: usize, s: usize, price: &dyn Fn(usize) -> f64) -> Option<Mix> {
    let auto = st.tasks.land_auto.get(t)?.get(s)?;
    mix_with(st, C, P, t, s, auto, price)
}

/// The mixture on sector `s` of town `t` where machines hold `auto` of each
/// class; None where they hold nothing.
fn mix_with(st: &State, C: &Content, P: &Params, t: usize, s: usize, auto: &[f64; K], price: &dyn Fn(usize) -> f64) -> Option<Mix> {
    let sh = C.tasks.sector.get(s)?;
    let (mach, held) = hold(st, C, P, sh, auto, &|k| land_parts(st, t, s, k))?;
    let tb = &C.tasks;
    let mut hands: f64 = (0..K).map(|k| sh[k] * (1.0 - held[k])).sum();
    let machine_work: f64 = (0..K).map(|k| sh[k] * held[k]).sum();
    let mut cost = 0.0;
    let mut run: BTreeMap<usize, f64> = BTreeMap::new();
    let mut plant: BTreeMap<usize, f64> = BTreeMap::new();
    for (u, &n) in mach.iter().enumerate().filter(|(_, n)| **n > 0.0) {
        let us = &tb.uses[u];
        hands += n * us.minders;
        let pv: f64 = us.plant.iter().map(|&(g, v)| v * price(g)).sum();
        let rv: f64 = us.run.iter().map(|&(g, v)| v * price(g)).sum();
        cost += n * ((P.hurdle + 1.0 / us.life) / 12.0 * pv + rv);
        for &(g, v) in &us.run {
            *run.entry(g).or_default() += n * v;
        }
        for &(g, v) in &us.plant {
            *plant.entry(g).or_default() += n * v;
        }
    }
    Some(Mix { hands, cost, run: run.into_iter().collect(), plant: plant.into_iter().collect(), machine_work, planned: 1.0 })
}

/// The cost of an hour of sector `s`'s work in town `t` at wage `w` and
/// prices `price`, done the cheapest way: `w` itself wherever no machine
/// holds any part of it (`land::sector_bids` and `mines::review` read it in
/// place of the wage).
pub fn land_cost(st: &State, C: &Content, P: &Params, t: usize, s: usize, w: f64, price: &dyn Fn(usize) -> f64) -> f64 {
    if !land_on(P) || !active(st, P) {
        return w;
    }
    match mix_at(st, C, P, t, s, price) {
        Some(m) => m.cost_of_work(w),
        None => w,
    }
}

/// The ceiling on yield is in force this year.
fn ceiling_live(st: &State, P: &Params) -> bool {
    P.tasks.yield_cap.is_some() && st.tasks.year_now >= CEILING_FROM && !st.tasks.best.is_empty()
}

/// The land's month (`econ::step_month`, before the hands are hired). None
/// where no machine holds any part of any land sector and no ceiling is in
/// force, so that the month is the first half's to the byte.
pub struct LandMonth {
    /// By town and sector (`t * NS + s`): the mixture, where the machines in
    /// place hold some part of the sector's work (`land_held`).
    pub mix: Vec<Option<Mix>>,
    /// By town and sector: the plant an hour of work takes at the share the
    /// sector means to hold (`land_auto`), where that is something: what its
    /// order for plant is sized to.
    pub aim: Vec<Option<Vec<(usize, f64)>>>,
    /// By unit: hands an hour of work (1 where no machine holds any part).
    pub hands: Vec<f64>,
    /// By unit: the ceiling on its output this month (infinite where none),
    /// whether it stood at the ceiling last month, and if so the work the
    /// ceiling needs.
    pub ceiling: Vec<f64>,
    pub capped: Vec<bool>,
    pub cap_work: Vec<f64>,
}

impl LandMonth {
    pub fn mix_of(&self, t: usize, s: usize, NS: usize) -> Option<&Mix> {
        self.mix[t * NS + s].as_ref()
    }
}

/// The land's month: what the plant in place on each town's sectors can do
/// (`land_held`, from `land_plant` over last month's work), the mixture there
/// at prices `price`, the plant an hour at the share meant, and each unit's
/// ceiling. The unit's output is `A * f * lam * (L / lam)^(1 - alpha)` with
/// `L` its work.
#[allow(clippy::too_many_arguments)]
pub fn land_month(st: &mut State, C: &Content, P: &Params, price: &Mat, ex: &[usize], sec: &[usize], et: &[usize], A: &[f64], f: &[f64], lam: &[f64], alpha: &[f64]) -> Option<LandMonth> {
    let (T, NS, N) = (st.T_max, C.NS, st.N);
    let mut mix: Vec<Option<Mix>> = vec![None; T * NS];
    let mut aim: Vec<Option<Vec<(usize, f64)>>> = vec![None; T * NS];
    let mut any = false;
    if land_on(P) && active(st, P) && !st.tasks.land_auto.is_empty() {
        fit_land(st, C);
        for t in 0..T {
            if !st.town_active[t] {
                continue;
            }
            for s in 0..NS {
                let sh = C.tasks.sector.get(s).copied().unwrap_or([0.0; K]);
                let auto = st.tasks.land_auto[t][s];
                let hours = st.tasks.land_work.get(t * NS + s).copied().unwrap_or(0.0);
                let held = held_by_plant(st, C, P, &sh, &auto, hours, &st.tasks.land_plant[t][s], &|k| land_parts(st, t, s, k));
                st.tasks.land_held[t][s] = held;
                if let Some(m) = mix_with(st, C, P, t, s, &auto, &|g| price[(t, g)]) {
                    aim[t * NS + s] = Some(m.plant);
                    any = true;
                }
                // the hands are hired for last month's fill of the machines' run
                // goods: the part of the machines' work it left idle is done by
                // hand, at the labour it took before them ("A machine short of
                // its run good", as in the works)
                let planned = st.tasks.land_run_fill.get(t).and_then(|r| r.get(s)).copied().unwrap_or(1.0);
                mix[t * NS + s] = mix_with(st, C, P, t, s, &held, &|g| price[(t, g)]).map(|mut m| {
                    m.planned = planned;
                    m.hands += m.machine_work * (1.0 - planned);
                    m
                });
            }
        }
    }
    let st = &*st;
    let cap = ceiling_live(st, P);
    if !any && !cap {
        return None;
    }
    let NE = ex.len();
    let hands: Vec<f64> = (0..NE).map(|u| mix[et[u] * NS + sec[u]].as_ref().map_or(1.0, |m| m.hands)).collect();
    let (mut ceiling, mut capped, mut cap_work) = (vec![f64::INFINITY; NE], vec![false; NE], vec![0.0; NE]);
    if cap {
        let k = P.tasks.yield_cap.unwrap_or(f64::INFINITY);
        // ground with no record before 1950 (a working sunk since, a field
        // ploughed since) is held to its sector's mean best yield
        let mut mean = vec![[0.0f64; 2]; NS];
        for (i, &b) in st.tasks.best.iter().enumerate() {
            if b > 0.0 {
                mean[i / N][0] += b;
                mean[i / N][1] += 1.0;
            }
        }
        for u in 0..NE {
            let i = sec[u] * N + ex[u];
            let b = if st.tasks.best[i] > 0.0 { st.tasks.best[i] } else if mean[sec[u]][1] > 0.0 { mean[sec[u]][0] / mean[sec[u]][1] } else { 0.0 };
            if b > 0.0 {
                ceiling[u] = k * b * lam[u];
                capped[u] = st.tasks.at_ceiling.get(i) == Some(&true);
                let yf = A[u] * f[u] * lam[u];
                if capped[u] && yf > 0.0 {
                    cap_work[u] = lam[u] * (ceiling[u] / yf).powf(1.0 / (1.0 - alpha[u]));
                }
            }
        }
    }
    Some(LandMonth { mix, aim, hands, ceiling, capped, cap_work })
}

/// The land machines' goods asked for in a month, by town and sector
/// (`t * NS + s`, rows of the matrices), and the work each was asked for.
pub struct LandBuy {
    /// Work (worker-months a month) by town and sector, and by unit.
    pub work: Vec<f64>,
    pub unit_work: Vec<f64>,
    /// Run goods asked for (the day's trade), and a quarter of the order for
    /// plant (served as the works' plant orders are).
    pub run: Mat,
    pub plant: Mat,
}

/// What the land's machines ask for this month, once the hands are hired:
/// the run goods of the machines in place for the work, and a quarter of
/// what their plant lacks for it at the share the sector means to hold. None
/// where no sector means to hold any part of its work.
pub fn land_orders(st: &State, C: &Content, lm: &LandMonth, et: &[usize], sec: &[usize], Lex: &[f64]) -> Option<LandBuy> {
    if lm.aim.iter().all(|m| m.is_none()) {
        return None;
    }
    let (T, NS, G) = (st.T_max, C.NS, C.G);
    let unit_work: Vec<f64> = (0..Lex.len()).map(|u| Lex[u] / lm.hands[u]).collect();
    let mut work = vec![0.0; T * NS];
    for u in 0..Lex.len() {
        work[et[u] * NS + sec[u]] += unit_work[u];
    }
    let (mut run, mut plant) = (Mat::zeros(T * NS, G), Mat::zeros(T * NS, G));
    for t in 0..T.min(st.tasks.land_plant.len()) {
        for s in 0..NS {
            let (r, w) = (t * NS + s, work[t * NS + s]);
            let Some(aim) = &lm.aim[r] else { continue };
            if w <= 0.0 {
                continue;
            }
            if let Some(m) = lm.mix_of(t, s, NS) {
                for &(g, v) in &m.run {
                    run[(r, g)] += w * v;
                }
            }
            for (j, &g) in C.tasks.plant_goods.iter().enumerate() {
                let need = aim.iter().find(|p| p.0 == g).map_or(0.0, |p| w * p.1);
                plant[(r, g)] = 0.25 * (need - st.tasks.land_plant[t][s][j]).max(0.0);
            }
        }
    }
    Some(LandBuy { work, unit_work, run, plant })
}

/// The land's plant delivered (`got`, by town and sector, of what `buy` asked
/// for): into the plant in place, and the share of the order filled.
pub fn land_deliver(st: &mut State, C: &Content, buy: &LandBuy, got: &Mat) {
    let NS = C.NS;
    fit_land(st, C);
    for t in 0..st.T_max {
        for s in 0..NS {
            let r = t * NS + s;
            let (mut asked, mut paid) = (0.0, 0.0);
            for (j, &g) in C.tasks.plant_goods.iter().enumerate() {
                asked += buy.plant[(r, g)] * st.price[(t, g)];
                paid += got[(r, g)] * st.price[(t, g)];
                st.tasks.land_plant[t][s][j] += got[(r, g)];
            }
            st.tasks.land_fill[t][s] = if asked > 1e-12 { (paid / asked).clamp(0.0, 1.0) } else { 1.0 };
        }
    }
}

/// The land's month, noted (`econ::step_month`, once the crop is in): the work
/// by town and sector, and, while the ceiling is in force, which units stood
/// at it (`at`, by unit) and the value of what was raised at it and in all.
pub fn land_note(st: &mut State, C: &Content, work: Option<&[f64]>, at: Option<(&[usize], &[usize], &[bool], f64, f64)>) {
    match work {
        Some(w) => {
            st.tasks.land_work.clear();
            st.tasks.land_work.extend_from_slice(w);
        }
        None => st.tasks.land_work.iter_mut().for_each(|v| *v = 0.0),
    }
    if let Some((ex, sec, flag, capped, all)) = at {
        let N = st.N;
        if st.tasks.at_ceiling.len() != C.NS * N {
            st.tasks.at_ceiling = vec![false; C.NS * N];
        }
        st.tasks.at_ceiling.iter_mut().for_each(|v| *v = false);
        for u in 0..ex.len() {
            st.tasks.at_ceiling[sec[u] * N + ex[u]] = flag[u];
        }
        st.tasks.acc.capped_value += capped;
        st.tasks.acc.land_value += all;
    }
}

/// Each unit's output this month, for the ceiling's record of the best years
/// before 1950 (nothing unless `yield_cap` is set, and nothing from 1950).
pub fn note_yields(st: &mut State, C: &Content, P: &Params, ex: &[usize], sec: &[usize], Y: &[f64], lam: &[f64]) {
    if P.tasks.yield_cap.is_none() || st.tasks.year_now >= CEILING_FROM {
        return;
    }
    let N = st.N;
    if st.tasks.yield_acc.len() != C.NS * N {
        st.tasks.yield_acc = vec![[0.0; 2]; C.NS * N];
        st.tasks.best = vec![0.0; C.NS * N];
    }
    for u in 0..ex.len() {
        let a = &mut st.tasks.yield_acc[sec[u] * N + ex[u]];
        a[0] += Y[u];
        a[1] += lam[u];
    }
}

/// The month's end (`econ::step_month`, last): each town's target at the
/// month's wage and prices; each works' machines moved toward it and worn;
/// the year's sums.
pub fn end_month(st: &mut State, C: &Content, P: &Params) {
    if !P.tasks.enabled || C.tasks.is_empty() {
        return;
    }
    let T = st.T_max;
    let NS = C.NS;
    let tb = &C.tasks;
    let nu = tb.uses.len();
    let on = active(st, P);
    let land = land_on(P);
    // ---- what a unit costs, and the town's target
    if on {
        let s = &mut st.tasks;
        if s.target.len() != T {
            s.target = vec![[0.0; K]; T];
            s.cost = vec![vec![f64::NAN; nu]; T];
            s.share = vec![vec![f64::NAN; nu]; T];
            s.parts = vec![Default::default(); T];
        }
        if land && s.land_target.len() != T {
            s.land_target = vec![vec![[0.0; K]; NS]; T];
            s.land_parts = vec![vec![Default::default(); NS]; T];
        }
        for t in 0..T {
            for u in 0..nu {
                let us = &tb.uses[u];
                let w = st.wage[t];
                let used = us.on_works || (land && us.on_sector.iter().any(|&b| b));
                // (a use behind a switch thrown off by hand after its find is priced
                // nowhere, and its works fall back as their machines wear)
                let known = st.town_active[t] && used && st.tasks.found[u] >= 0 && w > 0.0 && us.allowed(P);
                let (c, x) = if known {
                    let p = |g: usize| st.price[(t, g)];
                    let plant: f64 = us.plant.iter().map(|&(g, v)| v * p(g)).sum();
                    let run: f64 = us.run.iter().map(|&(g, v)| v * p(g)).sum();
                    let c = ((P.hurdle + 1.0 / us.life) / 12.0 * plant + run + us.minders * w) / qeff(st, C, P, u);
                    (c, share_at(&tb.spread[us.class], w / c).clamp(us.from, us.reach))
                } else {
                    (f64::NAN, f64::NAN)
                };
                st.tasks.cost[t][u] = c;
                st.tasks.share[t][u] = x;
            }
            for k in 0..K {
                let ps = work_out_parts(st, C, t, k, &|u| u.on_works);
                st.tasks.target[t][k] = ps.iter().map(|p| p.2).fold(0.0, f64::max);
                st.tasks.parts[t][k] = ps;
            }
            // the land: each sector from the uses that reach it
            if land {
                for s in 0..NS {
                    for k in 0..K {
                        let ps = work_out_parts(st, C, t, k, &|u| u.on_sector.get(s) == Some(&true));
                        st.tasks.land_target[t][s][k] = ps.iter().map(|p| p.2).fold(0.0, f64::max);
                        st.tasks.land_parts[t][s][k] = ps;
                    }
                }
            }
        }
    }
    fit(st, C);
    // ---- the works: toward the target, gated by the order's fill; the machines wear
    let step = 1.0 / (12.0 * P.tasks.adopt_years.max(1e-9));
    let mut acc = std::mem::take(&mut st.tasks.acc);
    if acc.town_work.len() != T {
        acc.town_work.resize(T, [0.0; K]);
        acc.town_held.resize(T, [0.0; K]);
        acc.town_wage.resize(T, 0.0);
    }
    if acc.run_used.len() != C.G {
        acc.run_used.resize(C.G, 0.0);
    }
    if acc.use_run.len() != nu {
        acc.use_run.resize(nu, [0.0; 2]);
    }
    acc.months += 1.0;
    for t in 0..T {
        acc.town_wage[t] += st.wage[t];
    }
    // this month's work in the works by town and class, for `use_run`
    let mut month_work = vec![[0.0; K]; if on { T } else { 0 }];
    // the farms' month, for the facts: output, hands, and the output of those
    // whose nitrogen and breeding stand (machines.rs's stock, by town)
    if let Some(farm) = C.sector_name.iter().position(|n| n == "farm") {
        let stock = |t: usize, slot: usize| {
            let ms = &st.machine_stock;
            if ms.cols == NS * crate::machines::SLOTS && t < ms.rows { ms[(t, farm * crate::machines::SLOTS + slot)].max(0.0) } else { 0.0 }
        };
        for u in (0..st.ex_sector.len().min(st.ex_out.len())).filter(|&u| st.ex_sector[u] == farm) {
            let (t, y) = (st.ex_town[u], st.ex_out[u]);
            acc.farm[0] += y;
            acc.farm[1] += st.ex_labor.get(u).copied().unwrap_or(0.0);
            acc.farm[2] += y * stock(t, crate::machines::FARM_NITROGEN_SLOT);
            acc.farm[3] += y * stock(t, crate::machines::FARM_BREEDING_SLOT);
        }
    }
    for f in 0..st.F_count {
        if !st.f_alive[f] {
            continue;
        }
        let (t, m) = (st.f_town[f], st.f_method[f]);
        let sh = tb.method.get(m).copied().unwrap_or([0.0; K]);
        // what the works did this month, by class: its hands over its hands a unit's
        // factor; what its machines in place did, and what it meant them to
        let base = st.f_labor[f] / st.tasks.f_factor[f].max(1e-12);
        let adj = if on { adjust(st, C, P, t, m, &st.tasks.f_held[f]) } else { None };
        let held = adj.as_ref().map_or([0.0; K], |a| a.held);
        let aim = if on { hold(st, C, P, &sh, &st.tasks.f_auto[f], &|k| parts(st, t, k)).map_or([0.0; K], |h| h.1) } else { [0.0; K] };
        for k in 0..K {
            let wk = base * sh[k];
            if wk <= 0.0 {
                continue;
            }
            acc.work[k] += wk;
            acc.held[k] += wk * held[k];
            acc.aim[k] += wk * aim[k];
            acc.hands[k] += wk * (1.0 - held[k]);
            acc.town_work[t][k] += wk;
            acc.town_held[t][k] += wk * held[k];
            if on {
                month_work[t][k] += wk;
                acc.target[k][0] += wk * st.tasks.target[t][k];
                acc.target[k][1] += wk;
                let ps = parts(st, t, k);
                if let Some(&(u0, _, _)) = ps.first() {
                    acc.water[k][0] += wk * st.wage[t] / st.tasks.cost[t][u0];
                    acc.water[k][1] += wk;
                }
                let x = st.tasks.f_held[f][k];
                if x > 0.0 {
                    // the use holding the works' last task done by machine
                    let u = above_top(tb, ps, x).or_else(|| ps.iter().rev().find(|p| p.1 < x).or(ps.last()).map(|p| p.0));
                    if let Some(u) = u {
                        let gap = st.wage[t] / (st.tasks.cost[t][u] * worth(&tb.spread[k], x));
                        if gap.is_finite() {
                            acc.gap[k][0] += wk * gap;
                            acc.gap[k][1] += wk;
                        }
                    }
                }
            }
        }
        if let Some(a) = &adj {
            acc.minders += base * (a.factor - (0..K).map(|k| sh[k] * (1.0 - a.held[k])).sum::<f64>());
            for &(g, v) in &a.run {
                acc.run_used[g] += base * v;
            }
        }
        if !on {
            continue;
        }
        // toward the target
        for k in 0..K {
            let target = st.tasks.target[t][k];
            let x = st.tasks.f_auto[f][k];
            st.tasks.f_auto[f][k] = toward(tb, parts(st, t, k), k, x, target, step, st.tasks.f_fill[f]);
            // what stands is never more than is meant
            st.tasks.f_held[f][k] = st.tasks.f_held[f][k].min(st.tasks.f_auto[f][k]);
        }
        // the machines in place wear by their life's twelfth a month
        if st.tasks.f_mach[f] > 0.0 || st.tasks.f_plant[f].iter().any(|&v| v > 0.0) {
            let wear = 1.0 / (12.0 * life_of(C, f, st));
            acc.wear += st.tasks.f_mach[f] * wear;
            st.tasks.f_mach[f] *= 1.0 - wear;
            st.tasks.f_plant[f].iter_mut().for_each(|v| *v *= 1.0 - wear);
        }
    }
    // what each use would hold in the towns that give it any part of its class
    if on {
        for t in 0..T {
            for (u, us) in tb.uses.iter().enumerate() {
                let (x, w) = (st.tasks.share[t][u], month_work[t][us.class]);
                if x > 0.0 && w > 0.0 {
                    acc.use_run[u][0] += x * w;
                    acc.use_run[u][1] += w;
                }
            }
        }
    }
    // ---- the land: each town's sectors toward their targets, the plant worn
    if land && on {
        land_end_month(st, C, P, &mut acc, step);
    }
    st.tasks.acc = acc;
}

/// The land's tables as long as the towns' and sectors'.
fn fit_land(st: &mut State, C: &Content) {
    let (T, NS, np) = (st.T_max, C.NS, C.tasks.plant_goods.len());
    let s = &mut st.tasks;
    if s.land_auto.len() != T {
        s.land_auto = vec![vec![[0.0; K]; NS]; T];
        s.land_plant = vec![vec![vec![0.0; np]; NS]; T];
        s.land_fill = vec![vec![1.0; NS]; T];
    }
    if s.land_held.len() != T {
        s.land_held = vec![vec![[0.0; K]; NS]; T];
    }
    if s.land_run_fill.len() != T {
        s.land_run_fill = vec![vec![1.0; NS]; T];
    }
}

/// The uses that would hold class `k` on sector `s` of town `t`, cheapest first.
fn land_parts(st: &State, t: usize, s: usize, k: usize) -> &[(usize, f64, f64)] {
    st.tasks.land_parts.get(t).and_then(|r| r.get(s)).map_or(&[], |p| &p[k][..])
}

/// The life of the machines on sector `s`: the longest of the uses that reach
/// it and have plant to wear.
fn land_life(C: &Content, s: usize) -> f64 {
    let sh = C.tasks.sector.get(s).copied().unwrap_or([0.0; K]);
    C.tasks.uses.iter().filter(|u| u.on_sector.get(s) == Some(&true) && sh[u.class] > 0.0 && !u.rented()).map(|u| u.life).fold(f64::NAN, f64::max).max(1.0)
}

/// The land's month's end (`end_month`): the year's sums at the part held this
/// month; each town's sectors moved toward their target as a works is (gated
/// by the fill of their order for plant, down no faster than wear); the plant
/// worn, and the return on it reckoned at the month's prices.
fn land_end_month(st: &mut State, C: &Content, P: &Params, acc: &mut Acc, step: f64) {
    let (T, NS) = (st.T_max, C.NS);
    let tb = &C.tasks;
    fit_land(st, C);
    if acc.land_work.len() != NS {
        acc.land_work.resize(NS, [0.0; K]);
        acc.land_held.resize(NS, [0.0; K]);
        acc.land_aim.resize(NS, [0.0; K]);
    }
    for t in 0..T {
        if !st.town_active[t] {
            continue;
        }
        for s in 0..NS {
            let sh = tb.sector.get(s).copied().unwrap_or([0.0; K]);
            let work = st.tasks.land_work.get(t * NS + s).copied().unwrap_or(0.0);
            if work > 0.0 {
                // what the plant in place did this month, and what was meant
                let now = hold(st, C, P, &sh, &st.tasks.land_held[t][s], &|k| land_parts(st, t, s, k));
                if let Some((mach, _)) = &now {
                    for (u, &n) in mach.iter().enumerate().filter(|(_, n)| **n > 0.0) {
                        for &(g, v) in &tb.uses[u].run {
                            acc.run_used[g] += work * n * v;
                        }
                    }
                }
                let held = now.map_or([0.0; K], |h| h.1);
                let aim = hold(st, C, P, &sh, &st.tasks.land_auto[t][s], &|k| land_parts(st, t, s, k)).map_or([0.0; K], |h| h.1);
                for k in 0..K {
                    acc.land_work[s][k] += work * sh[k];
                    acc.land_held[s][k] += work * sh[k] * held[k];
                    acc.land_aim[s][k] += work * sh[k] * aim[k];
                }
            }
            for k in 0..K {
                let (x, target) = (st.tasks.land_auto[t][s][k], st.tasks.land_target[t][s][k]);
                st.tasks.land_auto[t][s][k] = toward(tb, land_parts(st, t, s, k), k, x, target, step, st.tasks.land_fill[t][s]);
                st.tasks.land_held[t][s][k] = st.tasks.land_held[t][s][k].min(st.tasks.land_auto[t][s][k]);
            }
            let plant = &mut st.tasks.land_plant[t][s];
            if plant.iter().any(|&v| v > 0.0) {
                let value: f64 = tb.plant_goods.iter().enumerate().map(|(j, &g)| plant[j] * st.price[(t, g)]).sum();
                acc.land_return += P.hurdle / 12.0 * value;
                let wear = 1.0 / (12.0 * land_life(C, s));
                plant.iter_mut().for_each(|v| *v *= 1.0 - wear);
            }
        }
    }
}

/// The part of class `k` held a month on, from `x`, where the target is
/// `target` and `ps` are the uses that would hold it: up by `step` of the gap
/// times the fill of the last order for the machines; down no faster than the
/// machines wear, to where a life's twelfth fewer units hold it. A rented use
/// (no plant) has nothing to wear out, so where one holds the top of what is
/// held its part may go at once: the share falls to the target, or to the
/// foot of that part where a use with plant holds the tasks below it.
fn toward(tb: &TaskTable, ps: &[(usize, f64, f64)], k: usize, x: f64, target: f64, step: f64, fill: f64) -> f64 {
    if target > x {
        x + step * (target - x) * fill
    } else if target < x {
        // (above the town's top, the use `above_top` names holds from that top)
        let (u, a0) = match (above_top(tb, ps, x), ps.last()) {
            (Some(u), Some(&(_, _, top))) => (u, top),
            _ => ps.iter().rev().find(|p| p.1 < x).or(ps.last()).map_or((usize::MAX, 0.0), |p| (p.0, p.1)),
        };
        let floor = if u == usize::MAX {
            target
        } else if tb.uses[u].rented() {
            a0
        } else {
            let wear = 1.0 / (12.0 * tb.uses[u].life);
            top_for_units(&tb.spread[k], a0, units(&tb.spread[k], a0, x) * (1.0 - wear))
        };
        target.max(floor)
    } else {
        x
    }
}

/// The life of a works' machines: the longest of the uses some class of its
/// method could use in the works and that have plant to wear.
fn life_of(C: &Content, f: usize, st: &State) -> f64 {
    let sh = C.tasks.method.get(st.f_method[f]).copied().unwrap_or([0.0; K]);
    C.tasks.uses.iter().filter(|u| u.on_works && sh[u.class] > 0.0 && !u.rented()).map(|u| u.life).fold(f64::NAN, f64::max).max(1.0)
}

// ---------------------------------------------------------------- the year

/// The year's end (`sim::end_year`, after the books close and before the
/// gauges read them): the year's record; then what is found for the next
/// year, and each use's quality in it.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64) {
    // the ceiling on yield: the year's output for each share of each cell,
    // kept where it is the best before 1950
    if P.tasks.yield_cap.is_some() {
        if year < CEILING_FROM {
            for (b, a) in st.tasks.best.iter_mut().zip(st.tasks.yield_acc.iter_mut()) {
                if a[1] > 0.0 {
                    *b = b.max(a[0] / a[1]);
                }
                *a = [0.0; 2];
            }
        }
        st.tasks.year_now = year + 1;
    }
    if !P.tasks.enabled || C.tasks.is_empty() {
        return;
    }
    let nu = C.tasks.uses.len();
    if st.tasks.found.len() != nu {
        st.tasks.found = vec![-1; nu];
        st.tasks.quality = vec![1.0; nu];
    }
    let acc = std::mem::take(&mut st.tasks.acc);
    let n = acc.months.max(1.0);
    let mean2 = |v: &[[f64; 2]; K]| std::array::from_fn(|k| if v[k][1] > 0.0 { v[k][0] / v[k][1] } else { f64::NAN });
    let book: f64 = (0..st.F_count.min(st.tasks.f_mach.len())).filter(|&f| st.f_alive[f]).map(|f| st.tasks.f_mach[f]).sum();
    st.tasks.years.push(TasksYear {
        year,
        work: acc.work.map(|v| v / n),
        held: acc.held.map(|v| v / n),
        aim: acc.aim.map(|v| v / n),
        hands: acc.hands.map(|v| v / n),
        minders: acc.minders / n,
        target: mean2(&acc.target),
        waterline: mean2(&acc.water),
        margin_gap: mean2(&acc.gap),
        book,
        wear: acc.wear,
        town_work: acc.town_work.iter().map(|r| r.map(|v| v / n)).collect(),
        town_held: acc.town_held.iter().map(|r| r.map(|v| v / n)).collect(),
        town_wage: acc.town_wage.iter().map(|v| v / n).collect(),
        land_work: acc.land_work.iter().map(|r| r.map(|v| v / n)).collect(),
        land_held: acc.land_held.iter().map(|r| r.map(|v| v / n)).collect(),
        land_aim: acc.land_aim.iter().map(|r| r.map(|v| v / n)).collect(),
        land_return: acc.land_return,
        land_book: land_book(st, C),
        capped_value: acc.capped_value,
        land_value: acc.land_value,
        run_used: acc.run_used.clone(),
        use_run: C.tasks.uses.iter().zip(&acc.use_run).map(|(u, a)| [if a[1] > 0.0 { a[0] / a[1] } else { f64::NAN }, ratio(a[1], acc.work[u.class])]).collect(),
        farm_out: acc.farm[0] / n,
        farm_hands: acc.farm[1] / n,
        nitrogen_out: acc.farm[2] / n,
        breeding_out: acc.farm[3] / n,
    });
    // a use of the land alone, in the year it was found: the towns' share for it
    if st.tasks.land_share_found.len() != nu {
        st.tasks.land_share_found = vec![f64::NAN; nu];
    }
    let NS = C.NS;
    for u in 0..nu {
        let us = &C.tasks.uses[u];
        if st.tasks.found[u] != year || us.on_works || st.tasks.share.len() < st.T_max {
            continue;
        }
        let (mut a, mut b) = (0.0, 0.0);
        for t in (0..st.T_max).filter(|&t| st.town_active[t]) {
            let work: f64 = (0..NS).filter(|&s| us.on_sector[s]).map(|s| st.tasks.land_work.get(t * NS + s).copied().unwrap_or(0.0)).sum();
            let x = st.tasks.share[t][u];
            if work > 0.0 && x.is_finite() {
                a += work * x;
                b += work;
            }
        }
        st.tasks.land_share_found[u] = ratio(a, b);
    }
    // found at this turn of the year, for the next
    let next = year + 1;
    // the island's people at the year's end, for a threshold read per head
    let people: f64 = st.pop.iter().sum::<f64>().max(1.0);
    for u in 0..nu {
        let us = &C.tasks.uses[u];
        if st.tasks.found[u] < 0 {
            // a use behind a switch that is off is never found
            let made = |g: usize| if us.discover_per_head { st.cumprod[g] / people } else { st.cumprod[g] };
            if us.allowed(P) && next >= us.found_after && us.discover.iter().all(|&(g, x)| made(g) >= x) {
                st.tasks.found[u] = next;
                st.tasks.quality[u] = 1.0;
            }
        } else {
            // the control: held still from `freeze_from`
            let to = P.tasks.freeze_from.map_or(next, |y| next.min(y)).max(st.tasks.found[u]);
            st.tasks.quality[u] = (1.0 + us.pace).powf((to - st.tasks.found[u]) as f64).min(us.cap);
        }
    }
}

/// The land machines' plant in place, at the towns' prices.
fn land_book(st: &State, C: &Content) -> f64 {
    let NS = C.NS;
    let mut v = 0.0;
    for (t, row) in st.tasks.land_plant.iter().enumerate() {
        for s in 0..NS.min(row.len()) {
            v += C.tasks.plant_goods.iter().enumerate().map(|(j, &g)| row[s][j] * st.price[(t, g)]).sum::<f64>();
        }
    }
    v
}

/// The return on the land's machines over `year`, and their plant's worth at
/// its end, for the gauges' split of the land's rent (None unless they were in
/// use that year).
pub fn land_return_of(st: &State, year: i64) -> Option<(f64, f64)> {
    st.tasks.years.iter().rev().find(|y| y.year == year).filter(|y| y.land_return > 0.0 || y.land_book > 0.0).map(|y| (y.land_return, y.land_book))
}

/// The machines' book at the end of `year` and their wear in it, for the
/// gauges (None while the rule is off or found nothing).
pub fn book_of(st: &State, year: i64) -> Option<(f64, f64)> {
    st.tasks.years.iter().rev().find(|y| y.year == year).filter(|y| y.book > 0.0 || y.wear > 0.0).map(|y| (y.book, y.wear))
}

// ---------------------------------------------------------------- the facts

fn ratio(a: f64, b: f64) -> f64 {
    if b.abs() > 1e-12 { a / b } else { f64::NAN }
}

/// Spearman's rank correlation of two lists of the same length (ties take
/// their mean rank). NaN with fewer than three.
pub fn rank_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n < 3 || y.len() != n {
        return f64::NAN;
    }
    let ranks = |v: &[f64]| -> Vec<f64> {
        let mut idx: Vec<usize> = (0..n).collect();
        idx.sort_by(|&a, &b| v[a].total_cmp(&v[b]));
        let mut r = vec![0.0; n];
        let mut i = 0;
        while i < n {
            let mut j = i;
            while j + 1 < n && v[idx[j + 1]] == v[idx[i]] {
                j += 1;
            }
            for &k in &idx[i..=j] {
                r[k] = (i + j) as f64 / 2.0;
            }
            i = j + 1;
        }
        r
    };
    let (rx, ry) = (ranks(x), ranks(y));
    let m = (n as f64 - 1.0) / 2.0;
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for i in 0..n {
        sxy += (rx[i] - m) * (ry[i] - m);
        sxx += (rx[i] - m).powi(2);
        syy += (ry[i] - m).powi(2);
    }
    ratio(sxy, (sxx * syy).sqrt())
}

/// The rule's facts. For each class a machine may enter (haul, tend, clerk,
/// judge): `auto_CLASS_works_Y` (the share of the works' work in the class
/// machines in place do), `aim_CLASS_works_Y` (the share the works mean them
/// to do), `target_CLASS_works_Y` (the towns' target, which the works move
/// toward), `waterline_CLASS_Y`, `margin_gap_CLASS_Y`, under `land`
/// `auto_CLASS_land_Y` (the land's sectors together), and
/// `machine_share_CLASS_Y` (works and land together); `waiting_haul_works_Y`
/// (hauling meant for machines not yet delivered), the same three for the
/// pits under `land`, `hours_CLASS_Y` (hands at each class's tasks in works,
/// and `hours_minders_Y`), `hours_share_CLASS_Y` and `hours_share_minders_Y`
/// (the same over all the hours worked in works), `machines_years_Y` (the machines' book over the
/// year's wage bill), at the sweep's marks and in 1890; `CLASS_half_year_works`
/// and (under `land`) `haul_half_year_pits`, the first year the share held
/// passes a half;
/// `GOOD_to_machines_Y` for each good a use runs on (of what the island made
/// of it in the year, the share machines for tasks ran on), in 1890 and from
/// 1900; and for each use, by its name: `use_found_year_NAME`,
/// `use_share_found_NAME` (the share of its class the towns would give it in
/// the year found), `use_share_run_NAME` and `use_share_run_year_NAME` (the
/// same over the towns that would give it any, in the first year there are
/// some), and for a use in the works `use_auto_after10_NAME`,
/// `use_margin_gap_after40_NAME`, `use_margin_gap_two_lives_NAME` (two of its
/// machines' lives after the find), `dearer_first_NAME` (the rank correlation
/// across towns of the wage in the year found and the part of the class their
/// works' machines hold ten years on) and `local_run_first_NAME` (the same
/// with whether the town made the use's run goods in the year found in place
/// of the wage); and under `land`, for a use that reaches the land,
/// `use_auto_land_after10_NAME` (its class on the sectors it reaches, done by
/// the machines in place ten years after the find). A fact read past the
/// run's end is none. None unless the rule was on.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    if !r.params.tasks.enabled || C.tasks.is_empty() {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    // the sweep's marks, and 1890: the content after 1900 may find a machine
    // from then (the electric drive)
    let mut marks = crate::facts::marks(y0, y1);
    if y0 < 1890 && 1890 < y1 && !marks.contains(&1890) {
        marks.insert(marks.partition_point(|&y| y < 1890), 1890);
    }
    let at = |y: i64| st.tasks.years.iter().find(|t| t.year == y);
    // every class a machine may enter (hauling, tending, clerking, judging; not
    // presence): the share of the works' work in it the machines in place do,
    // and (`aim_`, "A machine works once it stands") the share the works meant
    // them to do
    let open: Vec<usize> = (0..K).filter(|&k| !C.tasks.closed[k] && !C.tasks.spread[k].is_empty()).collect();
    for &k in &open {
        for &y in &marks {
            out.push((format!("auto_{}_works_{y}", CLASSES[k]), Kind::Share, at(y).map_or(f64::NAN, |t| ratio(t.held[k], t.work[k]))));
        }
    }
    for &k in &open {
        for &y in &marks {
            out.push((format!("aim_{}_works_{y}", CLASSES[k]), Kind::Share, at(y).map_or(f64::NAN, |t| ratio(t.aim[k], t.work[k]))));
        }
    }
    // the towns' target in the works, weighted by each town's work in the
    // class (what a new works would hold, and what the standing works move
    // toward): none until a use of the class is found
    for &k in &open {
        for &y in &marks {
            out.push((format!("target_{}_works_{y}", CLASSES[k]), Kind::Share, at(y).map_or(f64::NAN, |t| t.target[k])));
        }
    }
    // the hauling meant for machines not yet delivered
    for &y in &marks {
        out.push((format!("waiting_haul_works_{y}"), Kind::Share, at(y).map_or(f64::NAN, |t| ratio(t.aim[HAUL] - t.held[HAUL], t.work[HAUL]))));
    }
    for k in 0..K {
        for &y in &marks {
            out.push((format!("hours_{}_{y}", CLASSES[k]), Kind::Count, at(y).map_or(f64::NAN, |t| t.hands[k])));
        }
    }
    for &y in &marks {
        out.push((format!("hours_minders_{y}"), Kind::Count, at(y).map_or(f64::NAN, |t| t.minders)));
    }
    // the same as shares of all the hours worked in works, the minders' among them
    let all_hours = |t: &TasksYear| t.hands.iter().sum::<f64>() + t.minders;
    for k in 0..K {
        for &y in &marks {
            out.push((format!("hours_share_{}_{y}", CLASSES[k]), Kind::Share, at(y).map_or(f64::NAN, |t| ratio(t.hands[k], all_hours(t)))));
        }
    }
    for &y in &marks {
        out.push((format!("hours_share_minders_{y}"), Kind::Share, at(y).map_or(f64::NAN, |t| ratio(t.minders, all_hours(t)))));
    }
    for &k in &open {
        for &y in &marks {
            out.push((format!("waterline_{}_{y}", CLASSES[k]), Kind::Ratio, at(y).map_or(f64::NAN, |t| t.waterline[k])));
        }
    }
    for &k in &open {
        for &y in &marks {
            out.push((format!("margin_gap_{}_{y}", CLASSES[k]), Kind::Ratio, at(y).map_or(f64::NAN, |t| t.margin_gap[k])));
        }
    }
    for &y in &marks {
        let bill = st.acct.years.iter().find(|a| a.year == y).map_or(f64::NAN, |a| a.wage_bill);
        out.push((format!("machines_years_{y}"), Kind::Real, at(y).map_or(f64::NAN, |t| ratio(t.book, bill))));
    }
    // the land ("On the land"): the share of the pits' and quarries' hauling
    // done by machine (each sector's hauling weighted by its work), and the
    // land's rent split into the return on its machines and the ground's part
    if r.params.tasks.land {
        let pits: Vec<usize> = (0..C.NS).filter(|&s| crate::mines::is_extractive(&C.sector_name[s])).collect();
        // the pits' hauling: done by the plant in place, meant, and waiting
        let pits_of = |t: &TasksYear| pits.iter().filter(|&&s| s < t.land_work.len() && s < t.land_aim.len()).fold((0.0, 0.0, 0.0), |(h, a, w), &s| (h + t.land_held[s][HAUL], a + t.land_aim[s][HAUL], w + t.land_work[s][HAUL]));
        for &y in &marks {
            let v = at(y).map_or(f64::NAN, |t| {
                let (h, _, w) = pits_of(t);
                ratio(h, w)
            });
            out.push((format!("auto_haul_pits_{y}"), Kind::Share, v));
        }
        for &y in &marks {
            let v = at(y).map_or(f64::NAN, |t| {
                let (_, a, w) = pits_of(t);
                ratio(a, w)
            });
            out.push((format!("aim_haul_pits_{y}"), Kind::Share, v));
        }
        for &y in &marks {
            let v = at(y).map_or(f64::NAN, |t| {
                let (h, a, w) = pits_of(t);
                ratio(a - h, w)
            });
            out.push((format!("waiting_haul_pits_{y}"), Kind::Share, v));
        }
        let half = st.tasks.years.iter().find(|t| {
            let (h, _, w) = pits_of(t);
            ratio(h, w) >= 0.5
        });
        out.push(("haul_half_year_pits".to_string(), Kind::Year, half.map_or(f64::NAN, |t| t.year as f64)));
        // the fields and the woods (the tractor): each one's hauling done by
        // the machines in place, over its hauling
        for name in ["farm", "pasture", "forestry"] {
            let Some(s) = C.sector_name.iter().position(|n| n == name) else { continue };
            for &y in &marks {
                let v = at(y).filter(|t| s < t.land_work.len()).map_or(f64::NAN, |t| ratio(t.land_held[s][HAUL], t.land_work[s][HAUL]));
                out.push((format!("auto_haul_{name}_{y}"), Kind::Share, v));
            }
        }
        // the combine: the farms' and pastures' tending done by the machines in place
        for name in ["farm", "pasture"] {
            let Some(s) = C.sector_name.iter().position(|n| n == name) else { continue };
            for &y in &marks {
                let v = at(y).filter(|t| s < t.land_work.len()).map_or(f64::NAN, |t| ratio(t.land_held[s][TEND], t.land_work[s][TEND]));
                out.push((format!("auto_tend_{name}_{y}"), Kind::Share, v));
            }
        }
        // every class on the land, all its sectors together: the share of the
        // land's work in the class the machines in place do (nothing where the
        // land has none of the class, as clerking)
        for &k in &open {
            for &y in &marks {
                let v = at(y).map_or(f64::NAN, |t| {
                    let (h, w) = t.land_held.iter().zip(&t.land_work).fold((0.0, 0.0), |(h, w), (a, b)| (h + a[k], w + b[k]));
                    ratio(h, w)
                });
                out.push((format!("auto_{}_land_{y}", CLASSES[k]), Kind::Share, v));
            }
        }
        let books = |y: i64| st.acct.years.iter().find(|a| a.year == y);
        for &y in &marks {
            let v = match (at(y), books(y)) {
                (Some(t), Some(a)) => ratio(t.land_return, a.rent_farm + a.rent_pits),
                _ => f64::NAN,
            };
            out.push((format!("land_rent_machines_{y}"), Kind::Share, v));
        }
        for &y in &marks {
            let v = match (at(y), books(y)) {
                (Some(t), Some(a)) => ratio(a.rent_farm + a.rent_pits - t.land_return, a.income),
                _ => f64::NAN,
            };
            out.push((format!("income_share_ground_{y}"), Kind::Share, v));
        }
    }
    // the ceiling on yield: output at the ceiling over all the land's, by value
    if r.params.tasks.yield_cap.is_some() {
        for &y in &marks {
            out.push((format!("yield_capped_share_{y}"), Kind::Share, at(y).map_or(f64::NAN, |t| ratio(t.capped_value, t.land_value))));
        }
    }
    // every class, the works and the land together: the share of all its work
    // the machines in place do. Both are counted in worker-months a month of
    // work as by hand (the works' hands over their hands a unit's factor; the
    // land's units' hands over their hands an hour), so they add; the land's
    // part is nothing before its first machine and without `land`.
    for &k in &open {
        for &y in &marks {
            let v = at(y).map_or(f64::NAN, |t| {
                let (h, w) = t.land_held.iter().zip(&t.land_work).fold((t.held[k], t.work[k]), |(h, w), (a, b)| (h + a[k], w + b[k]));
                ratio(h, w)
            });
            out.push((format!("machine_share_{}_{y}", CLASSES[k]), Kind::Share, v));
        }
    }
    // each class's work in the works first half done by machines in place
    for &k in &open {
        let half = st.tasks.years.iter().find(|t| ratio(t.held[k], t.work[k]) >= 0.5);
        out.push((format!("{}_half_year_works", CLASSES[k]), Kind::Year, half.map_or(f64::NAN, |t| t.year as f64)));
    }
    // the goods machines run on: of what the island made of each in the year,
    // the share its machines for tasks ran on, in 1890 and from 1900
    let mut runs: Vec<usize> = C.tasks.uses.iter().flat_map(|u| u.run.iter().map(|r| r.0)).collect();
    runs.sort_unstable();
    runs.dedup();
    let made = |g: usize, y: i64| -> f64 {
        let col = crate::accounts::GOODS_FIELDS.iter().position(|&n| n == "made").expect("a goods column");
        let book = st.acct.years.iter().position(|b| b.year == y).and_then(|k| st.acct.year(k));
        book.map_or(f64::NAN, |b| (0..st.T_max).map(|t| b.goods[(t * C.G + g) * crate::accounts::KG + col] as f64).sum())
    };
    for &g in &runs {
        for &y in marks.iter().filter(|&&y| y == 1890 || y >= 1900) {
            let v = at(y).map_or(f64::NAN, |t| ratio(t.run_used.get(g).copied().unwrap_or(0.0), made(g, y)));
            out.push((format!("{}_to_machines_{y}", C.good_name[g]), Kind::Share, v));
        }
    }
    for (u, us) in C.tasks.uses.iter().enumerate() {
        let k = us.class;
        let found = st.tasks.found.get(u).copied().filter(|&y| y >= 0 && y <= y1);
        let name = &us.name;
        out.push((format!("use_found_year_{name}"), Kind::Year, found.map_or(f64::NAN, |y| y as f64)));
        // the towns' target in the year found: the works' for a use of the works,
        // the land's sectors' for a use of the land alone
        let share = if us.on_works { found.and_then(at).map_or(f64::NAN, |t| t.target[k]) } else { st.tasks.land_share_found.get(u).copied().unwrap_or(f64::NAN) };
        out.push((format!("use_share_found_{name}"), Kind::Share, share));
        // what the towns that can run it give it, at a quality of one, once it
        // can be run nearly everywhere: the first year from the find in which
        // the towns that give it any part of its class do three quarters of
        // the class's work in the works (for a machine whose run goods are not
        // had everywhere at once, the year their makers have spread), and the
        // mean of the five years after it, each year's share taken back to a
        // quality of one (new stations open at the ceiling price, and the
        // first years read their fall)
        let run = found.and_then(|y| st.tasks.years.iter().find(|t| t.year >= y && t.use_run.get(u).is_some_and(|x| x[0].is_finite() && x[1] >= 0.75)));
        let at_one = |t: &TasksYear, y: i64| -> f64 {
            let to = r.params.tasks.freeze_from.map_or(t.year, |z| t.year.min(z)).max(y);
            let q = (1.0 + us.pace).powf((to - y) as f64).min(us.cap);
            let kn = &C.tasks.spread[k];
            match t.use_run.get(u) {
                Some(x) if x[0].is_finite() => share_at(kn, worth(kn, x[0]) / q).clamp(us.from, us.reach),
                _ => f64::NAN,
            }
        };
        let settled = run.zip(found).map_or(f64::NAN, |(t0, y)| {
            let v: Vec<f64> = (1..=5).filter_map(|d| at(t0.year + d)).map(|t| at_one(t, y)).filter(|x| x.is_finite()).collect();
            if v.len() == 5 { v.iter().sum::<f64>() / 5.0 } else { f64::NAN }
        });
        out.push((format!("use_share_run_{name}"), Kind::Share, settled));
        out.push((format!("use_share_run_year_{name}"), Kind::Year, run.map_or(f64::NAN, |t| t.year as f64)));
        // the rest read the works, and are nothing for a use not used there
        let works = |v: f64| if us.on_works { v } else { f64::NAN };
        let auto10 = found.and_then(|y| at(y + 10)).map_or(f64::NAN, |t| ratio(t.held[k], t.work[k]));
        out.push((format!("use_auto_after10_{name}"), Kind::Share, works(auto10)));
        let gap40 = found.and_then(|y| at(y + 40)).map_or(f64::NAN, |t| t.margin_gap[k]);
        out.push((format!("use_margin_gap_after40_{name}"), Kind::Ratio, works(gap40)));
        // the same two of its machines' lives after the find (the design's
        // test: within a tenth of one), for a use whose life is not twenty years;
        // none for a rented use, whose life is not read
        let lives2 = found.filter(|_| !us.rented()).and_then(|y| at(y + (2.0 * us.life).round() as i64)).map_or(f64::NAN, |t| t.margin_gap[k]);
        out.push((format!("use_margin_gap_two_lives_{name}"), Kind::Ratio, works(lives2)));
        // for a use that reaches the land: its class on the sectors it reaches,
        // the share the machines in place do ten years after the find (none
        // where the run ends before then)
        if r.params.tasks.land && us.on_sector.iter().any(|&b| b) {
            let v = found.and_then(|y| at(y + 10)).map_or(f64::NAN, |t| {
                let (h, w) = (0..us.on_sector.len().min(t.land_work.len())).filter(|&s| us.on_sector[s]).fold((0.0, 0.0), |(h, w), s| (h + t.land_held[s][k], w + t.land_work[s][k]));
                ratio(h, w)
            });
            out.push((format!("use_auto_land_after10_{name}"), Kind::Share, v));
        }
        // dearer towns first: towns whose works do the class in both years, the
        // wage in the year found against the share held ten years on; and
        // beside it the same with, in place of the wage, whether a works making
        // each of the use's run goods stood in the town in the year found (for
        // the drive, a station)
        let (mut dearer, mut local) = (f64::NAN, f64::NAN);
        if let (Some(y), Some(a), Some(b)) = (found, found.and_then(at), found.and_then(|y| at(y + 10))) {
            let stood = |t: usize, g: usize| (0..st.F_count).any(|f| st.f_town[f] == t && C.method_out[st.f_method[f]] == g && st.f_born[f] <= y && (st.f_alive[f] || st.f_died[f] > y));
            let (mut w, mut s, mut h) = (Vec::new(), Vec::new(), Vec::new());
            for t in 0..a.town_wage.len().min(b.town_work.len()) {
                if a.town_wage[t] > 0.0 && a.town_work[t][k] > 0.0 && b.town_work[t][k] > 0.0 {
                    w.push(a.town_wage[t]);
                    s.push(if !us.run.is_empty() && us.run.iter().all(|r| stood(t, r.0)) { 1.0 } else { 0.0 });
                    h.push(b.town_held[t][k] / b.town_work[t][k]);
                }
            }
            dearer = rank_correlation(&w, &h);
            local = rank_correlation(&s, &h);
        }
        out.push((format!("dearer_first_{name}"), Kind::Real, works(dearer)));
        out.push((format!("local_run_first_{name}"), Kind::Real, works(local)));
    }
    out.extend(land_century_facts(r, &marks));
    out
}

/// The land's twentieth century (docs/rules/machines.md, "Nitrogen and
/// breeding"; docs/rules/tree.md, "Oil, nitrogen and the land"), at the marks
/// from 1900; nothing for a content without oil. `nitrogen_share_Y` and
/// `breeding_share_Y`: of the farms' output, the share from farms whose
/// improvement stands; `grain_a_hand_Y`: the farms' output over their hands,
/// against 1900's; `oil_made_Y`: oil made in the year; `oil_over_coal_heat_Y`:
/// oil's price over coal's for a unit of heat, each the year's mean price in
/// the towns that bought oil, weighted by what they bought of it.
fn land_century_facts(r: &RunResult, marks: &[i64]) -> Vec<(String, Kind, f64)> {
    use crate::accounts::{GOODS_FIELDS, KG};
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    let (Some(&go), Some(&gc)) = (C.good_index.get("oil"), C.good_index.get("coal")) else { return out };
    let years: Vec<i64> = marks.iter().copied().filter(|&y| y >= 1900).collect();
    let at = |y: i64| st.tasks.years.iter().find(|t| t.year == y);
    for (name, f) in [("nitrogen_share", 0), ("breeding_share", 1)] {
        for &y in &years {
            let v = at(y).map_or(f64::NAN, |t| ratio(if f == 0 { t.nitrogen_out } else { t.breeding_out }, t.farm_out));
            out.push((format!("{name}_{y}"), Kind::Share, v));
        }
    }
    let a_hand = |y: i64| at(y).map_or(f64::NAN, |t| ratio(t.farm_out, t.farm_hands));
    for &y in &years {
        out.push((format!("grain_a_hand_{y}"), Kind::Ratio, ratio(a_hand(y), a_hand(1900))));
    }
    let col = |name: &str| GOODS_FIELDS.iter().position(|&n| n == name).expect("a goods column");
    let (c_made, c_price) = (col("made"), col("price"));
    let c_use = [col("households"), col("landlords"), col("families"), col("building"), col("land"), col("projects"), col("works_inputs"), col("works_fuel")];
    let (T, G) = (st.T_max, C.G);
    for &y in &years {
        let book = st.acct.years.iter().position(|b| b.year == y).and_then(|k| st.acct.year(k));
        let (mut made, mut bought, mut po, mut pc) = (0.0, 0.0, 0.0, 0.0);
        if let Some(b) = &book {
            let v = |t: usize, g: usize, c: usize| b.goods[(t * G + g) * KG + c] as f64;
            for t in 0..T {
                made += v(t, go, c_made);
                let x: f64 = c_use.iter().map(|&c| v(t, go, c)).sum();
                bought += x;
                po += x * v(t, go, c_price);
                pc += x * v(t, gc, c_price);
            }
        }
        let n = book.is_some();
        out.push((format!("oil_made_{y}"), Kind::Count, if n { made } else { f64::NAN }));
        let per_heat = ratio(ratio(po, bought) / C.heat[go], ratio(pc, bought) / C.heat[gc]);
        out.push((format!("oil_over_coal_heat_{y}"), Kind::Ratio, if n { per_heat } else { f64::NAN }));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two uses on one class (the office machines, reaching a quarter of
    /// clerking, and office computing, reaching all of it): what a works holds
    /// above its town's top goes to the dearest use only where that use reaches
    /// so far; else to the cheapest that does. A class with one use, or a
    /// dearest use that reaches, is as it always was (None).
    #[test]
    fn above_the_top_goes_to_a_use_that_reaches() {
        let mut tb = TaskTable::default();
        let quarter = Use { name: "quarter".into(), reach: 0.25, ..Use::default() };
        let whole = Use { name: "whole".into(), reach: 1.0, ..Use::default() };
        tb.uses = vec![quarter, whole];
        // the whole is cheaper: it holds to 0.6, the quarter nothing above it
        let ps = [(1, 0.0, 0.6), (0, 0.6, 0.6)];
        assert_eq!(above_top(&tb, &ps, 0.62), Some(1), "above the top, the quarter cannot reach");
        assert_eq!(above_top(&tb, &ps, 0.6), None, "at the top, nothing is above it");
        assert_eq!(above_top(&tb, &ps, 0.5), None);
        // the quarter is cheaper: the whole is the dearest and reaches
        let ps = [(0, 0.0, 0.25), (1, 0.25, 0.4)];
        assert_eq!(above_top(&tb, &ps, 0.45), None);
        // one use alone, even beyond its reach, is as it was
        assert_eq!(above_top(&tb, &[(0, 0.0, 0.2)], 0.3), None);
        assert_eq!(above_top(&tb, &[], 0.3), None);
    }
}
