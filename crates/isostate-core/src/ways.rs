//! Promoters of ways: who proposes a line, what it is reckoned to be worth, who
//! pays for it and when it opens. Not in the Python (`docs/rules/railways.md`).
//!
//! The port's promoters (`rail.rs`) look at pairs of towns and at the 25 busiest
//! cells of the map, and by the 1850s they have nothing left to look at. These
//! look further:
//!
//! - **Trunks** between two towns, as before.
//! - **Town branches**: every town with no track is routed to the nearest point of
//!   the network, so a line may leave from any junction, not only from a town.
//! - **District branches**: the places where most is spent on carting away from
//!   the line (a quarry, a group of works, a farming district), routed the same way.
//!
//! Every line, trunks too, is valued by one sum: the carting it would save in a
//! year over the whole map at today's traffic (`Net::sums`). The cost graph is
//! copied, the line is laid on the copy, and every cell's cost to market and the
//! served towns' cost to every other town are measured again. Nothing is assumed
//! about where a line "ought" to pay.
//!
//! Lines being built count as laid when the next one is reckoned, so two projects
//! of one year never pay twice for the cells they share, and the second is not
//! credited with what the first already saves. A line that leans on another in
//! this way does not open before it.
//!
//! The cost a line is set against is reckoned at prices somebody pays: the
//! buying town's own for what its makers can supply, the country's makers' for
//! the rest (`reckoned_price`), and rails marked up by the order book.
//!
//! The module knows nothing about railways in particular. A [`Way`] says what is
//! being built: what it costs to run on, what a cell of it is made of, how long
//! it takes, what the ground charges for it, which technology it needs, and which
//! cell layer and which book it writes. `railways.rs` supplies the railway and
//! `canals.rs` the canal, which differs in four things a `Way` can say: it floats
//! (a layer of its own over the cells, joined link by link, with a wharf between
//! it and the ground), its locks take materials by the height climbed, it may
//! start from any town or navigable river, and it keeps its own knowledge.

use std::collections::{BTreeMap, BTreeSet};

use crate::content::Content;
use crate::land::{USE_INDUSTRY, USE_NAMES, USE_SECTOR0};
use crate::mat::Mat;
use crate::ndimage::{self, Mode};
use crate::params::Params;
use crate::state::{Event, Project, RailOpened, State};
use crate::transport as tr;
use crate::util::{argsort, fmt_pct, pairwise_sum, pairwise_sum_by};
use crate::world::{HILLS, LOWLAND, MOUNTAINS, World};

/// `Project::way` of the port's own railway projects (`rail.rs`).
pub const PORT: u8 = 0;
/// `Project::way` of the game's railways.
pub const RAILWAY: u8 = 1;
/// `Project::way` of the canals (`canals.rs`).
pub const CANAL: u8 = 2;

/// `Project::serves` and `Line::serves`: a line between two towns.
pub const SERVES_TRUNK: u8 = 0;
/// A line that brings a town its first track.
pub const SERVES_TOWN: u8 = 1;
// Any other value is the `land::USE_*` code of what fills the carts in the
// district a branch runs to (farms, quarries, collieries, works...).

/// What the ground charges for building a way, in the units of the route search
/// (`transport::rail_build_graph` has the railway's numbers).
#[derive(Clone, Debug, PartialEq)]
pub struct BuildCost {
    pub lowland: f64,
    pub hills: f64,
    pub mountains: f64,
    /// Any other terrain (the sea's edge).
    pub other: f64,
    /// Added on a cell with a river: a bridge for a railway. A canal would make
    /// this negative, since a river valley is where it wants to be.
    pub river: f64,
    /// Per unit of height between neighbouring cells: gradients for a railway,
    /// locks for a canal.
    pub slope: f64,
    /// A cell that already carries the way.
    pub laid: f64,
}

/// The words the news uses for a way.
#[derive(Clone, Debug, PartialEq)]
pub struct Words {
    /// The news kind of an opening ("railway") and of a scheme ("rail_plan").
    pub kind: &'static str,
    pub plan_kind: &'static str,
    /// "railway" and "Railway".
    pub noun: &'static str,
    pub Noun: &'static str,
    /// "track", as in "40 km of track".
    pub track: &'static str,
    /// "branch line".
    pub branch: &'static str,
}

/// One kind of way and the promoters' rules for it.
#[derive(Clone, Debug)]
pub struct Way {
    /// Which projects are this way's (`Project::way`).
    pub id: u8,
    pub words: Words,
    /// Path cost of one cell along it (`transport::RAIL_COST` for a railway).
    pub run_cost: f64,
    /// Units of each good one new cell takes (length G).
    pub materials: Vec<f64>,
    /// Units of each good for every unit of height the new line climbs or
    /// falls (length G, or empty): a canal's locks. A railway has none.
    pub rise: Vec<f64>,
    /// Wages paid on every shilling of materials (`econ::step_month` pays the
    /// navvies so): three tenths for a railway, as in the port.
    pub wages: f64,
    /// A way that floats: it is a layer of its own over the cells, joined only
    /// along its lines, and a load pays this to get on to it and again to get
    /// off (`transport::Graph::boat`). `None` for a way laid on the ground,
    /// where every cell is a station and neighbouring cells of track are joined.
    pub wharf: Option<f64>,
    /// Months a line takes if the market fills every order: 1 or more
    /// (`Project::months` of 0 marks a project of the port's).
    pub build_months: f64,
    pub build: BuildCost,
    /// How well a town knows how to build the way (a column of `st.know` for
    /// the railway; the canal keeps its own), and how well it must.
    pub knows: fn(&State, usize) -> f64,
    pub know: f64,
    /// The yearly return on cost a line must promise.
    pub hurdle: f64,
    /// Promoters multiply today's saving by this for the traffic a line will draw.
    pub induced: f64,
    /// The good whose order book the promoters read before pricing a line: its
    /// price is marked up by what all lines have on order over a year's make.
    pub order_book: Option<usize>,
    /// Lines of this way that may be building at once.
    pub max_projects: usize,
    /// Districts looked at each year.
    pub districts: usize,
    /// A district is a candidate only this many cells or more from the way...
    pub branch_gap: f64,
    /// ...and this far from the other districts looked at that year.
    pub district_spacing: f64,
    /// Years a refused branch is left alone.
    pub cool_years: i64,
    /// The cell layer the way is written to, and the book its promoters keep.
    pub layer: fn(&State) -> &[u8],
    pub layer_mut: fn(&mut State) -> &mut Vec<u8>,
    pub book_mut: fn(&mut State) -> &mut WayBook,
    /// Cells a branch may start from besides the way itself: for a canal, the
    /// towns and the navigable rivers. A railway branches only from track.
    pub heads: Option<fn(&State) -> Vec<usize>>,
    /// Called when a line opens, after its cells are written to the layer, with
    /// the moment it opened: what the way keeps beyond the layer and the book.
    pub opened: Option<fn(&mut State, &Project, f64)>,
    /// The first line of any kind is the great news, not the first between two
    /// towns (the first canal is as likely to run from a coalfield).
    pub first_any: bool,
    /// The districts looked at each year are the dearest among those whose own
    /// market town knows how to build the way, not the dearest of the island
    /// (where the know-how is in one town, its districts are the ones looked at).
    pub local: bool,
}

