//! The machines mechanism (docs/rules/machines.md): off it leaves no trace, on it
//! buys machines where they pay, lets them improve with the steel poured, starts
//! a new method from its predecessor's skill, and keeps the books straight.
#![allow(non_snake_case)]

use isostate_core::content::method_index;
use isostate_core::machines::{self, MachineParams};
use isostate_core::mat::Mat;
use isostate_core::sim::{self, RunResult, Sim};
use isostate_core::{Content, Params, default_content, default_params, econ, firms, game_content, game_params};

fn game(seed: u64, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.end_year = Some(end_year);
    P
}

/// A game at January 1750, and its first active town.
fn at_start() -> (Sim, usize) {
    let s = sim::start(&game(3, 1751), game_content());
    let town = (0..s.st.T_max).find(|&t| s.st.town_active[t]).expect("a town");
    (s, town)
}

fn sector(C: &Content, name: &str) -> usize {
    C.sector_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("no sector {name}"))
}

/// The town's prices with machinery at `p`.
fn prices_with_machinery(s: &Sim, town: usize, p: f64) -> Mat {
    let mut price = s.st.price.clone();
    price[(town, s.content.gi("machinery"))] = p;
    price
}

#[test]
fn off_by_default_and_on_in_the_game() {
    assert!(!default_params().machines.enabled);
    assert_eq!(default_params().machines, MachineParams::default());
    assert!(game_params().machines.enabled);
    // a port run keeps no machines and its machines never improve
    let mut P = default_params();
    P.seed = 3;
    P.end_year = Some(1753);
    let r = sim::run_with(&P, default_content(), false, true).expect("money conserved");
    assert_eq!(r.state.machine_stock.rows, 0, "the port keeps no stock of machines");
    assert_eq!(machines::quality(&r.state, &r.content, &r.params), 1.0);
}

#[test]
fn a_farm_buys_machinery_only_where_it_pays() {
    let (mut s, town) = at_start();
    let (C, P) = (s.content.clone(), s.params.clone());
    let farm = sector(&C, "farm");
    let gm = C.gi("machinery");
    s.st.know[(town, s.st.t_fmach)] = 1.0;
    s.st.method_Q.iter_mut().for_each(|q| *q = 0.0); // the machines of 1750
    s.st.machine_stock = Mat::default(); // no month counted yet: the machines standing are those kept up
    let grain = s.st.price[(town, C.sector_good[farm])];
    // the machine's gain on a unit of grain, with no other machine on the farm
    let gain = grain * P.farm_mach_boost / (1.0 + P.farm_mach_boost);
    let at = |cost_over_gain: f64| {
        let price = prices_with_machinery(&s, town, cost_over_gain * gain / P.farm_mach_input);
        let u = machines::uses(&s.st, &C, &P, farm, town, &price);
        assert_eq!(u.len(), 1, "a farm has one machine");
        (u[0], econ::sector_tfp(&s.st, &C, &P, farm, town, &price, &s.st.fill))
    };
    let span = P.machines.adopt_span;
    // dearer than it earns: nobody buys, and the farm asks the market for none
    let (u, (_, mach, _, unit_cost)) = at(1.5);
    assert_eq!(u.share, 0.0);
    assert_eq!((mach, unit_cost), (0.0, 0.0));
    // earning its cost 1 + span times over: everyone who knows how
    let (u, (_, mach, _, _)) = at(1.0 / (1.0 + span));
    assert!((u.share - 1.0).abs() < 1e-9, "share {}", u.share);
    assert!((mach - P.farm_mach_input).abs() < 1e-12);
    // half way between: half of the farms. Each buys for its own output, which
    // is 1 + boost times an unmechanised farm's, while the town's average farm
    // makes 1 + boost / 2: so much machinery for each unit the town grows
    let (u, (mult, mach, _, _)) = at(1.0 / (1.0 + 0.5 * span));
    assert!((u.share - 0.5).abs() < 1e-9, "share {}", u.share);
    let (equipped, average) = (1.0 + P.farm_mach_boost, 1.0 + 0.5 * P.farm_mach_boost);
    assert!((mach - 0.5 * P.farm_mach_input * equipped / average).abs() < 1e-12);
    // which is the same machinery whether it is counted on the half that buys or on the whole town
    let (_, (mult_all, mach_all, _, _)) = at(1.0 / (1.0 + span));
    assert!((mach * mult - 0.5 * mach_all * mult_all).abs() < 1e-12);
    assert_eq!(u.good, gm);
    // the port buys it in full whatever it costs
    let mut port = P.clone();
    port.machines = MachineParams::default();
    let dear = prices_with_machinery(&s, town, 1.5 * gain / P.farm_mach_input);
    let (_, mach, _, _) = econ::sector_tfp(&s.st, &C, &port, farm, town, &dear, &s.st.fill);
    assert_eq!(mach, P.farm_mach_input);
    // and with half the know-how, however cheap the machine, half the farms at most
    s.st.know[(town, s.st.t_fmach)] = 0.5;
    let cheap = prices_with_machinery(&s, town, 1e-6);
    let (_, mach, _, _) = econ::sector_tfp(&s.st, &C, &P, farm, town, &cheap, &s.st.fill);
    assert!((mach - 0.5 * P.farm_mach_input * equipped / average).abs() < 1e-12);
}

