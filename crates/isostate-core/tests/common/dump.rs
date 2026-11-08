//! Rebuild a Rust `State` from a Python state dump (`parity/dump_state.py`).
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use isostate_core::mat::Mat;
use isostate_core::state::{MonthStats, Project, RailOpened, State};
use isostate_core::transport::{Graph, neighbour};
use isostate_core::world::World;
use serde_json::Value;

use super::read_npy;

pub struct Dump {
    pub dir: PathBuf,
    pub scalars: Value,
    pub args: Value,
}

impl Dump {
    pub fn open(dir: &Path) -> Dump {
        let read_json = |name: &str| -> Value {
            let p = dir.join(name);
            if p.exists() { serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap() } else { Value::Null }
        };
        Dump { dir: dir.to_path_buf(), scalars: read_json("scalars.json"), args: read_json("args.json") }
    }

    pub fn has(&self, name: &str) -> bool {
        self.dir.join(format!("{name}.npy")).exists()
    }

    pub fn f64(&self, name: &str) -> Vec<f64> {
        read_npy(&self.dir.join(format!("{name}.npy"))).f64()
    }

    pub fn usize(&self, name: &str) -> Vec<usize> {
        read_npy(&self.dir.join(format!("{name}.npy"))).usize()
    }

    pub fn i64(&self, name: &str) -> Vec<i64> {
        read_npy(&self.dir.join(format!("{name}.npy"))).i64()
    }

    pub fn u8(&self, name: &str) -> Vec<u8> {
        self.i64(name).into_iter().map(|x| x as u8).collect()
    }

    pub fn bool(&self, name: &str) -> Vec<bool> {
        read_npy(&self.dir.join(format!("{name}.npy"))).bool()
    }

    pub fn mat(&self, name: &str) -> Mat {
        let a = read_npy(&self.dir.join(format!("{name}.npy")));
        let (rows, cols) = match a.shape.len() {
            2 => (a.shape[0], a.shape[1]),
            1 => (1, a.shape[0]),
            _ => panic!("{name}: shape {:?} is not 2D", a.shape),
        };
        Mat::from_vec(rows, cols, a.f64())
    }

    pub fn scalar(&self, key: &str) -> &Value {
        &self.scalars[key]
    }

    pub fn sf64(&self, key: &str) -> f64 {
        num(&self.scalars[key])
    }

    pub fn arg_f64(&self, key: &str) -> f64 {
        num(&self.args[key])
    }

    pub fn arg_mat(&self, name: &str) -> Mat {
        self.mat(&format!("arg_{name}"))
    }

    pub fn arg_vec(&self, name: &str) -> Vec<f64> {
        self.f64(&format!("arg_{name}"))
    }
}

pub fn num(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap(),
        Value::String(s) if s == "inf" => f64::INFINITY,
        Value::String(s) if s == "-inf" => f64::NEG_INFINITY,
        Value::String(s) if s == "nan" => f64::NAN,
        Value::Bool(b) => *b as u8 as f64,
        other => panic!("not a number: {other}"),
    }
}

/// The Python edge list (direction-major `a`, `b`, `w`) in the Rust per-cell layout.
#[allow(non_snake_case)]
pub fn graph_from_edges(H: usize, W: usize, a: &[usize], b: &[usize], w: &[f64], railed: &[bool]) -> Graph {
    let N = H * W;
    let (mut gw, mut gr) = (vec![f64::INFINITY; N * 8], vec![false; N * 8]);
    for e in 0..a.len() {
        let k = (0..8).find(|&k| neighbour(H, W, a[e], k) == Some(b[e])).expect("edge between neighbours");
        gw[a[e] * 8 + k] = w[e];
        gr[a[e] * 8 + k] = railed[e];
    }
    Graph::new(N, W, H, gw, gr)
}

