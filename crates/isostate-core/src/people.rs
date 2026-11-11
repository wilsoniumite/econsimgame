//! Population: growth from living standards, migration toward better wages,
//! and new towns where work appears far from any market. Python: `people.py`.

use crate::content::Content;
use crate::mat::Mat;
use crate::names::town_name;
use crate::ndimage::{self, Mode};
use crate::params::Params;
use crate::rng::Rng;
use crate::state::{Event, State};
use crate::util::{argmax, argmin, bool_f, fmt_pct, pairwise_sum};
use crate::world::site_context;

pub fn growth(st: &mut State, _C: &Content, P: &Params, year: i64, events: &mut Vec<Event>) -> Vec<f64> {
    let T = st.T_max;
    let act = st.town_active.clone();
    let y: Vec<f64> = st.y_acc.iter().map(|&v| v / 12.0).collect();
    st.y_ratio = (0..T).map(|t| if act[t] { y[t] } else { 0.0 }).collect();
    let mut g: Vec<f64> = y.iter().map(|&v| (P.growth_slope * v.max(1e-3).ln() + P.growth_base).clamp(-0.04, P.growth_cap)).collect();
    let famine: Vec<bool> = (0..T).map(|t| act[t] && st.food_min_acc[t] < 0.82).collect();
    for t in 0..T {
        if famine[t] {
            g[t] -= 0.02;
        }
    }
    // people (docs/rules/people.md): the game's rule sets the rates in place of the capped one
    if P.people.enabled {
        g = crate::peopling::growth(st, P, &y, year, events);
    }
    let hit: Vec<usize> = (0..T).filter(|&t| famine[t] && st.town_founded[t] < year - 2).collect();
    if hit.len() >= 3 {
        let mins: Vec<f64> = hit.iter().map(|&t| st.food_min_acc[t]).collect();
        let worst = hit[argmin(&mins)];
        let mut ev = Event::new(
            year as f64 + 0.6,
            "famine",
            0.7,
            format!("Hunger in {} towns", hit.len()),
            format!(
                "Bread runs short across the country. In {}, the worst hit, households get {} of the food they need at the worst point of the year.",
                st.town_name[worst],
                fmt_pct(st.food_min_acc[worst], 0)
            ),
        );
        ev.town = Some(worst);
        ev.cell = Some(st.town_node[worst]);
        events.push(ev);
    } else {
        for &i in &hit {
            let mut ev = Event::new(
                year as f64 + 0.6,
                "famine",
                0.5,
                format!("Hunger in {}", st.town_name[i]),
                format!(
                    "Bread runs short in {}: households get {} of the food they need at the worst point of the year.",
                    st.town_name[i],
                    fmt_pct(st.food_min_acc[i], 0)
                ),
            );
            ev.town = Some(i);
            ev.cell = Some(st.town_node[i]);
            events.push(ev);
        }
    }
    // chronicle: the year's hunger says how short the food ran, before the measure is cleared
    crate::chronicle::famine_provenance(st, P, year, events);
    for t in 0..T {
        st.pop[t] = if act[t] { st.pop[t] * (1.0 + g[t]) } else { 0.0 };
    }
    st.y_acc.iter_mut().for_each(|v| *v = 0.0);
    st.food_min_acc.iter_mut().for_each(|v| *v = 9.0);
    g
}

