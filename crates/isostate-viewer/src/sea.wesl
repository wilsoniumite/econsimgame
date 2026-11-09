// The sea's surface. It knows how deep the water is under every pixel, from
// the ground's own heightfield, so the colour runs from clear turquoise over
// the shallows to dark blue far out, and surf runs in to the shore: a line of
// foam at the water's edge and bands of it that roll in and fade. Small
// ripples move over it. Light, shadows and haze are Bevy's, as for any
// StandardMaterial.

import bevy_pbr::render::{
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::globals,
};

// terrain.rs SeaParams
struct SeaParams {
    // the heightfield's size in vertices, and vertices per cell
    grid_w: f32,
    grid_h: f32,
    res: f32,
    // phase of the year
    season: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> sea: SeaParams;
// the ground's height at every vertex
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var heights: texture_2d<f32>;

fn hex(c: u32) -> vec3<f32> {
    return vec3<f32>(f32((c >> 16u) & 255u), f32((c >> 8u) & 255u), f32(c & 255u)) / 255.0;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

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

// The ground under the water, as the terrain mesh has it (deep sea off the map).
fn ground_at(p: vec2<f32>) -> f32 {
    let g = p * sea.res;
    if (g.x < 0.0 || g.y < 0.0 || g.x > sea.grid_w - 1.001 || g.y > sea.grid_h - 1.001) {
        return -1.2;
    }
    let i = vec2<i32>(floor(g));
    let f = g - floor(g);
    let a = textureLoad(heights, i, 0).x;
    let b = textureLoad(heights, i + vec2<i32>(1, 0), 0).x;
    let c = textureLoad(heights, i + vec2<i32>(0, 1), 0).x;
    let d = textureLoad(heights, i + vec2<i32>(1, 1), 0).x;
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let p = in.world_position.xz;
    let foot = max(length(dpdx(p)), length(dpdy(p)));
    let t = globals.time;
    let depth = max(-ground_at(p), 0.0);

    // clear over the sand, green-blue over the shelf, dark blue far out
    let k1 = smoothstep(0.0, 0.07, depth);
    let k2 = smoothstep(0.05, 0.5, depth);
    let k3 = smoothstep(0.4, 1.1, depth);
    var col = mix(hex(0x9ad6cbu), hex(0x5aa8b8u), k1);
    col = mix(col, hex(0x3b7fa3u), k2);
    col = mix(col, hex(0x2c5f86u), k3);
    var alpha = mix(0.28, 0.9, smoothstep(0.0, 0.3, depth));

    // surf: a line of foam at the edge, and bands rolling in over the
    // shallows, broken up so they never make a perfect ring; gone before they
    // are smaller than a pixel
    let breakup = 0.5 + 0.5 * vnoise(p.x * 9.0 + t * 0.05, p.y * 9.0, 7);
    let edge = 1.0 - smoothstep(0.0, 0.006 + 0.6 * foot, depth);
    let roll = fract(depth * 34.0 - t * 0.12 + 0.35 * vnoise(p.x * 2.0, p.y * 2.0, 5));
    let bands = smoothstep(0.82, 0.97, roll) * (1.0 - smoothstep(0.004, 0.05, depth)) * smoothstep(0.25, 0.75, breakup);
    let fine = 1.0 - smoothstep(0.004, 0.02, foot);
    let foam = clamp(edge * (0.55 + 0.45 * breakup) + 0.7 * bands * fine, 0.0, 1.0);
    col = mix(col, hex(0xf4f6f2u), foam);
    alpha = max(alpha, foam * 0.95);

    // ripples: the surface's normal wobbles a little, so the sun glints
    let rx = vnoise(p.x * 22.0 + t * 0.35, p.y * 22.0 - t * 0.2, 11) + 0.5 * vnoise(p.x * 47.0 - t * 0.5, p.y * 47.0 + t * 0.3, 13);
    let rz = vnoise(p.x * 22.0 - t * 0.25, p.y * 22.0 + t * 0.3, 17) + 0.5 * vnoise(p.x * 47.0 + t * 0.4, p.y * 47.0 - t * 0.45, 19);
    let calm = 1.0 - smoothstep(0.003, 0.012, foot);
    pbr_input.N = normalize(pbr_input.N + vec3<f32>(rx, 0.0, rz) * 0.06 * calm);

    pbr_input.material.base_color = vec4<f32>(srgb_to_linear(col), alpha);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