/// One line that was built.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Line {
    /// The year it was promoted and the moment it opened.
    pub start: i64,
    pub opened: f64,
    /// `SERVES_TRUNK`, `SERVES_TOWN`, or the land-use code of a district.
    pub serves: u8,
    /// The two towns that subscribed (the same town twice for some branches).
    pub a: usize,
    pub b: usize,
    /// The far end: a town's cell or the middle of a district.
    pub target: usize,
    pub new_cells: usize,
    pub km: f64,
    /// What the promoters expected to pay, and the return they expected on it.
    pub cost: f64,
    pub roi: f64,
    /// What was paid in the end.
    pub spent: f64,
}

/// What the network came to at the end of one year.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct YearNote {
    pub year: i64,
    pub km: f64,
    /// Fares on the year's traffic over the way: the weight through each cell of
    /// it, at the way's running cost. Each cell passed is counted as one straight
    /// step, so this is a little low.
    pub fares: f64,
    /// Money laid out on lines opened so far, in the money of the day.
    pub capital: f64,
    /// Lines building at the year's end.
    pub building: usize,
}

/// What the promoters of one way remember. Empty while the way is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WayBook {
    /// Pairs of towns already joined (or with a line between them building).
    pub links: BTreeSet<(usize, usize)>,
    /// Pairs refused, and the year they may be looked at again.
    pub pair_cooldown: BTreeMap<(usize, usize), i64>,
    /// Towns and districts (by cell) a branch was refused for, likewise.
    pub spot_cooldown: BTreeMap<usize, i64>,
    /// Every line opened.
    pub lines: Vec<Line>,
    /// District branches opened since the last yearly news item (indices into `lines`).
    pub unreported: Vec<usize>,
    pub years: Vec<YearNote>,
}

/// Path length in cells (1 per straight step, sqrt(2) per diagonal).
fn path_length(path: &[usize], W: usize) -> f64 {
    let steps: Vec<f64> = path
        .windows(2)
        .map(|p| {
            let (dy, dx) = ((p[1] / W) as f64 - (p[0] / W) as f64, (p[1] % W) as f64 - (p[0] % W) as f64);
            dy.hypot(dx)
        })
        .collect();
    pairwise_sum(&steps)
}

/// Distance between two cells, in cells.
fn apart(W: usize, a: usize, b: usize) -> f64 {
    ((a / W) as f64 - (b / W) as f64).hypot((a % W) as f64 - (b % W) as f64)
}

/// The graph a route is searched on: what each step costs to build.
/// With the railway's `BuildCost` and only laid track marked this is
/// `transport::rail_build_graph`, number for number.
pub fn build_graph(world: &World, laid: &[bool], b: &BuildCost) -> tr::Graph {
    let (H, W) = (world.H, world.W);
    let N = H * W;
    let cell: Vec<f64> = (0..N)
        .map(|c| {
            let base = match world.terrain[c] {
                LOWLAND => b.lowland,
                HILLS => b.hills,
                MOUNTAINS => b.mountains,
                _ => b.other,
            };
            if laid[c] { b.laid } else { base + b.river * (world.river[c] as u8 as f64) }
        })
        .collect();
    let mut w = vec![f64::INFINITY; N * 8];
    for a in 0..N {
        for k in 0..8 {
            let Some(n) = tr::neighbour(H, W, a, k) else { continue };
            let step = tr::step_len(k);
            let mut cost = step * 0.5 * (cell[a] + cell[n]) + b.slope * (world.h[a] - world.h[n]).abs();
            if !(world.land[a] && world.land[n]) {
                cost = 1e6;
            }
            w[a * 8 + k] = cost.max(1e-3);
        }
    }
    tr::Graph::new(N, W, H, w, vec![false; N * 8])
}

/// Lay the way on `cells` of the cost graph `g`: every step between two land
/// cells that both carry it (`laid`, which must already include `cells`) costs
/// `run_cost` a cell, where that is cheaper than what is there. Returns the
/// edges it made cheaper (as indices into `g.w`) with what they cost before, so
/// that `unlay` can put them back.
pub fn lay(g: &mut tr::Graph, world: &World, laid: &[bool], cells: &[usize], run_cost: f64) -> Vec<(usize, f64)> {
    let (H, W) = (world.H, world.W);
    let mut changed = Vec::new();
    for &a in cells {
        if !world.land[a] {
            continue;
        }
        for k in 0..8 {
            let Some(b) = tr::neighbour(H, W, a, k) else { continue };
            if laid[b] && world.land[b] {
                let w = (run_cost * tr::step_len(k)).max(1e-3);
                // NEIGH8[7 - k] is the opposite direction
                for e in [a * 8 + k, b * 8 + (7 - k)] {
                    if w < g.w[e] {
                        changed.push((e, g.w[e]));
                        g.w[e] = w;
                    }
                }
            }
        }
    }
    g.min_w = g.min_w.min(run_cost.max(1e-3));
    changed
}

/// Undo a `lay` (the graph's note of its cheapest edge may stay lowered, which
/// only makes the bucket search a little finer).
fn unlay(g: &mut tr::Graph, changed: &[(usize, f64)]) {
    for &(e, w) in changed.iter().rev() {
        g.w[e] = w;
    }
}

/// What laying one line changed on a cost graph, so that it can be taken up again.
#[derive(Default)]
struct Laid {
    /// Edges of the ground made cheaper, with what they cost before.
    edges: Vec<(usize, f64)>,
    /// Links of water added.
    links: Vec<(usize, usize)>,
}

/// Lay a line of `way` on the cost graph `g`. A way on the ground makes every
/// step between two cells that carry it cheaper (`lay`; `laid` must already
/// include `new_cells`). A way that floats is joined link by link along `path`,
/// the line's cells in order, and to nothing else: two canals in neighbouring
/// cells are not joined unless a line was dug from one into the other.
fn lay_line(g: &mut tr::Graph, world: &World, way: &Way, laid: &[bool], path: &[usize], new_cells: &[usize]) -> Laid {
    match way.wharf {
        None => Laid { edges: lay(g, world, laid, new_cells, way.run_cost), links: Vec::new() },
        Some(wharf) => {
            g.wharf = wharf;
            let mut links = Vec::new();
            for p in path.windows(2) {
                let step = apart(world.W, p[0], p[1]);
                if g.link_boats(p[0], p[1], way.run_cost * step) {
                    links.push((p[0], p[1]));
                }
            }
            Laid { edges: Vec::new(), links }
        }
    }
}

/// Take a line laid by `lay_line` up again.
fn take_up(g: &mut tr::Graph, laid: &Laid) {
    unlay(g, &laid.edges);
    for &(a, b) in laid.links.iter().rev() {
        g.unlink_boats(a, b);
    }
}

