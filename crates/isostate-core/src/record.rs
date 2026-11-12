//! Record a run as yearly snapshots and pack it for the viewer. Python: `record.py`.
//!
//! The export is one binary container: a JSON header (names, events, firm
//! histories, array layout) followed by raw typed arrays. Map layers are stored
//! layer-major so consecutive years of the same layer sit next to each other and
//! compress well. The Python package also embeds runs into the HTML viewer
//! (`build_html`); that stays in Python (`python3 -m isostate html DIR`), which
//! reads the run.bin written here.

use serde_json::{Value, json};

use crate::content::Content;
use crate::land::{USE_INDUSTRY, USE_NAMES, USE_URBAN, USE_WILD};
use crate::observer::SHARE_GROUPS;
use crate::params::Params;
use crate::state::{Event, State};
use crate::world::World;

pub const LAYERS: [&str; 9] = ["use", "inten", "rent", "trans", "catch", "dist", "mix_a", "mix_b", "mix_s"];

/// towns: three more layers, kept only by a run with `towns.enabled` (a port
/// run's file has none of them). One byte per cell and year, as the nine above:
///
/// - `built`: the year the cell joined its town less `BUILT_YEAR0`, held to 1 to
///   255; 0 where the cell is not town ground;
/// - `res`: the cell's residents, in units of `RES_UNIT` people (255 at the most);
/// - `works`: the share of the cell under works, 0 to 255, on every cell, town
///   ground included.
pub const TOWN_LAYERS: [&str; 3] = ["built", "res", "works"];
/// `built` counts years from here.
pub const BUILT_YEAR0: i64 = 1645;
/// People to one step of `res`.
pub const RES_UNIT: f64 = 8.0;

/// Display classes for the map. Each cell is drawn as a patchwork of its two
/// largest classes, so mixed land shows as mixed land instead of one colour.
pub const DISP: [&str; 12] =
    ["sea", "farm", "pasture", "wood", "heath", "quarry", "claypit", "ironmine", "coalmine", "industry", "urban", "derelict"];

fn sector_disp(name: &str) -> usize {
    match name {
        "farm" => 1,
        "pasture" => 2,
        "forestry" => 3,
        "quarry" => 5,
        "claypit" => 6,
        "ironmine" => 7,
        "coalmine" => 8,
        _ => panic!("no display class for sector {name:?}"),
    }
}

fn extractive(name: &str) -> bool {
    matches!(name, "quarry" | "claypit" | "ironmine" | "coalmine")
}

/// Top two display classes per cell and the share of the first.
///
/// Quarries and mines take land in the model, but most of it is rough ground
/// around the workings, so only part of their share is drawn as diggings.
/// Wild land shows as wood or heath depending on its tree cover.
pub fn display_mix(st: &State, C: &Content) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let (N, NS) = (st.N, C.NS);
    let nd = DISP.len();
    let mut a_out = vec![0u8; N];
    let mut b_out = vec![0u8; N];
    let mut s_out = vec![0u8; N];
    let mut D = vec![0.0f64; nd];
    // mines: under the mechanism a working's share is all pit head and spoil
    let (dig, rough) = crate::mines::display_split(st);
    for c in 0..N {
        D.iter_mut().for_each(|v| *v = 0.0);
        let ks = st.cell_K_share[c].clamp(0.0, 1.0);
        let mut tot = 0.0;
        for r in 0..=NS {
            tot += st.lam[(r, c)];
        }
        let scale = if tot > 0.0 { (1.0 - ks) / tot.max(1e-12) } else { 0.0 };
        for (s, name) in C.sector_name.iter().enumerate() {
            let share = st.lam[(s, c)] * scale;
            if extractive(name) {
                D[sector_disp(name)] += dig * share;
                D[4] += rough * share;
            } else {
                D[sector_disp(name)] += share;
            }
        }
        let pot = st.world.forest_pot[c].max(1e-6);
        let woody = ((st.cover[c] / pot - 0.3) / 0.4).clamp(0.0, 1.0);
        let wild = st.lam[(NS, c)] * scale + if tot > 0.0 { 0.0 } else { 1.0 - ks };
        D[3] += wild * woody;
        D[4] += wild * (1.0 - woody);
        D[9] += ks;
        let urban = st.urban[c];
        let der = st.derelict_timer[c] > 0;
        if urban {
            D.iter_mut().for_each(|v| *v = 0.0);
            D[10] = 1.0;
        }
        if der && !urban {
            D.iter_mut().for_each(|v| *v = 0.0);
            D[11] = 1.0;
        }
        if !st.world.land[c] {
            D.iter_mut().for_each(|v| *v = 0.0);
            D[0] = 1.0;
        }
        // the two largest classes, ties to the lower index (a stable descending sort)
        let mut a = 0;
        for k in 1..nd {
            if D[k] > D[a] {
                a = k;
            }
        }
        let mut b = if a == 0 { 1 } else { 0 };
        for k in 0..nd {
            if k != a && D[k] > D[b] {
                b = k;
            }
        }
        let (da, db) = (D[a], D[b]);
        let sh = if db > 0.02 { da / (da + db).max(1e-12) } else { 1.0 };
        let b = if db > 0.02 { b } else { a };
        a_out[c] = a as u8;
        b_out[c] = b as u8;
        s_out[c] = (sh * 255.0).round_ties_even() as u8;
    }
    (a_out, b_out, s_out)
}

