//! TREE: a deeper production tree, laid over the port's content by name.
//! Not in the Python; see `docs/rules/tree.md`.
//!
//! The port's content is `data/content.toml` and stays the Python's file. The
//! game's tree is a second file, `data/game.toml`, laid over it: a good, a need,
//! a sector, a method or an invention whose name the content already has changes
//! only the keys the overlay writes, and a new name is added after the old ones,
//! so the port's sixteen goods and sixteen methods keep their places. Every
//! entry of the overlay carries a stage, and `P.tree.stage` is the ceiling: 0
//! lays nothing, and a run is the port's (or the game's as it was) to the last
//! digit; 1 is the building trades, sawn timber, lime, beer, pottery, town gas,
//! soap, glass and paper with their works.
//!
//! What the engine had to learn for a seventeenth good is small and is here or
//! marked "tree" where it stands: decay and opening stock from the content
//! (`decay`, `stock0`), a width of the logit for each need (`widths`), goods that
//! are not carried between towns (`Content::traded`), the 1750 trades of the new
//! goods (`seed_plan`), a family trade for each new method (`group` in the file),
//! and the observer's shares taken from the content (`share_groups`).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::content::{Content, RawContent, RawGood, RawMeta, RawMethod, RawSector, RawServices, Threshold, content_from_raw};
use crate::mat::Mat;
use crate::params::Params;

/// The game's overlay, embedded at compile time.
pub const GAME_OVERLAY_TOML: &str = include_str!("../../../data/game.toml");

/// The highest stage the overlay has that runs soundly (docs/rules/tree.md).
/// Stage 6 is the first content after 1900, electricity and electrical goods
/// ("After 1900: electricity"); the tree's own stages 2 to 5 have no entries
/// in the file yet, so 6 lays stage 1 and the content after 1900.
pub const GAME_STAGE: u32 = 6;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TreeParams {
    /// How much of the game's tree is laid over the content: every entry of the
    /// overlay with a stage no greater than this. 0 is none of it.
    pub stage: u32,
    /// An overlay file to read in place of the one built in (`data/game.toml`):
    /// for trying numbers without building again. Empty is the one built in.
    pub file: String,
    /// Open 1750 with the new goods' trades in place (the methods the overlay
    /// marks `seed = true`), as the port opens with its smiths and weavers.
    /// Without it they are founded during the warm-up decade, a works a year.
    pub seed_trades: bool,
    /// Plant waits: where a market is short, households and the works at work
    /// are served first and the orders for new plant and for the materials of
    /// railways and canals share what is left (`econ::step_month`). Under the
    /// port everyone is cut alike, which is harmless while nothing that plant
    /// is built of is also a works' daily input; with the tree bricks, sawn
    /// timber, lime and glass are both, and each spring's orders for plant
    /// stopped the builders. It acts only with a stage laid: at stage 0 a run
    /// is the run without the tree.
    pub plant_waits: bool,
    /// With a stage laid, the stone the canals' first survey waits for
    /// (`canals.discover_stone`) is this share of its number. That number was
    /// set on an island that heaps raw stone into walls; with builders the
    /// same houses take a third of the stone (0.34 of it dug by 1760, 0.35 by
    /// 1770), so a clock that counts stone dug runs at a third of the speed
    /// and the first canal came nine years late. 1 leaves the clock alone.
    pub canal_stone: f64,
}

impl Default for TreeParams {
    /// Off: the port's content as it is.
    fn default() -> TreeParams {
        TreeParams { stage: 0, file: String::new(), seed_trades: true, plant_waits: false, canal_stone: 1.0 }
    }
}

impl TreeParams {
    /// The game's tree: the highest stage that works.
    pub fn game() -> TreeParams {
        TreeParams { stage: GAME_STAGE, plant_waits: true, canal_stone: 0.35, ..TreeParams::default() }
    }
}

