//! Scratch: force a town with a tiny catchment and watch it. probe_tiny SEED YEAR on|off [MAXCELLS]
#![allow(non_snake_case)]
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seed: u64 = args[0].parse().unwrap();
    let when: i64 = args[1].parse().unwrap();
    let on = args[2] == "on";
    let maxc: usize = args.get(3).map_or(6, |s| s.parse().unwrap());
    let mut P = default_params();
    P.seed = seed;
    P.people.enabled = on;
    P.end_year = Some(when + 12);
    let mut sim = sim::start(&P, default_content());
    let T = sim.st.T_max;
    let mut forced: Option<usize> = None;
    while sim.year <= sim.end_year {
        let year = sim.year;
        if year == when {
            let st = &sim.st;
            let (W, N) = (st.world.W, st.N);
            let mut best: Option<(usize, usize)> = None;
            for c in 0..N {
                if st.world.coast[c] || !(st.world.land[c] && st.world.terrain[c] < 3 && st.catch[c] >= 0 && st.cell_K_share[c] < 0.5) {
                    continue;
                }
                let near = (0..T).filter(|&t| st.town_active[t]).map(|t| ((st.town_node[t] / W) as f64 - (c / W) as f64).hypot((st.town_node[t] % W) as f64 - (c % W) as f64)).fold(f64::INFINITY, f64::min);
                if !(2.0..=7.0).contains(&near) {
                    continue;
                }
                let d = transport::distances(&st.graph, &[(c, 0.0)]);
                let cells = (0..N).filter(|&k| st.catch[k] >= 0 && d[k] < st.dcatch[k]).count();
                if cells >= 2 && cells <= maxc && best.map_or(true, |(_, b)| cells < b) {
                    best = Some((c, cells));
                }
            }
            let (c, cells) = best.expect("no tiny site");
            println!("forcing a town at cell {c} with about {cells} cells");
            if on {
                sim.st.pp_found_next = Some(c);
            } else {
                people::TEMP_FORCE.store(c, std::sync::atomic::Ordering::SeqCst);
            }
        }
        if let Err(e) = sim::run_year(&mut sim) {
            println!("FAILED in {year}: {e}");
            return;
        }
        let st = &sim.st;
        if year == when {
            forced = Some((0..T).filter(|&t| st.town_active[t]).count() - 1);
            println!("town {} {}", forced.unwrap(), st.town_name[forced.unwrap()]);
        }
        if let Some(t) = forced {
            let tot: f64 = (0..T).filter(|&t| st.town_active[t]).map(|t| st.pop[t]).sum();
            let nat: f64 = (0..T).filter(|&t| st.town_active[t]).map(|t| st.wage[t] * st.pop[t]).sum::<f64>() / tot;
            let ms = &st.month_stats;
            println!(
                "{year} pop {:8.0} wage {:8.2} nat {:6.2} employed/supply {:6.2} cells {:4} y {:6.2} cash/head {:8.1}",
                st.pop[t],
                st.wage[t],
                nat,
                ms.employment[t] / ms.supply[t].max(1e-9),
                st.town_cells[t].len(),
                st.y_ratio[t],
                st.hh_cash[t] / st.pop[t].max(1.0)
            );
        }
    }
}
