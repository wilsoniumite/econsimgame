//! What is built up on the ground of an island's largest town (docs/rules/ground.md):
//! for every cell of its ground, in the years asked, its distance from the
//! town's centre (in cells, a straight line, and along the ways), its `dens`,
//! its room and its people. Plays the island under the game's rules.
//!
//! ```text
//! cargo run --release -p isostate-core --example ground_profile -- --seed 7 --years 1950,2000,2050 --out profile.csv
//! ```
//!
//! `--off` plays it with the ground off (every `dens` is then 1), for setting
//! beside it. The largest town is the one with the most townsfolk in each year.
#![allow(non_snake_case)]

use std::io::Write;

use isostate_core::sim;
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seed = 7u64;
    let mut years: Vec<i64> = vec![1950, 2000, 2050];
    let mut out = String::from("ground_profile.csv");
    let mut off = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                seed = args[i + 1].parse().expect("--seed N");
                i += 1;
            }
            "--years" => {
                years = args[i + 1].split(',').map(|y| y.parse().expect("--years Y,Y,...")).collect();
                i += 1;
            }
            "--out" => {
                out = args[i + 1].clone();
                i += 1;
            }
            "--off" => off = true,
            a => panic!("unknown argument {a}"),
        }
        i += 1;
    }
    let end = *years.iter().max().expect("a year");
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end);
    P.ground.enabled = !off;
    let mut s = sim::start(&P, game_content());
    let mut f = std::fs::File::create(&out).expect("the file can be written");
    writeln!(f, "year,town,name,townsfolk,cell,x,y,from_centre,by_way,dens,room,people,floors").unwrap();
    while s.year <= end {
        let year = s.year;
        sim::run_year(&mut s).unwrap_or_else(|e| panic!("{e}"));
        if !years.contains(&year) {
            continue;
        }
        let st = &s.st;
        let W = st.world.W;
        let t = (0..st.T_max).filter(|&t| st.town_active[t]).max_by(|&a, &b| st.urban_pop[a].total_cmp(&st.urban_pop[b])).expect("a town");
        let node = st.town_node[t];
        let frame = s.content.method_name.iter().position(|n| n == "steel_frame");
        let know = frame.map_or(0.0, |m| st.know[(t, m)]);
        let top = P.ground.floors + (P.ground.floors_frame - P.ground.floors) * know;
        for &c in &st.town_ground[t] {
            let d = ((c / W) as f64 - (node / W) as f64).hypot((c % W) as f64 - (node % W) as f64);
            let dens = st.ground.dens.get(c).copied().unwrap_or(1.0);
            writeln!(
                f,
                "{year},{t},{},{:.0},{c},{},{},{d:.3},{:.4},{dens:.4},{:.1},{:.1},{top:.3}",
                st.town_name[t],
                st.urban_pop[t],
                c % W,
                c / W,
                st.dist[(t, c)],
                st.town_room[c],
                st.urban_res[c]
            )
            .unwrap();
        }
        eprintln!("{year}: {} ({:.0} townsfolk, {} cells)", st.town_name[t], st.urban_pop[t], st.town_ground[t].len());
    }
}