pub fn migration(st: &mut State, _C: &Content, P: &Params) {
    let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    let n = act.len();
    if n < 2 {
        return;
    }
    let y: Vec<f64> = act.iter().map(|&t| st.y_ratio[t].max(1e-3)).collect();
    let mut wgt = Mat::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let gain = (y[j] / y[i] - 1.0).max(0.0);
            let c = st.town_cost[(act[i], act[j])];
            wgt[(i, j)] = gain * if c.is_finite() { (-c / P.migration_dist).exp() } else { 0.0 };
        }
    }
    let tot: Vec<f64> = (0..n).map(|i| pairwise_sum(wgt.row(i))).collect();
    let movers: Vec<f64> = (0..n).map(|i| P.migration_rate * st.pop[act[i]] * tot[i].min(1.0)).collect();
    let mut flow = Mat::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            let share = if tot[i] > 0.0 { wgt[(i, j)] / tot[i].max(1e-12) } else { 0.0 };
            flow[(i, j)] = movers[i] * share;
        }
    }
    // people (docs/rules/people.md): a town takes in only as many as it can house
    if P.people.enabled {
        crate::peopling::limit_arrivals(st, P, &act, &mut flow);
    }
    let cash_pc: Vec<f64> = act.iter().map(|&t| st.hh_cash[t].max(0.0) / st.pop[t].max(1.0)).collect();
    // flow.sum(axis=0) runs down the columns one by one; flow.sum(axis=1) is pairwise
    let mut inflow = vec![0.0; n];
    let mut money_in = vec![0.0; n];
    for i in 0..n {
        for j in 0..n {
            inflow[j] += flow[(i, j)];
            money_in[j] += flow[(i, j)] * cash_pc[i];
        }
    }
    for (k, &t) in act.iter().enumerate() {
        let outflow = pairwise_sum(flow.row(k));
        st.pop[t] += inflow[k] - outflow;
        let money_out = crate::util::pairwise_sum_by(n, |j| flow[(k, j)] * cash_pc[k]);
        st.hh_cash[t] += money_in[k] - money_out;
    }
    st.migration_flow = flow;
}

/// Mining and industrial settlements far from existing markets become towns.
pub fn found_towns(st: &mut State, _C: &Content, P: &Params, rng: &mut Rng, year: i64, events: &mut Vec<Event>) -> Vec<usize> {
    let (H, W, N) = (st.world.H, st.world.W, st.N);
    let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    if act.len() >= st.T_max {
        return Vec::new();
    }
    let mut work = vec![0.0; N];
    for u in 0..st.ex_cell.len() {
        work[st.ex_cell[u]] += st.ex_labor[u];
    }
    for f in 0..st.f_alive.len() {
        if st.f_alive[f] {
            work[st.f_cell[f]] += st.f_labor[f];
        }
    }
    let near: Vec<f64> = ndimage::uniform_filter(&work, H, W, 7, Mode::Constant).iter().map(|v| v * 49.0).collect();
    let land: Vec<bool> = (0..N).map(|c| st.world.land[c] && st.world.terrain[c] < 3).collect();
    let mut far: Vec<bool> = st.dcatch.iter().map(|&d| d > P.found_distance).collect();
    for &i in &act {
        let (ty, tx) = ((st.town_node[i] / W) as f64, (st.town_node[i] % W) as f64);
        for c in 0..N {
            let (y, x) = ((c / W) as f64, (c % W) as f64);
            far[c] = far[c] && (y - ty).hypot(x - tx) > 8.0;
        }
    }
    let score: Vec<f64> = (0..N).map(|c| if land[c] && far[c] && st.dcatch[c].is_finite() { near[c] } else { 0.0 }).collect();
    let mut founded = Vec::new();
    let c = argmax(&score);
    if score[c] > P.found_activity {
        // settle on the best nearby site: flat ground, water, close to the work
        let (cy, cx) = ((c / W) as isize, (c % W) as isize);
        let (mut best, mut bc) = (f64::NEG_INFINITY, c);
        for dy in -2isize..=2 {
            for dx in -2isize..=2 {
                let (ny, nx) = (cy + dy, cx + dx);
                if ny >= 0 && ny < H as isize && nx >= 0 && nx < W as isize {
                    let nc = ny as usize * W + nx as usize;
                    if land[nc] && st.cell_K_share[nc] < 0.5 {
                        let s = near[nc] / score[c].max(1.0) + 0.3 * bool_f(st.world.river[nc]) + 0.3 * bool_f(st.world.coast[nc])
                            - 0.2 * (st.world.terrain[nc] as f64 - 1.0);
                        if s > best {
                            best = s;
                            bc = nc;
                        }
                    }
                }
            }
        }
        let n = act.len();
        if n < st.T_max {
            let ctx = site_context(&st.world, bc);
            let name = town_name(rng, ctx, &mut st.used_names);
            st.town_active[n] = true;
            st.town_node[n] = bc;
            st.town_name.push(name.clone());
            st.town_founded[n] = year;
            st.town_new[n] = true;
            let donor = st.catch[bc] as usize;
            for g in 0..st.price.cols {
                st.price[(n, g)] = st.price[(donor, g)];
                st.fill[(n, g)] = st.fill[(donor, g)];
            }
            // prices: and her merchants' anchors, and what her people expect less the
            // carriage from here (nothing happens while the mechanism is off)
            crate::prices::found_town(st, _C, P, n, donor, bc);
            st.wage[n] = st.wage[donor];
            for t in 0..st.know.cols {
                st.know[(n, t)] = st.know[(donor, t)];
            }
            st.tools_mult[n] = st.tools_mult[donor];
            st.imp_pc[n] = st.imp_pc[donor];
            let what = match ctx {
                "mine" => "a mining settlement",
                "coast" => "a harbour town",
                "river" => "a river town",
                "hills" => "a hill town",
                _ => "a market town",
            };
            let mut ev = Event::new(
                year as f64 + 0.9,
                "town",
                0.6,
                format!("{name} founded"),
                format!("Enough people now work around this spot, far from any market, that {name} grows up as {what}."),
            );
            ev.town = Some(n);
            ev.cell = Some(bc);
            events.push(ev);
            founded.push(n);
        }
    }
    founded
}