// ---------------------------------------------------------------- the overlay file

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Overlay {
    meta: OverMeta,
    services: OverServices,
    heat: BTreeMap<String, f64>,
    sector: Vec<OverSector>,
    method: Vec<OverMethod>,
    tech: Vec<OverMethod>,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct OverMeta {
    goods: Vec<OverGood>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverGood {
    name: String,
    label: Option<String>,
    weight: Option<f64>,
    price: Option<f64>,
    color: Option<String>,
    decay: Option<f64>,
    stock0: Option<f64>,
    traded: Option<bool>,
    stock_months: Option<f64>,
    opens_dear: Option<bool>,
    stage: Option<u32>,
}

/// The needs. Each list written replaces the content's; a row of `props` adds
/// to or changes the needs it names. `change` holds the same again for later
/// stages, applied in the order written.
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct OverServices {
    names: Option<Vec<String>>,
    labels: Option<Vec<String>>,
    subsistence: Option<Vec<f64>>,
    shares: Option<Vec<f64>>,
    shares_rich: Option<Vec<f64>>,
    lord_shares: Option<Vec<f64>>,
    theta: Option<Vec<f64>>,
    stage: Option<u32>,
    change: Vec<OverServices>,
    props: BTreeMap<String, BTreeMap<String, f64>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverSector {
    name: String,
    label: Option<String>,
    good: Option<String>,
    alpha: Option<f64>,
    #[serde(rename = "A")]
    a: Option<f64>,
    field: Option<String>,
    mine: Option<bool>,
    stage: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverMethod {
    name: String,
    label: Option<String>,
    output: Option<String>,
    inputs: Option<BTreeMap<String, f64>>,
    heat: Option<f64>,
    labor: Option<f64>,
    capital: Option<BTreeMap<String, f64>>,
    #[serde(rename = "K0")]
    k0: Option<f64>,
    known: Option<bool>,
    #[serde(rename = "A0")]
    a0: Option<f64>,
    #[serde(rename = "Q0")]
    q0: Option<f64>,
    b: Option<f64>,
    discover: Option<BTreeMap<String, Threshold>>,
    headline: Option<String>,
    story: Option<String>,
    group: Option<String>,
    seed: Option<bool>,
    target: Option<i64>,
    /// The first year it may be found (`Content::tech_after`).
    found_after: Option<i64>,
    /// Where the works must stand. Read from stage 2; kept so that a stage 1
    /// method may already say it.
    #[allow(dead_code)]
    site: Option<toml::Value>,
    stage: Option<u32>,
}

/// An entry without a stage belongs to the first.
fn stage_of(s: Option<u32>) -> u32 {
    s.unwrap_or(1)
}

// ---------------------------------------------------------------- laying it

/// The content written out again as a file would say it, by name: what the
/// overlay is laid over. `content_from_raw` of it gives the content back.
fn raw_of(C: &Content) -> RawContent {
    let line = |row: &[f64]| -> BTreeMap<String, f64> {
        (0..C.G).filter(|&g| row[g] != 0.0).map(|g| (C.good_name[g].clone(), row[g])).collect()
    };
    let item = |t: usize| -> RawMethod {
        let m = (t < C.M).then_some(t);
        RawMethod {
            name: C.tech_name[t].clone(),
            label: C.tech_label[t].clone(),
            output: m.map(|m| C.good_name[C.method_out[m]].clone()),
            inputs: m.map(|m| line(C.m_in.row(m))).unwrap_or_default(),
            heat: m.map(|m| C.m_heat[m]),
            labor: m.map(|m| C.m_labor[m]),
            capital: m.map(|m| line(C.m_cap.row(m))).unwrap_or_default(),
            k0: m.map(|m| C.m_K0[m]),
            known: C.tech_known[t],
            a0: m.map(|m| C.m_A0[m]),
            q0: m.map(|m| C.m_Q0[m]),
            b: m.map(|m| C.m_b[m]),
            // a negative threshold is "N years of 1750's output", and is read back as it is
            discover: line(C.tech_disc.row(t)).into_iter().map(|(g, v)| (g, Threshold::Num(v))).collect(),
            headline: C.tech_headline[t].clone(),
            story: C.tech_story[t].clone(),
            group: m.and_then(|m| C.method_group.get(m)).filter(|g| !g.is_empty()).cloned(),
            seed: m.and_then(|m| C.method_seed.get(m)).copied().unwrap_or(false),
            target: C.tech_target.get(t).copied().filter(|&y| y != 0),
            found_after: C.tech_after.get(t).copied().filter(|&y| y != 0),
        }
    };
    RawContent {
        meta: RawMeta {
            start_year: C.start_year,
            end_year: C.end_year,
            goods: (0..C.G)
                .map(|g| RawGood {
                    name: C.good_name[g].clone(),
                    label: C.good_label[g].clone(),
                    weight: C.weight[g],
                    price: C.base_price[g],
                    color: C.good_color[g].clone(),
                    decay: C.decay.get(g).copied().flatten(),
                    stock0: C.stock0.get(g).copied().flatten(),
                    traded: C.traded.get(g).copied().filter(|&t| !t),
                    stock_months: C.stock_months.get(g).copied().flatten(),
                    opens_dear: C.opens_dear.get(g).copied().filter(|&d| d),
                })
                .collect(),
        },
        services: RawServices {
            names: C.service_name.clone(),
            labels: C.service_label.clone(),
            subsistence: C.subsistence.clone(),
            shares: C.shares.clone(),
            lord_shares: Some(C.lord_shares.clone()),
            shares_rich: Some(C.shares_rich.clone()),
            theta: (!C.theta.is_empty()).then(|| C.theta.clone()),
            props: (0..C.G)
                .filter_map(|g| {
                    let row: BTreeMap<String, f64> =
                        (0..C.S).filter(|&s| C.props[(g, s)] != 0.0).map(|s| (C.service_name[s].clone(), C.props[(g, s)])).collect();
                    (!row.is_empty()).then(|| (C.good_name[g].clone(), row))
                })
                .collect(),
        },
        heat: (0..C.G).filter(|&g| C.heat[g] != 0.0).map(|g| (C.good_name[g].clone(), C.heat[g])).collect(),
        sector: (0..C.NS)
            .map(|s| RawSector {
                name: C.sector_name[s].clone(),
                label: C.sector_label[s].clone(),
                good: C.good_name[C.sector_good[s]].clone(),
                alpha: C.sector_alpha[s],
                a: C.sector_A[s],
                field: C.sector_field[s].clone(),
                mine: C.sector_mine[s],
            })
            .collect(),
        method: (0..C.M).map(item).collect(),
        tech: (C.M..C.NT).map(item).collect(),
    }
}

/// `over` laid over `base` up to `stage`. Stage 0 lays nothing.
pub fn lay(base: &Content, over: &str, stage: u32) -> Result<Content, String> {
    if stage == 0 {
        return Ok(base.clone());
    }
    let over: Overlay = toml::from_str(over).map_err(|e| format!("tree: {e}"))?;
    let mut raw = raw_of(base);
    let missing = |what: &str, name: &str, key: &str| format!("tree: the new {what} {name:?} has no {key}");

    // goods: a new one goes after the old ones; one of a later stage does not
    // exist yet, and whatever names it below is dropped without complaint
    let later: BTreeSet<&str> = over.meta.goods.iter().filter(|g| stage_of(g.stage) > stage).map(|g| g.name.as_str()).collect();
    for g in over.meta.goods.iter().filter(|g| stage_of(g.stage) <= stage) {
        match raw.meta.goods.iter_mut().find(|r| r.name == g.name) {
            Some(r) => {
                r.label = g.label.clone().unwrap_or(std::mem::take(&mut r.label));
                r.color = g.color.clone().unwrap_or(std::mem::take(&mut r.color));
                r.weight = g.weight.unwrap_or(r.weight);
                r.price = g.price.unwrap_or(r.price);
                r.decay = g.decay.or(r.decay);
                r.stock0 = g.stock0.or(r.stock0);
                r.traded = g.traded.or(r.traded);
                r.stock_months = g.stock_months.or(r.stock_months);
                r.opens_dear = g.opens_dear.or(r.opens_dear);
            }
            None => raw.meta.goods.push(RawGood {
                name: g.name.clone(),
                label: g.label.clone().ok_or_else(|| missing("good", &g.name, "label"))?,
                weight: g.weight.ok_or_else(|| missing("good", &g.name, "weight"))?,
                price: g.price.ok_or_else(|| missing("good", &g.name, "price"))?,
                color: g.color.clone().ok_or_else(|| missing("good", &g.name, "color"))?,
                decay: g.decay,
                stock0: g.stock0,
                traded: g.traded,
                stock_months: g.stock_months,
                opens_dear: g.opens_dear,
            }),
        }
    }
    let goods: BTreeSet<String> = raw.meta.goods.iter().map(|g| g.name.clone()).collect();
    // a line of goods (inputs, plant, thresholds) without those of a later stage
    let known = |what: &str, g: &str| -> Result<bool, String> {
        if goods.contains(g) {
            Ok(true)
        } else if later.contains(g) {
            Ok(false)
        } else {
            Err(format!("tree: {what} names a good {g:?} that neither the content nor the overlay has"))
        }
    };

    lay_needs(&mut raw.services, &over.services, stage, &known)?;
    let S = raw.services.names.len();
    let s = &raw.services;
    for (what, n) in [
        ("labels", s.labels.len()),
        ("subsistence", s.subsistence.len()),
        ("shares", s.shares.len()),
        ("shares_rich", s.shares_rich.as_ref().map_or(S, |v| v.len())),
        ("lord_shares", s.lord_shares.as_ref().map_or(S, |v| v.len())),
        ("theta", s.theta.as_ref().map_or(S, |v| v.len())),
    ] {
        if n != S {
            return Err(format!("tree: there are {S} needs and {n} values in the needs' {what}"));
        }
    }
    if s.theta.as_ref().is_some_and(|v| v.iter().any(|&t| !(t > 0.0))) {
        return Err("tree: every need's theta must be above nothing".to_string());
    }

    for (g, v) in &over.heat {
        if known("[heat]", g)? {
            raw.heat.insert(g.clone(), *v);
        }
    }

    for o in over.sector.iter().filter(|o| stage_of(o.stage) <= stage) {
        match raw.sector.iter_mut().find(|r| r.name == o.name) {
            Some(r) => {
                r.label = o.label.clone().unwrap_or(std::mem::take(&mut r.label));
                r.good = o.good.clone().unwrap_or(std::mem::take(&mut r.good));
                r.field = o.field.clone().unwrap_or(std::mem::take(&mut r.field));
                r.alpha = o.alpha.unwrap_or(r.alpha);
                r.a = o.a.unwrap_or(r.a);
                r.mine = o.mine.unwrap_or(r.mine);
            }
            None => raw.sector.push(RawSector {
                name: o.name.clone(),
                label: o.label.clone().ok_or_else(|| missing("sector", &o.name, "label"))?,
                good: o.good.clone().ok_or_else(|| missing("sector", &o.name, "good"))?,
                alpha: o.alpha.ok_or_else(|| missing("sector", &o.name, "alpha"))?,
                a: o.a.ok_or_else(|| missing("sector", &o.name, "A"))?,
                field: o.field.clone().ok_or_else(|| missing("sector", &o.name, "field"))?,
                mine: o.mine.unwrap_or(false),
            }),
        }
    }

    // methods and inventions: a table written (inputs, plant, thresholds)
    // replaces the content's whole; any other key changes that key alone
    let line = |what: &str, t: &BTreeMap<String, f64>| -> Result<BTreeMap<String, f64>, String> {
        let mut out = BTreeMap::new();
        for (g, v) in t {
            if known(what, g)? {
                out.insert(g.clone(), *v);
            }
        }
        Ok(out)
    };
    for (list, kind, over_list) in [(&mut raw.method, "method", &over.method), (&mut raw.tech, "invention", &over.tech)] {
        for o in over_list.iter().filter(|o| stage_of(o.stage) <= stage) {
            let what = format!("the {kind} {:?}", o.name);
            let r = match list.iter().position(|r| r.name == o.name) {
                Some(k) => &mut list[k],
                None => {
                    let label = o.label.clone().ok_or_else(|| missing(kind, &o.name, "label"))?;
                    list.push(RawMethod { name: o.name.clone(), label, ..RawMethod::default() });
                    list.last_mut().unwrap()
                }
            };
            if let Some(v) = &o.label {
                r.label = v.clone();
            }
            if let Some(v) = &o.output {
                r.output = Some(v.clone());
            }
            if let Some(t) = &o.inputs {
                r.inputs = line(&what, t)?;
            }
            if let Some(t) = &o.capital {
                r.capital = line(&what, t)?;
            }
            if let Some(t) = &o.discover {
                r.discover = BTreeMap::new();
                for (g, v) in t {
                    if known(&what, g)? {
                        r.discover.insert(g.clone(), v.clone());
                    }
                }
            }
            r.heat = o.heat.or(r.heat);
            r.labor = o.labor.or(r.labor);
            r.k0 = o.k0.or(r.k0);
            r.a0 = o.a0.or(r.a0);
            r.q0 = o.q0.or(r.q0);
            r.b = o.b.or(r.b);
            r.known = o.known.unwrap_or(r.known);
            r.seed = o.seed.unwrap_or(r.seed);
            r.group = o.group.clone().or(r.group.take());
            r.target = o.target.or(r.target);
            r.found_after = o.found_after.or(r.found_after);
            if let Some(v) = &o.headline {
                r.headline = v.clone();
            }
            if let Some(v) = &o.story {
                r.story = v.clone();
            }
            if let Some(g) = r.group.as_deref() {
                if crate::families::Group::from_name(g).is_none() {
                    return Err(format!("tree: {what} is of the family trade {g:?}, which is not one of the six"));
                }
            }
        }
    }

    let mut C = content_from_raw(&raw).map_err(|e| format!("tree: {e}"))?;
    C.stage = stage;
    // tasks (docs/rules/tasks.md): the game's table, laid again on the new
    // methods by name, where the content laid over had it
    if !base.tasks.is_empty() {
        crate::tasks::fill(&mut C);
    }
    Ok(C)
}

/// The needs of the overlay laid over the content's: each list written
/// replaces the content's (the names may only grow), a row of props adds to or
/// changes the needs it names, and then the same for each later change.
fn lay_needs(s: &mut RawServices, o: &OverServices, stage: u32, known: &dyn Fn(&str, &str) -> Result<bool, String>) -> Result<(), String> {
    if stage_of(o.stage) <= stage {
        if let Some(names) = &o.names {
            if names.len() < s.names.len() || names[..s.names.len()] != s.names[..] {
                return Err("tree: the overlay's needs must begin with the content's own, in their order".to_string());
            }
            s.names = names.clone();
        }
        if let Some(v) = &o.labels {
            s.labels = v.clone();
        }
        if let Some(v) = &o.subsistence {
            s.subsistence = v.clone();
        }
        if let Some(v) = &o.shares {
            s.shares = v.clone();
        }
        if let Some(v) = &o.shares_rich {
            s.shares_rich = Some(v.clone());
        }
        if let Some(v) = &o.lord_shares {
            s.lord_shares = Some(v.clone());
        }
        if let Some(v) = &o.theta {
            s.theta = Some(v.clone());
        }
        for (g, row) in &o.props {
            if !known("a row of [services.props]", g)? {
                continue;
            }
            let have = s.props.entry(g.clone()).or_default();
            for (need, v) in row {
                have.insert(need.clone(), *v);
            }
        }
    }
    for c in &o.change {
        lay_needs(s, c, stage, known)?;
    }
    Ok(())
}

/// The content a run under `P` is played on: the one given, with the game's
/// overlay laid over it up to `P.tree.stage`. Content that already carries that
/// stage (or a later one) is returned as it is.
pub fn content(P: &Params, C: Content) -> Result<Content, String> {
    if P.tree.stage <= C.stage {
        return Ok(C);
    }
    if P.tree.file.is_empty() {
        lay(&C, GAME_OVERLAY_TOML, P.tree.stage)
    } else {
        let text = std::fs::read_to_string(&P.tree.file).map_err(|e| format!("tree: {}: {e}", P.tree.file))?;
        lay(&C, &text, P.tree.stage)
    }
}

/// The game's content at a stage of the tree: `game_content()` with the overlay
/// built in laid over it.
pub fn game_content_at(stage: u32) -> Content {
    lay(&crate::content::game_content(), GAME_OVERLAY_TOML, stage).expect("embedded data/game.toml is valid")
}

// ---------------------------------------------------------------- what the engine reads

/// Monthly loss of stock of each good: the content's where it says, the
/// port's table for the port's goods, and nothing for a good neither names.
pub fn decay(C: &Content, P: &Params) -> Vec<f64> {
    (0..C.G).map(|g| C.decay.get(g).copied().flatten().unwrap_or_else(|| P.decay.get(g).copied().unwrap_or(0.0))).collect()
}

/// What is lost of each good's stock left at a month's end, once the month's
/// sales are taken out of it: its decay, and all of it for a good that does
/// not keep (`stock_months = 0`, electricity), whose month's make is what the
/// next month sells and no more. `econ::step_month` and the books
/// (`accounts::note_month`) both read this, so the books count the loss as
/// spoiled. With no such good it is `decay`, to the bit.
pub fn month_loss(C: &Content, P: &Params) -> Vec<f64> {
    let mut d = decay(C, P);
    for (g, v) in d.iter_mut().enumerate() {
        if !keeps(C, g) {
            *v = 1.0;
        }
    }
    d
}

/// False for a good that does not keep from one month to the next
/// (`stock_months = 0` in the content: electricity).
pub fn keeps(C: &Content, g: usize) -> bool {
    C.stock_months.get(g).copied().flatten() != Some(0.0)
}

/// The gap the month's price of good `g` in a town moves on (`econ::step_month`,
/// read as the port's rule reads it: above nothing while the market wants more).
/// For a good that keeps, how far the merchants' stock `stock` is below the
/// months of sales they mean to hold (`stock_months`, or the island's
/// `price_target_months`) times the smoothed sales `outflow`, as a share of the
/// larger. For a good that does not keep, the month's asking against its
/// making, `(asked - made) / max(asked, made)`, and nothing where neither.
pub fn price_gap(C: &Content, P: &Params, g: usize, stock: f64, outflow: f64, asked: f64, made: f64) -> f64 {
    match C.stock_months.get(g).copied().flatten() {
        Some(m) if m == 0.0 => {
            let big = asked.max(made);
            if big > 1e-12 { (asked - made) / big } else { 0.0 }
        }
        Some(m) => {
            let target = m * outflow;
            (target - stock) / target.max(stock).max(1e-9)
        }
        None => {
            let target = P.price_target_months * outflow;
            (target - stock) / target.max(stock).max(1e-9)
        }
    }
}

/// The price a good opens at in every town: its base price, or its ceiling for
/// a good the overlay marks `opens_dear`. Such a good is had nowhere in 1750
/// and cannot be made for a century, and at its base price it would be bought
/// as if it could: electricity is then the cheapest fuel on the island, and
/// every works would put most of its heat on a fuel it could not get (a works
/// makes what its scarcest input allows); electrical goods the cheapest of the
/// household goods. The price rule would carry either to its ceiling anyway,
/// where nobody makes it (the stock's gap, or the asking's, is 1 while it is
/// asked for and not had), but only after some forty months of hunger for it.
pub fn opening_price(st: &crate::state::State, C: &Content, P: &Params, g: usize) -> f64 {
    if C.opens_dear.get(g).copied().unwrap_or(false) { crate::longrun::ceiling(st, P, C, g) } else { C.base_price[g] }
}

/// Opening stock a head of good `g`: the content's where it says, else the
/// port's table (`port`, in the order of the port's goods).
pub fn stock0(C: &Content, g: usize, port: &[f64]) -> f64 {
    C.stock0.get(g).copied().flatten().unwrap_or_else(|| port.get(g).copied().unwrap_or(0.0))
}

/// The logit's width within each need: the content's, or the port's one
/// number for all of them.
pub fn widths(C: &Content, P: &Params) -> Vec<f64> {
    if C.theta.len() == C.S { C.theta.clone() } else { vec![P.logit_theta; C.S] }
}

/// The goods carried between towns, in their order: all of them in the port.
pub fn traded_goods(C: &Content) -> Vec<usize> {
    (0..C.G).filter(|&g| C.traded.get(g).copied().unwrap_or(true)).collect()
}

/// The goods whose delivered price at a cell anybody reads: those some method
/// takes as an input, and the fuels. Under the port all are marked, and the
/// yearly pass is the port's.
pub fn delivered_goods(C: &Content) -> Vec<bool> {
    (0..C.G).map(|g| C.stage == 0 || C.heat[g] > 0.0 || (0..C.M).any(|m| C.m_in[(m, g)] > 0.0)).collect()
}

/// Plant waits, in one market: `have` is the stock, `first` what households and
/// the works at work asked for, `all` that and the orders for new plant and for
/// the materials of railways and canals. The day's trade is served first and
/// the orders share what is left. Returns the share of what it asked for that
/// each is given: the day's trade, then the orders. Nobody is given more than
/// the market has.
pub fn served_first(have: f64, first: f64, all: f64) -> (f64, f64) {
    let fill = if first > 1e-12 { (have / first).min(1.0) } else { 1.0 };
    let left = (have - fill * first).max(0.0);
    let asked = all - first;
    let fill_plant = if asked > 1e-12 { (left / asked).min(1.0) } else { 1.0 };
    (fill, fill_plant)
}

/// Plant waits in every town's market (`econ::step_month`): `inv` is the stock,
/// `first` and `all` the requests as in `served_first`. `fill` becomes the
/// day's trade's and the orders' is returned.
pub fn plant_waits(inv: &Mat, first: &Mat, all: &Mat, fill: &mut Mat) -> Mat {
    let mut fill_plant = Mat::zeros(fill.rows, fill.cols);
    for k in 0..fill.data.len() {
        (fill.data[k], fill_plant.data[k]) = served_first(inv.data[k], first.data[k], all.data[k]);
    }
    fill_plant
}

/// What the canals' clock asks of `good` before a first survey
/// (`canals::knowledge`), as a share of the canals' own number: all of it,
/// but for stone with a stage laid (`TreeParams::canal_stone`).
pub fn canal_clock(C: &Content, P: &Params, good: &str) -> f64 {
    if C.stage > 0 && good == "stone" { P.tree.canal_stone } else { 1.0 }
}

/// The 1750 trades of the goods the port does not have. `need` is what each
/// town's households want of every good a month (and, for the port's goods,
/// what its own plan adds); this adds what the seeded works will themselves
/// buy of the new goods, and returns the goods to found works for, in the
/// order they are to be founded (a good before the goods it is made of).
pub fn seed_plan(C: &Content, P: &Params, need: &mut Mat) -> Vec<usize> {
    if C.stage == 0 || !P.tree.seed_trades {
        return Vec::new();
    }
    let mut plan: Vec<usize> = Vec::new();
    for m in (0..C.M).filter(|&m| C.method_seed[m] && C.tech_known[m]) {
        if !plan.contains(&C.method_out[m]) {
            plan.push(C.method_out[m]);
        }
    }
    // a good that feeds another on the list comes after it: its need then
    // counts what the first one's works will buy
    let feeds = |a: usize, b: usize| (0..C.M).any(|m| C.method_seed[m] && C.method_out[m] == b && C.m_in[(m, a)] > 0.0);
    let mut order: Vec<usize> = Vec::new();
    while !plan.is_empty() {
        let k = plan.iter().position(|&g| !plan.iter().any(|&h| h != g && feeds(g, h))).unwrap_or(0);
        order.push(plan.remove(k));
    }
    for &g in &order {
        // the first seeded method of g stands for its trade
        for m in (0..C.M).filter(|&m| C.method_seed[m] && C.tech_known[m] && C.method_out[m] == g) {
            for t in 0..need.rows {
                for h in 0..C.G {
                    if order.contains(&h) && h != g {
                        need[(t, h)] += C.m_in[(m, h)] * need[(t, g)];
                    }
                }
            }
            break;
        }
    }
    order
}

/// A group of shares the observer keeps: its name, its kind ("method",
/// "service" or "heat") and its members.
pub type ShareGroup = (String, String, Vec<String>);

/// The observer's groups with the tree laid: the port's own, each with any
/// further good that now serves its need or method that now makes its good,
/// then a group for every need and every good that has more than one member
/// and no group yet. Under the port (`C.stage == 0`) the list is the port's.
pub fn share_groups(C: &Content, port: &[(&str, &str, &[&str])]) -> Vec<ShareGroup> {
    let mut groups: Vec<ShareGroup> =
        port.iter().map(|&(n, k, mem)| (n.to_string(), k.to_string(), mem.iter().map(|m| m.to_string()).collect())).collect();
    if C.stage == 0 {
        return groups;
    }
    for (s, need) in C.service_name.iter().enumerate() {
        let serve: Vec<String> = (0..C.G).filter(|&g| C.props[(g, s)] > 0.0).map(|g| C.good_name[g].clone()).collect();
        merge_group(&mut groups, need, "service", serve);
    }
    for (g, good) in C.good_name.iter().enumerate() {
        let makers: Vec<String> = (0..C.M).filter(|&m| C.method_out[m] == g).map(|m| C.method_name[m].clone()).collect();
        merge_group(&mut groups, good, "method", makers);
    }
    // the works' heat, with any fuel the overlay adds (electricity, from stage 6;
    // stage 1 adds none, and the group is the port's)
    let fuels: Vec<String> = C.fuel_goods.iter().map(|&g| C.good_name[g].clone()).collect();
    merge_group(&mut groups, "heat", "heat", fuels);
    groups
}

/// Facts of the content after 1900 (docs/rules/tree.md, "After 1900:
/// electricity"), for the sweep's table under "Work and wages": nothing for a
/// content without electricity. In 1890 and at the marks from 1900:
///
/// - `electricity_warmth_share_Y`: of the fuel and light households and
///   landlords bought, in units of warmth, the share electricity gave (the
///   observer's `share_warmth_electricity`);
/// - `electricity_heat_share_Y`: of the works' heat, the share electricity gave;
/// - `stations_towns_Y`: towns with a coal power station standing at the year's end;
/// - `electricity_lost_Y`: of the electricity made in the year, the share lost
///   at the months' ends (the books' `spoiled` over `made`);
/// - `electricity_over_coal_heat_Y`: electricity's price over coal's for a unit
///   of heat, each the year's mean price in the towns that bought electricity,
///   weighted by what they bought of it;
/// - `electricals_to_households_Y` and `electricals_to_plant_Y`: of the
///   electrical goods bought in the year, the shares households (with landlords
///   and families) and plant (works being built, the land's machines, railways
///   and canals) took.
///
/// And `found_year_coal_power_station`, `found_year_electrical_works`.
pub fn facts(r: &crate::sim::RunResult) -> Vec<(String, crate::facts::Kind, f64)> {
    use crate::facts::Kind;
    use crate::accounts::{GOODS_FIELDS, KG};
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    let (Some(&ge), Some(&gx), Some(&gc)) = (C.good_index.get("electricity"), C.good_index.get("electricals"), C.good_index.get("coal")) else {
        return out;
    };
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let mut years: Vec<i64> = crate::facts::marks(y0, y1).into_iter().filter(|&y| y >= 1900).collect();
    if (y0..=y1).contains(&1890) {
        years.insert(0, 1890);
    }
    let at = |name: &str, y: i64| st.series.index.get(name).map_or(f64::NAN, |&k| crate::observer::annual(&st.series.data[k], y0, y));
    let ratio = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
    let station = C.method_name.iter().position(|n| n == "coal_power_station");
    let col = |name: &str| GOODS_FIELDS.iter().position(|&n| n == name).expect("a goods column");
    let (c_made, c_spoiled, c_price) = (col("made"), col("spoiled"), col("price"));
    let c_hh = [col("households"), col("landlords"), col("families")];
    let c_plant = [col("building"), col("land"), col("projects")];
    let c_use = [col("works_inputs"), col("works_fuel")];
    let (T, G) = (st.T_max, C.G);
    let heat = |g: usize| C.heat[g];
    for &y in &years {
        out.push((format!("electricity_warmth_share_{y}"), Kind::Share, at("share_warmth_electricity", y)));
        out.push((format!("electricity_heat_share_{y}"), Kind::Share, at("share_heat_electricity", y)));
        let towns: std::collections::BTreeSet<usize> = (0..st.F_count)
            .filter(|&f| Some(st.f_method[f]) == station && st.f_born[f] <= y && (st.f_alive[f] || st.f_died[f] > y))
            .map(|f| st.f_town[f])
            .collect();
        out.push((format!("stations_towns_{y}"), Kind::Count, if station.is_some() { towns.len() as f64 } else { f64::NAN }));
        // the books of the year, where they were kept
        let book = st.acct.years.iter().position(|b| b.year == y).and_then(|k| st.acct.year(k));
        let (mut made, mut lost, mut bought_e, mut pe, mut pc) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let (mut x_hh, mut x_plant, mut x_all) = (0.0, 0.0, 0.0);
        if let Some(b) = &book {
            let v = |t: usize, g: usize, c: usize| b.goods[(t * G + g) * KG + c] as f64;
            for t in 0..T {
                made += v(t, ge, c_made);
                lost += v(t, ge, c_spoiled);
                let e: f64 = c_hh.iter().chain(&c_plant).chain(&c_use).map(|&c| v(t, ge, c)).sum();
                bought_e += e;
                pe += e * v(t, ge, c_price);
                pc += e * v(t, gc, c_price);
                let hh: f64 = c_hh.iter().map(|&c| v(t, gx, c)).sum();
                let plant: f64 = c_plant.iter().map(|&c| v(t, gx, c)).sum();
                x_hh += hh;
                x_plant += plant;
                x_all += hh + plant + c_use.iter().map(|&c| v(t, gx, c)).sum::<f64>();
            }
        }
        let (n, kn) = (book.is_some(), f64::NAN);
        out.push((format!("electricity_lost_{y}"), Kind::Share, if n { ratio(lost, made) } else { kn }));
        let per_heat = ratio(ratio(pe, bought_e) / heat(ge), ratio(pc, bought_e) / heat(gc));
        out.push((format!("electricity_over_coal_heat_{y}"), Kind::Ratio, if n { per_heat } else { kn }));
        out.push((format!("electricals_to_households_{y}"), Kind::Share, if n { ratio(x_hh, x_all) } else { kn }));
        out.push((format!("electricals_to_plant_{y}"), Kind::Share, if n { ratio(x_plant, x_all) } else { kn }));
    }
    for name in ["coal_power_station", "electrical_works"] {
        let found = C.tech_name.iter().position(|n| n == name).map(|t| st.tech_year[t]).filter(|&y| y >= y0);
        out.push((format!("found_year_{name}"), Kind::Year, found.map_or(f64::NAN, |y| y as f64)));
    }
    out
}

fn merge_group(groups: &mut Vec<ShareGroup>, name: &str, kind: &str, members: Vec<String>) {
    match groups.iter_mut().find(|(n, k, _)| n == name && k == kind) {
        Some((_, _, have)) => {
            for m in members {
                if !have.contains(&m) {
                    have.push(m);
                }
            }
        }
        None if members.len() > 1 => groups.push((name.to_string(), kind.to_string(), members)),
        None => {}
    }
}

/// News of a new way of making a good overtaking the old, for the methods the
/// port has no words for: (group, member, title, text), as the observer's own.
pub fn crossings(C: &Content, groups: &[ShareGroup], port: &[(&str, &str, &str, &str)]) -> Vec<(String, String, String, String)> {
    let mut out: Vec<(String, String, String, String)> =
        port.iter().map(|&(g, m, t, x)| (g.to_string(), m.to_string(), t.to_string(), x.to_string())).collect();
    if C.stage == 0 {
        return out;
    }
    for (name, _, members) in groups.iter().filter(|(_, k, _)| k == "method") {
        for mem in members {
            let Some(m) = C.method_name.iter().position(|n| n == mem) else { continue };
            if C.tech_known[m] || out.iter().any(|(g, x, _, _)| g == name && x == mem) {
                continue;
            }
            let good = C.good_label[C.method_out[m]].to_lowercase();
            out.push((
                name.clone(),
                mem.clone(),
                format!("{}: now most of the {good}", C.method_label[m]),
                format!("{{share}} of the island's {good} is now made the new way. The older works are paid less each year for the same work."),
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{default_content, game_content};

    #[test]
    fn written_out_and_read_back_the_content_is_itself() {
        for mut C in [default_content(), game_content()] {
            let mut back = content_from_raw(&raw_of(&C)).unwrap();
            // the classes of task come from a file of their own, laid again by name
            if !C.tasks.is_empty() {
                crate::tasks::fill(&mut back);
            }
            // the one table with no order of its own
            C.good_index.clear();
            back.good_index.clear();
            assert_eq!(format!("{C:?}"), format!("{back:?}"));
        }
    }

    #[test]
    fn the_days_trade_is_served_before_the_orders_for_plant() {
        // plenty: everyone has what was asked
        assert_eq!(served_first(100.0, 30.0, 50.0), (1.0, 1.0));
        // enough for the day's trade and half of the orders
        assert_eq!(served_first(40.0, 30.0, 50.0), (1.0, 0.5));
        // short of the day's trade: it shares what there is and the orders wait
        assert_eq!(served_first(15.0, 30.0, 50.0), (0.5, 0.0));
        // an empty market, and one nobody asks anything of
        assert_eq!(served_first(0.0, 30.0, 50.0), (0.0, 0.0));
        assert_eq!(served_first(0.0, 0.0, 20.0), (1.0, 0.0));
        assert_eq!(served_first(7.0, 0.0, 0.0), (1.0, 1.0));
        // only orders: they share the stock as the port would share it
        assert_eq!(served_first(5.0, 0.0, 20.0), (1.0, 0.25));
        // whatever is asked, no more leaves the market than it has
        for have in [0.0, 1e-9, 0.3, 7.0, 1e6] {
            for first in [0.0, 1e-13, 0.2, 9.0] {
                for plant in [0.0, 1e-13, 0.5, 40.0] {
                    let (a, b) = served_first(have, first, first + plant);
                    assert!((0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b));
                    assert!(a * first + b * plant <= have + 1e-9, "{have} {first} {plant}");
                    // and the orders are given nothing (a rounding aside) while the day's trade goes short
                    assert!(a == 1.0 || b < 1e-9 || plant <= 1e-12, "{have} {first} {plant}");
                }
            }
        }
    }

    #[test]
    fn stage_nothing_lays_nothing() {
        let mut C = game_content();
        let mut same = lay(&C, GAME_OVERLAY_TOML, 0).unwrap();
        C.good_index.clear();
        same.good_index.clear();
        assert_eq!(format!("{C:?}"), format!("{same:?}"));
    }
}
