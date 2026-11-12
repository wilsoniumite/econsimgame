//! Reader for the run.bin replay container written by `record.pack`.
//!
//! Layout: a little-endian u32 header length (including padding), a JSON
//! header, then typed arrays aligned to 8 bytes. Map layers are stored
//! layer-major, `[year, cell]`.

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Meta {
    /// The run's own name, as the recorder gives it ("a new game" in a live
    /// one). Nothing shows it yet, and it is kept because a live game fills it
    /// (live.rs): a list of games to choose from would be where it is read.
    #[serde(default)]
    #[allow(dead_code)]
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
    /// the sums that made its founders build here (see firms.reason_record)
    #[serde(default)]
    pub reason: Option<serde_json::Value>,
    /// the family that owns it, in a game with families
    #[serde(default)]
    pub owner: Option<usize>,
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
    /// The square the news is about, where it is about one. Nothing in the
    /// viewer goes there yet (a click on a piece of news that took the camera
    /// to its place would); kept because a live game fills it (live.rs).
    #[serde(default)]
    #[allow(dead_code)]
    pub cell: Option<i64>,
    /// the family the news is about
    #[serde(default)]
    pub family: Option<usize>,
    /// chronicle: whether a history of the decade would keep this item (absent
    /// in a run made without the chronicle)
    #[serde(default)]
    pub kept: Option<bool>,
    /// chronicle: the item's place in a live game's event list, by which the
    /// simulation says that its `kept` mark has changed
    #[serde(skip)]
    pub seq: usize,
}

impl Event {
    /// Whether this belongs in the major feed, on the timeline and in the
    /// toasts: the chronicle's mark where there is one, else weight 0.6 or more.
    pub fn major(&self) -> bool {
        self.kept.unwrap_or(self.weight >= 0.6)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ArrayInfo {
    pub name: String,
    /// how its numbers are stored, in numpy's words (uint8, int32, float32):
    /// checked against how the viewer reads the array (`typed`)
    pub dtype: String,
    pub shape: Vec<usize>,
    pub offset: usize,
    pub nbytes: usize,
}

/// The names of the monthly series, in the order of the rows of the `series` array.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct SeriesNames {
    #[serde(default)]
    pub names: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Header {
    pub meta: Meta,
    pub goods: Vec<Good>,
    pub methods: Vec<Method>,
    pub towns: Vec<Town>,
    pub firms: Vec<Firm>,
    pub events: Vec<Event>,
    /// the names of the display classes of `mix_a` and `mix_b`, and of the land
    /// uses of `use`, in the order of their numbers: checked against the order
    /// the viewer draws them in (`known_names`). Both recorders, the Python's
    /// and the core's, have always written both.
    #[serde(default)]
    pub disp: Vec<String>,
    pub uses: Vec<String>,
    pub arrays: Vec<ArrayInfo>,
    #[serde(default)]
    pub series: SeriesNames,
    /// what a run made on the second island generator's land says of it
    /// (newland.rs); a run made on the old island has none
    #[serde(default)]
    pub land: Option<crate::newland::LandHeader>,
}

/// Everything the viewer needs from one run, copied into plain vectors.
#[derive(Clone)]
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
    /// land rent of every cell by year, on the record's log scale (see `rent_of`); empty if the run has none
    pub rent: Vec<u8>,
    /// path cost from every cell to its market town by year, in quarter units (255: no way there)
    pub dist: Vec<u8>,
    pub mix_a: Vec<u8>,
    pub mix_b: Vec<u8>,
    pub mix_s: Vec<u8>,
    pub town_active: Vec<u8>,
    pub town_pop: Vec<f32>,
    pub town_urban: Vec<f32>,
    /// the year's mean wage in every town, `[year, town]`; empty if the run has none
    pub town_wage: Vec<f32>,
    /// the year's mean price of every good in every town, `[year, town, good]`
    pub town_price: Vec<f32>,
    pub firm_first: Vec<i32>,
    pub firm_len: Vec<i32>,
    pub firm_off: Vec<i32>,
    pub firm_k: Vec<f32>,
    pub series_names: Vec<String>,
    pub series: Vec<f32>,
    pub series_len: usize,
    /// years with data so far: all of them for a recorded run, a growing number in a live game
    pub filled: usize,
    /// owning family of every cell by year, `[year, cell]`, -1 for none (empty without families)
    pub owner: Vec<i8>,
    /// Town ground that is kept (docs/rules/towns.md), `[year, cell]`; all three
    /// are empty in a run whose towns do not keep their ground (the port's rules).
    /// The year the cell joined its town less `BUILT_YEAR0`, 1 to 255; 0 where
    /// the cell is not town ground.
    pub built: Vec<u8>,
    /// the cell's residents, in units of `RES_UNIT` people
    pub res: Vec<u8>,
    /// the share of the cell under works, 0 to 255, on every cell
    pub works: Vec<u8>,
    /// the quarries, pits and mines of each year, where the run keeps them as
    /// workings (the core's docs/rules/mines.md); empty for a run that does not,
    /// whose pits are read off the land mix as before
    pub workings: Vec<Vec<Working>>,
    /// The second island generator's land, where the run was made on it
    /// (newland.rs): the heights every 500 m, the rivers' lines, the lakes.
    /// None for a run made on the old island, which is drawn as it always
    /// was. It hangs here in an `Arc` of its own, so that copying a run (as a
    /// live game does when a year arrives while a scenery build holds it)
    /// copies a pointer and not four megabytes.
    pub land: Option<std::sync::Arc<crate::newland::NewLand>>,
}

/// `Replay::built` counts years from here (`isostate_core::record::BUILT_YEAR0`).
pub const BUILT_YEAR0: i32 = 1645;
/// People to one step of `Replay::res` (`isostate_core::record::RES_UNIT`).
pub const RES_UNIT: f32 = 8.0;

impl Replay {
    /// Whether the run's towns keep their ground, and the record says when
    /// each cell joined its town, who lives on it and how much of it is works.
    pub fn kept(&self) -> bool {
        !self.built.is_empty() && !self.res.is_empty()
    }

