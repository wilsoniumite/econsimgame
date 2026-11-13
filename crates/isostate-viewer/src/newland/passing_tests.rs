//! A railway's way by the ground (passing.rs), on made-up land small enough
//! to say what the right answer is: a plain with a lake in it, a hillside
//! with a spur across the line's cells; and on the three islands of
//! on_islands.rs, a line laid along a row of cells from coast to coast.

use bevy::math::Vec2;

use super::on_islands::bare_run;
use super::{Lake, NewLand};
use crate::rails::{self, Kind};
use crate::replay::Replay;
use crate::world::{self, Ground};

const W: usize = 16;
const H: usize = 8;
const FINE: usize = 4;

/// A map of 16 by 8 cells, all land, whose nodes stand `height` metres high
/// (given the node's middle in cells) and lie under one lake where `lake`
/// says so, with track laid along `track` (cells as x, y).
fn made_up(height: impl Fn(Vec2) -> f32, lake: impl Fn(Vec2) -> bool, level: f32, track: &[(usize, usize)]) -> (Replay, Ground, Vec<(f32, f32)>) {
    let (fw, fh) = (W * FINE, H * FINE);
    let at = |n: usize| Vec2::new(((n % fw) as f32 + 0.5) / FINE as f32, ((n / fw) as f32 + 0.5) / FINE as f32);
    let under: Vec<u8> = (0..fw * fh).map(|n| lake(at(n)) as u8).collect();
    let land = NewLand {
        w: W,
        h: H,
        fine: FINE,
        fw,
        fh,
        // (a lake's bed lies 4 m under its water)
        height: (0..fw * fh).map(|n| if under[n] > 0 { level - 4.0 } else { height(at(n)) }).collect(),
        rock: vec![0; fw * fh],
        lake: under,
        wet: vec![0; fw * fh],
        flow: vec![0.0; fw * fh],
        lines: Vec::new(),
        oxbows: Vec::new(),
        lakes: vec![Lake { level, area_km2: 1.0, depth: 4.0, x: 0.0, y: 0.0, outlet: (0.0, 0.0), outflow: 0.0 }],
        drowned: 0.0,
        lake_cell: vec![false; W * H],
        shore: vec![false; W * H],
        estuary: vec![false; W * H],
        middles: Default::default(),
    };
    let mut r = bare_run(1, W, H, vec![1; W * H], vec![0; W * H], vec![0; W * H], vec![-1; W * H], &[], Some(land));
    for &(x, y) in track {
        r.trans[y * W + x] |= 4;
    }
    let g = world::build_ground(&r);
    let cen = crate::network::centres(&r);
    (r, g, cen)
}

/// The cells of a row from x0 to x1.
fn row(y: usize, x0: usize, x1: usize) -> Vec<(usize, usize)> {
    (x0..=x1).map(|x| (x, y)).collect()
}

#[test]
fn a_line_goes_round_a_lake_where_its_cells_have_dry_ground_and_crosses_one_that_fills_them() {
    // a plain 60 m up, and a line along row 3 from x = 2 to x = 13
    let plain = |_: Vec2| 60.0;
    // a lake across the line's cells that leaves their southern third dry
    let (r, g, cen) = made_up(plain, |p| (6.0..9.0).contains(&p.x) && (1.0..3.6).contains(&p.y), 58.0, &row(3, 2, 13));
    let ry = rails::year(&r, &g, &cen, 0);
    assert_eq!(ry.lines.len(), 1);
    let wet = rails::over_water(&ry, &g);
    assert!(wet.is_empty(), "the line stands over the lake though its cells have dry ground south of it: {wet:?}");
    // and it has gone south of the lake to do it, not through it
    let line = &ry.lines[0];
    assert!(line.pts.iter().filter(|p| (6.0..9.0).contains(&p.x)).all(|p| p.z > 3.6), "the line passes the lake to the south");
    assert!(line.kind.iter().all(|&k| k != Kind::Viaduct && k != Kind::Tunnel), "a plain asks for no viaduct and no tunnel");

    // a lake that fills the line's cells from side to side and stops at their southern edge: the
    // line goes round by the edge of the next row, which is within the room it has outside its cells
    let (r, g, cen) = made_up(plain, |p| (6.0..9.0).contains(&p.x) && (1.0..4.0).contains(&p.y), 58.0, &row(3, 2, 13));
    let ry = rails::year(&r, &g, &cen, 0);
    let wet = rails::over_water(&ry, &g);
    assert!(wet.is_empty(), "the line stands over a lake it could pass by the edge of its cells: {wet:?}");
    assert!(ry.lines[0].pts.iter().all(|p| p.z < 4.3), "the line strays further from its cells than it may");

    // the same lake from the top of the map to the bottom: there is no way round, and the line crosses it
    let (r, g, cen) = made_up(plain, |p| (6.0..9.0).contains(&p.x), 58.0, &row(3, 2, 13));
    let ry = rails::year(&r, &g, &cen, 0);
    let wet = rails::over_water(&ry, &g);
    assert_eq!(wet.len(), 1, "{wet:?}");
    assert!(!wet[0].way_round, "the check finds a dry way round a lake that has none");
    // squarely: about the lake's own width of it, 3 cells
    assert!(wet[0].cells > 2.9 && wet[0].cells < 3.4, "the line is over {} cells of water crossing a lake 3 cells wide", wet[0].cells);
    // and over its water, not at the height of its bed
    let over: Vec<f32> = (0..line_len(&ry)).filter(|&q| (6.2..8.8).contains(&ry.lines[0].pts[q].x)).map(|q| ry.lines[0].pts[q].y - ry.lines[0].ground[q]).collect();
    let surface = super::ground::units(58.0);
    assert!(ry.lines[0].pts.iter().filter(|p| (6.2..8.8).contains(&p.x)).all(|p| p.y > surface), "the line lies under the lake's water");
    assert!(over.iter().all(|&d| d < 0.1), "the line's height over a lake is measured from its water: {:?}", over.iter().copied().fold(0.0, f32::max));
}

