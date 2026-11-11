//! Canals: a canal age before the railways. Not in the Python
//! (`docs/rules/canals.md`).
//!
//! A canal is dug where the carting it would save pays for it: from a coalfield
//! or a quarry district to its town or to a navigable river, or between two
//! towns the sea does not join. Nothing says when. The promoters are those of
//! `ways.rs`, who price a canal over the whole map as they price a railway, and
//! the canal differs from the railway only in what this file tells them:
//!
//! - **It floats.** A canal is a layer of its own over the cells
//!   (`transport::Graph::boat`). A load is on the road or on the water and pays
//!   a wharf's charge to change, so a canal carries what goes far and a cart
//!   that crosses one pays nothing. A cell of water costs `cost` to pass: less
//!   than half a turnpike or a navigable river, nearly twice a railway.
//! - **It is joined link by link.** Two canals in neighbouring cells are not one
//!   canal. The links are kept (`CanalState::links`) and given to the viewer.
//! - **It keeps to low ground.** The route is cheap along rivers and lowland,
//!   every unit of height is locks (which take stone, bricks and timber of their
//!   own), and nothing is dug over mountains.
//! - **It is mostly labour.** Stone, bricks, timber and clay, and twice their
//!   price again in wages; the navvies are hands taken from the town that
//!   supplies the works for as long as it is dug.
//! - **It may start anywhere there is water or a market**: at a town, on a
//!   navigable river, or on a canal already there.
//! - **It has to be found out, and then seen working.** Nobody on the island
//!   has cut a canal in 1750. Somebody surveys one once enough coal and stone
//!   have been dug, by the rule every invention is found by. The know-how then
//!   spreads from town to town along the trade routes as every invention does,
//!   but mostly from towns that have a canal working: a town that has only
//!   heard of one passes it on at a fifth of the rate (`hearsay`). So the canal
//!   age moves across the island at the pace canals are dug. The know-how is
//!   kept here and not in the content's list of inventions, so that the canals
//!   are the same canals under the port's rules and the game's, and so that
//!   switching them on or off leaves the dates of every other invention where
//!   they were.
//!
//! Railways supersede canals through the same cost graph: a cell of track costs
//! half a cell of water to pass and has no wharf. Nothing closes a canal.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::mat::Mat;
use crate::observer::annual;
use crate::params::Params;
use crate::rng::{Rng, choice_index, choice_p, random};
use crate::sim::RunResult;
use crate::state::{Event, Project, State};
use crate::transport as tr;
use crate::ways::{self, BuildCost, SERVES_TRUNK, Way, WayBook, Words};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CanalParams {
    /// Promoters may dig canals.
    pub enabled: bool,
    /// Path cost of a cell of canal (a railway is 0.07, a navigable river or a
    /// turnpike on lowland 0.3, a made road 0.45).
    pub cost: f64,
    /// The charge for getting a load on to a boat, and again for getting it
    /// off, in the same units (a quay on the sea is 0.6).
    pub wharf: f64,
    /// Stone, bricks, timber and clay for a cell of canal (2 km).
    pub stone: f64,
    pub bricks: f64,
    pub timber: f64,
    pub clay: f64,
    /// Locks: for every unit of height climbed (the map's heights run from 0 to
    /// 1, the lowland ends at 0.24), this many cells' worth of stone, bricks
    /// and timber. The route search charges the same.
    pub lock_cells: f64,
    /// Wages paid on every shilling of materials: a canal is mostly digging.
    pub wages: f64,
    /// The navvies are hands taken from the town that supplies the works.
    pub navvies: bool,
    /// At most this share of a town's hands can be digging.
    pub navvy_max_share: f64,
    /// Months a canal takes if the market fills every order.
    pub build_months: f64,
    /// The yearly return on cost a canal must promise, on the carting it saves...
    pub hurdle: f64,
    /// ...today's saving multiplied by this for the traffic a canal will draw.
    pub induced: f64,
    /// Canals that may be digging at once.
    pub max_projects: usize,
    /// Districts looked at each year: those with the largest carting bill.
    pub districts: usize,
    /// A district is looked at only this many cells or more from a canal, a
    /// town or a navigable river...
    pub branch_gap: f64,
    /// ...and this far from the other districts looked at that year.
    pub district_spacing: f64,
    /// Years a refused branch is left alone.
    pub cool_years: i64,
    /// The first canal can be thought of once this much coal and this much
    /// stone have been dug since the run began (as the content's `discover`
    /// for a method: the chance in a year is `discovery_rate * p^4 / (1 + p^4)`,
    /// p the lesser of the two against its threshold).
    pub discover_coal: f64,
    pub discover_stone: f64,
    /// How well a town must know canals before its promoters dig one (the
    /// town of the first canal starts at 0.25, as with any invention).
    pub know: f64,
    /// What a town with no canal working counts for when the know-how
    /// spreads, against a town that has one (1 would be the content's rule
    /// for every other invention).
    pub hearsay: f64,
}

impl Default for CanalParams {
    /// Off: what the port runs with. The other numbers are the game's, so that
    /// `--set canals.enabled=true` measures the mechanism as it is played.
    fn default() -> CanalParams {
        CanalParams {
            enabled: false,
            cost: 0.13,
            wharf: 0.4,
            stone: 150.0,
            bricks: 80.0,
            timber: 60.0,
            clay: 200.0,
            lock_cells: 80.0,
            wages: 2.0,
            navvies: true,
            navvy_max_share: 0.2,
            build_months: 36.0,
            hurdle: 0.1,
            induced: 1.0,
            max_projects: 12,
            districts: 6,
            branch_gap: 3.0,
            district_spacing: 6.0,
            cool_years: 5,
            discover_coal: 3.0e6,
            discover_stone: 3.0e6,
            know: 0.25,
            hearsay: 0.2,
        }
    }
}

