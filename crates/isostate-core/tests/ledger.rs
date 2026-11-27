//! The month's ledger of buyers (ledger.rs) adds up. For a month of island 7
//! in 1760, 1850, 1950 and 2040, from each of the two starts, what each kind
//! of buyer asked, got and paid agrees with the ledger's own totals and with
//! the books (docs/rules/accounts.md): kind by kind where the books keep the
//! kinds apart, and together where they join them (the households and their
//! lorries; the land's machinery, chemicals and machines for tasks; works
//! being built and their machines' plant; railways, canals and the
//! university's rooms).
//!
//! Played under `game_params()` with six families, as the game's golden test
//! is. The test leans on no island's history: whatever buyers a month has,
//! they must add up. Which kinds were there is printed, not asserted (with
//! `--nocapture`), since that is the island's history.
#![allow(non_snake_case)]

use isostate_core::accounts::{GOODS_FIELDS, KG, KT, TOWN_FIELDS};
use isostate_core::ledger::{ASK_ORDER, Buyer, Kind, Ledger, Trade};
use isostate_core::mat::Mat;
use isostate_core::*;

const YEARS: [i64; 4] = [1760, 1850, 1950, 2040];
/// The month of each year looked at: July, after six months played as a run plays them.
const MONTH: usize = 6;

fn gcol(name: &str) -> usize {
    GOODS_FIELDS.iter().position(|&n| n == name).unwrap()
}

fn tcol(name: &str) -> usize {
    TOWN_FIELDS.iter().position(|&n| n == name).unwrap()
}

/// Two sums of the same terms added in other orders agree to a few parts in
/// 1e12 of the largest number either was reckoned from: the month's figure in
/// the books is the year's running total after it less the one before, so it
/// is as close as that total is (`scale`).
fn agree(a: f64, b: f64, scale: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(scale).max(1e-6)
}

/// What the lines of `kinds` got, by town and good.
fn got_by_town(lg: &Ledger, kinds: &[Kind], T: usize, G: usize) -> Mat {
    let mut m = Mat::zeros(T, G);
    for &k in kinds {
        let Some(l) = lg.line(k) else { continue };
        for r in 0..l.ask.rows {
            for j in 0..l.ask.cols {
                m[(l.town[r], l.good(j))] += l.got[(r, j)];
            }
        }
    }
    m
}

/// What the lines of `kinds` got, at `price`, by town.
fn paid_by_town(lg: &Ledger, kinds: &[Kind], price: &Mat, T: usize) -> Vec<f64> {
    let mut v = vec![0.0; T];
    for &k in kinds {
        let Some(l) = lg.line(k) else { continue };
        for r in 0..l.ask.rows {
            for j in 0..l.ask.cols {
                v[l.town[r]] += l.got[(r, j)] * price[(l.town[r], l.good(j))];
            }
        }
    }
    v
}

const HH: Kind = (Buyer::Households, Trade::Day);
const LORDS: Kind = (Buyer::Landlords, Trade::Day);
const WORKS: Kind = (Buyer::Works, Trade::Day);
const LORRIES: [Kind; 2] = [(Buyer::Lorries, Trade::Day), (Buyer::Lorries, Trade::Plant)];
const LAND: [Kind; 4] = [(Buyer::LandMachinery, Trade::Day), (Buyer::LandChemicals, Trade::Day), (Buyer::LandTasks, Trade::Day), (Buyer::LandTasks, Trade::Plant)];
const BUILDING: [Kind; 2] = [(Buyer::WorksOrders, Trade::Plant), (Buyer::WorksTasks, Trade::Plant)];
const WAYS: Kind = (Buyer::Ways, Trade::Plant);
const ROOMS: Kind = (Buyer::University, Trade::Plant);

