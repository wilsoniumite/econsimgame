//! The island itself: a heightfield mesh, a painted texture of the land
//! patchwork for each year, and the sea.

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rayon::prelude::*;

use crate::replay::Replay;
use crate::world::{self, Ground, TS, VRES};

pub fn terrain_mesh(g: &Ground) -> Mesh {
    let (gw, gh) = (g.gw, g.gh);
    let step = 1.0 / VRES as f32;
    let mut pos = Vec::with_capacity(gw * gh);
    let mut nrm = Vec::with_capacity(gw * gh);
    let mut uv = Vec::with_capacity(gw * gh);
    for j in 0..gh {
        for i in 0..gw {
            let h = g.hgt[j * gw + i];
            pos.push([i as f32 * step, h, j as f32 * step]);
            let hl = g.hgt[j * gw + i.saturating_sub(1)];
            let hr = g.hgt[j * gw + (i + 1).min(gw - 1)];
            let hu = g.hgt[j.saturating_sub(1) * gw + i];
            let hd = g.hgt[(j + 1).min(gh - 1) * gw + i];
            let n = bevy::math::Vec3::new(hl - hr, 2.0 * step, hu - hd).normalize();
            nrm.push(n.to_array());
            uv.push([i as f32 / (gw - 1) as f32, j as f32 / (gh - 1) as f32]);
        }
    }
    let mut idx = Vec::with_capacity((gw - 1) * (gh - 1) * 6);
    for j in 0..gh - 1 {
        for i in 0..gw - 1 {
            let a = (j * gw + i) as u32;
            let b = a + 1;
            let c = a + gw as u32;
            let d = c + 1;
            idx.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    m.insert_indices(Indices::U32(idx));
    m
}

/// A flat square at height `y`, facing up.
pub fn plane(x0: f32, z0: f32, x1: f32, z1: f32, y: f32) -> Mesh {
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[x0, y, z0], [x1, y, z0], [x1, y, z1], [x0, y, z1]]);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 4]);
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    m.insert_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]));
    m
}

/// Everything about a texel that does not change from year to year.
pub struct TexStatic {
    pub tw: usize,
    pub th: usize,
    cell: Vec<u16>,
    sub: Vec<u8>,
    field: Vec<u8>,
    /// bits 4-5: 0 land, 1 beach, 2 sea floor
    flags: Vec<u8>,
    /// how much of the texel a hedge covers, 0..255 (anti-aliased)
    hedge: Vec<u8>,
    depth: Vec<u8>,
    rock: Vec<u8>,
    snow: Vec<u8>,
}

const FIELDS_PER_CELL: f32 = 3.2;