impl CanalParams {
    /// The game's canals.
    pub fn game() -> CanalParams {
        CanalParams { enabled: true, ..CanalParams::default() }
    }
}

/// One link of canal between two neighbouring cells.
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub a: usize,
    pub b: usize,
    /// The year the canal it belongs to was promoted, and the moment it opened.
    pub begun: i64,
    pub opened: f64,
}

/// A link as the record and the live view give it: `opened` is absent while the
/// canal is still being dug.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LinkOut {
    pub a: usize,
    pub b: usize,
    pub begun: i64,
    pub opened: Option<f64>,
}

/// What the canals carried in one year.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanalYear {
    pub year: i64,
    /// Weight put on a boat, and weight times cells of canal passed.
    pub boarded: f64,
    pub ton_cells: f64,
    /// Of `ton_cells`: the towns' trade with each other, by good (length G)...
    pub trade: Vec<f64>,
    /// ...the land's produce on its way to market, by land sector (length NS)...
    pub land: Vec<f64>,
    /// ...and what the works move beyond that, inputs and output.
    pub works: f64,
    /// Cells of canal that carried anything, and cells open.
    pub busy_cells: usize,
    pub cells: usize,
    /// Navvies at work, the mean of the year's months.
    pub navvies: f64,
}

/// What the canals keep between years. Empty while the mechanism is off.
#[derive(Clone, Debug, Default)]
pub struct CanalState {
    /// What the promoters remember.
    pub book: WayBook,
    /// Every link opened, in the order dug.
    pub links: Vec<Link>,
    /// The weight that passed each cell by boat last year.
    pub traffic: Vec<f64>,
    /// (T, T): cells of canal on the cheapest path from one town to another,
    /// on this year's paths.
    pub pair_cells: Mat,
    /// The year's trade on the water so far, by good: weight times cells.
    pub trade_acc: Vec<f64>,
    pub years: Vec<CanalYear>,
    /// Navvies at work this month, by town, and the sum over the year's months so far.
    pub navvies: Vec<f64>,
    pub navvy_acc: f64,
    /// How well each town knows how to cut a canal, 0 to 1.
    pub know: Vec<f64>,
    /// The year the first canal was thought of and the town it was, once it is.
    pub found: Option<(i64, usize)>,
    /// Stream 9: when and where that is, and who does it.
    pub rng: Option<Rng>,
}

fn canal_layer(st: &State) -> &[u8] {
    &st.canal
}

fn canal_layer_mut(st: &mut State) -> &mut Vec<u8> {
    &mut st.canal
}

fn canal_book_mut(st: &mut State) -> &mut WayBook {
    &mut st.cn.book
}

fn canal_knows(st: &State, t: usize) -> f64 {
    st.cn.know.get(t).copied().unwrap_or(0.0)
}

/// The know-how: found once, where most coal and stone are dug, and spread
/// between towns afterwards. Both rules are `tech.rs`'s, for one invention that
/// is not in the content: the chance of the discovery from the goods handled so
/// far, the town by what it digs of them, the spread by `tech::diffusion`'s sum.
/// One thing is the canal's own: a town teaches by its canal. What a town knows
/// counts in full, for itself and for its neighbours, once a canal it subscribed
/// to is open, and for `hearsay` of that until then.
fn knowledge(st: &mut State, C: &Content, P: &Params, year: i64, events: &mut Vec<Event>) {
    let T = st.T_max;
    if st.cn.know.len() != T {
        st.cn.know = vec![0.0; T];
    }
    let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
    if st.cn.found.is_none() {
        let req: Vec<(usize, f64)> = [("coal", P.canals.discover_coal), ("stone", P.canals.discover_stone)]
            .into_iter()
            .filter_map(|(name, x)| C.good_index.get(name).map(|&g| (g, x.max(1e-9))))
            .collect();
        let progress = req.iter().map(|&(g, x)| st.cumprod[g] / x).fold(f64::INFINITY, f64::min).max(0.0);
        let p4 = if progress.is_finite() { progress.powf(4.0) } else { 0.0 };
        let hz = P.discovery_rate * p4 / (1.0 + p4);
        let rng = st.cn.rng.get_or_insert_with(|| crate::rng::canal_stream(P.seed));
        if !act.is_empty() && random(rng) < hz {
            let w: Vec<f64> = act
                .iter()
                .map(|&i| {
                    let score: f64 = req.iter().map(|&(g, x)| st.town_prod_year[(i, g)] / x).sum();
                    let v = score + 1e-9 * st.pop[i];
                    v * v
                })
                .collect();
            let total: f64 = w.iter().sum();
            let p: Vec<f64> = w.iter().map(|&x| x / total).collect();
            let i = act[choice_p(rng, &p)];
            let who = format!("{} {}", crate::tech::FIRST_NAMES[choice_index(rng, crate::tech::FIRST_NAMES.len())], crate::names::surname(rng));
            st.cn.know[i] = st.cn.know[i].max(0.25);
            st.cn.found = Some((year, i));
            let town = st.town_name[i].clone();
            let mut ev = Event::new(
                year as f64 + 0.5,
                "discovery",
                0.9,
                format!("A canal is surveyed at {town}"),
                format!(
                    "{who} levels a line across the fields at {town} and shows that a cut lined with puddled clay will hold its water. One horse on a towpath draws what sixty draw on the road. Coal owners begin to count what their carting costs them."
                ),
            );
            ev.town = Some(i);
            ev.cell = Some(st.town_node[i]);
            events.push(ev);
        }
    }
    if st.cn.found.is_none() {
        return;
    }
    // tech::diffusion, for this one invention, each town counted by whether it has a canal
    let shown: Vec<f64> = (0..T).map(|t| if st.cn.book.lines.iter().any(|l| l.a == t || l.b == t) { 1.0 } else { P.canals.hearsay }).collect();
    let k = st.cn.know.clone();
    for &i in &act {
        let row: Vec<f64> = (0..T).map(|j| if i != j && st.town_active[j] && st.town_cost[(i, j)].is_finite() { (-st.town_cost[(i, j)] / P.diffusion_dist).exp() } else { 0.0 }).collect();
        let rs: f64 = row.iter().sum();
        let contact: f64 = if rs > 0.0 { (0..T).map(|j| row[j] / rs * rs.min(1.0) * shown[j] * k[j]).sum() } else { 0.0 };
        let dk = P.diffusion_rate * (1.0 - k[i]) * (shown[i] * k[i] + 0.7 * contact);
        st.cn.know[i] = (k[i] + dk).clamp(0.0, 1.0);
    }
}