/// One project's progress in a yearly snapshot.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ProjectSnap {
    pub a: usize,
    pub b: usize,
    pub done: f64,
}

/// Yearly snapshots (`rec` in Python). Layers are `[year * N + cell]` in `LAYERS` order.
#[derive(Clone, Debug, Default)]
pub struct Record {
    pub n_years: usize,
    pub N: usize,
    pub T: usize,
    pub layers: Vec<Vec<u8>>,
    pub town_active: Vec<u8>,
    pub town_pop: Vec<f32>,
    pub town_urban: Vec<f32>,
    pub town_wage: Vec<f32>,
    pub town_y: Vec<f32>,
    /// `[(year * T + town) * G + good]`
    pub town_price: Vec<f32>,
    /// `[(year * T + town) * NT + tech]`
    pub town_know: Vec<u8>,
    /// Per year: the firms alive and their capacity.
    pub firm_K: Vec<(Vec<usize>, Vec<f32>)>,
    pub projects: Vec<Vec<ProjectSnap>>,
    /// harvest: each town's harvest against an average year, `[year * T + town]`,
    /// and the country's, `[year]`. Empty unless the mechanism is on.
    pub town_harvest: Vec<f32>,
    pub harvest: Vec<f32>,
    /// towns: the `TOWN_LAYERS`, laid out as `layers` is (empty unless `towns.enabled`).
    pub town_layers: Vec<Vec<u8>>,
    /// mines: every working there is or has been, year by year. Empty unless the
    /// mechanism is on.
    pub workings: Vec<Vec<crate::mines::Working>>,
}

impl Record {
    pub fn layer(&self, name: &str) -> &[u8] {
        &self.layers[LAYERS.iter().position(|&l| l == name).unwrap()]
    }

    /// One of the `TOWN_LAYERS`, if the run kept them.
    pub fn town_layer(&self, name: &str) -> Option<&[u8]> {
        let k = TOWN_LAYERS.iter().position(|&l| l == name)?;
        self.town_layers.get(k).map(|l| &l[..])
    }
}

pub fn init(st: &State, C: &Content, P: &Params, n_years: usize) -> Record {
    let (N, T) = (st.N, st.T_max);
    Record {
        n_years,
        N,
        T,
        town_layers: if P.towns.enabled { (0..TOWN_LAYERS.len()).map(|_| vec![0u8; n_years * N]).collect() } else { Vec::new() },
        layers: (0..LAYERS.len()).map(|_| vec![0u8; n_years * N]).collect(),
        town_active: vec![0; n_years * T],
        town_pop: vec![0.0; n_years * T],
        town_urban: vec![0.0; n_years * T],
        town_wage: vec![0.0; n_years * T],
        town_y: vec![0.0; n_years * T],
        town_price: vec![0.0; n_years * T * C.G],
        town_know: vec![0; n_years * T * C.NT],
        firm_K: Vec::new(),
        projects: Vec::new(),
        town_harvest: Vec::new(),
        harvest: Vec::new(),
        workings: Vec::new(),
    }
}

fn to_u8(x: f64) -> u8 {
    x as u8
}

