//! The atlas: plates that draw the island whole, for the eye. Not in the
//! Python; a tool, not a rule, so it has no switch and changes nothing a run
//! does. `docs/atlas.md` is its page.
//!
//! ```text
//! isostate-core atlas making --out DIR [--seeds 1-8] [--rules port|game] [--families N] [--end-year Y]
//!                            [--set NAME=VALUE]... [--step 10] [--no-towns] [--jobs N] [--quiet]
//! isostate-core atlas making --from DIR [--out DIR2]
//! ```
//!
//! The first plate, "making", draws the production tree as the islands live
//! it: the land's sectors and the pits on the left, then goods and kinds of
//! works in turn, and on the right what households buy, by need, with the
//! railways and canals beside them. Every line is a flow of a year, averaged
//! over a period (a decade unless `--step` says otherwise): what a works uses
//! up, what it buys for its plant, what it makes, what households buy. Each
//! good is coloured by its price against the wage, set against the first
//! period. The drawing is laid out here from the content the runs were played
//! on and from what the books saw, with no place set by hand, so a good or a
//! kind of works added to the data is drawn the next time the command runs.
//!
//! The figures come from the books of account (`accounts.rs`), which keep a
//! year's figures for every town and good, every town and method, every works
//! and every (town, method, good) a works used. They are on under the game's
//! rules; the atlas refuses a run without them. Nothing new is recorded during
//! the run: the islands are played as a sweep plays them, and the books are
//! read once each run is over. Three things the books do not say are shared
//! out by rule, and the page says so: what the works bought for their plant
//! (kept by town and good, and by works in money), what the land bought (by
//! town and good, and by sector in money), and which need a household's
//! purchase of a good served (kept by need in money).
//!
//! What it writes to DIR: `making.html` (the page, one file that opens without
//! the network), `making.json` (the figures the page carries, which `--from`
//! reads to draw the page again without playing), `making.txt` (what the page
//! shows, in words: the dearest goods by period, what moved most, the growth
//! after 1900, the recipe usage) and `README.txt`.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crate::accounts::{GOODS_FIELDS, KG, KL, KM, KT, KW, LAND_FIELDS, METHOD_FIELDS, TOWN_FIELDS, WORKS_FIELDS};
use crate::cli::{check_end_year, parse_rules};
use crate::content::Content;
use crate::params::{Params, Rules};
use crate::sim::RunResult;
use crate::sweep;

pub const USAGE: &str = "usage: isostate-core atlas making --out DIR [--seeds 1-8] [--rules port|game] [--families N] [--end-year Y]
                                  [--set NAME=VALUE]... [--step 10] [--no-towns] [--jobs N] [--quiet]
       isostate-core atlas making --from DIR [--out DIR2]   (draw the page again from DIR/making.json)";

/// The plates, in the order the backlog ranks them; the first and the third
/// are built (the third in `atlas_ideas.rs`).
pub const PLATES: [&str; 4] = ["making", "moving", "ideas", "ground"];

/// What the command line asked for.
#[derive(Clone, Debug)]
pub struct Options {
    pub out: PathBuf,
    pub seeds: Vec<u64>,
    pub rules: Rules,
    pub families: usize,
    pub end_year: i64,
    /// Each `--set`: a rule's dotted name and its one value, as typed.
    pub sets: Vec<(String, String)>,
    /// Years in a period.
    pub step: i64,
    /// Keep every town's figures, for the town picker.
    pub towns: bool,
    /// Draw the page again from this folder's `making.json`.
    pub from: Option<PathBuf>,
    pub jobs: Option<usize>,
    pub quiet: bool,
    pub command: String,
}

pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let plate = args.first().map(String::as_str);
    match plate {
        Some("making") => {}
        Some("ideas") => return Err(format!("the plate \"ideas\" reads its own command line (atlas_ideas::parse_args)\n{}", crate::atlas_ideas::USAGE)),
        Some(p) if PLATES.contains(&p) => return Err(format!("the plate {p:?} is designed (docs/BACKLOG.md, \"The atlas\") and not built; \"making\" and \"ideas\" are\n{USAGE}")),
        Some(p) => return Err(format!("{p:?} is not a plate of the atlas; the plates are {}\n{USAGE}", PLATES.join(", "))),
        None => return Err(format!("name the plate: making\n{USAGE}")),
    }
    let mut o = Options {
        out: PathBuf::new(),
        seeds: (1..=8).collect(),
        rules: Rules::Game,
        families: 6,
        end_year: 2050,
        sets: Vec::new(),
        step: 10,
        towns: true,
        from: None,
        jobs: None,
        quiet: false,
        command: command_line(args),
    };
    let mut out: Option<PathBuf> = None;
    let mut it = args[1..].iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value\n{USAGE}"));
        match a.as_str() {
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            "--seeds" => o.seeds = sweep::parse_seeds(&value("--seeds")?)?,
            "--rules" => o.rules = parse_rules(&value("--rules")?)?,
            "--families" => o.families = value("--families")?.parse().map_err(|e| format!("--families: {e}"))?,
            "--end-year" => o.end_year = value("--end-year")?.parse().map_err(|e| format!("--end-year: {e}"))?,
            "--set" => {
                let v = value("--set")?;
                let (k, x) = v.split_once('=').ok_or_else(|| format!("--set {v:?}: write it as NAME=VALUE"))?;
                if sweep::split_values(x).len() > 1 {
                    return Err(format!("--set {v:?}: the atlas draws one setting; a sweep of several is sweep --set's"));
                }
                o.sets.push((k.trim().to_string(), x.trim().to_string()));
            }
            "--step" => {
                o.step = value("--step")?.parse().map_err(|e| format!("--step: {e}"))?;
                if o.step < 1 {
                    return Err("--step: a period is at least a year".to_string());
                }
            }
            "--no-towns" => o.towns = false,
            "--from" => o.from = Some(PathBuf::from(value("--from")?)),
            "--jobs" => o.jobs = Some(value("--jobs")?.parse::<usize>().map_err(|e| format!("--jobs: {e}"))?.max(1)),
            "--quiet" => o.quiet = true,
            other => return Err(format!("unknown option {other:?}\n{USAGE}")),
        }
    }
    o.out = match (out, &o.from) {
        (Some(d), _) => d,
        (None, Some(f)) => f.clone(),
        (None, None) => return Err(format!("--out is required\n{USAGE}")),
    };
    if o.seeds.is_empty() {
        return Err("--seeds: no seeds".to_string());
    }
    Ok(o)
}

/// The rules the islands are played under: the chosen set, the families, the
/// end year and every `--set`, checked before anything runs.
pub fn params(o: &Options) -> Result<(Params, Content), String> {
    let mut P = o.rules.params();
    P.n_families = o.families;
    P.end_year = Some(o.end_year);
    for (k, v) in &o.sets {
        P = sweep::set_field(&P, k, &sweep::parse_value(v))?;
    }
    P.check()?;
    if !P.accounts.enabled {
        return Err("the atlas reads the books of account, and these rules keep none: add --set accounts.enabled=true (they are on under the game's rules)".to_string());
    }
    let content = o.rules.content();
    check_end_year(&P, &content)?;
    Ok((P, content))
}

// ---------------------------------------------------------------- the drawing's skeleton

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// A use of the land: a farm, a pasture, the woods, a quarry, a pit or a mine.
    Sector,
    Good,
    /// A kind of works (a method).
    Works,
    /// A household need.
    Need,
    /// The railways and canals being built.
    Ways,
}

impl NodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeKind::Sector => "sector",
            NodeKind::Good => "good",
            NodeKind::Works => "works",
            NodeKind::Need => "need",
            NodeKind::Ways => "ways",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeKind {
    /// A sector or a works makes a good.
    Make,
    /// A works uses a good up, as material or as fuel.
    Use,
    /// A works, a sector or a railway buys a good to hold as plant.
    Plant,
    /// Households buy a good for a need.
    Buy,
}

