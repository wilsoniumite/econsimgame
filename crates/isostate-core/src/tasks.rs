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
//! - **The works.** A new works is built at its town's target. A standing
//!   works moves toward it by `1 / (12 * adopt_years)` of the gap a month,
//!   times how well last month's order for its machines was filled, and falls
//!   no faster than its machines wear out (`f_auto`). Its hands a unit are
//!   the port's times `sum over classes of share * (1 - held + minders of the
//!   machines held)`; the machines' run goods join its inputs a unit; and the
//!   machines' plant, sized to its capacity, is ordered through a table of its
//!   own (`f_order`), a quarter of what is missing a month, paid from its cash
//!   and by its backers when it is short (`firms::back_firm`).
//! - **Sites and reviews.** `firms::evaluate_sites`, `firms::review` and
//!   `firms::entry` see the works at its town's target: the machines' plant
//!   for a unit of capacity is part of what the capacity costs, and the run
//!   goods and minders come off the margin.
//!
//! Nothing acts where no works holds any part of any class, so a run in which
//! no machine is found (every run to 1900) is the same run to the byte, and
//! with the switch off the state stays empty. The land is out of the machines'
//! reach in this first half: `land_auto` is the next field of the state, and
//! the land's machines of `machines.rs` stand as they are.

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
pub const PRESENCE: usize = 4;

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
}

impl Default for TasksParams {
    /// Off: what the port runs with.
    fn default() -> TasksParams {
        TasksParams { enabled: false, adopt_years: 8.0, dial: 1.0 }
    }
}

impl TasksParams {
    /// The game's.
    pub fn game() -> TasksParams {
        TasksParams { enabled: true, ..TasksParams::default() }
    }
}

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
    /// Multiplies everything one machine takes.
    pub scale: f64,
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
    scale: f64,
}

/// The table of `text` for content `C`: methods and sectors by name (a name the
/// content lacks, as a method of a later stage of the tree, is passed over),
/// and the uses' goods by name (one the content lacks is an error).
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
    let good = |n: &str| C.good_index.get(n).copied().ok_or_else(|| format!("tasks: no good {n:?}"));
    let goods = |m: &BTreeMap<String, f64>| -> Result<Vec<(usize, f64)>, String> { m.iter().map(|(n, &v)| Ok((good(n)?, v))).collect() };
    for u in &raw.uses {
        let k = class(&u.class)?;
        if t.closed[k] {
            return Err(format!("tasks: {:?} works in {:?}, which is closed to machines", u.name, u.class));
        }
        if !(0.0 <= u.from && u.from < u.reach && u.reach <= 1.0) || !(u.life > 0.0) || !(u.scale > 0.0) || !(u.cap >= 1.0) {
            return Err(format!("tasks: {:?} needs 0 <= from < reach <= 1, a life and a scale above nothing and a cap of one or more", u.name));
        }
        t.uses.push(Use {
            name: u.name.clone(),
            class: k,
            from: u.from,
            reach: u.reach,
            on_works: u.on.iter().any(|o| o == "works"),
            plant: goods(&u.plant)?,
            run: goods(&u.run)?,
            minders: u.minders,
            life: u.life,
            pace: u.pace,
            cap: u.cap,
            found_after: u.found_after,
            discover: goods(&u.discover)?,
            scale: u.scale,
        });
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
    /// Each class's work in the works (by hand or by machine), and the part
    /// of it machines did.
    pub work: [f64; K],
    pub held: [f64; K],
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
}

