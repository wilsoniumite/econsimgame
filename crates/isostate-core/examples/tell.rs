//! What the simulation knows of a town, a works, a square or the island
//! (`inspect::tell`, `docs/designs/sandbox.md` section 6.2), printed as the card
//! the viewer's inspector will show, with how long it took to make.
//!
//!     cargo run --release -p isostate-core --example tell -- [--seed 7] [--year 1800]
//!         [--month 1] [--edits FILE] (--town NAME | --works N | --cell X,Y | --island | --list)
//!
//! A game of six families under the game's rules (`save::Game`, as the viewer
//! starts one) is played to January of `--year`, or `--month` months on into
//! it (1 is January, nothing of the year played; 7 is July, six months played).
//! `--list` names the towns and the ten largest works, with their numbers, to
//! choose from. Several of the four may be given; each card is printed in turn.
//! The island's card ends with what the hand has done ("By hand"), which in a
//! game played here with no edits reads "none yet".
//!
//! A town's card ends with its ideas (`inspect::town_ideas`), dated by what
//! is watched at each year's end on the way (`inspect::Learnt`), as the
//! viewer's thread watches it; `--said` prints what a click on each says.
//!
//! `--edits FILE` gives the game a list of dated edits on the way, in the
//! form `edits::play` reads (`[{"before": [1800, 1], "edit": {...}}]`), each
//! as an edit given before its month is played; the line of each is printed.
#![allow(non_snake_case)]
use isostate_core::edits::Dated;
use isostate_core::families;
use isostate_core::inspect::{self, Thing, Told};
use isostate_core::save::{Game, Order};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut year, mut month) = (7u64, 1800i64, 1usize);
    let (mut things, mut list, mut edits, mut said) = (Vec::<(String, String)>::new(), false, None::<String>, false);
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
            "--said" => said = true,
            "--edits" => edits = Some(next()),
            other => panic!("unknown option {other}"),
        }
        i += if matches!(args[i].as_str(), "--island" | "--list" | "--said") { 1 } else { 2 };
    }
    assert!((1..=12).contains(&month), "--month is 1 to 12");
    let t0 = std::time::Instant::now();
    let mut g = Game::new(seed, 6, 0);
    let mut learnt = inspect::Learnt::default();
    // the edits of a list, each given before its month is played, as `edits::play` carries them out
    let mut dated: Vec<Dated> = match &edits {
        Some(file) => serde_json::from_str(&std::fs::read_to_string(file).expect("the list of edits can be read")).expect("a JSON list of dated edits"),
        None => Vec::new(),
    };
    dated.sort_by_key(|d| d.before);
    let mut next = 0;
    let mut give_due = |g: &mut Game| {
        while next < dated.len() && dated[next].before <= (g.sim.year, g.month + 1) {
            let d = &dated[next];
            next += 1;
            match g.give(Order::ByHand { edit: d.edit.clone() }) {
                // (a works built or closed by hand is named by its number, for --works)
                Ok(_) => match g.sim.events.last() {
                    Some(e) => println!("{}{}", e.title, e.firm.map_or(String::new(), |f| format!(" (works {f})"))),
                    None => {}
                },
                Err(why) => panic!("the edit before month {} of {}, {:?}, was refused: {why}", d.before.1, d.before.0, d.edit),
            }
        }
    };
    while g.sim.year < year && !g.over() {
        give_due(&mut g);
        if g.month().expect("a month is played") {
            learnt.year_end(&g.sim);
            g.turn();
        }
    }
    for _ in 1..month {
        give_due(&mut g);
        g.month().expect("a month is played");
    }
    give_due(&mut g);
    println!("island {seed}, the game's rules, six families, played to {} in {:.1} s", inspect::as_of(&g.sim), t0.elapsed().as_secs_f64());
    let st = &g.sim.st;

    if list {
        println!("\ntowns:");
        for t in (0..st.town_name.len()).filter(|&t| st.town_active[t]) {
            let node = st.town_node[t];
            println!("  {t:3}  {:<14} {:>8.0} people, its market place on the square at {}, {}", st.town_name[t], st.pop[t], node % st.world.W, node / st.world.W);
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
        let told = inspect::tell_with(&g.sim, thing, Some(&learnt));
        let once = t0.elapsed().as_secs_f64();
        let t0 = std::time::Instant::now();
        for _ in 0..1000 {
            std::hint::black_box(inspect::tell_with(&g.sim, thing, Some(&learnt)).ok());
        }
        let mean = t0.elapsed().as_secs_f64() / 1000.0;
        match told {
            Ok(told) => print_card(&told, said),
            Err(e) => println!("\n{e}"),
        }
        println!("  (made in {:.3} ms the first time, {:.3} ms on the mean of a thousand)", 1000.0 * once, 1000.0 * mean);
    }
}

fn print_card(told: &Told, said: bool) {
    println!("\n{}", told.title);
    println!("As it stands, {}", told.as_of);
    let wide = told.groups.iter().flat_map(|g| &g.rows).map(|(n, _)| n.chars().count()).max().unwrap_or(0);
    for g in &told.groups {
        println!("\n  {}", g.name);
        for (name, figure) in &g.rows {
            println!("    {name:<wide$}  {figure}");
        }
    }
    if let Some(ideas) = &told.ideas {
        let n = |s: inspect::Stands| ideas.count(s);
        println!(
            "\n  Its ideas, {} to {}: {} known here, {} known elsewhere, {} found nowhere yet",
            ideas.from,
            ideas.to,
            n(inspect::Stands::Here),
            n(inspect::Stands::Elsewhere),
            n(inspect::Stands::Nowhere)
        );
        for g in &ideas.groups {
            println!("    {}", g.name);
            for i in &g.ideas {
                let year = i.year.map_or(String::new(), |y| format!("{y} "));
                let state = format!("{:?}", i.stands);
                println!("      {state:<9} {year}{} ({}){}", i.label, i.how, if i.discover { ", may be found here by hand" } else { "" });
                if said {
                    println!("                {}", i.said);
                }
            }
        }
    }
}