pub fn tex_static(r: &Replay, g: &Ground) -> TexStatic {
    let (tw, th) = (r.w * TS, r.h * TS);
    let rows: Vec<_> = (0..th)
        .into_par_iter()
        .map(|ty| {
            let mut cell = Vec::with_capacity(tw);
            let mut sub = Vec::with_capacity(tw);
            let mut field = Vec::with_capacity(tw);
            let mut hedge = Vec::with_capacity(tw);
            let mut kind = Vec::with_capacity(tw);
            let mut depth = Vec::with_capacity(tw);
            let mut rock = Vec::with_capacity(tw);
            let mut snow = Vec::with_capacity(tw);
            let fy = (ty as f32 + 0.5) / TS as f32;
            for tx in 0..tw {
                let fx = (tx as f32 + 0.5) / TS as f32;
                let (wx, wy) = world::warp(fx, fy);
                let cx = (wx.floor() as isize).clamp(0, r.w as isize - 1) as usize;
                let cy = (wy.floor() as isize).clamp(0, r.h as isize - 1) as usize;
                cell.push((cy * r.w + cx) as u16);
                sub.push((world::sub_uniform(g, fx, fy) * 255.0) as u8);
                let (fi, fj) = ((wx * FIELDS_PER_CELL).floor() as i32, (wy * FIELDS_PER_CELL).floor() as i32);
                field.push((world::hash2(fi, fj, 41) * 255.0) as u8);
                // distance to the nearest field edge, in cells, gives smooth hedge lines
                let (gx, gy) = (wx * FIELDS_PER_CELL, wy * FIELDS_PER_CELL);
                let (ux, uy) = (gx - gx.floor(), gy - gy.floor());
                let d = ux.min(1.0 - ux).min(uy).min(1.0 - uy) / FIELDS_PER_CELL;
                let texel = 1.0 / TS as f32;
                hedge.push((((0.017 - d) / texel + 0.5).clamp(0.0, 1.0) * 255.0) as u8);
                let h = world::height_at(g, fx, fy);
                let k = if h < -0.004 { 2u8 } else if h < 0.03 { 1 } else { 0 };
                kind.push(k);
                depth.push(((-h).clamp(0.0, 1.0) * 255.0) as u8);
                let e = 0.02;
                let sx = world::height_at(g, fx + e, fy) - world::height_at(g, fx - e, fy);
                let sy = world::height_at(g, fx, fy + e) - world::height_at(g, fx, fy - e);
                let slope = (sx * sx + sy * sy).sqrt() / (2.0 * e);
                rock.push((((slope - 1.1) / 1.2).clamp(0.0, 1.0) * 255.0) as u8);
                snow.push((((h - 1.75) / 0.45).clamp(0.0, 1.0) * 255.0) as u8);
            }
            (cell, sub, field, hedge, kind, depth, rock, snow)
        })
        .collect();
    let mut s = TexStatic {
        tw,
        th,
        cell: Vec::with_capacity(tw * th),
        sub: Vec::with_capacity(tw * th),
        field: Vec::with_capacity(tw * th),
        flags: vec![0; tw * th],
        hedge: Vec::with_capacity(tw * th),
        depth: Vec::with_capacity(tw * th),
        rock: Vec::with_capacity(tw * th),
        snow: Vec::with_capacity(tw * th),
    };
    let mut kinds: Vec<u8> = Vec::with_capacity(tw * th);
    for (cell, sub, field, hedge, kind, depth, rock, snow) in rows {
        s.cell.extend(cell);
        s.sub.extend(sub);
        s.field.extend(field);
        s.hedge.extend(hedge);
        kinds.extend(kind);
        s.depth.extend(depth);
        s.rock.extend(rock);
        s.snow.extend(snow);
    }
    for p in 0..tw * th {
        s.flags[p] = kinds[p] << 4;
    }
    s
}

type Rgb = [f32; 3];

fn c(hex: u32) -> Rgb {
    [((hex >> 16) & 255) as f32, ((hex >> 8) & 255) as f32, (hex & 255) as f32]
}