/// Path costs `base` brought up to date after the edges `changed` of `g` got
/// cheaper. Only what a cheaper edge can reach more cheaply is visited, so
/// reckoning a short branch costs a search over its own neighbourhood and not
/// over the whole map. The result is what a search from scratch would give.
fn improve(g: &tr::Graph, base: &[f64], changed: &[(usize, f64)]) -> Vec<f64> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    if g.has_boats() {
        return improve_boats(g, base, changed);
    }
    let mut d = base.to_vec();
    let mut heap: BinaryHeap<(Reverse<u64>, u32)> = BinaryHeap::new();
    let relax = |d: &mut Vec<f64>, heap: &mut BinaryHeap<(Reverse<u64>, u32)>, u: usize, k: usize| {
        if let Some(v) = tr::neighbour(g.H, g.W, u, k) {
            let nd = d[u] + g.w[u * 8 + k];
            if nd < d[v] {
                d[v] = nd;
                heap.push((Reverse(nd.to_bits()), v as u32));
            }
        }
    };
    for &(e, _) in changed {
        relax(&mut d, &mut heap, e / 8, e % 8);
    }
    while let Some((Reverse(bits), u)) = heap.pop() {
        let u = u as usize;
        if f64::from_bits(bits) > d[u] {
            continue;
        }
        for k in 0..8 {
            relax(&mut d, &mut heap, u, k);
        }
    }
    d
}

/// `improve` on a graph with canals (`transport::dijkstra_boats` has the two
/// layers). `base` has only the ground's costs, so the water's are worked out
/// again from every cell with a canal; that is a search over the canals alone,
/// and then the ground is visited only where a cheaper edge or a boat improves
/// on it. The result is again what a search from scratch would give.
fn improve_boats(g: &tr::Graph, base: &[f64], changed: &[(usize, f64)]) -> Vec<f64> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let N = g.N;
    let mut d = base.to_vec();
    let mut afloat = vec![f64::INFINITY; N];
    // node c is the ground of cell c, node N + c its water
    let mut heap: BinaryHeap<(Reverse<u64>, u32)> = BinaryHeap::new();
    for c in 0..N {
        if !g.boat[c].is_empty() && d[c].is_finite() {
            afloat[c] = d[c] + g.wharf;
            heap.push((Reverse(afloat[c].to_bits()), (N + c) as u32));
        }
    }
    for &(e, _) in changed {
        let (u, k) = (e / 8, e % 8);
        if let Some(v) = tr::neighbour(g.H, g.W, u, k) {
            let nd = d[u] + g.w[e];
            if nd < d[v] {
                d[v] = nd;
                heap.push((Reverse(nd.to_bits()), v as u32));
            }
        }
    }
    while let Some((Reverse(bits), node)) = heap.pop() {
        let at = f64::from_bits(bits);
        if (node as usize) < N {
            let u = node as usize;
            if at > d[u] {
                continue;
            }
            for k in 0..8 {
                if let Some(v) = tr::neighbour(g.H, g.W, u, k) {
                    let nd = at + g.w[u * 8 + k];
                    if nd < d[v] {
                        d[v] = nd;
                        heap.push((Reverse(nd.to_bits()), v as u32));
                    }
                }
            }
            if !g.boat[u].is_empty() && at + g.wharf < afloat[u] {
                afloat[u] = at + g.wharf;
                heap.push((Reverse(afloat[u].to_bits()), (N + u) as u32));
            }
        } else {
            let u = node as usize - N;
            if at > afloat[u] {
                continue;
            }
            for &(v, w) in &g.boat[u] {
                let v = v as usize;
                if at + w < afloat[v] {
                    afloat[v] = at + w;
                    heap.push((Reverse(afloat[v].to_bits()), (N + v) as u32));
                }
            }
            if at + g.wharf < d[u] {
                d[u] = at + g.wharf;
                heap.push((Reverse(d[u].to_bits()), u as u32));
            }
        }
    }
    d
}

/// The network as the promoters reckon with it this year: what is laid and what
/// is being laid, the cost graph with all of that in place, and every cell's
/// cost to market on it. Remade after each promotion.
struct Net {
    /// Cells that carry the way or have it building.
    laid: Vec<bool>,
    /// Whether a branch has anywhere to start from.
    any: bool,
    /// Cells a branch may start from besides the way itself (`Way::heads`).
    heads: Vec<usize>,
    /// The cost graph with the lines under way laid as well.
    g: tr::Graph,
    /// The active towns.
    act: Vec<usize>,
    /// Each cell's cost to its cheapest market town on `g`.
    d: Vec<f64>,
    /// Cost from a town to every cell on `g`, worked out when first asked for.
    from_town: BTreeMap<usize, Vec<f64>>,
    /// The route graph, and the cheapest route from the network to every cell.
    route: tr::Graph,
    reach: Option<tr::Paths>,
}

impl Net {
    fn new(st: &State, way: &Way) -> Net {
        let layer = (way.layer)(st);
        let mut laid: Vec<bool> = layer.iter().map(|&r| r > 0).collect();
        let mut g = st.graph.clone();
        let under_way: Vec<usize> = st.projects.iter().filter(|pr| pr.way == way.id).flat_map(|pr| pr.new.iter().copied()).collect();
        for &c in &under_way {
            laid[c] = true;
        }
        if way.wharf.is_none() {
            lay(&mut g, &st.world, &laid, &under_way, way.run_cost);
        } else {
            // a way that floats is joined along each line, not cell to neighbouring cell
            for pr in st.projects.iter().filter(|pr| pr.way == way.id) {
                lay_line(&mut g, &st.world, way, &laid, &pr.cells, &pr.new);
            }
        }
        let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
        let nodes: Vec<(usize, f64)> = act.iter().map(|&t| (st.town_node[t], 0.0)).collect();
        let d = tr::distances(&g, &nodes);
        let route = build_graph(&st.world, &laid, &way.build);
        let heads = way.heads.map_or_else(Vec::new, |heads| heads(st));
        let any = laid.iter().any(|&l| l) || !heads.is_empty();
        Net { laid, any, heads, g, act, d, from_town: BTreeMap::new(), route, reach: None }
    }

    /// The cheapest route from any point of the network to `target`: its cells
    /// from the junction outward. Empty if there is none over land.
    fn route_to(&mut self, target: usize) -> Vec<usize> {
        let (laid, route, heads) = (&self.laid, &self.route, &self.heads);
        let reach = self.reach.get_or_insert_with(|| {
            let mut sources: Vec<(usize, f64)> = (0..laid.len()).filter(|&c| laid[c]).map(|c| (c, 0.0)).collect();
            sources.extend(heads.iter().filter(|&&c| !laid[c]).map(|&c| (c, 0.0)));
            tr::dijkstra(route, &sources, None)
        });
        if !reach.dist[target].is_finite() || reach.dist[target] > 1e5 {
            return Vec::new();
        }
        let mut path = vec![target];
        let mut n = target;
        while reach.pred[n] >= 0 && path.len() < 100_000 {
            n = reach.pred[n] as usize;
            path.push(n);
        }
        path.reverse();
        path
    }