impl EdgeKind {
    pub fn code(self) -> u8 {
        match self {
            EdgeKind::Make => 0,
            EdgeKind::Use => 1,
            EdgeKind::Plant => 2,
            EdgeKind::Buy => 3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: NodeKind,
    /// Its place in the content's own list of its kind (0 for the ways).
    pub index: usize,
    pub name: String,
    pub label: String,
    pub col: usize,
    pub row: usize,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub kind: EdgeKind,
    /// The good that flows along it.
    pub good: usize,
}

/// Every flow the books can show, as a place in one dense array (`Keys`), so
/// that a year's figures can be added up before the drawing is laid out.
#[derive(Clone, Copy, Debug)]
pub struct Keys {
    pub G: usize,
    pub M: usize,
    pub NS: usize,
    pub S: usize,
}

impl Keys {
    pub fn of(C: &Content) -> Keys {
        Keys { G: C.G, M: C.M, NS: C.NS, S: C.S }
    }
    pub fn make_sector(&self, s: usize) -> usize {
        s
    }
    pub fn make_works(&self, m: usize) -> usize {
        self.NS + m
    }
    pub fn uses(&self, m: usize, g: usize) -> usize {
        self.NS + self.M + m * self.G + g
    }
    pub fn plant(&self, m: usize, g: usize) -> usize {
        self.NS + self.M + (self.M + m) * self.G + g
    }
    pub fn land(&self, s: usize, g: usize) -> usize {
        self.NS + self.M + 2 * self.M * self.G + s * self.G + g
    }
    pub fn ways(&self, g: usize) -> usize {
        self.NS + self.M + 2 * self.M * self.G + self.NS * self.G + g
    }
    pub fn buy(&self, g: usize, s: usize) -> usize {
        self.NS + self.M + 2 * self.M * self.G + self.NS * self.G + self.G + g * self.S + s
    }
    pub fn len(&self) -> usize {
        self.NS + self.M + 2 * self.M * self.G + self.NS * self.G + self.G + self.G * self.S
    }
}

/// The drawing: its nodes in place and its lines, and for every line the
/// place in `Keys` its figures come from.
#[derive(Clone, Debug)]
pub struct Layout {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub key_of_edge: Vec<usize>,
    pub width: f64,
    pub height: f64,
    pub cols: usize,
    /// Node of each sector, good, method and need, and of the ways.
    pub sector_node: Vec<usize>,
    pub good_node: Vec<usize>,
    pub works_node: Vec<usize>,
    pub need_node: Vec<usize>,
    pub ways_node: usize,
}

/// Each method's depth in the tree and each good's: a good the land makes is
/// 0, a method is one more than the deepest of its materials, and a good is as
/// deep as its shallowest maker. Fuel and plant are left out of the reckoning,
/// since coal and machinery go into nearly everything. A cycle of materials
/// with no way in from the land is cut where it is met.
pub fn depths(C: &Content) -> (Vec<usize>, Vec<usize>) {
    const NONE: usize = usize::MAX;
    let mut good = vec![NONE; C.G];
    for s in 0..C.NS {
        good[C.sector_good[s]] = 0;
    }
    let mut method = vec![NONE; C.M];
    let inputs: Vec<Vec<usize>> = (0..C.M).map(|m| (0..C.G).filter(|&g| C.m_in[(m, g)] > 0.0).collect()).collect();
    // first only methods whose materials are all placed, then any with one placed
    for strict in [true, false] {
        for _ in 0..=(C.G + C.M) {
            let mut changed = false;
            for m in 0..C.M {
                let placed: Vec<usize> = inputs[m].iter().map(|&g| good[g]).filter(|&d| d != NONE).collect();
                if strict && placed.len() < inputs[m].len() {
                    continue;
                }
                if !strict && placed.is_empty() && !inputs[m].is_empty() {
                    continue;
                }
                let d = placed.iter().max().map_or(1, |x| x + 1);
                if method[m] == NONE || (strict && d > method[m]) {
                    method[m] = d;
                    changed = true;
                }
                let g = C.method_out[m];
                if good[g] == NONE || d < good[g] {
                    good[g] = d;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }
    // whatever is still unplaced stands with the land's goods and the first works
    for d in good.iter_mut().filter(|d| **d == NONE) {
        *d = 0;
    }
    for d in method.iter_mut().filter(|d| **d == NONE) {
        *d = 1;
    }
    (good, method)
}

/// The lines the content says may carry something, whether or not any run
/// sends anything along them: each recipe's materials and fuels, its plant,
/// the machines' run goods and plant (`data/tasks.toml`), the land's machinery
/// and rails, and every good that serves a need.
pub fn content_keys(C: &Content) -> BTreeSet<usize> {
    let k = Keys::of(C);
    let mut out = BTreeSet::new();
    for s in 0..C.NS {
        out.insert(k.make_sector(s));
    }
    let fuels: Vec<usize> = (0..C.G).filter(|&g| C.heat.get(g).copied().unwrap_or(0.0) > 0.0).collect();
    for m in 0..C.M {
        out.insert(k.make_works(m));
        for g in 0..C.G {
            if C.m_in[(m, g)] > 0.0 {
                out.insert(k.uses(m, g));
            }
            if C.m_cap[(m, g)] > 0.0 {
                out.insert(k.plant(m, g));
            }
        }
        if C.m_heat[m] > 0.0 {
            for &g in &fuels {
                out.insert(k.uses(m, g));
            }
        }
        // the machines for tasks this kind of works may take up
        let shares = C.tasks.method.get(m).copied().unwrap_or_default();
        for u in C.tasks.uses.iter().filter(|u| u.on_works && shares.get(u.class).copied().unwrap_or(0.0) > 0.0) {
            for &(g, _) in &u.run {
                out.insert(k.uses(m, g));
            }
            for &(g, _) in &u.plant {
                out.insert(k.plant(m, g));
            }
        }
    }
    for s in 0..C.NS {
        for u in C.tasks.uses.iter().filter(|u| u.on_sector.get(s).copied().unwrap_or(false)) {
            for &(g, _) in u.run.iter().chain(&u.plant) {
                out.insert(k.land(s, g));
            }
        }
    }
    for g in 0..C.G {
        for s in 0..C.S {
            if C.props[(g, s)] > 0.0 {
                out.insert(k.buy(g, s));
            }
        }
    }
    out
}

/// The drawing laid out from the content and from the lines any island used:
/// columns by depth, from the land on the left to the needs on the right, and
/// within each column an order that keeps lines short (a few passes of the
/// mean place of each node's neighbours).
pub fn layout(C: &Content, used: &BTreeSet<usize>) -> Layout {
    let k = Keys::of(C);
    let (gd, md) = depths(C);
    let mut nodes: Vec<Node> = Vec::new();
    let mut push = |kind: NodeKind, index: usize, name: &str, label: &str, col: usize| {
        nodes.push(Node { kind, index, name: name.to_string(), label: label.to_string(), col, row: 0, x: 0.0, y: 0.0 });
        nodes.len() - 1
    };
    let sector_node: Vec<usize> = (0..C.NS).map(|s| push(NodeKind::Sector, s, &C.sector_name[s], &C.sector_label[s], 0)).collect();
    let good_node: Vec<usize> = (0..C.G).map(|g| push(NodeKind::Good, g, &C.good_name[g], &C.good_label[g], 2 * gd[g] + 1)).collect();
    let works_node: Vec<usize> = (0..C.M).map(|m| push(NodeKind::Works, m, &C.method_name[m], &C.method_label[m], 2 * md[m])).collect();
    let last = gd.iter().map(|d| 2 * d + 1).chain(md.iter().map(|d| 2 * d)).max().unwrap_or(1) + 1;
    let need_node: Vec<usize> = (0..C.S).map(|s| push(NodeKind::Need, s, &C.service_name[s], &C.service_label[s], last)).collect();
    let ways_node = push(NodeKind::Ways, 0, "ways", "Railways and canals", last);

    // the lines: every key the content names or a run used
    let mut keys: BTreeSet<usize> = content_keys(C);
    keys.extend(used.iter().copied());
    let mut edges = Vec::new();
    let mut key_of_edge = Vec::new();
    let mut add = |e: Edge, key: usize| {
        edges.push(e);
        key_of_edge.push(key);
    };
    for s in 0..C.NS {
        if keys.contains(&k.make_sector(s)) {
            add(Edge { from: sector_node[s], to: good_node[C.sector_good[s]], kind: EdgeKind::Make, good: C.sector_good[s] }, k.make_sector(s));
        }
    }
    for m in 0..C.M {
        let g = C.method_out[m];
        add(Edge { from: works_node[m], to: good_node[g], kind: EdgeKind::Make, good: g }, k.make_works(m));
        for g in 0..C.G {
            if keys.contains(&k.uses(m, g)) {
                add(Edge { from: good_node[g], to: works_node[m], kind: EdgeKind::Use, good: g }, k.uses(m, g));
            }
            if keys.contains(&k.plant(m, g)) {
                add(Edge { from: good_node[g], to: works_node[m], kind: EdgeKind::Plant, good: g }, k.plant(m, g));
            }
        }
    }
    for s in 0..C.NS {
        for g in 0..C.G {
            if keys.contains(&k.land(s, g)) {
                // what the land buys and burns (a machine's coal, oil or power) is used up; the rest is plant
                let burnt = C.tasks.uses.iter().any(|u| u.run.iter().any(|&(x, _)| x == g));
                let kind = if burnt { EdgeKind::Use } else { EdgeKind::Plant };
                add(Edge { from: good_node[g], to: sector_node[s], kind, good: g }, k.land(s, g));
            }
        }
    }
    for g in 0..C.G {
        if keys.contains(&k.ways(g)) {
            add(Edge { from: good_node[g], to: ways_node, kind: EdgeKind::Plant, good: g }, k.ways(g));
        }
        for s in 0..C.S {
            if keys.contains(&k.buy(g, s)) {
                add(Edge { from: good_node[g], to: need_node[s], kind: EdgeKind::Buy, good: g }, k.buy(g, s));
            }
        }
    }

    // the order within each column
    let cols = last + 1;
    let mut by_col: Vec<Vec<usize>> = vec![Vec::new(); cols];
    for (i, n) in nodes.iter().enumerate() {
        by_col[n.col].push(i);
    }
    let mut pos = vec![0.0f64; nodes.len()];
    let place = |by_col: &Vec<Vec<usize>>, pos: &mut Vec<f64>| {
        for list in by_col {
            let h = list.len() as f64;
            for (r, &i) in list.iter().enumerate() {
                pos[i] = (r as f64 + 0.5) / h.max(1.0);
            }
        }
    };
    place(&by_col, &mut pos);
    // the lines of the tree proper pull nodes into line; plant lines, which run back across it, do not
    let tree: Vec<(usize, usize)> = edges.iter().filter(|e| e.kind != EdgeKind::Plant).map(|e| (e.from, e.to)).collect();
    for pass in 0..8 {
        let forward = pass % 2 == 0;
        let order: Vec<usize> = if forward { (1..cols).collect() } else { (0..cols - 1).rev().collect() };
        for c in order {
            let mut keyed: Vec<(f64, usize)> = by_col[c]
                .iter()
                .map(|&i| {
                    let near: Vec<f64> = tree
                        .iter()
                        .filter_map(|&(a, b)| {
                            if forward && b == i && nodes[a].col < c {
                                Some(pos[a])
                            } else if !forward && a == i && nodes[b].col > c {
                                Some(pos[b])
                            } else {
                                None
                            }
                        })
                        .collect();
                    let m = if near.is_empty() { pos[i] } else { near.iter().sum::<f64>() / near.len() as f64 };
                    (m, i)
                })
                .collect();
            keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            by_col[c] = keyed.into_iter().map(|(_, i)| i).collect();
            place(&by_col, &mut pos);
        }
    }
    // a works stands beside the good it makes where it can: goods keep their order, and
    // a column's rows are spaced over the tallest column's height
    let tallest = by_col.iter().map(|l| l.len()).max().unwrap_or(1) as f64;
    let (row_h, col_w, top, left) = (30.0, 168.0, 24.0, 70.0);
    let height = top * 2.0 + tallest * row_h;
    for (c, list) in by_col.iter().enumerate() {
        let n = list.len() as f64;
        let gap = (height - 2.0 * top) / n.max(1.0);
        for (r, &i) in list.iter().enumerate() {
            nodes[i].row = r;
            nodes[i].x = left + c as f64 * col_w;
            nodes[i].y = top + (r as f64 + 0.5) * gap;
        }
    }
    let width = left * 2.0 + (cols - 1) as f64 * col_w;
    Layout { nodes, edges, key_of_edge, width, height, cols, sector_node, good_node, works_node, need_node, ways_node }
}

// ---------------------------------------------------------------- reading the books

/// The column of a books table by its name.
fn col(fields: &[&str], name: &str) -> usize {
    fields.iter().position(|f| *f == name).unwrap_or_else(|| panic!("the books have no column {name:?}"))
}

/// One place's figures over one period, added up year by year; `Figures` is
/// what is left once the period is over.
#[derive(Clone, Debug, Default)]
struct Acc {
    years: f64,
    wage_pop: f64,
    pop: f64,
    /// price times what was bought, and what was bought; price times people, for a good nobody bought
    price_q: Vec<f64>,
    bought: Vec<f64>,
    price_pop: Vec<f64>,
    /// each year's price (the towns' weighted mean, for the island), added up over the years
    price_yr: Vec<f64>,
    made: Vec<f64>,
    m_out: Vec<f64>,
    m_hands: Vec<f64>,
    m_works: Vec<f64>,
    s_out: Vec<f64>,
    s_hands: Vec<f64>,
    keys: Vec<f64>,
}

impl Acc {
    fn new(k: &Keys) -> Acc {
        Acc {
            price_q: vec![0.0; k.G],
            bought: vec![0.0; k.G],
            price_pop: vec![0.0; k.G],
            price_yr: vec![0.0; k.G],
            made: vec![0.0; k.G],
            m_out: vec![0.0; k.M],
            m_hands: vec![0.0; k.M],
            m_works: vec![0.0; k.M],
            s_out: vec![0.0; k.NS],
            s_hands: vec![0.0; k.NS],
            keys: vec![0.0; k.len()],
            ..Acc::default()
        }
    }
    fn add(&mut self, o: &Acc) {
        self.wage_pop += o.wage_pop;
        self.pop += o.pop;
        let pairs: [(&mut Vec<f64>, &Vec<f64>); 11] = [
            (&mut self.price_yr, &o.price_yr),
            (&mut self.price_q, &o.price_q),
            (&mut self.bought, &o.bought),
            (&mut self.price_pop, &o.price_pop),
            (&mut self.made, &o.made),
            (&mut self.m_out, &o.m_out),
            (&mut self.m_hands, &o.m_hands),
            (&mut self.m_works, &o.m_works),
            (&mut self.s_out, &o.s_out),
            (&mut self.s_hands, &o.s_hands),
            (&mut self.keys, &o.keys),
        ];
        for (a, b) in pairs {
            for (x, y) in a.iter_mut().zip(b) {
                *x += y;
            }
        }
    }
    fn figures(&self) -> Option<Figures> {
        if self.years <= 0.0 || self.pop <= 0.0 {
            return None;
        }
        let n = self.years;
        let per = |v: &Vec<f64>| v.iter().map(|x| x / n).collect::<Vec<f64>>();
        // the mean of the years' prices: a year of famine prices counts as a year, however little was bought in it
        let price = per(&self.price_yr);
        Some(Figures {
            wage: self.wage_pop / self.pop,
            pop: self.pop / n,
            price,
            made: per(&self.made),
            m_out: per(&self.m_out),
            m_hands: per(&self.m_hands),
            m_works: per(&self.m_works),
            s_out: per(&self.s_out),
            s_hands: per(&self.s_hands),
            keys: per(&self.keys),
        })
    }
}

/// One place over one period: each figure a year's, averaged over the years of
/// the period in which the place had people. Money is in the period's own
/// money; the page turns it into wages.
#[derive(Clone, Debug, PartialEq)]
pub struct Figures {
    /// A month's wage: the towns' mean wage weighted by their people.
    pub wage: f64,
    pub pop: f64,
    /// Each good's price: the mean of the period's years, each year's the
    /// towns' mean price weighted by what was bought in each (by people where
    /// nothing was).
    pub price: Vec<f64>,
    pub made: Vec<f64>,
    pub m_out: Vec<f64>,
    pub m_hands: Vec<f64>,
    pub m_works: Vec<f64>,
    pub s_out: Vec<f64>,
    pub s_hands: Vec<f64>,
    /// Every flow of `Keys`, in money a year.
    pub keys: Vec<f64>,
}

/// One town's figures, period by period (None before it was founded or once it is gone).
#[derive(Clone, Debug)]
pub struct Town {
    pub index: usize,
    pub name: String,
    pub founded: i64,
    pub periods: Vec<Option<Figures>>,
}

/// What the books left out of the drawing, in money over the whole run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LeftOut {
    /// Plant bought in a town where no works paid for any.
    pub plant: f64,
    /// Bought by the land in a town where no sector paid for anything.
    pub land: f64,
    /// Bought by households of a good that serves no need.
    pub households: f64,
    /// Everything that is drawn, to set the rest against.
    pub drawn: f64,
}

/// One island, read from its books.
#[derive(Clone, Debug)]
pub struct Island {
    pub seed: u64,
    pub periods: Vec<Option<Figures>>,
    pub towns: Vec<Town>,
    /// Each method's year of finding (its first year for one known in 1750), None if never.
    pub found: Vec<Option<i64>>,
    /// Per method: first and last year any works of it worked, and its peak hands
    /// (the period's mean) with the period they peaked in.
    pub usage: Vec<Usage>,
    /// The observer's real wage against the first year, and the share of hands working
    /// the land, as each year's mean. The wage is the linked one where the run keeps it
    /// (`wage_linked`, docs/rules/measures.md), else the chained one; the field keeps its
    /// old name.
    pub wage_chained: Vec<f64>,
    pub wage_linked: bool,
    pub land_share: Vec<f64>,
    pub left_out: LeftOut,
    pub seconds: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Usage {
    pub first: Option<i64>,
    pub last: Option<i64>,
    pub peak_hands: f64,
    pub peak_period: Option<i64>,
}

/// The periods: the first year of each, from the start year by `step`, up to the end year.
pub fn periods(y0: i64, y1: i64, step: i64) -> Vec<i64> {
    (0..).map(|k| y0 + k * step).take_while(|&y| y <= y1).collect()
}

/// Read one finished run's books into the atlas's figures.
pub fn read_run(r: &RunResult, seed: u64, step: i64, towns: bool, seconds: f64) -> Result<Island, String> {
    let (st, C) = (&r.state, &r.content);
    let b = &st.acct;
    if !b.on {
        return Err("the run kept no books of account (accounts.enabled is off)".to_string());
    }
    let k = Keys::of(C);
    let (G, M, NS, S) = (C.G, C.M, C.NS, C.S);
    let y0 = b.start_year;
    let n_years = b.hist.n;
    let y1 = y0 + n_years as i64 - 1;
    let starts = periods(C.start_year, r.params.end_year.unwrap_or(C.end_year), step);
    let period_of = |y: i64| starts.iter().rposition(|&s| s <= y);

    // the books' columns, by name
    let (g_made, g_hh, g_lords, g_fam, g_in, g_fuel, g_land, g_build, g_proj, g_price) = (
        col(&GOODS_FIELDS, "made"),
        col(&GOODS_FIELDS, "households"),
        col(&GOODS_FIELDS, "landlords"),
        col(&GOODS_FIELDS, "families"),
        col(&GOODS_FIELDS, "works_inputs"),
        col(&GOODS_FIELDS, "works_fuel"),
        col(&GOODS_FIELDS, "land"),
        col(&GOODS_FIELDS, "building"),
        col(&GOODS_FIELDS, "projects"),
        col(&GOODS_FIELDS, "price"),
    );
    let (l_out, l_value, l_hands, l_inputs) = (col(&LAND_FIELDS, "out"), col(&LAND_FIELDS, "value"), col(&LAND_FIELDS, "hands"), col(&LAND_FIELDS, "inputs"));
    let (m_out, m_value, m_works, m_hands) = (col(&METHOD_FIELDS, "out"), col(&METHOD_FIELDS, "value"), col(&METHOD_FIELDS, "works"), col(&METHOD_FIELDS, "hands"));
    let (t_pop, t_wage) = (col(&TOWN_FIELDS, "pop"), col(&TOWN_FIELDS, "wage"));
    let (w_method, w_town, w_invest) = (col(&WORKS_FIELDS, "method"), col(&WORKS_FIELDS, "town"), col(&WORKS_FIELDS, "invest"));

    // which sector may have bought a good: what its machines take, machinery anywhere, rails in the pits
    let mach = C.good_index.get("machinery").copied();
    let rails = C.good_index.get("rails").copied();
    let land_may = |s: usize, g: usize| -> bool {
        Some(g) == mach
            || (Some(g) == rails && C.sector_mine[s])
            || C.tasks.uses.iter().any(|u| u.on_sector.get(s).copied().unwrap_or(false) && u.run.iter().chain(&u.plant).any(|&(x, _)| x == g))
    };
    // each method's plant: what a unit of capacity holds of each good, at base prices, as shares
    let plant_share: Vec<Vec<f64>> = (0..M)
        .map(|m| {
            let tot: f64 = (0..G).map(|g| C.m_cap[(m, g)] * C.base_price[g]).sum();
            (0..G).map(|g| if tot > 0.0 { C.m_cap[(m, g)] * C.base_price[g] / tot } else { 0.0 }).collect()
        })
        .collect();

    let np = starts.len();
    let mut island_acc: Vec<Acc> = (0..np).map(|_| Acc::new(&k)).collect();
    let mut town_acc: Vec<Vec<Option<Acc>>> = Vec::new();
    let mut left = LeftOut::default();
    let mut usage = vec![Usage::default(); M];

    for yk in 0..n_years {
        let year = y0 + yk as i64;
        let Some(p) = period_of(year) else { continue };
        let Some(rec) = b.year(yk) else { continue };
        let T = rec.towns.len() / KT;
        if town_acc.len() < T {
            town_acc.resize_with(T, || vec![None; np]);
        }
        // invest by town and method, from the works' own books
        let mut invest = vec![0.0f64; T * M];
        for row in rec.works.chunks_exact(KW) {
            let (m, t) = (row[w_method] as usize, row[w_town] as usize);
            if m < M && t < T {
                invest[t * M + m] += row[w_invest] as f64;
            }
        }
        let mut year_acc = Acc::new(&k);
        year_acc.years = 1.0;
        for t in 0..T {
            let tr = &rec.towns[t * KT..][..KT];
            let pop = tr[t_pop] as f64;
            if !(pop > 0.0) {
                continue;
            }
            let mut a = Acc::new(&k);
            a.years = 1.0;
            a.pop = pop;
            a.wage_pop = tr[t_wage] as f64 * pop;
            let gr = |g: usize| &rec.goods[(t * G + g) * KG..][..KG];
            let price: Vec<f64> = (0..G).map(|g| gr(g)[g_price] as f64).collect();
            for g in 0..G {
                let r = gr(g);
                let bought: f64 = [g_hh, g_lords, g_fam, g_in, g_fuel, g_land, g_build, g_proj].iter().map(|&c| r[c] as f64).sum();
                a.bought[g] = bought;
                a.price_q[g] = price[g] * bought;
                a.price_pop[g] = price[g] * pop;
                a.price_yr[g] = price[g];
                a.made[g] = r[g_made] as f64;
            }
            for s in 0..NS {
                let r = &rec.land[(t * NS + s) * KL..][..KL];
                a.s_out[s] = r[l_out] as f64;
                a.s_hands[s] = r[l_hands] as f64;
                a.keys[k.make_sector(s)] = r[l_value] as f64;
            }
            for m in 0..M {
                let r = &rec.methods[(t * M + m) * KM..][..KM];
                a.m_out[m] = r[m_out] as f64;
                a.m_hands[m] = r[m_hands] as f64;
                a.m_works[m] = r[m_works] as f64;
                a.keys[k.make_works(m)] = r[m_value] as f64;
            }
            // what the works used up, at the town's price
            for u in rec.uses.iter().filter(|u| u.town as usize == t) {
                let (m, g) = (u.method as usize, u.good as usize);
                if m < M && g < G {
                    a.keys[k.uses(m, g)] += (u.used as f64 + u.burnt as f64) * price[g];
                }
            }
            // plant: each good bought for building, shared among the town's works by what each
            // paid for plant and how much of the good its recipe's plant holds
            for g in 0..G {
                let v = gr(g)[g_build] as f64 * price[g];
                if v == 0.0 {
                    continue;
                }
                let w: Vec<f64> = (0..M).map(|m| invest[t * M + m] * plant_share[m][g]).collect();
                let (sw, si): (f64, f64) = (w.iter().sum(), (0..M).map(|m| invest[t * M + m]).sum());
                if sw > 0.0 {
                    for m in 0..M {
                        a.keys[k.plant(m, g)] += v * w[m] / sw;
                    }
                } else if si > 0.0 {
                    for m in 0..M {
                        a.keys[k.plant(m, g)] += v * invest[t * M + m] / si;
                    }
                } else {
                    left.plant += v;
                }
            }
            // the land's purchases, shared among the sectors that may have made them by what each paid
            let paid: Vec<f64> = (0..NS).map(|s| rec.land[(t * NS + s) * KL + l_inputs] as f64).collect();
            for g in 0..G {
                let v = gr(g)[g_land] as f64 * price[g];
                if v == 0.0 {
                    continue;
                }
                let may: Vec<f64> = (0..NS).map(|s| if land_may(s, g) { paid[s].max(0.0) } else { 0.0 }).collect();
                let all: Vec<f64> = paid.iter().map(|x| x.max(0.0)).collect();
                let w = if may.iter().sum::<f64>() > 0.0 { may } else { all };
                let sw: f64 = w.iter().sum();
                if sw > 0.0 {
                    for s in 0..NS {
                        a.keys[k.land(s, g)] += v * w[s] / sw;
                    }
                } else {
                    left.land += v;
                }
            }
            // railways and canals
            for g in 0..G {
                a.keys[k.ways(g)] += gr(g)[g_proj] as f64 * price[g];
            }
            // households, landlords and families: each good shared among the needs it serves,
            // by how much of each it gives and what the town spent on each
            let needs = &rec.needs[t * S * 2..][..S * 2];
            let spend: Vec<f64> = (0..S).map(|s| needs[s * 2] as f64 + needs[s * 2 + 1] as f64).collect();
            for g in 0..G {
                let r = gr(g);
                let v = (r[g_hh] as f64 + r[g_lords] as f64 + r[g_fam] as f64) * price[g];
                if v == 0.0 {
                    continue;
                }
                let w: Vec<f64> = (0..S).map(|s| C.props[(g, s)] * spend[s].max(0.0)).collect();
                let w = if w.iter().sum::<f64>() > 0.0 { w } else { (0..S).map(|s| C.props[(g, s)]).collect() };
                let sw: f64 = w.iter().sum();
                if sw > 0.0 {
                    for s in 0..S {
                        a.keys[k.buy(g, s)] += v * w[s] / sw;
                    }
                } else {
                    left.households += v;
                }
            }
            for m in 0..M {
                if a.m_out[m] > 0.0 || a.m_works[m] > 0.0 {
                    let u = &mut usage[m];
                    u.first = Some(u.first.map_or(year, |f| f.min(year)));
                    u.last = Some(u.last.map_or(year, |l| l.max(year)));
                }
            }
            year_acc.add(&a);
            if towns {
                let slot = town_acc[t][p].get_or_insert_with(|| {
                    let mut z = Acc::new(&k);
                    z.years = 0.0;
                    z
                });
                slot.add(&a);
                slot.years += 1.0;
            }
        }
        // the island's year is one year, whatever the number of its towns, and its price
        // the towns' weighted by what was bought in each (by people where nothing was)
        year_acc.years = 1.0;
        for g in 0..G {
            year_acc.price_yr[g] = if year_acc.bought[g] > 0.0 { year_acc.price_q[g] / year_acc.bought[g] } else { year_acc.price_pop[g] / year_acc.pop };
        }
        island_acc[p].add(&year_acc);
        island_acc[p].years += 1.0;
    }
    let periods: Vec<Option<Figures>> = island_acc.iter().map(|a| a.figures()).collect();
    for f in periods.iter().flatten() {
        left.drawn += f.keys.iter().sum::<f64>() * step as f64;
    }
    for m in 0..M {
        for (p, f) in periods.iter().enumerate() {
            if let Some(f) = f {
                if f.m_hands[m] > usage[m].peak_hands {
                    usage[m].peak_hands = f.m_hands[m];
                    usage[m].peak_period = Some(starts[p]);
                }
            }
        }
    }
    let town_list = if towns {
        town_acc
            .iter()
            .enumerate()
            .filter(|(_, ps)| ps.iter().any(|a| a.is_some()))
            .map(|(t, ps)| Town {
                index: t,
                name: st.town_name.get(t).cloned().unwrap_or_else(|| format!("town {t}")),
                founded: st.town_founded.get(t).copied().unwrap_or(y0),
                periods: ps.iter().map(|a| a.as_ref().and_then(|a| a.figures())).collect(),
            })
            .collect()
    } else {
        Vec::new()
    };
    // when each kind of works was found: in the content from the start, or the year the island found it
    let found = (0..M)
        .map(|m| {
            let t = (0..C.NT).find(|&t| C.tech_kind[t] == "method" && C.tech_name[t] == C.method_name[m]);
            match t {
                None => Some(C.start_year),
                Some(t) if C.tech_known[t] => Some(C.start_year),
                Some(t) => (st.tech_year[t] >= 0).then_some(st.tech_year[t]),
            }
        })
        .collect();
    let yearly = |name: &str| -> Vec<f64> {
        if !st.series.index.contains_key(name) {
            return Vec::new();
        }
        let s = st.series.get(name);
        (C.start_year..=y1).map(|y| crate::observer::annual(s, C.start_year, y)).collect()
    };
    // linked (docs/rules/measures.md): the wage the standing check reads, where the run keeps it
    let wage_linked = st.series.index.contains_key("real_wage_linked");
    let chained = yearly(if wage_linked { "real_wage_linked" } else { "real_wage_chained" });
    let first = chained.first().copied().unwrap_or(f64::NAN);
    Ok(Island {
        seed,
        periods,
        towns: town_list,
        found,
        usage,
        wage_chained: chained.iter().map(|w| w / first).collect(),
        wage_linked,
        land_share: yearly("ag_share"),
        left_out: left,
        seconds,
    })
}

/// The keys any island sent anything along.
pub fn used_keys(islands: &[Island]) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    for isl in islands {
        for f in isl.periods.iter().flatten() {
            out.extend(f.keys.iter().enumerate().filter(|(_, v)| **v != 0.0 && v.is_finite()).map(|(i, _)| i));
        }
    }
    out
}

/// The middle of the islands, figure by figure: the median of each across the
/// islands that have it (the mean of the two middle ones for an even count).
pub fn middle(islands: &[Island], np: usize) -> Vec<Option<Figures>> {
    fn med(mut v: Vec<f64>) -> f64 {
        v.retain(|x| x.is_finite());
        if v.is_empty() {
            return f64::NAN;
        }
        v.sort_by(f64::total_cmp);
        let n = v.len();
        if n % 2 == 1 { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) }
    }
    (0..np)
        .map(|p| {
            let have: Vec<&Figures> = islands.iter().filter_map(|i| i.periods.get(p).and_then(|f| f.as_ref())).collect();
            let first = have.first()?;
            let each = |f: &dyn Fn(&Figures) -> &Vec<f64>, n: usize| -> Vec<f64> { (0..n).map(|j| med(have.iter().map(|x| f(x)[j]).collect())).collect() };
            Some(Figures {
                wage: med(have.iter().map(|x| x.wage).collect()),
                pop: med(have.iter().map(|x| x.pop).collect()),
                price: each(&|x| &x.price, first.price.len()),
                made: each(&|x| &x.made, first.made.len()),
                m_out: each(&|x| &x.m_out, first.m_out.len()),
                m_hands: each(&|x| &x.m_hands, first.m_hands.len()),
                m_works: each(&|x| &x.m_works, first.m_works.len()),
                s_out: each(&|x| &x.s_out, first.s_out.len()),
                s_hands: each(&|x| &x.s_hands, first.s_hands.len()),
                keys: each(&|x| &x.keys, first.keys.len()),
            })
        })
        .collect()
}

// ---------------------------------------------------------------- what the page says in words

/// A good's price against the wage in a period, set against the base period's.
pub fn against_base(base: Option<&Figures>, now: &Figures, g: usize) -> f64 {
    match base {
        Some(b) => (now.price[g] / now.wage) / (b.price[g] / b.wage),
        None => f64::NAN,
    }
}

/// The first period in which a good was made: its price before that is a
/// ceiling nobody paid, so the comparison starts there.
pub fn base_period(periods: &[Option<Figures>], g: usize) -> Option<usize> {
    periods.iter().position(|f| f.as_ref().is_some_and(|f| f.made[g] > 0.0 && f.price[g].is_finite() && f.wage > 0.0))
}

/// The goods in order of their price against the wage set against their base, dearest first.
pub fn dearest(periods: &[Option<Figures>], p: usize) -> Vec<(usize, f64)> {
    let Some(Some(now)) = periods.get(p) else { return Vec::new() };
    let G = now.price.len();
    let mut out: Vec<(usize, f64)> = (0..G)
        .filter_map(|g| {
            let b = base_period(periods, g)?;
            if b > p || now.made[g] <= 0.0 {
                return None;
            }
            let r = against_base(periods[b].as_ref(), now, g);
            r.is_finite().then_some((g, r))
        })
        .collect();
    out.sort_by(|a, b| b.1.total_cmp(&a.1));
    out
}

/// The kinds of works that hold good `g` as plant in period `p`, the largest first, with the value a year.
pub fn holders(L: &Layout, f: &Figures, g: usize, kind: EdgeKind) -> Vec<(usize, f64)> {
    let mut out: Vec<(usize, f64)> = L
        .edges
        .iter()
        .zip(&L.key_of_edge)
        .filter(|(e, _)| e.kind == kind && e.good == g && L.nodes[e.to].kind == NodeKind::Works)
        .map(|(e, &key)| (L.nodes[e.to].index, f.keys[key]))
        .filter(|(_, v)| *v > 0.0)
        .collect();
    out.sort_by(|a, b| b.1.total_cmp(&a.1));
    out
}

/// The growth after 1900 as the standing check reads it: the wage's growth from 1900 to
/// 1970 in per cent a year, its level in 2050 over 1970's, and the share of hands on the
/// land in 1900 and in 2050. The linked wage is read as its facts are, each the mean of
/// the nine years round the year (`linked::nine_years`); the chained one, single years.
pub fn growth(isl: &Island, y0: i64) -> [Option<f64>; 4] {
    let at = |v: &Vec<f64>, y: i64| usize::try_from(y - y0).ok().and_then(|k| v.get(k).copied()).filter(|x| x.is_finite());
    let y1 = y0 + isl.wage_chained.len() as i64 - 1;
    let wage_at = |y: i64| -> Option<f64> {
        if !isl.wage_linked {
            return at(&isl.wage_chained, y);
        }
        if y > y1 {
            return None;
        }
        let (a, z) = crate::linked::nine_years(y0, y1, y);
        let v: Vec<f64> = (a..=z).filter_map(|t| at(&isl.wage_chained, t)).collect();
        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
    };
    let (w00, w70, w50) = (wage_at(1900), wage_at(1970), wage_at(2050));
    [
        w00.zip(w70).map(|(a, b)| 100.0 * ((b / a).powf(1.0 / 70.0) - 1.0)),
        w70.zip(w50).map(|(a, b)| b / a),
        at(&isl.land_share, 1900),
        at(&isl.land_share, 2050),
    ]
}

/// The atlas whole: the drawing, the islands and their middle.
#[derive(Clone, Debug)]
pub struct Atlas {
    pub layout: Layout,
    pub islands: Vec<Island>,
    pub middle: Vec<Option<Figures>>,
    pub starts: Vec<i64>,
    pub step: i64,
    pub content: Content,
    pub failed: Vec<(u64, String)>,
    pub seconds: f64,
    pub jobs: usize,
}

fn fmt_ratio(r: f64) -> String {
    if !r.is_finite() {
        "-".to_string()
    } else if r >= 10.0 {
        format!("{r:.0}")
    } else {
        format!("{r:.2}")
    }
}

/// What the page shows, in words: for each island and the middle, the dearest
/// good against 1750 from 1900 on with what holds it as plant, then the growth
/// after 1900, then the recipe usage.
pub fn summary(a: &Atlas) -> String {
    let C = &a.content;
    let mut s = String::new();
    let mut places: Vec<(String, &Vec<Option<Figures>>)> = vec![("the middle of the islands".to_string(), &a.middle)];
    for isl in &a.islands {
        places.push((format!("island {}", isl.seed), &isl.periods));
    }
    let _ = writeln!(s, "The making: the dearest goods against the wage, set against the period each was first made in (1750 for the port's goods)\n");
    for (name, ps) in &places {
        let _ = writeln!(s, "{name}");
        for (p, &y) in a.starts.iter().enumerate() {
            if y < 1850 {
                continue;
            }
            let Some(Some(f)) = ps.get(p) else { continue };
            let top = dearest(ps, p);
            let three: Vec<String> = top.iter().take(3).map(|&(g, r)| format!("{} x{}", C.good_label[g], fmt_ratio(r))).collect();
            let g0 = top.first().map(|x| x.0);
            let held = g0.map_or(String::new(), |g| {
                let h: Vec<String> = holders(&a.layout, f, g, EdgeKind::Plant).iter().take(6).map(|&(m, _)| C.method_label[m].clone()).collect();
                if h.is_empty() { String::new() } else { format!("; held as plant by {}", h.join(", ")) }
            });
            let _ = writeln!(s, "  {y}  wage {:>9.3}  dearest: {}{held}", f.wage, three.join(", "));
        }
        s.push('\n');
    }
    let _ = writeln!(s, "The growth after 1900, read off the page (the observer's chained wage; the share of hands working the land)");
    let _ = writeln!(s, "  island   1900-70 %/yr   2050/1970   land 1900   land 2050");
    let mut cols: [Vec<f64>; 4] = Default::default();
    for isl in &a.islands {
        let g = growth(isl, C.start_year);
        let f = |v: Option<f64>, d: usize| v.map_or("-".to_string(), |x| format!("{x:.d$}"));
        let _ = writeln!(s, "  {:>6}   {:>12}   {:>9}   {:>9}   {:>9}", isl.seed, f(g[0], 2), f(g[1], 3), f(g[2], 3), f(g[3], 3));
        for k in 0..4 {
            if let Some(x) = g[k] {
                cols[k].push(x);
            }
        }
    }
    let mean = |v: &Vec<f64>| if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 };
    let _ = writeln!(s, "  {:>6}   {:>12.2}   {:>9.3}   {:>9.3}   {:>9.3}\n", "mean", mean(&cols[0]), mean(&cols[1]), mean(&cols[2]), mean(&cols[3]));

    let _ = writeln!(s, "Recipe usage: the first and last year each kind of works ran, and its hands at the peak period, island by island");
    for m in 0..C.M {
        let cells: Vec<String> = a
            .islands
            .iter()
            .map(|isl| {
                let u = &isl.usage[m];
                match (u.first, u.last) {
                    (Some(f), Some(l)) => format!("{f}-{l} ({:.0})", u.peak_hands),
                    _ => "never".to_string(),
                }
            })
            .collect();
        let _ = writeln!(s, "  {:<32} {}", C.method_label[m], cells.join("  "));
    }
    let never = never_run(a);
    let _ = writeln!(
        s,
        "\nNever run on any island ({} of {} kinds of works): {}",
        never.len(),
        C.M,
        if never.is_empty() { "none".to_string() } else { never.iter().map(|&m| C.method_label[m].clone()).collect::<Vec<_>>().join(", ") }
    );
    let (lp, ll, lh, dr) = a.islands.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, i| (acc.0 + i.left_out.plant, acc.1 + i.left_out.land, acc.2 + i.left_out.households, acc.3 + i.left_out.drawn));
    let pct = |x: f64| if dr > 0.0 { 100.0 * x / dr } else { 0.0 };
    let _ = writeln!(
        s,
        "Left out of the drawing, as a share of what is drawn: plant with no works to pay for it {:.3}%, the land's purchases with no sector to pay {:.3}%, households' goods that serve no need {:.3}%",
        pct(lp),
        pct(ll),
        pct(lh)
    );
    for (seed, why) in &a.failed {
        let _ = writeln!(s, "FAILED: island {seed}: {why}");
    }
    s
}

/// The kinds of works no island ran.
pub fn never_run(a: &Atlas) -> Vec<usize> {
    (0..a.content.M).filter(|&m| a.islands.iter().all(|i| i.usage[m].first.is_none())).collect()
}

// ---------------------------------------------------------------- running

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()));
    format!("panicked: {}", text.unwrap_or_else(|| "no message".to_string()))
}