/// Everything the dump holds, in the Rust state's layout.
#[allow(non_snake_case)]
pub fn load_state(dir: &Path) -> State {
    let d = Dump::open(dir);
    let mut st = State::default();
    let W = d.sf64("world_W") as usize;
    let H = d.sf64("world_H") as usize;
    st.world = World {
        W,
        H,
        land: d.bool("world_land"),
        h: d.f64("world_h"),
        terrain: d.u8("world_terrain"),
        down: d.i64("world_down"),
        acc: d.f64("world_acc"),
        river: d.bool("world_river"),
        navigable: d.bool("world_navigable"),
        coast: d.bool("world_coast"),
        fertility: d.f64("world_fertility"),
        pasture: d.f64("world_pasture"),
        forest_pot: d.f64("world_forest_pot"),
        rock: d.f64("world_rock"),
        clay: d.f64("world_clay"),
        ore: d.f64("world_ore"),
        coal: d.f64("world_coal"),
        jitter: d.f64("world_jitter"),
        town_sites: d.scalar("world_town_sites").as_array().unwrap().iter().map(|v| num(v) as usize).collect(),
        forest0: d.f64("world_forest0"),
    };
    st.N = d.sf64("N") as usize;
    st.T_max = d.sf64("T_max") as usize;

    st.town_active = d.bool("town_active");
    st.town_node = d.usize("town_node");
    st.town_founded = d.i64("town_founded");
    st.town_new = d.bool("town_new");
    st.town_name = strings(d.scalar("town_name"));
    st.used_names = strings(d.scalar("used_names")).into_iter().collect();
    st.pop = d.f64("pop");

    st.land_use = d.u8("use");
    st.cover = d.f64("cover");
    st.road = d.u8("road");
    st.rail = d.u8("rail");
    st.road_quiet = d.i64("road_quiet");
    st.firm_at = d.i64("firm_at");
    st.cell_K_share = d.f64("cell_K_share");
    st.land_rent_only = d.f64("land_rent_only");
    st.derelict_timer = d.i64("derelict_timer");
    st.cell_rent = d.f64("cell_rent");
    st.cell_flow = d.f64("cell_flow");
    st.traffic = d.f64("traffic");
    st.urban = d.bool("urban");
    st.urban_res = d.f64("urban_res");
    st.water = d.bool("water");

    st.F_count = d.sf64("F_count") as usize;
    st.f_alive = d.bool("f_alive");
    st.f_method = d.usize("f_method");
    st.f_cell = d.usize("f_cell");
    st.f_town = d.usize("f_town");
    st.f_born = d.i64("f_born");
    st.f_died = d.i64("f_died");
    st.f_loss_years = d.i64("f_loss_years");
    st.f_parent = d.i64("f_parent");
    st.f_K = d.f64("f_K");
    st.f_order_K = d.f64("f_order_K");
    st.f_cash = d.f64("f_cash");
    st.f_rent = d.f64("f_rent");
    st.f_u = d.f64("f_u");
    st.f_fill = d.f64("f_fill");
    st.f_prod = d.f64("f_prod");
    st.f_profit_acc = d.f64("f_profit_acc");
    st.f_out_acc = d.f64("f_out_acc");
    st.f_cost_m = d.f64("f_cost_m");
    st.f_labor = d.f64("f_labor");
    st.f_order = d.mat("f_order");
    st.f_name = strings(d.scalar("f_name"));
    st.new_firms_year = d.scalar("new_firms_year").as_array().unwrap().iter().map(|v| num(v) as usize).collect();
    st.closed_year = d.scalar("closed_year").as_array().unwrap().iter().map(|v| num(v) as usize).collect();

    st.hh_cash = d.f64("hh_cash");
    st.lord_cash = d.f64("lord_cash");
    st.merch_cash = d.f64("merch_cash");
    st.hh_income = d.f64("hh_income");
    st.lord_income = d.f64("lord_income");
    st.hh_income_prev = d.f64("hh_income_prev");
    st.lord_income_prev = d.f64("lord_income_prev");
    st.fund_cash = d.sf64("fund_cash");
    st.issued = d.sf64("issued");
    st.price = d.mat("price");
    st.pexp = d.mat("pexp");
    st.lam = d.mat("lam");
    st.inv = d.mat("inv");
    st.outflow = d.mat("outflow");
    st.fill = d.mat("fill");
    st.wage = d.f64("wage");
    st.tools_mult = d.f64("tools_mult");
    st.imp_pc = d.f64("imp_pc");
    st.urban_rent = d.f64("urban_rent");
    st.urban_pop = d.f64("urban_pop");
    st.ag_labor = d.f64("ag_labor");
    st.ind_labor = d.f64("ind_labor");
    st.y_ratio = d.f64("y_ratio");

    st.know = d.mat("know");
    st.tech_year = d.i64("tech_year");
    st.tech_town = d.i64("tech_town");
    st.t_pump = d.sf64("t_pump") as usize;
    st.t_wagon = d.sf64("t_wagon") as usize;
    st.t_rail = d.sf64("t_rail") as usize;
    st.t_husb = d.sf64("t_husb") as usize;
    st.t_fmach = d.sf64("t_fmach") as usize;
    st.forestry_sector = d.sf64("forestry_sector") as usize;
    st.method_Q = d.f64("method_Q");
    st.method_A = d.f64("method_A");

    st.y_acc = d.f64("y_acc");
    st.price_acc = d.mat("price_acc");
    st.wage_acc = d.f64("wage_acc");
    st.town_prod = d.mat("town_prod");
    st.town_prod_year = d.mat("town_prod_year");
    st.cumprod = d.f64("cumprod");
    st.trade_F = d.mat("trade_F");
    st.trade_F_year = d.mat("trade_F_year");
    st.food_min_acc = d.f64("food_min_acc");
    st.projects = load_projects(&d);
    for pair in d.scalar("rail_links").as_array().unwrap() {
        let p = pair.as_array().unwrap();
        st.rail_links.insert((num(&p[0]) as usize, num(&p[1]) as usize));
    }
    for e in d.scalar("rail_cooldown").as_array().unwrap() {
        let p = e.as_array().unwrap();
        st.rail_cooldown.insert((num(&p[0]) as usize, num(&p[1]) as usize), num(&p[2]) as i64);
    }
    st.spur_done = d.scalar("spur_done").as_array().unwrap().iter().map(|v| num(v) as usize).collect();
    st.cell_flow_year = d.f64("cell_flow_year");
    for e in d.scalar("rail_opened").as_array().unwrap() {
        let p = e.as_array().unwrap();
        st.rail_opened.push(RailOpened { year: num(&p[0]), a: num(&p[1]) as usize, b: num(&p[2]) as i64, km: num(&p[3]) });
    }
    st.m_rent = d.sf64("m_rent");
    st.m_invest = d.sf64("m_invest");
    if !d.scalar("m_merch_margin").is_null() {
        st.m_merch_margin = d.sf64("m_merch_margin");
        st.m_trade_value = d.sf64("m_trade_value");
    }

    st.ex_cell = d.usize("ex_cell");
    st.ex_sector = d.usize("ex_sector");
    st.ex_town = d.usize("ex_town");
    st.ex_lam = d.f64("ex_lam");
    st.ex_field = d.f64("ex_field");
    st.ex_tau = d.f64("ex_tau");
    st.ex_Lprev = d.f64("ex_Lprev");
    st.ex_labor = d.f64("ex_labor");
    if d.has("ex_out") {
        st.ex_out = d.f64("ex_out");
    }

    st.graph = graph_from_edges(H, W, &d.usize("graph_a"), &d.usize("graph_b"), &d.f64("graph_w"), &d.bool("graph_railed"));
    st.dist = d.mat("dist");
    let pred = d.mat("pred");
    st.pred = (0..pred.rows).map(|r| pred.row(r).iter().map(|&x| x as i64).collect()).collect();
    st.catch = d.i64("catch");
    st.dcatch = d.f64("dcatch");
    st.town_cost = d.mat("town_cost");
    st.delta = d.mat("delta");

    st.town_cells = (0..st.T_max)
        .map(|i| (0..st.N).filter(|&c| st.catch[c] == i as i64).collect())
        .collect();
    st.any_counts = d.f64("any_counts");
    st.cell_firms = d.f64("cell_firms");
    st.same_counts = d.mat("same_counts");

    st.bids = d.mat("bids");
    st.cell_urban_rent = d.f64("cell_urban_rent");
    st.urban_density = d.sf64("urban_density");

    if d.has("ms_prod") {
        st.month_stats = MonthStats {
            prod: d.f64("ms_prod"),
            method_out: d.f64("ms_method_out"),
            hh_got: d.mat("ms_hh_got"),
            lord_got: d.mat("ms_lord_got"),
            share: Mat::default(),
            hh_spend: d.sf64("ms_hh_spend"),
            lord_spend: d.sf64("ms_lord_spend"),
            urban_rent: d.sf64("ms_urban_rent"),
            employment: d.f64("ms_employment"),
            ex_labor_by_sector: d.f64("ms_ex_labor_by_sector"),
            firm_labor_by_method: d.f64("ms_firm_labor_by_method"),
            heat_use: d.f64("ms_heat_use"),
            supply: d.f64("ms_supply"),
        };
    }
    st.subcost = d.f64("subcost");
    st.food_ratio = d.f64("food_ratio");

    if d.has("pbar") {
        st.pbar = d.mat("pbar");
        st.wbar = d.f64("wbar");
    }
    if d.has("prev_catch") {
        st.prev_catch = d.i64("prev_catch");
    }
    if d.has("cell_labor") {
        st.cell_labor = d.f64("cell_labor");
    }
    if d.has("cumprod_1750") {
        st.cumprod_1750 = d.f64("cumprod_1750");
        st.disc_X0 = d.mat("disc_X0");
        st.cpi_basket = d.f64("cpi_basket");
        st.cpi_base_prices = d.f64("cpi_base_prices");
        st.wage_base = d.sf64("wage_base");
        st.money_start = d.sf64("money_start");
        st.money_now = d.sf64("money_now");
    }
    st
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_string()).collect()
}

fn load_projects(d: &Dump) -> Vec<Project> {
    let mut out = Vec::new();
    for (k, m) in d.scalar("projects").as_array().unwrap().iter().enumerate() {
        let spur = m.get("kind").and_then(|v| v.as_str()) == Some("spur");
        out.push(Project {
            a: num(&m["a"]) as usize,
            b: num(&m["b"]) as usize,
            spur,
            target: if spur { num(&m["target"]) as usize } else { 0 },
            town: num(&m["town"]) as usize,
            cells: d.usize(&format!("project{k}_cells")),
            new: d.usize(&format!("project{k}_new")),
            need: d.f64(&format!("project{k}_need")),
            order: d.f64(&format!("project{k}_order")),
            cash: num(&m["cash"]),
            start: num(&m["start"]) as i64,
            roi: num(&m["roi"]),
            cost: num(&m["cost"]),
            length_km: num(&m["length_km"]),
        });
    }
    out
}
