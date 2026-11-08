//! Knowledge: discovery from use, diffusion between towns, learning by doing.
//! Python: `tech.py`.
//!
//! Nothing is unlocked by a tree. A method becomes discoverable once the economy
//! has handled enough of the goods it combines (Weitzman-style recombination),
//! appears first in the town that handles most of them, and spreads along trade
//! routes. Its productivity then climbs a learning curve with cumulative output.

use crate::content::Content;
use crate::names::surname;
use crate::params::Params;
use crate::rng::{Rng, choice_index, choice_p, random};
use crate::state::{Event, State};
use crate::util::pairwise_sum_by;

pub const FIRST_NAMES: &[&str] = &[
    "Abraham", "Benjamin", "Caleb", "Daniel", "Edmund", "Ezra", "George", "Hannah",
    "Isaac", "Jedediah", "John", "Josiah", "Lydia", "Martha", "Matthew", "Nathaniel",
    "Obadiah", "Richard", "Samuel", "Sarah", "Thomas", "William", "Zachariah", "Eliza",
];

pub fn discovery(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, year: i64, events: &mut Vec<Event>) -> Vec<usize> {
    let mut found = Vec::new();
    for t in 0..C.NT {
        if st.tech_year[t] >= 0 {
            continue;
        }
        let X0 = st.disc_X0.row(t).to_vec();
        let req: Vec<usize> = (0..C.G).filter(|&g| X0[g] > 0.0).collect();
        if req.is_empty() {
            continue;
        }
        let min_ratio = req.iter().map(|&g| st.cumprod[g] / X0[g]).fold(f64::INFINITY, f64::min);
        let progress = min_ratio.max(0.0);
        let p4 = progress.powf(4.0);
        let hz = P.discovery_rate * p4 / (1.0 + p4);
        if random(rng) < hz {
            let act: Vec<usize> = (0..st.T_max).filter(|&i| st.town_active[i]).collect();
            // the likeliest place is the one handling most of the ingredients,
            // but any town that handles them at all has a chance
            let w: Vec<f64> = act
                .iter()
                .map(|&i| {
                    let score = pairwise_sum_by(req.len(), |k| st.town_prod_year[(i, req[k])] / X0[req[k]]);
                    let v = score + 1e-9 * st.pop[i];
                    v * v
                })
                .collect();
            let total: f64 = crate::util::pairwise_sum(&w);
            let p: Vec<f64> = w.iter().map(|&x| x / total).collect();
            let i = act[choice_p(rng, &p)];
            st.know[(i, t)] = st.know[(i, t)].max(0.25);
            st.tech_year[t] = year;
            st.tech_town[t] = i as i64;
            let who = format!("{} {}", FIRST_NAMES[choice_index(rng, FIRST_NAMES.len())], surname(rng));
            let town = st.town_name[i].clone();
            let fill = |s: &str| s.replace("{town}", &town).replace("{who}", &who);
            let mut ev = Event::new(year as f64 + 0.5, "discovery", 0.9, fill(&C.tech_headline[t]), fill(&C.tech_story[t]));
            ev.town = Some(i);
            ev.cell = Some(st.town_node[i]);
            ev.tech = Some(t);
            events.push(ev);
            found.push(t);
        }
    }
    found
}

pub fn diffusion(st: &mut State, C: &Content, P: &Params) {
    let T = st.T_max;
    let act = &st.town_active;
    let c = &st.town_cost;
    let mut Wn = vec![0.0; T * T];
    for i in 0..T {
        let mut row = vec![0.0; T];
        for j in 0..T {
            if i != j && act[i] && act[j] && c[(i, j)].is_finite() {
                row[j] = (-c[(i, j)] / P.diffusion_dist).exp();
            }
        }
        let rs = crate::util::pairwise_sum(&row);
        for j in 0..T {
            Wn[i * T + j] = (if rs > 0.0 { row[j] / rs.max(1e-12) } else { 0.0 }) * rs.min(1.0);
        }
    }
    let k = st.know.clone();
    let discovered: Vec<bool> = (0..C.NT).map(|t| st.tech_year[t] >= 0 || C.tech_known[t]).collect();
    for i in 0..T {
        for t in 0..C.NT {
            let mut contact = 0.0;
            for j in 0..T {
                contact += Wn[i * T + j] * k[(j, t)];
            }
            let kv = k[(i, t)];
            let dk = P.diffusion_rate * (1.0 - kv) * (kv + 0.7 * contact);
            let on = if discovered[t] && act[i] { 1.0 } else { 0.0 };
            st.know[(i, t)] = (kv + dk * on).clamp(0.0, 1.0);
        }
    }
}
