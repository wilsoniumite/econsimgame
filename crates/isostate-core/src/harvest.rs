//! Good years and bad: the harvest. Not in the Python; see `docs/rules/harvest.md`.
//!
//! Each year the land gives a little more or less than anyone could have
//! planned for. One number is drawn for the whole island and a smooth field for
//! its districts, so that one valley can fail while the next does well; together
//! they make a multiplier on what every farm, and more faintly every pasture,
//! yields that year. The multiplier is applied in `econ::step_month` after the
//! hands are hired and the machines asked for, so the weather is a surprise:
//! the crop is not what the wage bill was laid out for, and the landlord, who
//! takes what is left, carries the difference. Half of it, that is. About half
//! a farm's year is ploughing and sowing, done before anyone knows, and half is
//! reaping and threshing, done when the crop can be seen standing; so half the
//! weather (`foreseen`) is known when the hands are hired, a thin crop wants
//! fewer of them, and the wage feels a bad year too. Land bids, works and
//! investors go on planning for an average year. The rest follows through the
//! markets that were already there: stocks of grain run down, its price rises,
//! the real wage falls, and in the worst years `people::growth` finds towns
//! short of food.
//!
//! The draws come from a stream of their own (`Rngs::harvest`, stream 7), and
//! nothing here runs, draws or is stored while `P.harvest.enabled` is off.
//!
//! The yearly order, in `sim::yearly`: the books close on the year just
//! finished (its factors move to `harvest_year` and `harvest_town`, where the
//! record and the live view read them, and the news is written), then next
//! year's weather is drawn. No weather is drawn for the warm-up years, so 1750
//! starts from the same settled economy with the mechanism on or off.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::families;
use crate::mat::Mat;
use crate::observer::annual;
use crate::params::Params;
use crate::record::Record;
use crate::rng::{Rng, random};
use crate::state::{Event, State};
use crate::util::fmt_pct;
use crate::world::{World, smooth_noise};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HarvestParams {
    /// Harvests differ from year to year.
    pub enabled: bool,
    /// Standard deviation of the island-wide part of the log of a year's yield.
    pub island_sd: f64,
    /// Standard deviation of the district part, across the island's land in one year.
    pub district_sd: f64,
    /// How far a district's weather reaches, in cells: land this far apart
    /// shares little of it.
    pub reach: f64,
    /// Share of the shock that pastures feel (farms feel all of it).
    pub pasture: f64,
    /// Share of the shock that woods feel.
    pub woods: f64,
    /// Share of the year's weather that the land's masters can see when they
    /// hire. At 0 it is a pure surprise: the hands are taken on for an average
    /// year, the rent alone feels the harvest, and the real wage does not fall
    /// in a bad year (measured: docs/rules/harvest.md). At 1 the crop is seen
    /// standing all year and fewer hands are taken on to bring a thin one in.
    /// The game's half is the half of a farm's year that comes after the crop
    /// shows: reaping, threshing and carting.
    pub foreseen: f64,
    /// The country's harvest is news when it is this many `island_sd` from an
    /// average year, either way.
    pub news_z: f64,
    /// A district's failure is news when its harvest is this many `district_sd`
    /// below the country's, and as far below an average year.
    pub news_local_z: f64,
    /// Grain is in dearth when its yearly price stands this far above its trend.
    pub dearth_rise: f64,
}

impl Default for HarvestParams {
    /// Off: what the port runs with. The numbers are the game's, so that
    /// `--set harvest.enabled=true` measures the mechanism as it is played.
    fn default() -> HarvestParams {
        HarvestParams {
            enabled: false,
            island_sd: 0.05,
            district_sd: 0.05,
            reach: 30.0,
            pasture: 0.4,
            woods: 0.0,
            foreseen: 0.5,
            news_z: 1.8,
            news_local_z: 2.3,
            dearth_rise: 0.12,
        }
    }
}

impl HarvestParams {
    /// The game's harvests.
    pub fn game() -> HarvestParams {
        HarvestParams { enabled: true, ..HarvestParams::default() }
    }
}