#[test]
fn machines_stand_for_years() {
    let (mut s, town) = at_start();
    let (C, P) = (s.content.clone(), s.params.clone());
    let farm = sector(&C, "farm");
    let gm = C.gi("machinery");
    let column = farm * machines::SLOTS;
    s.st.method_Q.iter_mut().for_each(|q| *q = 0.0); // the machines of 1750
    s.st.know[(town, s.st.t_fmach)] = 1.0;
    s.st.pexp[(town, gm)] = 1e-6; // so cheap that every farm keeps one up
    s.st.machine_stock = Mat::default();
    // the first month counted starts from what is kept up and was supplied
    s.st.fill[(town, gm)] = 0.5;
    machines::end_month(&mut s.st, &C, &P);
    assert_eq!(s.st.machine_stock[(town, column)], 0.5);
    // after that a month moves the machines standing one part in life_months
    s.st.fill[(town, gm)] = 1.0;
    machines::end_month(&mut s.st, &C, &P);
    let standing = 0.5 + 0.5 / P.machines.life_months;
    assert!((s.st.machine_stock[(town, column)] - standing).abs() < 1e-12);
    // what the machines add is what stands, not what is being bought
    let price = s.st.pexp.clone();
    let (with, mach, _, _) = econ::sector_tfp(&s.st, &C, &P, farm, town, &price, &s.st.fill);
    s.st.machine_stock[(town, column)] = 0.0;
    let (without, ..) = econ::sector_tfp(&s.st, &C, &P, farm, town, &price, &s.st.fill);
    assert!((with / without - (1.0 + P.farm_mach_boost * standing)).abs() < 1e-12);
    // while every farm buys for a farm fully equipped
    assert!((mach - P.farm_mach_input * (1.0 + P.farm_mach_boost) / (1.0 + P.farm_mach_boost * standing)).abs() < 1e-12);
    // a month with nothing delivered lets them run down as slowly
    s.st.machine_stock[(town, column)] = standing;
    s.st.fill[(town, gm)] = 0.0;
    machines::end_month(&mut s.st, &C, &P);
    assert!((s.st.machine_stock[(town, column)] - standing * (1.0 - 1.0 / P.machines.life_months)).abs() < 1e-12);
    // a town not yet founded has none, and starts from what its settlers keep up
    if let Some(new) = (0..s.st.T_max).find(|&t| !s.st.town_active[t]) {
        assert_eq!(s.st.machine_stock[(new, column)], -1.0);
        s.st.town_active[new] = true;
        s.st.know[(new, s.st.t_fmach)] = 0.5;
        for g in 0..C.G {
            s.st.pexp[(new, g)] = s.st.pexp[(town, g)];
        }
        s.st.fill[(new, gm)] = 1.0;
        machines::end_month(&mut s.st, &C, &P);
        assert_eq!(s.st.machine_stock[(new, column)], 0.5);
    }
    // switched off, nothing is kept
    let mut port = P.clone();
    port.machines = MachineParams::default();
    let before = s.st.machine_stock.clone();
    machines::end_month(&mut s.st, &C, &port);
    assert_eq!(s.st.machine_stock, before);
}

