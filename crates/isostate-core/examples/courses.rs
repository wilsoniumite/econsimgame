//! The ways' places and courses the simulation keeps (`docs/rules/courses.md`),
//! measured and pictured: each island is played twice side by side, with the
//! courses on (the game's rules) and off, so that the squares facing the
//! courses can be set against the squares as they were.
//!
//!     cargo run --profile work -p isostate-core --example courses -- [--seeds 7] [--to 2050]
//!         [--families 6] [--start settled|landfall] [--time] [--out DIR]
//!
//! - By year: links drawn, courses and places kept, the share of the drawn
//!   courses' length within 5 degrees of the map's eight directions (and of
//!   the straight lines between the meeting places, as the drawn links would
//!   be without a course), and the share of house blocks within one square
//!   (125 m) of a drawn course, with the courses on and off.
//! - Every year: the two runs' people and money are the same (the courses are
//!   read by no rule), every place is where it was the year it was chosen and
//!   every course has the points it was found with; and played as a live game
//!   is (`live_year`), the live year's ways, squares and holdings are the
//!   record's bytes, and a reader of the ways' log holds the book's ways.
//! - `--time`: a third run with the courses and the squares off, the year's
//!   `courses::close_year` and `squares::close_year` called beside it and timed
//!   alone (both read the books and write nothing else).
//! - `--out DIR`: pictures of the first island's squares with and without the
//!   courses, as `examples/squares.rs` draws them, in 1800, 1850, 1900 and 2050:
//!   the edge of the largest town and a cell built in part.
#![allow(non_snake_case)]

use std::collections::BTreeMap;
use std::time::Instant;

use isostate_core::courses::{self, CourseBook};
use isostate_core::squares::{self, PER_CELL, SIDE, Tenant};
use isostate_core::*;

fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

