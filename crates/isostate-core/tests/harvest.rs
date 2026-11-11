//! The harvest (`harvest.rs`, docs/rules/harvest.md): off under the port and on
//! in the game; felt by farms and pastures after their labour is hired; money
//! conserved and runs repeatable with it on; a bad year dear in bread and low in
//! the real wage, and told; news that does not flood the feed; each town's
//! harvest kept for the viewer only when it is on.

use isostate_core::harvest::HarvestParams;
use isostate_core::mat::Mat;
use isostate_core::sim::{self, RunResult};
use isostate_core::{Params, default_content, default_params, econ, families, game_params, live, record};

#[allow(non_snake_case)]
fn params(seed: u64, end_year: i64, harvest: Option<HarvestParams>) -> Params {
    let mut P = default_params();
    P.seed = seed;
    P.end_year = Some(end_year);
    if let Some(h) = harvest {
        P.harvest = h;
    }
    P
}

#[allow(non_snake_case)]
fn run(P: &Params) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(P, default_content(), false, true).expect("money conserved")
}

/// The names of the arrays in a packed run's header.
fn array_names(blob: &[u8]) -> Vec<String> {
    let n = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    let header: serde_json::Value = serde_json::from_slice(&blob[4..4 + n]).expect("the header is JSON");
    header["arrays"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap().to_string()).collect()
}

fn harvest_news(r: &RunResult) -> Vec<&isostate_core::Event> {
    r.events.iter().filter(|e| e.kind == "harvest" || e.kind == "dearth").collect()
}

