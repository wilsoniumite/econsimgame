//! Railway companies: lines get built where existing traffic would save most.
//! Python: `rail.py`.
//!
//! A line is proposed between two towns once railway know-how reaches them. Its
//! promoters estimate the yearly carting cost it would save on current traffic
//! (times an induced-traffic factor), compare that with the cost of rails, timber
//! and stone along the cheapest buildable route, and raise money locally if the
//! return clears the hurdle. Rails are then bought month by month, so a burst of
//! projects shows up as a rail and iron boom.

use crate::content::Content;
use crate::mat::Mat;
use crate::ndimage::{self, Mode};
use crate::params::Params;
use crate::rng::Rng;
use crate::state::{Event, Project, RailOpened, State};
use crate::transport as tr;
use crate::util::{argsort, fmt_pct, pairwise_sum, pairwise_sum_by};

/// Path length in cells (1 per straight step, sqrt(2) per diagonal), summed as numpy does.
fn path_length(path: &[usize], W: usize) -> f64 {
    let steps: Vec<f64> = path
        .windows(2)
        .map(|p| {
            let (dy, dx) = ((p[1] / W) as f64 - (p[0] / W) as f64, (p[1] % W) as f64 - (p[0] % W) as f64);
            dy.hypot(dx)
        })
        .collect();
    pairwise_sum(&steps)
}

fn materials(C: &Content, P: &Params, n_new: usize) -> Vec<f64> {
    let mut need = vec![0.0; C.G];
    need[C.gi("rails")] = P.rail_rails_per_cell * n_new as f64;
    need[C.gi("timber")] = P.rail_timber_per_cell * n_new as f64;
    need[C.gi("stone")] = P.rail_stone_per_cell * n_new as f64;
    need
}

pub fn plan(st: &mut State, C: &Content, P: &Params, _rng: &mut Rng, year: i64, pbar: &Mat, events: &mut Vec<Event>) {
    let tr_idx = st.t_rail;
    let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    let k: Vec<f64> = (0..st.T_max).map(|t| st.know[(t, tr_idx)]).collect();
    let kmax = act.iter().map(|&t| k[t]).fold(f64::NEG_INFINITY, f64::max);
    if act.len() < 2 || kmax < P.rail_know || st.projects.len() >= P.rail_max_projects {
        return;
    }
    let gr = C.gi("rails");
    let F = &st.trade_F_year;
    let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
    for &a_ in &act {
        for &b_ in &act {
            if a_ < b_ && k[a_].max(k[b_]) >= P.rail_know && k[a_].min(k[b_]) > 0.05 {
                if st.rail_links.contains(&(a_, b_)) || st.projects.iter().any(|pr| (pr.a, pr.b) == (a_, b_)) {
                    continue;
                }
                if st.rail_cooldown.get(&(a_, b_)).copied().unwrap_or(-99) > year {
                    continue;
                }
                let flow = F[(a_, b_)] + F[(b_, a_)];
                if flow > 0.0 {
                    pairs.push((flow, a_, b_));
                }
            }
        }
    }
    if pairs.is_empty() {
        return;
    }
    // Python sorts the (flow, a, b) tuples in reverse
    pairs.sort_by(|x, y| y.0.total_cmp(&x.0).then(y.1.cmp(&x.1)).then(y.2.cmp(&x.2)));
    let rg = tr::rail_build_graph(&st.world, &st.rail);
    let W = st.world.W;
    for &(flow, a_, b_) in pairs.iter().take(30) {
        if st.projects.len() >= P.rail_max_projects {
            break;
        }
        let (na, nb) = (st.town_node[a_], st.town_node[b_]);
        // only the path to nb is used, so the search can stop there
        let paths = tr::dijkstra(&rg, &[(na, 0.0)], Some(nb));
        if !paths.dist[nb].is_finite() || paths.dist[nb] > 1e5 {
            continue;
        }
        let path = tr::path_cells(&paths.pred, na, nb);
        if path.len() < 2 {
            continue;
        }
        let new_cells: Vec<usize> = path.iter().copied().filter(|&c| st.rail[c] == 0).collect();
        let n_new = new_cells.len();
        if n_new == 0 {
            st.rail_links.insert((a_, b_));
            continue;
        }
        let length = path_length(&path, W);
        let cost_now = st.town_cost[(a_, b_)];
        let cost_rail = tr::RAIL_COST * length + 1.0;
        let saving_per_w = P.freight * (cost_now - cost_rail).max(0.0);
        let benefit = P.rail_induced * flow * saving_per_w;
        let need = materials(C, P, n_new);
        let buy_town = if pbar[(a_, gr)] <= pbar[(b_, gr)] { a_ } else { b_ };
        let cost = pairwise_sum_by(C.G, |g| need[g] * pbar[(buy_town, g)]) * 1.3;
        let roi = benefit / cost.max(1e-9);
        if roi < P.rail_hurdle {
            st.rail_cooldown.insert((a_, b_), year + 4);
            continue;
        }
        // promoters in both towns subscribe
        let mut raised = 0.0;
        for t in [a_, b_] {
            for (lords, frac) in [(true, 0.35), (false, 0.08)] {
                let cash = if lords { &mut st.lord_cash } else { &mut st.hh_cash };
                let take = (cash[t].max(0.0) * frac).min(cost - raised);
                if take > 0.0 {
                    cash[t] -= take;
                    raised += take;
                }
            }
        }
        if raised < 0.5 * cost {
            for t in [a_, b_] {
                st.hh_cash[t] += raised / 2.0;
            }
            st.rail_cooldown.insert((a_, b_), year + 2);
            continue;
        }
        st.projects.push(Project {
            a: a_,
            b: b_,
            spur: false,
            target: 0,
            town: buy_town,
            cells: path,
            new: new_cells,
            order: need.clone(),
            need,
            cash: raised,
            start: year,
            roi,
            cost,
            length_km: 2.0 * length,
        });
        let mut ev = Event::new(
            year as f64 + 0.95,
            "rail_plan",
            0.45,
            format!("{} and {} Railway promoted", st.town_name[a_], st.town_name[b_]),
            format!(
                "Promoters raise money for {:.0} km of line. They expect the traffic now carted between the towns to repay {} a year.",
                2.0 * length,
                fmt_pct(roi, 0)
            ),
        );
        ev.town = Some(a_);
        ev.cell = Some(na);
        events.push(ev);
    }
}

