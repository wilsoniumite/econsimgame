// throwaway: town sizes over time (not committed)
#![allow(non_snake_case)]
use isostate_core::*;

fn main() {
    let seed: u64 = std::env::args().nth(1).map_or(7, |s| s.parse().unwrap());
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    let mut s = sim::start(&P, game_content());
    while s.year <= s.end_year {
        sim::run_year(&mut s).unwrap();
        let y = s.year - 1;
        if y % 5 == 0 {
            let st = &s.st;
            let mut by_u: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
            by_u.sort_by(|&a, &b| st.urban_pop[b].total_cmp(&st.urban_pop[a]));
            let mut by_p = by_u.clone();
            by_p.sort_by(|&a, &b| st.pop[b].total_cmp(&st.pop[a]));
            let u: Vec<String> = by_u.iter().take(4).map(|&t| format!("{} {:.0}", st.town_name[t], st.urban_pop[t])).collect();
            let p: Vec<String> = by_p.iter().take(4).map(|&t| format!("{} {:.0}", st.town_name[t], st.pop[t])).collect();
            println!("{y} urban: {}\n     pop:   {}", u.join(", "), p.join(", "));
            // closures this year by town
            let mut closed = vec![0usize; st.T_max];
            for &f in &st.closed_year {
                if s.content.method_name[st.f_method[f]] != "services" {
                    closed[st.f_town[f]] += 1;
                }
            }
            let mut standing = vec![0usize; st.T_max];
            for f in 0..st.F_count {
                if st.f_alive[f] && s.content.method_name[st.f_method[f]] != "services" {
                    standing[st.f_town[f]] += 1;
                }
            }
            let c: Vec<String> = (0..st.T_max).filter(|&t| closed[t] >= 3).map(|t| format!("{} {}/{}", st.town_name[t], closed[t], closed[t] + standing[t])).collect();
            println!("     closures: {}", c.join(", "));
            let dec: Vec<String> = (0..st.T_max)
                .filter(|&t| st.town_active[t] && st.pop[t] < 0.95 * st.chron.peak[t].1)
                .map(|t| format!("{} {:.0}% of {} peak {:.0}", st.town_name[t], 100.0 * st.pop[t] / st.chron.peak[t].1, st.chron.peak[t].0, st.chron.peak[t].1))
                .collect();
            println!("     below peak: {}", dec.join(", "));
        }
    }
}