/// The nine map layers of the current year, one byte per cell each, in `LAYERS`
/// order (what `snapshot` records for the year).
pub fn layers(st: &State, C: &Content) -> Vec<Vec<u8>> {
    let N = st.N;
    let mut inten: Vec<f64> = st.cell_labor.iter().map(|&l| (l / 45.0).clamp(0.0, 1.0)).collect();
    for c in 0..N {
        let u = st.land_use[c];
        if u == USE_INDUSTRY {
            inten[c] = st.cell_K_share[c].clamp(0.05, 1.0);
        }
        if u == USE_URBAN {
            inten[c] = (st.urban_res[c] / st.urban_density.max(1.0)).clamp(0.05, 1.0);
        }
        if u == USE_WILD {
            inten[c] = st.cover[c];
        }
    }
    let (mix_a, mix_b, mix_s) = display_mix(st, C);
    let mut trans: Vec<u8> = (0..N).map(|c| st.road[c].min(3) + if st.rail[c] > 0 { 4 } else { 0 }).collect();
    // canals (docs/rules/canals.md): 16 marks a cell with a canal open and 32 one
    // being dug (the layer is empty while they are off); 8 stays a railway building
    for (t, &c) in trans.iter_mut().zip(&st.canal) {
        if c > 0 {
            *t |= 16;
        }
    }
    for pr in &st.projects {
        // roads (docs/probes/dx-roads.md): a road works is not a railway building
        if pr.way == crate::ways::ROAD {
            continue;
        }
        let bit = if pr.way == crate::ways::CANAL { 32 } else { 8 };
        for &c in &pr.new {
            trans[c] |= bit;
        }
    }
    let mut out: Vec<Vec<u8>> = (0..LAYERS.len()).map(|_| vec![0u8; N]).collect();
    for c in 0..N {
        let rent = st.cell_rent[c];
        let lr = rent.max(1e-3).log10();
        let rent_byte = if rent > 1e-3 { to_u8((((lr + 3.0) / 6.0).clamp(0.0, 1.0) * 255.0).round_ties_even()) } else { 0 };
        let d = if st.dcatch[c].is_finite() { st.dcatch[c] } else { 63.75 };
        let values = [
            st.land_use[c],
            to_u8((inten[c] * 255.0).round_ties_even()),
            rent_byte,
            trans[c],
            (st.catch[c] + 1) as u8,
            to_u8((d * 4.0).clamp(0.0, 255.0).round_ties_even()),
            mix_a[c],
            mix_b[c],
            mix_s[c],
        ];
        for (layer, v) in out.iter_mut().zip(values) {
            layer[c] = v;
        }
    }
    out
}

/// towns: the three `TOWN_LAYERS` of the current year, or None in a run whose
/// towns do not keep their ground.
pub fn town_layers(st: &State) -> Option<Vec<Vec<u8>>> {
    if !crate::towns::kept(st) {
        return None;
    }
    let N = st.N;
    let built = (0..N).map(|c| if st.urban[c] { (st.town_built[c] - BUILT_YEAR0).clamp(1, 255) as u8 } else { 0 }).collect();
    let res = (0..N).map(|c| to_u8((st.urban_res[c] / RES_UNIT).round_ties_even().clamp(0.0, 255.0))).collect();
    let works = (0..N).map(|c| to_u8((st.cell_K_share[c].clamp(0.0, 1.0) * 255.0).round_ties_even())).collect();
    Some(vec![built, res, works])
}

/// Knowledge as recorded: 0 to 255.
pub fn know_byte(x: f64) -> u8 {
    to_u8((x.clamp(0.0, 1.0) * 255.0).round_ties_even())
}