fn line_len(ry: &rails::RailYear) -> usize {
    ry.lines[0].pts.len()
}

#[test]
fn a_line_goes_round_a_spur_and_not_over_it() {
    // a hillside that rises to the north, 150 m in a cell, and a spur 120 m high and half a cell
    // wide that comes down it across the line's row and ends a third of a cell into that row
    let nose = 4.35;
    let hill = move |p: Vec2| {
        let slope = 400.0 + 150.0 * (4.0 - p.y);
        let spur = if p.y < nose { 120.0 * (1.0 - ((p.x - 8.0).abs() / 0.5).min(1.0)) * ((nose - p.y) / 0.3).min(1.0) } else { 0.0 };
        slope + spur
    };
    let (r, g, cen) = made_up(hill, |_| false, 0.0, &row(4, 2, 13));
    let ry = rails::year(&r, &g, &cen, 0);
    assert_eq!(ry.lines.len(), 1);
    let line = &ry.lines[0];
    let c = rails::check(&ry, &g);
    // it keeps to the contour: nothing bored, nothing on arches, nothing roped
    assert_eq!((c.tunnels, c.viaducts, c.inclines), (0, 0, 0), "a line along a hillside with a spur to go round: {c:?}");
    assert!(c.step.0 < 1e-4 && c.steepest.0 <= rails::laws(&g).rules.ruling + 1e-4, "{c:?}");
    // past the spur it lies below the nose, and either side of it higher up the hill than that
    let at_spur = line.pts.iter().filter(|p| (p.x - 8.0).abs() < 0.2).map(|p| p.z).fold(f32::MAX, f32::min);
    assert!(at_spur > nose - 0.1, "the line passes the spur at {at_spur}, and its nose is at {nose}");
    // and the ground under it rises and falls little beyond what its gradient climbs
    let (beyond, _) = rails::beyond_the_gradient(&ry, &g);
    assert!(beyond < 0.05, "the ground along the line rises and falls {beyond} beyond the gradient");
}

#[test]
fn the_rules_in_metres_are_the_old_island_s_at_three_times_the_relief() {
    let (_, new_land, _) = made_up(|_| 60.0, |_| false, 0.0, &[]);
    let got = rails::laws(&new_land);
    let old = rails::RULES;
    for (name, new, was) in [
        ("the ruling gradient", got.rules.ruling, old.ruling),
        ("an incline", got.rules.steep, old.steep),
        ("how high a line may stand", got.rules.high, old.high),
        ("how long a viaduct may be", got.rules.bank, old.bank),
        ("how deep before a tunnel", got.rules.tunnel, old.tunnel),
    ] {
        assert!((new / was - 1.0).abs() < 0.01, "{name}: {new} on the new land and {was} on the old island");
    }
    assert_eq!((got.rules.ease, got.rules.wind, got.rules.wind_steps), (old.ease, old.wind, old.wind_steps));
    assert!((got.viaduct / 0.07 - 1.0).abs() < 0.01);
    // and a ground with no land of the core's has the old island's rules to the bit
    let old_island = world::build_ground(&bare_run(1, W, H, vec![1; W * H], vec![40; W * H], vec![0; W * H], vec![-1; W * H], &[], None));
    let kept = rails::laws(&old_island);
    assert_eq!(format!("{:?}", kept.rules), format!("{:?}", old));
    assert_eq!(kept.viaduct, 0.07);
}