/// Where a canal may start besides a canal: the towns and the navigable rivers.
fn canal_heads(st: &State) -> Vec<usize> {
    let mut heads: Vec<usize> = (0..st.N).filter(|&c| st.world.navigable[c]).collect();
    for t in 0..st.T_max {
        if st.town_active[t] && !heads.contains(&st.town_node[t]) {
            heads.push(st.town_node[t]);
        }
    }
    heads
}

fn has_link(links: &[Link], a: usize, b: usize) -> bool {
    links.iter().any(|l| (l.a == a && l.b == b) || (l.a == b && l.b == a))
}

/// A canal opens: its links are kept. Only its own: where its route runs along
/// a canal already open, those links are there.
fn canal_opened(st: &mut State, pr: &Project, when: f64) {
    for p in pr.cells.windows(2) {
        if !has_link(&st.cn.links, p[0], p[1]) {
            st.cn.links.push(Link { a: p[0], b: p[1], begun: pr.start, opened: when });
        }
    }
}

/// The railways being laid, put on the canal promoters' copy of the cost graph:
/// a canal is not dug for carting that a line already building will take.
fn railways_building(st: &State, _P: &Params, g: &mut tr::Graph) {
    let new: Vec<usize> = st.projects.iter().filter(|pr| pr.way != ways::CANAL).flat_map(|pr| pr.new.iter().copied()).collect();
    if new.is_empty() {
        return;
    }
    let mut laid: Vec<bool> = st.rail.iter().map(|&r| r > 0).collect();
    for &c in &new {
        laid[c] = true;
    }
    ways::lay(g, &st.world, &laid, &new, tr::RAIL_COST);
}

/// And the other way about (`railways::way` hands this to its promoters): the
/// canals being dug, put on the railway promoters' copy of the cost graph, so
/// that no line is promoted on carting a canal will have taken by the time it
/// opens. With no canal digging the graph is not touched.
pub fn digging(st: &State, P: &Params, g: &mut tr::Graph) {
    if !P.canals.enabled {
        return;
    }
    let W = st.world.W;
    for pr in st.projects.iter().filter(|pr| pr.way == ways::CANAL) {
        g.wharf = P.canals.wharf;
        for p in pr.cells.windows(2) {
            let step = ((p[0] / W) as f64 - (p[1] / W) as f64).hypot((p[0] % W) as f64 - (p[1] % W) as f64);
            g.link_boats(p[0], p[1], P.canals.cost * step);
        }
    }
}

/// The canal as the promoters of `ways.rs` see it.
pub fn way(_st: &State, C: &Content, P: &Params) -> Way {
    let K = &P.canals;
    let mut materials = vec![0.0; C.G];
    let mut rise = vec![0.0; C.G];
    for (name, per_cell, lock) in [("stone", K.stone, true), ("bricks", K.bricks, true), ("timber", K.timber, true), ("clay", K.clay, false)] {
        if let Some(&g) = C.good_index.get(name) {
            materials[g] = per_cell;
            if lock {
                rise[g] = K.lock_cells * per_cell;
            }
        }
    }
    Way {
        id: ways::CANAL,
        // the first canal is news of the first weight; the rest are told modestly
        words: Words { kind: "canal", plan_kind: "canal_plan", noun: "canal", Noun: "Canal", track: "canal", branch: "branch canal", line: "canal", weight: 0.3, branches_weight: 0.25 },
        run_cost: K.cost,
        materials,
        rise,
        wages: K.wages.max(1e-9),
        wharf: Some(K.wharf),
        // a month at the least: `Project::months` of 0 marks a project of the port's
        build_months: K.build_months.max(1.0),
        // a river's valley is where a canal wants to be, height is locks, and
        // mountains are priced like the sea: no canal crosses them
        build: BuildCost { lowland: 1.0, hills: 2.0, mountains: 1e6, other: 50.0, river: -0.3, slope: K.lock_cells, laid: 0.15 },
        knows: canal_knows,
        know: K.know,
        hurdle: K.hurdle,
        induced: K.induced,
        order_book: None,
        max_projects: K.max_projects,
        districts: K.districts,
        branch_gap: K.branch_gap,
        district_spacing: K.district_spacing,
        cool_years: K.cool_years,
        layer: canal_layer,
        layer_mut: canal_layer_mut,
        book_mut: canal_book_mut,
        heads: Some(canal_heads),
        opened: Some(canal_opened),
        first_any: true,
        local: true,
        rivals: Some(railways_building),
    }
}

