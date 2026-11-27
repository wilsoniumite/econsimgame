//! The ways' places and courses (`docs/rules/courses.md`, not in the Python):
//! switched off they are the port (tests/golden.rs holds that to the byte) and
//! the book stays empty; switched on, nothing in the economy, the books or the
//! news moves with them; a live game is handed what the record keeps, year by
//! year, for the ways and for the order the works took their squares in; a
//! place, once chosen, and a course, once found, never move; and a course runs
//! from the place of one of its cells to the place of the other.
//!
//! The cases are played under the game's rules with six families, with the
//! three switches they lean on set by name (`courses`, `squares` and
//! `traffic`, whose carting the courses wear by): the courses read the books
//! and the year's carting and are read by no rule, so no other switch matters.
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use isostate_core::cli::{events_json, series_csv};
use isostate_core::courses::{self, CourseBook, CourseParams};
use isostate_core::squares::{self, Tenant};
use isostate_core::*;

fn params(seed: u64, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end_year);
    P.courses.enabled = true;
    P.squares.enabled = true;
    P.traffic.enabled = true;
    P
}

#[test]
fn off_is_the_port_and_on_is_the_game() {
    assert_eq!(default_params().courses, CourseParams::default());
    assert!(!default_params().courses.enabled, "the port keeps no courses");
    assert!(game_params().courses.enabled, "the game does");
    let mut P = default_params();
    P.seed = 3;
    P.end_year = Some(1752);
    let r = sim::run_with(&P, default_content(), false, true).expect("money is conserved");
    assert!(!r.state.wc.kept() && r.state.wc.courses.is_empty() && r.rec.way_first.is_empty());
    assert!(!serde_json::to_string(&r.params).unwrap().contains("\"courses\""), "params.json does not mention it");
}

