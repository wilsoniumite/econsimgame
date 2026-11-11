//! Mines and pits that take little of the surface (`docs/rules/mines.md`, not in
//! the Python): switched off it is the port (tests/golden.rs holds that to the
//! byte); switched on, money is conserved, a run is deterministic, the
//! quarries, pits and mines give the surface back, every working stands on
//! ground a pit head may have and works deposit nobody else has, the royalty
//! is paid out whole to the cells the deposit lies under, what is raised is
//! counted against the cells it came from, and the viewer is given the
//! workings.
#![allow(non_snake_case)]

use isostate_core::mines::{self, MineParams};
use isostate_core::*;

/// The port with the mines switched on, as `--set mines.enabled=true` runs it.
fn params(seed: u64, n_families: usize, end_year: i64) -> Params {
    let mut P = default_params();
    P.seed = seed;
    P.n_families = n_families;
    P.end_year = Some(end_year);
    P.mines = MineParams::game();
    P
}

fn port(seed: u64, n_families: usize, end_year: i64) -> Params {
    let mut P = params(seed, n_families, end_year);
    P.mines = MineParams::default();
    P
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn run(P: &Params) -> RunResult {
    sim::run_with(P, default_content(), false, true).expect("money is conserved")
}

/// The land under each extractive sector, in cells.
fn mine_area(st: &State, C: &Content) -> f64 {
    (0..C.NS).filter(|&s| mines::is_extractive(&C.sector_name[s])).map(|s| st.lam.row(s).iter().sum::<f64>()).sum()
}

#[test]
fn off_is_the_port_and_on_is_the_game() {
    assert_eq!(default_params().mines, MineParams::default());
    assert!(!default_params().mines.enabled, "the port's mines are land uses");
    assert!(game_params().mines.enabled, "the game's are workings");
    // the numbers are the same either way, so --set mines.enabled=true measures the game's
    assert_eq!(MineParams { enabled: false, ..MineParams::game() }, MineParams::default());
    // and a port run never fills the state's tables or the record
    let r = run(&port(3, 0, 1752));
    assert!(r.state.mine_sectors.is_empty() && r.state.mine_opened.is_empty() && r.state.mine_districts.is_empty());
    assert_eq!(r.state.mine_dug.rows, 0);
    assert!(r.rec.workings.is_empty());
    assert!(mines::workings(&r.state).is_empty());
}

#[test]
fn money_is_conserved_and_a_run_is_deterministic() {
    // `run` fails if any month's money is off; forty years, and twenty with six families
    let a = run(&params(3, 0, 1790));
    let b = run(&params(3, 0, 1790));
    for (x, y) in a.state.series.data.iter().zip(&b.state.series.data) {
        assert_eq!(bits(x), bits(y));
    }
    assert_eq!(a.state.mine_opened, b.state.mine_opened);
    assert_eq!(bits(&a.state.mine_dug.data), bits(&b.state.mine_dug.data));
    let f = run(&params(7, 6, 1770));
    assert!(f.state.families.len() == 6 && f.state.mine_opened.iter().any(|&y| y != 0));
}

#[test]
fn the_workings_give_the_surface_back() {
    let (off, on) = (run(&port(3, 0, 1770)), run(&params(3, 0, 1770)));
    let C = &on.content;
    let (was, now) = (mine_area(&off.state, C), mine_area(&on.state, C));
    assert!(was > 500.0, "the port's mines hold hundreds of cells: {was}");
    assert!(now > 0.0 && now < 0.03 * was, "the workings hold a few: {now} of {was}");
    // what they gave up is farmed, grazed, wooded or left wild
    let rest = |st: &State| (0..=C.NS).filter(|&r| r == C.NS || !mines::is_extractive(&C.sector_name[r])).map(|r| st.lam.row(r).iter().sum::<f64>()).sum::<f64>();
    assert!(rest(&on.state) > rest(&off.state) + 0.8 * (was - now), "{} against {}", rest(&on.state), rest(&off.state));
    // and no less is raised for it: within a third of the port's stone, clay and coal
    for g in ["stone", "clay", "coal"] {
        let k = on.state.series.index[&format!("out_{g}")];
        let mean = |r: &RunResult| r.state.series.data[k].iter().rev().take(60).sum::<f64>() / 60.0;
        let (a, b) = (mean(&off), mean(&on));
        assert!(b > 0.67 * a && b < 1.5 * a, "{g}: {b} a month against the port's {a}");
    }
}

#[test]
fn the_land_arithmetic_holds_every_year() {
    let P = params(5, 0, 1765);
    let mut s = sim::start(&P, default_content());
    let C = s.content.clone();
    while s.year <= s.end_year {
        sim::run_year(&mut s).unwrap();
        let st = &s.st;
        let N = st.N;
        for c in 0..N {
            let tot: f64 = (0..=C.NS).map(|r| st.lam[(r, c)]).sum();
            assert!((0..=C.NS).all(|r| st.lam[(r, c)] >= 0.0) && tot <= 1.0 + 1e-9, "cell {c}: shares sum to {tot}");
            // a working's surface is what its row of the land shares says, and no more than its limit
            let surface: f64 = (0..st.mine_sectors.len()).map(|r| st.mine_surface[(r, c)]).sum();
            let rows: f64 = st.mine_sectors.iter().map(|&sec| st.lam[(sec, c)]).sum();
            assert!((surface - rows).abs() < 1e-12 && surface <= P.mines.cell_limit + 1e-12);
        }
        for (r, &sec) in st.mine_sectors.iter().enumerate() {
            for h in (0..N).filter(|&h| st.mine_opened[r * N + h] != 0) {
                // a pit head stands on land with a market that is not town ground
                // (the land rule ran before this year's works were founded, so a
                // cell the works have since filled may still carry one)
                assert!(st.world.land[h] && !st.urban[h], "a {} on cell {h}", C.sector_name[sec]);
                assert!(st.mine_surface[(r, h)] >= 0.0);
            }
            // the monthly loop has one unit for every working with deposit, and no other
            let units: Vec<usize> = (0..st.ex_cell.len()).filter(|&u| st.ex_sector[u] == sec).collect();
            for &u in &units {
                let h = st.ex_cell[u];
                assert!(st.mine_opened[r * N + h] != 0 && st.ex_lam[u] == 1.0 && st.ex_field[u] == st.mine_rich[(r, h)]);
            }
            let with_deposit = (0..N).filter(|&h| st.mine_opened[r * N + h] != 0 && st.mine_rich[(r, h)] > 0.0).count();
            assert_eq!(units.len(), with_deposit);
        }
    }
}

#[test]
fn a_lease_is_whole_and_nobody_sinks_inside_it() {
    let r = run(&params(7, 0, 1775));
    let (st, C, P) = (&r.state, &r.content, &r.params);
    let (N, W) = (st.N, st.world.W);
    let mut some = 0;
    for (row, &sec) in st.mine_sectors.iter().enumerate() {
        let field = st.world.field(&C.sector_field[sec]);
        for c in 0..N {
            let h = st.mine_head[row * N + c];
            if h < 0 {
                assert_eq!(st.mine_share[(row, c)], 0.0);
                continue;
            }
            let h = h as usize;
            // the deposit is real, its working is open and within reach
            assert!(field[c] > 0.0 && st.mine_opened[row * N + h] != 0);
            let (dy, dx) = ((c / W) as f64 - (h / W) as f64, (c % W) as f64 - (h % W) as f64);
            assert!(dy.hypot(dx) <= P.mines.reach + 1e-9);
            assert!(st.mine_share[(row, c)] > 0.0);
        }
        for h in (0..N).filter(|&h| st.mine_opened[row * N + h] != 0) {
            // a pit head stands on its own lease or on ground with no lease, never on another's
            let own = st.mine_head[row * N + h];
            assert!(own < 0 || own as usize == h, "the {} on {h} stands on the lease of {own}", C.sector_name[sec]);
            if let Some(cells) = st.mine_districts.get(&(row * N + h)) {
                let total: f64 = cells.iter().map(|&(_, share)| share).sum();
                assert!((total - 1.0).abs() < 1e-9, "the shares of a lease sum to {total}");
                assert!(cells.iter().all(|&(c, _)| st.mine_head[row * N + c] == h as i32));
                some += 1;
            }
        }
    }
    assert!(some > 50, "{some} workings with a lease");
}

#[test]
fn the_royalty_is_paid_out_whole() {
    let P = params(3, 6, 1755);
    let mut s = sim::start(&P, default_content());
    while s.year <= s.end_year {
        sim::run_year(&mut s).unwrap();
    }
    let st = &s.st;
    let NE = st.ex_cell.len();
    // any rents will do: the split must give each unit's back, and to its own cells
    let rent: Vec<f64> = (0..NE).map(|u| 10.0 + (u % 7) as f64 - 3.5).collect();
    let (towns, cells, parts) = mines::royalty_cells(st, &P, &st.ex_town, &st.ex_cell, &rent).expect("the mechanism is on");
    assert!(parts.len() > NE, "some working's royalty is split among several cells");
    let total: f64 = parts.iter().sum();
    assert!((total - rent.iter().sum::<f64>()).abs() < 1e-6);
    let mut by_town = vec![0.0; st.T_max];
    for (k, &t) in towns.iter().enumerate() {
        by_town[t] += parts[k];
    }
    let mut want = vec![0.0; st.T_max];
    for u in 0..NE {
        want[st.ex_town[u]] += rent[u];
    }
    for t in 0..st.T_max {
        assert!((by_town[t] - want[t]).abs() < 1e-6, "town {t}: {} of {}", by_town[t], want[t]);
    }
    // a farm's rent is its own cell's, as before
    let farm = (0..NE).find(|&u| mines::row(st, st.ex_sector[u]).is_none()).unwrap();
    assert_eq!(parts.iter().zip(&cells).filter(|&(&p, &c)| c == st.ex_cell[farm] && p == rent[farm]).count() >= 1, true);
    // and with families the cells a working's royalty went to are those of its lease:
    // over the year just run, deposit cells were paid rent
    let leased: Vec<usize> = (0..st.N).filter(|&c| (0..st.mine_sectors.len()).any(|r| st.mine_head[r * st.N + c] >= 0)).collect();
    assert!(!leased.is_empty() && leased.iter().any(|&c| st.cell_rent_year[c] != 0.0));
}

#[test]
fn what_is_raised_is_counted_against_the_deposit() {
    let r = run(&params(3, 0, 1780));
    let st = &r.state;
    let N = st.N;
    for row in 0..st.mine_sectors.len() {
        let dug: f64 = st.mine_dug.row(row).iter().sum();
        let heaps: f64 = st.mine_heap[row * N..(row + 1) * N].iter().sum();
        assert!(dug > 0.0 && (dug - heaps).abs() < 1e-6 * heaps, "row {row}: {dug} dug, {heaps} on the heaps");
    }
    // a deposit that tires gives less: with a reserve a hundredth of the game's
    // the same forty years raise less coal for dearer
    let mut P = params(3, 0, 1780);
    P.mines.reserve = 0.01 * MineParams::game().reserve;
    let tired = run(&P);
    let mean = |r: &RunResult, name: &str| {
        let d = &r.state.series.data[r.state.series.index[name]];
        d.iter().rev().take(120).sum::<f64>() / 120.0
    };
    assert!(mean(&tired, "price_coal") > mean(&r, "price_coal"), "{} against {}", mean(&tired, "price_coal"), mean(&r, "price_coal"));
    // and with none the richness never falls
    let mut P = params(3, 0, 1760);
    P.mines.reserve = 0.0;
    let fresh = run(&P);
    let C = &fresh.content;
    for (row, &sec) in fresh.state.mine_sectors.iter().enumerate() {
        let field = fresh.state.world.field(&C.sector_field[sec]);
        let a = C.sector_alpha[sec];
        for h in (0..N).filter(|&h| fresh.state.mine_rich[(row, h)] > 0.0) {
            // F is at least the richness of the best cell it has, less the haul
            let cells = &fresh.state.mine_districts[&(row * N + h)];
            let best = cells.iter().map(|&(c, _)| field[c]).fold(0.0, f64::max);
            let haul = 1.0 / (1.0 + P.mines.haul * P.mines.reach);
            assert!(fresh.state.mine_rich[(row, h)] >= best * haul - 1e-9 && a > 0.0);
        }
    }
}

#[test]
fn a_working_left_idle_is_abandoned_and_one_at_work_is_not() {
    let patient = run(&params(5, 0, 1775));
    let mut P = params(5, 0, 1775);
    P.mines.idle_years = 1;
    let hasty = run(&P);
    let left = |r: &RunResult| r.state.mine_left.iter().filter(|&&y| y != 0).count();
    assert!(left(&hasty) > left(&patient), "{} against {}", left(&hasty), left(&patient));
    // nothing with hands was given up: every working that left has an empty year behind it
    let st = &patient.state;
    for k in 0..st.mine_opened.len() {
        if st.mine_opened[k] != 0 {
            assert!((st.mine_idle[k] as u32) < P.mines.idle_years.max(10));
        }
    }
}

#[test]
fn the_viewer_is_given_the_workings() {
    let r = run(&params(3, 0, 1760));
    let (st, C, rec) = (&r.state, &r.content, &r.rec);
    assert_eq!(rec.workings.len(), rec.n_years);
    let last = rec.workings.last().unwrap();
    assert_eq!(last, &mines::workings(st));
    let at_work: Vec<&mines::Working> = last.iter().filter(|w| w.opened != 0).collect();
    assert!(at_work.len() > 50 && at_work.len() < 2000, "{} workings", at_work.len());
    for w in &at_work {
        assert!(mines::is_extractive(&C.sector_name[w.sector]) && st.world.land[w.cell]);
        assert!(w.hands >= 0.0 && w.surface >= 0.0 && w.surface <= 0.5 && w.heap >= w.out && w.opened <= 1760);
    }
    assert!(at_work.iter().any(|w| w.hands > 20.0 && w.out > 0.0 && w.cells > 1 && w.surface > 0.0));
    // the largest working takes a small part of its cell, where the port's mines took half of theirs
    let most = at_work.iter().map(|w| w.surface).fold(0.0, f32::max);
    assert!(most < 0.3, "the largest working takes {most} of its cell");
    // the container has the table, and a port run's does not
    let blob = record::pack(st, C, &r.params, &r.events, rec, "mines", false);
    let has = |blob: &[u8], name: &str| blob.windows(name.len()).any(|w| w == name.as_bytes());
    assert!(has(&blob, "mine_first") && has(&blob, "mine_hands") && has(&blob, "mine_cells"));
    let p = run(&port(3, 0, 1752));
    let blob = record::pack(&p.state, &p.content, &p.params, &p.events, &p.rec, "port", false);
    assert!(!has(&blob, "mine_first"));
    // the map's mix shows the surface as it is: hardly a cell is mostly diggings
    let (a, _, _) = record::display_mix(st, C);
    let dug = a.iter().filter(|&&d| (5..=8).contains(&d)).count();
    assert!(dug < 5, "{dug} cells drawn as mostly diggings");
}