fn mixc(a: Rgb, b: Rgb, t: f32) -> Rgb {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

struct Palette {
    crops: [Rgb; 10],
    meadows: [Rgb; 4],
    hedge: Rgb,
    wood: Rgb,
    heath: [Rgb; 4],
    quarry: Rgb,
    clay: Rgb,
    iron: Rgb,
    coal: Rgb,
    works: Rgb,
    urban: Rgb,
    derelict: Rgb,
    beach: Rgb,
    shallow: Rgb,
    mid: Rgb,
    deep: Rgb,
    rock: Rgb,
    snow: Rgb,
}

fn palette() -> Palette {
    let (wheat, barley, oats, green, plough, hay) = (c(0xe3c25e), c(0xebd891), c(0xd8c779), c(0x9cc05a), c(0xb08455), c(0xcdbb69));
    Palette {
        crops: [wheat, wheat, barley, barley, oats, green, green, hay, plough, wheat],
        meadows: [c(0x8fc15a), c(0x9bcb63), c(0x86b851), c(0xa6cf71)],
        hedge: c(0x4a7533),
        wood: c(0x4b7738),
        heath: [c(0xbcbb7e), c(0xbcbb7e), c(0xaead73), c(0xa39d8f)],
        quarry: c(0xc9c3b7),
        clay: c(0xc69466),
        iron: c(0xa76c4c),
        coal: c(0x4a4643),
        works: c(0x8f857a),
        urban: c(0xcbb9a1),
        derelict: c(0x968f81),
        beach: c(0xead9a3),
        shallow: c(0xd2cd9c),
        mid: c(0x6fa8a3),
        deep: c(0x2f5f78),
        rock: c(0x9b9588),
        snow: c(0xf5f5f1),
    }
}

/// Paint year k into an RGBA8 sRGB buffer of tw x th texels.
pub fn paint(r: &Replay, ts: &TexStatic, k: usize, out: &mut [u8]) {
    let pal = palette();
    let tw = ts.tw;
    out.par_chunks_mut(tw * 4).enumerate().for_each(|(ty, row)| {
        for tx in 0..tw {
            let p = ty * tw + tx;
            let kind = ts.flags[p] >> 4;
            let n = 0.97 + 0.06 * world::hash2(tx as i32, ty as i32, 77);
            let col: Rgb = match kind {
                2 => {
                    let d = ts.depth[p] as f32 / 255.0;
                    if d < 0.12 { mixc(pal.shallow, pal.mid, d / 0.12) } else { mixc(pal.mid, pal.deep, ((d - 0.12) / 0.6).min(1.0)) }
                }
                1 => pal.beach,
                _ => {
                    let c0 = ts.cell[p] as usize;
                    let cls = world::pick_class(r, k, c0, ts.sub[p] as f32);
                    let f = ts.field[p] as usize;
                    let hedge = ts.hedge[p] as f32 / 255.0;
                    let mut col = match cls {
                        world::FARM => mixc(pal.crops[f % 10], pal.hedge, hedge),
                        world::PASTURE => mixc(pal.meadows[f % 4], pal.hedge, hedge),
                        world::WOOD => mixc(pal.wood, c(0x3f6a30), (world::hash2(tx as i32 >> 1, ty as i32 >> 1, 9) > 0.6) as u8 as f32),
                        world::HEATH => {
                            let base = pal.heath[r.terrain[c0] as usize % 4];
                            let spot = world::hash2(tx as i32 >> 1, ty as i32 >> 1, 17);
                            if spot > 0.82 { mixc(base, c(0xa58fa6), 0.55) } else if spot < 0.1 { mixc(base, c(0x7d8a45), 0.5) } else { base }
                        }
                        world::QUARRY => pal.quarry,
                        world::CLAYPIT => pal.clay,
                        world::IRONMINE => pal.iron,
                        world::COALMINE => pal.coal,
                        world::WORKS => pal.works,
                        world::URBAN => {
                            if (tx % 5 == 0) || (ty % 5 == 0) { mixc(pal.urban, c(0xe6dccb), 0.5) } else { pal.urban }
                        }
                        world::DERELICT => pal.derelict,
                        _ => pal.heath[1],
                    };
                    if cls != world::URBAN && cls != world::WORKS {
                        let rk = ts.rock[p] as f32 / 255.0;
                        if rk > 0.0 { col = mixc(col, pal.rock, rk); }
                        let sn = ts.snow[p] as f32 / 255.0;
                        if sn > 0.0 { col = mixc(col, pal.snow, sn); }
                    }
                    col
                }
            };
            let o = tx * 4;
            row[o] = (col[0] * n).min(255.0) as u8;
            row[o + 1] = (col[1] * n).min(255.0) as u8;
            row[o + 2] = (col[2] * n).min(255.0) as u8;
            row[o + 3] = 255;
        }
    });
}

/// Append box-filtered mip levels to an RGBA8 image. Returns (data, level count).
pub fn with_mips(w: usize, h: usize, base: Vec<u8>) -> (Vec<u8>, u32) {
    let mut all = base;
    let (mut cw, mut ch) = (w, h);
    let mut start = 0usize;
    let mut levels = 1u32;
    while cw > 1 || ch > 1 {
        let (nw, nh) = ((cw / 2).max(1), (ch / 2).max(1));
        let mut next = vec![0u8; nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                for k in 0..4 {
                    let mut s = 0u32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let sx = (x * 2 + dx).min(cw - 1);
                        let sy = (y * 2 + dy).min(ch - 1);
                        s += all[start + (sy * cw + sx) * 4 + k] as u32;
                    }
                    next[(y * nw + x) * 4 + k] = (s / 4) as u8;
                }
            }
        }
        start = all.len();
        all.extend_from_slice(&next);
        cw = nw;
        ch = nh;
        levels += 1;
    }
    (all, levels)
}

pub fn make_image(w: usize, h: usize, rgba: Vec<u8>) -> Image {
    let size = Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 };
    let level0 = rgba[..w * h * 4].to_vec();
    let mut img = Image::new(size, TextureDimension::D2, level0, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    let (all, levels) = with_mips(w, h, rgba);
    img.data = Some(all);
    img.texture_descriptor.mip_level_count = levels;
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 16,
        ..Default::default()
    });
    img
}

/// Replace an image's pixels (and mips) in place.
pub fn update_image(img: &mut Image, w: usize, h: usize, rgba: Vec<u8>) {
    let (all, levels) = with_mips(w, h, rgba);
    img.data = Some(all);
    img.texture_descriptor.mip_level_count = levels;
}