/// The year so far.
#[derive(Clone, Debug, Default, PartialEq)]
struct Acc {
    months: f64,
    work: [f64; K],
    held: [f64; K],
    hands: [f64; K],
    minders: f64,
    target: [[f64; 2]; K],
    water: [[f64; 2]; K],
    gap: [[f64; 2]; K],
    wear: f64,
    town_work: Vec<[f64; K]>,
    town_held: Vec<[f64; K]>,
    town_wage: Vec<f64>,
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
    /// By works: the share of each class its machines hold, and their book.
    pub f_auto: Vec<[f64; K]>,
    pub f_mach: Vec<f64>,
    /// By works and plant good (`TaskTable::plant_goods`): the machines'
    /// plant in place, and what is on order for it. A table of its own: the
    /// works' plant order is wiped when no capacity is on order.
    pub f_plant: Vec<Vec<f64>>,
    pub f_order: Vec<Vec<f64>>,
    /// By works: the share of last month's machine order that was delivered,
    /// and its hands a unit over what they would be by hand.
    pub f_fill: Vec<f64>,
    pub f_factor: Vec<f64>,
    /// By town and class, this month: the uses that would hold it, cheapest
    /// first, each with the part it holds (`parts`).
    parts: Vec<[Vec<(usize, f64, f64)>; K]>,
    // The land's half comes next: `land_auto[t][s]`, the share of each class
    // machines hold on each town's land sectors, beside `target` by town.
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
        s.f_mach.resize(n, 0.0);
        s.f_plant.resize(n, vec![0.0; np]);
        s.f_order.resize(n, vec![0.0; np]);
        s.f_fill.resize(n, 1.0);
        s.f_factor.resize(n, 1.0);
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

/// `parts`, worked out from this month's costs and shares (`end_month`).
fn work_out_parts(st: &State, C: &Content, t: usize, k: usize) -> Vec<(usize, f64, f64)> {
    let s = &st.tasks;
    let mut us: Vec<usize> = (0..C.tasks.uses.len()).filter(|&u| C.tasks.uses[u].class == k && s.cost.get(t).is_some_and(|c| c[u].is_finite())).collect();
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
        let ps = parts(st, t, k);
        let n = ps.len();
        for (j, &(u, a, b)) in ps.iter().enumerate() {
            // the dearest use holds whatever the works holds above the town's top
            let b = if j + 1 == n { b.max(x) } else { b };
            let e = b.min(x);
            if e > a {
                held[k] += e - a;
                mach[u] += sh[k] * units(&tb.spread[k], a, e) / qeff(st, C, P, u);
            }
        }
    }
    if held.iter().all(|&h| h == 0.0) {
        return None;
    }
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
/// made): where a works' machines hold some part of a class, its hands a unit
/// `l` are cut to the work left and the minders, the machines' run goods join
/// its inputs a unit `a`, and its order for their plant is what its capacity
/// needs less what it has. `fi` are the works, `cap` their capacity.
pub fn works_month(st: &mut State, C: &Content, P: &Params, fi: &[usize], cap: &[f64], a: &mut Mat, l: &mut [f64]) {
    if !active(st, P) {
        return;
    }
    fit(st, C);
    let pg = C.tasks.plant_goods.clone();
    for (i, &f) in fi.iter().enumerate() {
        let auto = st.tasks.f_auto[f];
        let adj = adjust(st, C, P, st.f_town[f], st.f_method[f], &auto);
        let s = &mut st.tasks;
        let Some(adj) = adj else {
            s.f_factor[f] = 1.0;
            s.f_order[f].iter_mut().for_each(|v| *v = 0.0);
            continue;
        };
        let base = l[i];
        l[i] = base * adj.factor;
        s.f_factor[f] = adj.factor;
        for &(g, v) in &adj.run {
            a[(i, g)] += base * v;
        }
        for (j, &g) in pg.iter().enumerate() {
            let need = adj.plant.iter().find(|p| p.0 == g).map_or(0.0, |p| cap[i] * base * p.1);
            s.f_order[f][j] = (need - s.f_plant[f][j]).max(0.0);
        }
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

/// A works founded (`firms::create_firm`): it is built at its town's target.
pub fn opened(st: &mut State, C: &Content, P: &Params, f: usize) {
    if !active(st, P) {
        return;
    }
    fit(st, C);
    let t = st.f_town[f];
    let target = st.tasks.target.get(t).copied().unwrap_or([0.0; K]);
    let s = &mut st.tasks;
    s.f_auto[f] = target;
    s.f_mach[f] = 0.0;
    s.f_plant[f].iter_mut().for_each(|v| *v = 0.0);
    s.f_order[f].iter_mut().for_each(|v| *v = 0.0);
    s.f_fill[f] = 1.0;
    s.f_factor[f] = 1.0;
}

/// The month's end (`econ::step_month`, last): each town's target at the
/// month's wage and prices; each works' machines moved toward it and worn;
/// the year's sums.
pub fn end_month(st: &mut State, C: &Content, P: &Params) {
    if !P.tasks.enabled || C.tasks.is_empty() {
        return;
    }
    let T = st.T_max;
    let tb = &C.tasks;
    let nu = tb.uses.len();
    let on = active(st, P);
    // ---- what a unit costs, and the town's target
    if on {
        let s = &mut st.tasks;
        if s.target.len() != T {
            s.target = vec![[0.0; K]; T];
            s.cost = vec![vec![f64::NAN; nu]; T];
            s.share = vec![vec![f64::NAN; nu]; T];
            s.parts = vec![Default::default(); T];
        }
        for t in 0..T {
            for u in 0..nu {
                let us = &tb.uses[u];
                let w = st.wage[t];
                let known = st.town_active[t] && us.on_works && st.tasks.found[u] >= 0 && w > 0.0;
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
                let ps = work_out_parts(st, C, t, k);
                st.tasks.target[t][k] = ps.iter().map(|p| p.2).fold(0.0, f64::max);
                st.tasks.parts[t][k] = ps;
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
    acc.months += 1.0;
    for t in 0..T {
        acc.town_wage[t] += st.wage[t];
    }
    for f in 0..st.F_count {
        if !st.f_alive[f] {
            continue;
        }
        let (t, m) = (st.f_town[f], st.f_method[f]);
        let sh = tb.method.get(m).copied().unwrap_or([0.0; K]);
        // what the works did this month, by class: its hands over its hands a unit's factor
        let base = st.f_labor[f] / st.tasks.f_factor[f].max(1e-12);
        let adj = if on { adjust(st, C, P, t, m, &st.tasks.f_auto[f]) } else { None };
        let held = adj.as_ref().map_or([0.0; K], |a| a.held);
        for k in 0..K {
            let wk = base * sh[k];
            if wk <= 0.0 {
                continue;
            }
            acc.work[k] += wk;
            acc.held[k] += wk * held[k];
            acc.hands[k] += wk * (1.0 - held[k]);
            acc.town_work[t][k] += wk;
            acc.town_held[t][k] += wk * held[k];
            if on {
                acc.target[k][0] += wk * st.tasks.target[t][k];
                acc.target[k][1] += wk;
                let ps = parts(st, t, k);
                if let Some(&(u0, _, _)) = ps.first() {
                    acc.water[k][0] += wk * st.wage[t] / st.tasks.cost[t][u0];
                    acc.water[k][1] += wk;
                }
                let x = st.tasks.f_auto[f][k];
                if x > 0.0 {
                    // the use holding the works' last task
                    let u = ps.iter().rev().find(|p| p.1 < x).or(ps.last()).map(|p| p.0);
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
        }
        if !on {
            continue;
        }
        // toward the target
        for k in 0..K {
            let target = st.tasks.target[t][k];
            let x = st.tasks.f_auto[f][k];
            if target > x {
                st.tasks.f_auto[f][k] = x + step * (target - x) * st.tasks.f_fill[f];
            } else if target < x {
                // no faster than the machines wear: the part held falls to where
                // a life's twelfth fewer units hold it
                let ps = parts(st, t, k);
                let (u, a0) = ps.iter().rev().find(|p| p.1 < x).or(ps.last()).map_or((usize::MAX, 0.0), |p| (p.0, p.1));
                let floor = if u == usize::MAX {
                    target
                } else {
                    let wear = 1.0 / (12.0 * tb.uses[u].life);
                    top_for_units(&tb.spread[k], a0, units(&tb.spread[k], a0, x) * (1.0 - wear))
                };
                st.tasks.f_auto[f][k] = target.max(floor);
            }
        }
        // the machines in place wear by their life's twelfth a month
        if st.tasks.f_mach[f] > 0.0 || st.tasks.f_plant[f].iter().any(|&v| v > 0.0) {
            let wear = 1.0 / (12.0 * life_of(C, f, st));
            acc.wear += st.tasks.f_mach[f] * wear;
            st.tasks.f_mach[f] *= 1.0 - wear;
            st.tasks.f_plant[f].iter_mut().for_each(|v| *v *= 1.0 - wear);
        }
    }
    st.tasks.acc = acc;
}

/// The life of a works' machines: the longest of the uses some class of its
/// method could use (one use in this half).
fn life_of(C: &Content, f: usize, st: &State) -> f64 {
    let sh = C.tasks.method.get(st.f_method[f]).copied().unwrap_or([0.0; K]);
    C.tasks.uses.iter().filter(|u| sh[u.class] > 0.0).map(|u| u.life).fold(f64::NAN, f64::max).max(1.0)
}

// ---------------------------------------------------------------- the year

/// The year's end (`sim::end_year`, after the books close and before the
/// gauges read them): the year's record; then what is found for the next
/// year, and each use's quality in it.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64) {
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
    });
    // found at this turn of the year, for the next
    let next = year + 1;
    for u in 0..nu {
        let us = &C.tasks.uses[u];
        if st.tasks.found[u] < 0 {
            if next >= us.found_after && us.discover.iter().all(|&(g, x)| st.cumprod[g] >= x) {
                st.tasks.found[u] = next;
                st.tasks.quality[u] = 1.0;
            }
        } else {
            st.tasks.quality[u] = (1.0 + us.pace).powf((next - st.tasks.found[u]) as f64).min(us.cap);
        }
    }
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

/// The rule's facts. `auto_haul_works_Y` (the share of the works' hauling
/// machines do), `hours_CLASS_Y` (hands at each class's tasks in works, and
/// `hours_minders_Y`), `waterline_haul_Y`, `margin_gap_haul_Y`,
/// `machines_years_Y` (the machines' book over the year's wage bill), at the
/// sweep's marks; and for each use, the year it was found, the year its
/// class's work in works is first half done by machine, the share of its
/// class the towns would give it in the year found, the margin's gap forty
/// years on, and whether dearer towns took it first (the rank correlation
/// across towns of the wage in the year found and the part of the class
/// their works' machines hold ten years on). None unless the rule was on.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    if !r.params.tasks.enabled || C.tasks.is_empty() {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let marks = crate::facts::marks(y0, y1);
    let at = |y: i64| st.tasks.years.iter().find(|t| t.year == y);
    for &y in &marks {
        out.push((format!("auto_haul_works_{y}"), Kind::Share, at(y).map_or(f64::NAN, |t| ratio(t.held[HAUL], t.work[HAUL]))));
    }
    for k in 0..K {
        for &y in &marks {
            out.push((format!("hours_{}_{y}", CLASSES[k]), Kind::Count, at(y).map_or(f64::NAN, |t| t.hands[k])));
        }
    }
    for &y in &marks {
        out.push((format!("hours_minders_{y}"), Kind::Count, at(y).map_or(f64::NAN, |t| t.minders)));
    }
    for &y in &marks {
        out.push((format!("waterline_haul_{y}"), Kind::Ratio, at(y).map_or(f64::NAN, |t| t.waterline[HAUL])));
    }
    for &y in &marks {
        out.push((format!("margin_gap_haul_{y}"), Kind::Ratio, at(y).map_or(f64::NAN, |t| t.margin_gap[HAUL])));
    }
    for &y in &marks {
        let bill = st.acct.years.iter().find(|a| a.year == y).map_or(f64::NAN, |a| a.wage_bill);
        out.push((format!("machines_years_{y}"), Kind::Real, at(y).map_or(f64::NAN, |t| ratio(t.book, bill))));
    }
    for (u, us) in C.tasks.uses.iter().enumerate() {
        let k = us.class;
        let found = st.tasks.found.get(u).copied().filter(|&y| y >= 0 && y <= y1);
        let name = &us.name;
        out.push((format!("{name}_found_year"), Kind::Year, found.map_or(f64::NAN, |y| y as f64)));
        let half = st.tasks.years.iter().find(|t| ratio(t.held[k], t.work[k]) >= 0.5).map(|t| t.year);
        out.push((format!("{name}_half_year"), Kind::Year, half.map_or(f64::NAN, |y| y as f64)));
        let share = found.and_then(at).map_or(f64::NAN, |t| t.target[k]);
        out.push((format!("{name}_share_found"), Kind::Share, share));
        let auto10 = found.and_then(|y| at(y + 10)).map_or(f64::NAN, |t| ratio(t.held[k], t.work[k]));
        out.push((format!("{name}_auto_after10"), Kind::Share, auto10));
        let gap40 = found.and_then(|y| at(y + 40)).map_or(f64::NAN, |t| t.margin_gap[k]);
        out.push((format!("{name}_margin_gap_after40"), Kind::Ratio, gap40));
        // dearer towns first: towns whose works do the class in both years
        let dearer = match (found.and_then(at), found.and_then(|y| at(y + 10))) {
            (Some(a), Some(b)) => {
                let (mut w, mut h) = (Vec::new(), Vec::new());
                for t in 0..a.town_wage.len().min(b.town_work.len()) {
                    if a.town_wage[t] > 0.0 && a.town_work[t][k] > 0.0 && b.town_work[t][k] > 0.0 {
                        w.push(a.town_wage[t]);
                        h.push(b.town_held[t][k] / b.town_work[t][k]);
                    }
                }
                rank_correlation(&w, &h)
            }
            _ => f64::NAN,
        };
        out.push((format!("{name}_dearer_first"), Kind::Real, dearer));
    }
    out
}
