//! Reader for the run.bin replay container written by `record.pack`.
//!
//! Layout: a little-endian u32 header length (including padding), a JSON
//! header, then typed arrays aligned to 8 bytes. Map layers are stored
//! layer-major, `[year, cell]`.

use isostate_core::squares::{Change, PER_CELL, Tenant};
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
    /// How the towns' layers and the rent are packed (the core's
    /// `record::TownBytes`): 2 for a run past 1900, and 0 or 1, the first
    /// layout, for a run to 1900 and every record made before there were two.
    #[serde(default)]
    pub town_bytes: u8,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Good {
    pub name: String,
    pub label: String,
    /// what kind of good it is, where the content at hand says (a live game's
    /// does, `live::good_kinds`; a recorded run's header does not)
    #[serde(default)]
    pub kind: Option<GoodKind>,
    /// Its bulk in carting and its price at the start, as the run's own
    /// content has them: by these a works' site planner tells how the good
    /// is kept, in heaps, in stacks or under a roof (`scatter::process_of`).
    /// A record that does not say leaves them out. (The Python's records and
    /// the core's have always written the weight; the core's write the
    /// price from 12 November 2026.)
    #[serde(default)]
    pub weight: Option<f32>,
    #[serde(default)]
    pub price: Option<f32>,
    /// When it begins, for a good the run does not have from the start
    /// (docs/rules/goods.md); None for the others, and in every record made
    /// before 26 November 2026. A good is listed from then (`begun`).
    #[serde(default)]
    pub begins: Option<Begins>,
}

/// When a later good begins: a year, or the name of the idea it comes with.
#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Begins {
    Year(i32),
    Idea(String),
}

impl Begins {
    /// As a live game's content says it.
    pub fn of(c: &isostate_core::content::Content, g: usize) -> Option<Begins> {
        match c.good_begins.get(g).copied().unwrap_or_default() {
            isostate_core::content::Begins::Start => None,
            isostate_core::content::Begins::Year(y) => Some(Begins::Year(y as i32)),
            isostate_core::content::Begins::Idea(t) => Some(Begins::Idea(c.tech_name[t].clone())),
        }
    }
}

/// The three kinds the Markets panel lists the goods under, so that a list
/// of twenty-five reads as three short ones.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoodKind {
    /// grown, felled or dug: the good of a land sector
    Land,
    /// made in a works, for other works and for building
    Works,
    /// made in a works, for the household
    Household,
}

impl GoodKind {
    pub const ALL: [GoodKind; 3] = [GoodKind::Land, GoodKind::Works, GoodKind::Household];