    /// What the line along `path`, with its `new_cells`, would save in a year at
    /// today's traffic, in money: the carting between every cell and its market,
    /// and the carting on the trade of each town in `towns` with every other town.
    fn sums(&mut self, st: &State, P: &Params, way: &Way, path: &[usize], new_cells: &[usize], towns: &[usize]) -> (f64, f64) {
        // each served town's cost to every cell, on the network as it stands
        for &t in towns {
            let g = &self.g;
            self.from_town.entry(t).or_insert_with(|| tr::distances(g, &[(st.town_node[t], 0.0)]));
        }
        // lay the line on the graph for the length of this sum
        for &c in new_cells {
            self.laid[c] = true;
        }
        let line = lay_line(&mut self.g, &st.world, way, &self.laid, path, new_cells);
        let changed = &line.edges;
        let d1 = improve(&self.g, &self.d, changed);
        let mut local = 0.0;
        for c in 0..st.N {
            let f = st.cell_flow_year[c];
            if f > 0.0 && self.d[c].is_finite() && d1[c].is_finite() {
                local += f * (self.d[c] - d1[c]).max(0.0);
            }
        }
        let F = &st.trade_F_year;
        let mut trade = 0.0;
        for (n, &t) in towns.iter().enumerate() {
            let c0 = &self.from_town[&t];
            let c1 = improve(&self.g, c0, changed);
            for &j in &self.act {
                if j == t || towns[..n].contains(&j) {
                    continue;
                }
                let (was, now) = (c0[st.town_node[j]], c1[st.town_node[j]]);
                if was.is_finite() && now.is_finite() && now < was {
                    trade += (F[(t, j)] + F[(j, t)]) * (was - now);
                }
            }
        }
        take_up(&mut self.g, &line);
        for &c in new_cells {
            self.laid[c] = false;
        }
        (P.freight * local, P.freight * trade)
    }
}

/// Materials for the line along `path` of which `new_cells` are still to be
/// made: so much a cell, and for a way with locks so much for every unit of
/// height between a new cell and its neighbour on the line.
fn materials(way: &Way, world: &World, path: &[usize], new_cells: &[usize]) -> Vec<f64> {
    // (a junction cut between two canals that lie side by side is a cell's work)
    let cells = if way.wharf.is_some() { new_cells.len().max(1) } else { new_cells.len() };
    let mut need: Vec<f64> = way.materials.iter().map(|&m| m * cells as f64).collect();
    if !way.rise.is_empty() {
        let climb: f64 = path.windows(2).filter(|p| new_cells.contains(&p[0]) || new_cells.contains(&p[1])).map(|p| (world.h[p[0]] - world.h[p[1]]).abs()).sum();
        for (n, &r) in need.iter_mut().zip(&way.rise) {
            *n += r * climb;
        }
    }
    need
}

/// The wages a project pays on every shilling of materials: its way's, or the
/// port's three tenths (`Project::wages` of 0 marks a project of the port's).
pub fn navvy_share(pr: &Project) -> f64 {
    if pr.wages > 0.0 { pr.wages } else { 0.3 }
}

/// The price the promoters reckon on for `need` of good `g` bought in town `t`
/// over `months`: the year's price there for what the town's own makers can
/// supply in the time, and for the rest no less than what the country's makers
/// got for it (each town's price weighted by what it made). A town that makes no
/// rails may show a price of next to nothing, because nobody there has bought
/// any; an order of some hundreds would not be filled at it.
fn reckoned_price(st: &State, pbar: &Mat, g: usize, t: usize, need: f64, months: f64) -> f64 {
    let local = pbar[(t, g)];
    let made: f64 = (0..st.T_max).map(|i| st.town_prod_year[(i, g)]).sum();
    if made <= 0.0 || need <= 0.0 {
        return local;
    }
    let makers = (0..st.T_max).map(|i| st.town_prod_year[(i, g)] * pbar[(i, g)]).sum::<f64>() / made;
    let own = (st.town_prod_year[(t, g)] * months / 12.0 / need).min(1.0);
    own * local + (1.0 - own) * local.max(makers)
}

/// What the promoters expect to pay for `need` bought in town `t`: the prices of
/// `reckoned_price`, with the order-book good marked up by what the lines already
/// building still have on order, measured in years of the whole country's make
/// of it; and the way's wages on top for the navvies (`econ::step_month` pays them so).
fn expected_cost(st: &State, C: &Content, way: &Way, pbar: &Mat, need: &[f64], t: usize) -> f64 {
    let mut markup = 1.0;
    if let Some(gb) = way.order_book {
        let backlog: f64 = st.projects.iter().map(|pr| pr.order[gb]).sum();
        let made: f64 = (0..st.T_max).map(|i| st.town_prod_year[(i, gb)]).sum();
        markup += backlog / made.max(1.0);
    }
    let price = |g: usize| reckoned_price(st, pbar, g, t, need[g], way.build_months);
    pairwise_sum_by(C.G, |g| if need[g] > 0.0 { need[g] * price(g) * if Some(g) == way.order_book { markup } else { 1.0 } } else { 0.0 }) * (1.0 + way.wages)
}

/// The cheaper of two towns to buy `need` in, by the promoters' own reckoning,
/// and what they expect to pay there.
fn cheaper_town(st: &State, C: &Content, way: &Way, pbar: &Mat, need: &[f64], a: usize, b: usize) -> (usize, f64) {
    let (ca, cb) = (expected_cost(st, C, way, pbar, need, a), expected_cost(st, C, way, pbar, need, b));
    if ca <= cb { (a, ca) } else { (b, cb) }
}

/// Subscriptions from the landlords (35 per cent of their cash) and households
/// (8 per cent) of `towns`, in that order, up to `cost`. If less than half is
/// raised everyone gets his money back and nothing is returned.
fn subscribe(st: &mut State, towns: &[usize], cost: f64) -> Option<(f64, Vec<(bool, usize, f64)>)> {
    let mut subs: Vec<(bool, usize, f64)> = Vec::new();
    let mut raised = 0.0;
    for (n, &t) in towns.iter().enumerate() {
        if towns[..n].contains(&t) {
            continue;
        }
        for (lords, frac) in [(true, 0.35), (false, 0.08)] {
            let cash = if lords { &mut st.lord_cash } else { &mut st.hh_cash };
            let take = (cash[t].max(0.0) * frac).min(cost - raised);
            if take > 0.0 {
                cash[t] -= take;
                raised += take;
                subs.push((lords, t, take));
            }
        }
    }
    if raised < 0.5 * cost {
        for &(lords, t, put) in &subs {
            let cash = if lords { &mut st.lord_cash } else { &mut st.hh_cash };
            cash[t] += put;
        }
        return None;
    }
    Some((raised, subs))
}

/// Share `amount` among the subscribers in proportion to what each put in. A
/// negative amount is a last call on them.
fn repay(st: &mut State, subs: &[(bool, usize, f64)], amount: f64) {
    let total: f64 = subs.iter().map(|s| s.2).sum();
    if total <= 0.0 {
        return;
    }
    for &(lords, t, put) in subs {
        let cash = if lords { &mut st.lord_cash } else { &mut st.hh_cash };
        cash[t] += amount * put / total;
    }
}