/// Mineral and factory lines: from a town out to a busy cluster of pits or works.
pub fn plan_spurs(st: &mut State, C: &Content, P: &Params, _rng: &mut Rng, year: i64, pbar: &Mat, _events: &mut Vec<Event>) {
    let tr_idx = st.t_rail;
    if st.projects.len() >= P.rail_max_projects {
        return;
    }
    let (H, W) = (st.world.H, st.world.W);
    let flow: Vec<f64> = ndimage::uniform_filter(&st.cell_flow_year, H, W, 5, Mode::Constant).iter().map(|v| v * 25.0).collect();
    let railed: Vec<bool> = st.rail.iter().map(|&r| r > 0).collect();
    if !railed.iter().any(|&r| r) {
        return;
    }
    let not_railed: Vec<bool> = railed.iter().map(|&r| !r).collect();
    let d_rail = ndimage::distance_transform_edt(&not_railed, H, W);
    let mut rg: Option<tr::Graph> = None;
    let neg_flow: Vec<f64> = flow.iter().map(|&f| -f).collect();
    for &c in argsort(&neg_flow).iter().take(25) {
        if st.projects.len() >= P.rail_max_projects {
            break;
        }
        let i = st.catch[c];
        if i < 0 || d_rail[c] < 4.0 || flow[c] < P.spur_min_flow || st.know[(i as usize, tr_idx)] < P.rail_know {
            continue;
        }
        let i = i as usize;
        // Python's nearness test, reproduced as written: `abs(c - d) % W <= 3` is not
        // the column distance (see PORTING.md)
        let (ci, wi) = (c as i64, W as i64);
        if st.spur_done.iter().any(|&d| {
            let d = d as i64;
            (ci - d).abs() % wi <= 3 && (ci / wi - d / wi).abs() <= 3
        }) {
            continue;
        }
        if st.rail[st.town_node[i]] == 0 {
            continue;
        }
        let graph = rg.get_or_insert_with(|| tr::rail_build_graph(&st.world, &st.rail));
        let src = st.town_node[i];
        let paths = tr::dijkstra(graph, &[(src, 0.0)], Some(c));
        if !paths.dist[c].is_finite() || paths.dist[c] > 1e5 {
            continue;
        }
        let path = tr::path_cells(&paths.pred, src, c);
        let new_cells: Vec<usize> = path.iter().copied().filter(|&x| st.rail[x] == 0).collect();
        if new_cells.len() < 2 {
            st.spur_done.insert(c);
            continue;
        }
        let length = path_length(&path, W);
        let saving = P.freight * (st.dcatch[c] - (tr::RAIL_COST * length + 0.5)).max(0.0);
        let benefit = P.rail_induced * flow[c] * saving;
        let need = materials(C, P, new_cells.len());
        let cost = pairwise_sum_by(C.G, |g| need[g] * pbar[(i, g)]) * 1.3;
        let roi = benefit / cost.max(1e-9);
        st.spur_done.insert(c);
        if roi < P.rail_hurdle {
            continue;
        }
        let mut raised = 0.0;
        for (lords, frac) in [(true, 0.3), (false, 0.06)] {
            let cash = if lords { &mut st.lord_cash } else { &mut st.hh_cash };
            let take = (cash[i].max(0.0) * frac).min(cost - raised);
            if take > 0.0 {
                cash[i] -= take;
                raised += take;
            }
        }
        if raised < 0.5 * cost {
            st.hh_cash[i] += raised;
            continue;
        }
        st.projects.push(Project {
            a: i,
            b: i,
            spur: true,
            target: c,
            town: i,
            cells: path,
            new: new_cells,
            order: need.clone(),
            need,
            cash: raised,
            start: year,
            roi,
            cost,
            length_km: 2.0 * length,
        });
    }
}

