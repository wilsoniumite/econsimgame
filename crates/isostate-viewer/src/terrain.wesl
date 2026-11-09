// The island's ground, worked out per pixel so fields and hedges stay crisp at
// any zoom, while Bevy's lighting, shadows, ambient occlusion and fog work as
// for any StandardMaterial.
//
// Each 2 km cell has a mix of two land classes for the year: mix_a, mix_b and
// mix_s, the share of mix_a in 255ths. The patchwork that turns the mix into
// fields, woods and towns follows world.rs exactly: the same warp, the same
// integer hash, the same value noise and the same quantile table, and the lower
// class number always takes the low noise values. The trees and houses that
// scatter.rs places with that rule therefore stand on matching ground.
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
    // the 257 quantiles of the patch noise, four to a vector
    quant: array<vec4<f32>, 65>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> ground: PatchworkParams;
// per cell, this year: mix_a, mix_b, mix_s, town density
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var year_cells: texture_2d<f32>;
// per cell, fixed: terrain class (0 sea, 1 lowland, 2 hills, 3 mountains)
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var fixed_cells: texture_2d<f32>;

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

const FIELDS_PER_CELL: f32 = 3.2;
// hedge width, in fields
const HEDGE: f32 = 0.05;
const TAU: f32 = 6.2831853;

// ------------------------------------------------------------------ the CPU rule
// hash2, vnoise and warp are world.rs line for line; keep them in step. WGSL
// u32 arithmetic wraps like Rust's wrapping_mul, so the hash matches bit for bit.

fn hash2(x: i32, y: i32, s: i32) -> f32 {
    var h = (bitcast<u32>(x) * 374761393u) ^ (bitcast<u32>(y) * 668265263u) ^ (bitcast<u32>(s) * 1442695041u);
    h = (h ^ (h >> 13u)) * 1274126177u;
    h = h ^ (h >> 16u);
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

fn warp(p: vec2<f32>) -> vec2<f32> {
    let wx = p.x + 0.40 * vnoise(p.x * 0.7, p.y * 0.7, 5) + 0.15 * vnoise(p.x * 2.3, p.y * 2.3, 9);
    let wy = p.y + 0.40 * vnoise(p.x * 0.7 + 31.7, p.y * 0.7 + 17.3, 7) + 0.15 * vnoise(p.x * 2.3 + 5.1, p.y * 2.3 + 9.7, 13);
    return vec2<f32>(wx, wy);
}

fn quantile(i: u32) -> f32 {
    return ground.quant[i >> 2u][i & 3u];
}

// ------------------------------------------------------------------ helpers

fn hex(c: u32) -> vec3<f32> {
    return vec3<f32>(f32((c >> 16u) & 255u), f32((c >> 8u) & 255u), f32(c & 255u)) / 255.0;
}

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
    // field-grid position (the warped position times FIELDS_PER_CELL) and its
    // change one pixel right and one pixel down
    g: vec2<f32>,
    gdx: vec2<f32>,
    gdy: vec2<f32>,
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
    let soil = mix(hex(0xb08455u), hex(0x9a7651u), 0.6 * winter(ph));
    let stubble = hex(0xdccb98u);
    var canopy = hex(0xa4c660u);
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
                canopy = mix(canopy, hex(0xe3c25eu), smoothstep(0.42, 0.53, ph));
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
            let ripe = select(hex(0xebd891u), hex(0xd8c779u), crop == 4u);
            canopy = mix(canopy, ripe, smoothstep(0.44, 0.54, ph));
            rows = mix(0.07, 0.02, cover);
        }
        case 5u, 6u: {
            // turnips and clover: green all year, darker and grazed in winter
            cover = 0.96 - 0.3 * winter(ph);
            canopy = mix(hex(0x9cc05au), hex(0x7f9d5bu), winter(ph));
            rows = 0.045;
        }
        case 7u: {
            // hay meadow: fresh in spring, cut in early summer and left in
            // windrows to dry, green again by autumn
            let cut = 0.5 + 0.04 * k;
            cover = 1.0;
            var grass = mix(hex(0x9bc35eu), hex(0xa8cc66u), bell(ph, 0.32, 0.12));
            grass = mix(grass, hex(0xb8c46au), smoothstep(0.38, 0.48, ph) * (1.0 - smoothstep(cut, cut + 0.01, ph)));
            let drying = smoothstep(cut, cut + 0.01, ph) * (1.0 - smoothstep(cut + 0.08, cut + 0.2, ph));
            canopy = mix(grass, hex(0xcdbb69u), drying);
            canopy = mix(canopy, hex(0x8d9a6au), 0.5 * winter(ph));
            rows = 0.06 * drying;
        }
        default: {
            // fallow: bare furrows, a little green with weeds in summer
            cover = 0.25 * bell(ph, 0.58, 0.14);
            canopy = hex(0x9aae5cu);
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
    c = mix(c, mix(c, hex(0xbdbb72u), 0.3), bell(ph, 0.67, 0.08));
    c = mix(c, mix(c, hex(0x8f9f5au), 0.35), bell(ph, 0.84, 0.1));
    c = mix(c, mix(c, hex(0x8a9470u), 0.55), winter(ph));
    return c;
}

