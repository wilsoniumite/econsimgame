//! Haulage (not in the Python; docs/rules/haulage.md): from 1901 the railway a
//! layer of its own in the cost graph, a load paying the wharf's charge to be
//! put on a train and again to be taken off (`haulage.handling`); and a road
//! worn by its own traffic, not the railway's (`haulage.wear`).
//!
//! The searches are checked on a made-up map. The island run is played on
//! island 7 to 1906 under the game's rules with six families, with the two
//! switches set by name and every other rule as the game has it; what it checks
//! holds on any island's history (a run against another run, a tree's steps,
//! weights that add up), but for one thing: that island 7 has track by 1902.
#![allow(non_snake_case)]

use isostate_core::haulage::{self, HaulageParams};
use isostate_core::transport::{self as tr, Graph, BY_GROUND, BY_TRAIN, NO_PRED, RAIL_COST};
use isostate_core::*;

/// A map of `W` by 3 squares, every step on the ground costing `ground` a
/// square (a straight one; a diagonal sqrt 2 of it), with track along the
/// middle row from x = 1 to x = W - 2, a train costing `RAIL_COST` a step and
/// the station `h` each way.
fn map(W: usize, ground: f64, h: f64) -> Graph {
    let (H, N) = (3, 3 * W);
    let mut w = vec![f64::INFINITY; N * 8];
    for a in 0..N {
        for k in 0..8 {
            if tr::neighbour(H, W, a, k).is_some() {
                w[a * 8 + k] = ground * tr::step_len(k);
            }
        }
    }
    let mut g = Graph::new(N, W, H, w, vec![false; N * 8]);
    g.train = (0..N).map(|c| c / W == 1 && c % W >= 1 && c % W <= W - 2).collect();
    g.handling = h;
    g
}

/// A load that rides the line pays the station twice and the rail between;
/// one that goes a square stays on the road; one that only crosses the line
/// pays nothing for it; the tree says which cells were reached off a train,
/// from where it was boarded; the bucket queue's distances are the heap's;
/// and the railway put back on the ground is the port's railed edge.
#[test]
fn the_station_is_a_layer_of_its_own() {
    let (W, h) = (14, 0.4);
    let g = map(W, 0.3, h);
    let at = |x: usize, y: usize| y * W + x;
    let p = tr::dijkstra(&g, &[(at(1, 1), 0.0)], None);
    // to the far end of the line by train: 0.4 + 11 steps at 0.07 + 0.4
    let far = at(W - 2, 1);
    let rail = 2.0 * h + RAIL_COST * (W - 3) as f64;
    assert!((p.dist[far] - rail).abs() < 1e-12, "{} against {rail}", p.dist[far]);
    assert!(rail < 0.3 * (W - 3) as f64);
    assert_eq!((p.pred[far], p.by[far]), (at(1, 1) as i64, BY_TRAIN), "left the train where it was boarded");
    // one square along the line: the road (0.3) is cheaper than the station's 0.87
    assert_eq!((p.dist[at(2, 1)], p.pred[at(2, 1)], p.by[at(2, 1)]), (0.3, at(1, 1) as i64, BY_GROUND));
    // across the line from the row above to the row below: two squares of ground, no station
    let q = tr::dijkstra(&g, &[(at(5, 0), 0.0)], None);
    assert!((q.dist[at(5, 2)] - 0.6).abs() < 1e-12 && q.by[at(5, 2)] == BY_GROUND);
    // every tree's step is a neighbour's over the ground, or a train boarded at a square with track
    for c in 0..g.N {
        let pr = p.pred[c];
        if pr == NO_PRED {
            continue;
        }
        let pr = pr as usize;
        if p.by[c] == BY_TRAIN {
            assert!(g.train[c] && g.train[pr], "{c} left a train boarded at {pr}");
        } else {
            assert!((0..8).any(|k| tr::neighbour(g.H, g.W, pr, k) == Some(c)), "{c} reached over the ground from {pr}");
        }
    }
    // the bucket queue gives the heap's distances, from several sources at offsets too
    let srcs = [(at(1, 1), 0.0), (at(9, 0), 0.25), (at(12, 2), 1.1)];
    let (heap, buckets) = (tr::dijkstra(&g, &srcs, None).dist, tr::distances(&g, &srcs));
    for c in 0..g.N {
        assert!((heap[c] - buckets[c]).abs() < 1e-12, "square {c}: {} against {}", heap[c], buckets[c]);
    }
    // put back on the ground, the railway is the port's railed edge and nothing is charged
    let mut flat = g.clone();
    flat.flatten_train();
    assert!(!flat.has_train() && flat.handling == 0.0);
    for a in 0..g.N {
        for k in 0..8 {
            let Some(b) = tr::neighbour(g.H, g.W, a, k) else { continue };
            let both = g.train[a] && g.train[b];
            assert_eq!(flat.railed[a * 8 + k], both);
            let want = if both { RAIL_COST * tr::step_len(k) } else { g.w[a * 8 + k] };
            assert_eq!(flat.w[a * 8 + k].to_bits(), want.to_bits());
        }
    }
    let f = tr::dijkstra(&flat, &[(at(1, 1), 0.0)], None);
    assert!((f.dist[far] - RAIL_COST * (W - 3) as f64).abs() < 1e-12 && f.by.is_empty());
}

