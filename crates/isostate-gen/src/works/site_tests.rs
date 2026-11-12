//! The tests of `works::site` (a file of their own, for its length).

use super::*;
use crate::site::Model;

const SIZES: [f32; 6] = [0.4, 1.0, 3.0, 10.0, 30.0, 100.0];

/// Prints a plan, for looking at what the planner did with one works
/// (`KIND=7 SIZE=10 PLOT=2 cargo test -p isostate-gen show_a_plan -- --ignored --nocapture`).
#[test]
#[ignore]
fn show_a_plan() {
    let pal = Palette::standard();
    let env = |name: &str, or: f32| std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(or);
    let kind = Kind::ALL[env("KIND", 0.0) as usize];
    let size = env("SIZE", 3.0);
    let j = env("PLOT", 0.0) as usize;
    let i = SIZES.iter().position(|s| *s == size).unwrap_or(0);
    let (plan, _) = specimen(kind, env("FOUNDED", 1760.0) as i32, 1890, size, &plots(kind, size)[j], Seed::of(kind as u32 * 31 + i as u32 + j as u32), &pal);
    println!("{kind:?} on {:?}\nstages {:?}\ncart yard {:?} lane {:?} siding {:?} wharf {:?}", plan.plot.outline, plan.stages, plan.cart_yard, plan.lane, plan.siding, plan.wharf);
    for p in &plan.parts {
        println!("{:?} of stage {} at {:.3}, {:.3}: {:.3} by {:.3}, turned {}, size {:.2}", p.role, p.stage, p.x, p.z, p.w, p.d, p.turn, p.size);
    }
    for y in &plan.yards {
        println!("{:?} at {:.3}, {:.3}: {:.3} by {:.3}", y.hold, y.x, y.z, y.w, y.d);
    }
    println!("{} dropped, {} squeezed, taken in {:?}", plan.dropped, plan.squeezed, plan.enclosed.outline);
}

/// The plots the lab shows a works on: square, by a stream, aslant by a railway, a corner cut, on navigable water.
fn plots(kind: Kind, size: f32) -> Vec<Plot> {
    let (w, d) = ground(kind, size);
    vec![
        Plot::rect(w, d),
        Plot::rect(w, d).facing(1, Edge::Water { navigable: false }),
        Plot::wedge(w * 1.15, d * 1.15, 0.25).facing(2, Edge::Rail),
        Plot::cut(w * 1.05, d * 1.05, 0.35),
        Plot::rect(w * 1.3, d * 0.95).facing(3, Edge::Water { navigable: true }).facing_from(2, Edge::Rail, 1850),
    ]
}

fn draw(plan: &Plan, now: &Now, yaw: f32) -> Model {
    let mut m = Model::default();
    site(&mut m.solid, &mut m.smoke, &mut m.flat, &Site::level(Vec3::ZERO, yaw), plan, now, &Palette::standard());
    m
}

/// Every kind at every size, founded early, in the middle and late, on every plot.
fn all(mut each: impl FnMut(&Plan, Now)) {
    let pal = Palette::standard();
    for kind in Kind::ALL {
        for (i, size) in SIZES.into_iter().enumerate() {
            for founded in [1760, 1820, 1870] {
                for (j, plot) in plots(kind, size).into_iter().enumerate() {
                    // (not every plot at every size and year, to keep the tests quick)
                    if (i + j + founded as usize / 10) % 2 == 0 && j > 0 {
                        continue;
                    }
                    let (plan, now) = specimen(kind, founded, 1890, size, &plot, Seed::of(kind as u32 * 31 + i as u32 + j as u32), &pal);
                    each(&plan, now);
                }
            }
        }
    }
}

#[test]
fn the_shipped_content_gives_every_kind_its_process() {
    for kind in Kind::ALL {
        if kind != Kind::Other {
            let p = Process::standard(kind.method()).unwrap_or_else(|| panic!("{kind:?} has no method in data/content.toml"));
            assert!(p.labour > 0.0 && p.capital > 0.0 && !p.output.name.is_empty(), "{kind:?}: {p:?}");
        }
    }
    let coke = Process::of(Kind::CokeFurnace);
    assert_eq!(coke.output.name, "iron");
    assert!(coke.inputs.iter().any(|(g, a)| g.name == "iron_ore" && *a == 2.0) && coke.inputs.iter().any(|(g, _)| g.name == "coal"));
    assert!(Process::of(Kind::TextileMill).heat > 0.0 && Process::of(Kind::HandLoom).heat == 0.0);
    assert!(Process::standard("no such method").is_none());
}