/// After catchments change, new towns take the people working in their area.
pub fn resettle_new_towns(st: &mut State, _C: &Content, _P: &Params) {
    let new: Vec<usize> = (0..st.T_max).filter(|&t| st.town_new[t]).collect();
    if new.is_empty() {
        return;
    }
    let N = st.N;
    let mut work = vec![0.0; N];
    for u in 0..st.ex_cell.len() {
        work[st.ex_cell[u]] += st.ex_labor[u];
    }
    for f in 0..st.f_alive.len() {
        if st.f_alive[f] {
            work[st.f_cell[f]] += st.f_labor[f];
        }
    }
    let old_catch = st.prev_catch.clone();
    for &n in &new {
        let cells: Vec<usize> = (0..N).filter(|&c| st.catch[c] == n as i64).collect();
        let mut olds: Vec<i64> = cells.iter().map(|&c| old_catch[c]).collect();
        olds.sort();
        olds.dedup();
        for i in olds {
            if i < 0 || i == n as i64 {
                continue;
            }
            let i = i as usize;
            let mine: Vec<f64> = cells.iter().filter(|&&c| old_catch[c] == i as i64).map(|&c| work[c]).collect();
            let all_i: Vec<f64> = (0..N).filter(|&c| old_catch[c] == i as i64).map(|c| work[c]).collect();
            let tot_i = pairwise_sum(&all_i);
            let frac = if tot_i > 0.0 { pairwise_sum(&mine) / tot_i.max(1e-9) } else { 0.0 };
            let frac = frac.clamp(0.02, 0.5);
            for v in [&mut st.pop, &mut st.hh_cash, &mut st.lord_cash] {
                let moved = v[i] * frac;
                v[i] -= moved;
                v[n] += moved;
            }
            let mv = st.merch_cash[i] * frac;
            st.merch_cash[i] -= mv;
            st.merch_cash[n] += mv;
            for g in 0..st.inv.cols {
                let iv = st.inv[(i, g)] * frac;
                st.inv[(i, g)] -= iv;
                st.inv[(n, g)] += iv;
                st.outflow[(n, g)] += st.outflow[(i, g)] * frac;
            }
        }
        // firms in the new catchment now trade through the new town
    }
    for f in 0..st.f_alive.len() {
        if st.f_alive[f] && st.catch[st.f_cell[f]] >= 0 {
            st.f_town[f] = st.catch[st.f_cell[f]] as usize;
        }
    }
    st.town_new.iter_mut().for_each(|v| *v = false);
}
