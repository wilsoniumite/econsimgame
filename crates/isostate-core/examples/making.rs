//! A town's making (`making::town_making`, the viewer's Making panel) printed
//! in words: its boxes column by column, its largest ribbons, the kinds it
//! could have and has not, each good's balance, and how long the answer took.
//!
//!     cargo run --release -p isostate-core --example making -- [--seed 7] [--year 1800]
//!         [--to 2050] (--town NAME | --largest | --list | --every) [--said] [--all]
//!
//! A game of six families under the game's rules is played to January of
//! `--year` (`--to` the last year, 1900 unless said), as the viewer starts
//! one, and the town's books of the year before are read. `--largest` takes
//! the town with the most people; `--list` names the towns with their people
//! and their hands on the land and in works; `--every` gives a line a town,
//! its boxes of the land and the works by hands. `--said` prints what each box
//! and ribbon says under the pointer; `--all` prints every ribbon; `--hand`
//! what the panel's menus offer the hand for each kind of works; `--edits
//! FILE` gives a list of dated edits on the way, as the tell example does.
#![allow(non_snake_case)]
use isostate_core::inspect::{self, Thing};
use isostate_core::params::Rules;
use isostate_core::save::Game;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut year, mut to) = (7u64, 1800i64, 1900i64);
    let (mut towns, mut list, mut largest, mut said, mut all, mut every) = (Vec::<String>::new(), false, false, false, false, false);
    let (mut hand, mut edits) = (false, None::<String>);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        let flag = matches!(args[i].as_str(), "--list" | "--largest" | "--said" | "--all" | "--every" | "--hand");
        match args[i].as_str() {
            "--seed" => seed = next().parse().expect("a seed"),
            "--year" => year = next().parse().expect("a year"),
            "--to" => to = next().parse().expect("a year"),
            "--town" => towns.push(next()),
            "--list" => list = true,
            "--largest" => largest = true,
            "--said" => said = true,
            "--all" => all = true,
            "--every" => every = true,
            "--hand" => hand = true,
            "--edits" => edits = Some(next()),
            other => panic!("unknown option {other}"),
        }
        i += if flag { 1 } else { 2 };
    }
    let t0 = std::time::Instant::now();
    let mut g = Game::under_to(Rules::Game, seed, 6, 0, to.max(1900)).expect("a game to that year");
    // TOWN-FLOW-HAND: --edits FILE, dated edits given on the way, as the tell example gives them
    let mut dated: Vec<isostate_core::edits::Dated> = match &edits {
        Some(file) => serde_json::from_str(&std::fs::read_to_string(file).expect("the list of edits can be read")).expect("a JSON list of dated edits"),
        None => Vec::new(),
    };
    dated.sort_by_key(|d| d.before);
    let mut next = 0;
    while g.sim.year < year && !g.over() {
        while next < dated.len() && dated[next].before <= (g.sim.year, g.month + 1) {
            let edit = dated[next].edit.clone();
            next += 1;
            match g.give(isostate_core::save::Order::ByHand { edit }) {
                Ok(_) => println!("{}{}", g.sim.events.last().map_or("", |e| e.title.as_str()), g.sim.events.last().and_then(|e| e.firm).map_or(String::new(), |f| format!(" (works {f})"))),
                Err(why) => panic!("an edit was refused: {why}"),
            }
        }
        if g.month().expect("a month is played") {
            g.turn();
        }
    }
    println!("island {seed}, the game's rules, six families, played to {} in {:.1} s", inspect::as_of(&g.sim), t0.elapsed().as_secs_f64());
    let st = &g.sim.st;
    // the works built by hand on the way, and what became of them
    for f in (0..st.F_count).filter(|&f| st.f_reason[f].kind == "hand") {
        let fate = if st.f_alive[f] { format!("standing, plant {:.1} a month, cash {:.0}", st.f_K[f], st.f_cash[f]) } else { format!("closed in {}", st.f_died[f]) };
        println!("  built by hand: {} (works {f}), {} in {}: {fate}", st.f_name[f], g.sim.content.method_label[st.f_method[f]], st.f_born[f]);
    }
    let active: Vec<usize> = (0..st.town_name.len()).filter(|&t| st.town_active[t]).collect();
    if list {
        for &t in &active {
            println!("  {t:3}  {:<16} {:>9.0} people, {:>7.0} on the land, {:>7.0} in works", st.town_name[t], st.pop[t], st.ag_labor[t], st.ind_labor[t]);
        }
    }
    // --every: a line a town, its boxes of the land and the works with their hands
    if every {
        for &t in &active {
            match inspect::tell(&g.sim, Thing::Making(t)).map(|x| x.making) {
                Ok(Some(m)) => {
                    let mut kinds: Vec<(f64, String)> = m.nodes.iter().filter(|x| x.kind.makes()).map(|x| (x.hands, x.label.clone())).collect();
                    kinds.sort_by(|a, b| b.0.total_cmp(&a.0));
                    let words: Vec<String> = kinds.iter().map(|(h, l)| format!("{l} {h:.0}")).collect();
                    println!("  {:<14} {:>7.0} people: {}", m.name, st.pop[t], words.join(", "));
                    // TOWN-FLOW-HAND: with --hand, the faint kinds and what Build one here would build
                    if hand {
                        for f in &m.faint {
                            let offer = match m.hand_of(f.good).map(|k| &k.build) {
                                Some(Ok(o)) => format!("{} on {}, {} for {:.0}, {:.0}% a year", o.label.to_lowercase(), o.cell % st.world.W, o.cell / st.world.W, o.cost, 100.0 * o.ret),
                                Some(Err(why)) => format!("none: {why}"),
                                None => "no hand".to_string(),
                            };
                            println!("      (faint{}) {}: {offer}", if f.known { "" } else { ", found nowhere" }, f.label);
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => println!("  {}: {e}", st.town_name[t]),
            }
        }
    }
    if largest {
        let t = active.iter().copied().max_by(|&a, &b| st.pop[a].total_cmp(&st.pop[b])).expect("a town");
        towns.push(st.town_name[t].clone());
    }
    for name in &towns {
        let Some(t) = st.town_name.iter().position(|n| n == name) else {
            println!("\nThere is no town called {name}.");
            continue;
        };
        let t1 = std::time::Instant::now();
        let told = inspect::tell(&g.sim, Thing::Making(t));
        let once = t1.elapsed().as_secs_f64();
        let t1 = std::time::Instant::now();
        for _ in 0..100 {
            std::hint::black_box(inspect::tell(&g.sim, Thing::Making(t)).ok());
        }
        let mean = t1.elapsed().as_secs_f64() / 100.0;
        let m = match told {
            Ok(told) => told.making.expect("a making"),
            Err(e) => {
                println!("\n{e}");
                continue;
            }
        };
        println!("\n{}", m.said);
        println!("{} boxes, {} ribbons ({} run back), {} faint kinds, {} columns; {:.0} hands", m.nodes.len(), m.ribbons.len(), m.ribbons.iter().filter(|r| r.back).count(), m.faint.len(), m.cols, m.hands);
        for c in 0..m.cols {
            println!("  column {c}:");
            for n in m.nodes.iter().filter(|n| n.col == c) {
                println!("    {:<34} {:>9.0} hands {:>6.1} works  in {:>12.0}  out {:>12.0}", n.label, n.hands, n.works, n.value_in, n.value_out);
                if said {
                    for l in &n.said {
                        println!("        {l}");
                    }
                }
            }
            for f in m.faint.iter().filter(|f| f.col == c) {
                println!("    (faint) {}", f.label);
                if said {
                    for l in &f.said {
                        println!("        {l}");
                    }
                }
            }
        }
        let mut rs: Vec<&isostate_core::making::Ribbon> = m.ribbons.iter().filter(|r| r.from != r.to).collect();
        rs.sort_by(|a, b| b.value.total_cmp(&a.value));
        let total: f64 = rs.iter().map(|r| r.value).sum();
        println!("  ribbons, the largest first (of {total:.0} in all):");
        for r in rs.iter().take(if all { usize::MAX } else { 15 }) {
            println!("    {:<14} {:<30} -> {:<30} {:>12.0}{}", m.good_label[r.good], m.nodes[r.from].label, m.nodes[r.to].label, r.value, if r.back { "  (back)" } else { "" });
            if said {
                for l in &r.said {
                    println!("        {l}");
                }
            }
        }
        // each good: in (made, came in, out of store) against out (bought, sold away, into store),
        // from the drawing's own ribbons
        let mut worst = 0.0f64;
        for gf in &m.goods {
            let into: f64 = m.ribbons.iter().filter(|r| r.good == gf.good).map(|r| r.qty).sum();
            let books_in = gf.made + gf.received + gf.from_store;
            let books_out = gf.bought + gf.sent + gf.to_store;
            let off = (into - books_in).abs().max((into - books_out).abs()) / books_in.max(books_out).max(1e-9);
            worst = worst.max(off);
        }
        println!("  each good's ribbons against its books: the worst differs by {:.2e} of what flows", worst);
        println!("  (made in {:.3} ms the first time, {:.3} ms on the mean of a hundred)", 1000.0 * once, 1000.0 * mean);
        // TOWN-FLOW-HAND: --hand, what the panel's menus offer for each kind
        if hand {
            println!("  what the hand may do, kind by kind:");
            for k in &m.hand {
                let build = match &k.build {
                    Ok(o) => format!("build {} on {}, {}, for {:.0}, {:.0}% a year", o.label.to_lowercase(), o.cell % st.world.W, o.cell / st.world.W, o.cost, 100.0 * o.ret),
                    Err(why) => format!("none offered: {why}"),
                };
                println!("    {:<22} {} standing; {build}", m.good_label[k.good], k.standing.len());
                for x in &k.methods {
                    let state = if x.forbidden {
                        "turned off by hand".to_string()
                    } else if x.standing > 0 {
                        format!("in use, {} works", x.standing)
                    } else if x.known {
                        "known, used by none".to_string()
                    } else if x.discover.is_some() {
                        "found nowhere yet".to_string()
                    } else {
                        format!("known in {} towns, not here", x.towns)
                    };
                    println!("        {:<30} {state}", x.label);
                }
                for l in &k.by_hand {
                    println!("        {l}");
                }
            }
        }
    }
}
