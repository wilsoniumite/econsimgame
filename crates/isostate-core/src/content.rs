//! Load content (goods, services, sectors, methods, techs) into flat arrays.
//!
//! Everything downstream works with integer indices and plain arrays, so this is
//! the only place that knows about names. Python: `content.py`.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use serde::Deserialize;

use crate::mat::Mat;

/// The content file shipped with the repository, embedded at compile time.
pub const DEFAULT_CONTENT_TOML: &str = include_str!("../../../data/content.toml");

/// The flat tables the simulation reads. Field names follow the Python content dict `C`.
#[derive(Clone, Debug, Default)]
pub struct Content {
    pub start_year: i64,
    pub end_year: i64,
    pub G: usize,
    pub good_name: Vec<String>,
    pub good_label: Vec<String>,
    pub good_color: Vec<String>,
    pub good_index: HashMap<String, usize>,
    pub weight: Vec<f64>,
    pub base_price: Vec<f64>,
    pub S: usize,
    pub service_name: Vec<String>,
    pub service_label: Vec<String>,
    pub subsistence: Vec<f64>,
    pub shares: Vec<f64>,
    pub lord_shares: Vec<f64>,
    /// (G, S): how much of each service one unit of each good provides.
    pub props: Mat,
    pub heat: Vec<f64>,
    pub fuel_goods: Vec<usize>,
    pub NS: usize,
    pub sector_name: Vec<String>,
    pub sector_label: Vec<String>,
    pub sector_good: Vec<usize>,
    pub sector_alpha: Vec<f64>,
    pub sector_A: Vec<f64>,
    pub sector_field: Vec<String>,
    pub sector_mine: Vec<bool>,
    pub M: usize,
    pub method_name: Vec<String>,
    pub method_label: Vec<String>,
    pub method_out: Vec<usize>,
    /// (M, G): goods per unit of output.
    pub m_in: Mat,
    pub m_heat: Vec<f64>,
    pub m_labor: Vec<f64>,
    /// (M, G): goods per unit of monthly capacity.
    pub m_cap: Mat,
    pub m_K0: Vec<f64>,
    pub m_A0: Vec<f64>,
    pub m_Q0: Vec<f64>,
    pub m_b: Vec<f64>,
    pub NT: usize,
    pub tech_name: Vec<String>,
    pub tech_label: Vec<String>,
    /// "method" or "tech"
    pub tech_kind: Vec<String>,
    pub tech_known: Vec<bool>,
    /// (NT, G): cumulative production thresholds counted from the start year, 0 = not needed;
    /// negative values mean "N years of start-year output" and are resolved after warm-up.
    pub tech_disc: Mat,
    pub tech_headline: Vec<String>,
    pub tech_story: Vec<String>,
}

impl Content {
    /// Index of a good by name; panics on a name the content does not have.
    pub fn gi(&self, name: &str) -> usize {
        match self.good_index.get(name) {
            Some(&g) => g,
            None => panic!("unknown good {name:?}"),
        }
    }
}

#[derive(Deserialize)]
struct RawContent {
    meta: RawMeta,
    services: RawServices,
    heat: BTreeMap<String, f64>,
    sector: Vec<RawSector>,
    method: Vec<RawMethod>,
    #[serde(default)]
    tech: Vec<RawMethod>,
}

#[derive(Deserialize)]
struct RawMeta {
    start_year: i64,
    end_year: i64,
    goods: Vec<RawGood>,
}

#[derive(Deserialize)]
struct RawGood {
    name: String,
    label: String,
    weight: f64,
    price: f64,
    color: String,
}

#[derive(Deserialize)]
struct RawServices {
    names: Vec<String>,
    labels: Vec<String>,
    subsistence: Vec<f64>,
    shares: Vec<f64>,
    lord_shares: Option<Vec<f64>>,
    props: BTreeMap<String, BTreeMap<String, f64>>,
}

#[derive(Deserialize)]
struct RawSector {
    name: String,
    label: String,
    good: String,
    alpha: f64,
    #[serde(rename = "A")]
    a: f64,
    field: String,
    #[serde(default)]
    mine: bool,
}

/// A threshold is either a number of units or "xN", N years of start-year output.
#[derive(Deserialize)]
#[serde(untagged)]
enum Threshold {
    Num(f64),
    Text(String),
}

/// Methods and extra techs share this shape; techs leave the firm fields out.
#[derive(Deserialize)]
struct RawMethod {
    name: String,
    label: String,
    output: Option<String>,
    #[serde(default)]
    inputs: BTreeMap<String, f64>,
    heat: Option<f64>,
    labor: Option<f64>,
    #[serde(default)]
    capital: BTreeMap<String, f64>,
    #[serde(rename = "K0")]
    k0: Option<f64>,
    #[serde(default)]
    known: bool,
    #[serde(rename = "A0")]
    a0: Option<f64>,
    #[serde(rename = "Q0")]
    q0: Option<f64>,
    b: Option<f64>,
    #[serde(default)]
    discover: BTreeMap<String, Threshold>,
    #[serde(default)]
    headline: String,
    #[serde(default)]
    story: String,
}