/// What fills the carts in the 5 by 5 block round cell `c`: the `land::USE_*`
/// code of the land sector, or of industry, with the most weight to move.
fn district_use(st: &State, C: &Content, c: usize) -> u8 {
    let (H, W) = (st.world.H, st.world.W);
    let (cy, cx) = ((c / W) as i64, (c % W) as i64);
    let near = |x: usize| ((x / W) as i64 - cy).abs() <= 2 && ((x % W) as i64 - cx).abs() <= 2;
    let mut by_sector = vec![0.0; C.NS];
    let mut land_flow: BTreeMap<usize, f64> = BTreeMap::new();
    for u in 0..st.ex_cell.len().min(st.ex_out.len()) {
        let x = st.ex_cell[u];
        if near(x) {
            let s = st.ex_sector[u];
            let w = 12.0 * st.ex_out[u] * C.weight[C.sector_good[s]];
            by_sector[s] += w;
            *land_flow.entry(x).or_insert(0.0) += w;
        }
    }
    // what a cell moved beyond its land's output is its works' inputs and output
    let mut works = 0.0;
    for y in (cy - 2).max(0)..=(cy + 2).min(H as i64 - 1) {
        for x in (cx - 2).max(0)..=(cx + 2).min(W as i64 - 1) {
            let cell = y as usize * W + x as usize;
            works += (st.cell_flow_year[cell] - land_flow.get(&cell).copied().unwrap_or(0.0)).max(0.0);
        }
    }
    let mut best = (works, USE_INDUSTRY);
    for s in 0..C.NS {
        if by_sector[s] > best.0 {
            best = (by_sector[s], USE_SECTOR0 + s as u8);
        }
    }
    best.1
}

/// The plural the news uses for what a line serves: for a district branch, the
/// land use of `district_use` by its name in `land::USE_NAMES`.
pub fn serves_word(serves: u8) -> &'static str {
    match serves {
        SERVES_TRUNK => "towns",
        SERVES_TOWN => "town",
        _ => match USE_NAMES.get(serves as usize).copied() {
            Some("farm") => "farms",
            Some("pasture") => "pastures",
            Some("forestry") => "woods",
            Some("quarry") => "quarries",
            Some("claypit") => "clay pits",
            Some("ironmine") => "iron mines",
            Some("coalmine") => "collieries",
            Some("industry") => "works",
            _ => "district",
        },
    }
}

