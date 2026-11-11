// The island's ground, worked out per pixel so fields and hedges stay crisp at
// any zoom, while Bevy's lighting, shadows, ambient occlusion and fog work as
// for any StandardMaterial.
//
// Each 2 km cell has a mix of two land classes for the year: mix_a, mix_b and
// mix_s, the share of mix_a in 255ths. The land is divided into fields by the
// field system of fields.rs, and every parcel of fields takes one class by the
// rule of world.rs: the same integer hash, value noise, warp and quantile table,
// looked up at the parcel's own point, with the lower class number always taking
// the low noise values. The trees and houses that scatter.rs places with that
// rule therefore stand on matching ground, and land classes change only at
// hedge lines.
//
// Colours are mixed in sRGB, as the old CPU painter did, and turned linear at
// the end.

import bevy_pbr::render::{
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
};

// terrain.rs PatchworkParams
struct PatchworkParams {
    // phase of the year: 0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn
    season: f32,
    // map size in cells
    width: f32,
    height: f32,
    // 1 draws flat, unlit class colours, to check the patchwork against the CPU
    debug: u32,
    // the calendar year shown, for the open fields
    year: f32,
    // how strongly the map overlay is washed over the land: 0 leaves the
    // ground exactly as it is without one
    overlay: f32,
    // how far the overlay's tints run into each other across the edge of a
    // square, in squares: 0.5 is a smooth wash, a small value keeps flat areas
    overlay_soft: f32,
    // 1 draws an ink line where the overlay's regions meet
    overlay_lines: f32,
    // the first block of the parcel textures along each axis (world.rs Land)
    block_i0: i32,
    block_j0: i32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> ground: PatchworkParams;
// per parcel (eight texels in a row for each block of the field system), this
// year: its class, how wooded it is (128 and up inside a wood, deeper as it
// rises; below that, other open land as it comes near to being wood), and the
// town density of its cell
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var year_parcels: texture_2d<f32>;
// per parcel, fixed: how many times it is cut into fields (the low four bits)
// and its terrain class (0 sea, 1 lowland, 2 hills, 3 mountains; the high
// four); the year its open fields are enclosed, less 1750 (0 if they never
// were open); how much of its arable is under grass; the tone of its soil
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var fixed_parcels: texture_2d<f32>;
// per cell, the map overlay (overlay.rs): a tint, and in alpha the region the
// cell belongs to, 0 for none (the sea, which is left alone)
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var overlay_cells: texture_2d<f32>;

// The land's colours, from the look file (look.rs Land, in the same order), in
// sRGB as they are mixed here.
struct LandPaint {
    soil: vec4<f32>,
    soil_winter: vec4<f32>,
    stubble: vec4<f32>,
    corn_green: vec4<f32>,
    wheat_ripe: vec4<f32>,
    barley_ripe: vec4<f32>,
    oats_ripe: vec4<f32>,
    roots: vec4<f32>,
    roots_winter: vec4<f32>,
    hay: vec4<f32>,
    hay_spring: vec4<f32>,
    hay_tall: vec4<f32>,
    hay_drying: vec4<f32>,
    hay_winter: vec4<f32>,
    fallow_weeds: vec4<f32>,
    pasture: vec4<f32>,
    pasture_light: vec4<f32>,
    pasture_deep: vec4<f32>,
    pasture_pale: vec4<f32>,
    grass_dry: vec4<f32>,
    grass_autumn: vec4<f32>,
    grass_winter: vec4<f32>,
    balk: vec4<f32>,
    headland: vec4<f32>,
    water_meadow: vec4<f32>,
    rock: vec4<f32>,
    snow: vec4<f32>,
    frost: vec4<f32>,
    hedge: vec4<f32>,
    hedge_autumn: vec4<f32>,
    hedge_winter: vec4<f32>,
    field_wall: vec4<f32>,
    wood: vec4<f32>,
    wood_shade: vec4<f32>,
    wood_spring: vec4<f32>,
    wood_autumn: vec4<f32>,
    wood_winter: vec4<f32>,
    heath: vec4<f32>,
    heath_hill: vec4<f32>,
    heath_mountain: vec4<f32>,
    heath_brown: vec4<f32>,
    heather_drift: vec4<f32>,
    heather_drift_bloom: vec4<f32>,
    heather: vec4<f32>,
    heather_bloom: vec4<f32>,
    gorse: vec4<f32>,
    gorse_flower: vec4<f32>,
    town_earth: vec4<f32>,
    garden: vec4<f32>,
    garden_dug: vec4<f32>,
    cobbles: vec4<f32>,
    town_far: vec4<f32>,
    yard: vec4<f32>,
    yard_cinder: vec4<f32>,
    yard_gravel: vec4<f32>,
    yard_lane: vec4<f32>,
    derelict: vec4<f32>,
    derelict_weeds: vec4<f32>,
    quarry: vec4<f32>,
    clay_pit: vec4<f32>,
    iron_mine: vec4<f32>,
    colliery: vec4<f32>,
    seabed_sand: vec4<f32>,
    seabed_shelf: vec4<f32>,
    seabed_deep: vec4<f32>,
    beach_wet: vec4<f32>,
    beach: vec4<f32>,
    shingle: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(103) var<uniform> paint: LandPaint;

// display classes, as in world.rs
const SEA: u32 = 0u;
const FARM: u32 = 1u;
const PASTURE: u32 = 2u;
const WOOD: u32 = 3u;
const HEATH: u32 = 4u;
const QUARRY: u32 = 5u;
const CLAYPIT: u32 = 6u;
const IRONMINE: u32 = 7u;
const COALMINE: u32 = 8u;
const WORKS: u32 = 9u;
const URBAN: u32 = 10u;
const DERELICT: u32 = 11u;

// fields.rs
const BLOCK: f32 = 0.9;
const TURN_C: f32 = 0.9460423;
const TURN_S: f32 = 0.3240436;
const PARCEL_LEVEL: u32 = 3u;
// half the width of a hedge, in cells
const HEDGE: f32 = 0.0072;
const TAU: f32 = 6.2831853;
// towards the sun, on the map: it stands low in the south-west
const SUN_XZ: vec2<f32> = vec2<f32>(-0.682, 0.731);

// ------------------------------------------------------------------ the CPU rule
// hash2, hash3, vnoise, warp_raw, sub_raw and the field system are world.rs
// and fields.rs line for line; keep them in step. WGSL u32 arithmetic wraps like
// Rust's wrapping_mul, so the hashes match bit for bit.

fn hash2(x: i32, y: i32, s: i32) -> f32 {
    var h = (bitcast<u32>(x) * 374761393u) ^ (bitcast<u32>(y) * 668265263u) ^ (bitcast<u32>(s) * 1442695041u);
    h = (h ^ (h >> 13u)) * 1274126177u;
    h = h ^ (h >> 16u);
    return f32(h) / 4294967295.0;
}

fn hash3(a: u32, b: u32, c: u32) -> f32 {
    var h = (a * 374761393u) ^ (b * 668265263u) ^ (c * 1442695041u);
    h = (h ^ (h >> 13u)) * 1274126177u;
    h = h ^ (h >> 16u);
    h = (h ^ (h >> 15u)) * 2246822519u;
    h = h ^ (h >> 13u);
    return f32(h) / 4294967295.0;
}

fn vnoise(x: f32, y: f32, s: i32) -> f32 {
    let fx = floor(x);
    let fy = floor(y);
    let tx = x - fx;
    let ty = y - fy;
    let ix = i32(fx);
    let iy = i32(fy);
    let sx = tx * tx * (3.0 - 2.0 * tx);
    let sy = ty * ty * (3.0 - 2.0 * ty);
    let a = hash2(ix, iy, s);
    let b = hash2(ix + 1, iy, s);
    let c = hash2(ix, iy + 1, s);
    let d = hash2(ix + 1, iy + 1, s);
    return (a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy) * 2.0 - 1.0;
}

// --- the field system (fields.rs)

fn to_block(p: vec2<f32>) -> vec2<f32> {
    let q = vec2<f32>(TURN_C * p.x + TURN_S * p.y, -TURN_S * p.x + TURN_C * p.y) / BLOCK;
    let a = 0.13 * smoothstep(-0.3, 0.5, vnoise(q.x * 0.21 + 3.7, q.y * 0.21 + 1.3, 141));
    let wx = vnoise(q.x * 1.1, q.y * 1.1, 143);
    let wy = vnoise(q.x * 1.1 + 7.1, q.y * 1.1 + 2.9, 147);
    return vec2<f32>(q.x + a * wx, q.y + a * wy);
}

fn from_block(q: vec2<f32>) -> vec2<f32> {
    let s = q * BLOCK;
    return vec2<f32>(TURN_C * s.x - TURN_S * s.y, TURN_S * s.x + TURN_C * s.y);
}

fn corner(i: i32, j: i32) -> vec2<f32> {
    return vec2<f32>(f32(i) + 0.5 * (hash2(i, j, 151) - 0.5), f32(j) + 0.5 * (hash2(i, j, 153) - 0.5));
}

fn cross2(a: vec2<f32>, b: vec2<f32>) -> f32 {
    return a.x * b.y - a.y * b.x;
}

fn inv_bilinear(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, d: vec2<f32>) -> vec2<f32> {
    let e = b - a;
    let f = d - a;
    let g = a - b + c - d;
    let h = p - a;
    let k2 = cross2(g, f);
    let k1 = cross2(e, f) + cross2(h, g);
    let k0 = cross2(h, e);
    if (abs(k2) < 1e-4) {
        let v = -k0 / k1;
        let u = (h.x - f.x * v) / (e.x + g.x * v);
        return vec2<f32>(u, v);
    }
    let w = sqrt(max(k1 * k1 - 4.0 * k0 * k2, 0.0));
    let ik2 = 0.5 / k2;
    var v = (-k1 - w) * ik2;
    var u = (h.x - f.x * v) / (e.x + g.x * v);
    if (u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0) {
        v = (-k1 + w) * ik2;
        u = (h.x - f.x * v) / (e.x + g.x * v);
    }
    return vec2<f32>(u, v);
}

fn bilinear(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, d: vec2<f32>, uv: vec2<f32>) -> vec2<f32> {
    return a + (b - a) * uv.x + (d - a) * uv.y + (a - b + c - d) * (uv.x * uv.y);
}

struct Block {
    i: i32,
    j: i32,
    uv: vec2<f32>,
    a: vec2<f32>,
    b: vec2<f32>,
    c: vec2<f32>,
    d: vec2<f32>,
}

fn locate(q: vec2<f32>) -> Block {
    var i = i32(floor(q.x));
    var j = i32(floor(q.y));
    var out: Block;
    for (var it = 0; it < 4; it++) {
        let a = corner(i, j);
        let b = corner(i + 1, j);
        let c = corner(i + 1, j + 1);
        let d = corner(i, j + 1);
        let uv = inv_bilinear(q, a, b, c, d);
        out = Block(i, j, uv, a, b, c, d);
        var di = 0;
        var dj = 0;
        if (uv.x < 0.0) { di = -1; } else if (uv.x >= 1.0) { di = 1; }
        if (uv.y < 0.0) { dj = -1; } else if (uv.y >= 1.0) { dj = 1; }
        if (di == 0 && dj == 0) {
            break;
        }
        i += di;
        j += dj;
    }
    out.uv = clamp(out.uv, vec2<f32>(0.0), vec2<f32>(0.99999));
    return out;
}

struct Cut {
    lo: vec2<f32>,
    hi: vec2<f32>,
    id: u32,
}

fn cut(bl: Block, start: Cut, level: u32, depth: u32) -> Cut {
    let lu = 0.5 * (length(bl.b - bl.a) + length(bl.c - bl.d));
    let lv = 0.5 * (length(bl.d - bl.a) + length(bl.c - bl.b));
    let bi = bitcast<u32>(bl.i);
    let bj = bitcast<u32>(bl.j);
    let strips = hash3(bi, bj, 157u) < 0.25;
    let strip_u = hash3(bi, bj, 158u) < 0.5;
    var c = start;
    var lvl = level;
    loop {
        if (lvl >= depth) {
            break;
        }
        let ex = (c.hi.x - c.lo.x) * lu;
        let ey = (c.hi.y - c.lo.y) * lv;
        let k = hash3(bi, bj + c.id * 7919u, 160u + lvl);
        var along_u = ex >= ey;
        if (strips && lvl >= PARCEL_LEVEL) {
            along_u = strip_u;
        } else if (k < 0.12) {
            along_u = !along_u;
        }
        let t = 0.36 + 0.28 * hash3(bi + c.id * 104729u, bj, 170u + lvl);
        if (along_u) {
            let s = c.lo.x + (c.hi.x - c.lo.x) * t;
            if (bl.uv.x < s) {
                c.hi.x = s;
                c.id = c.id * 2u;
            } else {
                c.lo.x = s;
                c.id = c.id * 2u + 1u;
            }
        } else {
            let s = c.lo.y + (c.hi.y - c.lo.y) * t;
            if (bl.uv.y < s) {
                c.hi.y = s;
                c.id = c.id * 2u;
            } else {
                c.lo.y = s;
                c.id = c.id * 2u + 1u;
            }
        }
        lvl += 1u;
    }
    return c;
}

// How far the block's point lies from the nearest side of c, in cells.
fn cut_edge(bl: Block, c: Cut) -> f32 {
    let g = bl.a - bl.b + bl.c - bl.d;
    let ju = (bl.b - bl.a) + g * bl.uv.y;
    let jv = (bl.d - bl.a) + g * bl.uv.x;
    let det = max(abs(cross2(ju, jv)), 1e-6);
    let eu = min(bl.uv.x - c.lo.x, c.hi.x - bl.uv.x) * det / max(length(jv), 1e-6);
    let ev = min(bl.uv.y - c.lo.y, c.hi.y - bl.uv.y) * det / max(length(ju), 1e-6);
    return min(eu, ev) * BLOCK;
}

fn field_key(bl: Block, c: Cut) -> u32 {
    return (bitcast<u32>(bl.i) * 73856093u) ^ (bitcast<u32>(bl.j) * 19349663u) ^ (c.id * 83492791u);
}

// ------------------------------------------------------------------ helpers

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

// A one-dimensional value noise in 0..1 along t, different for every line k.
fn noise1(t: f32, k: i32, s: i32) -> f32 {
    let i = floor(t);
    let f = t - i;
    let a = hash2(i32(i), k, s);
    let b = hash2(i32(i) + 1, k, s);
    return mix(a, b, f * f * (3.0 - 2.0 * f));
}

// Box-filtered cover of lines of width lw centred on the integers, seen through
// a pixel that spans fw units: exact at any zoom, so thin lines neither shimmer
// nor vanish, and far away they settle on their average share of the ground.
fn lines(x: f32, fw: f32, lw: f32) -> f32 {
    let w = max(fw, 1e-4);
    let x0 = x - floor(x);
    let a = x0 + 0.5 * lw + 0.5 * w;
    let b = x0 + 0.5 * lw - 0.5 * w;
    let ia = floor(a) * lw + min(fract(a), lw);
    let ib = floor(b) * lw + min(fract(b), lw);
    return clamp((ia - ib) / w, 0.0, 1.0);
}

// Cover of a band from distance 0 to w, for a spot at distance d from its edge,
// seen through a pixel of size fw: crisp up close, its average share far away.
fn band(d: f32, w: f32, fw: f32) -> f32 {
    let f = max(fw, 1e-5);
    return clamp((w - d) / f + 0.5, 0.0, 1.0);
}

// Cover of the spots where a value noise n is above thr. Edges are softened
// over nf, the noise's change across a pixel, and when the spots shrink below a
// pixel (detail falling to 0) the cover settles on its average share `area`.
fn spots(n: f32, thr: f32, nf: f32, d: f32, area: f32) -> f32 {
    return mix(area, smoothstep(thr - nf, thr + nf, n), d);
}

// 1 where a pattern of `freq` repeats per cell is comfortably larger than a
// pixel, falling to 0 as it shrinks below one.
fn detail(freq: f32, foot: f32) -> f32 {
    return 1.0 - smoothstep(0.35, 0.7, freq * foot);
}

// 1 at phase c, falling to 0 at distance w either side, round the year.
fn bell(ph: f32, c: f32, w: f32) -> f32 {
    let d = abs(fract(ph - c + 0.5) - 0.5);
    return 1.0 - smoothstep(0.0, w, d);
}

// 1 in midwinter, gone by early March and from early November.
fn winter(ph: f32) -> f32 {
    return 1.0 - smoothstep(0.06, 0.2, min(ph, 1.0 - ph));
}

// ------------------------------------------------------------------ what a spot knows

struct Spot {
    // map position in cells
    p: vec2<f32>,
    // cells per pixel
    foot: f32,
    // phase of the year
    ph: f32,
    // terrain class of the cell and its town density, 0..1
    terr: u32,
    dense: f32,
    // rock on steep slopes, snow on high ground, winter frost: 0..1
    rock: f32,
    snow: f32,
    frost: f32,
    // on a river's valley floor: 0..1
    flood: f32,
    // the field: its number, its block and cut, and the spot's distance to the
    // field's nearest edge (cells) and which way that edge lies on the map
    key: u32,
    bl: Block,
    leaf: Cut,
    edge: f32,
    toward: vec2<f32>,
    // the field's narrower side, in cells
    size: f32,
    // an open field, not yet enclosed: strips and no hedges
    open: bool,
}

// ------------------------------------------------------------------ fields

struct Look {
    col: vec3<f32>,
    // how strongly rows, furrows or windrows show, as a share of the brightness
    rows: f32,
}

// A field of `crop` (the ten-way choice of the old painter: wheat, wheat,
// barley, barley, oats, green crop, green crop, hay, fallow, wheat) at phase ph
// of its own year; k is the field's own random number.
fn crop_look(crop: u32, ph: f32, k: f32) -> Look {
    let soil = mix(paint.soil.rgb, paint.soil_winter.rgb, 0.6 * winter(ph));
    let stubble = paint.stubble.rgb;
    var canopy = paint.corn_green.rgb;
    var cover = 0.0;
    var stub = 0.0;
    var rows = 0.0;
    switch crop {
        case 0u, 1u, 9u: {
            // winter wheat: green shoots through the winter, gold in July,
            // stubble in August, ploughed and sown again in October
            let harvest = 0.61 + 0.05 * k;
            let plough = 0.75 + 0.07 * k;
            if (ph < harvest) {
                let cut = smoothstep(harvest - 0.015, harvest, ph);
                cover = mix(0.3, 1.0, smoothstep(0.1, 0.36, ph)) * (1.0 - cut);
                stub = cut;
                canopy = mix(canopy, paint.wheat_ripe.rgb, smoothstep(0.42, 0.53, ph));
            } else {
                // stubble, then the plough, then next year's shoots
                stub = 1.0 - smoothstep(plough, plough + 0.03, ph);
                cover = 0.3 * smoothstep(plough + 0.05, plough + 0.15, ph);
            }
            rows = mix(0.07, 0.02, cover);
        }
        case 2u, 3u, 4u: {
            // barley and oats: sown in spring, pale gold in high summer, the
            // stubble left until late autumn
            let sow = 0.24 + 0.04 * k;
            let harvest = 0.64 + 0.05 * k;
            let plough = 0.84 + 0.1 * k;
            if (ph >= sow && ph < harvest) {
                let cut = smoothstep(harvest - 0.015, harvest, ph);
                cover = smoothstep(sow + 0.03, sow + 0.17, ph) * (1.0 - cut);
                stub = cut;
            } else if (ph >= harvest) {
                stub = 1.0 - smoothstep(plough, plough + 0.03, ph);
            }
            let ripe = select(paint.barley_ripe.rgb, paint.oats_ripe.rgb, crop == 4u);
            canopy = mix(canopy, ripe, smoothstep(0.44, 0.54, ph));
            rows = mix(0.07, 0.02, cover);
        }
        case 5u, 6u: {
            // turnips and clover: green all year, darker and grazed in winter
            cover = 0.96 - 0.3 * winter(ph);
            canopy = mix(paint.roots.rgb, paint.roots_winter.rgb, winter(ph));
            rows = 0.045;
        }
        case 7u: {
            // hay meadow: fresh in spring, cut in early summer and left in
            // windrows to dry, green again by autumn
            let cut = 0.5 + 0.04 * k;
            cover = 1.0;
            var grass = mix(paint.hay.rgb, paint.hay_spring.rgb, bell(ph, 0.32, 0.12));
            grass = mix(grass, paint.hay_tall.rgb, smoothstep(0.38, 0.48, ph) * (1.0 - smoothstep(cut, cut + 0.01, ph)));
            let drying = smoothstep(cut, cut + 0.01, ph) * (1.0 - smoothstep(cut + 0.08, cut + 0.2, ph));
            canopy = mix(grass, paint.hay_drying.rgb, drying);
            canopy = mix(canopy, paint.hay_winter.rgb, 0.5 * winter(ph));
            rows = 0.06 * drying;
        }
        default: {
            // fallow: bare furrows, a little green with weeds in summer
            cover = 0.25 * bell(ph, 0.58, 0.14);
            canopy = paint.fallow_weeds.rgb;
            rows = 0.07;
        }
    }
    let base = mix(soil, stubble, stub);
    return Look(mix(base, canopy, cover), rows);
}

// Pasture through the year: fresh in spring, a little dry late in summer, olive
// in autumn and dull in winter.
fn grass_season(c0: vec3<f32>, ph: f32) -> vec3<f32> {
    var c = c0;
    c = mix(c, c * vec3<f32>(1.02, 1.06, 0.92), bell(ph, 0.33, 0.12));
    c = mix(c, mix(c, paint.grass_dry.rgb, 0.3), bell(ph, 0.67, 0.08));
    c = mix(c, mix(c, paint.grass_autumn.rgb, 0.35), bell(ph, 0.84, 0.1));
    c = mix(c, mix(c, paint.grass_winter.rgb, 0.55), winter(ph));
    return c;
}

fn meadow_look(m: u32, ph: f32) -> Look {
    var c = paint.pasture.rgb;
    switch m {
        case 1u: { c = paint.pasture_light.rgb; }
        case 2u: { c = paint.pasture_deep.rgb; }
        case 3u: { c = paint.pasture_pale.rgb; }
        default: {}
    }
    return Look(grass_season(c, ph), 0.0);
}

// The look of field number `key`: its crop chosen by hash as the old painter
// did, its own calendar a week or so either way, and its own shade.
fn field_look(key: u32, pasture: bool, ph: f32) -> Look {
    let f = u32(hash3(key, 41u, 0u) * 255.0);
    let k = hash3(key, 53u, 0u);
    var look: Look;
    if (pasture) {
        look = meadow_look(f % 4u, ph);
    } else {
        look = crop_look(f % 10u, fract(ph + 0.06 * (k - 0.5)), k);
    }
    look.col *= 0.955 + 0.09 * fract(k * 7.31);
    return look;
}

// What a crowd of fields too small to see one by one averages out to.
fn fields_far(pasture: bool, ph: f32) -> vec3<f32> {
    if (pasture) {
        return 0.5 * (meadow_look(0u, ph).col + meadow_look(2u, ph).col);
    }
    return 0.35 * crop_look(0u, ph, 0.5).col + 0.25 * crop_look(2u, ph, 0.5).col
        + 0.2 * crop_look(5u, ph, 0.5).col + 0.1 * crop_look(7u, ph, 0.5).col + 0.1 * crop_look(8u, ph, 0.5).col;
}

// An open field before its parish is enclosed: each furlong ploughed in long
// strips, every strip its own crop, with ridge and furrow along them and a
// grass balk between, and a headland of grass at the furlong's edge instead of
// a hedge. Common pasture and meadow lie open.
fn open_field(sp: Spot, pasture: bool) -> vec3<f32> {
    let bl = sp.bl;
    let g = bl.a - bl.b + bl.c - bl.d;
    let ju = (bl.b - bl.a) + g * bl.uv.y;
    let jv = (bl.d - bl.a) + g * bl.uv.x;
    let det = abs(cross2(ju, jv));
    let su = det / max(length(jv), 1e-6) * BLOCK;
    let sv = det / max(length(ju), 1e-6) * BLOCK;
    var c: vec3<f32>;
    if (pasture) {
        c = meadow_look(u32(hash3(sp.key, 5u, 0u) * 4.0) % 4u, sp.ph).col;
        c *= 1.0 + 0.06 * vnoise(sp.p.x * 11.0, sp.p.y * 11.0, 83) * detail(11.0, sp.foot);
    } else {
        // across the strips, which run the long way of the furlong
        let long_u = (sp.leaf.hi.x - sp.leaf.lo.x) * su > (sp.leaf.hi.y - sp.leaf.lo.y) * sv;
        var x = (bl.uv.x - sp.leaf.lo.x) * su;
        if (long_u) {
            x = (bl.uv.y - sp.leaf.lo.y) * sv;
        }
        let strip = 0.026;
        let si = floor(x / strip);
        let skey = sp.key * 31u + bitcast<u32>(i32(si)) * 2654435761u;
        let look = field_look(skey, false, sp.ph);
        c = look.col;
        let fx = fract(x / strip);
        let ridges = cos(TAU * fx * 3.0) * (1.0 - smoothstep(0.25, 0.5, sp.foot * 3.0 / strip));
        c *= 1.0 + (0.05 + look.rows) * ridges;
        let balk = band(min(fx, 1.0 - fx) * strip, 0.0018, sp.foot);
        c = mix(c, paint.balk.rgb, 0.8 * balk);
        let lod = smoothstep(1.5, 4.0, strip / max(sp.foot, 1e-5));
        c = mix(fields_far(false, sp.ph), c, lod);
    }
    // the headland round the furlong
    let headland = band(sp.edge, 0.005, sp.foot);
    c = mix(c, grass_season(paint.headland.rgb, sp.ph), 0.85 * headland);
    c = mix(c, paint.rock.rgb, sp.rock);
    c = mix(c, paint.snow.rgb, sp.snow);
    return c;
}

// A field of farm or pasture, with rows along one of its sides, and its hedges
// (stone walls on the hills), each standing a little proud with a painted
// shadow on the side away from the sun.
fn fields(sp: Spot, pasture: bool) -> vec3<f32> {
    if (sp.open) {
        return open_field(sp, pasture);
    }
    let look = field_look(sp.key, pasture, sp.ph);
    var c = look.col;
    // rows across the field, along one side, fading before they are finer than a pixel
    let r = hash3(sp.key, 47u, 0u);
    let bl = sp.bl;
    let g = bl.a - bl.b + bl.c - bl.d;
    let ju = (bl.b - bl.a) + g * bl.uv.y;
    let jv = (bl.d - bl.a) + g * bl.uv.x;
    let det = abs(cross2(ju, jv));
    var x: f32;
    if (r < 0.5) {
        x = (bl.uv.x - sp.leaf.lo.x) * det / max(length(jv), 1e-6) * BLOCK;
    } else {
        x = (bl.uv.y - sp.leaf.lo.y) * det / max(length(ju), 1e-6) * BLOCK;
    }
    let spacing = 0.016 + 0.006 * fract(r * 13.0);
    let wave = cos(TAU * x / spacing) * (1.0 - smoothstep(0.25, 0.5, sp.foot / spacing));
    c *= 1.0 + look.rows * wave;
    // far away the fields blend into their average
    let lod = smoothstep(1.5, 4.0, sp.size / max(sp.foot, 1e-5));
    c = mix(fields_far(pasture, sp.ph) * 1.0, c, lod);
    if (pasture) {
        // tussocks and wetter hollows
        c *= 1.0 + 0.06 * vnoise(sp.p.x * 11.0, sp.p.y * 11.0, 83) * detail(11.0, sp.foot);
    }
    // water meadows on the valley floors: greener, with darker rushes
    c = mix(c, paint.water_meadow.rgb * (1.0 + 0.08 * vnoise(sp.p.x * 9.0, sp.p.y * 9.0, 87)), 0.5 * sp.flood);
    c = mix(c, paint.rock.rgb, sp.rock);
    c = mix(c, paint.snow.rgb, sp.snow);
    // the hedge or wall: uneven along its length and bushy up close
    let along = dot(sp.p, vec2<f32>(-sp.toward.y, sp.toward.x));
    let wall = sp.terr >= 3u || (sp.terr == 2u && hash3(bitcast<u32>(bl.i), bitcast<u32>(bl.j), 181u) < 0.65);
    let bushy = 0.3 * detail(140.0, sp.foot);
    var hw = HEDGE * (0.8 + 0.4 * noise1(along * 9.0, i32(sp.key & 1023u), 71) + bushy * (noise1(along * 40.0, i32(sp.key & 1023u), 75) - 0.5));
    if (wall) {
        hw = HEDGE * 0.55;
    }
    // the shadow falls on the side away from the sun
    let shade_side = smoothstep(0.0, 0.35, dot(sp.toward, SUN_XZ));
    let sw = 1.2 * hw;
    let shadow = band(sp.edge, hw + sw, sp.foot) * shade_side;
    c *= 1.0 - 0.18 * shadow;
    let cover = band(sp.edge, hw, sp.foot);
    var hc = paint.hedge.rgb;
    hc = mix(hc, paint.hedge_autumn.rgb, 0.6 * bell(sp.ph, 0.84, 0.1));
    hc = mix(hc, paint.hedge_winter.rgb, 0.6 * winter(sp.ph));
    // darker and lighter bushes along the hedge, which shows through snow
    hc *= 1.0 + 0.14 * vnoise(sp.p.x * 60.0, sp.p.y * 60.0, 77) * detail(60.0, sp.foot);
    if (wall) {
        hc = paint.field_wall.rgb * (1.0 + 0.1 * vnoise(along * 80.0, 3.0, 79) * detail(80.0, sp.foot));
    }
    hc = mix(hc, paint.snow.rgb, 0.35 * sp.snow);
    return mix(c, hc, 0.92 * cover);
}

// ------------------------------------------------------------------ other ground

fn wood_floor(sp: Spot) -> vec3<f32> {
    let n = vnoise(sp.p.x * 12.0, sp.p.y * 12.0, 81);
    let shade = spots(n, 0.125, 12.0 * sp.foot, detail(12.0, sp.foot), 0.4);
    var c = mix(paint.wood.rgb, paint.wood_shade.rgb, shade);
    c = mix(c, paint.wood_spring.rgb, 0.35 * bell(sp.ph, 0.33, 0.12));
    c = mix(c, paint.wood_autumn.rgb, 0.45 * bell(sp.ph, 0.83, 0.12));
    c = mix(c, paint.wood_winter.rgb, 0.5 * winter(sp.ph));
    return c;
}

fn heath(sp: Spot) -> vec3<f32> {
    var base = paint.heath.rgb;
    if (sp.terr == 2u) {
        base = paint.heath_hill.rgb;
    } else if (sp.terr == 3u) {
        base = paint.heath_mountain.rgb;
    }
    // heather flowers in late summer, gorse in spring; brown in autumn and winter
    let bloom = smoothstep(0.52, 0.6, sp.ph) * (1.0 - smoothstep(0.72, 0.8, sp.ph));
    let flower = bell(sp.ph, 0.34, 0.09);
    // soft drifts where heather grows thicker, a few hundred metres across
    let drift = smoothstep(-0.2, 0.6, vnoise(sp.p.x * 6.0 + 4.1, sp.p.y * 6.0 + 2.3, 19));
    base = mix(base, mix(paint.heather_drift.rgb, paint.heather_drift_bloom.rgb, bloom), (0.22 + 0.14 * bloom) * drift);
    base = mix(base, paint.heath_brown.rgb, 0.3 * max(winter(sp.ph), bell(sp.ph, 0.86, 0.1)));
    // and small clumps of heather and gorse
    let n = vnoise(sp.p.x * 34.0, sp.p.y * 34.0, 17);
    let nf = 34.0 * sp.foot;
    let d = detail(34.0, sp.foot);
    let heather = spots(n, 0.433, nf, d, 0.18);
    let gorse = spots(-n, 0.58, nf, d, 0.1);
    var c = mix(base, mix(paint.heather.rgb, paint.heather_bloom.rgb, bloom), heather * mix(0.4, 0.6, bloom));
    c = mix(c, mix(paint.gorse.rgb, paint.gorse_flower.rgb, flower), gorse * mix(0.4, 0.7, flower));
    return c;
}

// Light mottling for bare ground.
fn mottled(base: vec3<f32>, sp: Spot, amount: f32) -> vec3<f32> {
    let n = vnoise(sp.p.x * 9.0, sp.p.y * 9.0, 95);
    return base * (1.0 + amount * n * detail(9.0, sp.foot));
}

// Plain town ground, with no streets of its own: the parcel cut a little
// further into plots of beaten earth, cobbles and gardens, fewer gardens in a
// crowded town, and a thin line of wall or fence between the plots.
fn town(sp: Spot) -> vec3<f32> {
    // two more cuts than the fields round the town
    let plot = cut(sp.bl, sp.leaf, 7u, 9u);
    let k = hash3(field_key(sp.bl, plot), 91u, 0u);
    let garden_share = max(0.42 - 0.38 * sp.dense, 0.04);
    var c = paint.town_earth.rgb;
    if (k < garden_share) {
        // a garden: beds of green and dug earth
        let rows = 0.5 + 0.5 * cos(TAU * dot(sp.p, vec2<f32>(0.6, 0.8)) / 0.011);
        c = mix(paint.garden.rgb, paint.garden_dug.rgb, rows * detail(90.0, sp.foot));
    } else if (k < garden_share + 0.3) {
        // cobbles
        c = paint.cobbles.rgb * (1.0 + 0.06 * vnoise(sp.p.x * 120.0, sp.p.y * 120.0, 97) * detail(120.0, sp.foot));
    }
    // far away the plots blend into the town's average ground
    let lod = detail(18.0, sp.foot);
    let avg = mix(paint.town_far.rgb, paint.garden.rgb, garden_share * 0.8);
    c = mix(avg, c, lod);
    let edge = band(cut_edge(sp.bl, plot), 0.0016, sp.foot);
    c = mix(c, c * 0.84, edge * lod);
    return mottled(c, sp, 0.03);
}

// Works stand in yards of cinder and gravel with lanes between them.
fn works_yard(sp: Spot) -> vec3<f32> {
    let q = sp.p * 3.5;
    let qf = 3.5 * sp.foot;
    let id = vec2<i32>(floor(q));
    let b = hash2(id.x, id.y, 93);
    let base = paint.yard.rgb;
    let cinder = paint.yard_cinder.rgb;
    let gravel = paint.yard_gravel.rgb;
    var yard = base;
    if (b < 0.3) {
        yard = cinder;
    } else if (b > 0.72) {
        yard = gravel;
    }
    let average = base * 0.42 + cinder * 0.3 + gravel * 0.28;
    var c = mix(average, yard, detail(10.0, sp.foot));
    let lane = 1.0 - (1.0 - lines(q.x, qf, 0.07)) * (1.0 - lines(q.y, qf, 0.07));
    c = mix(c, paint.yard_lane.rgb, 0.8 * lane);
    return mottled(c, sp, 0.05);
}

fn derelict(sp: Spot) -> vec3<f32> {
    let n = vnoise(sp.p.x * 7.0, sp.p.y * 7.0, 97);
    let weeds = spots(n, 0.19, 7.0 * sp.foot, detail(7.0, sp.foot), 0.35);
    return mix(paint.derelict.rgb, paint.derelict_weeds.rgb, 0.6 * weeds);
}

fn class_colour(cls: u32, sp: Spot) -> vec3<f32> {
    var c: vec3<f32>;
    switch cls {
        case FARM: { c = fields(sp, false); }
        case PASTURE: { c = fields(sp, true); }
        case WOOD: { c = wood_floor(sp); }
        case HEATH: { c = heath(sp); }
        case QUARRY: { c = mottled(paint.quarry.rgb, sp, 0.06); }
        case CLAYPIT: { c = mottled(paint.clay_pit.rgb, sp, 0.08); }
        case IRONMINE: { c = mottled(paint.iron_mine.rgb, sp, 0.08); }
        case COALMINE: { c = mottled(paint.colliery.rgb, sp, 0.12); }
        case WORKS: { c = works_yard(sp); }
        case URBAN: { c = town(sp); }
        case DERELICT: { c = derelict(sp); }
        // land that lies in a sea cell: rough coastal grass
        default: { c = paint.heath.rgb; }
    }
    var frost = sp.frost;
    if (cls == URBAN || cls == WORKS) {
        frost *= 0.6;
    } else if (cls != FARM && cls != PASTURE) {
        // fields() lays its own rock and snow under the hedges
        c = mix(c, paint.rock.rgb, sp.rock);
        c = mix(c, paint.snow.rgb, sp.snow);
    }
    return mix(c, paint.frost.rgb, frost);
}

fn debug_colour(cls: u32) -> vec3<f32> {
    var c = vec3<f32>(0.5);
    switch cls {
        case SEA: { c = vec3<f32>(0.0, 0.0, 1.0); }
        case FARM: { c = vec3<f32>(1.0, 1.0, 0.0); }
        case PASTURE: { c = vec3<f32>(0.0, 1.0, 0.0); }
        case WOOD: { c = vec3<f32>(0.0, 0.5, 0.0); }
        case HEATH: { c = vec3<f32>(1.0, 0.0, 1.0); }
        case QUARRY: { c = vec3<f32>(0.0, 1.0, 1.0); }
        case CLAYPIT: { c = vec3<f32>(1.0, 0.5, 0.0); }
        case IRONMINE: { c = vec3<f32>(0.5, 0.0, 0.0); }
        case COALMINE: { c = vec3<f32>(0.5, 0.0, 1.0); }
        case WORKS: { c = vec3<f32>(1.0, 0.0, 0.0); }
        case URBAN: { c = vec3<f32>(0.0, 0.5, 1.0); }
        case DERELICT: { c = vec3<f32>(0.5, 1.0, 0.5); }
        default: {}
    }
    return c;
}

// ------------------------------------------------------------------ the map overlay

fn overlay_tap(c: vec2<i32>) -> vec4<f32> {
    let lim = vec2<i32>(i32(ground.width) - 1, i32(ground.height) - 1);
    return textureLoad(overlay_cells, clamp(c, vec2<i32>(0), lim), 0);
}

// The overlay washed over the ground colour `col` at `p`: the tint of the
// square the economy works in, as flat colour on a paper map, with the fields
// and hedges showing through faintly. The edges of the squares wander a
// little, as if drawn by hand, and where two regions meet (two towns' market
// areas) there is an ink line, kept at least a pixel and a half wide.
fn overlay_wash(col: vec3<f32>, p: vec2<f32>, foot: f32, land: f32) -> vec3<f32> {
    let q = p + 0.11 * vec2<f32>(vnoise(p.x * 1.7 + 3.1, p.y * 1.7 + 9.2, 91), vnoise(p.x * 1.7 + 7.7, p.y * 1.7 + 1.3, 93));
    let b = q - 0.5;
    let i0 = vec2<i32>(floor(b));
    let f = b - floor(b);
    let t00 = overlay_tap(i0);
    let t10 = overlay_tap(i0 + vec2<i32>(1, 0));
    let t01 = overlay_tap(i0 + vec2<i32>(0, 1));
    let t11 = overlay_tap(i0 + vec2<i32>(1, 1));
    // blend the four squares round the nearest corner, leaving out any with no tint
    let s = max(ground.overlay_soft, 0.75 * foot);
    let wx = smoothstep(0.5 - s, 0.5 + s, f.x);
    let wy = smoothstep(0.5 - s, 0.5 + s, f.y);
    let w00 = (1.0 - wx) * (1.0 - wy) * step(0.002, t00.a);
    let w10 = wx * (1.0 - wy) * step(0.002, t10.a);
    let w01 = (1.0 - wx) * wy * step(0.002, t01.a);
    let w11 = wx * wy * step(0.002, t11.a);
    let cover = w00 + w10 + w01 + w11;
    if (cover <= 0.001) {
        return col;
    }
    let tint = (w00 * t00.rgb + w10 * t10.rgb + w01 * t01.rgb + w11 * t11.rgb) / cover;
    // the ground under the wash, as light and dark only
    let grey = dot(col, vec3<f32>(0.30, 0.59, 0.11));
    var washed = tint * (0.74 + 0.52 * grey);
    // the square this spot is in, and its neighbours across the two nearest edges
    let right = f.x >= 0.5;
    let down = f.y >= 0.5;
    let own = select(select(t00.a, t10.a, right), select(t01.a, t11.a, right), down);
    let nx = select(select(t10.a, t00.a, right), select(t11.a, t01.a, right), down);
    let ny = select(select(t01.a, t11.a, right), select(t00.a, t10.a, right), down);
    let hw = max(0.035, 0.8 * foot);
    let aa = max(0.5 * foot, 1e-4);
    var ink = 0.0;
    if (own > 0.002 && nx > 0.002 && abs(own - nx) > 0.002) {
        ink = max(ink, 1.0 - smoothstep(hw - aa, hw + aa, abs(f.x - 0.5)));
    }
    if (own > 0.002 && ny > 0.002 && abs(own - ny) > 0.002) {
        ink = max(ink, 1.0 - smoothstep(hw - aa, hw + aa, abs(f.y - 0.5)));
    }
    washed = mix(washed, vec3<f32>(0.27, 0.22, 0.17), 0.72 * ink * ground.overlay_lines);
    return mix(col, washed, ground.overlay * min(cover, 1.0) * land);
}

// ------------------------------------------------------------------ the fragment

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let p = in.world_position.xz;
    let h = in.world_position.y;

