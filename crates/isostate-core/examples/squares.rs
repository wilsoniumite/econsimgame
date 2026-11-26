//! The squares of 125 m that the simulation keeps where something is built
//! (`docs/rules/squares.md`): a census by decade, the book checked every year,
//! the time the squares take at a year's end, and pictures of a town's squares.
//!
//!     cargo run --profile work -p isostate-core --example squares -- [--seeds 1-8] [--to 2050]
//!         [--families 6] [--check] [--time] [--out DIR]
//!
//! Each island runs under the game's rules on its own thread.
//!
//! - The census: squares held by works, house blocks, parks, pits, derelict
//!   ground and the towns' open ground, by decade, and the works with no square
//!   of their own.
//! - `--check`: `squares::check` every year, and every square that keeps its
//!   tenant from one year to the next keeps its year; the first fault is printed.
//! - `--time`: the island is run with the squares switched off and the year's
//!   allotting is done beside it, `squares::close_year` called after each year
//!   and timed alone (the squares read the books and write nothing else, so it
//!   is the same work as in the run).
//! - `--out DIR`: for the first island, DIR/squares-SEED-big.png and
//!   DIR/squares-SEED-small.png, the squares of the largest town of the last
//!   year and of a small one, from above, in 1800, 1850, 1900, 1950, 2000 and
//!   2050 (those the run reaches), side by side.
#![allow(non_snake_case)]

use std::collections::BTreeMap;
use std::time::Instant;

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

/// An 8-bit RGB picture as a PNG file, stored uncompressed (as `land_picture` writes it).
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

/// One year's squares, kept for the pictures.
#[derive(Clone, Default)]
struct Shot {
    year: i64,
    /// By cell with a table: tenants, and whether each square is under water.
    cells: BTreeMap<usize, (Vec<u32>, Vec<bool>)>,
    road: Vec<u8>,
    town_ground: Vec<Vec<usize>>,
    town_pop: Vec<f64>,
}

fn shot(st: &State, year: i64) -> Shot {
    let mut cells = BTreeMap::new();
    for (c, &k) in st.sq.index.iter().enumerate() {
        if k > 0 {
            let t = &st.sq.tables[k as usize - 1];
            cells.insert(c, (t.tenant.to_vec(), t.water.to_vec()));
        }
    }
    Shot { year, cells, road: st.road.clone(), town_ground: st.town_ground.clone(), town_pop: st.urban_pop.clone() }
}

struct Out {
    seed: u64,
    census: Vec<(i64, squares::Census)>,
    fault: Option<String>,
    /// Years in which the check found a fault.
    faulty: usize,
    /// Seconds of the squares' allotting, each year, with `--time`.
    times: Vec<(i64, f64)>,
    shots: Vec<Shot>,
    middles: Vec<(f32, f32)>,
    town_node: Vec<usize>,
    town_name: Vec<String>,
    w: usize,
    h: usize,
    run_s: f64,
}

