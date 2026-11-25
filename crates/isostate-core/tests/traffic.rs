//! The traffic record (docs/rules/traffic.md) only watches, and walks the
//! same paths with the same weights as the roads' wear.
#![allow(non_snake_case)]
use isostate_core::*;

/// Island 7 under the game's rules with six families, to `end`, with the
/// traffic record on or off. It leans on no history of the island: only on the
/// two runs being the same run, and on the roads' wear being laid as it is.
fn island(on: bool, end: i64) -> Sim {
    let rules = Rules::Game;
    let mut P = rules.params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(end);
    P.traffic.enabled = on;
    let mut sim = sim::start(&P, rules.content());
    while sim.year <= sim.end_year {
        sim::run_year(&mut sim).unwrap();
    }
    sim
}

#[test]
fn the_traffic_record_changes_nothing_played_and_is_the_roads_own_walk() {
    assert!(!default_params().traffic.enabled && game_params().traffic.enabled);
    let (on, off) = std::thread::scope(|s| {
        let a = s.spawn(|| island(true, 1775));
        let b = s.spawn(|| island(false, 1775));
        (a.join().unwrap(), b.join().unwrap())
    });
    // every monthly series, the news and the roads the same to the bit
    assert_eq!(on.st.series.data.len(), off.st.series.data.len());
    for (k, (a, b)) in on.st.series.data.iter().zip(&off.st.series.data).enumerate() {
        assert!(a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits()), "series {} differs", on.st.series.names[k]);
    }
    assert_eq!(serde_json::to_string(&on.events).unwrap(), serde_json::to_string(&off.events).unwrap());
    assert_eq!(on.st.road, off.st.road);
    // nothing kept with the rule off, and the record has no traffic in it
    assert!(off.st.tf.goods.is_empty() && off.rec.way_goods.is_empty());
    // the weight leaving each land square is the roads' own count of it
    let st = &on.st;
    for c in (0..st.N).filter(|&c| st.world.land[c]) {
        let (a, b) = (st.tf.through[c], st.traffic[c]);
        assert!((a - b).abs() <= 1e-9 * b.abs().max(1.0), "square {c}: {a} against the roads' {b}");
    }
    // every year from the first is kept, with weight sent and carried
    assert_eq!(st.tf.years.len(), on.rec.n_years);
    let last = st.tf.years.last().unwrap();
    assert!(last.sent_district > 0.0 && last.sent_trade > 0.0 && last.weight_squares[traffic::CART as usize] > 0.0);
    // the record's two layers say the same as the state, year by year
    let N = st.N;
    let k = on.rec.n_years - 1;
    for l in 0..N * 4 {
        assert_eq!(on.rec.way_goods[k * N * 4 + l], traffic::goods_byte(st.tf.goods[l] as f64));
        assert_eq!(traffic::mode_of(on.rec.way_mode[k * N + l / 4], l % 4), st.tf.mode[l]);
    }
}