fn meadow_look(m: u32, ph: f32) -> Look {
    var c = hex(0x8fc15au);
    switch m {
        case 1u: { c = hex(0x9bcb63u); }
        case 2u: { c = hex(0x86b851u); }
        case 3u: { c = hex(0xa6cf71u); }
        default: {}
    }
    return Look(grass_season(c, ph), 0.0);
}

// The field with grid id `id`: its crop chosen by hash as the CPU painter did,
// its own calendar a week or so either way, and its own shade.
fn field_look(id: vec2<i32>, pasture: bool, ph: f32) -> Look {
    let f = u32(hash2(id.x, id.y, 41) * 255.0);
    let k = hash2(id.x, id.y, 53);
    var look: Look;
    if (pasture) {
        look = meadow_look(f % 4u, ph);
    } else {
        look = crop_look(f % 10u, fract(ph + 0.06 * (k - 0.5)), k);
    }
    look.col *= 0.955 + 0.09 * fract(k * 7.31);
    return look;
}

// Up close: one field, with rows that run along one of its sides, give or take
// a few degrees, and fade out before they get finer than a pixel.
fn field_near(sp: Spot, pasture: bool) -> vec3<f32> {
    let id = vec2<i32>(floor(sp.g));
    let look = field_look(id, pasture, sp.ph);
    let r = hash2(id.x, id.y, 47);
    let ang = select(0.0, 1.5707963, r < 0.5) + (fract(r * 7.0) - 0.5) * 0.3;
    let dir = vec2<f32>(cos(ang), sin(ang));
    let n = 11.0 + floor(fract(r * 13.0) * 6.0);
    let x = dot(sp.g - floor(sp.g), dir) * n;
    let xf = (abs(dot(sp.gdx, dir)) + abs(dot(sp.gdy, dir))) * n;
    let wave = cos(TAU * x) * (1.0 - smoothstep(0.2, 0.5, xf));
    return look.col * (1.0 + look.rows * wave);
}

fn field_flat(q: vec2<f32>, pasture: bool, ph: f32) -> vec3<f32> {
    return field_look(vec2<i32>(floor(q)), pasture, ph).col;
}

fn fields(sp: Spot, pasture: bool) -> vec3<f32> {
    let gf = max(length(sp.gdx), length(sp.gdy));
    var c: vec3<f32>;
    if (gf < 0.25) {
        c = field_near(sp, pasture);
    } else {
        // far away a pixel spans several fields: average four points across it
        let a = 0.25 * (sp.gdx + sp.gdy);
        let b = 0.25 * (sp.gdx - sp.gdy);
        c = 0.25 * (field_flat(sp.g + a, pasture, sp.ph) + field_flat(sp.g - a, pasture, sp.ph)
            + field_flat(sp.g + b, pasture, sp.ph) + field_flat(sp.g - b, pasture, sp.ph));
    }
    if (pasture) {
        // tussocks and wetter hollows
        c *= 1.0 + 0.06 * vnoise(sp.p.x * 11.0, sp.p.y * 11.0, 83) * detail(11.0, sp.foot);
    }
    c = mix(c, hex(0x9b9588u), sp.rock);
    c = mix(c, hex(0xf5f5f1u), sp.snow);
    // hedges on every field edge, uneven along their length and bushy at the
    // edges up close
    let fx = max(abs(sp.gdx.x), abs(sp.gdy.x));
    let fy = max(abs(sp.gdx.y), abs(sp.gdy.y));
    let ix = i32(round(sp.g.x));
    let iy = i32(round(sp.g.y));
    let bushy = 0.3 * detail(55.0, sp.foot);
    let wx = HEDGE * (0.8 + 0.4 * noise1(sp.g.y * 4.0, ix, 71) + bushy * (noise1(sp.g.y * 17.0, ix, 75) - 0.5));
    let wy = HEDGE * (0.8 + 0.4 * noise1(sp.g.x * 4.0, iy, 73) + bushy * (noise1(sp.g.x * 17.0, iy, 79) - 0.5));
    // like a model's, each hedge stands a little proud and casts a soft shadow
    // to the north-east, away from the low south-western sun
    let sw = 0.7 * HEDGE;
    let shadow = 1.0 - (1.0 - lines(sp.g.x - 0.5 * (wx + sw), fx, sw)) * (1.0 - lines(sp.g.y + 0.5 * (wy + sw), fy, sw));
    c *= 1.0 - 0.16 * shadow;
    let cover = 1.0 - (1.0 - lines(sp.g.x, fx, wx)) * (1.0 - lines(sp.g.y, fy, wy));
    var hc = hex(0x4a7533u);
    hc = mix(hc, hex(0x6b6837u), 0.6 * bell(sp.ph, 0.84, 0.1));
    hc = mix(hc, hex(0x575443u), 0.6 * winter(sp.ph));
    // darker and lighter bushes along the hedge, which shows through snow
    hc *= 1.0 + 0.14 * vnoise(sp.g.x * 9.0, sp.g.y * 9.0, 77) * detail(29.0, sp.foot);
    hc = mix(hc, hex(0xf5f5f1u), 0.35 * sp.snow);
    return mix(c, hc, 0.9 * cover);
}