#[test]
fn a_good_is_kept_by_what_the_content_says_of_it() {
    let keep = |name: &str| Keep::of(&Process::good(name));
    for heap in ["coal", "iron_ore", "stone", "clay"] {
        assert_eq!(keep(heap), Keep::Heap, "{heap}");
    }
    for stack in ["iron", "steel", "rails", "bricks", "machinery"] {
        assert_eq!(keep(stack), Keep::Stack, "{stack}");
    }
    for cover in ["wool", "cloth", "charcoal", "tools", "grain"] {
        assert_eq!(keep(cover), Keep::Cover, "{cover}");
    }
    assert_eq!(keep("timber"), Keep::Logs);
    assert_eq!(keep("services"), Keep::Nothing);
    // and a good nobody has heard of is kept by its numbers
    assert_eq!(Keep::of(&Good::new("salt_rock", 2.0, 0.5)), Keep::Heap);
    assert_eq!(Keep::of(&Good::new("glass", 1.1, 9.0)), Keep::Stack);
    assert_eq!(Keep::of(&Good::new("thread", 0.3, 4.0)), Keep::Cover);
}

#[test]
fn a_works_has_a_stage_for_its_founding_and_for_each_time_it_was_much_enlarged() {
    let st = stages(&history(1760, 1890, 30.0));
    assert_eq!(st[0], Stage { year: 1760, size: 0.5 });
    assert!(st.len() == 5, "{st:?}");
    for w in st.windows(2) {
        assert!(w[1].year > w[0].year && w[1].size >= w[0].size * ENLARGED);
    }
    // a works that never grew much has the one, and shrinking adds none
    assert_eq!(stages(&[(1800, 1.0), (1801, 1.2), (1802, 0.5), (1803, 2.0)]).len(), 1);
    assert_eq!(stages(&[]).len(), 0);
}

#[test]
fn every_kind_is_laid_out_on_every_plot_and_is_the_same_twice() {
    all(|plan, now| {
        assert!(!plan.parts.is_empty(), "{:?} at {} has no buildings", plan.kind, now.size);
        // a stage of its growth may find no room on a plot of an awkward shape with a siding
        // along its back; on a square plot of the ground it asks for, none goes without
        assert!(plan.dropped <= 1, "{:?} at {} on {:?}: {} stages found no ground", plan.kind, now.size, plan.plot.outline, plan.dropped);
        if plan.plot.outline.len() == 4 && plan.plot.edges[1..].iter().all(|e| *e == Edge::Wall) && (plan.plot.outline[2].1 - plan.plot.outline[3].1).abs() < 1e-6 {
            assert_eq!(plan.dropped, 0, "{:?} at {}: a stage found no ground on a square plot", plan.kind, now.size);
        }
        for state in [State::Working, State::GoingUp, State::Idle, State::Derelict] {
            let now = Now { state, ..now };
            let m = draw(plan, &now, 0.0);
            assert!(!m.solid.is_empty() && m.is_sound(), "{:?} at {} {state:?}", plan.kind, now.size);
            assert_eq!(m, draw(plan, &now, 0.0));
        }
    });
}