#[test]
fn mines_buy_as_the_port_does_unless_asked() {
    let (mut s, town) = at_start();
    let (C, mut P) = (s.content.clone(), s.params.clone());
    let coal = sector(&C, "coalmine");
    s.st.know[(town, s.st.t_pump)] = 1.0;
    s.st.know[(town, s.st.t_wagon)] = 1.0;
    let dear = prices_with_machinery(&s, town, 1e9);
    let (_, mach, rails, _) = econ::sector_tfp(&s.st, &C, &P, coal, town, &dear, &s.st.fill);
    assert_eq!((mach, rails), (P.pump_input, P.wagon_input));
    P.machines.adopt_pumps = true;
    let (_, mach, rails, _) = econ::sector_tfp(&s.st, &C, &P, coal, town, &dear, &s.st.fill);
    assert_eq!((mach, rails), (0.0, P.wagon_input));
}

#[test]
fn machines_improve_with_the_steel_poured_and_reach_the_quarries() {
    let (mut s, town) = at_start();
    let (C, P) = (s.content.clone(), s.params.clone());
    let M = &P.machines;
    let (crucible, bessemer) = (method_index(&C, "crucible_steel"), method_index(&C, "bessemer"));
    let (quarry, coal) = (sector(&C, "quarry"), sector(&C, "coalmine"));
    s.st.know[(town, s.st.t_pump)] = 1.0;
    s.st.know[(town, s.st.t_wagon)] = 0.0;
    // before the first month is counted the machines standing are those kept up
    s.st.machine_stock = Mat::default();
    let cheap = prices_with_machinery(&s, town, 1e-6);
    // no steel yet: the engines of 1750, and none in a quarry
    s.st.method_Q.iter_mut().for_each(|q| *q = 0.0);
    assert_eq!(machines::quality(&s.st, &C, &P), 1.0);
    assert!(machines::uses(&s.st, &C, &P, quarry, town, &cheap).is_empty());
    let (mult_1750, ..) = econ::sector_tfp(&s.st, &C, &P, coal, town, &cheap, &s.st.fill);
    // steel by either method counts alike
    s.st.method_Q[crucible] = M.quality_Q0;
    s.st.method_Q[bessemer] = 2.0 * M.quality_Q0;
    let q = machines::quality(&s.st, &C, &P);
    assert!((q - 4f64.powf(M.quality_b)).abs() < 1e-12, "quality {q}");
    let u = machines::uses(&s.st, &C, &P, quarry, town, &cheap);
    assert_eq!(u.len(), 1);
    assert_eq!(u[0].kind, "quarry machinery");
    assert!((u[0].boost - M.quarry_reach * (q - 1.0)).abs() < 1e-12);
    assert!((u[0].share - 1.0).abs() < 1e-12);
    // the mine's pump is better by as much
    let (mult_later, ..) = econ::sector_tfp(&s.st, &C, &P, coal, town, &cheap, &s.st.fill);
    let want = (1.0 + P.pump_boost * q) / (1.0 + P.pump_boost);
    assert!((mult_later / mult_1750 - want).abs() < 1e-9, "{} against {want}", mult_later / mult_1750);
    // and it stops at the cap
    s.st.method_Q[bessemer] = 1e15;
    assert_eq!(machines::quality(&s.st, &C, &P), M.quality_cap);
}

#[test]
fn a_new_method_starts_from_its_predecessors_skill() {
    let (mut s, _) = at_start();
    let (C, P) = (s.content.clone(), s.params.clone());
    let (crucible, bessemer) = (method_index(&C, "crucible_steel"), method_index(&C, "bessemer"));
    let coke = method_index(&C, "coke_furnace");
    s.st.method_Q.iter_mut().for_each(|q| *q = 0.0);
    s.st.method_Q[crucible] = 1e9; // two generations of crucible steel
    // (a method's place among the techs is its own number)
    s.st.tech_year[crucible] = 1770;
    let own = firms::learning(&C, &P, &s.st.method_Q);
    // not yet found: nothing to inherit
    s.st.tech_year[bessemer] = -1;
    let mut A = own.clone();
    machines::inherit_skill(&s.st, &C, &P, &mut A);
    assert_eq!(A, own);
    // found: the converter starts from half of what crucible steel has learned
    s.st.tech_year[bessemer] = 1855;
    machines::inherit_skill(&s.st, &C, &P, &mut A);
    assert!((A[bessemer] - P.machines.inherit * own[crucible]).abs() < 1e-12);
    assert!(A[bessemer] > own[bessemer]);
    // crucible steel takes nothing back, and other goods' methods are untouched
    assert_eq!(A[crucible], own[crucible]);
    assert_eq!(A[coke], own[coke]);
    // off: nothing
    let mut port = P.clone();
    port.machines = MachineParams::default();
    let mut A = own.clone();
    machines::inherit_skill(&s.st, &C, &port, &mut A);
    assert_eq!(A, own);
}