/// Play every island, a few at a time on their own threads (the sweep's pool,
/// copied), and read each one's books as it finishes.
pub fn run(o: &Options, P: &Params, content: &Content) -> Atlas {
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    let jobs = o.jobs.unwrap_or((cores / 2).max(1)).min(o.seeds.len()).max(1);
    let t0 = Instant::now();
    let next = AtomicUsize::new(0);
    let finished = AtomicUsize::new(0);
    let slots: Mutex<Vec<Option<(Result<Island, String>, Option<Content>)>>> = Mutex::new(vec![None; o.seeds.len()]);
    std::thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| {
                loop {
                    let id = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&seed) = o.seeds.get(id) else { break };
                    let mut P = P.clone();
                    P.seed = seed;
                    let t = Instant::now();
                    let c = content.clone();
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let r = crate::sim::run_with(&P, c, false, true)?;
                        let isl = read_run(&r, seed, o.step, o.towns, t.elapsed().as_secs_f64())?;
                        Ok::<_, String>((isl, r.content))
                    }));
                    let (outcome, played) = match outcome {
                        Ok(Ok((isl, c))) => (Ok(isl), Some(c)),
                        Ok(Err(e)) => (Err(e), None),
                        Err(payload) => (Err(panic_message(payload)), None),
                    };
                    if !o.quiet {
                        let n = finished.fetch_add(1, Ordering::Relaxed) + 1;
                        let how = match &outcome {
                            Ok(_) => format!("{:.1} s on the clock", t.elapsed().as_secs_f64()),
                            Err(e) => format!("FAILED: {e}"),
                        };
                        eprintln!("[{n}/{}] seed {seed}: {how}", o.seeds.len());
                    }
                    slots.lock().unwrap()[id] = Some((outcome, played));
                }
            });
        }
    });
    let all: Vec<(Result<Island, String>, Option<Content>)> = slots.into_inner().unwrap().into_iter().map(|x| x.expect("every run reports")).collect();
    // the content as the runs played it, with the game's overlay laid over it
    let played = all
        .iter()
        .find_map(|(_, c)| c.clone())
        .unwrap_or_else(|| crate::tree::content(P, content.clone()).unwrap_or_else(|_| content.clone()));
    let mut islands = Vec::new();
    let mut failed = Vec::new();
    for (seed, (r, _)) in o.seeds.iter().zip(all) {
        match r {
            Ok(isl) => islands.push(isl),
            Err(e) => failed.push((*seed, e)),
        }
    }
    let starts = periods(played.start_year, P.end_year.unwrap_or(played.end_year), o.step);
    let layout = layout(&played, &used_keys(&islands));
    let middle = middle(&islands, starts.len());
    Atlas { layout, islands, middle, starts, step: o.step, content: played, failed, seconds: t0.elapsed().as_secs_f64(), jobs }
}