    /// The year cell `c` joined its town, if it is town ground in year `k` of a run that keeps it.
    pub fn joined(&self, k: usize, c: usize) -> Option<i32> {
        self.built.get(k * self.n + c).filter(|&&b| b > 0).map(|&b| BUILT_YEAR0 + b as i32)
    }

    /// The residents of cell `c` in year `k` (nothing in a run that does not record them).
    pub fn residents(&self, k: usize, c: usize) -> f32 {
        self.res.get(k * self.n + c).map_or(0.0, |&b| b as f32 * RES_UNIT)
    }

    /// The share of cell `c` under works in year `k`, 0 to 1.
    pub fn works_share(&self, k: usize, c: usize) -> f32 {
        self.works.get(k * self.n + c).map_or(0.0, |&b| b as f32 / 255.0)
    }
}

/// One working in one year: a pit head on a cell, with what the core says of it.
#[derive(Clone, Debug)]
pub struct Working {
    pub cell: u32,
    /// its display class (`world::QUARRY` .. `world::COALMINE`)
    pub class: u8,
    /// the year it was opened; 0 for one that lies abandoned
    pub opened: i32,
    /// mean hands over the year
    pub hands: f32,
    /// What it raised over the year, everything raised there so far (its spoil
    /// heap), and the share of its cell its pit head, yards and spoil take.
    /// Nothing draws them yet: they are what pits drawn to their size would
    /// read, and a live game fills them (live.rs).
    #[allow(dead_code)]
    pub out: f32,
    #[allow(dead_code)]
    pub heap: f32,
    #[allow(dead_code)]
    pub surface: f32,
}

/// The display class of one of the core's sectors: the sectors come in the
/// order of the display classes, after sea and before heath (farm, pasture and
/// wood are 1 to 3, the four kinds of diggings 5 to 8).
pub fn sector_class(sector: usize) -> u8 {
    if sector < 3 { sector as u8 + 1 } else { sector as u8 + 2 }
}

/// The bytes of one array, or an error if the file ends before the array does.
fn slice<'a>(path: &str, buf: &'a [u8], base: usize, a: &ArrayInfo) -> Result<&'a [u8], String> {
    let end = base.checked_add(a.offset).and_then(|s| s.checked_add(a.nbytes));
    match end {
        Some(end) if end <= buf.len() => Ok(&buf[end - a.nbytes..end]),
        _ => Err(format!(
            "{path} is cut short: its array {} should end {} bytes in and the file has {}. Was it copied while the run was still being written?",
            a.name,
            end.map_or_else(|| "too many".to_string(), |e| e.to_string()),
            buf.len()
        )),
    }
}

/// The bytes of one array the viewer reads as `want` (in numpy's words, as the
/// recorders write them: uint8, int32, float32), or an error if the run says
/// it holds something else: read by its name alone, an array of other numbers
/// would come out as nonsense and nothing would say so.
fn typed<'a>(path: &str, buf: &'a [u8], base: usize, a: &ArrayInfo, want: &str) -> Result<&'a [u8], String> {
    if a.dtype != want {
        return Err(format!("{path}: its array {} holds {} numbers, and the viewer reads it as {want}", a.name, a.dtype));
    }
    slice(path, buf, base, a)
}

/// Whether the run numbers its display classes and its land uses as the
/// viewer does: the ground's shader, the inspector and the towns read them by
/// their numbers, so a run that numbered them otherwise would be drawn wrong
/// without a word. A list the run does not give (as in the tests' tiny runs)
/// is taken on trust.
fn known_names(path: &str, header: &Header) -> Result<(), String> {
    let lists: [(&str, &[String], &[&str]); 2] = [
        ("display classes", &header.disp, &isostate_core::record::DISP),
        ("land uses", &header.uses, &isostate_core::land::USE_NAMES),
    ];
    for (what, names, known) in lists {
        if !names.is_empty() && !names.iter().map(String::as_str).eq(known.iter().copied()) {
            return Err(format!("{path}: its {what} are {}, where the viewer draws {}, in that order", names.join(", "), known.join(", ")));
        }
    }
    Ok(())
}

/// What is said of a run whose header speaks of the new land and which has
/// none of its arrays (they were left out, as a lite pack leaves them out):
/// that its land cannot be drawn, and what is drawn in its place.
fn no_land_arrays(path: &str) -> String {
    format!(
        "{path} was made on the new land (the second island generator's) and does not carry its arrays, so its land cannot be drawn. What is drawn in its place is the old kind of island, made up from the cells' heights: the viewer's own rivers, no lakes, and the tideways as dry land"
    )
}

pub fn load(path: &str) -> Result<Replay, String> {
    let buf = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    parse(path, &buf)
}