/// Island 7 under the game's rules with six families, to `end`, with the two switches as given.
fn island(handling: bool, wear: bool, end: i64) -> Sim {
    let rules = Rules::Game;
    let mut P = rules.params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(end);
    P.traffic.enabled = true;
    P.haulage = HaulageParams { handling, wear };
    let mut sim = sim::start(&P, rules.content());
    while sim.year <= sim.end_year {
        sim::run_year(&mut sim).expect("money is conserved every month");
        if sim.year - 1 == 1900 {
            break;
        }
    }
    sim
}

/// The switches are off under the port's rules and left out of its file.
/// Island 7 to 1900 is the same island with them on and off, to the bit; on
/// from there to 1906, money is conserved every month, the cost graph has its
/// railway as a layer of its own from 1902, some square is reached off a
/// train, the weight passing each square is the traffic record's to a part
/// in a thousand million (the two walks are one walk, with the squares a train
/// passes laid in both).
#[test]
fn island_7_stands_to_1900_and_the_station_acts_after() {
    let port = default_params();
    assert!(port.haulage == HaulageParams::default() && !port.haulage.handling && !port.haulage.wear);
    assert!(!serde_json::to_string(&port).unwrap().contains("haulage"), "the port's file says nothing of haulage");
    assert_eq!(haulage::FROM, 1901);
    let (mut on, off) = std::thread::scope(|s| {
        let a = s.spawn(|| island(true, true, 1906));
        let b = s.spawn(|| island(false, false, 1906));
        (a.join().unwrap(), b.join().unwrap())
    });
    // to the end of 1900, the same island
    assert_eq!(on.st.series.data.len(), off.st.series.data.len());
    for (k, (a, b)) in on.st.series.data.iter().zip(&off.st.series.data).enumerate() {
        assert!(a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits()), "series {} differs to 1900", on.st.series.names[k]);
    }
    assert_eq!(on.st.road, off.st.road);
    assert!(!on.st.graph.has_train() && on.st.hl.by.is_empty(), "the graph made at the close of 1900 is the port's");
    // and on from there
    let mut seen = false;
    while on.year <= on.end_year {
        let year = on.year;
        sim::run_year(&mut on).expect("money is conserved every month, with the station");
        let st = &on.st;
        assert!(st.graph.has_train() && st.graph.handling == st_wharf(&on), "{year}: the railway is not a layer of its own");
        assert_eq!(st.hl.by.len(), st.T_max);
        seen |= st.hl.by.iter().any(|b| b.iter().any(|&x| x == BY_TRAIN));
        for c in (0..st.N).filter(|&c| st.world.land[c]) {
            let (a, b) = (st.tf.through[c], st.traffic[c]);
            assert!((a - b).abs() <= 1e-9 * b.abs().max(1.0), "{year}, square {c}: {a} against the roads' {b}");
        }
    }
    assert!(seen, "no load rode a train");
    let last = on.st.tf.years.last().unwrap();
    let by_haul: f64 = last.by_haul.iter().flatten().sum();
    let carried: f64 = last.weight_squares.iter().sum();
    assert!((by_haul - carried).abs() <= 1e-9 * carried, "the hauls by length add up to {by_haul}, the freight to {carried}");
    let stretches: f64 = last.rail_stretch.iter().sum();
    let rail = last.weight_squares[traffic::RAIL as usize];
    assert!((stretches - rail).abs() <= 1e-9 * rail.max(1.0), "the railway's stretches add up to {stretches}, its freight to {rail}");
}

fn st_wharf(s: &Sim) -> f64 {
    haulage::station(&s.params)
}