    // Everything that needs screen-space derivatives comes first, while the
    // control flow is still uniform.
    let foot = max(length(dpdx(p)), length(dpdy(p)));
    let hf = max(fwidth(h), 1e-5);

    // the parcel here, and the point that stands for it (fields::parcel)
    let bl = locate(to_block(p));
    let pc = cut(bl, Cut(vec2<f32>(0.0), vec2<f32>(1.0), 1u), 0u, PARCEL_LEVEL);
    // its class this year and what does not change about it, from the parcel
    // textures (world::Land::index): the CPU decides the class of every
    // parcel (world::year_land) and the shader only looks it up
    let tex_lim = vec2<i32>(textureDimensions(year_parcels)) - vec2<i32>(1);
    let pt = clamp(vec2<i32>(8 * (bl.i - ground.block_i0) + i32(pc.id) - 8, bl.j - ground.block_j0), vec2<i32>(0), tex_lim);
    let yp = textureLoad(year_parcels, pt, 0);
    let fp = textureLoad(fixed_parcels, pt, 0);
    let cls = u32(round(yp.x * 255.0));
    let fixed_bits = u32(round(fp.x * 255.0));
    let terr = fixed_bits >> 4u;

    if (ground.debug != 0u) {
        // the class alone, unlit, for checking against world::class_at
        var dbg: FragmentOutput;
        dbg.color = vec4<f32>(srgb_to_linear(debug_colour(cls)), 1.0);
        return dbg;
    }

