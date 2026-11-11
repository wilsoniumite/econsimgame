//! Canals (not in the Python; docs/rules/canals.md): somebody surveys one, the
//! first is dug where its know-how is, the rest follow from town to town, the
//! links are the network, boats carry what goes far, the railway takes the
//! traffic, money is conserved and a run is deterministic. With the switch off
//! the golden test (tests/golden.rs) covers that the port did not move.
#![allow(non_snake_case)]

use std::sync::OnceLock;

use isostate_core::transport as tr;
use isostate_core::*;

/// The year the runs here stop: on seed 3 the port's railways open in the
/// 1830s, so by now they have taken the canals' traffic.
const END: i64 = 1860;

fn params(on: bool) -> Params {
    let mut P = default_params();
    P.seed = 3;
    P.end_year = Some(END);
    P.canals.enabled = on;
    P
}

fn run(on: bool) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(&params(on), default_content(), false, true).expect("money is conserved every month")
}

/// One run with the canals on, shared by the tests below.
fn canals() -> &'static RunResult {
    static RUN: OnceLock<RunResult> = OnceLock::new();
    RUN.get_or_init(|| run(true))
}

fn header(r: &RunResult) -> serde_json::Value {
    let blob = record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "test", true);
    let len = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    serde_json::from_slice(&blob[4..4 + len]).expect("the header is JSON")
}