/// Parse content from TOML text.
pub fn content_from_str(text: &str) -> Result<Content, String> {
    let raw: RawContent = toml::from_str(text).map_err(|e| format!("content: {e}"))?;
    let goods = &raw.meta.goods;
    let gname: Vec<String> = goods.iter().map(|g| g.name.clone()).collect();
    let gi: HashMap<String, usize> = gname.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
    let G = goods.len();
    let good = |name: &str| -> Result<usize, String> {
        gi.get(name).copied().ok_or_else(|| format!("content: unknown good {name:?}"))
    };

    let svc = &raw.services;
    let sname = svc.names.clone();
    let S = sname.len();
    let mut props = Mat::zeros(G, S);
    for (g, d) in &svc.props {
        for (s, v) in d {
            let si = sname.iter().position(|x| x == s).ok_or_else(|| format!("content: unknown service {s:?}"))?;
            props[(good(g)?, si)] = *v;
        }
    }

    let mut heat = vec![0.0; G];
    for (g, v) in &raw.heat {
        heat[good(g)?] = *v;
    }
    let fuel_goods: Vec<usize> = (0..G).filter(|&g| heat[g] > 0.0).collect();

    let sectors = &raw.sector;
    let methods = &raw.method;
    let M = methods.len();
    let mut m_in = Mat::zeros(M, G);
    let mut m_cap = Mat::zeros(M, G);
    let mut method_out = Vec::with_capacity(M);
    for (m, d) in methods.iter().enumerate() {
        for (g, v) in &d.inputs {
            m_in[(m, good(g)?)] = *v;
        }
        for (g, v) in &d.capital {
            m_cap[(m, good(g)?)] = *v;
        }
        let out = d.output.as_deref().ok_or_else(|| format!("content: method {:?} has no output", d.name))?;
        method_out.push(good(out)?);
    }
    let need = |v: Option<f64>, what: &str, name: &str| v.ok_or_else(|| format!("content: method {name:?} has no {what}"));

    // Knowledge items: every firm method, then the extra techs.
    let tech_list: Vec<(&RawMethod, &str)> =
        methods.iter().map(|d| (d, "method")).chain(raw.tech.iter().map(|d| (d, "tech"))).collect();
    let NT = tech_list.len();
    let mut t_disc = Mat::zeros(NT, G);
    for (t, (d, _)) in tech_list.iter().enumerate() {
        for (g, v) in &d.discover {
            let x = match v {
                Threshold::Num(x) => *x,
                Threshold::Text(s) if s.starts_with('x') => {
                    -s[1..].trim().parse::<f64>().map_err(|e| format!("content: threshold {s:?}: {e}"))?
                }
                Threshold::Text(s) => s.trim().parse::<f64>().map_err(|e| format!("content: threshold {s:?}: {e}"))?,
            };
            t_disc[(t, good(g)?)] = x;
        }
    }

    let mut c = Content {
        start_year: raw.meta.start_year,
        end_year: raw.meta.end_year,
        G,
        good_name: gname.clone(),
        good_label: goods.iter().map(|g| g.label.clone()).collect(),
        good_color: goods.iter().map(|g| g.color.clone()).collect(),
        good_index: gi.clone(),
        weight: goods.iter().map(|g| g.weight).collect(),
        base_price: goods.iter().map(|g| g.price).collect(),
        S,
        service_name: sname,
        service_label: svc.labels.clone(),
        subsistence: svc.subsistence.clone(),
        shares: svc.shares.clone(),
        lord_shares: svc.lord_shares.clone().unwrap_or_else(|| svc.shares.clone()),
        props,
        heat,
        fuel_goods,
        NS: sectors.len(),
        sector_name: sectors.iter().map(|s| s.name.clone()).collect(),
        sector_label: sectors.iter().map(|s| s.label.clone()).collect(),
        sector_good: Vec::new(),
        sector_alpha: sectors.iter().map(|s| s.alpha).collect(),
        sector_A: sectors.iter().map(|s| s.a).collect(),
        sector_field: sectors.iter().map(|s| s.field.clone()).collect(),
        sector_mine: sectors.iter().map(|s| s.mine).collect(),
        M,
        method_name: methods.iter().map(|d| d.name.clone()).collect(),
        method_label: methods.iter().map(|d| d.label.clone()).collect(),
        method_out,
        m_in,
        m_heat: methods.iter().map(|d| d.heat.unwrap_or(0.0)).collect(),
        m_labor: Vec::with_capacity(M),
        m_cap,
        m_K0: Vec::with_capacity(M),
        m_A0: methods.iter().map(|d| d.a0.unwrap_or(1.0)).collect(),
        m_Q0: methods.iter().map(|d| d.q0.unwrap_or(1e12)).collect(),
        m_b: methods.iter().map(|d| d.b.unwrap_or(0.0)).collect(),
        NT,
        tech_name: tech_list.iter().map(|(d, _)| d.name.clone()).collect(),
        tech_label: tech_list.iter().map(|(d, _)| d.label.clone()).collect(),
        tech_kind: tech_list.iter().map(|(_, k)| k.to_string()).collect(),
        tech_known: tech_list.iter().map(|(d, _)| d.known).collect(),
        tech_disc: t_disc,
        tech_headline: tech_list.iter().map(|(d, _)| d.headline.clone()).collect(),
        tech_story: tech_list.iter().map(|(d, _)| d.story.clone()).collect(),
    };
    for s in sectors {
        c.sector_good.push(good(&s.good)?);
    }
    for d in methods {
        c.m_labor.push(need(d.labor, "labor", &d.name)?);
        c.m_K0.push(need(d.k0, "K0", &d.name)?);
    }
    Ok(c)
}

/// Load content from a TOML file. Python: `content.load_content(path)`.
pub fn load_content(path: &Path) -> Result<Content, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    content_from_str(&text)
}

/// The content shipped with the repository (`data/content.toml`, embedded at build time).
pub fn default_content() -> Content {
    content_from_str(DEFAULT_CONTENT_TOML).expect("embedded data/content.toml is valid")
}

pub fn tech_index(c: &Content, name: &str) -> usize {
    c.tech_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("unknown tech {name:?}"))
}

pub fn method_index(c: &Content, name: &str) -> usize {
    c.method_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("unknown method {name:?}"))
}