// ---------------------------------------------------------------- the page's figures

/// A number as the page carries it: three significant figures, `null` for none.
fn num(v: f64) -> String {
    if !v.is_finite() {
        return "null".to_string();
    }
    if v == 0.0 {
        return "0".to_string();
    }
    let e = v.abs().log10().floor() as i32;
    let scale = 10f64.powi(2 - e);
    let r = (v * scale).round() / scale;
    let s = format!("{r}");
    // f64's Display never writes an exponent; long runs of zeros are written as one
    if s.len() > 9 { format!("{r:e}") } else { s }
}

fn nums(v: &[f64]) -> String {
    let mut s = String::with_capacity(v.len() * 4);
    s.push('[');
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&num(*x));
    }
    s.push(']');
    s
}

fn js_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

/// One place's periods as the page reads them: an array per figure, one entry per period.
fn per_json(ps: &[Option<Figures>], L: &Layout) -> String {
    let mut s = String::from("{");
    let field = |name: &str, f: &dyn Fn(&Figures) -> String| -> String {
        let items: Vec<String> = ps.iter().map(|p| p.as_ref().map_or("null".to_string(), |x| f(x))).collect();
        format!("{}:[{}]", js_str(name), items.join(","))
    };
    let parts = [
        field("wage", &|f| num(f.wage)),
        field("pop", &|f| num(f.pop)),
        field("price", &|f| nums(&f.price)),
        field("made", &|f| nums(&f.made)),
        field("mout", &|f| nums(&f.m_out)),
        field("mhands", &|f| nums(&f.m_hands)),
        field("mworks", &|f| nums(&f.m_works)),
        field("sout", &|f| nums(&f.s_out)),
        field("shands", &|f| nums(&f.s_hands)),
        field("edge", &|f| nums(&L.key_of_edge.iter().map(|&k| f.keys[k]).collect::<Vec<_>>())),
    ];
    s.push_str(&parts.join(","));
    s.push('}');
    s
}

