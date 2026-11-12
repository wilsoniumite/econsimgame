//! The facts the pass over the rules added for its card (docs/designs/rules-pass.md,
//! section 5, "Stage 1's facts"; docs/rules/scorecard.md), each held to its
//! definition: worked out a second way from the run as it goes, and where the fact
//! is a count, to an identity.

use isostate_core::accounts::{KT, TOWN_FIELDS};
use isostate_core::facts::{self, Gathered};
use isostate_core::observer::annual;
use isostate_core::params::Rules;
use isostate_core::sim;

/// What the test counts beside the sweep's notes, month by month, in its own words.
#[derive(Default)]
struct Beside {
    town_months: usize,
    wild: usize,
    over_people: usize,
    compared: usize,
    moved: usize,
    /// Per month: carters, navvies and road menders over all at work.
    others: Vec<f64>,
    /// Per year: what was paid to carry the land's produce and the works' goods, less
    /// nothing, and the trade between towns less its handling.
    paid_by_weight: Vec<f64>,
    hungry_after: usize,
}

#[test]
#[allow(non_snake_case)]
fn the_cards_facts_are_what_they_say() {
    let rules = Rules::Game;
    let mut P = rules.params();
    P.seed = 2;
    P.n_families = 6;
    P.end_year = Some(1790);
    let mut s = sim::start(&P, rules.content());
    let mut g = Gathered::default();
    let mut b = Beside::default();
    let T = s.st.T_max;
    let mut last = s.st.wage.clone();
    let mut took = vec![false; T];
    while s.year <= s.end_year {
        for month in 0..12 {
            sim::run_month(&mut s, month).expect("money is conserved");
            facts::gather_month(&mut g, &s, month);
            let st = &s.st;
            let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t] && st.pop[t] > 0.0).collect();
            let mut w: Vec<f64> = act.iter().map(|&t| st.wage[t]).collect();
            w.sort_by(f64::total_cmp);
            let middle = w[w.len() / 2];
            for &t in &act {
                b.town_months += 1;
                b.wild += (st.wage[t] > 10.0 * middle) as usize;
                b.over_people += (st.month_stats.employment[t] > P.participation * st.pop[t] * (1.0 + 1e-9)) as usize;
                if month > 0 && last[t] > 0.0 {
                    b.compared += 1;
                    let r = st.wage[t] / last[t];
                    b.moved += (r >= 1.5 || r <= 1.0 / 1.5) as usize;
                }
                b.hungry_after += (month < 2 && took[t] && st.food_ratio[t] < 0.82) as usize;
            }
            last = st.wage.clone();
            // everyone at work is on the land, in the works, digging, carting, on a canal or mending a road
            let mut others = vec![0.0; T];
            isostate_core::carting::count_carters(st, &s.params, &mut others);
            isostate_core::canals::count_navvies(st, &s.params, &mut others);
            isostate_core::roads::count_menders(st, &s.params, &mut others);
            b.others.push(others.iter().sum::<f64>() / st.month_stats.employment.iter().sum::<f64>());
        }
        let before = s.st.catch.clone();
        sim::end_year(&mut s);
        facts::gather_year(&mut g, &s);
        took = vec![false; T];
        for (c, &was) in before.iter().enumerate() {
            let now = s.st.catch[c];
            if was >= 0 && now >= 0 && now != was {
                took[now as usize] = true;
            }
        }
        // what the year's carting cost at the freight, from the books: the bill less the handling
        let st = &s.st;
        let k = st.acct.years.len() - 1;
        let rec = st.acct.year(k).expect("the year's books");
        let col = |name: &str| TOWN_FIELDS.iter().position(|&c| c == name).unwrap();
        let bill: f64 = (0..T).map(|t| ["cart_land", "cart_works", "cart_trade"].iter().map(|n| rec.towns[t * KT + col(n)] as f64).sum::<f64>()).sum();
        let handling = s.params.handling * rec.trade.iter().map(|f| f.value as f64).sum::<f64>();
        b.paid_by_weight.push(bill - handling);
    }
    let r = sim::finish(s);
    let f = facts::stylised_facts(&r, &g, 0.0);
    let get = |name: &str| f.get(name).unwrap_or_else(|| panic!("{name} has no value"));
    let (st, y0) = (&r.state, r.content.start_year);
    let series = |name: &str| &st.series.data[st.series.index[name]];
    let at = |name: &str, y: i64| annual(series(name), y0, y);

    // the town-months, counted the same way twice: an identity, to the town-month
    assert!(b.town_months > 1000 && b.compared > 1000, "{} town-months", b.town_months);
    assert_eq!(get("town_months_wage_over_ten_middle"), b.wild as f64);
    assert_eq!(get("town_months_more_at_work_than_people"), b.over_people as f64);
    assert!((get("town_months_wage_moved_by_half") - b.moved as f64 / b.compared as f64).abs() < 1e-12);
    assert!(get("town_wage_greatest_vs_middle") >= 1.0);
    assert_eq!(get("hungry_months_after_new_cells"), b.hungry_after as f64);

    // the diggings and the old mine share: what lies between them is the carters, navvies and
    // menders, to a thousandth in every year noted
    for y in [1750, 1790] {
        let others = b.others[(y - y0) as usize * 12..][..12].iter().sum::<f64>() / 12.0;
        let between = get(&format!("mine_share_{y}")) - get(&format!("diggings_share_{y}"));
        assert!((between - others).abs() < 1e-3, "{y}: mine share less diggings {between}, carters and the rest {others}");
        assert!(others > 0.01, "{y}: there are carters ({others})");
    }
    let span = get("diggings_share_1771_1790");
    assert!(span > 0.0 && span < get("mine_share_1771_1790"));

    // the carters between towns are some of the carters, never more
    for y in [1750, 1790] {
        let between = get(&format!("carters_between_towns_share_of_hands_{y}"));
        assert!(between > 0.0 && between < get(&format!("carters_share_of_hands_{y}")), "{y}: {between}");
    }
    // the weight carried times its path cost, at the freight, is about what the carting cost less
    // the handling: so the carters a unit of it are counted on the right units. Not exactly: the
    // land's carting is paid at a rate a unit set when the land is shared out (`ex_tau`), on the
    // path costs of that moment, and a works' carting at what its inputs cost it over the
    // town's price, where the notes take every flow at the year's last path costs.
    let warm = r.params.warmup_years as usize;
    for y in [1750, 1770, 1790] {
        let k = (y - y0) as usize;
        let (wp, paid) = (g.months.weight_path[k], b.paid_by_weight[k]);
        let ratio = r.params.freight * wp / paid;
        eprintln!("{y}: the freight on the weight and path cost is {ratio:.4} of the bill less the handling");
        assert!((ratio - 1.0).abs() < 0.1, "{y}: the freight on the weight and path cost is {ratio} of the bill less the handling");
        let per = 1000.0 * st.cart_years[warm + k].1 / 12.0 / wp;
        if y != 1770 {
            assert!((get(&format!("carters_per_1000_weight_path_{y}")) - per).abs() < 1e-12);
        }
    }
    assert!((get("carters_per_1000_weight_path_1790_vs_1750") - get("carters_per_1000_weight_path_1790") / get("carters_per_1000_weight_path_1750")).abs() < 1e-12);

    // closures by age: every works that closed from 1750 on closed at some age, and the works-years
    // at risk are every year a works stood from 1750 on
    let (mut closed, mut years) = (0.0, 0.0);
    for (a, z, _) in facts::CLOSING_AGES {
        let (risk, c) = facts::closing_at_age(st, y0, 1790, a, z);
        closed += c;
        years += risk;
    }
    assert_eq!(closed, get("works_closed"));
    let stood: f64 = (0..st.F_count)
        .map(|w| {
            let end = if st.f_alive[w] { 1790 } else { st.f_died[w] };
            (end - st.f_born[w].max(y0) + 1).max(0) as f64
        })
        .sum();
    assert_eq!(years, stood);

    // the works' hands near their market, at the last year's end, from the final state
    let (mut near, mut hands) = (0.0, 0.0);
    for w in (0..st.f_alive.len()).filter(|&w| st.f_alive[w]) {
        hands += st.f_labor[w];
        near += if st.dcatch[st.f_cell[w]] <= 2.0 { st.f_labor[w] } else { 0.0 };
    }
    assert!((get("works_hands_within_2_of_market_1790") - near / hands).abs() < 1e-12);

    // the people and the towns' share
    assert_eq!(get("pop_last_vs_first"), at("pop", 1790) / at("pop", 1750));
    assert!((get("urban_share_last20_gain") - (get("urban_share_1771_1790") - get("urban_share_1751_1770"))).abs() < 1e-12);

    // spending a head over the chained cost of living: decades as their years' means, and the
    // last over the one twenty years before is the two against the first set against each other
    let spend = |y: i64| at("gdp_nom", y) / at("pop", y) / at("cpi_chained", y);
    let decade = |a: i64, z: i64| (a..=z).map(spend).sum::<f64>() / (z - a + 1) as f64;
    assert!((get("spending_per_head_chained_1770s_vs_1750s") - decade(1770, 1779) / decade(1750, 1759)).abs() < 1e-9);
    assert!((get("spending_per_head_chained_1780s_vs_1750s") - decade(1780, 1790) / decade(1750, 1759)).abs() < 1e-9);
    let pair = get("spending_per_head_chained_1780s_vs_1760s");
    assert!((pair - get("spending_per_head_chained_1780s_vs_1750s") / get("spending_per_head_chained_1760s_vs_1750s")).abs() < 1e-9);
    let peak = get("spending_per_head_chained_peak_year") as i64;
    assert!((1750..=1790).all(|y| spend(y) <= spend(peak)));

    // the five worst years: the worst is among them, and their mean is no worse than it
    let rw: Vec<f64> = (1760..=1790).map(|y| at("real_wage", y)).collect();
    let mut falls: Vec<f64> = rw.windows(2).map(|w| w[1] / w[0] - 1.0).collect();
    falls.sort_by(f64::total_cmp);
    let five = falls[..5].iter().sum::<f64>() / 5.0;
    assert!((get("real_wage_five_worst_years_mean") - five).abs() < 1e-9, "{five}");
    assert!(get("real_wage_five_worst_years_mean") >= get("real_wage_worst_year") - 1e-12);

    // timber's swing is read as iron's; the one from 1800 needs a run that reaches it
    assert!(get("price_swing_timber") > 0.0);
    assert!(f.facts.iter().all(|x| x.name != "price_swing_timber_from_1800"));
}