/// Read a run from the bytes of its file; `path` is for the messages.
fn parse(path: &str, buf: &[u8]) -> Result<Replay, String> {
    if buf.len() < 8 {
        return Err(format!("{path} is too short to be a run"));
    }
    let hlen = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let base = 4 + hlen;
    if base > buf.len() {
        return Err(format!("{path} is cut short: its header should be {hlen} bytes long and the file has {} after the first four", buf.len() - 4));
    }
    let header: Header = serde_json::from_slice(&buf[4..base]).map_err(|e| format!("bad header in {path}: {e}"))?;
    if header.meta.lite {
        // a lite pack of a run made on the new land keeps the header's word of it and none of its arrays
        let land = if header.land.is_some() {
            ", and the land it was made on (the new island's heights, rivers and lakes) is not in a lite pack either: nothing of the island can be drawn from it, so record the run again without --lite"
        } else {
            ""
        };
        return Err(format!("{path} is a lite run (charts only) and has no map layers{land}"));
    }
    let find = |name: &str| -> Result<&ArrayInfo, String> {
        header
            .arrays
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| format!("{path} has no array called {name}"))
    };
    known_names(path, &header)?;
    let u8s = |name: &str| -> Result<Vec<u8>, String> { Ok(typed(path, buf, base, find(name)?, "uint8")?.to_vec()) };
    let f32s = |name: &str| -> Result<Vec<f32>, String> {
        Ok(bytemuck::pod_collect_to_vec::<u8, f32>(typed(path, buf, base, find(name)?, "float32")?))
    };
    let i32s = |name: &str| -> Result<Vec<i32>, String> {
        Ok(bytemuck::pod_collect_to_vec::<u8, i32>(typed(path, buf, base, find(name)?, "int32")?))
    };
    // the second dimension of a table: towns in `town_pop`, months in `series`
    let across = |name: &str| -> Result<usize, String> {
        find(name)?.shape.get(1).copied().ok_or_else(|| format!("{path}: its array {name} should be a table with two dimensions"))
    };
    let (w, h, ny) = (header.meta.w, header.meta.h, header.meta.n_years);
    let n_towns = across("town_pop")?;
    let series_len = across("series")?;
    let series_names = header.series.names.clone();
    let has_mix = header.arrays.iter().any(|a| a.name == "mix_a");
    // what the map overlays read; a run without them still plays, and the overlays say so
    let has = |name: &str| header.arrays.iter().any(|a| a.name == name);
    let rent = if has("rent") { u8s("rent")? } else { Vec::new() };
    let dist = if has("dist") { u8s("dist")? } else { Vec::new() };
    let town_wage = if has("town_wage") { f32s("town_wage")? } else { Vec::new() };
    let town_price = if has("town_price") { f32s("town_price")? } else { Vec::new() };
    // the three layers of town ground that is kept: all of them or none
    let (built, res, works) = if has("built") && has("res") && has("works") { (u8s("built")?, u8s("res")?, u8s("works")?) } else { (Vec::new(), Vec::new(), Vec::new()) };
    let use_ = u8s("use")?;
    let (mix_a, mix_b, mix_s) = if has_mix {
        (u8s("mix_a")?, u8s("mix_b")?, u8s("mix_s")?)
    } else {
        // older runs: fall back to the dominant use, drawn as a single class
        let map = |u: u8| -> u8 { [0, 4, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11][u as usize % 12] };
        let a: Vec<u8> = use_.iter().map(|&u| map(u)).collect();
        (a.clone(), a, vec![255; use_.len()])
    };
    // a year's layer has a value for every cell: anything less would be read past its end later
    if use_.len() != ny * w * h {
        return Err(format!("{path}: its land use has {} values, where {ny} years of {w} by {h} cells need {}", use_.len(), ny * w * h));
    }
    // the workings, where the run keeps its quarries, pits and mines as such: year k's
    // are rows mine_first[k] .. mine_first[k + 1] of the other arrays. A run without
    // them (the port, an older recording) has none, and its pits are read off the mix.
    let mut workings: Vec<Vec<Working>> = Vec::new();
    if has("mine_first") {
        let first = i32s("mine_first")?;
        let (cell, sector, opened) = (i32s("mine_cell")?, u8s("mine_sector")?, i32s("mine_opened")?);
        let (hands, out, heap, surface) = (f32s("mine_hands")?, f32s("mine_out")?, f32s("mine_heap")?, f32s("mine_surface")?);
        // every row has a value in every column, or a row would be read past a column's end
        let rows = cell.len();
        if [sector.len(), opened.len(), hands.len(), out.len(), heap.len(), surface.len()].iter().any(|&l| l != rows) {
            return Err(format!("{path}: the columns of its table of workings are not all {rows} rows long"));
        }
        for k in 0..first.len().saturating_sub(1) {
            let a = (first[k].max(0) as usize).min(rows);
            let b = (first[k + 1].max(0) as usize).min(rows).max(a);
            workings.push(
                (a..b)
                    .map(|i| Working {
                        cell: cell[i] as u32,
                        class: sector_class(sector[i] as usize),
                        opened: opened[i],
                        hands: hands[i],
                        out: out[i],
                        heap: heap[i],
                        surface: surface[i],
                    })
                    .collect(),
            );
        }
    }
    // The second island generator's land, all of it or none. A run whose header speaks of
    // the land and which has its arrays is read whole, and an array that is short is
    // refused by name. One whose header speaks of it and which has no heights is drawn as
    // the old island is, and says so: it must not be the invented island without a word.
    let land = match &header.land {
        Some(lh) if has("land_height") => {
            if lh.fine == 0 || lh.fw != w * lh.fine || lh.fh != h * lh.fine {
                return Err(format!("{path}: its land is {} by {} nodes at {} to a cell's side, which does not fit a map of {w} by {h} cells", lh.fw, lh.fh, lh.fine));
            }
            let sized = |name: &str, len: usize, want: usize, of: &str| -> Result<(), String> {
                if len == want { Ok(()) } else { Err(format!("{path}: its array {name} has {len} values, where {of} need {want}")) }
            };
            let fine = format!("the land's {} by {} nodes", lh.fw, lh.fh);
            let cells = format!("the map's {w} by {h} cells");
            let node_f32 = |name: &str| -> Result<Vec<f32>, String> {
                let v = f32s(name)?;
                sized(name, v.len(), lh.fw * lh.fh, &fine).map(|()| v)
            };
            let node_u8 = |name: &str| -> Result<Vec<u8>, String> {
                let v = u8s(name)?;
                sized(name, v.len(), lh.fw * lh.fh, &fine).map(|()| v)
            };
            let cell_flag = |name: &str| -> Result<Vec<bool>, String> {
                let v = u8s(name)?;
                sized(name, v.len(), w * h, &cells).map(|()| v.iter().map(|&b| b != 0).collect())
            };
            let named = |e: String| format!("{path}: {e}");
            let lines = crate::newland::lines_from(&crate::newland::LineArrays {
                start: i32s("river_start")?,
                xy: f32s("river_xy")?,
                width: f32s("river_width")?,
                level: f32s("river_level")?,
                flow: f32s("river_flow")?,
                end: i32s("river_end")?,
            })
            .map_err(named)?;
            let oxbows = crate::newland::oxbows_from(&i32s("oxbow_start")?, &f32s("oxbow_xy")?, &f32s("oxbow_width")?, &f32s("oxbow_level")?).map_err(named)?;
            Some(std::sync::Arc::new(crate::newland::NewLand {
                w,
                h,
                fine: lh.fine,
                fw: lh.fw,
                fh: lh.fh,
                height: node_f32("land_height")?,
                rock: node_u8("land_rock")?,
                lake: node_u8("land_lake")?,
                wet: node_u8("land_wet")?,
                flow: node_f32("land_flow")?,
                lines,
                oxbows,
                lakes: lh.lakes.clone(),
                drowned: lh.drowned,
                lake_cell: cell_flag("land_lakecell")?,
                shore: cell_flag("land_shore")?,
                estuary: cell_flag("land_estuary")?,
                middles: Default::default(),
            }))
        }
        Some(_) => {
            eprintln!("{}", no_land_arrays(path));
            None
        }
        None => None,
    };
    Ok(Replay {
        w,
        h,
        n: w * h,
        ny,
        y0: header.meta.start_year,
        n_towns,
        land,
        elev: u8s("elev")?,
        terrain: u8s("terrain")?,
        river: u8s("river")?,
        down: i32s("down")?,
        use_,
        inten: u8s("inten")?,
        trans: u8s("trans")?,
        catch: u8s("catch")?,
        rent,
        dist,
        mix_a,
        mix_b,
        mix_s,
        town_active: u8s("town_active")?,
        town_pop: f32s("town_pop")?,
        town_urban: f32s("town_urban")?,
        town_wage,
        town_price,
        firm_first: i32s("firm_first")?,
        firm_len: i32s("firm_len")?,
        firm_off: i32s("firm_off")?,
        firm_k: f32s("firm_Kvals")?,
        series: f32s("series")?,
        series_len,
        series_names,
        header,
        filled: ny,
        owner: Vec::new(),
        built,
        res,
        works,
        workings,
    })
}