#[test]
fn a_road_under_a_lake_is_left_out_of_the_picture_on_the_new_land() {
    let (_, g, _) = made_up(|_| 60.0, |p| (6.0..9.0).contains(&p.x) && (1.0..3.6).contains(&p.y), 58.0, &[]);
    let v = |x: f32, y: f32| Vec2::new(x, y);
    // across the lake, and across a corner of it: left out; south of it, and one that only touches its edge: drawn
    assert!(!crate::network::link_shown(&g, v(5.5, 2.5), v(9.5, 2.5)));
    assert!(!crate::network::link_shown(&g, v(5.5, 3.0), v(6.5, 3.5)));
    assert!(crate::network::link_shown(&g, v(5.5, 3.8), v(9.5, 3.8)));
    assert!(crate::network::link_shown(&g, v(5.5, 3.0), v(5.95, 3.0)));
    // a link that is left out has no bridge and no causeway
    assert!(crate::network::causeway_runs(&g, v(5.5, 2.5), v(9.5, 2.5)).is_empty());
    // and on the old island every road is in the picture, whatever it lies on
    let old_island = world::build_ground(&bare_run(1, W, H, vec![0; W * H], vec![0; W * H], vec![0; W * H], vec![-1; W * H], &[], None));
    assert!(crate::network::link_shown(&old_island, v(5.5, 2.5), v(9.5, 2.5)));
}

#[test]
fn a_line_across_each_island_holds_together_and_keeps_to_its_cells() {
    // On the three islands of on_islands.rs, track along the longest run of land cells in each
    // of three rows of the map, which no simulation chose: across the lowland, the belt and
    // whatever lakes and tideways lie in the way.
    for isl in super::on_islands::islands() {
        let (w, h) = (isl.r.w, isl.r.h);
        for y in [h / 3, h / 2, 2 * h / 3] {
            let mut best = (0, 0);
            let mut x = 0;
            while x < w {
                let x0 = x;
                while x < w && isl.r.terrain[y * w + x] != 0 {
                    x += 1;
                }
                if x - x0 > best.1 - best.0 {
                    best = (x0, x);
                }
                x += 1;
            }
            let mut r = isl.r.clone();
            for x in best.0..best.1 {
                r.trans[y * w + x] |= 4;
            }
            let ry = rails::year(&r, &isl.g, &isl.cen, 0);
            let c = rails::check(&ry, &isl.g);
            let what = format!("island {}, row {y}, cells {} to {}", isl.seed, best.0, best.1 - 1);
            assert_eq!(c.lines, 1, "{what}");
            // no step along it, and its trains on its rails
            assert!(c.step.0 < 1e-4 && c.meet.0 < 1e-4 && c.train_off.0 < 0.01, "{what}: {c:?}");
            // every cell is passed within the room a line has outside its cells (500 m)
            assert_eq!(ry.stops.len(), best.1 - best.0, "{what}");
            for (&cell, stop) in &ry.stops {
                let (cx, cy) = ((cell % w) as f32, (cell / w) as f32);
                let out = (cx - stop.at.x).max(stop.at.x - cx - 1.0).max(cy - stop.at.y).max(stop.at.y - cy - 1.0).max(0.0);
                assert!(out <= 0.25 + 1e-3, "{what}: cell {cell} is passed at {}, {out} outside it", stop.at);
            }
            // it stands over no lake or tideway that its cells have dry ground to go round by
            let needless: Vec<_> = rails::over_water(&ry, &isl.g).into_iter().filter(|s| s.way_round).collect();
            assert!(needless.is_empty(), "{what}: {needless:?}");
            // and the ground along it asks less of it than the ground along the straight way from
            // each cell's meeting place to the next, which is how a line ran before it chose by the ground
            let ruling = rails::laws(&isl.g).rules.ruling;
            let mut straight = 0.0;
            for x in best.0..best.1 - 1 {
                let (a, b) = (isl.cen[y * w + x], isl.cen[y * w + x + 1]);
                let (a, b) = (Vec2::new(a.0, a.1), Vec2::new(b.0, b.1));
                let n = (a.distance(b) / 0.1).ceil().max(1.0) as usize;
                let at = |i: usize| a.lerp(b, i as f32 / n as f32);
                for i in 0..n {
                    straight += ((rails::lie(&isl.g, at(i + 1).x, at(i + 1).y) - rails::lie(&isl.g, at(i).x, at(i).y)).abs() - ruling * a.distance(b) / n as f32).max(0.0);
                }
            }
            let (beyond, _) = rails::beyond_the_gradient(&ry, &isl.g);
            println!("{what}: the ground rises and falls {beyond:.2} beyond the gradient along the line, {straight:.2} along the straight way between the cells' middles; {} viaducts, {} tunnels, {} inclines", c.viaducts, c.tunnels, c.inclines);
            assert!(beyond <= straight + 0.05, "{what}: {beyond} along the line and {straight} along the straight way");
        }
    }
}