// ------------------------------------------------------------------ other ground

fn wood_floor(sp: Spot) -> vec3<f32> {
    let n = vnoise(sp.p.x * 12.0, sp.p.y * 12.0, 81);
    let shade = spots(n, 0.125, 12.0 * sp.foot, detail(12.0, sp.foot), 0.4);
    var c = mix(hex(0x4b7738u), hex(0x3f6a30u), shade);
    c = mix(c, hex(0x5a8a3eu), 0.35 * bell(sp.ph, 0.33, 0.12));
    c = mix(c, hex(0x7a6a3au), 0.45 * bell(sp.ph, 0.83, 0.12));
    c = mix(c, hex(0x5b5945u), 0.5 * winter(sp.ph));
    return c;
}

fn heath(sp: Spot) -> vec3<f32> {
    var base = hex(0xbcbb7eu);
    if (sp.terr == 2u) {
        base = hex(0xaead73u);
    } else if (sp.terr == 3u) {
        base = hex(0xa39d8fu);
    }
    // heather flowers in late summer, gorse in spring; brown in autumn and winter
    let bloom = smoothstep(0.52, 0.6, sp.ph) * (1.0 - smoothstep(0.72, 0.8, sp.ph));
    let flower = bell(sp.ph, 0.34, 0.09);
    // soft drifts where heather grows thicker, a few hundred metres across
    let drift = smoothstep(-0.2, 0.6, vnoise(sp.p.x * 6.0 + 4.1, sp.p.y * 6.0 + 2.3, 19));
    base = mix(base, mix(hex(0xa08f88u), hex(0xa27ca2u), bloom), (0.22 + 0.14 * bloom) * drift);
    base = mix(base, hex(0x9c8a68u), 0.3 * max(winter(sp.ph), bell(sp.ph, 0.86, 0.1)));
    // and small clumps of heather and gorse
    let n = vnoise(sp.p.x * 34.0, sp.p.y * 34.0, 17);
    let nf = 34.0 * sp.foot;
    let d = detail(34.0, sp.foot);
    let heather = spots(n, 0.433, nf, d, 0.18);
    let gorse = spots(-n, 0.58, nf, d, 0.1);
    var c = mix(base, mix(hex(0xa58fa6u), hex(0x9a62a6u), bloom), heather * mix(0.4, 0.6, bloom));
    c = mix(c, mix(hex(0x7d8a45u), hex(0xd9b93cu), flower), gorse * mix(0.4, 0.7, flower));
    return c;
}

// Light mottling for bare ground.
fn mottled(base: vec3<f32>, sp: Spot, amount: f32) -> vec3<f32> {
    let n = vnoise(sp.p.x * 9.0, sp.p.y * 9.0, 95);
    return base * (1.0 + amount * n * detail(9.0, sp.foot));
}

