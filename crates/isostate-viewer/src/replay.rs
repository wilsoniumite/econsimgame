//! Reader for the run.bin replay container written by `record.pack`.
//!
//! Layout: a little-endian u32 header length (including padding), a JSON
//! header, then typed arrays aligned to 8 bytes. Map layers are stored
//! layer-major, `[year, cell]`.

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Meta {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub lite: bool,
    #[serde(default)]
    pub seed: u64,
    #[serde(rename = "W")]
    pub w: usize,
    #[serde(rename = "H")]
    pub h: usize,
    pub start_year: i32,
    pub n_years: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Good {
    pub name: String,
    pub label: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Method {
    pub name: String,
    pub label: String,
    pub out: usize,
    #[serde(rename = "K0")]
    pub k0: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Town {
    pub name: String,
    pub node: usize,
    pub founded: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Firm {
    pub name: String,
    pub method: usize,
    pub cell: usize,
    #[serde(default)]
    pub town: i32,
    pub born: i32,
    #[serde(default)]
    pub died: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Event {
    pub year: f32,
    pub kind: String,
    #[serde(default)]
    pub weight: f32,
    pub title: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub cell: Option<i64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ArrayInfo {
    pub name: String,
    pub dtype: String,
    pub shape: Vec<usize>,
    pub offset: usize,
    pub nbytes: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Header {
    pub meta: Meta,
    pub goods: Vec<Good>,
    pub methods: Vec<Method>,
    pub towns: Vec<Town>,
    pub firms: Vec<Firm>,
    pub events: Vec<Event>,
    #[serde(default)]
    pub disp: Vec<String>,
    pub uses: Vec<String>,
    pub arrays: Vec<ArrayInfo>,
}

/// Everything the viewer needs from one run, copied into plain vectors.
pub struct Replay {
    pub header: Header,
    pub w: usize,
    pub h: usize,
    pub n: usize,
    pub ny: usize,
    pub y0: i32,
    pub n_towns: usize,
    pub elev: Vec<u8>,
    pub terrain: Vec<u8>,
    pub river: Vec<u8>,
    pub down: Vec<i32>,
    pub use_: Vec<u8>,
    pub inten: Vec<u8>,
    pub trans: Vec<u8>,
    pub catch: Vec<u8>,
    pub mix_a: Vec<u8>,
    pub mix_b: Vec<u8>,
    pub mix_s: Vec<u8>,
    pub town_active: Vec<u8>,
    pub town_pop: Vec<f32>,
    pub town_urban: Vec<f32>,
    pub firm_first: Vec<i32>,
    pub firm_len: Vec<i32>,
    pub firm_off: Vec<i32>,
    pub firm_k: Vec<f32>,
    pub series_names: Vec<String>,
    pub series: Vec<f32>,
    pub series_len: usize,
}

fn slice<'a>(buf: &'a [u8], base: usize, a: &ArrayInfo) -> &'a [u8] {
    &buf[base + a.offset..base + a.offset + a.nbytes]
}

pub fn load(path: &str) -> Result<Replay, String> {
    let buf = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    if buf.len() < 8 {
        return Err(format!("{path} is too short to be a run"));
    }
    let hlen = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let header: Header = serde_json::from_slice(&buf[4..4 + hlen])
        .map_err(|e| format!("bad header in {path}: {e}"))?;
    if header.meta.lite {
        return Err(format!("{path} is a lite run (charts only) and has no map layers"));
    }
    let base = 4 + hlen;
    let find = |name: &str| -> Result<&ArrayInfo, String> {
        header
            .arrays
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| format!("{path} has no array called {name}"))
    };
    let u8s = |name: &str| -> Result<Vec<u8>, String> { Ok(slice(&buf, base, find(name)?).to_vec()) };
    let f32s = |name: &str| -> Result<Vec<f32>, String> {
        Ok(bytemuck::pod_collect_to_vec::<u8, f32>(slice(&buf, base, find(name)?)))
    };
    let i32s = |name: &str| -> Result<Vec<i32>, String> {
        Ok(bytemuck::pod_collect_to_vec::<u8, i32>(slice(&buf, base, find(name)?)))
    };
    let (w, h, ny) = (header.meta.w, header.meta.h, header.meta.n_years);
    let n_towns = find("town_pop")?.shape[1];
    let series_info = find("series")?;
    let series_len = series_info.shape[1];
    let series_names = serde_json::from_slice::<serde_json::Value>(&buf[4..4 + hlen])
        .ok()
        .and_then(|v| v["series"]["names"].as_array().cloned())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let has_mix = header.arrays.iter().any(|a| a.name == "mix_a");
    let use_ = u8s("use")?;
    let (mix_a, mix_b, mix_s) = if has_mix {
        (u8s("mix_a")?, u8s("mix_b")?, u8s("mix_s")?)
    } else {
        // older runs: fall back to the dominant use, drawn as a single class
        let map = |u: u8| -> u8 { [0, 4, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11][u as usize % 12] };
        let a: Vec<u8> = use_.iter().map(|&u| map(u)).collect();
        (a.clone(), a, vec![255; use_.len()])
    };
    Ok(Replay {
        w,
        h,
        n: w * h,
        ny,
        y0: header.meta.start_year,
        n_towns,
        elev: u8s("elev")?,
        terrain: u8s("terrain")?,
        river: u8s("river")?,
        down: i32s("down")?,
        use_,
        inten: u8s("inten")?,
        trans: u8s("trans")?,
        catch: u8s("catch")?,
        mix_a,
        mix_b,
        mix_s,
        town_active: u8s("town_active")?,
        town_pop: f32s("town_pop")?,
        town_urban: f32s("town_urban")?,
        firm_first: i32s("firm_first")?,
        firm_len: i32s("firm_len")?,
        firm_off: i32s("firm_off")?,
        firm_k: f32s("firm_Kvals")?,
        series: f32s("series")?,
        series_len,
        series_names,
        header,
    })
}

/// Capacity of firm `f` in year index `k` (0 when it does not exist that year).
pub fn firm_k(r: &Replay, f: usize, k: usize) -> f32 {
    let first = r.firm_first[f];
    let len = r.firm_len[f];
    let k = k as i32;
    if k < first || k >= first + len {
        return 0.0;
    }
    r.firm_k[(r.firm_off[f] + k - first) as usize]
}

/// Annual mean of a named monthly series, or None if the run does not have it.
pub fn annual(r: &Replay, name: &str, k: usize) -> Option<f32> {
    let i = r.series_names.iter().position(|s| s == name)?;
    let row = &r.series[i * r.series_len..(i + 1) * r.series_len];
    let (a, b) = (k * 12, ((k + 1) * 12).min(r.series_len));
    if a >= b {
        return None;
    }
    let vals: Vec<f32> = row[a..b].iter().copied().filter(|v| v.is_finite()).collect();
    if vals.is_empty() {
        None
    } else {
        Some(vals.iter().sum::<f32>() / vals.len() as f32)
    }
}