/// The canals open so far, put on a cost graph that `transport::build_graph`
/// has just made (called from `land::update_transport`).
pub fn float(st: &State, P: &Params, g: &mut tr::Graph) {
    if !P.canals.enabled || st.cn.links.is_empty() {
        return;
    }
    g.wharf = P.canals.wharf;
    let W = st.world.W;
    for l in &st.cn.links {
        let step = ((l.a / W) as f64 - (l.b / W) as f64).hypot((l.a % W) as f64 - (l.b % W) as f64);
        g.link_boats(l.a, l.b, P.canals.cost * step);
    }
}

/// The yearly step (called from `sim::yearly`): the promoters look round, and
/// the coming year's paths are read for what the towns' trade will put on the water.
pub fn plan(st: &mut State, C: &Content, P: &Params, year: i64, pbar: &Mat, events: &mut Vec<Event>) {
    if !P.canals.enabled {
        return;
    }
    if st.canal.len() != st.N {
        st.canal = vec![0; st.N];
    }
    knowledge(st, C, P, year, events);
    let way = way(st, C, P);
    ways::plan(st, C, P, &way, year, pbar, events);
    // the book's fares are reckoned for a way on the ground; a canal's are what
    // its boats carried, and the wharves
    if let (Some(note), Some(y)) = (st.cn.book.years.last_mut(), st.cn.years.last()) {
        note.fares = if y.year == year { P.freight * (P.canals.cost * y.ton_cells + 2.0 * P.canals.wharf * y.boarded) } else { 0.0 };
    }
    st.cn.pair_cells = pair_cells(st);
}

/// The monthly step (called from `sim::run_month`): finished canals open.
pub fn progress(st: &mut State, C: &Content, P: &Params, year_float: f64, events: &mut Vec<Event>) {
    if !P.canals.enabled || !st.projects.iter().any(|pr| pr.way == ways::CANAL) {
        return;
    }
    let way = way(st, C, P);
    ways::progress(st, &way, year_float, events);
}

/// The start of a month's labour market: the navvies who dug last month, at last
/// month's wage, are not for hire (as `carting::take_carters`). They are counted
/// in the town whose market supplies the works, which is where they are paid.
pub fn take_navvies(st: &mut State, P: &Params, supply: &mut [f64]) {
    if !P.canals.enabled || !P.canals.navvies {
        return;
    }
    if !st.projects.iter().any(|pr| pr.way == ways::CANAL) {
        st.cn.navvies.clear();
        return;
    }
    let T = supply.len();
    let mut hands = vec![0.0; T];
    for pr in st.projects.iter().filter(|pr| pr.way == ways::CANAL) {
        if st.wage[pr.town] > 0.0 {
            hands[pr.town] += pr.navvy_pay / st.wage[pr.town];
        }
    }
    for t in 0..T {
        hands[t] = hands[t].min(P.canals.navvy_max_share * supply[t]).max(0.0);
        supply[t] -= hands[t];
    }
    st.cn.navvy_acc += hands.iter().sum::<f64>();
    st.cn.navvies = hands;
}

/// Navvies are at work: count them in each town's employment.
pub fn count_navvies(st: &State, P: &Params, employment: &mut [f64]) {
    if !P.canals.enabled {
        return;
    }
    for (e, n) in employment.iter_mut().zip(&st.cn.navvies) {
        *e += n;
    }
}

/// What the month's trade between towns put on the water, good by good (called
/// from `econ::step_month` with its `qt`, the quantity of each good sent from
/// each active town to each other).
pub fn note_trade(st: &mut State, C: &Content, P: &Params, A_idx: &[usize], qt: &[f64]) {
    if !P.canals.enabled || st.cn.pair_cells.rows == 0 {
        return;
    }
    let (na, G) = (A_idx.len(), C.G);
    if st.cn.trade_acc.len() != G {
        st.cn.trade_acc = vec![0.0; G];
    }
    for (i, &ti) in A_idx.iter().enumerate() {
        for (j, &tj) in A_idx.iter().enumerate() {
            let cells = st.cn.pair_cells[(ti, tj)];
            if cells > 0.0 {
                for g in 0..G {
                    st.cn.trade_acc[g] += qt[(i * na + j) * G + g] * C.weight[g] * cells;
                }
            }
        }
    }
}

/// The cheapest way by boat from one cell to every cell its canal reaches:
/// (cells of canal passed, the cell before on the way).
type BoatTree = BTreeMap<usize, (f64, usize)>;

fn boat_tree(g: &tr::Graph, from: usize) -> BoatTree {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let mut cost: BTreeMap<usize, f64> = BTreeMap::new();
    let mut tree: BoatTree = BTreeMap::new();
    let mut heap: BinaryHeap<(Reverse<u64>, usize)> = BinaryHeap::new();
    cost.insert(from, 0.0);
    tree.insert(from, (0.0, from));
    heap.push((Reverse(0f64.to_bits()), from));
    while let Some((Reverse(bits), u)) = heap.pop() {
        let d = f64::from_bits(bits);
        if d > cost[&u] {
            continue;
        }
        let len = tree[&u].0;
        for &(v, w) in &g.boat[u] {
            let v = v as usize;
            if cost.get(&v).is_none_or(|&old| d + w < old) {
                cost.insert(v, d + w);
                let step = ((u / g.W) as f64 - (v / g.W) as f64).hypot((u % g.W) as f64 - (v % g.W) as f64);
                tree.insert(v, (len + step, u));
                heap.push((Reverse((d + w).to_bits()), v));
            }
        }
    }
    tree
}