/// A year is lean when the country's harvest is this many `island_sd` short,
/// and this many lean years running are news.
const LEAN_Z: f64 = 0.5;
const LEAN_YEARS: usize = 3;
/// A dearth already announced is not announced again for this many years.
const DEARTH_QUIET: i64 = 3;

/// One standard normal draw (Box and Muller's method: two uniforms each).
fn normal(rng: &mut Rng) -> f64 {
    let u1 = 1.0 - random(rng); // in (0, 1], so the log is finite
    let u2 = random(rng);
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// The year's district weather: a smooth field with mean 0 and standard
/// deviation 1 across the land, alike over about `reach` cells.
fn district_field(rng: &mut Rng, world: &World, reach: f64) -> Vec<f64> {
    let raw = smooth_noise(rng, world.H, world.W, reach, 1, 0.5);
    let land: Vec<usize> = (0..raw.len()).filter(|&c| world.land[c]).collect();
    let n = land.len().max(1) as f64;
    let mean = land.iter().map(|&c| raw[c]).sum::<f64>() / n;
    let var = land.iter().map(|&c| (raw[c] - mean) * (raw[c] - mean)).sum::<f64>() / n;
    let sd = var.sqrt().max(1e-12);
    raw.iter().map(|v| (v - mean) / sd).collect()
}

/// The land sectors that feel the weather, and how much of it: (sector, share).
fn feeling(st: &State, C: &Content, P: &Params) -> Vec<(usize, f64)> {
    let H = &P.harvest;
    let mut v = Vec::new();
    for (s, name) in C.sector_name.iter().enumerate() {
        let share = if name == "farm" {
            1.0
        } else if name == "pasture" {
            H.pasture
        } else if s == st.forestry_sector {
            H.woods
        } else {
            0.0
        };
        if share != 0.0 {
            v.push((s, share));
        }
    }
    v
}

fn farm_sector(C: &Content) -> Option<usize> {
    C.sector_name.iter().position(|n| n == "farm")
}

/// Draw the coming year's weather and work out what it does to each sector's
/// land, to each town's farms and to the country's.
fn draw(st: &mut State, C: &Content, P: &Params, rng: &mut Rng) {
    let H = &P.harvest;
    let (N, T, NS) = (st.N, st.T_max, C.NS);
    // the same draws in the same order whatever the sizes are set to, so that a
    // sweep over them compares the same weather at different strengths
    let z = normal(rng);
    let field = district_field(rng, &st.world, H.reach);
    let var = H.island_sd * H.island_sd + H.district_sd * H.district_sd;
    let mut mult = Mat::filled(NS, N, 1.0);
    for (s, share) in feeling(st, C, P) {
        for c in 0..N {
            if st.world.land[c] {
                // log-normal with mean one: an average year is what the land bids expect
                let shock = H.island_sd * z + H.district_sd * field[c];
                mult[(s, c)] = (share * shock - 0.5 * share * share * var).exp();
            }
        }
    }
    // each town's harvest is the mean over its farmland, weighted by what the
    // land would bear (its share of the cell times its fertility); the country's
    // is the same mean over all farmland
    let mut town = vec![1.0; T];
    let mut nation = 1.0;
    if let Some(fs) = farm_sector(C) {
        let (mut acc, mut wsum) = (vec![0.0; T], vec![0.0; T]);
        for u in 0..st.ex_cell.len() {
            if st.ex_sector[u] == fs {
                let w = st.ex_lam[u] * st.ex_field[u];
                acc[st.ex_town[u]] += w * mult[(fs, st.ex_cell[u])];
                wsum[st.ex_town[u]] += w;
            }
        }
        for t in 0..T {
            if st.town_active[t] {
                // a town with no farmland yet is given the weather at its own door
                town[t] = if wsum[t] > 0.0 { acc[t] / wsum[t] } else { mult[(fs, st.town_node[t])] };
            }
        }
        let (a, w): (f64, f64) = (acc.iter().sum(), wsum.iter().sum());
        nation = if w > 0.0 { a / w } else { (H.island_sd * z - 0.5 * var).exp() };
    }
    st.harvest_mult = mult;
    st.harvest_now = nation;
    st.harvest_town_now = town;
}

/// What the land's masters can see of the year when they hire: each unit's
/// worth of a hand (`Cc` in `econ::step_month`) is scaled by its cell's
/// multiplier to the power `foreseen`. Nothing at 0, the pure surprise.
pub fn foreseen(st: &State, P: &Params, Cc: &mut [f64]) {
    let share = P.harvest.foreseen;
    if !P.harvest.enabled || share == 0.0 || st.harvest_mult.data.is_empty() {
        return;
    }
    for u in 0..Cc.len() {
        let m = st.harvest_mult[(st.ex_sector[u], st.ex_cell[u])];
        if m != 1.0 {
            Cc[u] *= m.powf(share);
        }
    }
}

/// The crop actually brought in. `Y` is what each land unit would yield in an
/// average year with the hands it has hired; farms and pastures get the year's
/// multiplier (all of it: what was foreseen only changed how many were hired). Called from `econ::step_month` once labour and machine requests
/// are settled, so everything after it (sales, rents, stocks) is the real crop.
pub fn brought_in(st: &mut State, C: &Content, P: &Params, Y: Vec<f64>) -> Vec<f64> {
    if !P.harvest.enabled || st.harvest_mult.data.is_empty() {
        return Y;
    }
    // with families, keep count of the rent the weather took or gave on each
    // cell, so that land can still be priced on an average year (`average_rent`)
    let track = families::active(st);
    if track && st.harvest_rent_gap.len() != st.N {
        st.harvest_rent_gap = vec![0.0; st.N];
    }
    let keep = 1.0 - P.merchant_spread;
    let mut Y = Y;
    for u in 0..Y.len() {
        let (s, c) = (st.ex_sector[u], st.ex_cell[u]);
        let m = st.harvest_mult[(s, c)];
        if m == 1.0 {
            continue;
        }
        let got = Y[u] * m;
        if track {
            // rent is what is left of the sale after cartage and wages, so every
            // unit of crop lost is lost to the rent at its price net of cartage
            let net = st.price[(st.ex_town[u], C.sector_good[s])] * keep - st.ex_tau[u];
            st.harvest_rent_gap[c] += (Y[u] - got) * net;
        }
        Y[u] = got;
    }
    Y
}

/// The rent a cell would have paid last year had the hands it hired brought in
/// an average crop, given the rent it did pay. Unchanged while the mechanism is
/// off. (With `foreseen` above 0 the hands hired were themselves fewer or more
/// than in an average year, which this does not undo: it is near enough for
/// pricing land, and exact at 0.)
pub fn average_rent(st: &State, cell: usize, paid: f64) -> f64 {
    match st.harvest_rent_gap_year.get(cell) {
        Some(gap) => paid + gap,
        None => paid,
    }
}

/// The yearly step, called from `sim::yearly` before the families act: close
/// the books on the year just finished, tell the news, draw the coming year.
#[allow(clippy::too_many_arguments)]
pub fn yearly(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, year: i64, events: &mut Vec<Event>, warm: bool) {
    if !P.harvest.enabled {
        return;
    }
    if !st.harvest_mult.data.is_empty() {
        st.harvest_year = st.harvest_now;
        st.harvest_town = st.harvest_town_now.clone();
        st.harvest_past.push(st.harvest_year);
        if families::active(st) {
            st.harvest_rent_gap_year = std::mem::replace(&mut st.harvest_rent_gap, vec![0.0; st.N]);
        }
        if !warm {
            news(st, C, P, year, events);
        }
    }
    // the warm-up has no weather: the first draw is for the first recorded year
    if year + 1 >= C.start_year {
        draw(st, C, P, rng);
    }
}

/// The yearly snapshot's part: each town's harvest and the country's, kept only
/// when the mechanism is on (the arrays stay empty otherwise, and `record::pack`
/// leaves them out of the file).
pub fn snapshot(rec: &mut Record, st: &State, P: &Params, k: usize) {
    if !P.harvest.enabled {
        return;
    }
    let T = rec.T;
    if rec.town_harvest.is_empty() {
        rec.town_harvest = vec![1.0; rec.n_years * T];
        rec.harvest = vec![1.0; rec.n_years];
    }
    if st.harvest_town.len() == T && k < rec.n_years {
        for t in 0..T {
            rec.town_harvest[k * T + t] = st.harvest_town[t] as f32;
        }
        rec.harvest[k] = st.harvest_year as f32;
    }
}

/// The news of the year just finished, in the observer's manner: it says what
/// was seen, and at most a few things a decade.
///
/// - a poor harvest or a bumper one, when the country's crop is `news_z`
///   island deviations from an average year;
/// - lean years, when the third short harvest running comes in (it takes the
///   place of that year's "poor harvest");
/// - a district's failure, when one town's farms do `news_local_z` district
///   deviations worse than the country and than an ordinary year, and the
///   country's own harvest is not news;
/// - a dearth, when the year's price of grain stands `dearth_rise` above its
///   trend (the mean of the five years before, carried forward at the drift of
///   the last twenty), at most once in `DEARTH_QUIET` years.
fn news(st: &mut State, C: &Content, P: &Params, year: i64, events: &mut Vec<Event>) {
    let H = &P.harvest;
    let Some(fs) = farm_sector(C) else { return };
    let g = C.sector_good[fs];
    let (gname, glabel) = (&C.good_name[g], &C.good_label[g]);
    let y0 = C.start_year;
    let nation = st.harvest_year;
    let dev = nation.ln();
    // the towns that stood when the weather was drawn
    let towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t] && st.town_founded[t] < year).collect();
    let by = |best: bool| -> Option<usize> {
        let pick = |a: &usize, b: &usize| st.harvest_town[*a].total_cmp(&st.harvest_town[*b]);
        if best { towns.iter().copied().max_by(pick) } else { towns.iter().copied().min_by(pick) }
    };
    // the price of grain this year against last, where the observer has both
    let price = st.series.index.get(&format!("price_{gname}")).map(|&k| &st.series.data[k]);
    let price_at = |y: i64| price.map_or(f64::NAN, |p| annual(p, y0, y));
    let dearer = price_at(year) / price_at(year - 1) - 1.0;
    let out_series = Some(format!("out_{gname}"));
    let when = year as f64 + 0.55;

    let limit = H.news_z * H.island_sd;
    let lean = |v: f64| H.island_sd > 0.0 && v.ln() < -LEAN_Z * H.island_sd;
    let past = &st.harvest_past;
    let n = past.len();
    let lean_run = n >= LEAN_YEARS
        && past[n - LEAN_YEARS..].iter().all(|&v| lean(v))
        && (n == LEAN_YEARS || !lean(past[n - LEAN_YEARS - 1]));
    let mut national = false;
    if lean_run {
        national = true;
        let short: Vec<String> = past[n - LEAN_YEARS..].iter().map(|&v| fmt_pct(1.0 - v, 0)).collect();
        // three years a little short are a remark, three bad ones a calamity
        let lost: f64 = past[n - LEAN_YEARS..].iter().map(|&v| 1.0 - v).sum();
        let mut ev = Event::new(
            when,
            "harvest",
            (0.3 + 2.0 * lost).clamp(0.45, 0.75),
            "Three lean years".to_string(),
            format!(
                "For the third year running the harvest falls short of an ordinary one: by {}, by {} and now by {}.",
                short[0], short[1], short[2]
            ),
        );
        ev.series = out_series.clone();
        events.push(ev);
    } else if H.island_sd > 0.0 && dev < -limit {
        national = true;
        let short = 1.0 - nation;
        let mut text = format!("The harvest comes in {} short of an ordinary year.", fmt_pct(short, 0));
        if let Some(w) = by(false) {
            if st.harvest_town[w] < nation - 0.01 {
                text += &format!(" {} has the worst of it, {} short.", st.town_name[w], fmt_pct(1.0 - st.harvest_town[w], 0));
            }
        }
        if dearer.is_finite() && dearer >= 0.01 {
            text += &format!(" {glabel} is {} dearer than last year.", fmt_pct(dearer, 0));
        }
        let mut ev = Event::new(when, "harvest", (0.35 + 2.5 * short).clamp(0.4, 0.75), "A poor harvest".to_string(), text);
        ev.series = out_series.clone();
        if let Some(w) = by(false) {
            ev.town = Some(w);
            ev.cell = Some(st.town_node[w]);
        }
        events.push(ev);
    } else if H.island_sd > 0.0 && dev > limit {
        national = true;
        let over = nation - 1.0;
        let mut text = format!("The barns are full: the harvest is {} above an ordinary year.", fmt_pct(over, 0));
        if let Some(b) = by(true) {
            if st.harvest_town[b] > nation + 0.01 {
                text += &format!(" It is best around {}, {} above.", st.town_name[b], fmt_pct(st.harvest_town[b] - 1.0, 0));
            }
        }
        if dearer.is_finite() && dearer <= -0.01 {
            text += &format!(" {glabel} is {} cheaper than last year.", fmt_pct(-dearer, 0));
        }
        let mut ev = Event::new(when, "harvest", (0.25 + 1.5 * over).clamp(0.3, 0.5), "A bumper harvest".to_string(), text);
        ev.series = out_series.clone();
        events.push(ev);
    }

    // one district fails while the country does not
    if !national && H.district_sd > 0.0 {
        if let Some(w) = by(false) {
            let f = st.harvest_town[w];
            // short of the country's harvest and of an ordinary year alike: a
            // district that merely misses a good year the rest enjoy is no failure
            let short = -H.news_local_z * H.district_sd;
            if (f / nation).ln() < short && f.ln() < short {
                let country = if nation < 0.98 {
                    format!("is {} short", fmt_pct(1.0 - nation, 0))
                } else if nation > 1.02 {
                    format!("is {} above one", fmt_pct(nation - 1.0, 0))
                } else {
                    "does about as usual".to_string()
                };
                let mut ev = Event::new(
                    when,
                    "harvest",
                    (0.3 + 1.5 * (1.0 - f)).clamp(0.35, 0.55),
                    format!("The harvest fails around {}", st.town_name[w]),
                    format!(
                        "Farms around {} bring in {} less than in an ordinary year, while the country as a whole {country}.",
                        st.town_name[w],
                        fmt_pct(1.0 - f, 0)
                    ),
                );
                ev.series = out_series.clone();
                ev.town = Some(w);
                ev.cell = Some(st.town_node[w]);
                events.push(ev);
            }
        }
    }

    // dearth: the price of grain far above its trend
    let k = year - y0;
    if price.is_some() && k >= 6 && year - st.harvest_dearth_year >= DEARTH_QUIET {
        let lp = |y: i64| price_at(y).ln();
        let level = (1..=5).map(|j| lp(year - j)).sum::<f64>() / 5.0;
        let m = (k - 1).min(20);
        let drift = (lp(year - 1) - lp(year - 1 - m)) / m as f64;
        // the five years before are centred three years back
        let above = lp(year) - (level + 3.0 * drift);
        if above.is_finite() && above >= (1.0 + H.dearth_rise).ln() {
            st.harvest_dearth_year = year;
            let mut text = format!("{glabel} sells {} above where the prices of the last years pointed", fmt_pct(above.exp_m1(), 0));
            if nation < 0.98 {
                text += &format!(", after a harvest {} short", fmt_pct(1.0 - nation, 0));
            }
            text.push('.');
            let wage = st.series.index.get("real_wage").map(|&k| &st.series.data[k]);
            let fall = wage.map_or(f64::NAN, |w| 1.0 - annual(w, y0, year) / annual(w, y0, year - 1));
            if fall.is_finite() && fall >= 0.01 {
                text += &format!(" A wage buys {} less than it did last year.", fmt_pct(fall, 0));
            }
            let mut ev = Event::new(
                year as f64 + 0.7,
                "dearth",
                (0.5 + above.exp_m1()).clamp(0.55, 0.8),
                format!("A dearth of {}", glabel.to_lowercase()),
                text,
            );
            ev.series = Some(format!("price_{gname}"));
            events.push(ev);
        }
    }
}