    /// The heading over its goods.
    pub fn heading(self) -> &'static str {
        match self {
            GoodKind::Land => "From the land",
            GoodKind::Works => "For the works",
            GoodKind::Household => "For the household",
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Method {
    pub name: String,
    pub label: String,
    pub out: usize,
    #[serde(rename = "K0")]
    pub k0: f32,
    /// What the method does, as the run's own content has it: its goods in
    /// (each good's place among the goods, and how much goes to a unit of
    /// output), the heat it burns and its labour to a unit of output, and
    /// what a unit of monthly capacity costs to build at the goods' starting
    /// prices. A works is laid out from these (`scatter::process_of`), so a
    /// trade the art has never heard of still gets a plan of its own. A
    /// record that does not say leaves them out. (Both recorders have always
    /// written the first three; the core's writes the last from 12 November 2026.)
    #[serde(default)]
    pub inputs: Option<Vec<(usize, f32)>>,
    #[serde(default)]
    pub heat: Option<f32>,
    #[serde(default)]
    pub labor: Option<f32>,
    #[serde(default)]
    pub capital: Option<f32>,
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
    /// crossings (the core's docs/rules/crossings.md): every link between two
    /// neighbouring cells whose way crosses water, as the run ended (a live
    /// game's as its last year ended), lowest cell first. Empty for a run that
    /// did not keep them, whose roads are bridged wherever they cross a river.
    #[serde(default)]
    pub crossings: Vec<CrossLink>,
    /// the content's techs, by name, in the order of `town_know`'s last
    /// dimension (a run that lists none has its towns' knowledge unread)
    #[serde(default)]
    pub techs: Vec<TechName>,
}

/// A tech as the header lists it: its name is all the viewer reads.
#[derive(Deserialize, Clone, Debug)]
pub struct TechName {
    pub name: String,
}

/// A link between two neighbouring cells whose way crosses water, and how it
/// is crossed (the core's `crossings::LinkOut`).
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct CrossLink {
    pub a: usize,
    pub b: usize,
    /// the year its bridge was built, if it has one
    #[serde(default)]
    pub bridged: Option<i64>,
    /// the river is too wide to ford, and it is crossed by a ferry
    #[serde(default)]
    pub ferry: bool,
}

/// How a road between two cells crosses its water in a year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Crossed {
    /// on a bridge: the run built one, or did not keep its crossings (then
    /// every road is bridged wherever it meets a river, as before)
    Bridge,
    /// through a ford, or by a ferry where the river is too wide to ford
    Ford,
    Ferry,
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
    /// The year the cell joined its town less `BUILT_YEAR0`, in steps of
    /// `built_step` years, 1 to 255; 0 where the cell is not town ground.
    pub built: Vec<u8>,
    /// the cell's residents, in units of `res_unit` people
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
    /// In a run past 1900, what each year's rent byte was taken over (the
    /// core's longrun anchor, docs/rules/longrun.md); empty for a run to 1900,
    /// whose rent bytes are in 1750's money.
    pub rent_scale: Vec<f32>,
    /// What the ground holds and what each town makes, for the map views of
    /// where a good is made and of the deposits (`overlay.rs`).
    pub holds: Holds,
    /// What a town's lots are built by after 1900 (`scatter/towns/forms.rs`):
    /// how densely each square is built, its land value, and what each town
    /// knows. Empty where the run does not keep them.
    pub builds: Builds,
    /// What each way carried and how, and each year's trade between pairs
    /// of towns (`traffic.rs`); empty where the run does not keep them.
    pub traffic: crate::traffic::Traffic,
}

/// `ISOSTATE_LOOK=plain`: draw a town after 1900 as it was drawn before the
/// look of a filled square (21 November 2026), for setting before beside after
/// in one program (`scatter/towns/kept/lots.rs`, `plain`).
pub fn look_plain() -> bool {
    static PLAIN: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *PLAIN.get_or_init(|| std::env::var("ISOSTATE_LOOK").as_deref() == Ok("plain"))
}

/// The share of its ground a square's buildings stand on from which it is a
/// filled square (`Builds::filled`), where no park says so: four fifths. The
/// core's notes put an edge square, a young one still filling in, and one of a
/// town on cheap ground at a fill under about 200 of 250; a square of the old
/// ground stands at 230 to 250 by 1950.
pub const FILLED: f32 = 0.8;

/// The people a square built whole holds at the ceiling of 1900, which the
/// core's `fill` is counted against (`isostate_core::towns::ceiling`): 1,430,
/// which the viewer draws as 119 houses with their yards and streets over the
/// whole of the cell (`world::PEOPLE_TO_A_HOUSE`).
pub const WHOLE_SQUARE: f32 = 1430.0;

/// The last year a town is drawn by its houses' own rules whatever the
/// record says of its fill: the forms' year (`scatter/towns/kept/lots.rs`, `FROM`).
pub const FILL_FROM: i32 = 1900;

/// The land value (the record's `value`, a square's rent a unit of room over
/// its town's of 1900) from which a square is drawn by its fill (`fill_of`):
/// its town's of 1900. A town whose ground never grew dear stands below it.
pub const FILLED_VALUE: f32 = 1.0;

/// How densely each square of a town is built and what its land is worth,
/// year by year, and what each town knew of what builders need: what the
/// lots of a town past 1900 are built and rebuilt by (the core's
/// `record::GROUND_LAYERS`, and its `town_know`). Each part is empty where the
/// run does not have it: a run to 1900, a run under the port's rules, or one
/// recorded before these were.
#[derive(Clone, Default, Debug)]
pub struct Builds {
    /// `[year, cell]`: 0 off town ground, else 1 plus hundredths of `dens` above 1
    pub dens: Vec<u8>,
    /// `[year, cell]`: 0 where unknown, else 1 plus twentieths of a doubling of
    /// the land value above a sixteenth
    pub value: Vec<u8>,
    /// `[year, cell]`: the share of the square its buildings stand on, in
    /// 250ths (the core's `fill`; 0 off town ground), and the share kept as a
    /// park (`park`). Empty in a record made before 21 November 2026.
    pub fill: Vec<u8>,
    pub park: Vec<u8>,
    /// `[year, town, tech]`: what each town knows of each, 0 to 255
    pub know: Vec<u8>,
    /// towns and techs to a year of `know`
    pub know_towns: usize,
    pub techs: usize,
    /// the places among the techs of the steel frame and of electricity
    /// (`coal_power_station`), where the content has them
    pub steel: Option<usize>,
    pub power: Option<usize>,
    /// Where in each cell each built thing stands, square by square (the
    /// core's docs/rules/squares.md): empty in a run that keeps no squares.
    /// Nothing draws them yet.
    pub squares: Squares,
}

/// One square of 125 m as the simulation keeps it: its tenant, the year the
/// tenant took it, and which side its way lies on (the core's `Table::face`,
/// in 256ths of a turn, 0 toward the map's east and 64 toward its south).
/// A square nothing holds is `Tenant::None`, its year and face 0.
///
/// Nothing draws the squares yet: they are read here for the stages that
/// will (works on their squares, the towns after 1900), and the tests read
/// them, so what only those read is allowed to stand unused for now.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Square {
    pub tenant: Tenant,
    pub year: i16,
    pub face: u8,
}

#[allow(dead_code)]
impl Square {
    pub const NONE: Square = Square { tenant: Tenant::None, year: 0, face: 0 };
}

/// One cell's squares, row by row from its north-west corner: square
/// `k = qy * 16 + qx` (the core's `squares::Table`).
#[allow(dead_code)]
pub type CellSquares = [Square; PER_CELL];

/// The squares' log, year by year (the core's docs/rules/squares.md, "In the
/// record"): each year's changes, in order of the square (`Change::at`),
/// read from a record's `sq_log` or handed over by a live game a year at a
/// time. The squares of a year are those of the year before with its changes
/// made; a year whose list is the whole book (a live game that missed a year)
/// stands alone. A record's years are kept in its bytes and read the first
/// time they are asked for, so that opening a record does not wait on them;
/// each year is shared, so copying a run copies pointers.
#[derive(Clone, Default, Debug)]
pub struct Squares {
    /// By year; None for a year not yet filled.
    years: Vec<Option<SquareYear>>,
}

/// One year of the squares' log.
#[derive(Clone, Debug)]
struct SquareYear {
    /// Its list is the whole book.
    whole: bool,
    /// The record's bytes of the year (`squares::encode_year`): the whole log,
    /// where the year begins and ends in it, and the year; None for a year a
    /// live game handed over, whose list is made already.
    bytes: Option<(std::sync::Arc<[u8]>, usize, usize, i64)>,
    list: std::sync::Arc<std::sync::OnceLock<std::sync::Arc<[Change]>>>,
}

impl SquareYear {
    /// Its changes by `Change::at`, read from the record's bytes the first
    /// time. Bytes that do not read (a file damaged inside the log, which
    /// opening checks only for its bounds) are said so, and read as no change.
    fn list(&self) -> &[Change] {
        self.list.get_or_init(|| match &self.bytes {
            Some((log, a, b, year)) => isostate_core::squares::decode_year(&log[*a..*b], *year)
                .unwrap_or_else(|e| {
                    eprintln!("the squares of {year} do not read ({e}), and are left out");
                    Vec::new()
                })
                .into(),
            None => Vec::new().into(),
        })
    }
}