/// "a", "a and b", "a, b and c".
fn list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [a] => a.clone(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

/// The yearly step for one way: fold last year's branch openings into one news
/// item, note what the network came to, then let the promoters look round.
pub fn plan(st: &mut State, C: &Content, P: &Params, way: &Way, year: i64, pbar: &Mat, events: &mut Vec<Event>) {
    let mut book = std::mem::take((way.book_mut)(st));
    report_branches(st, way, &mut book, year, events);
    promote(st, C, P, way, &mut book, year, pbar, events);
    note_year(st, P, way, &mut book, year);
    *(way.book_mut)(st) = book;
}

/// One news item for all the district branches opened in the year.
fn report_branches(st: &State, way: &Way, book: &mut WayBook, year: i64, events: &mut Vec<Event>) {
    let opened = std::mem::take(&mut book.unreported);
    if opened.is_empty() {
        return;
    }
    let lines: Vec<&Line> = opened.iter().map(|&k| &book.lines[k]).collect();
    let km: f64 = lines.iter().map(|l| l.km).sum();
    // what they serve, the kind with most line first; each kind's towns once
    let mut kinds: Vec<(u8, f64, Vec<String>)> = Vec::new();
    for l in &lines {
        let name = st.town_name[l.a].clone();
        match kinds.iter_mut().find(|k| k.0 == l.serves) {
            Some(k) => {
                k.1 += l.km;
                if !k.2.contains(&name) {
                    k.2.push(name);
                }
            }
            None => kinds.push((l.serves, l.km, vec![name])),
        }
    }
    kinds.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
    let parts: Vec<String> = kinds.iter().map(|k| format!("the {} near {}", serves_word(k.0), list(&k.2))).collect();
    let longest = lines.iter().fold(lines[0], |best, l| if l.km > best.km { l } else { best });
    let title = if lines.len() == 1 {
        format!("A {} opens near {}", way.words.branch, st.town_name[longest.a])
    } else {
        format!("{} {}s open", lines.len(), way.words.branch)
    };
    // dated by the last of them to open: a branch that opened in December with
    // the line it joins is not reported ahead of it
    let when = lines.iter().map(|l| l.opened).fold(year as f64 + 0.9, f64::max);
    let mut ev = Event::new(
        when,
        way.words.kind,
        0.35,
        title,
        format!("{km:.0} km of {} opened in {year}, to {}.", way.words.branch, list(&parts)),
    );
    ev.town = Some(longest.a);
    ev.cell = Some(longest.target);
    events.push(ev);
}

/// The year's figures for the book.
fn note_year(st: &State, P: &Params, way: &Way, book: &mut WayBook, year: i64) {
    let layer = (way.layer)(st);
    let on: f64 = (0..st.N).filter(|&c| layer[c] > 0).map(|c| st.traffic[c]).sum();
    book.years.push(YearNote {
        year,
        km: 2.0 * layer.iter().filter(|&&r| r > 0).count() as f64,
        fares: P.freight * way.run_cost * on,
        capital: book.lines.iter().map(|l| l.spent).sum(),
        building: st.projects.iter().filter(|pr| pr.way == way.id).count(),
    });
}

/// A new project and its subscribers' money.
#[allow(clippy::too_many_arguments)]
fn project(way: &Way, serves: u8, a: usize, b: usize, target: usize, town: usize, path: Vec<usize>, new: Vec<usize>, need: Vec<f64>, raised: f64, subs: Vec<(bool, usize, f64)>, year: i64, roi: f64, cost: f64, W: usize) -> Project {
    let length = path_length(&path, W);
    Project {
        a,
        b,
        spur: serves != SERVES_TRUNK && serves != SERVES_TOWN,
        target,
        town,
        cells: path,
        new,
        order: need.clone(),
        need,
        cash: raised,
        start: year,
        roi,
        cost,
        length_km: 2.0 * length,
        way: way.id,
        months: way.build_months,
        wages: way.wages,
        navvy_pay: 0.0,
        serves,
        paid_in: raised,
        subs,
    }
}

/// The promoters look round: trunks first, then towns without track, then the
/// districts that spend most on carting.
#[allow(clippy::too_many_arguments)]
fn promote(st: &mut State, C: &Content, P: &Params, way: &Way, book: &mut WayBook, year: i64, pbar: &Mat, events: &mut Vec<Event>) {
    let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    let k: Vec<f64> = (0..st.T_max).map(|t| (way.knows)(st, t)).collect();
    let kmax = act.iter().map(|&t| k[t]).fold(f64::NEG_INFINITY, f64::max);
    let building = |st: &State| st.projects.iter().filter(|pr| pr.way == way.id).count();
    if act.len() < 2 || kmax < way.know || building(st) >= way.max_projects {
        return;
    }
    // refusals that have run their time are forgotten
    book.pair_cooldown.retain(|_, until| *until > year);
    book.spot_cooldown.retain(|_, until| *until > year);
    let W = st.world.W;
    let mut net = Net::new(st, way);

    // ---- trunks: pairs of towns, the busiest first
    let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
    for &a_ in &act {
        for &b_ in &act {
            if a_ < b_ && k[a_].max(k[b_]) >= way.know && k[a_].min(k[b_]) > 0.05 {
                if book.links.contains(&(a_, b_)) || book.pair_cooldown.get(&(a_, b_)).copied().unwrap_or(-99) > year {
                    continue;
                }
                let flow = st.trade_F_year[(a_, b_)] + st.trade_F_year[(b_, a_)];
                if flow > 0.0 {
                    pairs.push((flow, a_, b_));
                }
            }
        }
    }
    pairs.sort_by(|x, y| y.0.total_cmp(&x.0).then(y.1.cmp(&x.1)).then(y.2.cmp(&x.2)));
    for &(_, a_, b_) in pairs.iter().take(30) {
        if building(st) >= way.max_projects {
            break;
        }
        let (na, nb) = (st.town_node[a_], st.town_node[b_]);
        let paths = tr::dijkstra(&net.route, &[(na, 0.0)], Some(nb));
        if !paths.dist[nb].is_finite() || paths.dist[nb] > 1e5 {
            // no way over land (or, for a canal, none that keeps off the mountains):
            // the ground does not change, so there is no hurry to look again
            book.pair_cooldown.insert((a_, b_), year + 10);
            continue;
        }
        let path = tr::path_cells(&paths.pred, na, nb);
        if path.len() < 2 {
            continue;
        }
        let new_cells: Vec<usize> = path.iter().copied().filter(|&c| !net.laid[c]).collect();
        // (a way that floats is not joined where two of its lines merely lie in
        // neighbouring cells: there a junction is still to be cut, with no new cell)
        let joined = way.wharf.is_none() || path.windows(2).all(|p| net.g.boat.get(p[0]).is_some_and(|links| links.iter().any(|l| l.0 as usize == p[1])));
        if new_cells.is_empty() && joined {
            book.links.insert((a_, b_));
            continue;
        }
        let (local, trade) = net.sums(st, P, way, &path, &new_cells, &[a_, b_]);
        let need = materials(way, &st.world, &path, &new_cells);
        let (buy_town, cost) = cheaper_town(st, C, way, pbar, &need, a_, b_);
        let roi = way.induced * (local + trade) / cost.max(1e-9);
        if roi < way.hurdle {
            book.pair_cooldown.insert((a_, b_), year + 4);
            continue;
        }
        let Some((raised, subs)) = subscribe(st, &[a_, b_], cost) else {
            book.pair_cooldown.insert((a_, b_), year + 2);
            continue;
        };
        let pr = project(way, SERVES_TRUNK, a_, b_, nb, buy_town, path, new_cells, need, raised, subs, year, roi, cost, W);
        let mut ev = Event::new(
            year as f64 + 0.95,
            way.words.plan_kind,
            0.45,
            format!("{} and {} {} promoted", st.town_name[a_], st.town_name[b_], way.words.Noun),
            format!(
                "Promoters raise money for {:.0} km of line. They expect the carting it saves, between the towns and along the way, to repay {} a year.",
                pr.length_km,
                fmt_pct(roi, 0)
            ),
        );
        ev.town = Some(a_);
        ev.cell = Some(na);
        events.push(ev);
        // a pair with a line building is not looked at again
        book.links.insert((a_, b_));
        st.projects.push(pr);
        net = Net::new(st, way);
    }

    // ---- branches: from the nearest point of the network
    if !net.any {
        return;
    }
    let (H, N) = (st.world.H, st.N);
    // (what it serves, the far end, the town there)
    let mut cands: Vec<(u8, usize, usize)> = Vec::new();
    let cooling = |book: &WayBook, c: usize| book.spot_cooldown.iter().any(|(&q, &until)| until > year && apart(W, q, c) < 4.0);
    let mut towns: Vec<usize> = act.iter().copied().filter(|&t| !net.laid[st.town_node[t]] && !cooling(book, st.town_node[t])).collect();
    towns.sort_by(|&x, &y| st.pop[y].total_cmp(&st.pop[x]).then(x.cmp(&y)));
    if way.heads.is_some() {
        // a way that may start from any town has no town to reach: its towns are
        // joined by trunks, and its branches run from the towns to the districts
        towns.clear();
    }
    for t in towns {
        cands.push((SERVES_TOWN, st.town_node[t], t));
    }
    // the carting bill of each 5 by 5 block, on the network as it will be
    let bill: Vec<f64> = (0..N).map(|c| if net.d[c].is_finite() { st.cell_flow_year[c] * net.d[c] } else { 0.0 }).collect();
    let block: Vec<f64> = ndimage::uniform_filter(&bill, H, W, 5, Mode::Constant).iter().map(|v| v * 25.0).collect();
    let free: Vec<bool> = net.laid.iter().map(|&l| !l).collect();
    let d_way = ndimage::distance_transform_edt(&free, H, W);
    let neg: Vec<f64> = block.iter().map(|&v| -v).collect();
    let mut picked: Vec<usize> = Vec::new();
    for &c in &argsort(&neg) {
        if picked.len() >= way.districts || block[c] <= 0.0 {
            break;
        }
        // (a cell on the line is never a district, whatever the gap is set to)
        if st.catch[c] < 0 || !st.world.land[c] || net.laid[c] || d_way[c] < way.branch_gap {
            continue;
        }
        // (nor is a district that sits on a town or a river a branch could start from)
        if net.heads.iter().any(|&q| apart(W, q, c) < way.branch_gap) {
            continue;
        }
        if way.local && k[st.catch[c] as usize] < way.know {
            continue;
        }
        if picked.iter().any(|&q| apart(W, q, c) < way.district_spacing) || cooling(book, c) {
            continue;
        }
        picked.push(c);
        cands.push((district_use(st, C, c), c, st.catch[c] as usize));
    }
    for (serves, target, town) in cands {
        if building(st) >= way.max_projects {
            break;
        }
        if net.laid[target] {
            continue; // a line promoted a moment ago already goes there
        }
        let path = net.route_to(target);
        let new_cells: Vec<usize> = path.iter().copied().filter(|&c| !net.laid[c]).collect();
        if path.len() < 2 || new_cells.is_empty() {
            continue;
        }
        let junction = path[0];
        // the town at the network's end: the junction's market town if it has the
        // way, else the town with it that is cheapest to reach from there
        let on_way = |t: usize| net.laid[st.town_node[t]] || way.heads.is_some();
        let ct = st.catch[junction];
        let jt = if ct >= 0 && on_way(ct as usize) {
            ct as usize
        } else {
            let mut best = (f64::INFINITY, town);
            for &t in &act {
                if on_way(t) && st.dist[(t, junction)] < best.0 {
                    best = (st.dist[(t, junction)], t);
                }
            }
            best.1
        };
        if k[town].max(k[jt]) < way.know {
            continue;
        }
        let serve: Vec<usize> = if serves == SERVES_TOWN { vec![town] } else { Vec::new() };
        let (local, trade) = net.sums(st, P, way, &path, &new_cells, &serve);
        let need = materials(way, &st.world, &path, &new_cells);
        let (buy_town, cost) = cheaper_town(st, C, way, pbar, &need, town, jt);
        let roi = way.induced * (local + trade) / cost.max(1e-9);
        if roi < way.hurdle {
            book.spot_cooldown.insert(target, year + way.cool_years);
            continue;
        }
        let Some((raised, subs)) = subscribe(st, &[town, jt], cost) else {
            book.spot_cooldown.insert(target, year + 2);
            continue;
        };
        st.projects.push(project(way, serves, town, jt, target, buy_town, path, new_cells, need, raised, subs, year, roi, cost, W));
        net = Net::new(st, way);
    }
}

/// What a month's order of `rq` costs a line at `price`: the materials, and the
/// navvies' wages on top, `wages` to the shilling (as `econ::step_month` pays them).
fn month_bill(rq: &[f64], price: &[f64], wages: f64) -> f64 {
    (1.0 + wages) * rq.iter().zip(price).map(|(q, p)| q * p).sum::<f64>()
}

/// The month's order of materials for a line of the game's promoters, cut down to
/// what its cash will pay for at today's prices. The port's projects (months 0)
/// buy on regardless and are topped up afterwards; these never owe anything, so
/// dear rails slow a line down and a rush is held back by the money behind it.
pub fn within_means(pr: &Project, rq: &mut [f64], price: &[f64]) {
    if pr.months <= 0.0 {
        return;
    }
    let bill = month_bill(rq, price, navvy_share(pr));
    let cash = pr.cash.max(0.0);
    if bill > cash {
        // a hair under, so that rounding never leaves the line a farthing short
        let scale = cash / bill * (1.0 - 1e-12);
        rq.iter_mut().for_each(|q| *q *= scale);
    }
}

/// The monthly step for one way: lines short of cash for the month's materials
/// call on their subscribers, and finished lines open. Projects of other ways
/// are left as they are.
pub fn progress(st: &mut State, way: &Way, year_float: f64, events: &mut Vec<Event>) {
    if !st.projects.iter().any(|pr| pr.way == way.id) {
        return;
    }
    let mut book = std::mem::take((way.book_mut)(st));
    let projects = std::mem::take(&mut st.projects);
    let mut keep = Vec::with_capacity(projects.len());
    for mut pr in projects {
        if pr.way != way.id {
            keep.push(pr);
            continue;
        }
        // Half the money may still be owing, and prices may have risen since it was
        // raised: a line without the cash for next month's materials makes a call
        // on the landlords of its two towns, a tenth of their cash at most. Where a
        // town's landlords are in debt (they can be for decades, and a line that
        // could call on nobody else would wait as long) its households, who
        // subscribed with them, answer in their place: a fortieth of their cash,
        // about the 35 to 8 in which the two subscribe.
        let months = if pr.months > 0.0 { pr.months } else { 10.0 };
        let rq: Vec<f64> = (0..pr.order.len()).map(|g| pr.order[g].min(pr.need[g] / months)).collect();
        let mut short = month_bill(&rq, st.price.row(pr.town), navvy_share(&pr)) - pr.cash;
        for (n, t) in [pr.a, pr.b].into_iter().enumerate() {
            if short <= 0.0 || (n == 1 && pr.b == pr.a) {
                continue;
            }
            let (lords, frac) = if st.lord_cash[t] > 0.0 { (true, 0.1) } else { (false, 0.025) };
            let cash = if lords { &mut st.lord_cash } else { &mut st.hh_cash };
            let call = (cash[t].max(0.0) * frac).min(short);
            if call > 0.0 {
                cash[t] -= call;
                pr.cash += call;
                pr.paid_in += call;
                match pr.subs.iter_mut().find(|s| s.0 == lords && s.1 == t) {
                    Some(s) => s.2 += call,
                    None => pr.subs.push((lords, t, call)),
                }
                short -= call;
            }
        }
        if pairwise_sum(&pr.order) > 0.01 * pairwise_sum(&pr.need) {
            keep.push(pr);
            continue;
        }
        // A line that runs over, or branches from, one still building waits for it:
        // a branch does not open onto a junction that is not there yet. (The other
        // line was promoted first and cannot be waiting for this one.)
        let layer = (way.layer)(st);
        if pr.cells.iter().any(|c| layer[*c] == 0 && !pr.new.contains(c)) {
            keep.push(pr);
            continue;
        }
        let first = if way.first_any { book.lines.is_empty() } else { !book.lines.iter().any(|l| l.serves == SERVES_TRUNK) };
        // Only its own cells: where its route runs over a line still building, that
        // line lays them when it opens (and pays for them, once).
        for &c in &pr.new {
            (way.layer_mut)(st)[c] = 1;
        }
        // what is left goes back to those who subscribed
        if let Some(opened) = way.opened {
            opened(st, &pr, year_float);
        }
        let subs = std::mem::take(&mut pr.subs);
        repay(st, &subs, pr.cash);
        let spent = pr.paid_in - pr.cash;
        pr.cash = 0.0;
        book.lines.push(Line {
            start: pr.start,
            opened: year_float,
            serves: pr.serves,
            a: pr.a,
            b: pr.b,
            target: pr.target,
            new_cells: pr.new.len(),
            km: pr.length_km,
            cost: pr.cost,
            roi: pr.roi,
            spent,
        });
        let trunk = pr.serves == SERVES_TRUNK;
        if way.id == RAILWAY {
            // the record's list of opened railways (b is -1 for a branch, as for the port's spurs)
            st.rail_opened.push(RailOpened { year: year_float, a: pr.a, b: if trunk { pr.b as i64 } else { -1 }, km: pr.length_km });
        }
        let (a_, b_) = (pr.a, pr.b);
        if trunk {
            let title = format!(
                "{}{} to {}",
                if first { format!("The first {} opens: ", way.words.noun) } else { format!("{} opens: ", way.words.Noun) },
                st.town_name[a_],
                st.town_name[b_]
            );
            let mut ev = Event::new(
                year_float,
                way.words.kind,
                if first { 1.0 } else { 0.45 },
                title,
                format!(
                    "{:.0} km of {}, begun in {}. Freight between the towns now moves at a fraction of the carting cost.",
                    pr.length_km, way.words.track, pr.start
                ),
            );
            ev.town = Some(a_);
            ev.cell = Some(st.town_node[a_]);
            ev.cell2 = Some(st.town_node[b_]);
            events.push(ev);
        } else if pr.serves == SERVES_TOWN {
            let from = if a_ == b_ {
                "the line that passes it".to_string()
            } else {
                format!("the network near {}", st.town_name[b_])
            };
            let mut ev = Event::new(
                year_float,
                way.words.kind,
                0.45,
                format!("The {} reaches {}", way.words.noun, st.town_name[a_]),
                format!("{:.0} km of {} join {} to {from}. The line was begun in {}.", pr.length_km, way.words.track, st.town_name[a_], pr.start),
            );
            ev.town = Some(a_);
            ev.cell = Some(pr.target);
            ev.cell2 = Some(pr.cells[0]);
            events.push(ev);
        } else if first && way.first_any {
            // the first line of all runs to a district (`Way::first_any`)
            let what = serves_word(pr.serves);
            let mut ev = Event::new(
                year_float,
                way.words.kind,
                1.0,
                format!("The first {} opens: the {what} near {}", way.words.noun, st.town_name[a_]),
                format!(
                    "{:.0} km of {}, begun in {}. What the {what} send to {} now goes by water at a fraction of the carting cost.",
                    pr.length_km, way.words.track, pr.start, st.town_name[a_]
                ),
            );
            ev.town = Some(a_);
            ev.cell = Some(pr.target);
            ev.cell2 = Some(pr.cells[0]);
            events.push(ev);
        } else {
            // district branches make one news item a year (`report_branches`)
            book.unreported.push(book.lines.len() - 1);
        }
    }
    st.projects = keep;
    *(way.book_mut)(st) = book;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{default_content, default_params, init_state};

    /// The island of seed 3 as it starts, and the railway's rules.
    fn island() -> (State, Way) {
        let (P, C) = (default_params(), default_content());
        let (st, _) = init_state(&P, &C);
        let way = crate::railways::way(&st, &C, &P);
        (st, way)
    }

    /// A route between the first two towns, as the promoters would find it.
    fn a_line(st: &State, way: &Way) -> Vec<usize> {
        let route = build_graph(&st.world, &vec![false; st.N], &way.build);
        let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
        let (na, nb) = (st.town_node[act[0]], st.town_node[act[1]]);
        let paths = tr::dijkstra(&route, &[(na, 0.0)], Some(nb));
        let path = tr::path_cells(&paths.pred, na, nb);
        assert!(path.len() > 3, "the two towns are joined over land");
        path
    }

    #[test]
    fn the_railways_route_graph_is_the_ports() {
        let (mut st, way) = island();
        for c in a_line(&st, &way) {
            st.rail[c] = 1;
        }
        let laid: Vec<bool> = st.rail.iter().map(|&r| r > 0).collect();
        let (ours, port) = (build_graph(&st.world, &laid, &way.build), tr::rail_build_graph(&st.world, &st.rail));
        assert!(ours.w.iter().zip(&port.w).all(|(x, y)| x.to_bits() == y.to_bits()));
    }

    #[test]
    fn promoters_are_not_fooled_by_a_price_nobody_pays() {
        let (mut st, _) = island();
        let C = default_content();
        let g = C.gi("rails");
        let (maker, other) = (0, 1);
        let mut pbar = Mat::zeros(st.T_max, C.G);
        pbar[(maker, g)] = 10.0;
        pbar[(other, g)] = 0.5; // nobody there ever bought a rail
        st.town_prod_year = Mat::zeros(st.T_max, C.G);
        // before anyone makes rails there is only the town's own price to go by
        assert_eq!(reckoned_price(&st, &pbar, g, other, 600.0, 24.0), 0.5);
        st.town_prod_year[(maker, g)] = 1200.0;
        // the making town fills an order within its make at its own price
        assert_eq!(reckoned_price(&st, &pbar, g, maker, 600.0, 24.0), 10.0);
        // the other town's rails must come from the makers, at what they get
        assert_eq!(reckoned_price(&st, &pbar, g, other, 600.0, 24.0), 10.0);
        // unless it makes some itself: a quarter of the order here, at its own price
        st.town_prod_year[(other, g)] = 75.0;
        let makers = (1200.0 * 10.0 + 75.0 * 0.5) / 1275.0;
        let want = 0.25 * 0.5 + 0.75 * makers;
        assert!((reckoned_price(&st, &pbar, g, other, 600.0, 24.0) - want).abs() < 1e-12);
        // and where rails are dearer than the makers' price, the town's price stands
        pbar[(other, g)] = 14.0;
        assert_eq!(reckoned_price(&st, &pbar, g, other, 600.0, 24.0), 14.0);
    }

    #[test]
    fn a_line_whose_landlords_are_in_debt_calls_on_the_households() {
        let (mut st, way) = island();
        let path = a_line(&st, &way);
        let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
        let (a, b) = (act[0], act[1]);
        let need = materials(&way, &st.world, &path, &path);
        let W = st.world.W;
        st.projects.push(project(&way, SERVES_TRUNK, a, b, path[path.len() - 1], a, path.clone(), path, need, 0.0, Vec::new(), 1800, 0.1, 1.0, W));
        for t in [a, b] {
            st.lord_cash[t] = -1000.0;
            st.hh_cash[t] = 1e9;
        }
        let before = crate::econ::money_total(&st);
        progress(&mut st, &way, 1800.5, &mut Vec::new());
        let pr = &st.projects[0];
        // the month's bill, all of it from the first town's households
        let rq: Vec<f64> = pr.need.iter().map(|n| n / way.build_months).collect();
        let bill = month_bill(&rq, st.price.row(a), 0.3);
        assert!(bill > 0.0 && (pr.cash - bill).abs() <= 1e-9 * bill, "{} called against a bill of {bill}", pr.cash);
        assert_eq!(pr.subs, vec![(false, a, pr.cash)]);
        assert_eq!((st.lord_cash[a], st.lord_cash[b], st.hh_cash[b]), (-1000.0, -1000.0, 1e9));
        let after = crate::econ::money_total(&st);
        assert!((after - before).abs() <= 1e-9 * before.abs(), "a call is a transfer");
        // landlords with anything at all answer for themselves, a tenth of it each
        st.projects[0].cash = 0.0;
        for t in [a, b] {
            st.lord_cash[t] = 10.0;
        }
        progress(&mut st, &way, 1800.6, &mut Vec::new());
        assert!((st.projects[0].cash - 2.0).abs() < 1e-12 && st.lord_cash[a] == 9.0 && st.lord_cash[b] == 9.0);
        assert_eq!(st.hh_cash[b], 1e9);
    }

    #[test]
    fn laying_a_line_and_improving_is_a_search_from_scratch() {
        let (st, way) = island();
        let line = a_line(&st, &way);
        let mut g = st.graph.clone();
        let before = g.w.clone();
        let towns: Vec<(usize, f64)> = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| (st.town_node[t], 0.0)).collect();
        let base = tr::distances(&g, &towns);
        let mut laid = vec![false; st.N];
        for &c in &line {
            laid[c] = true;
        }
        let changed = lay(&mut g, &st.world, &laid, &line, way.run_cost);
        assert!(!changed.is_empty(), "a railway is cheaper than the road it follows");
        let quick = improve(&g, &base, &changed);
        let fresh = tr::distances(&g, &towns);
        let mut saved = 0.0;
        for c in 0..st.N {
            if fresh[c].is_finite() {
                assert!((quick[c] - fresh[c]).abs() <= 1e-9 * fresh[c].max(1.0), "cell {c}: {} against {}", quick[c], fresh[c]);
                saved += base[c] - fresh[c];
            } else {
                assert!(!quick[c].is_finite());
            }
        }
        assert!(saved > 0.0, "the line brings some cells nearer a market");
        // and taking it up again leaves the graph as it was
        unlay(&mut g, &changed);
        assert!(g.w.iter().zip(&before).all(|(x, y)| x.to_bits() == y.to_bits()));
    }
}
