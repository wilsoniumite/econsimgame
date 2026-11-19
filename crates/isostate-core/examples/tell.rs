//! What the simulation knows of a town, a works, a square or the island
//! (`inspect::tell`, `docs/designs/sandbox.md` section 6.2), printed as the card
//! the viewer's inspector will show, with how long it took to make.
//!
//!     cargo run --release -p isostate-core --example tell -- [--seed 7] [--year 1800]
//!         [--month 1] (--town NAME | --works N | --cell X,Y | --island | --list)
//!
//! A game of six families under the game's rules (`save::Game`, as the viewer
//! starts one) is played to January of `--year`, or `--month` months on into
//! it (1 is January, nothing of the year played; 7 is July, six months played).
//! `--list` names the towns and the ten largest works, with their numbers, to
//! choose from. Several of the four may be given; each card is printed in turn.
//! The island's card ends with what the hand has done ("By hand"), which in a
//! game played here with no edits reads "none yet".
#![allow(non_snake_case)]
use isostate_core::families;
use isostate_core::inspect::{self, Thing, Told};
use isostate_core::save::Game;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut year, mut month) = (7u64, 1800i64, 1usize);
    let (mut things, mut list) = (Vec::<(String, String)>::new(), false);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seed" => seed = next().parse().expect("a seed"),
            "--year" => year = next().parse().expect("a year"),
            "--month" => month = next().parse().expect("a month, 1 to 12"),
            "--town" | "--works" | "--cell" => things.push((args[i].clone(), next())),
            "--island" => things.push(("--island".into(), String::new())),
            "--list" => list = true,
            other => panic!("unknown option {other}"),
        }
        i += if matches!(args[i].as_str(), "--island" | "--list") { 1 } else { 2 };
    }
    assert!((1..=12).contains(&month), "--month is 1 to 12");
    let t0 = std::time::Instant::now();
    let mut g = Game::new(seed, 6, 0);
    while g.sim.year < year && !g.over() {
        g.year().expect("a year is played");
        g.turn();
    }
    for _ in 1..month {
        g.month().expect("a month is played");
    }
    println!("island {seed}, the game's rules, six families, played to {} in {:.1} s", inspect::as_of(&g.sim), t0.elapsed().as_secs_f64());
    let st = &g.sim.st;

    if list {
        println!("\ntowns:");
        for t in (0..st.town_name.len()).filter(|&t| st.town_active[t]) {
            println!("  {t:3}  {:<14} {:>8.0} people", st.town_name[t], st.pop[t]);
        }
        let mut works: Vec<(f64, usize)> = (0..st.F_count).filter(|&f| st.f_alive[f]).map(|f| (families::works_value(st, &g.sim.content, f), f)).collect();
        works.sort_by(|a, b| b.0.total_cmp(&a.0));
        println!("\nthe ten largest works, by their plant at today's prices:");
        for &(v, f) in works.iter().take(10) {
            println!("  {f:4}  {:<34} {:<24} {:<12} {v:>10.0}", st.f_name[f], g.sim.content.method_label[st.f_method[f]], st.town_name[st.f_town[f]]);
        }
    }

    for (opt, value) in &things {
        let thing = match opt.as_str() {
            "--town" => match st.town_name.iter().position(|n| n == value) {
                Some(t) => Thing::Town(t),
                None => {
                    println!("\nThere is no town called {value}.");
                    continue;
                }
            },
            "--works" => Thing::Works(value.parse().expect("a works' number")),
            "--cell" => {
                let (x, y) = value.split_once(',').expect("--cell X,Y");
                let (x, y): (usize, usize) = (x.trim().parse().expect("X"), y.trim().parse().expect("Y"));
                Thing::Cell(y * st.world.W + x)
            }
            _ => Thing::Island,
        };
        // once, as the viewer would ask; then a thousand times, for the mean
        let t0 = std::time::Instant::now();
        let told = inspect::tell(&g.sim, thing);
        let once = t0.elapsed().as_secs_f64();
        let t0 = std::time::Instant::now();
        for _ in 0..1000 {
            std::hint::black_box(inspect::tell(&g.sim, thing).ok());
        }
        let mean = t0.elapsed().as_secs_f64() / 1000.0;
        match told {
            Ok(told) => print_card(&told),
            Err(e) => println!("\n{e}"),
        }
        println!("  (made in {:.3} ms the first time, {:.3} ms on the mean of a thousand)", 1000.0 * once, 1000.0 * mean);
    }
}

fn print_card(told: &Told) {
    println!("\n{}", told.title);
    println!("As it stands, {}", told.as_of);
    let wide = told.groups.iter().flat_map(|g| &g.rows).map(|(n, _)| n.chars().count()).max().unwrap_or(0);
    for g in &told.groups {
        println!("\n  {}", g.name);
        for (name, figure) in &g.rows {
            println!("    {name:<wide$}  {figure}");
        }
    }
}