fn run(seed: u64, to: i64, families: usize, check: bool, time: bool, pictures: bool) -> Out {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = Some(to);
    let on = P.clone();
    if time {
        P.squares = squares::SquaresParams::default();
    }
    let t0 = Instant::now();
    let mut s = sim::start(&P, game_content());
    let mut out = Out {
        seed,
        census: Vec::new(),
        fault: None,
        faulty: 0,
        times: Vec::new(),
        shots: Vec::new(),
        middles: Vec::new(),
        town_node: Vec::new(),
        town_name: Vec::new(),
        w: s.st.world.W,
        h: s.st.world.H,
        run_s: 0.0,
    };
    if time {
        let t = Instant::now();
        squares::open(&mut s.st, &s.content, &on, s.year);
        out.times.push((s.year - 1, t.elapsed().as_secs_f64()));
    }
    let mut last: Option<Vec<(usize, Vec<u32>, Vec<i16>)>> = None;
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved");
        if time {
            let t = Instant::now();
            squares::close_year(&mut s.st, &s.content, &on, year);
            out.times.push((year, t.elapsed().as_secs_f64()));
        }
        if check {
            if let Err(e) = squares::check(&s.st, &s.content, &on, year) {
                out.faulty += 1;
                if out.fault.is_none() {
                    out.fault = Some(e);
                }
            }
            // a square that keeps its tenant keeps its year
            let now: Vec<(usize, Vec<u32>, Vec<i16>)> = s.st.sq.index.iter().enumerate().filter(|(_, k)| **k > 0).map(|(c, &k)| {
                let t = &s.st.sq.tables[k as usize - 1];
                (c, t.tenant.to_vec(), t.year.to_vec())
            }).collect();
            if let Some(before) = &last {
                let by: BTreeMap<usize, (&Vec<u32>, &Vec<i16>)> = now.iter().map(|(c, a, b)| (*c, (a, b))).collect();
                'cells: for (c, ta, ya) in before {
                    if let Some((tb, yb)) = by.get(c) {
                        for k in 0..PER_CELL {
                            if ta[k] == tb[k] && ta[k] != 0 && ya[k] != yb[k] {
                                out.faulty += 1;
                                out.fault = out.fault.take().or(Some(format!("{year}: square {k} of cell {c} kept {:?} and its year went from {} to {}", Tenant::of(ta[k]), ya[k], yb[k])));
                                break 'cells;
                            }
                        }
                    }
                }
            }
            last = Some(now);
        }
        if year % 10 == 0 || year == s.end_year {
            out.census.push((year, squares::census(&s.st)));
        }
        if pictures && year % 50 == 0 && year >= 1800 {
            out.shots.push(shot(&s.st, year));
        }
    }
    out.run_s = t0.elapsed().as_secs_f64();
    out.middles = s.st.xs.table.middles.clone();
    out.town_node = s.st.town_node.clone();
    out.town_name = s.st.town_name.clone();
    out
}

fn colour(tn: Tenant, under: bool) -> [u8; 3] {
    let base = match tn {
        Tenant::None => [214, 222, 196],
        Tenant::Works(f) => {
            // works in slate blues and violets, one shade each
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
        // under water: the colour with blue laid over it
        if tn == Tenant::None {
            [150, 190, 220]
        } else {
            [(base[0] as u16 / 2 + 60) as u8, (base[1] as u16 / 2 + 80) as u8, (base[2] as u16 / 2 + 110) as u8]
        }
    } else {
        base
    }
}

fn line(rgb: &mut [u8], w: usize, h: usize, a: (f32, f32), b: (f32, f32), col: [u8; 3]) {
    let n = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) * 1.5).ceil().max(1.0) as usize;
    for j in 0..=n {
        let t = j as f32 / n as f32;
        let (x, y) = (a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1));
        for (ox, oy) in [(0.0, 0.0), (0.5, 0.0), (0.0, 0.5)] {
            let (px, py) = ((x + ox) as isize, (y + oy) as isize);
            if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                let i = (py as usize * w + px as usize) * 3;
                rgb[i..i + 3].copy_from_slice(&col);
            }
        }
    }
}