    // the field: the parcel cut as finely as it asks (world::field_at)
    let depth = max(fixed_bits & 15u, PARCEL_LEVEL);
    let open_code = round(fp.y * 255.0);
    let open = open_code > 0.0 && ground.year < 1750.0 + open_code;
    let leaf = cut(bl, pc, PARCEL_LEVEL, depth);
    // distance to its nearest side, through the Jacobian of the block (fields::field)
    let g = bl.a - bl.b + bl.c - bl.d;
    let ju = (bl.b - bl.a) + g * bl.uv.y;
    let jv = (bl.d - bl.a) + g * bl.uv.x;
    let jdet = cross2(ju, jv);
    let det = max(abs(jdet), 1e-6);
    let du0 = bl.uv.x - leaf.lo.x;
    let du1 = leaf.hi.x - bl.uv.x;
    let dv0 = bl.uv.y - leaf.lo.y;
    let dv1 = leaf.hi.y - bl.uv.y;
    let su = det / max(length(jv), 1e-6);
    let sv = det / max(length(ju), 1e-6);
    let eu = min(du0, du1) * su;
    let ev = min(dv0, dv1) * sv;
    let gu = vec2<f32>(jv.y, -jv.x) / jdet;
    let gv = vec2<f32>(-ju.y, ju.x) / jdet;
    var edge = ev;
    var dir = select(gv, -gv, dv0 < dv1);
    if (eu < ev) {
        edge = eu;
        dir = select(gu, -gu, du0 < du1);
    }
    dir = normalize(dir);
    let toward = vec2<f32>(TURN_C * dir.x - TURN_S * dir.y, TURN_S * dir.x + TURN_C * dir.y);
    let size = min((leaf.hi.x - leaf.lo.x) * su, (leaf.hi.y - leaf.lo.y) * sv) * BLOCK;

