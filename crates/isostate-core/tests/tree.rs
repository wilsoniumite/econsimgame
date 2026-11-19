//! The production tree (docs/rules/tree.md): an overlay laid over the port's
//! content by name, up to a stage. Off it lays nothing; on, the island has
//! builders, sawyers, brewers and the rest, the books stay straight and a run
//! is the same run twice.
#![allow(non_snake_case)]

use isostate_core::content::method_index;
use isostate_core::econ::{service_mix, service_mix_widths};
use isostate_core::families::Group;
use isostate_core::mat::Mat;
use isostate_core::sim::{self, RunResult};
use isostate_core::tree::{self, GAME_OVERLAY_TOML, TreeParams};
use isostate_core::{Content, Params, default_content, default_params, game_content, game_params, observer};

/// The content without its one table that has no order of its own.
fn printed(C: &Content) -> String {
    let mut C = C.clone();
    C.good_index.clear();
    format!("{C:?}")
}

fn game(seed: u64, families: usize, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = Some(end_year);
    P
}

#[test]
fn off_by_default_and_on_in_the_game() {
    assert_eq!(default_params().tree, TreeParams::default());
    assert_eq!(default_params().tree.stage, 0);
    assert_eq!(game_params().tree.stage, tree::GAME_STAGE);
    // off, the content a run is played on is the content given
    for (P, C) in [(default_params(), default_content()), (default_params(), game_content())] {
        assert_eq!(printed(&tree::content(&P, C.clone()).unwrap()), printed(&C));
    }
    let mut P = game_params();
    P.tree.stage = 0;
    assert_eq!(printed(&tree::content(&P, game_content()).unwrap()), printed(&game_content()));
    // and a port run's params.json does not name it
    assert!(!serde_json::to_string(&default_params()).unwrap().contains("tree"));
    // the canals' clock is the canals' own without a stage, and for coal with one
    let (off, on) = (game_content(), tree::game_content_at(1));
    let P = game_params();
    assert_eq!(default_params().tree.canal_stone, 1.0);
    assert_eq!((tree::canal_clock(&off, &P, "stone"), tree::canal_clock(&off, &P, "coal")), (1.0, 1.0));
    assert_eq!((tree::canal_clock(&on, &P, "stone"), tree::canal_clock(&on, &P, "coal")), (P.tree.canal_stone, 1.0));
    assert!(P.tree.canal_stone > 0.0 && P.tree.canal_stone < 1.0);
}