/// The figures the page carries, as one JSON object (`making.json`).
pub fn page_json(a: &Atlas, o: &Options, code: Option<&str>) -> String {
    let C = &a.content;
    let L = &a.layout;
    let mut s = String::new();
    s.push('{');
    let _ = write!(
        s,
        "\"plate\":\"making\",\"made\":{},\"command\":{},\"code\":{},\"rules\":{},\"families\":{},\"sets\":{},\"y0\":{},\"step\":{},\"periods\":{},",
        js_str(&crate::record::today()),
        js_str(&o.command),
        js_str(code.unwrap_or("unknown")),
        js_str(o.rules.as_str()),
        o.families,
        js_str(&o.sets.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")),
        C.start_year,
        a.step,
        nums(&a.starts.iter().map(|&y| y as f64).collect::<Vec<_>>()),
    );
    let _ = write!(s, "\"width\":{},\"height\":{},", num(L.width), num(L.height));
    let nodes: Vec<String> = L
        .nodes
        .iter()
        .map(|n| format!("{{\"k\":\"{}\",\"i\":{},\"name\":{},\"label\":{},\"x\":{},\"y\":{},\"col\":{}}}", n.kind.as_str(), n.index, js_str(&n.name), js_str(&n.label), num(n.x), num(n.y), n.col))
        .collect();
    let _ = write!(s, "\"nodes\":[{}],", nodes.join(","));
    let edges: Vec<String> = L.edges.iter().map(|e| format!("[{},{},{},{}]", e.from, e.to, e.kind.code(), e.good)).collect();
    let _ = write!(s, "\"edges\":[{}],", edges.join(","));
    let goods: Vec<String> = (0..C.G)
        .map(|g| format!("{{\"name\":{},\"label\":{},\"color\":{},\"node\":{}}}", js_str(&C.good_name[g]), js_str(&C.good_label[g]), js_str(&C.good_color[g]), L.good_node[g]))
        .collect();
    let _ = write!(s, "\"goods\":[{}],", goods.join(","));
    let methods: Vec<String> = (0..C.M)
        .map(|m| {
            let pairs = |row: &dyn Fn(usize) -> f64| -> String {
                let v: Vec<String> = (0..C.G).filter(|&g| row(g) > 0.0).map(|g| format!("[{g},{}]", num(row(g)))).collect();
                format!("[{}]", v.join(","))
            };
            format!(
                "{{\"name\":{},\"label\":{},\"out\":{},\"inputs\":{},\"heat\":{},\"labor\":{},\"plant\":{},\"node\":{}}}",
                js_str(&C.method_name[m]),
                js_str(&C.method_label[m]),
                C.method_out[m],
                pairs(&|g| C.m_in[(m, g)]),
                num(C.m_heat[m]),
                num(C.m_labor.get(m).copied().unwrap_or(f64::NAN)),
                pairs(&|g| C.m_cap[(m, g)]),
                L.works_node[m]
            )
        })
        .collect();
    let _ = write!(s, "\"methods\":[{}],", methods.join(","));
    let sectors: Vec<String> = (0..C.NS).map(|x| format!("{{\"name\":{},\"label\":{},\"good\":{},\"node\":{}}}", js_str(&C.sector_name[x]), js_str(&C.sector_label[x]), C.sector_good[x], L.sector_node[x])).collect();
    let _ = write!(s, "\"sectors\":[{}],", sectors.join(","));
    let needs: Vec<String> = (0..C.S).map(|x| format!("{{\"name\":{},\"label\":{},\"node\":{}}}", js_str(&C.service_name[x]), js_str(&C.service_label[x]), L.need_node[x])).collect();
    let _ = write!(s, "\"needs\":[{}],", needs.join(","));

    // the places: the middle first, then each island with its towns
    let mut places = Vec::new();
    let n = a.islands.len();
    let mean_growth: Vec<String> = (0..4)
        .map(|k| {
            let v: Vec<f64> = a.islands.iter().filter_map(|i| growth(i, C.start_year)[k]).collect();
            num(if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 })
        })
        .collect();
    let years = a.islands.iter().map(|i| i.wage_chained.len()).max().unwrap_or(0);
    let med_series = |f: &dyn Fn(&Island) -> &Vec<f64>| -> Vec<f64> {
        (0..years)
            .map(|y| {
                let mut v: Vec<f64> = a.islands.iter().filter_map(|i| f(i).get(y).copied()).filter(|x| x.is_finite()).collect();
                v.sort_by(f64::total_cmp);
                match v.len() {
                    0 => f64::NAN,
                    k if k % 2 == 1 => v[k / 2],
                    k => 0.5 * (v[k / 2 - 1] + v[k / 2]),
                }
            })
            .collect()
    };
    if n > 1 {
        // a kind of works is found in the middle when half the islands have found it
        let found: Vec<String> = (0..C.M)
            .map(|m| {
                let mut ys: Vec<i64> = a.islands.iter().filter_map(|i| i.found[m]).collect();
                ys.sort();
                if ys.len() * 2 >= n { num(ys[(n + 1) / 2 - 1] as f64) } else { "null".to_string() }
            })
            .collect();
        places.push(format!(
            "{{\"key\":\"middle\",\"label\":{},\"found\":[{}],\"chained\":{},\"land\":{},\"growth\":[{}],\"growth_is\":\"the mean of the islands' own, as the sweep reads them\",\"island\":{},\"towns\":[]}}",
            js_str(&format!("The middle of the {}", if n == 8 { "eight".to_string() } else { n.to_string() })),
            found.join(","),
            nums(&med_series(&|i| &i.wage_chained)),
            nums(&med_series(&|i| &i.land_share)),
            mean_growth.join(","),
            per_json(&a.middle, L)
        ));
    }
    for isl in &a.islands {
        let found: Vec<String> = isl.found.iter().map(|f| f.map_or("null".to_string(), |y| y.to_string())).collect();
        let towns: Vec<String> = isl
            .towns
            .iter()
            .map(|t| format!("{{\"t\":{},\"name\":{},\"from\":{},\"per\":{}}}", t.index, js_str(&t.name), t.founded, per_json(&t.periods, L)))
            .collect();
        let g = growth(isl, C.start_year);
        places.push(format!(
            "{{\"key\":\"{}\",\"label\":\"Island {}\",\"found\":[{}],\"chained\":{},\"land\":{},\"growth\":[{}],\"growth_is\":\"this island's\",\"island\":{},\"towns\":[{}]}}",
            isl.seed,
            isl.seed,
            found.join(","),
            nums(&isl.wage_chained),
            nums(&isl.land_share),
            g.iter().map(|x| num(x.unwrap_or(f64::NAN))).collect::<Vec<_>>().join(","),
            per_json(&isl.periods, L),
            towns.join(",")
        ));
    }
    let _ = write!(s, "\"places\":[{}],", places.join(","));
    // recipe usage: per method, per island [first, last, peak hands, peak period]
    let usage: Vec<String> = (0..C.M)
        .map(|m| {
            let cells: Vec<String> = a
                .islands
                .iter()
                .map(|i| {
                    let u = &i.usage[m];
                    let y = |v: Option<i64>| v.map_or("null".to_string(), |y| y.to_string());
                    format!("[{},{},{},{}]", y(u.first), y(u.last), num(u.peak_hands), y(u.peak_period))
                })
                .collect();
            format!("[{}]", cells.join(","))
        })
        .collect();
    let _ = write!(s, "\"usage\":[{}],\"usage_seeds\":{},", usage.join(","), nums(&a.islands.iter().map(|i| i.seed as f64).collect::<Vec<_>>()));
    let lo = a.islands.iter().fold(LeftOut::default(), |acc, i| LeftOut {
        plant: acc.plant + i.left_out.plant,
        land: acc.land + i.left_out.land,
        households: acc.households + i.left_out.households,
        drawn: acc.drawn + i.left_out.drawn,
    });
    let _ = write!(s, "\"left_out\":{{\"plant\":{},\"land\":{},\"households\":{},\"drawn\":{}}},", num(lo.plant), num(lo.land), num(lo.households), num(lo.drawn));
    let failed: Vec<String> = a.failed.iter().map(|(seed, why)| format!("[{seed},{}]", js_str(why))).collect();
    let _ = write!(s, "\"failed\":[{}]", failed.join(","));
    s.push('}');
    s
}