pub fn snapshot(rec: &mut Record, st: &State, C: &Content, _P: &Params, k: usize) {
    let (N, T, G, NT) = (st.N, st.T_max, C.G, C.NT);
    let base = k * N;
    for (dst, src) in rec.layers.iter_mut().zip(layers(st, C)) {
        dst[base..base + N].copy_from_slice(&src);
    }
    // towns: when each cell joined its town, its residents, and the works' share of it
    if !rec.town_layers.is_empty() {
        for (dst, src) in rec.town_layers.iter_mut().zip(town_layers(st).unwrap_or_default()) {
            dst[base..base + N].copy_from_slice(&src);
        }
    }
    for t in 0..T {
        let act = st.town_active[t];
        rec.town_active[k * T + t] = act as u8;
        rec.town_pop[k * T + t] = st.pop[t] as f32;
        rec.town_urban[k * T + t] = (if act { st.urban_pop[t] } else { 0.0 }) as f32;
        rec.town_wage[k * T + t] = st.wbar[t] as f32;
        rec.town_y[k * T + t] = st.y_ratio[t] as f32;
        for g in 0..G {
            rec.town_price[(k * T + t) * G + g] = st.pbar[(t, g)] as f32;
        }
        for j in 0..NT {
            rec.town_know[(k * T + t) * NT + j] = know_byte(st.know[(t, j)]);
        }
    }
    let alive: Vec<usize> = (0..st.f_alive.len()).filter(|&f| st.f_alive[f]).collect();
    let K: Vec<f32> = alive.iter().map(|&f| st.f_K[f] as f32).collect();
    rec.firm_K.push((alive, K));
    rec.projects.push(
        st.projects
            .iter()
            // canals: a canal being dug is not on the list of railways building;
            // roads (docs/probes/dx-roads.md): nor is a road works
            .filter(|pr| pr.way != crate::ways::CANAL && pr.way != crate::ways::ROAD)
            .map(|pr| {
                let order: f64 = crate::util::pairwise_sum(&pr.order);
                let need: f64 = crate::util::pairwise_sum(&pr.need);
                ProjectSnap { a: pr.a, b: pr.b, done: 1.0 - order / need.max(1e-9) }
            })
            .collect(),
    );
}

/// Python's `_clean`: floats rounded to 5 decimals, non-finite floats as null.
pub fn clean(v: &mut Value) {
    match v {
        Value::Array(a) => a.iter_mut().for_each(clean),
        Value::Object(o) => o.values_mut().for_each(clean),
        Value::Number(n) if n.is_f64() => {
            let x = n.as_f64().unwrap();
            *v = if x.is_finite() { json!(crate::util::round_dp(x, 5)) } else { Value::Null };
        }
        _ => {}
    }
}

/// JSON text with every non-ASCII character escaped, like Python's `json.dumps` default.
pub fn ensure_ascii(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii() {
            out.push(ch);
        } else {
            let mut buf = [0u16; 2];
            for unit in ch.encode_utf16(&mut buf) {
                out.push_str(&format!("\\u{:04x}", unit));
            }
        }
    }
    out
}

/// The events as Python's `_clean(events)` would leave them.
pub fn events_value(events: &[Event]) -> Value {
    let mut v = serde_json::to_value(events).expect("events serialise");
    clean(&mut v);
    v
}

/// One typed array of the container.
struct Array {
    name: String,
    dtype: &'static str,
    shape: Vec<usize>,
    bytes: Vec<u8>,
}

fn u8_array(name: &str, shape: Vec<usize>, data: Vec<u8>) -> Array {
    Array { name: name.to_string(), dtype: "uint8", shape, bytes: data }
}

fn i32_array(name: &str, shape: Vec<usize>, data: &[i32]) -> Array {
    Array { name: name.to_string(), dtype: "int32", shape, bytes: data.iter().flat_map(|v| v.to_le_bytes()).collect() }
}

fn f32_array(name: &str, shape: Vec<usize>, data: &[f32]) -> Array {
    Array { name: name.to_string(), dtype: "float32", shape, bytes: data.iter().flat_map(|v| v.to_le_bytes()).collect() }
}

/// Today's date as YYYY-MM-DD (UTC), for the header's `generated` field.
pub(crate) fn today() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = (secs / 86_400) as i64;
    // civil date from days since 1970-01-01 (Howard Hinnant's algorithm)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

/// The resource fields stored once per run, 0 to 255 per cell.
pub const STATIC_FIELDS: [&str; 7] = ["fertility", "ore", "coal", "rock", "clay", "pasture", "forest_pot"];

/// Height, 0 to 255 per cell.
pub fn elev_bytes(world: &World) -> Vec<u8> {
    world.h.iter().map(|&h| to_u8((h * 255.0).round_ties_even())).collect()
}