#[test]
fn stage_one_adds_to_the_port_and_moves_nothing_of_it() {
    let port = game_content();
    let C = tree::game_content_at(1);
    assert_eq!(C.stage, 1);
    assert_eq!((port.G, port.S, port.NS, port.M), (16, 6, 7, 16));
    assert_eq!((C.G, C.S, C.NS), (25, 8, 7));
    assert!(C.M > 40, "{} methods", C.M);
    // the port's goods, needs, sectors and methods keep their numbers
    assert_eq!(C.good_name[..port.G], port.good_name[..]);
    assert_eq!(C.service_name[..port.S], port.service_name[..]);
    assert_eq!(C.sector_name, port.sector_name);
    assert_eq!(C.method_name[..port.M], port.method_name[..]);
    // the inventions that are not works come after every method, old and new
    assert_eq!(C.tech_name[C.M..], port.tech_name[port.M..]);
    assert_eq!(C.tech_name[..C.M], C.method_name[..]);
    for g in ["building", "lumber", "lime", "beer", "pottery", "gas", "chemicals", "glass", "paper"] {
        let g = C.gi(g);
        assert!(g >= port.G && C.decay[g].is_some() && C.stock0[g].is_some());
        assert!((0..C.M).any(|m| C.method_out[m] == g && C.tech_known[m]) || C.good_name[g] == "gas", "{} is made in 1750", C.good_name[g]);
    }
    // a house is built where it stands and gas goes as far as its pipes
    assert!(!C.traded[C.gi("building")] && !C.traded[C.gi("gas")] && C.traded[C.gi("beer")] && C.traded[C.gi("grain")]);
    // the port's goods keep the port's decay and opening stock, which the content does not say
    let P = game_params();
    let decay = tree::decay(&C, &P);
    for g in 0..port.G {
        assert!(C.decay[g].is_none() && C.stock0[g].is_none());
        assert_eq!(decay[g], P.decay[g]);
    }
    assert_eq!(decay[C.gi("building")], 0.1);
    // household goods are less alike than fuels
    let widths = tree::widths(&C, &P);
    assert_eq!(widths[C.service_name.iter().position(|n| n == "wares").unwrap()], 2.0);
    assert_eq!(widths[0], P.logit_theta);
    assert_eq!(tree::widths(&port, &P), vec![P.logit_theta; port.S]);
    // an entry that names a method of the port changes only what it writes
    let (old, new) = (method_index(&port, "smithy"), method_index(&C, "smithy"));
    assert_eq!((port.m_K0[old], C.m_K0[new]), (5.0, 10.0));
    assert_eq!(C.m_in.row(new)[..port.G], port.m_in.row(old)[..]);
    assert_eq!((C.m_labor[new], C.m_heat[new], C.method_label[new].as_str()), (port.m_labor[old], port.m_heat[old], port.method_label[old].as_str()));
    // the shares of every need still add to one, at every income
    for shares in [&C.shares, &C.shares_rich, &C.lord_shares] {
        assert!((shares.iter().sum::<f64>() - 1.0).abs() < 1e-9 && shares.len() == C.S);
    }
    // each new works says whose trade it is, so a family can own one
    assert_eq!(Group::of_method(&C, method_index(&C, "masons")), Some(Group::Building));
    assert_eq!(Group::of_method(&C, method_index(&C, "gasworks")), Some(Group::CoalMining));
    assert_eq!(Group::of_method(&C, method_index(&C, "brewhouse")), Some(Group::Estates));
    assert_eq!(Group::of_method(&C, method_index(&C, "emporium")), None, "shops belong to no family trade");
    assert_eq!(Group::of_method(&C, method_index(&C, "smithy")), Some(Group::Engineering), "the port's own are as they were");
    for m in port.M..C.M {
        assert!(C.m_K0[m] > 0.0 && C.m_labor[m] > 0.0 && (0..C.G).any(|g| C.m_cap[(m, g)] > 0.0), "{}", C.method_name[m]);
        assert!(C.tech_known[m] || (0..C.G).any(|g| C.tech_disc[(m, g)] != 0.0), "{} can be found", C.method_name[m]);
    }
    // laying it twice is laying it once
    assert_eq!(printed(&tree::content(&P, C.clone()).unwrap()), printed(&C));
    assert_eq!(printed(&tree::lay(&C, GAME_OVERLAY_TOML, 1).unwrap()), printed(&C));
}

/// A small overlay with a good and a method at each of two stages.
const TWO_STAGES: &str = r##"
[meta]
goods = [
  { name = "ale", label = "Ale", weight = 2.0, price = 2.0, color = "#b8742a", decay = 0.06, stock0 = 0.07 },
  { name = "malt", label = "Malt", weight = 1.0, price = 1.5, color = "#c8a060", stage = 2 },
]

[services.props]
ale = { food = 0.5 }
malt = { food = 0.2 }

[[method]]
name = "alehouse"
label = "Alehouse"
output = "ale"
inputs = { grain = 1.2, malt = 0.1 }
labor = 0.1
capital = { timber = 3 }
K0 = 100.0
known = true
group = "estates"

[[method]]
name = "maltings"
label = "Maltings"
output = "malt"
inputs = { grain = 1.1 }
labor = 0.05
capital = { bricks = 2 }
K0 = 100.0
known = true
stage = 2

[[method]]
name = "brickworks"
labor = 0.123
stage = 2
"##;

#[test]
fn a_stage_is_a_ceiling_and_what_lies_above_it_is_not_there() {
    let port = default_content();
    let one = tree::lay(&port, TWO_STAGES, 1).unwrap();
    let two = tree::lay(&port, TWO_STAGES, 2).unwrap();
    assert_eq!((one.G, one.M, two.G, two.M), (port.G + 1, port.M + 1, port.G + 2, port.M + 2));
    // at stage 1 malt does not exist: the alehouse brews without it, and nothing serves food with it
    let ale = method_index(&one, "alehouse");
    assert!(!one.good_index.contains_key("malt"));
    assert_eq!(one.m_in.row(ale).iter().filter(|&&v| v > 0.0).count(), 1);
    assert_eq!(two.m_in[(method_index(&two, "alehouse"), two.gi("malt"))], 0.1);
    assert_eq!(two.props[(two.gi("malt"), 0)], 0.2);
    // a change to a method of the port waits for its stage too
    let brick = method_index(&port, "brickworks");
    assert_eq!(one.m_labor[brick], port.m_labor[brick]);
    assert_eq!(two.m_labor[brick], 0.123);
    assert_eq!(two.m_in.row(brick)[..port.G], port.m_in.row(brick)[..]);
    assert_eq!((one.stage, two.stage), (1, 2));
}