#[allow(dead_code)]
impl Squares {
    /// Whether the run keeps its squares.
    pub fn kept(&self) -> bool {
        self.years.iter().any(Option::is_some)
    }

    /// The changes of year `k` (None for a year not filled), and whether they are the whole book.
    pub fn changes(&self, k: usize) -> Option<(bool, &[Change])> {
        self.years.get(k)?.as_ref().map(|y| (y.whole, y.list()))
    }

    /// Forget every year from `kept` on, as a take back does (`back::forget`):
    /// the log is then what a game that had played only `kept` years holds.
    pub fn forget(&mut self, kept: usize) {
        for y in self.years.iter_mut().skip(kept) {
            *y = None;
        }
        if self.years.iter().all(Option::is_none) {
            self.years.clear();
        }
    }

    /// The years of a record's log: bytes `first[k] .. first[k + 1]` of `log`
    /// are year k's, read when they are first asked for.
    fn of_record(log: std::sync::Arc<[u8]>, first: &[usize], y0: i64) -> Squares {
        let years = (0..first.len().saturating_sub(1))
            .map(|k| Some(SquareYear { whole: false, bytes: Some((log.clone(), first[k], first[k + 1], y0 + k as i64)), list: Default::default() }))
            .collect();
        Squares { years }
    }

    /// Write year `k` of `ny` as a game hands it over (`LiveYear::squares`).
    pub fn take_year(&mut self, ny: usize, k: usize, whole: bool, changes: &[Change]) {
        if k >= ny {
            return;
        }
        if self.years.len() < ny {
            self.years.resize(ny, None);
        }
        let mut list = changes.to_vec();
        list.sort_unstable_by_key(|c| c.at);
        let made = std::sync::OnceLock::new();
        let _ = made.set(list.into());
        self.years[k] = Some(SquareYear { whole, bytes: None, list: std::sync::Arc::new(made) });
    }

    /// The years whose changes make year `k`: from the last that is the whole book.
    fn span(&self, k: usize) -> std::ops::RangeInclusive<usize> {
        let k = k.min(self.years.len().saturating_sub(1));
        let from = (0..=k).rev().find(|&j| self.years[j].as_ref().is_some_and(|y| y.whole)).unwrap_or(0);
        from..=k
    }

    /// The squares of cell `c` as year `k` ends, or None where nothing is
    /// built on it then (or the run keeps no squares). It reads the log and
    /// nothing else, so it may be asked anywhere, off the frame.
    pub fn cell(&self, k: usize, c: usize) -> Option<Box<CellSquares>> {
        if self.years.is_empty() {
            return None;
        }
        let mut sq = Box::new([Square::NONE; PER_CELL]);
        let (lo, hi) = ((c * PER_CELL) as u32, ((c + 1) * PER_CELL) as u32);
        for j in self.span(k) {
            let Some(y) = &self.years[j] else { continue };
            let list = y.list();
            let a = list.partition_point(|x| x.at < lo);
            let b = a + list[a..].partition_point(|x| x.at < hi);
            for x in &list[a..b] {
                sq[x.square()] = square_of(x);
            }
        }
        sq.iter().any(|s| s.tenant != Tenant::None).then_some(sq)
    }

    /// Every cell with something built on it as year `k` ends, and its
    /// squares: the whole log to year `k` read once.
    pub fn year(&self, k: usize) -> std::collections::BTreeMap<usize, Box<CellSquares>> {
        let mut out: std::collections::BTreeMap<usize, Box<CellSquares>> = std::collections::BTreeMap::new();
        if self.years.is_empty() {
            return out;
        }
        for j in self.span(k) {
            let Some(y) = &self.years[j] else { continue };
            for x in y.list() {
                out.entry(x.cell()).or_insert_with(|| Box::new([Square::NONE; PER_CELL]))[x.square()] = square_of(x);
            }
        }
        out.retain(|_, sq| sq.iter().any(|s| s.tenant != Tenant::None));
        out
    }
}

#[allow(dead_code)]
fn square_of(x: &Change) -> Square {
    if x.tenant == 0 { Square::NONE } else { Square { tenant: Tenant::of(x.tenant), year: x.year, face: x.face } }
}

/// What is said of a run that keeps no squares.
fn no_squares(path: &str) -> String {
    format!(
        "{path} keeps no squares of 125 m (the simulation has kept them since 26 November 2026, under the game's rules only): where each works and each town's houses stand in their cells is left to the viewer, as before"
    )
}

impl Builds {
    /// Whether the run says how densely its squares are built.
    pub fn kept(&self) -> bool {
        !self.dens.is_empty() && !self.value.is_empty()
    }

    /// How densely cell `c` is built in year `k` (1 where the run does not say).
    pub fn dens(&self, n: usize, k: usize, c: usize) -> f32 {
        self.dens.get(k * n + c).filter(|&&b| b > 0).map_or(1.0, |&b| 1.0 + (b - 1) as f32 / 100.0)
    }

    /// The land value of cell `c` in year `k`, if the run knows it.
    pub fn value(&self, n: usize, k: usize, c: usize) -> Option<f32> {
        self.value.get(k * n + c).filter(|&&b| b > 0).map(|&b| ((b - 1) as f32 / 20.0 - 4.0).exp2())
    }

    /// The share of cell `c`'s ground its buildings stand on in year `k`, and
    /// the share kept as a park, 0 to 1 each; None where the run does not say
    /// (a year to 1900, a record made before these layers, or not town ground).
    pub fn fill(&self, n: usize, k: usize, c: usize) -> Option<(f32, f32)> {
        if look_plain() {
            return None;
        }
        let f = *self.fill.get(k * n + c).filter(|&&b| b > 0)?;
        Some((f as f32 / 250.0, self.park.get(k * n + c).map_or(0.0, |&p| p as f32 / 250.0)))
    }

    /// Whether cell `c` is a filled square in year `k`: one the town built
    /// whole when it took it (a park kept), or one whose buildings stand on
    /// `FILLED` of its ground or more. The rest of a filled square is yards,
    /// gardens and courts, and its park; never a paddock or a field.
    pub fn filled(&self, n: usize, k: usize, c: usize) -> bool {
        self.fill(n, k, c).is_some_and(|(f, p)| p > 0.0 || f >= FILLED)
    }