// Streets run between every second row of the house plots in scatter.rs
// (seven to a cell), so the blocks are two plots square.
fn town(sp: Spot) -> vec3<f32> {
    let q = sp.p * 3.5;
    let qf = 3.5 * sp.foot;
    let id = vec2<i32>(floor(q));
    let b = hash2(id.x, id.y, 91);
    // some blocks keep their gardens, fewer in crowded towns
    let share = max(0.45 - 0.4 * sp.dense, 0.0);
    // (block by block only up close: far away a checkerboard would look busy)
    let garden = mix(share, select(0.0, 1.0, b > 0.55 + 0.4 * sp.dense), detail(10.0, sp.foot));
    var c = mix(hex(0xcbb9a1u), hex(0xaab27bu), 0.3 * garden);
    let street = 1.0 - (1.0 - lines(q.x, qf, 0.1)) * (1.0 - lines(q.y, qf, 0.1));
    c = mix(c, hex(0xdccfbbu), street);
    return mottled(c, sp, 0.03);
}

// Works stand in yards of cinder and gravel with lanes between them.
fn works_yard(sp: Spot) -> vec3<f32> {
    let q = sp.p * 3.5;
    let qf = 3.5 * sp.foot;
    let id = vec2<i32>(floor(q));
    let b = hash2(id.x, id.y, 93);
    let base = hex(0x8f857au);
    let cinder = hex(0x786f67u);
    let gravel = hex(0xa1998cu);
    var yard = base;
    if (b < 0.3) {
        yard = cinder;
    } else if (b > 0.72) {
        yard = gravel;
    }
    let average = base * 0.42 + cinder * 0.3 + gravel * 0.28;
    var c = mix(average, yard, detail(10.0, sp.foot));
    let lane = 1.0 - (1.0 - lines(q.x, qf, 0.07)) * (1.0 - lines(q.y, qf, 0.07));
    c = mix(c, hex(0xa69e92u), 0.8 * lane);
    return mottled(c, sp, 0.05);
}

fn derelict(sp: Spot) -> vec3<f32> {
    let n = vnoise(sp.p.x * 7.0, sp.p.y * 7.0, 97);
    let weeds = spots(n, 0.19, 7.0 * sp.foot, detail(7.0, sp.foot), 0.35);
    return mix(hex(0x968f81u), hex(0x8f9a62u), 0.6 * weeds);
}