#[test]
fn everything_stands_on_the_plot_and_nothing_on_anything_else() {
    all(|plan, now| {
        let what = format!("{:?} at {} founded {}", plan.kind, now.size, plan.stages[0].year);
        let field = Field::new(&plan.plot);
        let boxes: Vec<(f32, f32, f32, f32)> = plan.parts.iter().map(|p| (p.x - p.w * 0.5, p.z - p.d * 0.5, p.x + p.w * 0.5, p.z + p.d * 0.5)).chain(plan.yards.iter().map(|y| (y.x - y.w * 0.5, y.z - y.d * 0.5, y.x + y.w * 0.5, y.z + y.d * 0.5))).collect();
        for (i, b) in boxes.iter().enumerate() {
            // (the first works of a plot that barely holds it may stand hard against the wall)
            assert!(field.room(b.0, b.1, b.2, b.3) >= if i == 0 { -WALL } else { -1e-4 }, "{what}: {b:?} is off the plot {:?}", plan.plot.outline);
            for o in &boxes[..i] {
                assert!(b.0 >= o.2 - 1e-4 || b.2 <= o.0 + 1e-4 || b.1 >= o.3 - 1e-4 || b.3 <= o.1 + 1e-4, "{what}: {b:?} stands on {o:?}");
            }
        }
        // and what is drawn keeps to the ground the works has taken in, which is within its plot
        let (x0, x1, z0, z1) = plan.enclosed.bounds();
        let (px0, px1, pz0, pz1) = plan.plot.bounds();
        assert!(x0 >= px0 - 1e-4 && x1 <= px1 + 1e-4 && z0 >= pz0 - 1e-4 && z1 <= pz1 + 1e-4, "{what}");
        for state in [State::Working, State::GoingUp, State::Derelict] {
            let m = draw(plan, &Now { state, ..now }, 0.0);
            for b in [&m.solid, &m.flat] {
                let Some((lo, hi)) = b.bounds() else { continue };
                let e = 0.006;
                assert!(lo.x >= x0 - e && hi.x <= x1 + e && lo.z >= z0 - e && hi.z <= z1 + e, "{what} {state:?}: {lo} to {hi} on ground {x0} to {x1}, {z0} to {z1}: {:?}", plan.parts);
            }
            assert!(m.solid.bounds().unwrap().1.y < 0.45);
        }
    });
}

#[test]
fn nothing_that_stands_moves_when_a_works_is_enlarged_or_the_railway_comes() {
    let pal = Palette::standard();
    for kind in Kind::ALL {
        let (w, d) = ground(kind, 30.0);
        let st = stages(&history(1770, 1890, 30.0));
        for plot in [Plot::rect(w, d), Plot::wedge(w, d, 0.3).facing_from(2, Edge::Rail, 1848).facing_from(1, Edge::Fed, 1820), Plot::cut(w, d, 0.3).facing(1, Edge::Water { navigable: false })] {
            let whole = plan(kind, &Process::of(kind), &st, &plot, 1890, Seed::of(5), &pal);
            for n in 1..st.len() {
                let year = st[n].year - 1;
                let then = plan(kind, &Process::of(kind), &st[..n], &plot, year, Seed::of(5), &pal);
                assert!(then.parts.len() <= whole.parts.len() && then.parts[..] == whole.parts[..then.parts.len()], "{kind:?}: what stood in {year} has moved by 1890");
            }
        }
    }
}

#[test]
fn it_is_the_same_works_at_every_level_of_detail() {
    let pal = Palette::standard();
    let site_ = Site::level(Vec3::ZERO, 0.3);
    all(|plan, now| {
        for state in [State::Working, State::Derelict] {
            let built: Vec<(Model, Option<Vec3>, i32)> = Detail::ALL
                .iter()
                .map(|&detail| {
                    let mut m = Model::default();
                    let (peak, taken) = build(&mut m.solid, &mut m.smoke, &mut m.flat, &site_, plan, &Now { state, detail, ..now }, &pal);
                    (m, peak, taken)
                })
                .collect();
            let (far, play, near) = (&built[0], &built[1], &built[2]);
            for other in [play, near] {
                assert!(far.0.flat == other.0.flat, "{:?}: the ground differs by level", plan.kind);
                assert!(far.0.smoke == other.0.smoke, "{:?}: the smoke differs by level", plan.kind);
                assert_eq!(far.1, other.1, "{:?}: the flag's place differs by level", plan.kind);
                assert_eq!(far.2, other.2, "{:?}: its parts took other numbers of their own at another level", plan.kind);
            }
            assert!(far.0.solid.triangles() <= play.0.solid.triangles() && play.0.solid.triangles() <= near.0.solid.triangles(), "{:?}", plan.kind);
        }
    });
}