    /// Whether town `t` knew the tech at `tech` (half or more of it) in year `k`;
    /// None where the run does not say.
    pub fn knows(&self, k: usize, t: usize, tech: Option<usize>) -> Option<bool> {
        let j = tech?;
        (t < self.know_towns && j < self.techs).then(|| self.know.get((k * self.know_towns + t) * self.techs + j).is_some_and(|&b| b >= 128))
    }

    /// The places of the two techs, by the content's names in order.
    pub fn name_techs(&mut self, names: &[String]) {
        self.techs = names.len();
        self.steel = names.iter().position(|n| n == "steel_frame");
        self.power = names.iter().position(|n| n == "coal_power_station");
    }
}

/// What the ground holds and what each town makes, as far as the run keeps
/// them: the deposits and the soil (once for the run), what each town made of
/// each good in each year (the books' `made`, under the game's rules), and in a
/// game the standing timber and how much of each worked deposit is left.
/// Anything the run does not keep is empty, and the views say so.
#[derive(Clone, Default, Debug)]
pub struct Holds {
    /// Each land sector's name and the good it makes, in the core's order.
    pub sectors: Vec<(String, usize)>,
    /// The fields of the ground by name (`isostate_core::record::STATIC_FIELDS`:
    /// fertility, ore, coal, rock, clay, pasture, forest_pot), 0 to 255 a cell.
    pub fields: Vec<(String, Vec<u8>)>,
    /// What each town made of each good in a year, `[year, town, good]`, with
    /// `made_towns` towns to a year; empty for a run that kept no books.
    pub made: Vec<f32>,
    pub made_towns: usize,
    /// The timber standing on each cell as each year ends, 0 to 255 on the
    /// fields' scale, `[year, cell]`; empty where it is not known (a recorded run).
    pub cover: Vec<u8>,
    /// For each year, the cells whose deposit has been worked: the cell, the
    /// kind (`DEPOSIT_FIELDS`) and the share of its first richness the hands
    /// still meet there, 0 to 255. Empty where it is not known (a recorded run).
    pub left: Vec<Vec<(u32, u8, u8)>>,
    /// GROUND-BRUSH: for each year, the cells whose deposit the hand changed
    /// (ground.rs): the cell, the kind (`DEPOSIT_FIELDS`) and what it holds
    /// as the year ends, 0 to 255, in place of the field the island was made with.
    pub now: Vec<Vec<(u32, u8, u8)>>,
}

/// The fields that are deposits, in the order `Holds::left` numbers them.
pub const DEPOSIT_FIELDS: [&str; 4] = ["coal", "ore", "clay", "rock"];

impl Holds {
    /// A field of the ground by name, if the run has it.
    pub fn field(&self, name: &str) -> Option<&[u8]> {
        self.fields.iter().find(|f| f.0 == name).map(|f| f.1.as_slice())
    }

    /// What town `t` made of good `g` in year `k`, if the run kept it.
    pub fn made(&self, k: usize, t: usize, g: usize, ng: usize) -> Option<f32> {
        if t >= self.made_towns {
            return None;
        }
        self.made.get((k * self.made_towns + t) * ng + g).copied().filter(|v| v.is_finite())
    }

    /// Write the year a game has just played: what its towns made, its standing
    /// timber and what is left of its deposits (`live::holds_year`).
    pub fn take_year(&mut self, k: usize, ny: usize, n: usize, ng: usize, y: &HoldsYear) {
        if k >= ny {
            return;
        }
        if !y.made.is_empty() && ng > 0 {
            let nt = y.made.len() / ng;
            if self.made_towns != nt {
                // the towns' slots are fixed for a game, so this happens once
                self.made = vec![0.0; ny * nt * ng];
                self.made_towns = nt;
            }
            self.made[k * nt * ng..(k + 1) * nt * ng].copy_from_slice(&y.made);
        }
        if y.cover.len() == n {
            if self.cover.len() != ny * n {
                self.cover = vec![0; ny * n];
            }
            self.cover[k * n..(k + 1) * n].copy_from_slice(&y.cover);
        }
        if self.left.len() < ny {
            self.left.resize(ny, Vec::new());
        }
        self.left[k] = y.left.clone();
        // (kept only once the hand has touched the ground, as the workings are)
        if !y.now.is_empty() || !self.now.is_empty() {
            if self.now.len() < ny {
                self.now.resize(ny, Vec::new());
            }
            self.now[k] = y.now.clone();
        }
    }
}

/// One year of `Holds` as a game hands it over.
#[derive(Clone, Default, Debug)]
pub struct HoldsYear {
    /// `[town, good]`
    pub made: Vec<f32>,
    /// `[cell]`
    pub cover: Vec<u8>,
    pub left: Vec<(u32, u8, u8)>,
    /// GROUND-BRUSH: the cells whose deposit the hand changed, and what they hold
    pub now: Vec<(u32, u8, u8)>,
}

/// The cell field a land sector works, by the sector's name: the farms the
/// soil, the woods the trees, each digging its deposit.
pub fn sector_field(sector: &str) -> Option<&'static str> {
    Some(match sector {
        "farm" => "fertility",
        "pasture" => "pasture",
        "forestry" => "forest_pot",
        "quarry" => "rock",
        "claypit" => "clay",
        "ironmine" => "ore",
        "coalmine" => "coal",
        _ => return None,
    })
}

/// The display class of a land sector, by its name (`isostate_core::record::DISP`).
pub fn sector_class_named(sector: &str) -> Option<u8> {
    Some(match sector {
        "farm" => 1,
        "pasture" => 2,
        "forestry" => 3,
        "quarry" => 5,
        "claypit" => 6,
        "ironmine" => 7,
        "coalmine" => 8,
        _ => return None,
    })
}