/// Open finished lines; promoters top up projects that run short of money.
pub fn progress(st: &mut State, _C: &Content, _P: &Params, year_float: f64, events: &mut Vec<Event>) {
    let projects = std::mem::take(&mut st.projects);
    let mut keep = Vec::with_capacity(projects.len());
    for mut pr in projects {
        if pr.cash < 0.0 {
            for t in [pr.a, pr.b] {
                let top = (st.lord_cash[t].max(0.0) * 0.1).min(-pr.cash);
                st.lord_cash[t] -= top;
                pr.cash += top;
            }
        }
        let done = pairwise_sum(&pr.order) <= 0.01 * pairwise_sum(&pr.need);
        if !done {
            keep.push(pr);
            continue;
        }
        for &c in &pr.cells {
            st.rail[c] = 1;
        }
        st.rail_links.insert((pr.a, pr.b));
        st.hh_cash[pr.town] += pr.cash;
        pr.cash = 0.0;
        let (a_, b_) = (pr.a, pr.b);
        if pr.spur {
            st.rail_opened.push(RailOpened { year: year_float, a: a_, b: -1, km: pr.length_km });
            let what = match st.land_use[pr.target] {
                7 => "iron mines",
                8 => "collieries",
                9 => "works",
                _ => "pits and works",
            };
            let mut ev = Event::new(
                year_float,
                "railway",
                0.35,
                format!("Mineral line opens near {}", st.town_name[a_]),
                format!("{:.0} km of branch line now carries the output of the {what} into {}.", pr.length_km, st.town_name[a_]),
            );
            ev.town = Some(a_);
            ev.cell = Some(pr.target);
            events.push(ev);
            continue;
        }
        let first = st.rail_opened.is_empty();
        st.rail_opened.push(RailOpened { year: year_float, a: a_, b: b_ as i64, km: pr.length_km });
        let title = format!(
            "{}{} to {}",
            if first { "The first railway opens: " } else { "Railway opens: " },
            st.town_name[a_],
            st.town_name[b_]
        );
        let mut ev = Event::new(
            year_float,
            "railway",
            if first { 1.0 } else { 0.45 },
            title,
            format!(
                "{:.0} km of track, begun in {}. Freight between the towns now moves at a fraction of the carting cost.",
                pr.length_km, pr.start
            ),
        );
        ev.town = Some(a_);
        ev.cell = Some(st.town_node[a_]);
        ev.cell2 = Some(st.town_node[b_]);
        events.push(ev);
    }
    st.projects = keep;
}