/// Reads the trees of `land::update_transport` for the stretches that go by
/// water. In a town's tree a cell's predecessor is the next cell toward the
/// town, except where the load takes a boat: then it is the cell where the boat
/// is left, which may be far off (`transport::dijkstra_boats`).
struct Water<'a> {
    st: &'a State,
    trees: BTreeMap<usize, BoatTree>,
}

impl<'a> Water<'a> {
    fn new(st: &'a State) -> Water<'a> {
        Water { st, trees: BTreeMap::new() }
    }

    /// Whether the step from `n` to its predecessor `p` in town `i`'s tree is
    /// by boat: the two are not neighbours, or the step does not cost what the
    /// ground between them costs.
    fn afloat(&self, i: usize, p: usize, n: usize) -> bool {
        let g = &self.st.graph;
        // a boat is boarded and left only where there is a canal
        if g.boat[p].is_empty() || g.boat[n].is_empty() {
            return false;
        }
        for k in 0..8 {
            if tr::neighbour(g.H, g.W, p, k) == Some(n) {
                return self.st.dist[(i, n)] != self.st.dist[(i, p)] + g.w[p * 8 + k];
            }
        }
        true
    }

    /// Cells of canal between `p` and `n`, a stretch by boat.
    fn length(&mut self, p: usize, n: usize) -> f64 {
        let g = &self.st.graph;
        self.trees.entry(p).or_insert_with(|| boat_tree(g, p)).get(&n).map_or(0.0, |x| x.0)
    }

    /// Add `weight` to every cell of canal between `p` and `n`.
    fn carry(&mut self, p: usize, n: usize, weight: f64, on: &mut [f64]) {
        let g = &self.st.graph;
        let tree = self.trees.entry(p).or_insert_with(|| boat_tree(g, p));
        let mut c = n;
        let mut steps = 0;
        while let Some(&(_, before)) = tree.get(&c) {
            on[c] += weight;
            if c == p || steps > 100_000 {
                break;
            }
            c = before;
            steps += 1;
        }
    }
}

/// (T, T): cells of canal on the cheapest path from each town to each other,
/// on the paths as they stand.
fn pair_cells(st: &State) -> Mat {
    let T = st.T_max;
    let mut out = Mat::zeros(T, T);
    if !st.graph.has_boats() {
        return out;
    }
    let mut water = Water::new(st);
    for i in (0..T).filter(|&i| st.town_active[i] && !st.pred[i].is_empty()) {
        let root = st.town_node[i] as i64;
        for j in (0..T).filter(|&j| j != i && st.town_active[j]) {
            let mut n = st.town_node[j] as i64;
            let mut steps = 0;
            let mut cells = 0.0;
            while n != root && n >= 0 && steps < 100_000 {
                let p = st.pred[i][n as usize];
                if p >= 0 && water.afloat(i, p as usize, n as usize) {
                    cells += water.length(p as usize, n as usize);
                }
                n = p;
                steps += 1;
            }
            if n == root {
                out[(i, j)] = cells;
            }
        }
    }
    out
}

/// The year's goods on the water (called from `sim::yearly` after
/// `update_roads`, while the paths are still those the goods took): the weight
/// through every cell of canal, and what it was, as far as anyone can tell. The
/// towns' trade is known good by good (`note_trade` adds it up month by month).
/// The land's produce is taken at twelve times its last month, and what a cell
/// moved beyond that is its works' (as `ways::district_use` reckons).
pub fn year_traffic(st: &mut State, C: &Content, P: &Params, year: i64, warm: bool) {
    if !P.canals.enabled || warm || st.canal.is_empty() {
        return;
    }
    let (N, T) = (st.N, st.T_max);
    let trade = std::mem::replace(&mut st.cn.trade_acc, vec![0.0; C.G]);
    let navvies = std::mem::take(&mut st.cn.navvy_acc) / 12.0;
    let cells = st.canal.iter().filter(|&&c| c > 0).count();
    let mut y = CanalYear { year, trade: if trade.len() == C.G { trade } else { vec![0.0; C.G] }, land: vec![0.0; C.NS], cells, navvies, ..CanalYear::default() };
    let mut on = vec![0.0; N];
    if st.graph.has_boats() {
        // the land's produce by cell and sector, as weight a year
        let mut land_flow = vec![0.0; N];
        let mut by_sector: Vec<(usize, usize, f64)> = Vec::new();
        for u in 0..st.ex_cell.len().min(st.ex_out.len()) {
            let w = 12.0 * st.ex_out[u] * C.weight[C.sector_good[st.ex_sector[u]]];
            if w > 0.0 {
                land_flow[st.ex_cell[u]] += w;
                by_sector.push((st.ex_cell[u], st.ex_sector[u], w));
            }
        }
        let mut water = Water::new(st);
        // cells of canal between each cell and its market
        let mut to_market = vec![0.0; N];
        for i in (0..T).filter(|&i| st.town_active[i] && !st.pred[i].is_empty()) {
            // the weight each cell sends up the tree, children before parents
            let mut acc: Vec<f64> = (0..N).map(|c| if st.catch[c] == i as i64 { st.cell_flow_year[c] } else { 0.0 }).collect();
            for &n in st.settle[i].iter().rev() {
                let n = n as usize;
                let p = st.pred[i][n];
                if p < 0 {
                    continue;
                }
                let p = p as usize;
                if acc[n] > 0.0 && water.afloat(i, p, n) {
                    y.boarded += acc[n];
                    y.ton_cells += acc[n] * water.length(p, n);
                    water.carry(p, n, acc[n], &mut on);
                }
                acc[p] += acc[n];
            }
            // and how far each cell's own goods go by water, parents before children
            let mut far = vec![0.0; N];
            for &n in st.settle[i].iter() {
                let n = n as usize;
                let p = st.pred[i][n];
                if p < 0 {
                    continue;
                }
                let p = p as usize;
                far[n] = far[p] + if water.afloat(i, p, n) { water.length(p, n) } else { 0.0 };
                if st.catch[n] == i as i64 {
                    to_market[n] = far[n];
                }
            }
            // the town's trade, along the same tree
            let root = st.town_node[i] as i64;
            for j in (0..T).filter(|&j| j != i && st.town_active[j]) {
                let f = st.trade_F_year[(i, j)];
                if f <= 0.0 || !st.town_cost[(i, j)].is_finite() {
                    continue;
                }
                let mut n = st.town_node[j] as i64;
                let mut steps = 0;
                while n != root && n >= 0 && steps < 100_000 {
                    let p = st.pred[i][n as usize];
                    if p >= 0 && water.afloat(i, p as usize, n as usize) {
                        y.boarded += f;
                        y.ton_cells += f * water.length(p as usize, n as usize);
                        water.carry(p as usize, n as usize, f, &mut on);
                    }
                    n = p;
                    steps += 1;
                }
            }
        }
        for &(c, s, w) in &by_sector {
            y.land[s] += w * to_market[c];
        }
        for c in 0..N {
            y.works += (st.cell_flow_year[c] - land_flow[c]).max(0.0) * to_market[c];
        }
    }
    y.busy_cells = (0..N).filter(|&c| st.canal[c] > 0 && on[c] > 0.0).count();
    st.cn.traffic = on;
    st.cn.years.push(y);
}

/// Every link of canal for the record and the live view: those open, and those
/// of canals still being dug (with no opening).
pub fn links_out(st: &State) -> Vec<LinkOut> {
    let mut out: Vec<LinkOut> = st.cn.links.iter().map(|l| LinkOut { a: l.a, b: l.b, begun: l.begun, opened: Some(l.opened) }).collect();
    for pr in st.projects.iter().filter(|pr| pr.way == ways::CANAL) {
        for p in pr.cells.windows(2) {
            if !out.iter().any(|l| (l.a == p[0] && l.b == p[1]) || (l.a == p[1] && l.b == p[0])) {
                out.push(LinkOut { a: p[0], b: p[1], begun: pr.start, opened: None });
            }
        }
    }
    out
}

/// Facts for the sweep's table, under "Canals". All of them have no value in a
/// run with the mechanism off, the two that any run can answer aside.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C, rec, P) = (&r.state, &r.content, &r.rec, &r.params);
    let y0 = C.start_year;
    let y1 = y0 + rec.n_years as i64 - 1;
    let series = |name: &str| st.series.index.get(name).map_or(&[][..], |&k| &st.series.data[k][..]);
    let at = |name: &str, y: i64| annual(series(name), y0, y);
    let mut out: Vec<(String, Kind, f64)> = Vec::new();
    let on = P.canals.enabled;
    let book = &st.cn.book;