/// `Replay::built` counts years from here (`isostate_core::record::BUILT_YEAR0`).
pub const BUILT_YEAR0: i32 = 1645;
/// People to one step of `Replay::res` in a run to 1900 (`isostate_core::record::RES_UNIT`);
/// twice as many in one past it (`Replay::res_unit`).
pub const RES_UNIT: f32 = 8.0;

impl Replay {
    /// Whether the run keeps its crossings (the core's docs/rules/crossings.md):
    /// then a road goes round the meanders its straight line cuts, and crosses
    /// a river once, where the run says (`network::route`).
    pub fn round(&self) -> bool {
        !self.header.crossings.is_empty()
    }

    /// How the road between cells `c` and `m` crosses water in year `k`: on
    /// a bridge where the run built one, or where it kept no crossings at all.
    pub fn crossed(&self, c: usize, m: usize, k: usize) -> Crossed {
        let list = &self.header.crossings;
        if list.is_empty() {
            return Crossed::Bridge;
        }
        let (a, b) = (c.min(m), c.max(m));
        let year = self.y0 as i64 + k as i64;
        let from = list.partition_point(|l| l.a < a);
        match list[from..].iter().take_while(|l| l.a == a).find(|l| l.b == b) {
            Some(l) if l.bridged.is_some_and(|y| y <= year) => Crossed::Bridge,
            Some(l) if l.ferry => Crossed::Ferry,
            Some(_) => Crossed::Ford,
            // (a way the core found no water on; whatever the picture finds there is bridged)
            None => Crossed::Bridge,
        }
    }

    /// Whether the run's towns keep their ground, and the record says when
    /// each cell joined its town, who lives on it and how much of it is works.
    pub fn kept(&self) -> bool {
        !self.built.is_empty() && !self.res.is_empty()
    }

    /// Whether the record packs its towns as a run past 1900 does: `built` in
    /// steps of two years and residents in sixteens (the core's `record::TownBytes`).
    fn second_layout(&self) -> bool {
        self.header.meta.town_bytes >= 2
    }

    /// The year a `built` byte stands for (the first year of its step); a byte
    /// of 0, not town ground, is read as the year before the first.
    pub fn built_year(&self, b: u8) -> i32 {
        BUILT_YEAR0 + if self.second_layout() { 2 } else { 1 } * b as i32
    }

    /// People to one step of `res`.
    pub fn res_unit(&self) -> f32 {
        if self.second_layout() { 2.0 * RES_UNIT } else { RES_UNIT }
    }

    /// The year cell `c` joined its town, if it is town ground in year `k` of a run that keeps it.
    pub fn joined(&self, k: usize, c: usize) -> Option<i32> {
        self.built.get(k * self.n + c).filter(|&&b| b > 0).map(|&b| self.built_year(b))
    }

    /// The residents of cell `c` in year `k` (nothing in a run that does not record them).
    pub fn residents(&self, k: usize, c: usize) -> f32 {
        self.res.get(k * self.n + c).map_or(0.0, |&b| b as f32 * self.res_unit())
    }

    /// How much of cell `c` is built in year `k`, and how much is park
    /// (`Builds::fill`): only after 1900, where the forms build a town. The
    /// record keeps the layer from 1750, but a town to 1900 is drawn by its
    /// houses' own rules, from its residents, and those houses never change.
    ///
    /// And only where the square's ground grew dear: a park kept (the town
    /// built it whole because its rent outbid the plough's), or land worth
    /// its town's of 1900 or more. On cheap ground a town's houses follow its
    /// people as they always did: there the core's room by age stands far
    /// above them where a town has shrunk (island 7's Grancombe, one square
    /// of 112 people whose fill is 230 of 250), and the picture of a few
    /// cottages left in their fields is the truer one (`FILLED_VALUE`).
    pub fn fill_of(&self, k: usize, c: usize) -> Option<(f32, f32)> {
        if self.y0 + k as i32 <= FILL_FROM {
            return None;
        }
        self.builds.fill(self.n, k, c).filter(|&(_, park)| park > 0.0 || self.builds.value(self.n, k, c).is_none_or(|v| v >= FILLED_VALUE))
    }

    /// Whether cell `c` is a filled square in year `k` (`Builds::filled`),
    /// after 1900, on ground that grew dear (`fill_of`).
    pub fn filled(&self, k: usize, c: usize) -> bool {
        self.fill_of(k, c).is_some() && self.builds.filled(self.n, k, c)
    }

    /// The ground cell `c`'s houses take in year `k`, in people as a square of
    /// 1900 holds them. Where the record says how much of the square is built
    /// (`Builds::fill`, since 21 November 2026), that share of a whole
    /// square's people (`WHOLE_SQUARE`): the core counts it by room, which is
    /// what stands, and not by the heads that rent it, who take more room each
    /// when a town's people grow slower than its building. Else its residents
    /// over how densely it is built (`Builds::dens`); the residents themselves
    /// where the run says neither.
    pub fn footprint(&self, k: usize, c: usize) -> f32 {
        match self.fill_of(k, c) {
            Some((fill, _)) => fill * WHOLE_SQUARE,
            None => self.residents(k, c) / self.builds.dens(self.n, k, c),
        }
    }