#[test]
fn canals_conserve_money_and_runs_are_deterministic() {
    let (a, b) = (canals(), run(true));
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (k, name) in st.series.names.iter().enumerate() {
        let (x, y) = (&st.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs between identical runs");
    }
    assert!(st.canal == b.state.canal && st.cn.links == b.state.cn.links && st.cn.book == b.state.cn.book, "the same canals are dug both times");
    assert!(st.cn.found == b.state.cn.found && st.cn.years == b.state.cn.years);
    let titles = |r: &RunResult| r.events.iter().map(|e| e.title.clone()).collect::<Vec<_>>();
    assert_eq!(titles(a), titles(&b));
    // a canal orders what its cash will pay for, wages and all: it never owes anything
    assert!(st.projects.iter().all(|pr| pr.cash >= 0.0));
}

#[test]
fn switched_off_nothing_is_kept_or_recorded() {
    let off = run(false);
    let st = &off.state;
    assert!(st.canal.is_empty() && st.cn.links.is_empty() && st.cn.know.is_empty() && st.cn.years.is_empty());
    assert!(st.cn.book == ways::WayBook::default() && st.cn.found.is_none() && st.cn.rng.is_none());
    assert!(st.projects.iter().all(|pr| pr.way == ways::PORT));
    assert!(off.rec.layer("trans").iter().all(|&t| t & (16 | 32) == 0), "no canal bit in the transport layer");
    assert!(header(&off).get("canals").is_none(), "no canals in the run's header");
    assert!(!off.events.iter().any(|e| e.kind == "canal" || e.kind == "canal_plan"));
    // and the facts have only the rows any run can answer
    let facts = canals::facts(&off);
    assert!(facts.iter().all(|f| !f.0.starts_with("canal_")));
    assert!(facts.iter().any(|f| f.0 == "coal_price_spread_1800" && f.2.is_finite()));
    // with them on, the same rows and the canals' own
    let facts = canals::facts(canals());
    assert!(facts.iter().any(|f| f.0 == "coal_price_spread_1800" && f.2.is_finite()));
    assert!(facts.iter().any(|f| f.0 == "canal_first_year" && f.2 > 1750.0 && f.2 < END as f64));
}

#[test]
fn a_canal_is_surveyed_then_dug_then_copied() {
    let r = canals();
    let (st, book) = (&r.state, &r.state.cn.book);
    let (found, town) = st.cn.found.expect("somebody surveys a canal before 1860");
    let news = r.events.iter().find(|e| e.kind == "discovery" && e.title.contains("canal")).expect("the survey is news");
    assert!(news.year.floor() as i64 == found && news.town == Some(town));
    assert!(!book.lines.is_empty(), "canals are dug");
    // nothing is promoted before the survey, the first by the town that made it,
    // and it takes years to dig
    let first = book.lines.iter().fold(&book.lines[0], |a, l| if l.opened < a.opened { l } else { a });
    assert!(book.lines.iter().all(|l| l.start >= found), "no canal before anybody knows how");
    assert!(first.a == town || first.b == town, "the first canal is the surveying town's");
    assert!(first.opened - first.start as f64 >= 3.0, "a canal takes years to dig: {} to {}", first.start, first.opened);
    assert!(book.lines.iter().all(|l| l.spent > 0.0));
    // the know-how has reached other towns, and they have dug too
    let towns: std::collections::BTreeSet<usize> = book.lines.iter().flat_map(|l| [l.a, l.b]).collect();
    assert!(towns.len() >= 4, "canals in {} towns", towns.len());
    assert!(st.cn.know.iter().filter(|&&k| k >= r.params.canals.know).count() >= 4);
    // the first canal is the great news, and the only one; the rest are told modestly
    let items: Vec<&Event> = r.events.iter().filter(|e| e.kind == "canal" || e.kind == "canal_plan").collect();
    let great: Vec<&&Event> = items.iter().filter(|e| e.weight >= 0.9).collect();
    assert!(great.len() == 1 && great[0].title.starts_with("The first canal opens"), "{:?}", great.iter().map(|e| &e.title).collect::<Vec<_>>());
    assert!((great[0].year - first.opened).abs() < 1e-9);
    assert!(items.iter().filter(|e| e.weight < 0.9).all(|e| e.weight <= 0.3));
    // long before the railway
    let rail = st.series.data[st.series.index["rail_km"]].iter().position(|&v| v > 0.0).map(|t| r.content.start_year as f64 + t as f64 / 12.0);
    assert!(rail.is_none_or(|y| first.opened + 20.0 < y), "the first canal in {}, the first railway in {rail:?}", first.opened);
}

#[test]
fn the_links_are_the_network() {
    let r = canals();
    let st = &r.state;
    let (W, N) = (st.world.W, st.N);
    assert!(st.cn.links.len() >= 50, "{} links", st.cn.links.len());
    let mut linked = vec![false; N];
    for l in &st.cn.links {
        let (dy, dx) = ((l.a / W).abs_diff(l.b / W), (l.a % W).abs_diff(l.b % W));
        assert!(dy <= 1 && dx <= 1 && l.a != l.b, "a link joins neighbouring cells");
        assert!(l.opened > l.begun as f64 && st.canal[l.a] > 0 && st.canal[l.b] > 0);
        linked[l.a] = true;
        linked[l.b] = true;
    }
    // no link twice, in either direction
    let mut pairs: Vec<(usize, usize)> = st.cn.links.iter().map(|l| (l.a.min(l.b), l.a.max(l.b))).collect();
    pairs.sort_unstable();
    let n = pairs.len();
    pairs.dedup();
    assert_eq!(pairs.len(), n);
    for c in 0..N {
        // every cell of canal is on a link, on land, and off the mountains
        assert_eq!(st.canal[c] > 0, linked[c], "cell {c}");
        if st.canal[c] > 0 {
            assert!(st.world.land[c] && st.world.terrain[c] != world::MOUNTAINS);
        }
    }
    // every cell was paid for by exactly one canal, open or digging
    let paid: usize = st.cn.book.lines.iter().map(|l| l.new_cells).sum();
    assert_eq!(paid, st.canal.iter().filter(|&&c| c > 0).count());

    // the record: the links in the header, the cells in the transport layer
    let h = header(r);
    let links = h["canals"]["links"].as_array().expect("links in the header");
    let digging: usize = st.projects.iter().filter(|pr| pr.way == ways::CANAL).count();
    assert!(links.len() >= st.cn.links.len() && (digging > 0 || links.len() == st.cn.links.len()));
    let l0 = &links[0];
    assert!(l0["a"].as_u64() == Some(st.cn.links[0].a as u64) && l0["b"].as_u64() == Some(st.cn.links[0].b as u64));
    assert!(l0["begun"].as_i64() == Some(st.cn.links[0].begun) && l0["opened"].as_f64().is_some());
    let k = r.rec.n_years - 1;
    let trans = &r.rec.layer("trans")[k * N..(k + 1) * N];
    for c in 0..N {
        assert_eq!(trans[c] & 16 > 0, st.canal[c] > 0, "the canal bit of cell {c}");
    }
    // the live view is given the same links
    assert_eq!(canals::links_out(st).len(), links.len());
}

#[test]
fn boats_carry_what_goes_far_and_the_railway_takes_it() {
    let r = canals();
    let st = &r.state;
    let years = &st.cn.years;
    let peak = years.iter().fold(&years[0], |a, y| if y.ton_cells > a.ton_cells { y } else { a });
    assert!(peak.ton_cells > 0.0 && peak.boarded > 0.0);
    // the mean haul by boat is many cells: the wharves keep short hops on the road
    let haul = peak.ton_cells / peak.boarded;
    assert!(haul > 8.0, "the mean haul by boat is {haul:.1} cells");
    // coal, stone and grain are most of it
    let C = &r.content;
    let all: f64 = peak.trade.iter().sum::<f64>() + peak.land.iter().sum::<f64>() + peak.works;
    let sector = |name: &str| C.sector_name.iter().position(|n| n == name).unwrap();
    let bulk: f64 = ["coal", "stone", "grain"].iter().map(|g| peak.trade[C.gi(g)]).sum::<f64>()
        + ["coalmine", "quarry", "farm"].iter().map(|s| peak.land[sector(s)]).sum::<f64>();
    assert!(bulk > 0.6 * all, "coal, stone and grain are {:.2} of what the boats carry", bulk / all);
    // the two reckonings of the year's traffic agree roughly (one walks the year's
    // paths with the year's weights, the other adds up the months good by good)
    assert!(all > 0.5 * peak.ton_cells && all < 2.0 * peak.ton_cells, "{all:e} by goods against {:e} by paths", peak.ton_cells);
    // boats pass only cells of canal, and when most was carried nearly every cell was used
    assert!((0..st.N).all(|c| st.cn.traffic[c] == 0.0 || st.canal[c] > 0));
    assert!(peak.busy_cells as f64 > 0.8 * peak.cells as f64);
    // navvies dug them, and were counted
    assert!(years.iter().any(|y| y.navvies > 50.0));
    // then the railway: by the end the boats carry a fraction of their best year,
    // though nothing has closed a canal
    let last = years.last().unwrap();
    assert!(st.rail.iter().any(|&x| x > 0), "seed 3 has railways by {END}");
    assert!(peak.year < last.year && last.ton_cells < 0.5 * peak.ton_cells, "{:e} in {} against {:e} in {}", last.ton_cells, last.year, peak.ton_cells, peak.year);
    assert!(last.cells >= peak.cells);
}

/// A strip of six cells in a row, every step costing 1, with a canal along it.
fn strip() -> tr::Graph {
    let (W, H) = (6, 1);
    let mut w = vec![f64::INFINITY; W * H * 8];
    for c in 0..W {
        for k in 0..8 {
            if tr::neighbour(H, W, c, k).is_some() {
                w[c * 8 + k] = 1.0;
            }
        }
    }
    let mut g = tr::Graph::new(W * H, W, H, w, vec![false; W * H * 8]);
    g.wharf = 0.5;
    for c in 0..W - 1 {
        assert!(g.link_boats(c, c + 1, 0.1));
    }
    assert!(!g.link_boats(1, 0, 0.1), "a link is made once");
    g
}

#[test]
fn a_short_hop_stays_on_the_road_and_a_long_one_takes_the_boat() {
    let g = strip();
    let p = tr::dijkstra(&g, &[(0, 0.0)], None);
    // one cell on: the road (1.0) beats two wharves and a link (1.1)
    assert_eq!((p.dist[1], p.pred[1]), (1.0, 0));
    // further: on at cell 0, off where it is going, and the cell before is where it got on
    for c in 2..6 {
        assert!((p.dist[c] - (1.0 + 0.1 * c as f64)).abs() < 1e-12, "cell {c}: {}", p.dist[c]);
        assert_eq!(p.pred[c], 0);
    }
    // the search that only wants distances gives the same ones
    let d = tr::distances(&g, &[(0, 0.0)]);
    assert!(d.iter().zip(&p.dist).all(|(x, y)| x.to_bits() == y.to_bits()), "{d:?} against {:?}", p.dist);
    // from the middle, and from two sources at different starts
    for sources in [vec![(3usize, 0.0)], vec![(0, 0.25), (5, 0.0)]] {
        let (a, b) = (tr::dijkstra(&g, &sources, None).dist, tr::distances(&g, &sources));
        assert!(a.iter().zip(&b).all(|(x, y)| x.to_bits() == y.to_bits()), "{a:?} against {b:?}");
    }
    // a cart that crosses the canal pays nothing for it: with the links taken up
    // again the one-cell hop costs what it did
    let mut bare = g.clone();
    for c in 0..5 {
        bare.unlink_boats(c, c + 1);
    }
    assert_eq!(tr::distances(&bare, &[(0, 0.0)]), vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
}

#[test]
fn canals_alone_or_in_company_are_the_same_mechanism() {
    // the switch is one field of the parameters, and the game has it on
    assert!(!default_params().canals.enabled && game_params().canals.enabled);
    let on = default_params().with_json_overrides(r#"{"canals": {"enabled": true}}"#).unwrap();
    assert_eq!(on.canals, canals::CanalParams::game());
    // off, the parameters' file does not mention them
    let text = serde_json::to_string(&default_params()).unwrap();
    assert!(!text.contains("canals"));
}