#[test]
fn nothing_in_the_economy_moves_with_the_switch() {
    let runs: Vec<RunResult> = std::thread::scope(|sc| {
        let hs: Vec<_> = [true, false]
            .into_iter()
            .map(|on| {
                sc.spawn(move || {
                    let mut P = params(5, 1790);
                    P.courses.enabled = on;
                    sim::run_with(&P, game_content(), false, true).expect("money is conserved")
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let (on, off) = (&runs[0], &runs[1]);
    assert!(on.state.wc.kept() && !off.state.wc.kept());
    assert!(series_csv(on) == series_csv(off), "the months (the economy and the books) moved with the courses");
    assert!(events_json(on) == events_json(off), "the news moved with the courses");
    // what moves is where the squares stand (they face the courses), and the record's logs
    assert!(!on.rec.way_log.is_empty() && off.rec.way_first.is_empty());
}

/// What a reader of the ways' log holds: each cell's place, each link's course, the links drawn.
#[derive(Default, PartialEq, Debug)]
struct Ways {
    places: BTreeMap<u32, [u16; 2]>,
    courses: BTreeMap<u32, courses::Course>,
    drawn: std::collections::BTreeSet<u32>,
}

impl Ways {
    fn apply(&mut self, y: &courses::WaysYear) {
        for &(c, p, _) in &y.places {
            assert!(self.places.insert(c, p).is_none_or(|q| q == p), "cell {c}'s place moved");
        }
        for (l, drawn, course) in &y.links {
            if let Some(c) = course {
                assert!(self.courses.insert(*l, c.clone()).is_none_or(|was| was == *c), "link {l}'s course moved");
            }
            if *drawn {
                self.drawn.insert(*l);
            } else {
                self.drawn.remove(l);
            }
        }
    }

    fn of(st: &State) -> Ways {
        let b = &st.wc;
        Ways {
            places: (0..b.placed.len()).filter(|&c| b.placed[c] != 0).map(|c| (c as u32, b.place[c])).collect(),
            courses: b.courses.clone(),
            drawn: (0..b.drawn.len()).filter(|&l| b.drawn[l]).map(|l| l as u32).collect(),
        }
    }
}

/// What a reader of the squares' and the holdings' logs holds of each works'
/// and working's squares, in the order taken: (pit, number, cell) to its squares.
type Stacks = BTreeMap<(bool, u32, u32), Vec<u8>>;

/// A year's holdings: the squares the year's changes took from each works and
/// pit come off the top of its stack (what it took last goes first), then what
/// it took goes on, in order.
fn take(stacks: &mut Stacks, held: &mut BTreeMap<u32, u32>, changes: &[squares::Change], took: &[squares::Took], year: i64) {
    let key = |code: u32, cell: u32| match Tenant::of(code) {
        Tenant::Works(f) => Some((false, f, cell)),
        Tenant::Pit(r) => Some((true, r, cell)),
        _ => None,
    };
    let mut gone: BTreeMap<(bool, u32, u32), Vec<u8>> = BTreeMap::new();
    for ch in changes {
        if let Some(&was) = held.get(&ch.at) {
            if was != ch.tenant {
                if let Some(k) = key(was, ch.cell() as u32) {
                    gone.entry(k).or_default().push(ch.square() as u8);
                }
            }
        }
        if ch.tenant == 0 {
            held.remove(&ch.at);
        } else {
            held.insert(ch.at, ch.tenant);
        }
    }
    for (k, mut sq) in gone {
        let st = stacks.get_mut(&k).unwrap_or_else(|| panic!("{year}: {k:?} gave up squares it never took"));
        let n = st.len() - sq.len();
        let mut top = st[n..].to_vec();
        top.sort_unstable();
        sq.sort_unstable();
        assert_eq!(top, sq, "{year}: {k:?} gave up squares other than those it took last");
        st.truncate(n);
        if st.is_empty() {
            stacks.remove(&k);
        }
    }
    for t in took {
        let k = (t.pit, t.id, t.cell);
        let st = stacks.entry(k).or_default();
        assert_eq!(st.is_empty(), t.fresh, "{year}: {k:?} is told as fresh only when it held nothing");
        st.extend_from_slice(&t.squares);
    }
}

fn stacks_of(st: &State) -> Stacks {
    let mut out = Stacks::new();
    for (f, h) in st.sq.works.iter().enumerate() {
        if !h.squares.is_empty() {
            out.insert((false, f as u32, h.cell as u32), h.squares.clone());
        }
    }
    for (&(r, c), h) in &st.sq.pits {
        if !h.squares.is_empty() {
            out.insert((true, r as u32, c as u32), h.squares.clone());
        }
    }
    out
}

#[test]
fn a_replay_and_a_live_game_hold_the_ways_and_the_order_of_the_squares() {
    // Island 3 to 1800, played as a live game is (`live_year` after every year):
    // each year the live year's ways and holdings are the record's to the byte, and
    // a reader of either holds the book's places, courses, drawn links, and each
    // works' and working's squares in the order it took them
    let P = params(3, 1800);
    let mut s = sim::start(&P, game_content());
    let mut cursor = LiveCursor::new(&s.content);
    let (mut ways, mut stacks, mut held) = (Ways::default(), Stacks::new(), BTreeMap::new());
    let (mut found, mut taken) = (0usize, 0usize);
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved");
        let y = live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        let k = y.index;
        let w = y.ways.expect("a game with the courses hands them over each year");
        assert!(!w.whole, "{year}: a cursor told every year is sent the year's");
        assert!(s.rec.way_log[s.rec.way_first[k] as usize..s.rec.way_first[k + 1] as usize] == w.log[..], "{year}: the live year's ways are not the record's");
        let sq = y.squares.expect("and the squares");
        assert!(s.rec.sq_hold[s.rec.sq_hold_first[k] as usize..s.rec.sq_hold_first[k + 1] as usize] == sq.holds[..], "{year}: the live year's holdings are not the record's");
        let wy = courses::decode_year(&w.log).expect("the ways' log reads");
        assert_eq!(courses::encode_year(&wy), w.log, "{year}: the ways' log reads back what was written");
        found += wy.links.iter().filter(|x| x.2.is_some()).count();
        ways.apply(&wy);
        assert!(ways == Ways::of(&s.st), "{year}: the reader's ways are not the book's");
        let took = squares::decode_holds(&sq.holds).expect("the holdings' log reads");
        assert_eq!(squares::encode_holds(&took), sq.holds, "{year}: the holdings' log reads back what was written");
        taken += took.len();
        take(&mut stacks, &mut held, &sq.changes, &took, year);
        assert!(stacks == stacks_of(&s.st), "{year}: the reader's holdings are not the book's");
    }
    assert!(found > 1000 && ways.places.len() > 800 && taken > 300, "{found} courses, {} places, {taken} takings", ways.places.len());
    // the record as a reader finds it
    let blob = record::pack(&s.st, &s.content, &s.params, &s.events, &s.rec, "game", false);
    let n = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let header: serde_json::Value = serde_json::from_slice(&blob[4..4 + n]).unwrap();
    assert_eq!(header["ways"]["layout"], courses::LOG_LAYOUT);
    assert_eq!(header["squares"]["layout"], squares::LOG_LAYOUT);
    for name in ["way_first", "way_log", "sq_hold_first", "sq_hold"] {
        assert!(header["arrays"].as_array().unwrap().iter().any(|a| a["name"] == name), "the record has no {name}");
    }
    println!("island 3 to 1800: {found} courses found, {} places, {taken} takings; the ways' log {} bytes, the holdings' {}", ways.places.len(), s.rec.way_log.len(), s.rec.sq_hold.len());
}

#[test]
fn a_cursor_that_missed_a_year_is_sent_every_way_and_holding() {
    let P = params(2, 1753);
    let mut s = sim::start(&P, game_content());
    let mut early = LiveCursor::new(&s.content);
    sim::run_year(&mut s).unwrap();
    let first = live_year(&mut early, &s.st, &s.content, &s.params, &s.events);
    assert!(!first.ways.unwrap().whole);
    sim::run_year(&mut s).unwrap();
    sim::run_year(&mut s).unwrap();
    for mut c in [early, LiveCursor::new(&s.content)] {
        let y = live_year(&mut c, &s.st, &s.content, &s.params, &s.events);
        let w = y.ways.unwrap();
        assert!(w.whole);
        let mut ways = Ways::default();
        ways.apply(&courses::decode_year(&w.log).unwrap());
        assert!(ways == Ways::of(&s.st));
        let sq = y.squares.unwrap();
        let mut stacks = Stacks::new();
        take(&mut stacks, &mut BTreeMap::new(), &[], &squares::decode_holds(&sq.holds).unwrap(), 1752);
        assert!(stacks == stacks_of(&s.st));
    }
}

#[test]
fn a_place_never_moves_and_a_kept_course_keeps_its_points() {
    // Island 7 to 1820: every place where it was chosen, every course with the points
    // it was found with, in every later year; each course runs from the place of its
    // own cell to its neighbour's, and every link drawn has a course
    let P = params(7, 1820);
    let mut s = sim::start(&P, game_content());
    let (W, H) = (s.st.world.W, s.st.world.H);
    let mut places: BTreeMap<usize, [u16; 2]> = BTreeMap::new();
    let mut found: BTreeMap<u32, courses::Course> = BTreeMap::new();
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved");
        let b = &s.st.wc;
        for c in 0..b.placed.len() {
            if b.placed[c] != 0 {
                assert_eq!(*places.entry(c).or_insert(b.place[c]), b.place[c], "{year}: cell {c}'s place moved");
            }
        }
        assert!(places.keys().all(|&c| b.placed[c] != 0), "{year}: a place was forgotten");
        for (&l, c) in &b.courses {
            assert_eq!(found.entry(l).or_insert_with(|| c.clone()), c, "{year}: link {l}'s course moved");
            let m = traffic::other_end(W, H, l as usize / 4, l as usize % 4).unwrap();
            assert_eq!((c.pts[0], *c.pts.last().unwrap()), (b.place[l as usize / 4], b.place[m]), "{year}: link {l}'s course does not run between its places");
            // a course keeps near its two cells: no point further than a cell and a half from both their middles
            for p in &c.pts {
                let (x, y) = CourseBook::at(*p);
                let near = |cell: usize| (x - (cell % W) as f32 - 0.5).abs().max((y - (cell / W) as f32 - 0.5).abs()) <= 1.5;
                assert!(near(l as usize / 4) || near(m), "{year}: link {l}'s course strays to {x},{y}");
            }
        }
        assert!(found.keys().all(|l| b.courses.contains_key(l)), "{year}: a course was forgotten");
        assert!((0..b.drawn.len()).filter(|&l| b.drawn[l]).all(|l| b.courses.contains_key(&(l as u32))), "{year}: a drawn link has no course");
    }
    assert!(places.len() > 800 && found.len() > 1000, "{} places, {} courses", places.len(), found.len());
}