#[test]
fn the_tools_are_dear_against_the_years_about_them() {
    // a price that falls all century and has one dear year in the middle
    let mut p: Vec<f64> = (0..140).map(|k| 10.0 * 0.98f64.powi(k)).collect();
    p[70] *= 3.0;
    // against the years about it only the dear year is dear
    assert_eq!(facts::years_over_twice_local_median(&p, 25), 1);
    // against the whole run's middle the first years read as a famine too
    let mut sorted = p.clone();
    sorted.sort_by(f64::total_cmp);
    let middle = 0.5 * (sorted[69] + sorted[70]);
    assert!(p.iter().filter(|&&v| v > 2.0 * middle).count() > 20);
    // a window as long as the run is the whole run's middle
    assert_eq!(facts::years_over_twice_local_median(&p, 281), p.iter().filter(|&&v| v > 2.0 * middle).count());
    assert_eq!(facts::years_over_twice_local_median(&[], 25), 0);
}

#[test]
fn the_decades_end_with_the_run() {
    assert_eq!(facts::decades(1750, 1790), vec![(1750, 1750, 1759), (1760, 1760, 1769), (1770, 1770, 1779), (1780, 1780, 1790)]);
    let d = facts::decades(1750, 1900);
    assert_eq!((d.len(), d[0], d[d.len() - 1], d[d.len() - 3]), (15, (1750, 1750, 1759), (1890, 1890, 1900), (1870, 1870, 1879)));
    assert_eq!(facts::decades(1755, 1769), vec![(1750, 1755, 1759), (1760, 1760, 1769)]);
}
