//! A picture of an island made by the land mechanism, with no viewer: hill-shaded
//! relief, the rivers by their width, lakes, the coast, the first towns, and if
//! asked a field laid over it as a colour ramp.
//!
//!     cargo run --release -p isostate-core --example land_picture -- --seeds 1-12 --out DIR
//!         [--field rain|flow|fertility|pasture|forest|rock|clay|coal|ore|wetness|power|terrain|geology|basin|wet|navigable]
//!         [--zoom X,Y,WIDTH]   a window WIDTH cells wide about cell (X, Y)
//!         [--px N]             picture width in pixels (960)
//!         [--set NAME=VALUE]   any of the land's numbers, as often as wanted
//!
//! Writes DIR/land-SEED[-FIELD][-zoom].png. The PNG is written here, uncompressed
//! (stored blocks), so that nothing new is depended on.
#![allow(non_snake_case)]
use std::time::Instant;

use isostate_core::landcells::{Landform, generate_land};
use isostate_core::landform::LandParams;
use isostate_core::world::World;

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

/// An 8-bit RGB picture as a PNG file.
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

struct Canvas {
    w: usize,
    h: usize,
    rgb: Vec<u8>,
    /// The window in cell units: left, top, and cells per pixel.
    x0: f64,
    y0: f64,
    per_px: f64,
}

impl Canvas {
    fn put(&mut self, px: isize, py: isize, col: [u8; 3], alpha: f64) {
        if px >= 0 && py >= 0 && (px as usize) < self.w && (py as usize) < self.h {
            let p = (py as usize * self.w + px as usize) * 3;
            for q in 0..3 {
                self.rgb[p + q] = (self.rgb[p + q] as f64 * (1.0 - alpha) + col[q] as f64 * alpha) as u8;
            }
        }
    }

    /// A dot of radius r pixels about a point in cell units.
    fn dot(&mut self, x: f64, y: f64, r: f64, col: [u8; 3]) {
        let (cx, cy) = ((x - self.x0) / self.per_px, (y - self.y0) / self.per_px);
        let rr = r.ceil() as isize + 1;
        for dy in -rr..=rr {
            for dx in -rr..=rr {
                let (px, py) = (cx.floor() as isize + dx, cy.floor() as isize + dy);
                let d = ((px as f64 + 0.5 - cx).hypot(py as f64 + 0.5 - cy) - r).clamp(-0.5, 0.5);
                if d < 0.5 {
                    self.put(px, py, col, 0.5 - d);
                }
            }
        }
    }

    /// A line between two points in cell units, `wide` pixels across.
    fn line(&mut self, a: (f64, f64), b: (f64, f64), wide: f64, col: [u8; 3]) {
        let len = ((b.0 - a.0).hypot(b.1 - a.1) / self.per_px).ceil().max(1.0);
        let n = (len * 1.5) as usize;
        for k in 0..=n {
            let t = k as f64 / n as f64;
            self.dot(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, 0.5 * wide, col);
        }
    }
}

fn ramp(stops: &[(f64, [f64; 3])], v: f64) -> [f64; 3] {
    let mut out = stops[stops.len() - 1].1;
    for k in 0..stops.len() - 1 {
        if v < stops[k + 1].0 {
            let u = ((v - stops[k].0) / (stops[k + 1].0 - stops[k].0)).clamp(0.0, 1.0);
            for q in 0..3 {
                out[q] = stops[k].1[q] + u * (stops[k + 1].1[q] - stops[k].1[q]);
            }
            break;
        }
    }
    out
}

fn height_at(lf: &Landform, x: f64, y: f64) -> f64 {
    let k = lf.fine as f64;
    let fx = (x * k - 0.5).clamp(0.0, (lf.fw - 1) as f64);
    let fy = (y * k - 0.5).clamp(0.0, (lf.fh - 1) as f64);
    let (i, j) = ((fx as usize).min(lf.fw - 2), (fy as usize).min(lf.fh - 2));
    let (u, v) = (fx - i as f64, fy - j as f64);
    let n = j * lf.fw + i;
    let z = |n: usize| lf.height[n] as f64;
    (z(n) * (1.0 - u) + z(n + 1) * u) * (1.0 - v) + (z(n + lf.fw) * (1.0 - u) + z(n + lf.fw + 1) * u) * v
}