#[test]
fn an_overlay_that_is_wrong_says_so() {
    let port = default_content();
    let bad = |text: &str| tree::lay(&port, text, 1).err().unwrap_or_else(|| panic!("this should not load:\n{text}"));
    // a good nobody has
    assert!(bad("[[method]]\nname = \"smithy\"\ninputs = { mithril = 1.0 }\n").contains("mithril"));
    // a new good or method with half its numbers
    assert!(bad("[meta]\ngoods = [ { name = \"ale\", label = \"Ale\" } ]\n").contains("weight"));
    assert!(bad("[[method]]\nname = \"alehouse\"\noutput = \"grain\"\n").contains("label"));
    assert!(bad("[[method]]\nname = \"alehouse\"\nlabel = \"Alehouse\"\noutput = \"grain\"\n").contains("labor"));
    // needs that do not begin with the port's own, or lists of the wrong length
    assert!(bad("[services]\nnames = [\"warmth\", \"food\", \"shelter\", \"clothing\", \"implements\", \"comfort\"]\n").contains("needs"));
    assert!(bad("[services]\nnames = [\"food\", \"warmth\", \"shelter\", \"clothing\", \"implements\", \"comfort\", \"table\"]\n").contains("7 needs"));
    // a key the loader does not know, and a family trade that is not one
    assert!(bad("[[method]]\nname = \"smithy\"\nlabour = 2.0\n").contains("labour"));
    assert!(bad("[[method]]\nname = \"smithy\"\ngroup = \"alchemy\"\n").contains("alchemy"));
    // the port's file itself is not an overlay of anything missing: it lays cleanly over itself
    assert!(tree::lay(&port, "", 3).is_ok());
}

#[test]
fn one_width_for_every_need_is_the_ports_mix() {
    let C = default_content();
    let mut price = Mat::zeros(3, C.G);
    for t in 0..3 {
        for g in 0..C.G {
            price[(t, g)] = C.base_price[g] * (0.6 + 0.31 * ((g * 5 + t * 3) % 7) as f64);
        }
    }
    let (a, b) = (service_mix(&price, &C.props, 0.25), service_mix_widths(&price, &C.props, &vec![0.25; C.S]));
    assert_eq!(a, b);
    // a wider logit leaves more of a need with the dearer goods
    let wide = service_mix_widths(&price, &C.props, &vec![2.0; C.S]);
    let (S, warmth) = (C.S, 1);
    let dearest = |share: &Mat| (0..C.G).filter(|&g| C.props[(g, warmth)] > 0.0).map(|g| share[(0, g * S + warmth)]).fold(1.0, f64::min);
    assert!(dearest(&wide.0) > dearest(&a.0));
}

fn run(P: &Params) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(P, game_content(), false, true).expect("money is conserved with the tree laid")
}

fn year_mean(r: &RunResult, name: &str, year: i64) -> f64 {
    observer::annual(r.state.series.get(name), r.content.start_year, year)
}

#[test]
fn the_island_opens_with_builders_and_they_take_the_quarries_work() {
    let on = run(&game(3, 0, 1760));
    let mut P = game(3, 0, 1760);
    P.tree.stage = 0;
    let off = run(&P);
    assert_eq!((off.content.G, on.content.G), (16, 25));
    assert!(!off.state.series.index.contains_key("out_building"), "without the tree a run does not know of building work");
    // 1750 opens with the trades in place: builders, sawyers, lime burners, brewers, potters
    for good in ["building", "lumber", "lime", "beer", "pottery", "chemicals", "glass", "paper"] {
        assert!(year_mean(&on, &format!("out_{good}"), 1750) > 0.0, "{good} is made in 1750");
    }
    assert_eq!(year_mean(&on, "out_gas", 1759), 0.0, "nobody knows how to make gas yet");
    // households buy their shelter from builders, and the island digs less stone for it
    let share = year_mean(&on, "share_shelter_building", 1759);
    assert!(share > 0.8, "builders make {share:.2} of shelter in 1759");
    let (stone_on, stone_off) = (year_mean(&on, "out_stone", 1759), year_mean(&off, "out_stone", 1759));
    assert!(stone_on < 0.8 * stone_off, "stone dug a month in 1759: {stone_on:.0} with builders, {stone_off:.0} without");
    // and the builders are a town trade of some size
    let C = &on.content;
    let ms = &on.state.month_stats;
    let builders: f64 = (0..C.M).filter(|&m| C.method_out[m] == C.gi("building")).map(|m| ms.firm_labor_by_method[m]).sum();
    let hands: f64 = ms.employment.iter().sum();
    assert!(builders > 0.03 * hands, "{builders:.0} builders among {hands:.0} hands");
}