fn class_colour(cls: u32, sp: Spot) -> vec3<f32> {
    var c: vec3<f32>;
    switch cls {
        case FARM: { c = fields(sp, false); }
        case PASTURE: { c = fields(sp, true); }
        case WOOD: { c = wood_floor(sp); }
        case HEATH: { c = heath(sp); }
        case QUARRY: { c = mottled(hex(0xc9c3b7u), sp, 0.06); }
        case CLAYPIT: { c = mottled(hex(0xc69466u), sp, 0.08); }
        case IRONMINE: { c = mottled(hex(0xa76c4cu), sp, 0.08); }
        case COALMINE: { c = mottled(hex(0x4a4643u), sp, 0.12); }
        case WORKS: { c = works_yard(sp); }
        case URBAN: { c = town(sp); }
        case DERELICT: { c = derelict(sp); }
        // land that warps into a sea cell: rough coastal grass
        default: { c = hex(0xbcbb7eu); }
    }
    var frost = sp.frost;
    if (cls == URBAN || cls == WORKS) {
        frost *= 0.6;
    } else if (cls != FARM && cls != PASTURE) {
        // fields() lays its own rock and snow under the hedges
        c = mix(c, hex(0x9b9588u), sp.rock);
        c = mix(c, hex(0xf5f5f1u), sp.snow);
    }
    return mix(c, hex(0xdfe5e8u), frost);
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
    let w = warp(p);
    let g = w * FIELDS_PER_CELL;
    let gdx = dpdx(g);
    let gdy = dpdy(g);
    // the patch noise of world.rs, sub_raw; far away the octaves finer than a
    // pixel fade out so the patch edges do not crawl
    let n1 = vnoise(p.x * 1.7, p.y * 1.7, 31);
    let n2 = vnoise(p.x * 3.9, p.y * 3.9, 37);
    let n3 = vnoise(p.x * 8.3, p.y * 8.3, 43);
    let v = n1 + 0.55 * n2 * detail(3.9, foot) + 0.25 * n3 * detail(8.3, foot);
    let vf = max(fwidth(v), 1e-6);

    // the warped cell decides the mix, as in world::class_at
    let cell = clamp(vec2<i32>(floor(w)), vec2<i32>(0), vec2<i32>(i32(ground.width) - 1, i32(ground.height) - 1));
    let yc = textureLoad(year_cells, cell, 0);
    let fc = textureLoad(fixed_cells, cell, 0);
    let ca = u32(round(yc.x * 255.0));
    let cb = u32(round(yc.y * 255.0));
    let s = u32(round(yc.z * 255.0));
    let terr = u32(round(fc.x * 255.0));
    var lo = min(ca, cb);
    var hi = max(ca, cb);
    if (terr == 0u) {
        lo = SEA;
        hi = SEA;
    }
    // world::pick_class: the lower class takes the low noise values, and it
    // holds where sub_uniform * 255 < t, its share in 255ths. With the
    // quantile table that is the same as sub_raw <= quant[t].
    let t = select(255u - s, s, ca < cb);
    let single = lo == hi || t >= 255u;

    if (ground.debug != 0u) {
        // the exact rule, unlit, for checking against world::class_at
        var cls = lo;
        if (!single && (t == 0u || n1 + 0.55 * n2 + 0.25 * n3 > quantile(t))) {
            cls = hi;
        }
        var dbg: FragmentOutput;
        dbg.color = vec4<f32>(srgb_to_linear(debug_colour(cls)), 1.0);
        return dbg;
    }

    // share of the pixel that is the lower class, anti-aliased along the edge
    var cover = 1.0;
    if (!single) {
        if (t == 0u) {
            cover = 0.0;
        } else {
            cover = clamp((quantile(t) - v) / vf + 0.5, 0.0, 1.0);
        }
    }

    let ph = fract(ground.season);
    // a slow swell of lighter and darker, warmer and cooler ground, so wide
    // areas do not look flat
    let swell = 0.6 * vnoise(p.x * 0.35 + 11.3, p.y * 0.35 + 5.9, 61) + 0.4 * vnoise(p.x * 1.3, p.y * 1.3, 67);
    let warm = vnoise(p.x * 0.5 + 3.3, p.y * 0.5 + 8.1, 63);

    // sea floor below -0.004, beach up to 0.03, land above, as before
    let landk = clamp((h - 0.03) / hf + 0.5, 0.0, 1.0);
    let wetk = clamp((-0.004 - h) / hf + 0.5, 0.0, 1.0);
    let depth = clamp(-h, 0.0, 1.0);
    var col = select(
        mix(hex(0x6fa8a3u), hex(0x2f5f78u), min((depth - 0.12) / 0.6, 1.0)),
        mix(hex(0xd2cd9cu), hex(0x6fa8a3u), depth / 0.12),
        depth < 0.12,
    );
    if (wetk < 1.0) {
        // dry sand, darker where the sea has just been
        var top = mix(hex(0xcfbd8bu), hex(0xead9a3u), smoothstep(0.0, 0.014, h));
        if (landk > 0.0) {
            let nrm = normalize(in.world_normal);
            let slope = length(nrm.xz) / max(nrm.y, 0.05);
            let winter_k = winter(ph);
            var sp: Spot;
            sp.p = p;
            sp.g = g;
            sp.gdx = gdx;
            sp.gdy = gdy;
            sp.foot = foot;
            sp.ph = ph;
            sp.terr = terr;
            sp.dense = yc.w;
            sp.rock = clamp((slope - 1.1 + 0.3 * n2) / 1.2, 0.0, 1.0);
            // the snow line comes down the hills in winter
            sp.snow = clamp((h - 1.75 + 0.55 * winter_k + 0.12 * n1) / 0.45, 0.0, 1.0) * (1.0 - 0.5 * sp.rock);
            sp.frost = winter_k * clamp(0.36 + 0.2 * swell + 0.12 * h, 0.0, 0.62);
            var lc: vec3<f32>;
            if (cover >= 0.999) {
                lc = class_colour(lo, sp);
            } else if (cover <= 0.001) {
                lc = class_colour(hi, sp);
            } else {
                lc = mix(class_colour(hi, sp), class_colour(lo, sp), cover);
            }
            top = mix(top, lc, landk);
        }
        col = mix(top, col, wetk);
    }
    col *= 1.0 + 0.05 * swell;
    col *= vec3<f32>(1.0 + 0.015 * warm, 1.0, 1.0 - 0.035 * warm);
    // fine grain, gone before it gets smaller than a pixel
    col *= 1.0 + 0.025 * vnoise(p.x * 23.0, p.y * 23.0, 77) * detail(23.0, foot);

    pbr_input.material.base_color = vec4<f32>(srgb_to_linear(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