/// One month played through `econ::month_ledger` on a copy of the island, and
/// the ledger set against itself and against the books. Gives the kinds seen.
fn check_month(sim: &sim::Sim, year: i64) -> Vec<String> {
    let mut sim = sim.clone();
    for m in 0..MONTH {
        sim::run_month(&mut sim, m).expect("money is conserved");
    }
    let (T, G) = (sim.st.T_max, sim.content.G);
    let (g0, t0) = {
        let (g, t) = sim.st.acct.year_so_far();
        (g.to_vec(), t.to_vec())
    };
    // everything bought is paid at the prices the month began with
    let price = sim.st.price.clone();
    let lg = econ::month_ledger(&mut sim.st, &sim.content, &sim.params);
    let st = &sim.st;
    let (g1, t1) = st.acct.year_so_far();
    let book = |t: usize, g: usize, c: &str| g1[(t * G + g) * KG + gcol(c)] - g0[(t * G + g) * KG + gcol(c)];
    let book_total = |t: usize, g: usize, c: &str| g1[(t * G + g) * KG + gcol(c)].abs();
    let town = |t: usize, c: &str| t1[t * KT + tcol(c)] - t0[t * KT + tcol(c)];
    let town_total = |t: usize, c: &str| t1[t * KT + tcol(c)].abs();
    let at = |what: &str, t: usize, g: usize| format!("{what}, {year}, town {t}, good {}", sim.content.good_name[g]);

    // the ledger against itself: each line's got is its ask at its trade's fill
    // (but the works', which use what their scarcest input allows, and never
    // more than they asked); the markets' totals are the lines' sums
    let (mut asked, mut sold) = (Mat::zeros(T, G), Mat::zeros(T, G));
    let mut seen = Vec::new();
    for kind in ASK_ORDER {
        let Some(l) = lg.line(kind) else { continue };
        seen.push(format!("{:?} {:?} ({} rows)", kind.0, kind.1, l.ask.rows));
        let fill = if kind.1 == Trade::Day { &lg.fill } else { &lg.fill_plant };
        for r in 0..l.ask.rows {
            let t = l.town[r];
            for j in 0..l.ask.cols {
                let (g, a, x) = (l.good(j), l.ask[(r, j)], l.got[(r, j)]);
                asked[(t, g)] += a;
                sold[(t, g)] += x;
                if kind == WORKS {
                    assert!(x <= a * (1.0 + 1e-12) + 1e-12, "{}: a works used {x} of {a} asked", at("works", t, g));
                } else {
                    assert_eq!(x, a * fill[(t, g)], "{}: got is the ask at the fill", at(&format!("{kind:?}"), t, g));
                }
            }
        }
    }
    for t in (0..T).filter(|&t| st.town_active[t]) {
        for g in 0..G {
            assert!(agree(asked[(t, g)], lg.asked[(t, g)], 0.0), "{}: lines asked {} and the market {}", at("asked", t, g), asked[(t, g)], lg.asked[(t, g)]);
            assert!(agree(sold[(t, g)], lg.sold[(t, g)], 0.0), "{}: lines got {} and the market sold {}", at("sold", t, g), sold[(t, g)], lg.sold[(t, g)]);
        }
    }

    // the ledger against the books, good by good: what was asked, and what each
    // kind got where the books keep it
    let hh = got_by_town(&lg, &[HH, LORRIES[0], LORRIES[1]], T, G);
    let lords = got_by_town(&lg, &[LORDS], T, G);
    let works = got_by_town(&lg, &[WORKS], T, G);
    let land = got_by_town(&lg, &LAND, T, G);
    let building = got_by_town(&lg, &BUILDING, T, G);
    let projects = got_by_town(&lg, &[WAYS, ROOMS], T, G);
    for t in (0..T).filter(|&t| st.town_active[t]) {
        for g in 0..G {
            let pairs: [(&str, &[&str], f64); 7] = [
                ("asked", &["asked"], lg.asked[(t, g)]),
                ("households and their lorries", &["households"], hh[(t, g)]),
                ("landlords and families", &["landlords", "families"], lords[(t, g)]),
                ("works' materials and fuel", &["works_inputs", "works_fuel"], works[(t, g)]),
                ("the land", &["land"], land[(t, g)]),
                ("works being built", &["building"], building[(t, g)]),
                ("railways, canals and rooms", &["projects"], projects[(t, g)]),
            ];
            for (what, cols, l) in pairs {
                let b: f64 = cols.iter().map(|c| book(t, g, c)).sum();
                let scale: f64 = cols.iter().map(|c| book_total(t, g, c)).sum();
                assert!(agree(b, l, scale), "{}: the books {b}, the ledger {l}", at(what, t, g));
            }
            let cols = ["households", "landlords", "families", "works_inputs", "works_fuel", "land", "building", "projects"];
            let bought: f64 = cols.iter().map(|c| book(t, g, c)).sum();
            let scale: f64 = cols.iter().map(|c| book_total(t, g, c)).sum();
            assert!(agree(bought, lg.sold[(t, g)], scale), "{}: the books bought {bought}, the market sold {}", at("sold", t, g), lg.sold[(t, g)]);
        }
    }

    // and what each paid, town by town, at the month's prices
    let navvies = {
        let mut v = vec![0.0; T];
        for pr in &st.projects {
            v[pr.town] += pr.navvy_pay;
        }
        v
    };
    let spend_hh = paid_by_town(&lg, &[HH, LORRIES[0], LORRIES[1]], &price, T);
    let spend_lords = paid_by_town(&lg, &[LORDS], &price, T);
    let invest = paid_by_town(&lg, &BUILDING, &price, T);
    let projects = paid_by_town(&lg, &[WAYS, ROOMS], &price, T);
    for t in (0..T).filter(|&t| st.town_active[t]) {
        let pairs: [(&str, &[&str], f64); 4] = [
            ("households' spending", &["spend_households"], spend_hh[t]),
            ("landlords' and families' spending", &["spend_landlords", "spend_families"], spend_lords[t]),
            ("works' investment", &["invest"], invest[t]),
            ("railways', canals' and rooms' outlay", &["projects"], projects[t] + navvies[t]),
        ];
        for (what, cols, l) in pairs {
            let b: f64 = cols.iter().map(|c| town(t, c)).sum();
            let scale: f64 = cols.iter().map(|c| town_total(t, c)).sum();
            assert!(agree(b, l, scale), "{what}, {year}, town {t}: the books {b}, the ledger {l}");
        }
    }
    seen
}

fn check(start: &str) {
    let mut P = game_params();
    landfall::set_start(&mut P, start).expect("the start is one of the two");
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(*YEARS.last().unwrap());
    assert!(P.accounts.enabled, "the books are kept under the game's rules");
    let mut sim = sim::start(&P, game_content());
    for &year in &YEARS {
        while sim.year < year {
            sim::run_year(&mut sim).expect("money is conserved");
        }
        let seen = check_month(&sim, year);
        println!("{start}, July {year}: {}", seen.join(", "));
    }
}

#[test]
fn the_ledger_adds_up_on_the_settled_island() {
    check("settled");
}

#[test]
fn the_ledger_adds_up_at_landfall() {
    check("landfall");
}