/// The page: the drawing's code and its figures in one file, nothing fetched.
pub fn page(json: &str) -> String {
    // a "</" inside the figures would end the script early
    let safe = json.replace("</", "<\\/");
    PAGE.replace("/*ATLAS_DATA*/null", &safe)
}

const PAGE: &str = include_str!("atlas_page.html");

// ---------------------------------------------------------------- the command

fn git_describe() -> Option<String> {
    let out = std::process::Command::new("git").args(["describe", "--always", "--dirty"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !text.is_empty()).then_some(text)
}

fn readme(a: &Atlas, o: &Options, code: Option<&str>) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "The atlas, first plate: the making (isostate-core atlas making), {} (UTC).\n", crate::record::today());
    let _ = writeln!(s, "command:  {}", o.command);
    let _ = writeln!(s, "rules:    {}, {} families, to {}{}", o.rules.as_str(), o.families, o.end_year, if o.sets.is_empty() { String::new() } else { format!(", with {}", o.sets.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")) });
    let _ = writeln!(s, "code:     {}", code.unwrap_or("unknown (git describe gave nothing)"));
    let _ = writeln!(s, "seeds:    {}", o.seeds.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "));
    let _ = writeln!(s, "runs:     {} ({} failed), {} at a time, {:.0} s in all", o.seeds.len(), a.failed.len(), a.jobs, a.seconds);
    s.push_str(
        "\nFiles\n\
         making.html  the page: open it in any browser; it fetches nothing\n\
         making.json  the figures the page carries; atlas making --from DIR draws the page again from it\n\
         making.txt   what the page shows, in words\n\
         \nHow the page is made and how to read it: docs/atlas.md.\n",
    );
    s
}

pub fn main_with_args(args: &[String]) -> Result<(), String> {
    command(args, true)
}

/// The command as the page's header and README.txt give it: each folder or
/// file it names (`--out`, `--from`, `--against`, `--edits`) by its own name
/// only, so that no path of the machine it was made on is carried in the page.
pub fn command_line(args: &[String]) -> String {
    let mut out = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        out.push(a.clone());
        if ["--out", "--from", "--against", "--edits"].contains(&a.as_str()) {
            if let Some(v) = it.next() {
                let p = std::path::Path::new(v);
                out.push(p.file_name().map_or(v.clone(), |n| n.to_string_lossy().into_owned()));
            }
        }
    }
    format!("isostate-core atlas {}", out.join(" "))
}

/// The subcommand, with or without its summary on stdout (the tests leave it off).
pub fn command(args: &[String], print: bool) -> Result<(), String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}\n{}", crate::atlas_ideas::USAGE);
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("ideas") {
        return crate::atlas_ideas::command(args, print);
    }
    let o = parse_args(args)?;
    if let Some(from) = &o.from {
        let src = from.join("making.json");
        let json = std::fs::read_to_string(&src).map_err(|e| format!("--from: {}: {e}", src.display()))?;
        std::fs::create_dir_all(&o.out).map_err(|e| format!("--out {}: {e}", o.out.display()))?;
        let dst = o.out.join("making.html");
        std::fs::write(&dst, page(&json)).map_err(|e| format!("{}: {e}", dst.display()))?;
        if !o.quiet {
            eprintln!("drew {} again from {} (making.txt is not written again)", dst.display(), src.display());
        }
        return Ok(());
    }
    let (P, content) = params(&o)?;
    std::fs::create_dir_all(&o.out).map_err(|e| format!("--out {}: {e}", o.out.display()))?;
    let a = run(&o, &P, &content);
    if a.islands.is_empty() {
        let why: Vec<String> = a.failed.iter().map(|(s, w)| format!("island {s}: {w}")).collect();
        return Err(format!("every run failed: {}", why.join("; ")));
    }
    let code = git_describe();
    let json = page_json(&a, &o, code.as_deref());
    let text = summary(&a);
    let put = |name: &str, body: &str| std::fs::write(o.out.join(name), body).map_err(|e| format!("{}: {e}", o.out.join(name).display()));
    put("making.json", &json)?;
    put("making.html", &page(&json))?;
    put("making.txt", &text)?;
    put("README.txt", &readme(&a, &o, code.as_deref()))?;
    if print {
        let _ = std::io::Write::write_all(&mut std::io::stdout(), text.as_bytes());
    }
    if !o.quiet {
        eprintln!("{:.0} s on the clock, {} at a time; the page is {} ({:.1} MB)", a.seconds, a.jobs, o.out.join("making.html").display(), json.len() as f64 / 1e6);
    }
    if !a.failed.is_empty() {
        return Err(format!("{} of {} runs failed; the rest are drawn in {}", a.failed.len(), o.seeds.len(), o.out.display()));
    }
    Ok(())
}