/// A cell field's value and colour ramp, for `--field`.
fn field_colour(w: &World, lf: &Landform, name: &str, c: usize, node: usize) -> Option<[f64; 3]> {
    let heat: [(f64, [f64; 3]); 4] = [(0.0, [250.0, 248.0, 225.0]), (0.33, [245.0, 200.0, 90.0]), (0.66, [215.0, 95.0, 40.0]), (1.0, [110.0, 20.0, 30.0])];
    let wetr: [(f64, [f64; 3]); 4] = [(0.0, [240.0, 225.0, 170.0]), (0.33, [150.0, 205.0, 150.0]), (0.66, [60.0, 150.0, 190.0]), (1.0, [30.0, 40.0, 130.0])];
    if !w.land[c] {
        return None;
    }
    Some(match name {
        "rain" => ramp(&wetr, ((lf.rain[c] as f64 - 400.0) / 2400.0).clamp(0.0, 1.0)),
        "flow" => ramp(&wetr, ((lf.flow[node] as f64).max(1e-3).log10() + 2.0) / 4.0),
        "wetness" => ramp(&wetr, lf.wetness[c] as f64),
        "power" => ramp(&heat, ((lf.power[c] as f64).max(1.0).log10() / 4.5).clamp(0.0, 1.0)),
        "fertility" => ramp(&heat, w.fertility[c]),
        "pasture" => ramp(&heat, w.pasture[c]),
        "forest" => ramp(&heat, w.forest_pot[c]),
        "rock" => ramp(&heat, w.rock[c]),
        "clay" => ramp(&heat, w.clay[c]),
        "coal" => ramp(&heat, w.coal[c]),
        "ore" => ramp(&heat, w.ore[c]),
        "terrain" => [[0.0, 0.0, 0.0], [150.0, 200.0, 120.0], [215.0, 190.0, 120.0], [150.0, 120.0, 110.0]][w.terrain[c] as usize],
        // the fine grid's wet ground: flood plain pale, marsh dark; dry ground as it is
        "wet" => match lf.wet[node] {
            1 => [215.0, 225.0, 150.0],
            2 => [50.0, 140.0, 130.0],
            _ => return None,
        },
        // the cells boats use: a tideway's, a navigable river's, and a river too small for them
        "navigable" => {
            if lf.estuary[c] && w.navigable[c] {
                [150.0, 60.0, 160.0]
            } else if w.navigable[c] {
                [40.0, 80.0, 200.0]
            } else if w.river[c] {
                [140.0, 190.0, 235.0]
            } else {
                return None;
            }
        }
        "geology" => [
            [150.0, 110.0, 130.0],
            [235.0, 205.0, 120.0],
            [170.0, 190.0, 215.0],
            [225.0, 225.0, 190.0],
            [165.0, 205.0, 150.0],
            [90.0, 90.0, 95.0],
            [250.0, 240.0, 200.0],
        ][lf.rock[node] as usize],
        "basin" => {
            let b = lf.basin[c].max(0) as u64;
            let hsh = b.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40;
            [90.0 + (hsh & 127) as f64, 90.0 + ((hsh >> 7) & 127) as f64, 90.0 + ((hsh >> 14) & 127) as f64]
        }
        other => panic!("no field {other:?}"),
    })
}