#[test]
fn money_is_conserved_and_runs_are_deterministic_with_the_tree_laid() {
    // forty years with six families: the first inventions on the new goods are
    // found and built, and families found works in the new trades
    let (a, b) = (run(&game(5, 6, 1790)), run(&game(5, 6, 1790)));
    assert_eq!(a.state.series.names, b.state.series.names);
    for (k, name) in a.state.series.names.iter().enumerate() {
        let (x, y) = (&a.state.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs between identical runs");
    }
    assert_eq!(a.events.len(), b.events.len());
    // no price of a good that is made stands at its floor at the end (the floor as
    // the run has it: under longrun it rides on the island's cost of living,
    // docs/rules/longrun.md)
    let (st, C) = (&a.state, &a.content);
    for g in (0..C.G).filter(|&g| st.month_stats.prod[g] > 0.0) {
        let floor = isostate_core::longrun::floor(st, &a.params, C, g);
        for t in (0..st.T_max).filter(|&t| st.town_active[t]) {
            assert!(st.price[(t, g)] > 1.01 * floor, "{} in town {t}: {} against a floor of {floor}", C.good_name[g], st.price[(t, g)]);
        }
    }
}

#[test]
fn the_tree_alone_runs_under_the_ports_rules() {
    // `--rules port --set tree.stage=1`: the overlay over the port's content, no other mechanism
    let mut P = default_params();
    P.seed = 2;
    P.end_year = Some(1775);
    P.tree.stage = 1;
    let r = sim::run_with(&P, default_content(), false, true).expect("money is conserved");
    assert_eq!(r.content.G, 25);
    assert!(year_mean(&r, "out_building", 1774) > 0.0);
}

/// Bricks, of what each town's households and works at work asked for, the
/// share the market had: the mean over the months to `end_year`, each town
/// counted by what was asked of its market. The orders for plant are placed as
/// the year turns, so the shortage is the spring's and has to be read month
/// by month: the year is `sim::run_year`'s, taken apart.
fn bricks_for_the_days_trade(P: &Params) -> f64 {
    let mut s = sim::start(P, game_content());
    s.check_money = true;
    let bricks = s.content.gi("bricks");
    let (mut sum, mut n) = (0.0, 0.0);
    while s.year <= s.end_year {
        for month in 0..12 {
            sim::run_month(&mut s, month).expect("money is conserved with plant waiting");
            for t in (0..s.st.T_max).filter(|&t| s.st.town_active[t]) {
                let asked = s.st.requests[(t, bricks)];
                sum += asked * s.st.fill[(t, bricks)];
                n += asked;
            }
        }
        let year = s.year;
        let sim::Sim { st, content: C, params: P, rngs, events, .. } = &mut s;
        sim::yearly(st, C, P, rngs, year, events, false);
        observer::yearly(st, C, P, year, events);
        s.year += 1;
    }
    sum / n
}

#[test]
fn plant_waits_and_the_bricklayers_are_served() {
    assert!(!default_params().tree.plant_waits && game_params().tree.plant_waits);
    // seventy years with six families founding works: the island is short of
    // bricks, which every new works wants for its walls and the bricklayers
    // want every day
    let on = game(3, 6, 1820);
    let mut off = on.clone();
    off.tree.plant_waits = false;
    let (served, shared) = (bricks_for_the_days_trade(&on), bricks_for_the_days_trade(&off));
    assert!(shared < 0.95, "sharing alike, the market has {shared:.3} of the bricks asked for");
    assert!(served > shared + 0.03, "with plant waiting the day's trade has {served:.3} of its bricks; sharing alike, {shared:.3}");
}

#[test]
fn without_a_stage_plant_waiting_does_nothing() {
    // the game without the tree is `tree.stage = 0` and nothing more: plant
    // waiting and the canals' stone clock stay where the game's rules put
    // them, and the run is the same run to the last bit with them put back
    // (to 1780, by when the first canal has been surveyed and dug)
    let mut with = game(4, 6, 1780);
    with.tree.stage = 0;
    assert!(with.tree.plant_waits && with.tree.canal_stone < 1.0);
    let mut without = with.clone();
    without.tree.plant_waits = false;
    without.tree.canal_stone = 1.0;
    let (a, b) = (run(&with), run(&without));
    assert_eq!(a.content.G, 16);
    assert_eq!(a.state.series.names, b.state.series.names);
    for (k, name) in a.state.series.names.iter().enumerate() {
        let (x, y) = (&a.state.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs");
    }
    assert_eq!(a.events.len(), b.events.len());
}