    // ---- any run: the spread of coal's price between towns, and coal's share of
    // the households' warmth and of the works' heat
    let (T, G) = (rec.T, C.G);
    if let Some(&coal) = C.good_index.get("coal") {
        for y in [1780, 1800, 1820, 1850].into_iter().filter(|&y| y >= y0 && y <= y1) {
            let k = (y - y0) as usize;
            let p: Vec<f64> = (0..T).filter(|&t| rec.town_active[k * T + t] > 0).map(|t| rec.town_price[(k * T + t) * G + coal] as f64).collect();
            let mean = p.iter().sum::<f64>() / p.len().max(1) as f64;
            let var = p.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / p.len().max(1) as f64;
            out.push((format!("coal_price_spread_{y}"), Kind::Real, if mean > 0.0 { var.sqrt() / mean } else { f64::NAN }));
        }
        for y in [1780, 1800, 1820, 1850].into_iter().filter(|&y| y >= y0 && y <= y1) {
            for (fact, name) in [("coal_share_of_warmth", "share_warmth_coal"), ("coal_share_of_heat", "share_heat_coal")] {
                if st.series.index.contains_key(name) {
                    out.push((format!("{fact}_{y}"), Kind::Share, at(name, y)));
                }
            }
        }
    }

    // the rest is the canals' own: no rows of dashes in a table without them
    if !on {
        return out;
    }