    let ph = fract(ground.season);
    // a slow swell of lighter and darker, warmer and cooler ground, so wide
    // areas do not look flat
    let swell = 0.6 * vnoise(p.x * 0.35 + 11.3, p.y * 0.35 + 5.9, 61) + 0.4 * vnoise(p.x * 1.3, p.y * 1.3, 67);
    let warm = vnoise(p.x * 0.5 + 3.3, p.y * 0.5 + 8.1, 63);
    let n2 = vnoise(p.x * 3.9, p.y * 3.9, 37);

    // sea floor below -0.004, beach up to 0.022, land above; a river's valley
    // floor is land however low it lies
    let landk = max(clamp((h - 0.022) / hf + 0.5, 0.0, 1.0), smoothstep(0.35, 0.75, in.uv.x));
    let wetk = clamp((-0.004 - h) / hf + 0.5, 0.0, 1.0);
    let depth_w = clamp(-h, 0.0, 1.0);
    var col = select(
        mix(paint.seabed_shelf.rgb, paint.seabed_deep.rgb, min((depth_w - 0.12) / 0.6, 1.0)),
        mix(paint.seabed_sand.rgb, paint.seabed_shelf.rgb, depth_w / 0.12),
        depth_w < 0.12,
    );
    let nrm = normalize(in.world_normal);
    let slope = length(nrm.xz) / max(nrm.y, 0.05);
    if (wetk < 1.0) {
        // dry sand, darker where the sea has just been; shingle and rock on a steep shore
        var top = mix(paint.beach_wet.rgb, paint.beach.rgb, smoothstep(0.0, 0.012, h));
        top = mix(top, paint.shingle.rgb, smoothstep(0.5, 1.2, slope));
        if (landk > 0.0) {
            let winter_k = winter(ph);
            var sp: Spot;
            sp.p = p;
            sp.foot = foot;
            sp.ph = ph;
            sp.terr = terr;
            sp.dense = yp.z;
            sp.rock = clamp((slope - 1.15 + 0.3 * n2) / 1.1, 0.0, 1.0);
            // the snow line comes down the hills in winter
            sp.snow = clamp((h - 1.75 + 0.55 * winter_k + 0.12 * vnoise(p.x * 1.7, p.y * 1.7, 31)) / 0.45, 0.0, 1.0) * (1.0 - 0.5 * sp.rock);
            sp.frost = winter_k * clamp(0.36 + 0.2 * swell + 0.12 * h, 0.0, 0.62);
            sp.flood = in.uv.x;
            sp.key = field_key(bl, leaf);
            sp.bl = bl;
            sp.leaf = leaf;
            sp.edge = edge * BLOCK;
            sp.toward = toward;
            sp.size = size;
            sp.open = open;
            let lc = class_colour(cls, sp);
            top = mix(top, lc, landk);
        }
        col = mix(top, col, wetk);
    }
    col *= 1.0 + 0.05 * swell;
    col *= vec3<f32>(1.0 + 0.015 * warm, 1.0, 1.0 - 0.035 * warm);
    // faint contour lines on the open hillsides (heath and moor, rock and
    // snow), as on an engraved map; not across fields, where they would read as
    // marks on the crops. Gone before they crowd closer than a few pixels.
    let ci = h / 0.08;
    let cw = hf / 0.08;
    let open_ground = select(0.0, 1.0, cls == HEATH || cls == DERELICT || terr >= 3u);
    let contour = lines(ci, cw, 0.07) * smoothstep(0.45, 0.7, h) * (1.0 - smoothstep(0.12, 0.3, cw));
    col *= 1.0 - 0.15 * contour * landk * open_ground;
    // fine grain, gone before it gets smaller than a pixel
    col *= 1.0 + 0.025 * vnoise(p.x * 23.0, p.y * 23.0, 77) * detail(23.0, foot);
    // a map overlay, when one is chosen; without one nothing here runs
    if (ground.overlay > 0.0) {
        col = overlay_wash(col, p, foot, landk * (1.0 - wetk));
    }

    pbr_input.material.base_color = vec4<f32>(srgb_to_linear(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