/// An 8-bit RGB picture as a PNG file, stored uncompressed (as `squares` writes it).
fn png(w: usize, h: usize, rgb: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((w * 3 + 1) * h);
    for y in 0..h {
        raw.push(0);
        raw.extend_from_slice(&rgb[y * w * 3..(y + 1) * w * 3]);
    }
    let mut z = vec![0x78, 0x01];
    let (mut a, mut b) = (1u32, 0u32);
    for &v in &raw {
        a = (a + v as u32) % 65521;
        b = (b + a) % 65521;
    }
    let mut chunks = raw.chunks(65535).peekable();
    while let Some(ch) = chunks.next() {
        z.push(chunks.peek().is_none() as u8);
        z.extend_from_slice(&(ch.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(ch.len() as u16)).to_le_bytes());
        z.extend_from_slice(ch);
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut chunk = |kind: &[u8; 4], data: &[u8]| {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32(&body).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(b"IHDR", &ihdr);
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    out
}

/// One year of one run, kept for the pictures: each cell's tenants and water,
/// and the ways to draw (as lines in cells), with the places.
#[derive(Clone, Default)]
struct Shot {
    year: i64,
    cells: BTreeMap<usize, (Vec<u32>, Vec<bool>)>,
    ways: Vec<Vec<(f32, f32)>>,
    places: Vec<(f32, f32)>,
}

fn shot(st: &State, year: i64, own_courses: bool, A: &State) -> Shot {
    let mut cells = BTreeMap::new();
    for (c, &k) in st.sq.index.iter().enumerate() {
        if k > 0 {
            let t = &st.sq.tables[k as usize - 1];
            cells.insert(c, (t.tenant.to_vec(), t.water.to_vec()));
        }
    }
    let (W, H) = (st.world.W, st.world.H);
    let mut ways = Vec::new();
    let mut places = Vec::new();
    if own_courses {
        for (&l, c) in &A.wc.courses {
            if A.wc.drawn[l as usize] {
                ways.push(c.pts.iter().map(|&p| CourseBook::at(p)).collect());
            }
        }
        places = (0..A.N).filter_map(|c| A.wc.place_of(c)).collect();
    } else {
        // the ways as the squares faced them before: from each road cell's meeting place
        // halfway to each neighbouring road cell's
        let mid = |c: usize| st.xs.table.middles.get(c).copied().unwrap_or(((c % W) as f32 + 0.5, (c / W) as f32 + 0.5));
        for c in 0..st.N {
            if st.road[c] == 0 {
                continue;
            }
            for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
                let (nx, ny) = ((c % W) as isize + dx, (c / W) as isize + dy);
                if nx < 0 || ny < 0 || nx as usize >= W || ny as usize >= H {
                    continue;
                }
                let n = ny as usize * W + nx as usize;
                if st.road[n] > 0 {
                    ways.push(vec![mid(c), mid(n)]);
                }
            }
        }
    }
    Shot { year, cells, ways, places }
}

#[derive(Default)]
struct Row {
    year: i64,
    drawn: usize,
    courses: usize,
    places: usize,
    on_eight: (f64, f64),
    straight_on_eight: (f64, f64),
    /// house blocks within a square of a drawn course, and all, with the courses on and off
    houses_on: (usize, usize),
    houses_off: (usize, usize),
    /// works' first squares within a square of a drawn course, and all, on and off
    works_on: (usize, usize),
    works_off: (usize, usize),
}

struct Out {
    seed: u64,
    rows: Vec<Row>,
    faults: Vec<String>,
    times: Vec<(i64, f64, f64)>,
    run_s: f64,
    shots: Vec<(Shot, Shot)>,
    /// windows: (name, x0, y0, x1, y1)
    windows: Vec<(String, usize, usize, usize, usize)>,
    w: usize,
    log_bytes: usize,
    hold_bytes: usize,
}

/// House blocks taken since 1750 and works' first squares within a square of the drawn courses of `A`.
fn near(st: &State, A: &State) -> ((usize, usize), (usize, usize)) {
    let W = st.world.W;
    let (mut h, mut hn, mut wk, mut wn) = (0, 0, 0, 0);
    for (c, &k) in st.sq.index.iter().enumerate() {
        if k == 0 {
            continue;
        }
        let t = &st.sq.tables[k as usize - 1];
        for q in 0..PER_CELL {
            // house blocks taken since the run began (those of the opening stand on ground built before)
            if matches!(Tenant::of(t.tenant[q]), Tenant::Houses(_)) && t.year[q] >= 1750 {
                let (x, y) = ((c % W) as f32 + ((q % SIDE) as f32 + 0.5) / SIDE as f32, (c / W) as f32 + ((q / SIDE) as f32 + 0.5) / SIDE as f32);
                hn += 1;
                if courses::to_course(A, x, y).is_some_and(|d| d <= 1.0) {
                    h += 1;
                }
            }
        }
    }
    for (f, hold) in st.sq.works.iter().enumerate() {
        if !st.f_alive.get(f).copied().unwrap_or(false) || hold.squares.is_empty() {
            continue;
        }
        let (c, q) = (hold.cell, hold.squares[0] as usize);
        let (x, y) = ((c % W) as f32 + ((q % SIDE) as f32 + 0.5) / SIDE as f32, (c / W) as f32 + ((q / SIDE) as f32 + 0.5) / SIDE as f32);
        wn += 1;
        if courses::to_course(A, x, y).is_some_and(|d| d <= 1.0) {
            wk += 1;
        }
    }
    ((h, hn), (wk, wn))
}

/// The share of the straight lines between the meeting places of the drawn links within `deg` of the eight directions.
fn straight_on_eight(A: &State, deg: f32) -> (f64, f64) {
    let (W, H) = (A.world.W, A.world.H);
    let (mut on, mut all) = (0.0, 0.0);
    for l in 0..A.wc.drawn.len() {
        if !A.wc.drawn[l] {
            continue;
        }
        let Some(m) = traffic::other_end(W, H, l / 4, l % 4) else { continue };
        let (p, q) = (A.xs.table.middles[l / 4], A.xs.table.middles[m]);
        let len = (q.0 - p.0).hypot(q.1 - p.1) as f64;
        let a = (q.1 - p.1).atan2(q.0 - p.0).to_degrees().rem_euclid(45.0);
        if a.min(45.0 - a) <= deg {
            on += len;
        }
        all += len;
    }
    (on, all)
}

#[allow(clippy::too_many_arguments)]
fn run(seed: u64, to: i64, families: usize, landfall: bool, time: bool, pictures: bool) -> Out {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = Some(to);
    if landfall {
        isostate_core::landfall::landfall(&mut P);
    }
    let mut off = P.clone();
    off.courses = courses::CourseParams::default();
    let t0 = Instant::now();
    let mut a = sim::start(&P, game_content());
    let mut b = sim::start(&off, game_content());
    let mut out = Out { seed, rows: Vec::new(), faults: Vec::new(), times: Vec::new(), run_s: 0.0, shots: Vec::new(), windows: Vec::new(), w: a.st.world.W, log_bytes: 0, hold_bytes: 0 };
    let mut c_sim = time.then(|| {
        let mut bare = P.clone();
        bare.courses = courses::CourseParams::default();
        bare.squares = squares::SquaresParams::default();
        let mut s = sim::start(&bare, game_content());
        courses::open(&mut s.st, &s.content, &P, s.year);
        squares::open(&mut s.st, &s.content, &P, s.year);
        s
    });
    let mut places: BTreeMap<usize, [u16; 2]> = BTreeMap::new();
    let mut found: BTreeMap<u32, Vec<[u16; 2]>> = BTreeMap::new();
    let mut cursor = LiveCursor::new(&a.content);
    #[allow(clippy::type_complexity)]
    let mut reader: (BTreeMap<u32, [u16; 2]>, BTreeMap<u32, courses::Course>, std::collections::BTreeSet<u32>) = Default::default();
    let years: Vec<i64> = vec![1750, 1800, 1850, 1900, 1950, 2000, 2050];
    let shots_years = [1800i64, 1850, 1900, 2050];
    while a.year <= a.end_year {
        let year = a.year;
        sim::run_year(&mut a).expect("money is conserved");
        sim::run_year(&mut b).expect("money is conserved");
        if let Some(s) = c_sim.as_mut() {
            sim::run_year(s).expect("money is conserved");
            let t = Instant::now();
            courses::close_year(&mut s.st, &s.content, &P, year);
            let tc = t.elapsed().as_secs_f64();
            let t = Instant::now();
            squares::close_year(&mut s.st, &s.content, &P, year);
            out.times.push((year, tc, t.elapsed().as_secs_f64()));
        }
        out.log_bytes += a.st.wc.log.len();
        out.hold_bytes += a.st.sq.holds.len();
        // replay and live: the live year carries the record's bytes, and a reader of
        // them holds the book's places, courses and drawn links
        let y = live_year(&mut cursor, &a.st, &a.content, &a.params, &a.events);
        let k = y.index;
        let (r, w, sq) = (&a.rec, y.ways.as_ref().unwrap(), y.squares.as_ref().unwrap());
        if w.whole || r.way_log[r.way_first[k] as usize..r.way_first[k + 1] as usize] != w.log[..] {
            out.faults.push(format!("{year}: the live year's ways are not the record's"));
        }
        if r.sq_hold[r.sq_hold_first[k] as usize..r.sq_hold_first[k + 1] as usize] != sq.holds[..] || r.squares[r.squares_first[k] as usize..r.squares_first[k + 1] as usize] != sq.changes[..] {
            out.faults.push(format!("{year}: the live year's squares or holdings are not the record's"));
        }
        for (c, p, _) in courses::decode_year(&w.log).expect("the ways' log reads").places {
            reader.0.insert(c, p);
        }
        for (l, d, course) in courses::decode_year(&w.log).unwrap().links {
            if let Some(cs) = course {
                reader.1.insert(l, cs);
            }
            if d { reader.2.insert(l); } else { reader.2.remove(&l); }
        }
        let book_places: BTreeMap<u32, [u16; 2]> = (0..a.st.N).filter(|&c| a.st.wc.placed[c] != 0).map(|c| (c as u32, a.st.wc.place[c])).collect();
        let book_drawn: std::collections::BTreeSet<u32> = (0..a.st.wc.drawn.len()).filter(|&l| a.st.wc.drawn[l]).map(|l| l as u32).collect();
        if reader.0 != book_places || reader.1 != a.st.wc.courses || reader.2 != book_drawn {
            out.faults.push(format!("{year}: a reader of the ways' log does not hold the book's ways"));
        }
        // the courses are read by no rule
        if a.st.pop != b.st.pop || a.st.money_now.to_bits() != b.st.money_now.to_bits() {
            out.faults.push(format!("{year}: the people or the money differ with the courses on and off"));
        }
        // a place never moves; a kept course keeps its points
        for c in 0..a.st.N {
            if let Some(p) = a.st.wc.place_of(c) {
                let q = a.st.wc.place[c];
                let was = *places.entry(c).or_insert(q);
                if was != q {
                    out.faults.push(format!("{year}: the place of cell {c} moved from {was:?} to {p:?}"));
                }
            }
        }
        for (&l, c) in &a.st.wc.courses {
            let was = found.entry(l).or_insert_with(|| c.pts.clone());
            if *was != c.pts {
                out.faults.push(format!("{year}: the course of link {l} moved"));
            }
        }
        if years.contains(&year) || year == a.end_year {
            let ((h, hn), (wk, wn)) = near(&a.st, &a.st);
            let ((h2, hn2), (wk2, wn2)) = near(&b.st, &a.st);
            out.rows.push(Row {
                year,
                drawn: a.st.wc.drawn.iter().filter(|&&d| d).count(),
                courses: a.st.wc.courses.len(),
                places: a.st.wc.placed.iter().filter(|&&p| p != 0).count(),
                on_eight: courses::share_on_eight(&a.st, 5.0),
                straight_on_eight: straight_on_eight(&a.st, 5.0),
                houses_on: (h, hn),
                houses_off: (h2, hn2),
                works_on: (wk, wn),
                works_off: (wk2, wn2),
            });
        }
        if pictures && (year == 1900 || (year == a.end_year && out.windows.is_empty())) {
            out.windows = windows(&a.st);
        }
        if pictures && shots_years.contains(&year) {
            out.shots.push((shot(&b.st, year, false, &a.st), shot(&a.st, year, true, &a.st)));
        }
    }
    out.run_s = t0.elapsed().as_secs_f64();
    out
}

/// Two windows of the island in 1900: about the edge of the largest town (its
/// cell furthest from its middle, five cells by five) and about a cell its
/// town has built in part (three by three).
fn windows(st: &State) -> Vec<(String, usize, usize, usize, usize)> {
    let (W, H) = (st.world.W, st.world.H);
    let mut out = Vec::new();
    let Some(big) = (0..st.town_ground.len()).filter(|&i| !st.town_ground[i].is_empty()).max_by(|&a, &b| st.urban_pop[a].total_cmp(&st.urban_pop[b])) else { return out };
    let node = st.town_node[big];
    let far = |c: usize| ((c % W) as f32 - (node % W) as f32).hypot((c / W) as f32 - (node / W) as f32);
    let edge = st.town_ground[big].iter().copied().max_by(|&a, &b| far(a).total_cmp(&far(b)).then(b.cmp(&a))).unwrap_or(node);
    let win = |c: usize, r: usize| ((c % W).saturating_sub(r), (c / W).saturating_sub(r), (c % W + r).min(W - 1), (c / W + r).min(H - 1));
    let (x0, y0, x1, y1) = win(edge, 2);
    out.push((format!("edge-{}", st.town_name[big]), x0, y0, x1, y1));
    // a town cell whose house blocks are between a quarter and three fifths of its squares, with the most courses about it
    let mut best: Option<(usize, usize)> = None;
    for (c, &k) in st.sq.index.iter().enumerate() {
        if k == 0 || !st.urban[c] {
            continue;
        }
        let t = &st.sq.tables[k as usize - 1];
        let houses = t.tenant.iter().filter(|&&x| matches!(Tenant::of(x), Tenant::Houses(_))).count();
        if houses * 4 >= PER_CELL && houses * 5 <= PER_CELL * 3 {
            let n = st.wc.near[c].len();
            if best.is_none_or(|(m, _)| n > m) {
                best = Some((n, c));
            }
        }
    }
    if let Some((_, c)) = best {
        let (x0, y0, x1, y1) = win(c, 1);
        out.push((format!("part-built-{}", c), x0, y0, x1, y1));
    }
    out
}

fn colour(tn: Tenant, under: bool) -> [u8; 3] {
    let base = match tn {
        Tenant::None => [214, 222, 196],
        Tenant::Works(f) => {
            let h = isostate_core::crossings::hash2(f as i32, 3, 77);
            [(70.0 + 60.0 * h) as u8, (80.0 + 30.0 * (1.0 - h)) as u8, (140.0 + 50.0 * h) as u8]
        }
        Tenant::Houses(_) => [196, 110, 72],
        Tenant::Park(_) => [92, 158, 78],
        Tenant::Pit(_) => [60, 56, 52],
        Tenant::Derelict(_) => [150, 138, 110],
        Tenant::Open(_) => [236, 222, 168],
    };
    if under {
        if tn == Tenant::None { [150, 190, 220] } else { [(base[0] as u16 / 2 + 60) as u8, (base[1] as u16 / 2 + 80) as u8, (base[2] as u16 / 2 + 110) as u8] }
    } else {
        base
    }
}

fn dot(rgb: &mut [u8], w: usize, h: usize, x: f32, y: f32, r: isize, col: [u8; 3]) {
    for yy in -r..=r {
        for xx in -r..=r {
            let (px, py) = (x as isize + xx, y as isize + yy);
            if xx * xx + yy * yy <= r * r && px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                let i = (py as usize * w + px as usize) * 3;
                rgb[i..i + 3].copy_from_slice(&col);
            }
        }
    }
}