/// River size per cell: 0 for none, else 1 plus 20 times log2 of the upstream area.
pub fn river_bytes(world: &World) -> Vec<u8> {
    (0..world.h.len())
        .map(|c| if world.river[c] { to_u8(1.0 + (world.acc[c].max(1.0).log2() * 20.0).clamp(0.0, 253.0)) } else { 0 })
        .collect()
}

/// A 0 to 1 field as 0 to 255 per cell.
pub fn field_bytes(field: &[f64]) -> Vec<u8> {
    field.iter().map(|&v| to_u8((v.clamp(0.0, 1.0) * 255.0).round_ties_even())).collect()
}

/// Binary container for one run: JSON header + aligned typed arrays.
///
/// lite=true keeps series, events and town data but drops map layers and
/// firm histories; used for a comparison scenario shown in charts only.
#[allow(clippy::too_many_arguments)]
pub fn pack(st: &State, C: &Content, P: &Params, events: &[Event], rec: &Record, label: &str, lite: bool) -> Vec<u8> {
    let world = &st.world;
    let (H, W, N) = (world.H, world.W, st.N);
    let y0 = C.start_year;
    let n_years = rec.n_years;
    let F = if lite { 0 } else { st.F_count };
    let mut arrays: Vec<Array> = Vec::new();

    arrays.push(u8_array("elev", vec![H, W], elev_bytes(world)));
    arrays.push(u8_array("terrain", vec![H, W], world.terrain.clone()));
    arrays.push(u8_array("river", vec![H, W], river_bytes(world)));
    arrays.push(u8_array("nav", vec![H, W], world.navigable.iter().map(|&n| n as u8).collect()));
    let down: Vec<i32> = world.down.iter().map(|&d| d as i32).collect();
    arrays.push(i32_array("down", vec![N], &down));
    for f in STATIC_FIELDS {
        arrays.push(u8_array(f, vec![H, W], field_bytes(world.field(f))));
    }
    if !lite {
        for (name, layer) in LAYERS.iter().zip(&rec.layers) {
            arrays.push(u8_array(name, vec![n_years, N], layer.clone()));
        }
        // towns: three more, only in a run that kept them
        for (name, layer) in TOWN_LAYERS.iter().zip(&rec.town_layers) {
            arrays.push(u8_array(name, vec![n_years, N], layer.clone()));
        }
    }
    let T = rec.T;
    arrays.push(u8_array("town_active", vec![n_years, T], rec.town_active.clone()));
    arrays.push(f32_array("town_pop", vec![n_years, T], &rec.town_pop));
    arrays.push(f32_array("town_urban", vec![n_years, T], &rec.town_urban));
    arrays.push(f32_array("town_wage", vec![n_years, T], &rec.town_wage));
    arrays.push(f32_array("town_y", vec![n_years, T], &rec.town_y));
    arrays.push(f32_array("town_price", vec![n_years, T, C.G], &rec.town_price));
    arrays.push(u8_array("town_know", vec![n_years, T, C.NT], rec.town_know.clone()));

    // store each firm's capacity only over the years it existed
    let mut fk = vec![0.0f32; F * n_years];
    if !lite {
        for (k, (ids, K)) in rec.firm_K.iter().enumerate() {
            for (&f, &v) in ids.iter().zip(K) {
                if f < F {
                    fk[f * n_years + k] = v;
                }
            }
        }
    }
    let mut first = vec![n_years as i32; F];
    let mut lens = vec![0i32; F];
    let mut offs = vec![0i32; F];
    let mut vals: Vec<f32> = Vec::new();
    let mut acc = 0i32;
    for f in 0..F {
        let row = &fk[f * n_years..(f + 1) * n_years];
        let lo = row.iter().position(|&v| v > 0.0);
        let hi = row.iter().rposition(|&v| v > 0.0);
        offs[f] = acc;
        if let (Some(lo), Some(hi)) = (lo, hi) {
            first[f] = lo as i32;
            lens[f] = (hi - lo + 1) as i32;
            vals.extend_from_slice(&row[lo..=hi]);
            acc += lens[f];
        }
    }
    arrays.push(i32_array("firm_first", vec![F], &first));
    arrays.push(i32_array("firm_len", vec![F], &lens));
    arrays.push(i32_array("firm_off", vec![F], &offs));
    arrays.push(f32_array("firm_Kvals", vec![vals.len()], &vals));
    let names = &st.series.names;
    let n_months = st.series.data.first().map_or(0, |d| d.len());
    let ser: Vec<f32> = st.series.data.iter().flat_map(|d| d.iter().map(|&v| v as f32)).collect();
    arrays.push(f32_array("series", vec![names.len(), n_months], &ser));
    // harvest: only a run with the mechanism on has these two
    if !rec.town_harvest.is_empty() {
        arrays.push(f32_array("town_harvest", vec![n_years, T], &rec.town_harvest));
        arrays.push(f32_array("harvest", vec![n_years], &rec.harvest));
    }
    // mines: only a run with the mechanism on has these (docs/rules/mines.md).
    // Year k's workings are rows mine_first[k] .. mine_first[k + 1] of the rest.
    if !lite && !rec.workings.is_empty() {
        let mut first = vec![0i32; n_years + 1];
        for k in 0..n_years {
            first[k + 1] = first[k] + rec.workings.get(k).map_or(0, |w| w.len()) as i32;
        }
        let all: Vec<&crate::mines::Working> = rec.workings.iter().flatten().collect();
        let n = all.len();
        let ints = |f: &dyn Fn(&crate::mines::Working) -> i32| all.iter().map(|w| f(w)).collect::<Vec<i32>>();
        let reals = |f: &dyn Fn(&crate::mines::Working) -> f32| all.iter().map(|w| f(w)).collect::<Vec<f32>>();
        arrays.push(i32_array("mine_first", vec![n_years + 1], &first));
        arrays.push(i32_array("mine_cell", vec![n], &ints(&|w| w.cell as i32)));
        arrays.push(u8_array("mine_sector", vec![n], all.iter().map(|w| w.sector as u8).collect()));
        arrays.push(i32_array("mine_opened", vec![n], &ints(&|w| w.opened)));
        arrays.push(i32_array("mine_left", vec![n], &ints(&|w| w.left)));
        arrays.push(f32_array("mine_hands", vec![n], &reals(&|w| w.hands)));
        arrays.push(f32_array("mine_out", vec![n], &reals(&|w| w.out)));
        arrays.push(f32_array("mine_heap", vec![n], &reals(&|w| w.heap)));
        arrays.push(i32_array("mine_cells", vec![n], &ints(&|w| w.cells as i32)));
        arrays.push(f32_array("mine_surface", vec![n], &reals(&|w| w.surface)));
        arrays.push(f32_array("mine_royalty", vec![n], &reals(&|w| w.royalty)));
    }
    // land: the fine grid, the rivers' lines and the water's cell fields, only
    // for an island the mechanism made (`landcells::static_arrays` lists them)
    if let (Some(lf), false) = (&world.landform, lite) {
        for (name, dtype, shape, bytes) in crate::landcells::static_arrays(lf, W, H) {
            arrays.push(Array { name, dtype, shape, bytes });
        }
    }
    // accounts: only a run that kept books has these (docs/rules/accounts.md has their shapes)
    for table in crate::accounts::tables(st, lite) {
        arrays.push(match table {
            crate::accounts::Table::F32(name, shape, data) => f32_array(&name, shape, &data),
            crate::accounts::Table::I32(name, shape, data) => i32_array(&name, shape, &data),
        });
    }

    let towns: Vec<Value> = (0..st.town_name.len())
        .map(|i| json!({"name": st.town_name[i], "node": st.town_node[i], "founded": st.town_founded[i]}))
        .collect();
    let firms: Vec<Value> = (0..F)
        .map(|f| {
            json!({
                "name": st.f_name[f], "method": st.f_method[f], "cell": st.f_cell[f], "town": st.f_town[f],
                "born": st.f_born[f], "died": if st.f_alive[f] { -1 } else { st.f_died[f] },
                "parent": st.f_parent[f], "reason": st.f_reason[f],
            })
        })
        .collect();
    let goods: Vec<Value> = (0..C.G)
        .map(|g| json!({"name": C.good_name[g], "label": C.good_label[g], "color": C.good_color[g], "weight": C.weight[g], "price": C.base_price[g]}))
        .collect();
    let methods: Vec<Value> = (0..C.M)
        .map(|m| {
            let inputs: Vec<Value> = (0..C.G).filter(|&g| C.m_in[(m, g)] != 0.0).map(|g| json!([g, C.m_in[(m, g)]])).collect();
            // (not in the Python's header: each good's starting price above and, here, what
            // a unit of monthly capacity costs to build at those prices. With the inputs, the
            // heat and the labour they are the method's process, from which the viewer lays
            // a works out, so that a trade its art has never heard of gets a plan of its own)
            let capital: f64 = (0..C.G).map(|g| C.base_price[g] * C.m_cap[(m, g)]).sum();
            json!({"name": C.method_name[m], "label": C.method_label[m], "out": C.method_out[m], "inputs": inputs,
                   "heat": C.m_heat[m], "labor": C.m_labor[m], "K0": C.m_K0[m], "capital": capital})
        })
        .collect();
    let techs: Vec<Value> = (0..C.NT)
        .map(|t| {
            json!({"name": C.tech_name[t], "label": C.tech_label[t], "kind": C.tech_kind[t], "known": C.tech_known[t],
                   "year": st.tech_year[t], "town": st.tech_town[t]})
        })
        .collect();
    let sectors: Vec<Value> =
        (0..C.NS).map(|s| json!({"name": C.sector_name[s], "label": C.sector_label[s], "good": C.sector_good[s]})).collect();
    let mut series_meta = serde_json::Map::new();
    for (n, m) in names.iter().zip(&st.series.meta) {
        series_meta.insert(n.clone(), serde_json::to_value(m).unwrap());
    }
    let mut share_groups = serde_json::Map::new();
    for &(gname, kind, members) in SHARE_GROUPS {
        share_groups.insert(gname.to_string(), json!({"kind": kind, "members": members}));
    }
    let mut table: Vec<Value> = Vec::new();
    let mut offset = 0usize;
    for a in &arrays {
        let nbytes = a.bytes.len();
        table.push(json!({"name": a.name, "dtype": a.dtype, "shape": a.shape, "offset": offset, "nbytes": nbytes}));
        offset += nbytes + (8 - nbytes % 8) % 8;
    }
    let mut header = json!({
        "meta": {
            "label": label, "lite": lite, "seed": P.seed, "W": W, "H": H, "start_year": y0,
            "end_year": C.end_year, "n_years": n_years, "rent_regime": P.rent_regime.as_str(),
            "generated": today(), "freight": P.freight, "km_per_cell": 2.0,
        },
        "goods": goods,
        "methods": methods,
        "techs": techs,
        "sectors": sectors,
        "uses": USE_NAMES,
        "disp": DISP,
        "towns": towns,
        "firms": firms,
        "events": serde_json::to_value(events).unwrap(),
        "rail_opened": serde_json::to_value(&st.rail_opened).unwrap(),
        "projects": serde_json::to_value(&rec.projects).unwrap(),
        "series": {"names": names, "meta": series_meta, "start_year": y0},
        "share_groups": share_groups,
        "arrays": table,
    });
    // canals (docs/rules/canals.md): the network as links from cell to cell, each with
    // the year its canal was begun and the moment it opened (null if still digging)
    if P.canals.enabled {
        header["canals"] = json!({
            "cost": P.canals.cost,
            "wharf": P.canals.wharf,
            "links": serde_json::to_value(crate::canals::links_out(st)).unwrap(),
        });
    }
    // land: the lakes, the sea outlets and the fine grid's size
    if let Some(lf) = &world.landform {
        header["land"] = crate::landcells::static_header(lf);
    }
    // accounts: the tables' columns and the island's account of every year
    if let Some(books) = crate::accounts::header(st) {
        header["accounts"] = books;
    }
    clean(&mut header);
    let hjson = ensure_ascii(&serde_json::to_string(&header).unwrap()).into_bytes();
    let pad = (8 - (4 + hjson.len()) % 8) % 8;
    let mut blob = Vec::with_capacity(4 + hjson.len() + pad + offset);
    blob.extend_from_slice(&((hjson.len() + pad) as u32).to_le_bytes());
    blob.extend_from_slice(&hjson);
    blob.extend(std::iter::repeat_n(b' ', pad));
    for a in &arrays {
        blob.extend_from_slice(&a.bytes);
        blob.extend(std::iter::repeat_n(0u8, (8 - a.bytes.len() % 8) % 8));
    }
    blob
}
