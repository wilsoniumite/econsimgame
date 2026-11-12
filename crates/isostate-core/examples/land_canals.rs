//! Where the canals of a run lie with respect to the island's water: how much
//! of the cut runs through cells boats already use (a navigable river, a
//! tideway), along rivers, beside a lake or on the coast, and where the towns
//! stand. Written to look at canals on the land mechanism's islands
//! (docs/rules/land.md, "In company"), which have tideways and lakes; on the
//! port's island the tideway and lake columns are nothing.
//!
//!     cargo run --release -p isostate-core --example land_canals -- [SEEDS, e.g. 1-6] [--families N]
//!         [--set NAME=VALUE]...
//!
//! The game's rules, each seed twice: on the port's island and on the land's.
//! Under each run it lists any railway line that took eight years or more to
//! open, which is how a small town's branch waiting on its landlords' few
//! shillings was found (docs/rules/land.md, "Beside the mines and the canals").
#![allow(non_snake_case)]
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = (1..=6).collect();
    let mut families = 6usize;
    let mut sets: Vec<(String, String)> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--families" => {
                families = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--set" => {
                let (k, v) = args[i + 1].split_once('=').expect("--set NAME=VALUE");
                sets.push((k.to_string(), v.to_string()));
                i += 1;
            }
            s => {
                seeds = match s.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => s.split(',').map(|x| x.parse().unwrap()).collect(),
                }
            }
        }
        i += 1;
    }
    println!("canals and the island's water in 1900, the game's rules, {families} families");
    println!("km: of canal, 2 a cell; nav: share of its cells on water boats already use; river: on a river cell;");
    println!("tide: on a tideway; shore: beside a lake; coast: on the coast; twin: links of canal from one navigable cell to the next;");
    println!("towns: all, then those on the coast, on navigable water, beside a lake; lake: lake cells; navc: navigable cells;");
    println!("under a run, any railway line that took eight years or more to open");
    println!("{:>4} {:>5} {:>6} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5}  {:>14} {:>5} {:>5}", "seed", "land", "km", "nav", "river", "tide", "shore", "coast", "twin", "towns", "lake", "navc");
    for &seed in &seeds {
        for land in [false, true] {
            let mut P = Rules::Game.params();
            P.seed = seed;
            P.n_families = families;
            P.land.enabled = land;
            for (k, v) in &sets {
                let mut json = v.clone();
                for part in k.rsplit('.') {
                    json = format!("{{\"{part}\": {json}}}");
                }
                P = P.with_json_overrides(&json).unwrap_or_else(|e| panic!("--set {k}={v}: {e}"));
            }
            let r = sim::run_with(&P, Rules::Game.content(), false, true).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
            let st = &r.state;
            let w = &st.world;
            let N = st.N;
            let lf = w.landform.as_deref();
            let estuary = |c: usize| lf.is_some_and(|l| l.estuary[c]);
            let shore = |c: usize| lf.is_some_and(|l| l.shore[c]);
            let links = canals::links_out(st);
            let open: Vec<_> = links.iter().filter(|l| l.opened.is_some()).collect();
            let mut on = vec![false; N];
            let mut twin = 0;
            for l in &open {
                on[l.a] = true;
                on[l.b] = true;
                if w.navigable[l.a] && w.navigable[l.b] {
                    twin += 1;
                }
            }
            let cells: Vec<usize> = (0..N).filter(|&c| on[c]).collect();
            // 2 km a cell with canal in it, as the sweep's `canal_km` counts
            let km = 2.0 * cells.len() as f64;
            let share = |f: &dyn Fn(usize) -> bool| if cells.is_empty() { 0.0 } else { cells.iter().filter(|&&c| f(c)).count() as f64 / cells.len() as f64 };
            let towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.town_node[t]).collect();
            let tc = |f: &dyn Fn(usize) -> bool| towns.iter().filter(|&&c| f(c)).count();
            println!(
                "{:>4} {:>5} {:>6.0} {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>5}  {:>2} {:>3} {:>3} {:>3} {:>5} {:>5}",
                seed,
                if land { "new" } else { "port" },
                km,
                share(&|c| w.navigable[c]),
                share(&|c| w.river[c]),
                share(&|c| estuary(c)),
                share(&|c| shore(c)),
                share(&|c| w.coast[c]),
                twin,
                towns.len(),
                tc(&|c| w.coast[c]),
                tc(&|c| w.navigable[c]),
                tc(&|c| shore(c)),
                lf.map_or(0, |l| l.lake.iter().filter(|&&x| x).count()),
                (0..N).filter(|&c| w.navigable[c]).count(),
            );
            for l in st.rw.lines.iter().filter(|l| l.opened - l.start as f64 >= 8.0) {
                println!(
                    "       slow railway line: promoted {}, opened {:.1}, serves {}, towns {} and {}, {} new cells, expected {:.0}, spent {:.0}",
                    l.start,
                    l.opened,
                    ways::serves_word(l.serves),
                    st.town_name[l.a],
                    st.town_name[l.b],
                    l.new_cells,
                    l.cost,
                    l.spent
                );
            }
        }
    }
}
