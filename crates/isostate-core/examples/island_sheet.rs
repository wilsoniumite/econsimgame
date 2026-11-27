//! Where the fertile ground is and where the farms of 1800 are, at three rains
//! (the island's settings, docs/rules/land.md, "The island's settings").
//!
//!     cargo run --profile work -p isostate-core --example island_sheet -- SEEDS OUT.png [RAIN,RAIN,...]
//!
//! Each island of SEEDS (`1-8`) is made at each rain (450, 900 and 1350 mm a year
//! unless named) and played under the game's rules with six families to 1800.
//! A line for each: the rain, the land cells with corn land over 0.5, the farms
//! of 1800 (cells), the mean rain where they stand, the share of them on corn
//! land over 0.5, and how far east their middle lies, as a share of the land's
//! width (the wind comes from the west, so the wet side is the west).
//!
//! The picture is of the first island, the rains side by side from the least:
//! the land shaded by its corn land (pale where poor, deep green where rich),
//! the sea blue, the farms of 1800 a brown square in their cell, the towns of
//! 1800 black.
#![allow(non_snake_case)]
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

/// An 8-bit RGB picture as a PNG file (stored blocks, as `land_picture` writes it).
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

/// What a run to 1800 leaves for the sheet: by cell, land, corn land, rain, a
/// farm in 1800, a town in 1800.
struct Panel {
    W: usize,
    H: usize,
    land: Vec<bool>,
    fert: Vec<f64>,
    rain: Vec<f64>,
    farm: Vec<bool>,
    town: Vec<bool>,
}

fn play(seed: u64, rain: f64) -> Panel {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(1800);
    island::Island { rain: Some(rain), ..Default::default() }.set(&mut P).unwrap();
    let mut sim = sim::start(&P, game_content());
    while sim.year <= sim.end_year {
        for m in 0..12 {
            sim::run_month(&mut sim, m).unwrap();
        }
        sim::end_year(&mut sim);
    }
    let (W, H) = (P.W, P.H);
    let C = &sim.content;
    let grain = C.gi("grain");
    let farm_use: Vec<u8> = (0..C.NS).filter(|&s| C.sector_good[s] == grain).map(|s| land::USE_SECTOR0 + s as u8).collect();
    let st = &sim.st;
    let mut town = vec![false; W * H];
    for t in 0..st.T_max {
        if st.town_active[t] {
            town[st.town_node[t]] = true;
        }
    }
    let world = landcells::land_alone(seed, W, H, &P.land);
    let rain = world.landform.as_ref().map_or(vec![0.0; W * H], |lf| lf.rain.iter().map(|&r| r as f64).collect());
    Panel { W, H, land: world.land.clone(), fert: world.fertility.clone(), rain, farm: st.land_use.iter().map(|u| farm_use.contains(u)).collect(), town }
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seeds = sweep::parse_seeds(&a[0]).unwrap();
    let out = &a[1];
    let rains: Vec<f64> = a.get(2).map_or(vec![450.0, 900.0, 1350.0], |s| s.split(',').map(|x| x.parse().unwrap()).collect());
    let jobs: Vec<(u64, f64)> = seeds.iter().flat_map(|&s| rains.iter().map(move |&r| (s, r))).collect();
    let panels: Vec<Panel> = std::thread::scope(|sc| {
        let hs: Vec<_> = jobs.iter().map(|&(s, r)| sc.spawn(move || play(s, r))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    println!("seed rain | fertile_cells farms_1800 mean_rain_at_farms farms_on_fertile farms_middle_east");
    for ((s, r), p) in jobs.iter().zip(&panels) {
        let n = p.W * p.H;
        let fertile = (0..n).filter(|&c| p.land[c] && p.fert[c] > 0.5).count();
        let farms: Vec<usize> = (0..n).filter(|&c| p.farm[c]).collect();
        let nf = farms.len().max(1) as f64;
        let mean_rain = farms.iter().map(|&c| p.rain[c]).sum::<f64>() / nf;
        let on_fertile = farms.iter().filter(|&&c| p.fert[c] > 0.5).count() as f64 / nf;
        let xs: Vec<usize> = (0..n).filter(|&c| p.land[c]).map(|c| c % p.W).collect();
        let (x0, x1) = (*xs.iter().min().unwrap_or(&0) as f64, *xs.iter().max().unwrap_or(&0) as f64);
        let mid = farms.iter().map(|&c| (c % p.W) as f64).sum::<f64>() / nf;
        println!("{s} {r} | {fertile} {} {mean_rain:.0} {on_fertile:.3} {:.3}", farms.len(), (mid - x0) / (x1 - x0).max(1.0));
    }
    // the picture: the first island's panels side by side
    let px = 5;
    let gap = 12;
    let first: Vec<&Panel> = panels.iter().take(rains.len()).collect();
    let (W, H) = (first[0].W, first[0].H);
    let (pw, ph) = (W * px * first.len() + gap * (first.len() + 1), H * px + 2 * gap);
    let mut rgb = vec![250u8; pw * ph * 3];
    for (k, p) in first.iter().enumerate() {
        let ox = gap + k * (W * px + gap);
        for c in 0..W * H {
            let (x, y) = (c % W, c / W);
            let f = p.fert[c].clamp(0.0, 1.0);
            let base = if !p.land[c] {
                [150u8, 182, 204]
            } else {
                let lerp = |a: f64, b: f64| (a + (b - a) * f) as u8;
                [lerp(236.0, 52.0), lerp(226.0, 128.0), lerp(190.0, 52.0)]
            };
            for dy in 0..px {
                for dx in 0..px {
                    let inner = (1..px - 1).contains(&dx) && (1..px - 1).contains(&dy);
                    let col = if p.town[c] {
                        [20, 20, 20]
                    } else if p.farm[c] && inner {
                        [150, 92, 40]
                    } else {
                        base
                    };
                    let i = ((gap + y * px + dy) * pw + ox + x * px + dx) * 3;
                    rgb[i..i + 3].copy_from_slice(&col);
                }
            }
        }
    }
    std::fs::write(out, png(pw, ph, &rgb)).unwrap();
    println!("wrote {out}");
}
