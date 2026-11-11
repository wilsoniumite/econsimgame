//! Town ground that is kept (`docs/rules/towns.md`, not in the Python): switched
//! off it is the port (tests/golden.rs holds that to the byte); switched on,
//! money is conserved, a run is deterministic, a town keeps the ground it takes
//! and grows from it, the works on its ground stay, a new town is reckoned at
//! its donor's wage, and the record and the live view carry three more layers.
#![allow(non_snake_case)]

use isostate_core::mat::Mat;
use isostate_core::record::{BUILT_YEAR0, RES_UNIT, TOWN_LAYERS};
use isostate_core::towns::{self, TownsParams};
use isostate_core::works::WorksParams;
use isostate_core::*;

/// The port with kept town ground and the life cycle for works switched on:
/// the two are played together, and the first builds on the second.
fn params(seed: u64, n_families: usize, end_year: i64) -> Params {
    let mut P = default_params();
    P.seed = seed;
    P.n_families = n_families;
    P.end_year = Some(end_year);
    P.works = WorksParams::game();
    P.towns = TownsParams::game();
    P
}

/// The same without kept town ground.
fn without(seed: u64, n_families: usize, end_year: i64) -> Params {
    let mut P = params(seed, n_families, end_year);
    P.towns = TownsParams::default();
    P
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn town_cells(st: &State) -> usize {
    st.urban.iter().filter(|&&u| u).count()
}

/// The names of the arrays in a packed run.
fn arrays(blob: &[u8]) -> Vec<(String, Vec<u64>)> {
    let n = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    let header: serde_json::Value = serde_json::from_slice(&blob[4..4 + n]).expect("a JSON header");
    header["arrays"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| (a["name"].as_str().unwrap().to_string(), a["shape"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect()))
        .collect()
}

#[test]
fn off_is_the_port_and_on_is_the_game() {
    assert_eq!(default_params().towns, TownsParams::default());
    assert!(!default_params().towns.enabled, "the port's towns keep nothing");
    assert!(game_params().towns.enabled, "the game's do");
    // the numbers are the same either way, so --set towns.enabled=true measures the game's
    assert_eq!(TownsParams { enabled: false, ..TownsParams::game() }, TownsParams::default());
    // a port run never fills the state's tables, and its files have no new layer
    let mut P = default_params();
    P.seed = 3;
    P.end_year = Some(1752);
    let r = sim::run_with(&P, default_content(), false, true).expect("money is conserved");
    let st = &r.state;
    assert!(!towns::kept(st));
    assert!(st.town_built.is_empty() && st.town_room.is_empty() && st.town_ground.is_empty());
    assert!(st.town_peak.is_empty() && st.town_low_years.is_empty() && st.town_ground_rent.is_empty() && st.town_hands_acc.is_empty());
    assert!(r.rec.town_layers.is_empty() && r.rec.town_layer("built").is_none());
    assert!(record::town_layers(st).is_none());
    let blob = record::pack(st, &r.content, &r.params, &r.events, &r.rec, "port", false);
    let names: Vec<String> = arrays(&blob).into_iter().map(|(n, _)| n).collect();
    assert!(TOWN_LAYERS.iter().all(|l| !names.iter().any(|n| n == l)), "{names:?}");
    assert!(!serde_json::to_string(&r.params).unwrap().contains("\"towns\""), "params.json does not mention it");
}

#[test]
fn the_sums_of_kept_ground() {
    let C = default_content();
    let on = params(7, 0, 1750);
    let tp = &on.towns;
    let y0 = C.start_year;
    // a cell has room for d_new people the year it joins, and fills in toward the ceiling
    assert_eq!(towns::room(&C, &on, 1800, 1800), tp.d_new);
    assert_eq!(towns::ceiling(&C, &on, y0), tp.d_old);
    assert_eq!(towns::ceiling(&C, &on, y0 + 100), tp.d_old + 100.0 * tp.d_old_growth);
    let young = towns::room(&C, &on, 1800, 1790);
    let old = towns::room(&C, &on, 1800, 1700);
    assert!(tp.d_new < young && young < old && old < towns::ceiling(&C, &on, 1800));
    let at_tau = towns::room(&C, &on, y0, y0 - tp.tau as i64);
    assert!((at_tau - (tp.d_new + (tp.d_old - tp.d_new) * (1.0 - (-1.0f64).exp()))).abs() < 1e-9);
    // the same ground holds more people later: the ceiling rises
    assert!(towns::room(&C, &on, 1900, 1700) > towns::room(&C, &on, 1800, 1700));
    // works on a cell leave less room for people
    assert_eq!(towns::room_beside_works(&on, 0.0), 1.0);
    assert!((towns::room_beside_works(&on, 1.0) - tp.works_room).abs() < 1e-12);
    assert!((towns::room_beside_works(&on, 0.5) - (1.0 - 0.5 * (1.0 - tp.works_room))).abs() < 1e-12);
    // open land: one use bidding alone pays its bid on the share the land rule gives it
    let mut bids = Mat::zeros(C.NS, 2);
    assert_eq!(towns::open_rent(&on, &bids, 0.0, 0), 0.0, "nobody bids, nobody pays");
    bids[(0, 0)] = 100.0;
    assert!((towns::open_rent(&on, &bids, 0.0, 0) - 100.0).abs() < 1e-9);
    let shared = towns::open_rent(&on, &bids, 100.0, 0);
    assert!((shared - 50.0).abs() < 1e-9, "wild land bidding as much takes half and pays nothing: {shared}");
    bids[(1, 0)] = 100.0;
    assert!((towns::open_rent(&on, &bids, 0.0, 0) - 100.0).abs() < 1e-9, "two uses at the same bid pay it between them");
    assert_eq!(towns::open_rent(&on, &bids, 0.0, 1), 0.0);
    // under the plough: the farm, the pasture and the wood bid, the pits do not
    let plough = towns::plough_uses(&C);
    assert_eq!(plough.iter().map(|&s| C.sector_name[s].as_str()).collect::<Vec<_>>(), ["farm", "pasture", "forestry"]);
    let coal = C.sector_name.iter().position(|n| n == "coalmine").unwrap();
    bids[(coal, 0)] = 1e6;
    assert!(towns::open_rent(&on, &bids, 0.0, 0) > 1e5, "open land is let to the pit");
    assert!((towns::plough_rent(&on, &bids, 0.0, 0, &plough) - 100.0).abs() < 1e-9, "the plough's rent is not the pit's");
}

#[test]
fn a_town_cell_pays_at_least_the_ploughs_rent() {
    let P = params(7, 0, 1750);
    let C = default_content();
    let s = sim::start(&P, C.clone());
    let st = &s.st;
    let i = (0..st.T_max).find(|&i| st.town_active[i] && st.town_ground[i].len() >= 3).expect("a town");
    // its newest cell: the furthest out, where houses pay least
    let c = *st.town_ground[i].last().unwrap();
    let (pbar, wbar) = (st.pbar.clone(), st.wbar.clone());
    let bids = land::sector_bids(st, &C, &P, &pbar, &wbar);
    let wild = P.land_fixed_cost * wbar[i];
    let plough = towns::plough_uses(&C);
    let farm = C.sector_name.iter().position(|n| n == "farm").unwrap();
    let coal = C.sector_name.iter().position(|n| n == "coalmine").unwrap();
    let after = |bids: &Mat, people: f64| -> State {
        let mut st = st.clone();
        for v in [&mut st.pop[i], &mut st.ind_labor[i], &mut st.urban_pop[i]] {
            *v *= people;
        }
        towns::update_urban(&mut st, &C, &P, C.start_year, &wbar, bids, false);
        st
    };
    let plain = after(&bids, 1.0);
    // coal found under the town is nothing to its households, or to a works there
    let mut dear = bids.clone();
    dear[(coal, c)] = 1e7;
    let pit = after(&dear, 1.0);
    assert_eq!(pit.urban_rent[i].to_bits(), plain.urban_rent[i].to_bits());
    assert_eq!(pit.town_ground_rent[c].to_bits(), plain.town_ground_rent[c].to_bits());
    // good farm land is: the cell pays what the plough would, and the town's bill is that much more
    dear = bids.clone();
    dear[(farm, c)] = 1e7;
    let floor = towns::plough_rent(&P, &dear, wild, c, &plough);
    assert!(floor > 100.0 * plain.town_ground_rent[c]);
    let rich = after(&dear, 1.0);
    assert_eq!(rich.town_ground_rent[c], floor, "a works' yard there displaces the plough");
    let full = rich.urban_pop[i] / rich.town_ground[i].iter().map(|&d| rich.town_room[d]).sum::<f64>();
    assert!(full > 0.9 && full <= 1.0 + 1e-9, "the town fits its ground: {full}");
    assert!(rich.cell_urban_rent[c] > 0.9 * floor * towns::room_beside_works(&P, rich.cell_K_share[c]) * full.min(1.0));
    assert!((rich.urban_rent[i] - plain.urban_rent[i] - (rich.cell_urban_rent[c] - plain.cell_urban_rent[c])).abs() < 1e-6 * rich.urban_rent[i]);
    // in a town that has emptied, the few who stay do not pay the farm rent of ground nobody lives on
    let empty = after(&dear, 0.1);
    assert_eq!(empty.town_ground[i].len(), st.town_ground[i].len(), "the ground is kept");
    assert_eq!(empty.town_ground_rent[c], floor, "and costs a works what it did");
    assert!(empty.cell_urban_rent[c] < 0.15 * rich.cell_urban_rent[c], "{} against {}", empty.cell_urban_rent[c], rich.cell_urban_rent[c]);
    // without the floor the cell pays what its houses do, as under the port's rule
    let mut off = P.clone();
    off.towns.rent_floor = false;
    let mut bare = st.clone();
    towns::update_urban(&mut bare, &C, &off, C.start_year, &wbar, &dear, false);
    assert!(bare.town_ground_rent[c] < 0.01 * floor);
}

#[test]
fn a_town_with_nowhere_to_grow_crowds_the_ground_it_has() {
    let P = params(7, 0, 1750);
    let C = default_content();
    let mut s = sim::start(&P, C.clone());
    let st = &mut s.st;
    let i = (0..st.T_max).find(|&i| st.town_active[i]).expect("a town");
    let n = st.town_ground[i].len();
    let (pbar, wbar) = (st.pbar.clone(), st.wbar.clone());
    // the rest of its market area is lost to it, and its people grow sixfold
    for c in 0..st.N {
        if st.catch[c] == i as i64 && !st.urban[c] {
            st.catch[c] = -1;
        }
    }
    st.pop[i] *= 6.0;
    st.ind_labor[i] *= 6.0;
    for y in 0..8 {
        land::update_land_use(st, &C, &P, C.start_year + y, &pbar, &wbar, false);
    }
    assert_eq!(st.town_ground[i].len(), n, "there is nothing to take");
    let (room, res): (f64, f64) = st.town_ground[i].iter().map(|&c| (st.town_room[c], st.urban_res[c])).fold((0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));
    assert!(st.urban_pop[i] > 4.0 * room, "{} people in room for {room}", st.urban_pop[i]);
    assert!((res - st.urban_pop[i]).abs() < 1e-6 * res, "they all live somewhere");
    // the record's byte for residents stops at its top and does not wrap
    let layers = record::town_layers(st).expect("kept ground");
    let top = st.town_ground[i].iter().map(|&c| layers[1][c]).max().unwrap();
    assert_eq!(top, 255);
    assert!(st.town_ground[i].iter().all(|&c| st.urban_res[c] / RES_UNIT < 255.0 || layers[1][c] == 255));
}

/// What must hold of kept ground in any year.
fn check_ground(st: &State, C: &Content, P: &Params, year: i64) {
    let N = st.N;
    let mut owner = vec![-1i64; N];
    for i in 0..st.T_max {
        let list = &st.town_ground[i];
        if !st.town_active[i] {
            assert!(list.is_empty());
            continue;
        }
        assert_eq!(list[0], st.town_node[i], "{year}: {}'s first cell is its centre", st.town_name[i]);
        for &c in list {
            assert_eq!(owner[c], -1, "{year}: cell {c} is in two towns");
            owner[c] = i as i64;
            assert_eq!(st.catch[c], i as i64, "{year}: a town's ground is in its market area");
            assert!(st.world.land[c]);
        }
        let res: f64 = list.iter().map(|&c| st.urban_res[c]).sum();
        let room: f64 = list.iter().map(|&c| st.town_room[c]).sum();
        if room >= 1.0 {
            assert!((res - st.urban_pop[i]).abs() < 1e-6 * st.urban_pop[i].max(1.0), "{year}: every town person lives on a cell");
        }
        let bill: f64 = list.iter().map(|&c| st.cell_urban_rent[c]).sum();
        assert!((bill - st.urban_rent[i]).abs() < 1e-9 * bill.max(1.0), "{year}: the town's bill is its cells' rents");
    }
    for c in 0..N {
        assert_eq!(st.urban[c], owner[c] >= 0, "{year}: cell {c}");
        if st.urban[c] {
            assert!(st.town_built[c] > 1500 && st.town_built[c] <= year, "{year}: cell {c} built {}", st.town_built[c]);
            // (the works' share has moved on since the room was reckoned, so only the bounds are checked)
            let clear = towns::room(C, P, year, st.town_built[c]);
            assert!(st.town_room[c] <= clear + 1e-9 && st.town_room[c] >= P.towns.works_room * clear - 1e-9);
            assert!(st.land_use[c] == land::USE_URBAN);
            assert!(st.town_ground_rent[c] >= 0.0 && st.land_rent_only[c] == st.town_ground_rent[c]);
            assert!(st.cell_urban_rent[c] <= st.town_ground_rent[c] + 1e-9, "works on the cell pay for their part themselves");
        } else {
            assert_eq!((st.town_built[c], st.town_room[c], st.urban_res[c]), (0, 0.0, 0.0));
        }
    }
}

#[test]
fn a_town_keeps_its_ground_and_grows_from_it() {
    let P = params(7, 0, 1800);
    let C = default_content();
    let mut s = sim::start(&P, C.clone());
    let y0 = C.start_year;
    // 1750 opens on old ground: the first towns' cells are dated before the start, the centre oldest
    check_ground(&s.st, &C, &P, y0 - 1);
    for i in (0..s.st.T_max).filter(|&i| s.st.town_active[i]) {
        let list = &s.st.town_ground[i];
        assert!(list.len() >= 3, "{} stands on {} cells", s.st.town_name[i], list.len());
        assert!(list.iter().all(|&c| s.st.town_built[c] <= y0 - P.towns.back_years));
        assert_eq!(s.st.town_built[list[0]], y0 - P.towns.back_years - P.towns.back_span);
        // and the town fits its people, to within the little the dating moves: without
        // its newest cell it would be short of room
        let (room, U): (f64, f64) = (list.iter().map(|&c| s.st.town_room[c]).sum(), s.st.urban_pop[i]);
        assert!(room >= 0.97 * U && room - s.st.town_room[*list.last().unwrap()] < 1.03 * U, "{}: room for {room:.0}, {U:.0} people", s.st.town_name[i]);
    }
    let start = town_cells(&s.st);
    let mut before = s.st.urban.clone();
    let mut built = s.st.town_built.clone();
    let (mut left, mut joined, mut detached) = (0usize, 0usize, 0usize);
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved");
        let (st, year) = (&s.st, s.year - 1);
        check_ground(st, &C, &P, year);
        let W = st.world.W as i64;
        for c in 0..st.N {
            if before[c] && !st.urban[c] {
                left += 1;
            }
            if st.urban[c] && before[c] {
                assert_eq!(st.town_built[c], built[c], "{year}: ground keeps the year it was built");
            }
            if st.urban[c] && !before[c] {
                joined += 1;
                let new_town = st.town_founded.iter().zip(&st.town_node).any(|(&f, &n)| f == year && n == c);
                assert!(st.town_built[c] == year, "{year}: new ground is dated the year it is taken");
                // new ground touches the town's own (or is a new town's first cell; a pit grown round is the exception)
                let (y, x) = (c as i64 / W, c as i64 % W);
                let touches = (0..st.N).any(|d| {
                    d != c && st.urban[d] && st.catch[d] == st.catch[c] && (d as i64 / W - y).abs() <= 1 && (d as i64 % W - x).abs() <= 1
                });
                if !touches && !new_town {
                    detached += 1;
                }
            }
        }
        before = st.urban.clone();
        built = st.town_built.clone();
    }
    let end = town_cells(&s.st);
    assert!(end as f64 >= 1.4 * start as f64, "the towns grew from {start} cells to {end} in fifty years");
    assert!(joined >= end - start);
    assert!(left * 20 <= joined, "{left} cells left town ground while {joined} joined: ground is kept");
    assert!(detached * 20 <= joined, "{detached} of {joined} new cells did not touch their town");
    // the port's towns stand on about as many cells in 1800 as in 1750
    let port = sim::run_with(&without(7, 0, 1800), default_content(), false, true).expect("money is conserved");
    assert!(end as f64 > 1.5 * town_cells(&port.state) as f64, "{end} cells against the port's {}", town_cells(&port.state));
    // and works stand on town ground with people beside them
    let st = &s.st;
    let mixed = (0..st.N).filter(|&c| st.urban[c] && st.cell_K_share[c] > 0.1 && st.urban_res[c] > 0.0).count();
    assert!(mixed > 0, "some town cells hold works and people both");
    let (mut hands, mut on_town) = (0.0, 0.0);
    for f in (0..st.F_count).filter(|&f| st.f_alive[f]) {
        hands += st.f_labor[f];
        on_town += if st.urban[st.f_cell[f]] { st.f_labor[f] } else { 0.0 };
    }
    assert!(on_town > 0.3 * hands, "{on_town:.0} of {hands:.0} hands work on town ground");
}

#[test]
fn a_long_deep_decline_gives_ground_up_one_cell_a_year() {
    let P = params(7, 0, 1750);
    let C = default_content();
    let mut s = sim::start(&P, C.clone());
    let st = &mut s.st;
    let i = (0..st.T_max).find(|&i| st.town_active[i]).expect("a town");
    let n = st.town_ground[i].len();
    let (pbar, wbar) = (st.pbar.clone(), st.wbar.clone());
    // the town loses half its people and its works' hands
    st.pop[i] *= 0.5;
    st.ind_labor[i] *= 0.5;
    let mut year = C.start_year;
    for k in 0..P.towns.decline_years {
        // smoothing takes a year or two to show the fall, so it holds a little longer than the count says
        land::update_land_use(st, &C, &P, year, &pbar, &wbar, false);
        year += 1;
        assert_eq!(st.town_ground[i].len(), n, "year {k}: the ground is kept through the first {} lean years", P.towns.decline_years);
    }
    let mut sizes = Vec::new();
    for _ in 0..6 {
        land::update_land_use(st, &C, &P, year, &pbar, &wbar, false);
        year += 1;
        sizes.push(st.town_ground[i].len());
    }
    assert!(sizes[5] < n, "after the lean years it lets ground go: {sizes:?}");
    assert!(sizes.windows(2).all(|w| w[0] <= w[1] + 1), "one cell a year at the most: {sizes:?}");
    assert_eq!(st.town_ground[i][0], st.town_node[i], "and never its centre");
    // it stops when the people it has are no longer far under the most its ground has held
    for _ in 0..40 {
        land::update_land_use(st, &C, &P, year, &pbar, &wbar, false);
        year += 1;
    }
    let settled = st.town_ground[i].len();
    assert!(settled >= n / 3 && settled < n, "{settled} of {n} cells are left");
    // a town that only stands still gives nothing up, however much its old ground fills in
    let mut s = sim::start(&P, C.clone());
    let st = &mut s.st;
    for y in 0..60 {
        land::update_land_use(st, &C, &P, C.start_year + y, &pbar, &wbar, false);
    }
    assert!(st.town_ground[i].len() >= n);
}

#[test]
fn a_new_town_is_reckoned_at_its_donors_wage() {
    // seed 7 founds its seventh town within a few years of 1750
    let found = |P: &Params| -> (State, usize, i64) {
        let mut s = sim::start(P, default_content());
        while s.year <= s.end_year {
            sim::run_year(&mut s).expect("money is conserved");
            let year = s.year - 1;
            if let Some(n) = (0..s.st.T_max).find(|&n| s.st.town_active[n] && s.st.town_founded[n] == year) {
                return (s.st, n, year);
            }
        }
        panic!("no town was founded by {}", P.end_year.unwrap());
    };
    let on = params(7, 0, 1800);
    let (st, n, year) = found(&on);
    assert!(st.wbar[n] > 1.0, "{} of {year} has a wage for the year: {}", st.town_name[n], st.wbar[n]);
    let donor = (0..st.T_max).find(|&d| d != n && st.wbar[d] == st.wbar[n]).expect("it is some other town's wage");
    for g in 0..st.pbar.cols {
        assert_eq!(st.pbar[(n, g)], st.pbar[(donor, g)], "and that town's prices");
    }
    let dearest = (0..st.N).filter(|&c| st.catch[c] == n as i64).map(|c| st.land_rent_only[c]).fold(0.0, f64::max);
    assert!(dearest < 1e5, "the land round it is priced like land: {dearest:e}");
    // without the repair the wage is nothing and the land round the new town is priced accordingly
    let mut off = on.clone();
    off.towns.donor_wage = false;
    let (st, n, _) = found(&off);
    assert_eq!(st.wbar[n], 0.0);
    let dearest = (0..st.N).filter(|&c| st.catch[c] == n as i64 && !st.urban[c]).map(|c| st.land_rent_only[c]).fold(0.0, f64::max);
    assert!(dearest > 1e7, "{dearest:e}");
}

#[test]
fn town_people_follow_the_years_mean_of_hands() {
    let P = params(3, 0, 1751);
    let mut s = sim::start(&P, default_content());
    let T = s.st.T_max;
    let mut sum = vec![0.0; T];
    for month in 0..12 {
        sim::run_month(&mut s, month).expect("money is conserved");
        for f in (0..s.st.F_count).filter(|&f| s.st.f_alive[f]) {
            sum[s.st.f_town[f]] += s.st.f_labor[f];
        }
    }
    let december: Vec<f64> = (0..T).map(|t| (0..s.st.F_count).filter(|&f| s.st.f_alive[f] && s.st.f_town[f] == t).map(|f| s.st.f_labor[f]).sum()).collect();
    let year = s.year;
    let Sim { st, content: C, params, rngs, events, .. } = &mut s;
    sim::yearly(st, C, params, rngs, year, events, false);
    let mut differs = false;
    for t in (0..T).filter(|&t| sum[t] > 0.0) {
        assert!((st.ind_labor[t] - sum[t] / 12.0).abs() < 1e-9 * sum[t], "town {t}");
        differs |= (st.ind_labor[t] - december[t]).abs() > 1e-6 * december[t];
    }
    assert!(differs, "the mean is not December's count");
    assert!(st.town_hands_acc.iter().all(|&v| v == 0.0), "and the next year starts from nothing");
}

#[test]
fn the_record_and_the_live_view_carry_the_three_layers() {
    let P = params(3, 6, 1756);
    let C = default_content();
    let mut s = sim::start(&P, C.clone());
    let stat = live_static(&s.st, &C);
    assert_eq!(stat.layers.len(), 12);
    assert_eq!(&stat.layers[9..], &TOWN_LAYERS.map(|l| l.to_string()));
    let mut cursor = LiveCursor::new(&C);
    let N = s.st.N;
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved");
        let y = live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        let st = &s.st;
        let (built, res, works) = (y.layers.built.expect("built"), y.layers.res.expect("res"), y.layers.works.expect("works"));
        for c in 0..N {
            assert_eq!(built[c] > 0, st.urban[c]);
            if st.urban[c] {
                assert_eq!(built[c] as i64 + BUILT_YEAR0, st.town_built[c], "{}: cell {c}", y.year);
                assert!((res[c] as f64 * RES_UNIT - st.urban_res[c]).abs() <= RES_UNIT / 2.0 + 1e-9);
            } else {
                assert_eq!(res[c], 0);
            }
            assert!((works[c] as f64 / 255.0 - st.cell_K_share[c].min(1.0)).abs() <= 0.5 / 255.0 + 1e-9);
        }
        // the same bytes as the record keeps for run.bin
        for (name, live) in TOWN_LAYERS.iter().zip([&built, &res, &works]) {
            assert_eq!(&live[..], &s.rec.town_layer(name).unwrap()[y.index * N..(y.index + 1) * N]);
        }
        assert!(works.iter().zip(&built).any(|(&w, &b)| w > 0 && b > 0), "works on town ground show in both");
    }
    let r = sim::finish(s);
    let blob = record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "towns", false);
    let arrs = arrays(&blob);
    for l in TOWN_LAYERS {
        let (_, shape) = arrs.iter().find(|(n, _)| n == l).unwrap_or_else(|| panic!("no array {l}"));
        assert_eq!(shape, &vec![r.rec.n_years as u64, N as u64]);
    }
    // the comparison scenario shown in charts only keeps no map layers, these included
    let lite = record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "towns", true);
    assert!(arrays(&lite).iter().all(|(n, _)| !TOWN_LAYERS.contains(&n.as_str())));
}

#[test]
fn kept_ground_conserves_money_and_is_deterministic() {
    // six families for forty years; run_with checks the money stock every month
    let P = params(3, 6, 1789);
    let a = sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
    let b = sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (x, y) in a.state.series.data.iter().zip(&b.state.series.data) {
        assert_eq!(bits(x), bits(y), "the same parameters give the same run");
    }
    assert_eq!(a.state.town_built, b.state.town_built);
    assert_eq!(a.state.town_ground, b.state.town_ground);
    assert_eq!(bits(&a.state.urban_res), bits(&b.state.urban_res));
    assert_eq!(a.rec.town_layers, b.rec.town_layers);
    // and alone, without the life cycle for works or families, a little longer
    let mut P = params(5, 0, 1809);
    P.works = WorksParams::default();
    sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
}