    // ---- the network
    let km_at = |y: i64| book.years.iter().find(|n| n.year == y).map_or(f64::NAN, |n| n.km);
    let first = book.lines.iter().map(|l| l.opened).fold(f64::NAN, f64::min);
    out.push(("canal_first_year".into(), Kind::Year, first.floor()));
    let begun = book.lines.iter().map(|l| l.start as f64).fold(f64::NAN, f64::min);
    out.push(("canal_first_begun".into(), Kind::Year, begun));
    for y in (1760..=1890).step_by(10).chain([y1]).filter(|&y| y >= y0 && y <= y1) {
        out.push((format!("canal_km_{y}"), Kind::Count, km_at(y)));
    }
    // the last year a canal was promoted that was dug, and whether the first
    // canal was open before the first railway
    let last = book.lines.iter().map(|l| l.start as f64).fold(f64::NAN, f64::max);
    out.push(("canal_last_begun".into(), Kind::Year, last));
    let rail_first = series("rail_km").iter().position(|&v| v > 0.0).map(|t| y0 as f64 + (t / 12) as f64);
    let before = match rail_first {
        Some(rf) => (first < rf) as u8 as f64,
        None => first.is_finite() as u8 as f64,
    };
    out.push(("canal_before_railway".into(), Kind::Share, before));
    out.push(("canal_years_before_railway".into(), Kind::Real, rail_first.map_or(f64::NAN, |rf| rf - first.floor())));

    // ---- what they carry: million weight-km a year, and the shares of it
    let km_per_cell = 2.0;
    let decade = |y: i64| -> Option<CanalYear> {
        // the mean of the ten years ending in y
        let ys: Vec<&CanalYear> = st.cn.years.iter().filter(|c| c.year > y - 10 && c.year <= y).collect();
        if ys.is_empty() {
            return None;
        }
        let n = ys.len() as f64;
        let mut m = CanalYear { year: y, trade: vec![0.0; G], land: vec![0.0; C.NS], ..CanalYear::default() };
        for c in &ys {
            m.boarded += c.boarded / n;
            m.ton_cells += c.ton_cells / n;
            m.works += c.works / n;
            m.navvies += c.navvies / n;
            for (a, b) in m.trade.iter_mut().zip(&c.trade) {
                *a += b / n;
            }
            for (a, b) in m.land.iter_mut().zip(&c.land) {
                *a += b / n;
            }
        }
        m.busy_cells = ys.last().map_or(0, |c| c.busy_cells);
        m.cells = ys.last().map_or(0, |c| c.cells);
        Some(m)
    };
    let sector_of = |name: &str| C.sector_name.iter().position(|n| n == name);
    for y in [1780, 1800, 1820, 1840, 1860, 1880, y1].into_iter().filter(|&y| y >= y0 && y <= y1) {
        let d = decade(y);
        let v = |f: &dyn Fn(&CanalYear) -> f64| d.as_ref().map_or(f64::NAN, f);
        out.push((format!("canal_mton_km_{y}"), Kind::Real, v(&|d| d.ton_cells * km_per_cell / 1e6)));
        // the mean haul by boat
        out.push((format!("canal_haul_km_{y}"), Kind::Real, v(&|d| if d.boarded > 0.0 { d.ton_cells * km_per_cell / d.boarded } else { f64::NAN })));
        out.push((format!("canal_cells_in_use_{y}"), Kind::Share, v(&|d| if d.cells > 0 { d.busy_cells as f64 / d.cells as f64 } else { f64::NAN })));
    }
    // of everything the parts add up to (trade by good, land by sector, works),
    // the shares that are coal, stone, grain and so on: 1800, 1840 and the end
    for y in [1800, 1840, y1].into_iter().filter(|&y| y >= y0 && y <= y1) {
        let d = decade(y);
        let share = |f: &dyn Fn(&CanalYear) -> f64| match &d {
            Some(d) => {
                let all = d.trade.iter().sum::<f64>() + d.land.iter().sum::<f64>() + d.works;
                if all > 0.0 { f(d) / all } else { f64::NAN }
            }
            None => f64::NAN,
        };
        let good = |d: &CanalYear, good: &str, sector: &str| {
            C.good_index.get(good).map_or(0.0, |&g| d.trade[g]) + sector_of(sector).map_or(0.0, |s| d.land[s])
        };
        out.push((format!("canal_carries_coal_{y}"), Kind::Share, share(&|d| good(d, "coal", "coalmine"))));
        out.push((format!("canal_carries_stone_clay_{y}"), Kind::Share, share(&|d| good(d, "stone", "quarry") + good(d, "clay", "claypit"))));
        out.push((format!("canal_carries_grain_{y}"), Kind::Share, share(&|d| good(d, "grain", "farm"))));
        out.push((format!("canal_carries_timber_{y}"), Kind::Share, share(&|d| good(d, "timber", "forestry"))));
        out.push((format!("canal_carries_iron_ore_{y}"), Kind::Share, share(&|d| good(d, "iron_ore", "ironmine") + C.good_index.get("iron").map_or(0.0, |&g| d.trade[g]))));
        out.push((format!("canal_carries_works_goods_{y}"), Kind::Share, share(&|d| d.works)));
    }
    // the year the boats carried most, and what they carry at the end against it
    let peak = st.cn.years.iter().fold(None::<&CanalYear>, |best, c| if best.is_none_or(|b| c.ton_cells > b.ton_cells) { Some(c) } else { best });
    out.push(("canal_traffic_peak_year".into(), Kind::Year, peak.filter(|p| p.ton_cells > 0.0).map_or(f64::NAN, |p| p.year as f64)));
    let last_year = st.cn.years.last();
    out.push((
        "canal_traffic_end_vs_peak".into(),
        Kind::Ratio,
        match (peak, last_year) {
            (Some(p), Some(l)) if p.ton_cells > 0.0 => l.ton_cells / p.ton_cells,
            _ => f64::NAN,
        },
    ));