/// The rent a cell's land earns in a month, from the record's byte: 0 is no
/// rent, and 1 to 255 run from 0.001 to 1000 on a log scale (`record::layers`).
pub fn rent_of(byte: u8) -> f32 {
    if byte == 0 { 0.0 } else { 10f32.powf(byte as f32 / 255.0 * 6.0 - 3.0) }
}

/// Capacity of firm `f` in year index `k` (0 when it does not exist that year).
pub fn firm_k(r: &Replay, f: usize, k: usize) -> f32 {
    // a works founded this year in a live game is in the family's books
    // before it is in the run: it does not exist yet, rather than stopping the game
    let (Some(&first), Some(&len), Some(&off)) = (r.firm_first.get(f), r.firm_len.get(f), r.firm_off.get(f)) else { return 0.0 };
    let k = k as i32;
    if k < first || k >= first + len {
        return 0.0;
    }
    r.firm_k.get((off + k - first) as usize).copied().unwrap_or(0.0)
}

/// The annual means of a named series for every year there is so far, for a
/// chart: NaN where a year has no figure, and empty if the run has no such series.
pub fn yearly(r: &Replay, name: &str) -> Vec<f32> {
    let Some(i) = r.series_names.iter().position(|s| s == name) else { return Vec::new() };
    let row = &r.series[i * r.series_len..(i + 1) * r.series_len];
    (0..r.filled.min(r.ny))
        .map(|k| {
            let months = &row[(k * 12).min(row.len())..((k + 1) * 12).min(row.len())];
            let (sum, n) = months.iter().filter(|v| v.is_finite()).fold((0.0f32, 0), |(s, n), v| (s + v, n + 1));
            if n > 0 { sum / n as f32 } else { f32::NAN }
        })
        .collect()
}

/// `n` months of a named series from month `first` of the run (0 is the first
/// January), for a chart: NaN where there is no figure (a month a game has not
/// reached), and empty if the run has no such series.
pub fn monthly(r: &Replay, name: &str, first: usize, n: usize) -> Vec<f32> {
    let Some(i) = r.series_names.iter().position(|s| s == name) else { return Vec::new() };
    let row = &r.series[i * r.series_len..(i + 1) * r.series_len];
    (first..first + n).map(|t| row.get(t).copied().unwrap_or(f32::NAN)).collect()
}

/// One month of a named series, if the run has it and has got that far.
pub fn month(r: &Replay, name: &str, t: usize) -> Option<f32> {
    let i = r.series_names.iter().position(|s| s == name)?;
    r.series.get(i * r.series_len + t).copied().filter(|v| t < r.series_len && v.is_finite())
}

/// The newest month no later than `t`, and no more than a year before it,
/// that the run has figures for; `t` itself if there is none. A live game's
/// months go into the run a moment after they come when a scenery build holds
/// it, so the month the clock has just passed may not be there yet, and a
/// panel that marked it would have nothing to show.
pub fn month_reached(r: &Replay, t: usize) -> usize {
    newest_filled(&r.series, r.series_len, t)
}