#[test]
#[allow(non_snake_case)]
fn off_under_the_port_and_on_in_the_game() {
    assert_eq!(default_params().harvest, HarvestParams::default());
    assert!(!default_params().harvest.enabled);
    assert!(game_params().harvest.enabled);
    // the port's params.json never mentions it; the game's does
    let port = serde_json::to_value(default_params()).unwrap();
    assert!(port.get("harvest").is_none());
    assert_eq!(serde_json::to_value(game_params()).unwrap()["harvest"]["enabled"], true);
    // switching it on alone gives the game's numbers
    let on = default_params().with_json_overrides(r#"{"harvest": {"enabled": true}}"#).unwrap();
    assert_eq!(on.harvest, HarvestParams::game());

    // off: nothing is drawn, kept, told or written
    let r = run(&params(3, 1752, None));
    let st = &r.state;
    assert!(st.harvest_mult.data.is_empty() && st.harvest_town.is_empty() && st.harvest_past.is_empty());
    assert!(st.harvest_rent_gap.is_empty() && st.harvest_rent_gap_year.is_empty());
    assert!(r.rec.town_harvest.is_empty() && r.rec.harvest.is_empty());
    assert!(harvest_news(&r).is_empty());
    let blob = record::pack(st, &r.content, &r.params, &r.events, &r.rec, "off", false);
    assert!(!array_names(&blob).iter().any(|n| n.contains("harvest")));
}

#[test]
#[allow(non_snake_case)]
fn the_weather_reaches_farms_and_pastures_after_the_hands_are_hired() {
    // as a pure surprise first (`foreseen` 0), then with the crop seen standing
    let P = params(4, 1751, Some(HarvestParams { district_sd: 0.1, foreseen: 0.0, ..HarvestParams::game() }));
    let s = sim::start(&P, default_content());
    let (st, C) = (&s.st, &s.content);
    let (N, NS) = (st.N, C.NS);
    let sector = |name: &str| C.sector_name.iter().position(|n| n == name).unwrap();
    let (farm, pasture) = (sector("farm"), sector("pasture"));

    // the first year's weather is drawn at the end of the warm-up
    let m = &st.harvest_mult;
    assert_eq!((m.rows, m.cols), (NS, N));
    let land: Vec<usize> = (0..N).filter(|&c| st.world.land[c]).collect();
    for s in 0..NS {
        if s != farm && s != pasture {
            assert!(m.row(s).iter().all(|&v| v == 1.0), "{} feels the weather", C.sector_name[s]);
        }
    }
    assert!((0..N).filter(|&c| !st.world.land[c]).all(|c| m[(farm, c)] == 1.0));
    let (lo, hi) = land.iter().fold((f64::INFINITY, 0.0f64), |(lo, hi), &c| (lo.min(m[(farm, c)]), hi.max(m[(farm, c)])));
    assert!(lo > 0.5 && hi < 1.6 && hi / lo > 1.1, "farm factors run from {lo} to {hi}");
    // pastures feel a fixed share of what farms feel
    let share = P.harvest.pasture;
    let offset = |c: usize| m[(pasture, c)].ln() - share * m[(farm, c)].ln();
    assert!(land.iter().all(|&c| (offset(c) - offset(land[0])).abs() < 1e-12));
    // every town has a factor, and the country's lies among them
    let towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    let tf: Vec<f64> = towns.iter().map(|&t| st.harvest_town_now[t]).collect();
    let (tmin, tmax) = tf.iter().fold((f64::INFINITY, 0.0f64), |(a, b), &v| (a.min(v), b.max(v)));
    assert!(tmin - 1e-9 <= st.harvest_now && st.harvest_now <= tmax + 1e-9);
    assert!(tmax / tmin > 1.02, "districts differ: towns run from {tmin} to {tmax}");
    assert!((0..st.T_max).filter(|&t| !st.town_active[t]).all(|t| st.harvest_town_now[t] == 1.0));

    // one month with the weather and one without, from the same state: unseen,
    // the same hands at the same wage, and each unit's crop scaled by its cell's factor
    let mut wet = s.st.clone();
    let mut dry = s.st.clone();
    dry.harvest_mult = Mat::default();
    econ::step_month(&mut wet, C, &P);
    econ::step_month(&mut dry, C, &P);
    assert!(wet.ex_labor.iter().zip(&dry.ex_labor).all(|(a, b)| a.to_bits() == b.to_bits()));
    assert!(wet.wage.iter().zip(&dry.wage).all(|(a, b)| a.to_bits() == b.to_bits()));
    let mut moved = 0;
    for u in 0..wet.ex_cell.len() {
        let want = dry.ex_out[u] * m[(wet.ex_sector[u], wet.ex_cell[u])];
        assert!((wet.ex_out[u] - want).abs() <= 1e-12 * want.abs().max(1.0));
        moved += (wet.ex_out[u] != dry.ex_out[u]) as usize;
    }
    assert!(moved > 100, "only {moved} land units felt the weather");
    // no money is made or lost by the weather: what was not issued this month is
    // the same sum either way (the surprise is paid for out of rent)
    let (a, b) = (econ::money_total(&wet) - wet.issued, econ::money_total(&dry) - dry.issued);
    assert!((a - b).abs() < 1e-9 * a, "money differs: {a} against {b}");

    // with `foreseen` the thin crop is seen standing: where the harvest is poor
    // fewer hands are asked for and the town's wage is lower, the crop per hand the same
    let mut Q = P.clone();
    Q.harvest.foreseen = 1.0;
    let mut seen = s.st.clone();
    econ::step_month(&mut seen, C, &Q);
    let worst = *towns.iter().min_by(|&&a, &&b| st.harvest_town_now[a].total_cmp(&st.harvest_town_now[b])).unwrap();
    let best = *towns.iter().max_by(|&&a, &&b| st.harvest_town_now[a].total_cmp(&st.harvest_town_now[b])).unwrap();
    assert!(st.harvest_town_now[worst] < 1.0 && st.harvest_town_now[best] > 1.0);
    assert!(seen.wage[worst] < wet.wage[worst], "the wage where the harvest is worst: {} seen, {} unseen", seen.wage[worst], wet.wage[worst]);
    assert!(seen.wage[best] > wet.wage[best], "the wage where the harvest is best: {} seen, {} unseen", seen.wage[best], wet.wage[best]);
    let (a, b) = (econ::money_total(&seen) - seen.issued, econ::money_total(&dry) - dry.issued);
    assert!((a - b).abs() < 1e-9 * a, "money differs: {a} against {b}");

    // the share seen is held between 0 and 1: more than all of it is all of it,
    // and less than none is none
    let month = |foreseen: f64| {
        let mut Q = P.clone();
        Q.harvest.foreseen = foreseen;
        let mut st = s.st.clone();
        econ::step_month(&mut st, C, &Q);
        st
    };
    let same = |a: &[f64], b: &[f64]| a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits());
    assert!(same(&month(2.0).wage, &seen.wage) && same(&month(2.0).ex_labor, &seen.ex_labor));
    assert!(same(&month(-1.0).wage, &wet.wage) && same(&month(-1.0).ex_labor, &wet.ex_labor));
}

#[test]
#[allow(non_snake_case)]
fn a_sweep_over_sizes_compares_the_same_weather() {
    // the draws do not depend on the sizes, so twice the deviation is the same
    // year twice as far from average
    let z = |sd: f64| {
        let P = params(6, 1750, Some(HarvestParams { island_sd: sd, district_sd: 0.0, ..HarvestParams::game() }));
        let s = sim::start(&P, default_content());
        (s.st.harvest_now.ln() + 0.5 * sd * sd) / sd
    };
    let (a, b) = (z(0.05), z(0.1));
    assert!((a - b).abs() < 1e-9, "the island's draw was {a} at 0.05 and {b} at 0.1");
}