#[test]
fn the_game_content_changes_only_the_converter_and_the_steel_rail_mill() {
    let (port, game) = (default_content(), game_content());
    let (bessemer, mill) = (method_index(&port, "bessemer"), method_index(&port, "steel_rolling"));
    let (steel, bricks) = (port.gi("steel"), port.gi("bricks"));
    assert_eq!((port.G, port.M, port.NT), (game.G, game.M, game.NT));
    for m in 0..port.M {
        let changed = m == bessemer || m == mill;
        assert_eq!(port.m_cap.row(m) != game.m_cap.row(m), changed, "plant of {}", port.method_name[m]);
        assert_eq!(port.m_in.row(m) != game.m_in.row(m), m == mill, "inputs of {}", port.method_name[m]);
    }
    assert!(game.m_cap[(bessemer, bricks)] < port.m_cap[(bessemer, bricks)]);
    assert!(game.m_in[(mill, steel)] < port.m_in[(mill, steel)]);
    for t in 0..port.NT {
        let changed = t == bessemer || t == mill; // a method's place among the techs is its own number
        assert_eq!(port.tech_disc.row(t) != game.tech_disc.row(t), changed, "thresholds of {}", port.tech_name[t]);
    }
}

/// What `machines::end_month` promises of the machines standing, held after
/// every month and every year's end of a run: a town that has had a month has
/// a share between 0 and 1 in every column, and a town that has not (a slot
/// never founded, or a town founded as the last year closed) is marked -1, so
/// that its first month starts from what its settlers keep up.
fn machines_stand_as_promised(s: &Sim, had_a_month: &[bool], when: &str) {
    let st = &s.st;
    assert_eq!((st.machine_stock.rows, st.machine_stock.cols), (st.T_max, s.content.NS * machines::SLOTS));
    for t in 0..st.T_max {
        for k in 0..st.machine_stock.cols {
            let v = st.machine_stock[(t, k)];
            if had_a_month[t] {
                assert!((0.0..=1.0 + 1e-9).contains(&v), "{when}, town {t} column {k}: {v}");
            } else {
                assert_eq!(v, -1.0, "{when}, town {t}: a town that has not had a month has no machines");
            }
        }
    }
}

/// A run under the game's rules, played a month at a time with the money
/// checked as `sim::run_with` checks it, and the machines held to their promise
/// at every step. Returns the run and how many towns it saw founded.
fn run(seed: u64, end_year: i64) -> (RunResult, usize) {
    let mut s = sim::start(&game(seed, end_year), game_content());
    let mut had_a_month = vec![false; s.st.T_max];
    let mut founded = 0;
    while s.year <= s.end_year {
        for month in 0..12 {
            sim::run_month(&mut s, month).expect("money conserved with machines on");
            // towns are founded only at a year's end, so the towns active now are
            // the ones `end_month` saw this month
            for t in 0..s.st.T_max {
                if s.st.town_active[t] && !had_a_month[t] {
                    founded += (s.year > s.content.start_year || month > 0) as usize;
                    had_a_month[t] = true;
                }
            }
            machines_stand_as_promised(&s, &had_a_month, &format!("{}-{month}", s.year));
        }
        let year = s.year;
        sim::end_year(&mut s);
        machines_stand_as_promised(&s, &had_a_month, &format!("the end of {year}"));
    }
    (sim::finish(s), founded)
}

#[test]
fn money_is_conserved_and_runs_are_deterministic_with_machines_on() {
    // forty years: pumps and wagonways arrive, steel is poured, machines improve,
    // and towns are founded, whose machines must start where the rule says
    let ((a, founded), (b, _)) = (run(5, 1790), run(5, 1790));
    assert!(founded > 0, "forty years and no town founded: the rule for a new town's machines went untried");
    for (k, name) in a.state.series.names.iter().enumerate() {
        let (x, y) = (&a.state.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs between identical runs");
    }
    assert_eq!(a.state.machine_stock, b.state.machine_stock);
    let (st, C) = (&a.state, &a.content);
    let pumped = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.machine_stock[(t, sector(C, "coalmine") * machines::SLOTS)]).fold(0.0f64, f64::max);
    assert!(pumped > 0.5, "by 1790 some town's collieries are pumped: {pumped}");
    assert!(machines::quality(st, C, &a.params) > 1.0, "forty years of steel have taught the engine builders something");
}