    /// What year `k`'s rent bytes were taken over: 1 in a run to 1900.
    pub fn rent_scale(&self, k: usize) -> f32 {
        self.rent_scale.get(k).copied().unwrap_or(1.0)
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
pub(crate) fn parse(path: &str, buf: &[u8]) -> Result<Replay, String> {
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
    // a run past 1900 takes each year's rent bytes over that year's anchor
    let rent_scale = if has("rent_scale") { f32s("rent_scale")? } else { Vec::new() };
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
            // each node under a lake names it by its place in the header's list plus 1, and the
            // ground looks the lake's level up by it: a lake the header does not list is refused
            let lake = node_u8("land_lake")?;
            if let Some(&id) = lake.iter().find(|&&id| id as usize > lh.lakes.len()) {
                return Err(format!("{path}: its array land_lake puts a node under lake {id} (counting from 1), and its header lists {} lakes", lh.lakes.len()));
            }
            Some(std::sync::Arc::new(crate::newland::NewLand {
                w,
                h,
                fine: lh.fine,
                fw: lh.fw,
                fh: lh.fh,
                height: node_f32("land_height")?,
                rock: node_u8("land_rock")?,
                lake,
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
    // what the ground holds and what each town made (`Holds`): the sectors are read
    // from the header a second time, since the viewer's header leaves them out
    #[derive(Deserialize)]
    struct SectorsOnly {
        #[serde(default)]
        sectors: Vec<SectorInfo>,
    }
    #[derive(Deserialize)]
    struct SectorInfo {
        name: String,
        good: usize,
    }
    let sectors = serde_json::from_slice::<SectorsOnly>(&buf[4..base]).map(|s| s.sectors).unwrap_or_default();
    let mut holds = Holds { sectors: sectors.into_iter().map(|s| (s.name, s.good)).collect(), ..Default::default() };
    for name in isostate_core::record::STATIC_FIELDS {
        if has(name) {
            let v = u8s(name)?;
            if v.len() == w * h {
                holds.fields.push((name.to_string(), v));
            }
        }
    }
    // the books' table of goods, `[year, town, good, column]`: only its first column, what was made
    if has("acct_goods") {
        let shape = find("acct_goods")?.shape.clone();
        let ng = header.goods.len();
        let kg = isostate_core::accounts::KG;
        let made_col = isostate_core::accounts::GOODS_FIELDS.iter().position(|&c| c == "made").unwrap_or(0);
        if shape.len() == 4 && shape[2] == ng && shape[3] == kg && shape[0] <= ny {
            let table = f32s("acct_goods")?;
            let nt = shape[1];
            if table.len() == shape[0] * nt * ng * kg {
                holds.made = vec![0.0; ny * nt * ng];
                holds.made_towns = nt;
                for (i, row) in table.chunks_exact(kg).enumerate() {
                    holds.made[i] = row[made_col];
                }
            }
        }
    }
    // what the lots of a town past 1900 are built by: all of `dens` and `value` or neither,
    // and the towns' knowledge where the header names its techs
    let mut builds = Builds::default();
    if has("dens") && has("value") {
        let (dens, value) = (u8s("dens")?, u8s("value")?);
        if dens.len() == ny * w * h && value.len() == ny * w * h {
            builds.dens = dens;
            builds.value = value;
        }
    }
    // and how much of each square is built and which is park: both or neither
    if has("fill") && has("park") {
        let (fill, park) = (u8s("fill")?, u8s("park")?);
        if fill.len() == ny * w * h && park.len() == ny * w * h {
            builds.fill = fill;
            builds.park = park;
        }
    }
    // where in each cell each built thing stands (the core's docs/rules/squares.md): the
    // squares' log, both arrays or neither, in the layout the header names; a run made
    // before the squares were kept has none, and says so
    #[derive(Deserialize)]
    struct SquaresOnly {
        squares: Option<SquaresHeader>,
    }
    #[derive(Deserialize)]
    struct SquaresHeader {
        layout: u32,
    }
    let layout = serde_json::from_slice::<SquaresOnly>(&buf[4..base]).ok().and_then(|s| s.squares).map(|s| s.layout);
    match layout {
        Some(l) if l == isostate_core::squares::LOG_LAYOUT && has("sq_first") && has("sq_log") => {
            let (first, log) = (i32s("sq_first")?, typed(path, buf, base, find("sq_log")?, "uint8")?);
            if first.len() != ny + 1 {
                return Err(format!("{path}: its squares' log says where {} years begin, and the run has {ny}", first.len().saturating_sub(1)));
            }
            let first: Vec<usize> = first.iter().map(|&f| f.max(0) as usize).collect();
            if let Some(k) = (0..ny).find(|&k| first[k] > first[k + 1] || first[k + 1] > log.len()) {
                return Err(format!("{path}: its squares' log of year {k} runs from byte {} to {}, and the log has {}", first[k], first[k + 1], log.len()));
            }
            // (each year is read when it is first asked for: `Squares::cell`, `Squares::year`)
            builds.squares = Squares::of_record(log.into(), &first, header.meta.start_year as i64);
        }
        Some(l) if l != isostate_core::squares::LOG_LAYOUT => eprintln!("{path} keeps its squares in a layout the viewer does not know ({l}; it reads {}), so they are left out", isostate_core::squares::LOG_LAYOUT),
        _ => eprintln!("{}", no_squares(path)),
    }
    if has("town_know") && !header.techs.is_empty() {
        let know = u8s("town_know")?;
        let names: Vec<String> = header.techs.iter().map(|t| t.name.clone()).collect();
        builds.name_techs(&names);
        if know.len() == ny * n_towns * names.len() {
            builds.know = know;
            builds.know_towns = n_towns;
        }
    }
    // state (the core's docs/rules/state.md, "The university"): the universities, read
    // from the header a second time, since the viewer's header leaves them out
    #[derive(Deserialize)]
    struct UniOnly {
        #[serde(default)]
        university: Option<UniHeader>,
    }
    #[derive(Deserialize)]
    struct UniHeader {
        #[serde(default)]
        universities: Vec<UniRecord>,
    }
    let unis = serde_json::from_slice::<UniOnly>(&buf[4..base]).ok().and_then(|u| u.university).map_or_else(Vec::new, |u| u.universities);
    // traffic (the core's docs/rules/traffic.md): what each link carried and how, both or
    // neither; and the books' trade between towns summed a pair at a time, from the
    // books' first year (read from the header a second time, as the sectors are)
    let mut traffic = crate::traffic::Traffic::default();
    if has("way_goods") && has("way_mode") {
        let (goods, mode) = (u8s("way_goods")?, u8s("way_mode")?);
        if goods.len() == ny * w * h * 4 && mode.len() == ny * w * h {
            traffic.goods = goods;
            traffic.mode = mode;
        }
    }
    if has("acct_trade_off") && has("acct_trade_key") && has("acct_trade_val") {
        #[derive(Deserialize)]
        struct BooksOnly {
            accounts: Option<BooksStart>,
        }
        #[derive(Deserialize)]
        struct BooksStart {
            start_year: i32,
        }
        let start = serde_json::from_slice::<BooksOnly>(&buf[4..base]).ok().and_then(|b| b.accounts).map_or(header.meta.start_year, |b| b.start_year);
        let weight: Vec<f64> = header.goods.iter().map(|g| g.weight.unwrap_or(0.0) as f64).collect();
        let first = (start - header.meta.start_year).max(0) as usize;
        traffic.pairs = crate::traffic::pairs_of_books(&i32s("acct_trade_off")?, &i32s("acct_trade_key")?, &f32s("acct_trade_val")?, &weight, first, ny);
    }
    let mut r = Replay {
        traffic,
        builds,
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
        rent_scale,
        holds,
    };
    let ny = r.ny;
    stand_universities(&mut r, &unis, ny);
    Ok(r)
}

/// A university as the record's header has it (the core's `university::header`),
/// or as a live game's thread reads it off the simulation (`live::universities`).
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct UniRecord {
    pub town: i32,
    pub cell: usize,
    pub founded: i32,
    #[serde(default)]
    pub closed: Option<i32>,
    pub name: String,
    /// its students at each year's end: (year, students)
    #[serde(default)]
    pub students: Vec<(i32, f32)>,
}

/// Whether works `f` is a university standing as a works (`stand_universities`).
pub fn is_university(r: &Replay, f: usize) -> bool {
    r.header.firms.get(f).is_some_and(|x| x.owner.is_none() && x.reason.is_none() && x.name.starts_with("The University of") && r.header.methods.get(x.method).is_some_and(|m| m.name == "offices"))
}

/// How many universities stand at the end of the run's works.
pub fn universities_standing(r: &Replay) -> usize {
    (0..r.header.firms.len()).rev().take_while(|&f| is_university(r, f)).count()
}

/// Take the universities off the end of a live game's works, before the
/// year's new works are added after the simulation's own (`live::apply_year`
/// wants each new works at the place the simulation gave it), to be stood
/// again after them (`stand_universities`).
pub fn sit_universities(r: &mut Replay) {
    let n = r.header.firms.len() - universities_standing(r);
    if n == r.header.firms.len() {
        return;
    }
    let off = r.firm_off.get(n).map_or(r.firm_k.len(), |&o| o.max(0) as usize);
    r.header.firms.truncate(n);
    r.firm_first.truncate(n);
    r.firm_len.truncate(n);
    r.firm_off.truncate(n);
    r.firm_k.truncate(off);
}

/// Students to a standard works' size: a university of 400 is drawn as one
/// standard works of the offices, one of 20,000 as fifty.
const STUDENTS_A_SIZE: f32 = 400.0;

/// Each university of a recorded run stands on the map as a works does, until it
/// has a drawing of its own: it is put at the end of the record's works, of the
/// offices' kind (a building of storeys planned from the offices' process, as
/// `works::site` plans a kind it does not know), on its town's centre square,
/// its size each year by its students. A run with no offices among its methods
/// (the port's content) has no university to stand. A live game's works are
/// the simulation's own slots, so there they stand after the simulation's
/// works and are sat down and stood again as each year comes
/// (`sit_universities`), with sizes only for the years played (`upto`, the
/// years filled; a recorded run's are all of them), so that one founded at a
/// year's turn stands from the end of the year it is founded in, as in a
/// recorded run.
pub fn stand_universities(r: &mut Replay, unis: &[UniRecord], upto: usize) {
    let Some(m) = r.header.methods.iter().position(|x| x.name == "offices") else { return };
    let k0 = r.header.methods[m].k0.max(1e-6);
    for u in unis {
        let first = (u.founded - r.y0).max(0) as usize;
        let last = u.closed.map_or(r.ny, |y| ((y - r.y0).max(0) as usize).min(r.ny)).min(upto);
        if first >= last {
            continue;
        }
        let mut seen = 0.0f32;
        let sizes: Vec<f32> = (first..last)
            .map(|k| {
                if let Some(&(_, n)) = u.students.iter().find(|s| s.0 == r.y0 + k as i32) {
                    seen = n;
                }
                // a works has capacity from its first year: a university is drawn at its least size until its students come
                (seen / STUDENTS_A_SIZE).max(0.4) * k0
            })
            .collect();
        r.firm_first.push(first as i32);
        r.firm_len.push(sizes.len() as i32);
        r.firm_off.push(r.firm_k.len() as i32);
        r.firm_k.extend(sizes);
        r.header.firms.push(Firm {
            name: u.name.clone(),
            method: m,
            cell: u.cell,
            town: u.town,
            born: u.founded,
            died: u.closed.unwrap_or(-1),
            reason: None,
            owner: None,
        });
    }
}

/// The rent a cell's land earns in a month, from the record's byte: 0 is no
/// rent, and 1 to 255 run from 0.001 to 1000 on a log scale (`record::layers`),
/// in the money of the year's `Replay::rent_scale`.
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

/// Whether any of a good had been made by the end of the run's month `t`
/// (0 is the first January): false for a good no works has made yet, such as
/// town gas before the first gasworks, whose price in the record is one
/// nothing was sold at. A run that does not hold what is made says yes.
pub fn made_by(r: &Replay, good: &str, t: usize) -> bool {
    first_made(r, good).is_some_and(|m| m <= t)
}

/// Whether good `g` has begun by month `t` of the run (docs/rules/goods.md): a
/// good of the start always has; one of a year from its January; one that comes
/// with an idea once any of it is made, which is the first the record can tell
/// of it (the header keeps the ideas' names, not the years they were found).
pub fn begun(r: &Replay, g: usize, t: usize) -> bool {
    match r.header.goods.get(g).and_then(|good| good.begins.as_ref()) {
        None => true,
        Some(Begins::Year(y)) => r.y0 + (t / 12) as i32 >= *y,
        Some(Begins::Idea(_)) => made_by(r, &r.header.goods[g].name, t),
    }
}

/// The first month of the run (0 is the first January) in which any of a good
/// was made, or None if none has been. A run that does not hold what is made
/// says the first month.
pub fn first_made(r: &Replay, good: &str) -> Option<usize> {
    let name = format!("out_{good}");
    let Some(i) = r.series_names.iter().position(|s| *s == name) else { return Some(0) };
    r.series[i * r.series_len..(i + 1) * r.series_len].iter().position(|v| v.is_finite() && *v > 0.0)
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
    fn a_record_made_before_the_squares_opens_with_none_and_the_log_reads_year_by_year() {
        // a record with no squares' log (every record made before 26 November 2026) opens as before
        let old = parse("tiny.bin", &tiny_run()).unwrap();
        assert!(!old.builds.squares.kept() && old.builds.squares.cell(0, 0).is_none() && old.builds.squares.year(0).is_empty());
        assert!(no_squares("tiny.bin").starts_with("tiny.bin keeps no squares"));
        // a log of three years: a works takes two squares of cell 5, then gives one up and
        // town 2's houses take another; then a live game that missed a year is sent the whole book
        let at = |c: usize, k: usize| (c * PER_CELL + k) as u32;
        let works = Tenant::Works(9).code();
        let houses = Tenant::Houses(2).code();
        let mut sq = Squares::default();
        sq.take_year(4, 0, false, &[Change { at: at(5, 17), tenant: works, year: 1750, face: 4 }, Change { at: at(5, 16), tenant: works, year: 1749, face: 4 }]);
        sq.take_year(4, 1, false, &[Change { at: at(5, 17), tenant: 0, year: 0, face: 0 }, Change { at: at(5, 3), tenant: houses, year: 1751, face: 70 }]);
        sq.take_year(4, 2, true, &[Change { at: at(6, 0), tenant: houses, year: 1752, face: 1 }]);
        assert!(sq.kept() && sq.changes(3).is_none());
        let c0 = sq.cell(0, 5).unwrap();
        assert_eq!((c0[16], c0[17], c0[3]), (Square { tenant: Tenant::Works(9), year: 1749, face: 4 }, Square { tenant: Tenant::Works(9), year: 1750, face: 4 }, Square::NONE));
        let c1 = sq.cell(1, 5).unwrap();
        assert_eq!((c1[16].tenant, c1[17], c1[3].tenant, c1[3].year), (Tenant::Works(9), Square::NONE, Tenant::Houses(2), 1751));
        // the whole book of the third year stands alone: cell 5 is no longer held
        assert!(sq.cell(2, 5).is_none() && sq.cell(2, 6).is_some() && sq.cell(1, 6).is_none());
        // a year not yet filled reads as the last that was
        assert_eq!(sq.year(3).keys().copied().collect::<Vec<_>>(), [6]);
        for k in 0..3 {
            for (c, cells) in sq.year(k) {
                assert_eq!(sq.cell(k, c).as_deref(), Some(&*cells), "year {k}, cell {c}");
            }
        }
    }

    /// A measure, run by hand, not a test: how long the viewer takes to open each
    /// run named in `ISOSTATE_TIME_OPEN` (separated by `;`), each opened in turn
    /// seven times, so that a busy machine weighs on all of them alike; the
    /// median and the least, in ms of the wall clock.
    #[test]
    #[ignore]
    fn time_to_open() {
        let Ok(list) = std::env::var("ISOSTATE_TIME_OPEN") else { return };
        let paths: Vec<&str> = list.split(';').collect();
        let mut ms = vec![Vec::new(); paths.len()];
        let mut read = vec![Vec::new(); paths.len()];
        for _ in 0..7 {
            for (i, p) in paths.iter().enumerate() {
                let t = std::time::Instant::now();
                let r = load(p).unwrap();
                ms[i].push(t.elapsed().as_secs_f64() * 1e3);
                assert!(r.ny > 0);
                // and the squares of the last year, the whole log read for the first time
                let t = std::time::Instant::now();
                let cells = r.builds.squares.year(r.ny - 1).len();
                read[i].push((t.elapsed().as_secs_f64() * 1e3, cells));
            }
        }
        for ((p, mut v), mut w) in paths.iter().zip(ms).zip(read) {
            v.sort_by(|a, b| a.total_cmp(b));
            w.sort_by(|a, b| a.0.total_cmp(&b.0));
            eprintln!("{p}: opened in {:.0} ms on the median, {:.0} ms at the least; its last year's squares read in {:.0} ms on the median ({} cells)", v[v.len() / 2], v[0], w[w.len() / 2].0, w[0].1);
        }
    }

    #[test]
    fn a_run_past_1900_reads_its_towns_in_two_year_steps_and_its_rent_in_its_own_money() {
        // a record without the layout's number (every run to 1900, and every
        // record made before there were two) reads as it always has
        let mut r = parse("tiny.bin", &tiny_run()).unwrap();
        (r.built, r.res) = (vec![255, 0], vec![10, 0]);
        assert_eq!(r.header.meta.town_bytes, 0);
        assert_eq!((r.joined(0, 0), r.joined(0, 1), r.residents(0, 0)), (Some(1900), None, 80.0));
        assert_eq!(r.rent_scale(0), 1.0);
        // the second layout: steps of two years from 1645, sixteens, and the rent over the anchor
        r.header.meta.town_bytes = 2;
        r.rent_scale = vec![3.0];
        assert_eq!((r.joined(0, 0), r.residents(0, 0)), (Some(2155), 160.0));
        r.built[0] = 178;
        assert_eq!(r.joined(0, 0), Some(2001));
        assert_eq!(r.rent_scale(0), 3.0);
        assert_eq!(r.rent_scale(1), 1.0, "a year the record has no scale for is read in 1750's money");
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
        // a node under a lake the header does not list (it has one)
        let e = parse("odd.bin", &land_run(true, |n, bytes| if n == "land_lake" { bytes[9] = 2 })).err().unwrap();
        assert_eq!(e, "odd.bin: its array land_lake puts a node under lake 2 (counting from 1), and its header lists 1 lakes");
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