#[test]
fn a_works_shows_three_ages_and_what_falls_out_of_use() {
    let pal = Palette::standard();
    let (w, d) = ground(Kind::TextileMill, 24.0);
    let plot = Plot::rect(w, d).facing(1, Edge::Water { navigable: false });
    let st = [Stage { year: 1760, size: 0.6 }, Stage { year: 1820, size: 6.0 }, Stage { year: 1870, size: 24.0 }];
    let p = plan(Kind::TextileMill, &Process::of(Kind::TextileMill), &st, &plot, 1890, Seed::of(3), &pal);
    // a range of each stage, built as its year built
    for (i, stage) in st.iter().enumerate() {
        assert!(p.parts.iter().any(|part| part.role == Role::Core && part.stage == i), "no range of {}", stage.year);
    }
    let now = Now::working(&p, 1890);
    let whole = draw(&p, &now, 0.0);
    // the stone of 1760 (weathered these hundred and thirty years) and the brick of 1870 are both in it
    let has = |m: &Model, c: Rgba| m.solid.col.iter().any(|v| (v[0] - c[0]).abs() < 0.01 && (v[1] - c[1]).abs() < 0.01 && (v[2] - c[2]).abs() < 0.01);
    assert!(has(&whole, lerp(*pal.stone.works, *pal.smoke.soot, 0.14)), "no stone range of 1760");
    assert!(has(&whole, *pal.brick.red), "no brick range of 1870");
    // the wheel of 1760 is kept beside the chimneys of the later stages
    assert!(has(&whole, lerp(*pal.timber.waterwheel, *pal.smoke.soot, 0.14)), "the wheel is gone");
    assert!(!whole.smoke.is_empty());
    // shrunk to its size of 1820: the range of 1870 stands idle, and derelict after eight years
    let idle = draw(&p, &Now { size: 5.0, shrunk_for: 3, ..now }, 0.0);
    let gone = draw(&p, &Now { size: 5.0, shrunk_for: 9, ..now }, 0.0);
    assert!(idle.smoke.triangles() < whole.smoke.triangles() && idle != whole && gone != idle);
    assert!(gone.smoke.triangles() == idle.smoke.triangles() && !gone.smoke.is_empty());
    // and a stage built this year is under scaffolding
    let p70 = plan(Kind::TextileMill, &Process::of(Kind::TextileMill), &st, &plot, 1870, Seed::of(3), &pal);
    let pole = *pal.timber.scaffold_pole;
    assert!(draw(&p70, &Now::working(&p70, 1870), 0.0).solid.col.iter().any(|c| *c == pole) && !whole.solid.col.iter().any(|c| *c == pole));
}

#[test]
fn what_a_works_holds_and_how_busy_it_is_show() {
    let pal = Palette::standard();
    for kind in [Kind::CokeFurnace, Kind::RollingMill, Kind::Brickworks, Kind::TextileMill] {
        let (w, d) = ground(kind, 10.0);
        let (p, now) = specimen(kind, 1800, 1885, 10.0, &Plot::rect(w, d).facing(2, Edge::Rail), Seed::of(2), &pal);
        let tris = |stock: f32, busy: f32| draw(&p, &Now { stock_in: stock, stock_out: stock, busy, ..now }, 0.0).solid.triangles();
        assert!(tris(1.0, 0.8) > tris(0.5, 0.8) && tris(0.5, 0.8) > tris(0.05, 0.8), "{kind:?}: its stock does not show");
        assert!(tris(0.5, 1.0) > tris(0.5, 0.05), "{kind:?}: its busyness does not show in the wagons on its siding");
    }
}