/// The squares of town i's window in each shot, side by side in two rows of three.
fn picture(o: &Out, i: usize, path: &str) {
    let (W, H) = (o.w, o.h);
    // the window: every cell the town ever held in the shots, and one about it
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    for s in &o.shots {
        for &c in s.town_ground.get(i).map_or(&[][..], |v| &v[..]) {
            let (x, y) = (c % W, c / W);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
    }
    if x0 == usize::MAX {
        let c = o.town_node[i];
        (x0, y0, x1, y1) = (c % W, c / W, c % W, c / W);
    }
    let (mut x0, mut y0) = (x0.saturating_sub(1), y0.saturating_sub(1));
    let (mut x1, mut y1) = ((x1 + 1).min(W - 1), (y1 + 1).min(H - 1));
    // a large town: the nine cells by nine about its middle, so that the squares show
    let (nx, ny) = (o.town_node[i] % W, o.town_node[i] / W);
    if x1 - x0 > 8 {
        (x0, x1) = (nx.saturating_sub(4), (nx + 4).min(W - 1));
    }
    if y1 - y0 > 8 {
        (y0, y1) = (ny.saturating_sub(4), (ny + 4).min(H - 1));
    }
    let (cw, ch) = (x1 - x0 + 1, y1 - y0 + 1);
    let px = ((600 / (cw.max(ch) * SIDE)).clamp(2, 12)) as usize;
    let (pw, ph) = (cw * SIDE * px, ch * SIDE * px);
    let gap = 12;
    let cols = 3;
    let rows = o.shots.len().div_ceil(cols);
    let (W2, H2) = (cols * pw + (cols + 1) * gap, rows * ph + (rows + 1) * gap);
    let mut rgb = vec![255u8; W2 * H2 * 3];
    for (n, s) in o.shots.iter().enumerate() {
        let (ox, oy) = (gap + (n % cols) * (pw + gap), gap + (n / cols) * (ph + gap));
        let mut panel = vec![0u8; pw * ph * 3];
        let mut ten = vec![0u32; pw / px * ph / px];
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                let c = cy * W + cx;
                for k in 0..PER_CELL {
                    let (code, under) = match s.cells.get(&c) {
                        Some((t, wa)) => (t[k], wa[k]),
                        None => (0, false),
                    };
                    let col = colour(Tenant::of(code), under);
                    let (sx, sy) = ((cx - x0) * SIDE + k % SIDE, (cy - y0) * SIDE + k / SIDE);
                    ten[sy * (pw / px) + sx] = code;
                    for yy in 0..px {
                        for xx in 0..px {
                            let p = ((sy * px + yy) * pw + sx * px + xx) * 3;
                            panel[p..p + 3].copy_from_slice(&col);
                        }
                    }
                }
            }
        }
        // a darker edge where two tenants meet, so that each holding shows
        let qw = pw / px;
        for sy in 0..ph / px {
            for sx in 0..qw {
                let a = ten[sy * qw + sx];
                for (dx, dy) in [(1usize, 0usize), (0, 1)] {
                    let (bx, by) = (sx + dx, sy + dy);
                    if bx >= qw || by >= ph / px || ten[by * qw + bx] == a {
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
        // the cells' edges
        for cy in 0..=ch {
            for x in 0..pw {
                let y = (cy * SIDE * px).min(ph - 1);
                let p = (y * pw + x) * 3;
                panel[p..p + 3].copy_from_slice(&[40, 40, 40]);
            }
        }
        for cx in 0..=cw {
            for y in 0..ph {
                let x = (cx * SIDE * px).min(pw - 1);
                let p = (y * pw + x) * 3;
                panel[p..p + 3].copy_from_slice(&[40, 40, 40]);
            }
        }
        // the ways as the viewer joins them, between meeting places of road cells
        let spot = |c: usize| {
            let m = o.middles.get(c).copied().unwrap_or(((c % W) as f32 + 0.5, (c / W) as f32 + 0.5));
            ((m.0 - x0 as f32) * (SIDE * px) as f32, (m.1 - y0 as f32) * (SIDE * px) as f32)
        };
        for cy in y0.saturating_sub(1)..=(y1 + 1).min(H - 1) {
            for cx in x0.saturating_sub(1)..=(x1 + 1).min(W - 1) {
                let c = cy * W + cx;
                if s.road[c] == 0 {
                    continue;
                }
                for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
                    let (nx, ny) = (cx as isize + dx, cy as isize + dy);
                    if nx < 0 || ny < 0 || nx as usize >= W || ny as usize >= H {
                        continue;
                    }
                    let n = ny as usize * W + nx as usize;
                    if s.road[n] > 0 {
                        let col = if s.road[c] >= 2 && s.road[n] >= 2 { [70, 66, 60] } else { [150, 144, 132] };
                        line(&mut panel, pw, ph, spot(c), spot(n), col);
                    }
                }
            }
        }
        // the town's middle
        let (mx, my) = spot(o.town_node[i]);
        for yy in -3isize..=3 {
            for xx in -3isize..=3 {
                let (x, y) = (mx as isize + xx, my as isize + yy);
                if xx * xx + yy * yy <= 9 && x >= 0 && y >= 0 && (x as usize) < pw && (y as usize) < ph {
                    let p = (y as usize * pw + x as usize) * 3;
                    panel[p..p + 3].copy_from_slice(&[200, 20, 20]);
                }
            }
        }
        for y in 0..ph {
            let d = ((oy + y) * W2 + ox) * 3;
            rgb[d..d + pw * 3].copy_from_slice(&panel[y * pw * 3..(y + 1) * pw * 3]);
        }
    }
    std::fs::write(path, png(W2, H2, &rgb)).unwrap();
    println!("wrote {path}: {} in {} ({}x{} cells, {} px a square; years {:?})", o.town_name[i], o.seed, cw, ch, px, o.shots.iter().map(|s| s.year).collect::<Vec<_>>());
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = vec![7];
    let (mut to, mut families, mut check, mut time, mut out_dir) = (2050i64, 6usize, false, false, None::<String>);
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
            "--check" => check = true,
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
        let hs: Vec<_> = seeds.iter().map(|&s| sc.spawn(move || run(s, to, families, check, time, pictures && s == first))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for o in &outs {
        println!("\nisland {} ({:.1} s)", o.seed, o.run_s);
        println!("  year  tables   works  houses   park   pits derelict   open  built under_water built_wet  alive without split  split_steps short");
        for (y, c) in &o.census {
            let built = c.works + c.houses + c.park + c.pit + c.derelict;
            println!(
                "  {y}  {:6}  {:6}  {:6}  {:5}  {:5}  {:7}  {:6}  {:6}  {:10}  {:9}  {:6}  {:6}  {:5}  {:10}  {:5}",
                c.tables, c.works, c.houses, c.park, c.pit, c.derelict, c.open, built, c.under_water, c.built_under_water, c.works_alive, c.works_without, c.works_split, c.split, c.short
            );
        }
        if check {
            println!("  check: {} years with a fault; the first: {}", o.faulty, o.fault.as_deref().unwrap_or("none"));
        }
        if time && !o.times.is_empty() {
            let mut ms: Vec<f64> = o.times.iter().map(|t| t.1 * 1000.0).collect();
            let total: f64 = ms.iter().sum();
            ms.sort_by(|a, b| a.total_cmp(b));
            let max = o.times.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
            println!(
                "  squares' time: {:.0} ms in all ({:.2}% of the run), a year's median {:.2} ms, the 99th {:.2} ms, the most {:.2} ms in {}",
                total,
                100.0 * total / 1000.0 / o.run_s,
                ms[ms.len() / 2],
                ms[(ms.len() * 99 / 100).min(ms.len() - 1)],
                max.1 * 1000.0,
                max.0
            );
        }
    }
    if let Some(dir) = out_dir {
        let o = &outs[0];
        let Some(last) = o.shots.last() else { return };
        let active: Vec<usize> = (0..last.town_ground.len()).filter(|&i| !last.town_ground[i].is_empty()).collect();
        let big = *active.iter().max_by(|&&a, &&b| last.town_pop[a].total_cmp(&last.town_pop[b])).unwrap();
        // a small town: the smallest of those that stood in the first shot
        let small = active
            .iter()
            .copied()
            .filter(|&i| o.shots[0].town_ground.get(i).is_some_and(|v| !v.is_empty()) && i != big)
            .min_by(|&a, &b| last.town_pop[a].total_cmp(&last.town_pop[b]))
            .unwrap_or(big);
        std::fs::create_dir_all(&dir).unwrap();
        picture(o, big, &format!("{dir}/squares-{}-big.png", o.seed));
        picture(o, small, &format!("{dir}/squares-{}-small.png", o.seed));
    }
}