    // ---- where works stand: of all the works, each counted by its size against
    // the usual size of its kind, the share within a cell and a half of where the
    // canals are at the end, before there were any and after; and the share of
    // the land that is
    let (H, W) = (st.world.H, st.world.W);
    let near_years: Vec<i64> = [1760, 1800, 1830, y1].into_iter().filter(|&y| y >= y0 && y <= y1).collect();
    if st.canal.iter().any(|&c| c > 0) {
        let off: Vec<bool> = st.canal.iter().map(|&c| c == 0).collect();
        let d = crate::ndimage::distance_transform_edt(&off, H, W);
        for &y in &near_years {
            let (mut near, mut all) = (0.0, 0.0);
            if let Some((alive, K)) = rec.firm_K.get((y - y0) as usize) {
                for (&f, &k) in alive.iter().zip(K) {
                    let size = k as f64 / C.m_K0[st.f_method[f]];
                    all += size;
                    if d[st.f_cell[f]] < 1.5 {
                        near += size;
                    }
                }
            }
            out.push((format!("canal_works_near_{y}"), Kind::Share, if all > 0.0 { near / all } else { f64::NAN }));
        }
        let land = (0..st.N).filter(|&c| st.world.land[c]).count() as f64;
        let land_near = (0..st.N).filter(|&c| st.world.land[c] && d[c] < 1.5).count() as f64;
        out.push(("canal_land_near".into(), Kind::Share, land_near / land.max(1.0)));
        // and the share of canal cells that have track on them or beside them
        let railed: Vec<bool> = st.rail.iter().map(|&x| x == 0).collect();
        if st.rail.iter().any(|&x| x > 0) {
            let dr = crate::ndimage::distance_transform_edt(&railed, H, W);
            let n = st.canal.iter().filter(|&&c| c > 0).count() as f64;
            let beside = (0..st.N).filter(|&c| st.canal[c] > 0 && dr[c] < 1.5).count() as f64;
            out.push((format!("canal_km_beside_track_{y1}"), Kind::Share, beside / n.max(1.0)));
        } else {
            out.push((format!("canal_km_beside_track_{y1}"), Kind::Share, f64::NAN));
        }
    } else {
        for &y in &near_years {
            out.push((format!("canal_works_near_{y}"), Kind::Share, f64::NAN));
        }
        out.push(("canal_land_near".into(), Kind::Share, f64::NAN));
        out.push((format!("canal_km_beside_track_{y1}"), Kind::Share, f64::NAN));
    }

    // ---- from the promoters' book
    // (a junction cut between two canals side by side is in the book as a trunk
    // with no new cell: it is counted by itself and not among the canals)
    let junction = |l: &ways::Line| l.serves == SERVES_TRUNK && l.new_cells == 0;
    let lines_to = |words: &[&str]| book.lines.iter().filter(|l| !junction(l) && words.contains(&ways::serves_word(l.serves))).count() as f64;
    out.push(("canal_lines_trunk".into(), Kind::Count, lines_to(&[ways::serves_word(SERVES_TRUNK)])));
    out.push(("canal_junctions_cut".into(), Kind::Count, book.lines.iter().filter(|l| junction(l)).count() as f64));
    out.push(("canal_lines_to_collieries".into(), Kind::Count, lines_to(&["collieries"])));
    out.push(("canal_lines_to_quarries".into(), Kind::Count, lines_to(&["quarries", "clay pits"])));
    out.push(("canal_lines_to_iron_mines".into(), Kind::Count, lines_to(&["iron mines"])));
    out.push(("canal_lines_to_farms".into(), Kind::Count, lines_to(&["farms", "pastures"])));
    out.push(("canal_lines_to_woods".into(), Kind::Count, lines_to(&["woods"])));
    out.push(("canal_lines_to_works".into(), Kind::Count, lines_to(&["works"])));
    let cost_share: f64 = book
        .lines
        .iter()
        .map(|l| {
            let gdp = 12.0 * at("gdp_nom", (l.opened.floor() as i64).clamp(y0, y1));
            if gdp > 0.0 { l.spent / gdp } else { 0.0 }
        })
        .sum();
    out.push(("canal_cost_in_years_of_output".into(), Kind::Real, cost_share));
    let (spent, expected): (f64, f64) = book.lines.iter().fold((0.0, 0.0), |(s, e), l| (s + l.spent, e + l.cost));
    out.push(("canal_spent_over_expected".into(), Kind::Ratio, if expected > 0.0 { spent / expected } else { f64::NAN }));
    let slowest = book.lines.iter().map(|l| l.opened - l.start as f64).fold(f64::NAN, f64::max);
    out.push(("canal_slowest_years".into(), Kind::Real, slowest));
    let most = book.years.iter().map(|n| n.building).max().unwrap_or(0);
    out.push(("canal_most_digging".into(), Kind::Count, most as f64));
    let most_navvies = st.cn.years.iter().map(|c| c.navvies).fold(f64::NAN, f64::max);
    out.push(("canal_most_navvies".into(), Kind::Count, most_navvies));
    // fares over the capital laid out, the capital at that year's prices
    for y in [1800, 1830, y1].into_iter().filter(|&y| y >= y0 && y <= y1) {
        let v = match book.years.iter().find(|n| n.year == y) {
            Some(n) => {
                let cpi_y = at("cpi", y);
                let capital: f64 = book
                    .lines
                    .iter()
                    .filter(|l| l.opened < (y + 1) as f64)
                    .map(|l| l.spent * cpi_y / at("cpi", (l.opened.floor() as i64).clamp(y0, y1)))
                    .sum();
                if capital > 0.0 { n.fares / capital } else { f64::NAN }
            }
            None => f64::NAN,
        };
        out.push((format!("canal_fares_over_capital_{y}"), Kind::Real, v));
    }
    out
}