fn draw(w: &World, field: Option<&str>, zoom: Option<(f64, f64, f64)>, px: usize) -> (usize, usize, Vec<u8>) {
    let lf = w.landform.as_ref().expect("an island made by the land mechanism");
    let (x0, y0, wide) = match zoom {
        Some((x, y, wd)) => (x - 0.5 * wd, y - 0.5 * wd * w.H as f64 / w.W as f64, wd),
        None => (0.0, 0.0, w.W as f64),
    };
    let per_px = wide / px as f64;
    let ph = (px as f64 * w.H as f64 / w.W as f64) as usize;
    let mut cv = Canvas { w: px, h: ph, rgb: vec![0; px * ph * 3], x0, y0, per_px };
    let hyps: [(f64, [f64; 3]); 7] = [
        (0.0, [112.0, 160.0, 96.0]),
        (40.0, [150.0, 186.0, 108.0]),
        (120.0, [206.0, 208.0, 134.0]),
        (260.0, [214.0, 180.0, 116.0]),
        (450.0, [178.0, 132.0, 94.0]),
        (750.0, [152.0, 138.0, 132.0]),
        (1100.0, [245.0, 245.0, 245.0]),
    ];
    let k = lf.fine as f64;
    // metres on the ground per pixel, for the shading
    let step = per_px * 2000.0;
    for py in 0..ph {
        for pxi in 0..px {
            let (x, y) = (x0 + (pxi as f64 + 0.5) * per_px, y0 + (py as f64 + 0.5) * per_px);
            let inside = x >= 0.0 && y >= 0.0 && x < w.W as f64 && y < w.H as f64;
            let (i, j) = (((x * k) as usize).min(lf.fw - 1), ((y * k) as usize).min(lf.fh - 1));
            let node = j * lf.fw + i;
            let cell = ((y as usize).min(w.H - 1)) * w.W + (x as usize).min(w.W - 1);
            let z = if inside { height_at(lf, x, y) } else { -100.0 };
            let col = if !inside || (lf.height[node] <= 0.0 && lf.lake_id[node] == 0 && z <= 0.5) {
                let d = (-z / 120.0).clamp(0.0, 1.0);
                [86.0 - 46.0 * d, 132.0 - 52.0 * d, 178.0 - 40.0 * d]
            } else if lf.lake_id[node] > 0 {
                [70.0, 118.0, 186.0]
            } else {
                let (zl, zu) = (height_at(lf, x - per_px, y), height_at(lf, x, y - per_px));
                let (gx, gy) = ((z - zl.max(0.0)) / step, (z - zu.max(0.0)) / step);
                // low ground is drawn with its relief exaggerated, or it would look flat
                let ex = 1.0 + 5.0 / (1.0 + z.max(0.0) / 60.0);
                let shade = (0.82 + ex * 0.7 * (gx + gy)).clamp(0.35, 1.3);
                let mut base = ramp(&hyps, z.max(0.0));
                if let Some(f) = field {
                    if let Some(fc) = field_colour(w, lf, f, cell, node) {
                        base = fc;
                    }
                } else if lf.wet[node] == 2 {
                    base = [base[0] * 0.8, base[1] * 0.92, base[2] * 0.95 + 14.0];
                }
                [base[0] * shade, base[1] * shade, base[2] * shade]
            };
            let p = (py * px + pxi) * 3;
            for q in 0..3 {
                cv.rgb[p + q] = col[q].clamp(0.0, 255.0) as u8;
            }
        }
    }
    // rivers by their width (never thinner than a hair, or the small ones vanish)
    let m_per_px = per_px * 2000.0;
    for ox in &lf.oxbows {
        for i in 1..ox.x.len() {
            cv.line((ox.x[i - 1] as f64, ox.y[i - 1] as f64), (ox.x[i] as f64, ox.y[i] as f64), (ox.width as f64 / m_per_px).max(0.8), [96, 140, 196]);
        }
    }
    for r in &lf.rivers {
        for i in 1..r.x.len() {
            let wd = (r.width[i] as f64 / m_per_px).max(0.35 + 0.25 * (r.flow[i] as f64).max(0.1).log10().max(0.0));
            let col = if r.braided[i] { [120, 150, 200] } else { [44, 84, 168] };
            cv.line((r.x[i - 1] as f64, r.y[i - 1] as f64), (r.x[i] as f64, r.y[i] as f64), wd, col);
        }
    }
    for (k, &c) in w.town_sites.iter().enumerate() {
        let (x, y) = ((c % w.W) as f64 + 0.5, (c / w.W) as f64 + 0.5);
        cv.dot(x, y, 4.5, [20, 20, 20]);
        cv.dot(x, y, 3.0, if k == 0 { [250, 230, 60] } else { [240, 240, 240] });
    }
    (cv.w, cv.h, cv.rgb)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut out, mut field, mut zoom, mut px) = (vec![7u64], ".".to_string(), None, None, 960usize);
    let mut over = serde_json::to_value(LandParams::game()).unwrap();
    let mut i = 0;
    while i < args.len() {
        let v = args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--seeds" => {
                seeds = match v.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => v.split(',').map(|s| s.parse().unwrap()).collect(),
                }
            }
            "--out" => out = v,
            "--field" => field = Some(v),
            "--px" => px = v.parse().unwrap(),
            "--zoom" => {
                let p: Vec<f64> = v.split(',').map(|s| s.parse().unwrap()).collect();
                zoom = Some((p[0], p[1], p[2]));
            }
            "--set" => {
                let (k, val) = v.split_once('=').expect("--set NAME=VALUE");
                over[k] = serde_json::from_str(val).unwrap();
            }
            other => panic!("unknown argument {other:?}"),
        }
        i += 2;
    }
    let P: LandParams = serde_json::from_value(over).unwrap();
    std::fs::create_dir_all(&out).unwrap();
    for seed in seeds {
        let t = Instant::now();
        let w = generate_land(seed, 120, 84, 6, &P);
        let secs = t.elapsed().as_secs_f64();
        let (pw, ph, rgb) = draw(&w, field.as_deref(), zoom, px);
        let name = format!("{out}/land-{seed}{}{}.png", field.as_ref().map_or(String::new(), |f| format!("-{f}")), if zoom.is_some() { "-zoom" } else { "" });
        std::fs::write(&name, png(pw, ph, &rgb)).unwrap();
        let lf = w.landform.as_ref().unwrap();
        println!("seed {seed}: made in {secs:.2} s; {} lakes, {} rivers, {} oxbows; wrote {name}", lf.lakes.len(), lf.rivers.len(), lf.oxbows.len());
    }
}