fn line(rgb: &mut [u8], w: usize, h: usize, a: (f32, f32), b: (f32, f32), col: [u8; 3]) {
    let n = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) * 1.5).ceil().max(1.0) as usize;
    for j in 0..=n {
        let t = j as f32 / n as f32;
        dot(rgb, w, h, a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1), 1, col);
    }
}

/// The window's squares in each shot year: before on the left, with the courses on the right.
fn picture(o: &Out, win: &(String, usize, usize, usize, usize), path: &str) {
    let W = o.w;
    let (_, x0, y0, x1, y1) = *win;
    let (cw, ch) = (x1 - x0 + 1, y1 - y0 + 1);
    let px = (480 / (cw.max(ch) * SIDE)).clamp(2, 12);
    let (pw, ph) = (cw * SIDE * px, ch * SIDE * px);
    let gap = 12;
    let (W2, H2) = (2 * pw + 3 * gap, o.shots.len() * ph + (o.shots.len() + 1) * gap);
    let mut rgb = vec![255u8; W2 * H2 * 3];
    for (row, pair) in o.shots.iter().enumerate() {
        for (col, s) in [&pair.0, &pair.1].into_iter().enumerate() {
            let (ox, oy) = (gap + col * (pw + gap), gap + row * (ph + gap));
            let mut panel = vec![0u8; pw * ph * 3];
            let qw = cw * SIDE;
            let mut ten = vec![0u32; qw * ch * SIDE];
            for cy in y0..=y1 {
                for cx in x0..=x1 {
                    let c = cy * W + cx;
                    for k in 0..PER_CELL {
                        let (code, under) = s.cells.get(&c).map_or((0, false), |(t, wa)| (t[k], wa[k]));
                        let colr = colour(Tenant::of(code), under);
                        let (sx, sy) = ((cx - x0) * SIDE + k % SIDE, (cy - y0) * SIDE + k / SIDE);
                        ten[sy * qw + sx] = code;
                        for yy in 0..px {
                            for xx in 0..px {
                                let p = ((sy * px + yy) * pw + sx * px + xx) * 3;
                                panel[p..p + 3].copy_from_slice(&colr);
                            }
                        }
                    }
                }
            }
            // a darker edge where two tenants meet
            for sy in 0..ch * SIDE {
                for sx in 0..qw {
                    let a = ten[sy * qw + sx];
                    for (dx, dy) in [(1usize, 0usize), (0, 1)] {
                        let (bx, by) = (sx + dx, sy + dy);
                        if bx >= qw || by >= ch * SIDE || ten[by * qw + bx] == a {
                            continue;
                        }
                        for j in 0..px {
                            let (x, y) = if dx == 1 { (bx * px, sy * px + j) } else { (sx * px + j, by * px) };
                            let p = (y * pw + x) * 3;
                            for v in &mut panel[p..p + 3] {
                                *v = (*v as u16 * 3 / 5) as u8;
                            }
                        }
                    }
                }
            }
            // the cells' edges, faint
            for cy in 0..=ch {
                let y = (cy * SIDE * px).min(ph - 1);
                for x in 0..pw {
                    let p = (y * pw + x) * 3;
                    panel[p..p + 3].copy_from_slice(&[110, 110, 110]);
                }
            }
            for cx in 0..=cw {
                let x = (cx * SIDE * px).min(pw - 1);
                for y in 0..ph {
                    let p = (y * pw + x) * 3;
                    panel[p..p + 3].copy_from_slice(&[110, 110, 110]);
                }
            }
            let to = |p: (f32, f32)| ((p.0 - x0 as f32) * (SIDE * px) as f32, (p.1 - y0 as f32) * (SIDE * px) as f32);
            for way in &s.ways {
                for w2 in way.windows(2) {
                    line(&mut panel, pw, ph, to(w2[0]), to(w2[1]), [40, 36, 30]);
                }
            }
            for &p in &s.places {
                let (x, y) = to(p);
                dot(&mut panel, pw, ph, x, y, 3, [200, 20, 20]);
            }
            for y in 0..ph {
                let d = ((oy + y) * W2 + ox) * 3;
                rgb[d..d + pw * 3].copy_from_slice(&panel[y * pw * 3..(y + 1) * pw * 3]);
            }
        }
    }
    std::fs::write(path, png(W2, H2, &rgb)).unwrap();
    println!("wrote {path}: island {}, cells {x0}..={x1} by {y0}..={y1}, {px} px a square; rows {:?}; left before, right with the courses", o.seed, o.shots.iter().map(|s| s.0.year).collect::<Vec<_>>());
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = vec![7];
    let (mut to, mut families, mut time, mut landfall, mut out_dir) = (2050i64, 6usize, false, false, None::<String>);
    let mut k = 0;
    while k < args.len() {
        match args[k].as_str() {
            "--seeds" => {
                k += 1;
                seeds = args[k]
                    .split(',')
                    .flat_map(|p| match p.split_once('-') {
                        Some((a, b)) => (a.parse::<u64>().unwrap()..=b.parse::<u64>().unwrap()).collect::<Vec<_>>(),
                        None => vec![p.parse().unwrap()],
                    })
                    .collect();
            }
            "--to" => {
                k += 1;
                to = args[k].parse().unwrap();
            }
            "--families" => {
                k += 1;
                families = args[k].parse().unwrap();
            }
            "--start" => {
                k += 1;
                landfall = args[k] == "landfall";
            }
            "--time" => time = true,
            "--out" => {
                k += 1;
                out_dir = Some(args[k].clone());
            }
            a => panic!("unknown argument {a}"),
        }
        k += 1;
    }
    let first = seeds[0];
    let pictures = out_dir.is_some();
    let outs: Vec<Out> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.iter().map(|&s| sc.spawn(move || run(s, to, families, landfall, time, pictures && s == first))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let share = |x: (f64, f64)| if x.1 > 0.0 { x.0 / x.1 } else { 0.0 };
    let frac = |x: (usize, usize)| if x.1 > 0 { x.0 as f64 / x.1 as f64 } else { 0.0 };
    for o in &outs {
        println!("\nisland {} ({}, {:.1} s for both runs)", o.seed, if landfall { "landfall" } else { "settled" }, o.run_s);
        println!("  year  drawn  courses  places  on-eight(courses) on-eight(straight)  houses-near(on) houses-near(off)  works-near(on) works-near(off)");
        for r in &o.rows {
            println!(
                "  {}  {:5}  {:7}  {:6}  {:17.3} {:18.3}  {:7.3} of {:6}  {:7.3} of {:6}  {:6.3} of {:5}  {:6.3} of {:5}",
                r.year,
                r.drawn,
                r.courses,
                r.places,
                share(r.on_eight),
                share(r.straight_on_eight),
                frac(r.houses_on),
                r.houses_on.1,
                frac(r.houses_off),
                r.houses_off.1,
                frac(r.works_on),
                r.works_on.1,
                frac(r.works_off),
                r.works_off.1
            );
        }
        println!("  the ways' log {} bytes, the holdings' log {} bytes", o.log_bytes, o.hold_bytes);
        println!("  checks: {} faults{}", o.faults.len(), o.faults.first().map_or(String::new(), |f| format!("; the first: {f}")));
        if !o.times.is_empty() {
            let mut c: Vec<f64> = o.times.iter().map(|t| t.1 * 1e3).collect();
            let mut s: Vec<f64> = o.times.iter().map(|t| t.2 * 1e3).collect();
            let (tc, ts): (f64, f64) = (c.iter().sum(), s.iter().sum());
            c.sort_by(f64::total_cmp);
            s.sort_by(f64::total_cmp);
            let slow = o.times.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
            println!(
                "  time at the year's end (one thread, the clock): courses {:.0} ms in all, a year's median {:.2} ms, the most {:.1} ms in {}; squares {:.0} ms in all, median {:.2} ms",
                tc,
                c[c.len() / 2],
                slow.1 * 1e3,
                slow.0,
                ts,
                s[s.len() / 2]
            );
        }
    }
    if let Some(dir) = out_dir {
        std::fs::create_dir_all(&dir).unwrap();
        let o = &outs[0];
        for win in &o.windows {
            picture(o, win, &format!("{dir}/courses-{}-{}.png", o.seed, win.0));
        }
    }
}