/// `month_reached` on the table itself: one row of `len` months to a series.
fn newest_filled(series: &[f32], len: usize, t: usize) -> usize {
    let has = |m: usize| m < len && series.chunks(len.max(1)).any(|row| row.get(m).is_some_and(|v| v.is_finite()));
    (t.saturating_sub(12)..=t).rev().find(|&m| has(m)).unwrap_or(t)
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

/// A very small run (two squares, one year, the town of Ashby founded in
/// 1700) that begins in `start_year`, for tests of what is said about a run.
#[cfg(test)]
pub(crate) fn tiny(start_year: i32) -> Replay {
    parse("tiny.bin", &tests::with_header(|v| v["meta"]["start_year"] = serde_json::json!(start_year))).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bytes of a very small run: two cells, one year, one town, one firm,
    /// one series, laid out as `record.pack` lays them out.
    fn tiny_run() -> Vec<u8> {
        let (n, ny, nt) = (2usize, 1usize, 1usize);
        let mut body: Vec<u8> = Vec::new();
        let mut arrays = Vec::new();
        let mut add = |name: &str, dtype: &str, shape: Vec<usize>, bytes: Vec<u8>| {
            arrays.push(serde_json::json!({"name": name, "dtype": dtype, "shape": shape, "offset": body.len(), "nbytes": bytes.len()}));
            body.extend_from_slice(&bytes);
            while body.len() % 8 != 0 {
                body.push(0);
            }
        };
        for name in ["elev", "terrain", "river"] {
            add(name, "uint8", vec![1, n], vec![1; n]);
        }
        add("down", "int32", vec![n], vec![0; n * 4]);
        for name in ["use", "inten", "trans", "catch", "mix_a", "mix_b", "mix_s"] {
            add(name, "uint8", vec![ny, n], vec![1; ny * n]);
        }
        add("town_active", "uint8", vec![ny, nt], vec![1; ny * nt]);
        add("town_pop", "float32", vec![ny, nt], 1234.0f32.to_le_bytes().to_vec());
        add("town_urban", "float32", vec![ny, nt], 400.0f32.to_le_bytes().to_vec());
        // one firm, there for the one year, its capacities starting at the first value
        for (name, v) in [("firm_first", 0i32), ("firm_len", 1), ("firm_off", 0)] {
            add(name, "int32", vec![1], v.to_le_bytes().to_vec());
        }
        add("firm_Kvals", "float32", vec![1], 2.5f32.to_le_bytes().to_vec());
        add("series", "float32", vec![1, 12], (0..12).flat_map(|m| (m as f32).to_le_bytes()).collect());
        let header = serde_json::json!({
            "meta": {"W": 2, "H": 1, "start_year": 1750, "n_years": ny, "seed": 7},
            "goods": [], "methods": [], "towns": [{"name": "Ashby", "node": 0, "founded": 1700}],
            "firms": [], "events": [], "uses": [],
            "series": {"names": ["pop"], "start_year": 1750},
            "arrays": arrays,
        });
        let mut h = serde_json::to_vec(&header).unwrap();
        while (4 + h.len()) % 8 != 0 {
            h.push(b' ');
        }
        let mut out = (h.len() as u32).to_le_bytes().to_vec();
        out.extend_from_slice(&h);
        out.extend_from_slice(&body);
        out
    }

    #[test]
    fn a_whole_run_is_read() {
        let r = parse("tiny.bin", &tiny_run()).unwrap();
        assert_eq!((r.w, r.h, r.ny, r.n_towns, r.filled), (2, 1, 1, 1, 1));
        assert_eq!(r.town_pop, [1234.0]);
        assert_eq!(r.series_names, ["pop"]);
        assert_eq!(annual(&r, "pop", 0), Some(5.5));
        assert_eq!(firm_k(&r, 0, 0), 2.5);
    }

    #[test]
    fn the_layers_of_kept_town_ground_are_read_where_the_run_has_them() {
        // a run under the port's rules has none of them
        let port = parse("tiny.bin", &tiny_run()).unwrap();
        assert!(!port.kept() && port.built.is_empty() && port.joined(0, 0).is_none());
        assert_eq!((port.residents(0, 0), port.works_share(0, 1)), (0.0, 0.0));
        // the same run with the three layers after its last array
        let whole = tiny_run();
        let hlen = u32::from_le_bytes([whole[0], whole[1], whole[2], whole[3]]) as usize;
        let mut v: serde_json::Value = serde_json::from_slice(&whole[4..4 + hlen]).unwrap();
        let mut body = whole[4 + hlen..].to_vec();
        for (name, bytes) in [("built", [55u8, 0]), ("res", [61, 0]), ("works", [0, 51])] {
            v["arrays"].as_array_mut().unwrap().push(serde_json::json!({"name": name, "dtype": "uint8", "shape": [1, 2], "offset": body.len(), "nbytes": 2}));
            body.extend_from_slice(&bytes);
            body.resize(body.len() + 6, 0);
        }
        let mut h = serde_json::to_vec(&v).unwrap();
        while (4 + h.len()) % 8 != 0 {
            h.push(b' ');
        }
        let mut file = (h.len() as u32).to_le_bytes().to_vec();
        file.extend_from_slice(&h);
        file.extend_from_slice(&body);
        let r = parse("kept.bin", &file).unwrap();
        assert!(r.kept());
        // the first cell joined its town in 1700 and has 488 residents; the second is open land a fifth under works
        assert_eq!((r.joined(0, 0), r.joined(0, 1)), (Some(1700), None));
        assert_eq!((r.residents(0, 0), r.residents(0, 1)), (488.0, 0.0));
        assert!((r.works_share(0, 1) - 0.2).abs() < 1e-6);
    }

    /// The tiny run as one made on the new land: the header's `land` object
    /// and, if `arrays`, the land's arrays after its last array, as the core
    /// writes them (`landcells::static_arrays`): two cells at four nodes to a
    /// cell's side, one river of three points and no oxbow. `edit` is given
    /// each array's name and its values' bytes before they are laid down.
    fn land_run(arrays: bool, edit: impl Fn(&str, &mut Vec<u8>)) -> Vec<u8> {
        let whole = tiny_run();
        let hlen = u32::from_le_bytes([whole[0], whole[1], whole[2], whole[3]]) as usize;
        let mut v: serde_json::Value = serde_json::from_slice(&whole[4..4 + hlen]).unwrap();
        let mut body = whole[4 + hlen..].to_vec();
        v["land"] = serde_json::json!({
            "fine": 4, "fw": 8, "fh": 4, "h_ref": 1500.0, "wind": [1.0, 0.0], "drowned": 5.0,
            "lakes": [{"level": 95.0, "area_km2": 12.0, "depth": 9.0, "x": 1.2, "y": 0.4, "outlet": [1.0, 0.5], "outflow": 2.5}],
            "outlets": [], "rock_names": [], "wet_names": [], "site_names": [],
        });
        let floats = |v: &[f32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        let ints = |v: &[i32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        let heights: Vec<f32> = (0..32).map(|n| n as f32 * 10.0 - 20.0).collect();
        let land: Vec<(&str, &str, usize, Vec<u8>)> = vec![
            ("land_height", "float32", 4, floats(&heights)),
            ("land_rock", "uint8", 1, vec![2; 32]),
            ("land_lake", "uint8", 1, (0..32).map(|n| (n == 9) as u8).collect()),
            ("land_wet", "uint8", 1, (0..32).map(|n| (n % 3) as u8).collect()),
            ("land_flow", "float32", 4, floats(&[0.5; 32])),
            ("river_start", "int32", 4, ints(&[0, 3])),
            ("river_xy", "float32", 4, floats(&[0.2, 0.2, 0.6, 0.4, 1.0, 0.5])),
            ("river_width", "float32", 4, floats(&[6.0, 8.0, 9.0])),
            ("river_level", "float32", 4, floats(&[120.0, 100.0, 95.0])),
            ("river_flow", "float32", 4, floats(&[0.3, 1.0, 1.5])),
            ("river_braided", "uint8", 1, vec![0; 3]),
            ("river_end", "int32", 4, ints(&[-2])),
            ("oxbow_start", "int32", 4, ints(&[0])),
            ("oxbow_xy", "float32", 4, Vec::new()),
            ("oxbow_width", "float32", 4, Vec::new()),
            ("oxbow_level", "float32", 4, Vec::new()),
            ("land_lakecell", "uint8", 1, vec![0, 1]),
            ("land_shore", "uint8", 1, vec![1, 0]),
            ("land_estuary", "uint8", 1, vec![0, 0]),
        ];
        if arrays {
            for (name, dtype, size, mut bytes) in land {
                edit(name, &mut bytes);
                v["arrays"].as_array_mut().unwrap().push(serde_json::json!({"name": name, "dtype": dtype, "shape": [bytes.len() / size], "offset": body.len(), "nbytes": bytes.len()}));
                body.extend_from_slice(&bytes);
                while body.len() % 8 != 0 {
                    body.push(0);
                }
            }
        }
        let mut h = serde_json::to_vec(&v).unwrap();
        while (4 + h.len()) % 8 != 0 {
            h.push(b' ');
        }
        let mut file = (h.len() as u32).to_le_bytes().to_vec();
        file.extend_from_slice(&h);
        file.extend_from_slice(&body);
        file
    }

    #[test]
    fn a_run_made_on_the_new_land_is_read_with_its_land_and_one_without_has_none() {
        // the old island: no word of the land in the header, and none in the run
        assert!(parse("tiny.bin", &tiny_run()).unwrap().land.is_none());
        let r = parse("land.bin", &land_run(true, |_, _| {})).unwrap();
        let land = r.land.as_deref().expect("the run's land");
        assert_eq!((land.w, land.h, land.fine, land.fw, land.fh), (2, 1, 4, 8, 4));
        assert_eq!((land.height.len(), land.height[0], land.height[31]), (32, -20.0, 290.0));
        assert_eq!((land.rock[5], land.lake[9], land.lake[10], land.wet[5], land.flow[7]), (2, 1, 0, 2, 0.5));
        assert_eq!(land.lines.len(), 1);
        assert_eq!((land.lines[0].pts.len(), land.lines[0].end, land.lines[0].width[2], land.lines[0].level[0], land.lines[0].flow[1]), (3, crate::newland::End::Lake(0), 9.0, 120.0, 1.0));
        assert!(land.oxbows.is_empty());
        assert_eq!((land.lakes.len(), land.lakes[0].level, land.lakes[0].outlet, land.drowned), (1, 95.0, (1.0, 0.5), 5.0));
        assert_eq!((land.lake_cell.clone(), land.shore.clone(), land.estuary.clone()), (vec![false, true], vec![true, false], vec![false, false]));
        // the run itself is read as ever
        assert_eq!((r.w, r.h, r.town_pop.clone()), (2, 1, vec![1234.0]));
        // a copy of the run shares the land and does not copy it
        let copy = r.clone();
        assert!(std::sync::Arc::ptr_eq(r.land.as_ref().unwrap(), copy.land.as_ref().unwrap()));
    }

    #[test]
    fn an_array_of_the_land_cut_short_is_refused_by_name() {
        // each array the viewer reads, a value short: the run is refused, and the array named
        let per = |name: &str| if matches!(name, "land_rock" | "land_lake" | "land_wet" | "land_lakecell" | "land_shore" | "land_estuary") { 1 } else { 4 };
        for name in [
            "land_height", "land_rock", "land_lake", "land_wet", "land_flow", "river_start", "river_xy", "river_width", "river_level", "river_flow", "river_end", "land_lakecell", "land_shore",
            "land_estuary",
        ] {
            let file = land_run(true, |n, bytes| {
                if n == name {
                    bytes.truncate(bytes.len() - per(name));
                }
            });
            let e = parse("short.bin", &file).err().unwrap_or_else(|| panic!("a run whose {name} is a value short was read"));
            assert!(e.starts_with("short.bin: its array ") && e.contains(name), "{name}: {e}");
        }
        // said in full, for the heights and for a cell's flags
        let short = |name: &'static str, by: usize| {
            parse("short.bin", &land_run(true, move |n, bytes| if n == name { bytes.truncate(bytes.len() - by) })).err().unwrap()
        };
        assert_eq!(short("land_height", 4), "short.bin: its array land_height has 31 values, where the land's 8 by 4 nodes need 32");
        assert_eq!(short("land_estuary", 1), "short.bin: its array land_estuary has 1 values, where the map's 2 by 1 cells need 2");
        // an array that is not there at all, and one of other numbers
        let e = parse("short.bin", &land_run(true, |n, bytes| if n == "land_wet" { bytes.clear() })).err().unwrap();
        assert_eq!(e, "short.bin: its array land_wet has 0 values, where the land's 8 by 4 nodes need 32");
        // the file itself cut off in the land's last array
        let whole = land_run(true, |_, _| {});
        // (the last array is two bytes and six of padding)
        let e = parse("cut.bin", &whole[..whole.len() - 7]).err().unwrap();
        assert!(e.contains("cut.bin is cut short: its array land_estuary should end"), "{e}");
        // a grid that does not fit the map
        let file = reheaded(&whole, |v| v["land"]["fw"] = serde_json::json!(9));
        assert_eq!(parse("odd.bin", &file).err().unwrap(), "odd.bin: its land is 9 by 4 nodes at 4 to a cell's side, which does not fit a map of 2 by 1 cells");
    }

    /// A run with its header changed by `edit`, in a header no longer than it
    /// was, so that the arrays stay where they are.
    fn reheaded(whole: &[u8], edit: impl Fn(&mut serde_json::Value)) -> Vec<u8> {
        let hlen = u32::from_le_bytes([whole[0], whole[1], whole[2], whole[3]]) as usize;
        let mut v: serde_json::Value = serde_json::from_slice(&whole[4..4 + hlen]).unwrap();
        edit(&mut v);
        let mut h = serde_json::to_vec(&v).unwrap();
        let grown = (h.len().max(hlen) - hlen).div_ceil(8) * 8;
        h.resize(hlen + grown, b' ');
        let mut file = (h.len() as u32).to_le_bytes().to_vec();
        file.extend_from_slice(&h);
        file.extend_from_slice(&whole[4 + hlen..]);
        file
    }

    #[test]
    fn the_lines_starts_and_ends_agree_or_the_run_is_refused() {
        let with = |name: &'static str, ints: &'static [i32]| parse("lines.bin", &land_run(true, move |n, bytes| if n == name { *bytes = ints.iter().flat_map(|x| x.to_le_bytes()).collect() })).err().unwrap();
        // the last start is the number of points; the first is nought; an end for every river
        assert_eq!(with("river_start", &[0, 2]), "lines.bin: its array river_start ends at point 2, and there are 3 points");
        assert_eq!(with("river_start", &[1, 3]), "lines.bin: its array river_start should begin at 0 and begins at 1");
        assert_eq!(with("river_start", &[0, 2, 3]), "lines.bin: its array river_end has 1 rivers, where river_start has 2");
        assert_eq!(with("river_end", &[-1, -1]), "lines.bin: its array river_end has 2 rivers, where river_start has 1");
        assert_eq!(with("river_end", &[4]), "lines.bin: its array river_end says a river joins river 4, and there are 1");
        assert_eq!(with("oxbow_start", &[0, 2]), "lines.bin: its array oxbow_start ends at point 2, and there are 0 points");
    }

    #[test]
    fn a_run_that_speaks_of_the_land_and_has_none_of_its_arrays_says_what_is_drawn() {
        // the header's word of the land with no array of it: read, without the land, and it says so
        let r = parse("bare.bin", &land_run(false, |_, _| {})).unwrap();
        assert!(r.header.land.is_some() && r.land.is_none());
        let said = no_land_arrays("bare.bin");
        assert!(said.starts_with("bare.bin was made on the new land") && said.contains("its land cannot be drawn") && said.contains("What is drawn in its place is the old kind of island"), "{said}");
        // a lite pack, which has no map layers at all, is refused as ever, and says the land is not in it either
        let file = reheaded(&land_run(false, |_, _| {}), |v| v["meta"]["lite"] = serde_json::json!(true));
        let e = parse("lite.bin", &file).err().unwrap();
        assert!(e.starts_with("lite.bin is a lite run (charts only) and has no map layers, and the land it was made on") && e.ends_with("record the run again without --lite"), "{e}");
        // and a lite pack of the old island says what it always said
        let old = with_header(|v| v["meta"]["lite"] = serde_json::json!(true));
        assert_eq!(parse("lite.bin", &old).err().unwrap(), "lite.bin is a lite run (charts only) and has no map layers");
    }

    #[test]
    fn the_month_marked_is_one_the_run_has() {
        // two series of 36 months; the game has played 26 of them, and one series has no figures at all
        let n = f32::NAN;
        let mut table = vec![n; 72];
        table[..26].fill(1.0);
        // the clock says month 25 is over and it is there
        assert_eq!(newest_filled(&table, 36, 25), 25);
        // months 26 and 27 have come from the simulation but wait to go into the run: the newest that is in
        assert_eq!(newest_filled(&table, 36, 26), 25);
        assert_eq!(newest_filled(&table, 36, 27), 25);
        // past the end of the run, and in a run with no figures, the month asked for
        assert_eq!(newest_filled(&table, 36, 40), 40);
        assert_eq!(newest_filled(&[n; 36], 36, 5), 5);
        assert_eq!(newest_filled(&[], 0, 5), 5);
    }

    #[test]
    fn a_works_not_yet_in_the_run_has_no_size() {
        // the family's books name a works the moment it is founded; the run has it from the next year
        let r = parse("tiny.bin", &tiny_run()).unwrap();
        assert_eq!(firm_k(&r, 1, 0), 0.0);
        assert_eq!(firm_k(&r, 0, 7), 0.0);
    }

    #[test]
    fn a_run_cut_short_is_an_error_that_says_so() {
        let whole = tiny_run();
        // cut anywhere, the reader gives an error and never panics
        for len in 0..whole.len() {
            let e = parse("cut.bin", &whole[..len]).err().unwrap_or_else(|| panic!("{len} of {} bytes were read as a whole run", whole.len()));
            if len < 8 {
                assert!(e.contains("too short"), "{len} bytes: {e}");
            } else {
                assert!(e.contains("cut.bin is cut short"), "{len} bytes: {e}");
            }
        }
        // in the header, and in an array
        let hlen = u32::from_le_bytes([whole[0], whole[1], whole[2], whole[3]]) as usize;
        let e = parse("cut.bin", &whole[..4 + hlen / 2]).err().unwrap();
        assert!(e.contains("its header should be"), "{e}");
        let e = parse("cut.bin", &whole[..whole.len() - 1]).err().unwrap();
        assert!(e.contains("its array series should end"), "{e}");
    }

    #[test]
    fn a_table_with_one_dimension_is_an_error() {
        let whole = tiny_run();
        let hlen = u32::from_le_bytes([whole[0], whole[1], whole[2], whole[3]]) as usize;
        // the same file with town_pop described as a plain list
        let mut v: serde_json::Value = serde_json::from_slice(&whole[4..4 + hlen]).unwrap();
        for a in v["arrays"].as_array_mut().unwrap() {
            if a["name"] == "town_pop" {
                a["shape"] = serde_json::json!([1]);
            }
        }
        let mut h = serde_json::to_vec(&v).unwrap();
        // the header may only grow by whole steps of eight bytes, to keep the arrays where they are
        assert!(h.len() <= hlen);
        h.resize(hlen, b' ');
        let mut file = whole[..4].to_vec();
        file.extend_from_slice(&h);
        file.extend_from_slice(&whole[4 + hlen..]);
        let e = parse("odd.bin", &file).err().unwrap();
        assert!(e.contains("town_pop should be a table"), "{e}");
    }

    /// The tiny run with its header changed by `edit`, the arrays where they were.
    pub(super) fn with_header(edit: impl Fn(&mut serde_json::Value)) -> Vec<u8> {
        let whole = tiny_run();
        let hlen = u32::from_le_bytes([whole[0], whole[1], whole[2], whole[3]]) as usize;
        let mut v: serde_json::Value = serde_json::from_slice(&whole[4..4 + hlen]).unwrap();
        edit(&mut v);
        let mut h = serde_json::to_vec(&v).unwrap();
        // the arrays' offsets count from the header's end, which may only move by whole steps of eight bytes
        let grown = (h.len().max(hlen) - hlen).div_ceil(8) * 8;
        h.resize(hlen + grown, b' ');
        let mut file = (h.len() as u32).to_le_bytes().to_vec();
        file.extend_from_slice(&h);
        file.extend_from_slice(&whole[4 + hlen..]);
        file
    }

    #[test]
    fn an_array_of_other_numbers_than_the_viewer_reads_is_an_error() {
        // the town's people as doubles: read as floats by name alone they would be nonsense
        let file = with_header(|v| {
            for a in v["arrays"].as_array_mut().unwrap() {
                if a["name"] == "town_pop" {
                    a["dtype"] = serde_json::json!("float64");
                }
            }
        });
        let e = parse("doubles.bin", &file).err().unwrap();
        assert_eq!(e, "doubles.bin: its array town_pop holds float64 numbers, and the viewer reads it as float32");
        // and the tiny run as written, in the recorders' words, is read
        assert!(parse("tiny.bin", &tiny_run()).is_ok());
    }

    #[test]
    fn a_run_that_numbers_its_land_otherwise_is_an_error() {
        let ours = |names: &[&str]| serde_json::json!(names);
        // the lists as the core writes them are read, and so is a run that gives none (the tiny run here)
        let same = with_header(|v| {
            v["disp"] = ours(&isostate_core::record::DISP);
            v["uses"] = ours(&isostate_core::land::USE_NAMES);
        });
        assert!(parse("same.bin", &same).is_ok());
        // two classes the other way round
        let mut swapped = isostate_core::record::DISP;
        swapped.swap(1, 2);
        let file = with_header(|v| v["disp"] = ours(&swapped));
        let e = parse("swapped.bin", &file).err().unwrap();
        assert!(e.starts_with("swapped.bin: its display classes are sea, pasture, farm, wood,"), "{e}");
        assert!(e.ends_with(", where the viewer draws sea, farm, pasture, wood, heath, quarry, claypit, ironmine, coalmine, industry, urban, derelict, in that order"), "{e}");
        // a land use more
        let file = with_header(|v| {
            let mut uses: Vec<&str> = isostate_core::land::USE_NAMES.to_vec();
            uses.push("canal");
            v["uses"] = ours(&uses);
        });
        assert!(parse("more.bin", &file).err().unwrap().starts_with("more.bin: its land uses are sea, wild,"));
        // the viewer's own numbers for the classes, in the order of the core's names
        use crate::world::{CLAYPIT, COALMINE, DERELICT, FARM, HEATH, IRONMINE, PASTURE, QUARRY, SEA, URBAN, WOOD, WORKS};
        let numbers = [SEA, FARM, PASTURE, WOOD, HEATH, QUARRY, CLAYPIT, IRONMINE, COALMINE, WORKS, URBAN, DERELICT];
        assert_eq!(numbers.len(), isostate_core::record::DISP.len());
        for (i, n) in numbers.into_iter().enumerate() {
            assert_eq!(n as usize, i, "{}", isostate_core::record::DISP[i]);
        }
    }
}