#[test]
#[allow(non_snake_case)]
fn money_is_conserved_and_runs_repeat_with_harvests_on() {
    let P = params(3, 1772, Some(HarvestParams::game()));
    let a = run(&P);
    let b = run(&P);
    for (k, name) in a.state.series.names.iter().enumerate() {
        let (x, y) = (&a.state.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs between identical runs");
    }
    let titles = |r: &RunResult| r.events.iter().map(|e| e.title.clone()).collect::<Vec<_>>();
    assert_eq!(titles(&a), titles(&b));
    let blob_a = record::pack(&a.state, &a.content, &a.params, &a.events, &a.rec, "a", false);
    let blob_b = record::pack(&b.state, &b.content, &b.params, &b.events, &b.rec, "a", false);
    assert!(blob_a == blob_b, "identical runs pack to different bytes");
    // the run file carries each town's harvest and the country's
    let names = array_names(&blob_a);
    assert!(names.contains(&"town_harvest".to_string()) && names.contains(&"harvest".to_string()));

    // and it is not the port's run: the same seed without harvests grows other grain
    let off = run(&params(3, 1772, None));
    let grain = |r: &RunResult| r.state.series.get("out_grain").to_vec();
    assert!(grain(&a).iter().zip(grain(&off)).any(|(x, y)| (x / y - 1.0).abs() > 0.01));
}

#[test]
#[allow(non_snake_case)]
fn harvests_are_recorded_and_the_news_is_sparse() {
    let P = params(2, 1809, Some(HarvestParams::game()));
    let r = run(&P);
    let (rec, T) = (&r.rec, r.rec.T);
    let years = rec.n_years;
    assert_eq!(years, 60);
    assert_eq!(rec.town_harvest.len(), years * T);
    assert_eq!(rec.harvest.len(), years);
    // the country's harvests average out, and no two towns have the same sixty years
    let mean = rec.harvest.iter().map(|&v| v as f64).sum::<f64>() / years as f64;
    assert!((mean - 1.0).abs() < 0.03, "the mean harvest of {years} years is {mean}");
    assert!(rec.harvest.iter().any(|&v| v < 0.95) && rec.harvest.iter().any(|&v| v > 1.05));
    assert!((0..years).any(|k| (rec.town_harvest[k * T] - rec.town_harvest[k * T + 1]).abs() > 0.02));

    // a good year brings in more grain and sells it cheaper, a bad one the reverse:
    // over the sixty years the harvest goes with the year's change in grain
    // output and against the change in its price
    let y0 = r.content.start_year;
    let against_harvest = |name: &str| -> f64 {
        let v: Vec<f64> = (0..years).map(|k| isostate_core::observer::annual(r.state.series.get(name), y0, y0 + k as i64)).collect();
        let h: Vec<f64> = (1..years).map(|k| (rec.harvest[k] as f64).ln()).collect();
        let d: Vec<f64> = (1..years).map(|k| (v[k] / v[k - 1]).ln()).collect();
        let (mh, md) = (h.iter().sum::<f64>() / h.len() as f64, d.iter().sum::<f64>() / d.len() as f64);
        let cov: f64 = h.iter().zip(&d).map(|(a, b)| (a - mh) * (b - md)).sum();
        let (vh, vd): (f64, f64) = (h.iter().map(|a| (a - mh) * (a - mh)).sum(), d.iter().map(|b| (b - md) * (b - md)).sum());
        cov / (vh * vd).sqrt()
    };
    let (with_output, with_price) = (against_harvest("out_grain"), against_harvest("price_grain"));
    assert!(with_output > 0.2, "harvest and the change in grain output: correlation {with_output}");
    assert!(with_price < -0.2, "harvest and the change in the grain price: correlation {with_price}");

    let news = harvest_news(&r);
    assert!(!news.is_empty(), "sixty years and no harvest worth a line");
    assert!(news.len() <= 3 * years / 10, "{} harvest items in {years} years", news.len());
    for e in &news {
        assert!(e.weight >= 0.3 && e.weight <= 0.8, "{}: weight {}", e.title, e.weight);
        assert!(!e.title.is_empty() && e.text.ends_with('.'), "{}: {}", e.title, e.text);
    }
    // a poor harvest is told in the year it happened, with the shortfall it had
    for e in news.iter().filter(|e| e.title == "A poor harvest") {
        let k = (e.year.floor() as i64 - r.content.start_year) as usize;
        assert!(rec.harvest[k] < 0.95, "{}: the harvest was {}", e.year, rec.harvest[k]);
    }
}

#[test]
#[allow(non_snake_case)]
fn a_bad_year_is_dear_bread_and_a_lower_real_wage_and_the_news_says_so() {
    // shocks large enough to stand out from the port's own swing in grain
    let P = params(5, 1809, Some(HarvestParams { island_sd: 0.12, ..HarvestParams::game() }));
    let r = run(&P);
    let (y0, years) = (r.content.start_year, r.rec.n_years);
    let yearly = |name: &str| -> Vec<f64> {
        (0..years).map(|k| isostate_core::observer::annual(r.state.series.get(name), y0, y0 + k as i64)).collect()
    };
    let (price, wage) = (yearly("price_grain"), yearly("real_wage"));
    let harvest: Vec<f64> = r.rec.harvest.iter().map(|&v| v as f64).collect();
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;

    // the real wage falls in the years the harvest is short and rises in the full ones
    let change = |v: &[f64], k: usize| (v[k] / v[k - 1]).ln();
    let short: Vec<f64> = (1..years).filter(|&k| harvest[k] < 0.92).map(|k| change(&wage, k)).collect();
    let full: Vec<f64> = (1..years).filter(|&k| harvest[k] > 1.08).map(|k| change(&wage, k)).collect();
    assert!(short.len() >= 5 && full.len() >= 5, "{} short years and {} full ones", short.len(), full.len());
    assert!(mean(&short) < -0.01 && mean(&full) > 0.01, "the real wage moves {} in short years and {} in full ones", mean(&short), mean(&full));

    // a dearth is told in years when grain is dear, which are the years after short crops
    let dearths: Vec<usize> = r.events.iter().filter(|e| e.kind == "dearth").map(|e| (e.year.floor() as i64 - y0) as usize).collect();
    assert!(!dearths.is_empty(), "sixty years of hard weather and no dearth");
    assert!(dearths.windows(2).all(|w| w[1] - w[0] >= 3), "dearths told in {dearths:?}");
    for &k in &dearths {
        let before = mean(&price[k - 5..k]);
        assert!(price[k] > before, "{}: grain at {} after five years at {before}", y0 + k as i64, price[k]);
    }
    let in_dearth = mean(&dearths.iter().map(|&k| harvest[k]).collect::<Vec<_>>());
    assert!(in_dearth < 0.97, "the harvest in the years of dearth averaged {in_dearth}");
    for e in r.events.iter().filter(|e| e.kind == "dearth") {
        assert!(e.weight >= 0.55 && e.weight <= 0.8 && e.title == "A dearth of grain", "{}: weight {}", e.title, e.weight);
        assert_eq!(e.series.as_deref(), Some("price_grain"));
    }

    // every poor harvest that is told was one, and the worst year of the sixty is told
    let told = |title: &str| -> Vec<usize> {
        r.events.iter().filter(|e| e.kind == "harvest" && e.title == title).map(|e| (e.year.floor() as i64 - y0) as usize).collect()
    };
    let limit = (-P.harvest.news_z * P.harvest.island_sd).exp();
    assert!(told("A poor harvest").iter().all(|&k| harvest[k] < limit));
    assert!(told("A bumper harvest").iter().all(|&k| harvest[k] > 1.0 / limit));
    let worst = (0..years).min_by(|&a, &b| harvest[a].total_cmp(&harvest[b])).unwrap();
    assert!(told("A poor harvest").contains(&worst) || told("Three lean years").contains(&worst), "the worst harvest, {} in {}, went untold", harvest[worst], y0 + worst as i64);
}

#[test]
#[allow(non_snake_case)]
fn with_families_money_holds_and_land_is_priced_on_an_average_year() {
    let mut P = params(5, 1764, Some(HarvestParams { district_sd: 0.12, ..HarvestParams::game() }));
    P.n_families = 6;
    let mut s = sim::start(&P, default_content());
    let mut cursor = live::LiveCursor::new(&s.content);
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved");
        let y = live::live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        // the live view has the year's harvest of every town slot, and the country's
        assert_eq!(y.harvest_town.len(), s.st.T_max);
        assert!(y.harvest.is_some_and(|v| v > 0.5 && v < 1.6));
    }
    let st = &s.st;
    assert_eq!(st.harvest_rent_gap_year.len(), st.N);
    let hit: Vec<usize> = (0..st.N).filter(|&c| st.harvest_rent_gap_year[c].abs() > 1e-9).collect();
    assert!(hit.len() > 100, "the weather touched the rent of only {} cells", hit.len());
    // where the rent, set right for the weather, clears the floor, the price is 25 years of it
    let mut checked = 0;
    for &c in &hit {
        let t = st.catch[c];
        let floor = if t >= 0 { 12.0 * P.land_rent_floor * st.wage[t as usize] } else { 0.0 };
        let fair = st.cell_rent_year[c] + st.harvest_rent_gap_year[c];
        if fair > floor {
            let price = families::land_price(st, &P, c);
            assert!((price - P.years_purchase * fair).abs() <= 1e-9 * price.abs());
            checked += 1;
        }
    }
    assert!(checked > 50, "only {checked} cells priced above the floor");
}