#[test]
fn the_ways_in_and_out_follow_what_the_plot_faces() {
    let pal = Palette::standard();
    let (w, d) = ground(Kind::RollingMill, 10.0);
    let of = |plot: Plot, year: i32| specimen(Kind::RollingMill, 1800, year, 10.0, &plot, Seed::of(2), &pal).0;
    let plain = of(Plot::rect(w, d), 1885);
    assert!(plain.siding.is_none() && plain.wharf.is_none());
    // in at one side of the gate and out at the other
    assert!(plain.receive.0 < plain.gate.0 && plain.dispatch.0 > plain.gate.0);
    let railed = of(Plot::rect(w, d).facing_from(2, Edge::Rail, 1850), 1885);
    assert_eq!(railed.siding.expect("no siding by the railway").0, 2);
    // the railway had not come in 1840
    assert!(of(Plot::rect(w, d).facing_from(2, Edge::Rail, 1850), 1840).siding.is_none());
    let on_siding = |p: &Plan| draw(p, &Now::working(p, 1885), 0.0).solid.col.iter().filter(|c| pal.vehicles.wagons.iter().any(|w| **w == **c)).count();
    assert!(on_siding(&railed) > 0 && on_siding(&plain) == 0);
    let quay = of(Plot::rect(w, d).facing(3, Edge::Water { navigable: true }), 1885);
    assert_eq!(quay.wharf.map(|s| s.0), Some(3));
    assert!(of(Plot::rect(w, d).facing(3, Edge::Water { navigable: false }), 1885).wharf.is_none());
    // fed over one wall and feeding over another: the stock comes in at the one and goes out at the other
    let chain = of(Plot::rect(w, d).facing(3, Edge::Fed).facing(1, Edge::Feeds), 1885);
    assert!(chain.receive.0 < -w * 0.4 && chain.dispatch.0 > w * 0.4);
}

#[test]
fn a_works_of_a_kind_nobody_knows_is_planned_from_its_process() {
    let pal = Palette::standard();
    let glass = Process { inputs: vec![(Good::new("sand", 2.0, 0.3), 1.4), (Good::new("soda", 0.6, 4.0), 0.3)], heat: 1.8, labour: 0.6, capital: 120.0, output: Good::new("glass", 1.1, 9.0) };
    let (w, d) = ground(Kind::Other, 8.0);
    let st = stages(&history(1830, 1885, 8.0));
    let p = plan(Kind::Other, &glass, &st, &Plot::rect(w, d), 1885, Seed::of(4), &pal);
    // a store for the soda, a heap yard for the sand and one for its coal, a stack yard for the glass, a tip
    assert!(p.parts.iter().any(|x| matches!(x.role, Role::Store(_))));
    let holds: Vec<Hold> = p.yards.iter().map(|y| y.hold).collect();
    assert!(holds.contains(&Hold::Out) && holds.contains(&Hold::Tip) && holds.iter().filter(|h| matches!(h, Hold::In(_))).count() == 2, "{holds:?}");
    let m = draw(&p, &Now::working(&p, 1885), 0.0);
    assert!(m.is_sound() && !m.smoke.is_empty());
    // and one that burns nothing and makes light goods has a warehouse, no yards and no smoke but its counting house's
    let lace = Process { inputs: vec![(Good::new("thread", 0.3, 4.0), 1.0)], heat: 0.0, labour: 2.0, capital: 20.0, output: Good::new("lace", 0.2, 12.0) };
    let q = plan(Kind::Other, &lace, &st, &Plot::rect(w, d), 1885, Seed::of(4), &pal);
    assert!(q.parts.iter().any(|x| x.role == Role::Warehouse) && q.yards.is_empty());
    assert!(draw(&q, &Now::working(&q, 1885), 0.0).smoke.triangles() < m.smoke.triangles());
}

#[test]
fn the_flag_flies_from_the_works_and_the_ground_follows_a_slope() {
    let pal = Palette::standard();
    let slope = |x: f32, z: f32| 0.2 + 0.3 * x - 0.1 * z;
    all(|plan, now| {
        let s = Site::level(Vec3::ZERO, 0.3);
        let p = flag_point(&s, plan, &now, &pal);
        let (x0, x1, z0, z1) = plan.plot.bounds();
        assert!(p.y > 0.008 && p.y < 0.45 && p.x.hypot(p.z) <= (x1 - x0).hypot(z1 - z0), "{:?}: the flag is at {p}", plan.kind);
        if now.size == 10.0 {
            let mut m = Model::default();
            site(&mut m.solid, &mut m.smoke, &mut m.flat, &Site::on(Vec3::new(0.0, 0.2, 0.0), 0.0, &slope), plan, &now, &pal);
            assert!(m.is_sound());
            for p in &m.flat.pos {
                assert!((p[1] - slope(p[0], p[2])).abs() < 0.012, "{:?}: the ground at {p:?} is off the slope", plan.kind);
            }
        }
    });
}
