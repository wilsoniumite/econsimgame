// What is built, through the winter: houses, works, churches, farm buildings
// and bridges. Their meshes are made once a year and the seasons turn far
// faster, so nothing of winter is in the meshes. This is Bevy's
// StandardMaterial with one thing added before the light: where a surface
// faces the sky, winter lies on it.
//
// What winter does there is said by the mark each palette token carries, in
// the alpha of a corner's colour (isostate_gen::colour::settles), which a
// solid material has no other use for:
//
//   1     snow lies: a roof, the top of a wall, a stack of goods
//   0.5   ground, and what grows on it: the frost the land round it takes,
//         and its green gone dull (a lawn) or to bare earth (a bed, a hedge)
//   0.25  standing water: it freezes over
//   0     a fire, the mouth of a flue: nothing settles
//
// So it does not matter how a roof is put together, only which way each of
// its faces looks. With no winter the colour passes through untouched.

import bevy_pbr::render::{
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
};

// built.rs WinterParams
struct WinterParams {
    // x: how much snow lies (0 none, 1 midwinter); y: how much frost; z: how
    // much of a surface the snow covers at its deepest; w: how strong the frost is
    amounts: vec4<f32>,
    // linear colours
    snow: vec4<f32>,
    frost: vec4<f32>,
    ice: vec4<f32>,
    bare_earth: vec4<f32>,
    winter_grass: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> winter: WinterParams;

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
    return mix(mix(a, b, sx), mix(c, d, sx), sy);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let base = pbr_input.material.base_color;
    var col = base.rgb;
    let snow_k = winter.amounts.x;
    let frost_k = winter.amounts.y;
    if (frost_k > 0.0) {
        let mark = base.a;
        let p = in.world_position.xz;
        // how squarely the surface faces the sky: a roof's pitch holds snow, a spire's does not
        let up = smoothstep(0.28, 0.6, normalize(in.world_normal).y);
        // snow and frost come and go in patches a few paces across, never all at once
        let drift = 0.65 * vnoise(p.x * 55.0, p.y * 55.0, 5) + 0.35 * vnoise(p.x * 190.0, p.y * 190.0, 9);
        if (mark > 0.75) {
            let lying = smoothstep(0.55 * drift, 0.55 * drift + 0.45, snow_k);
            let cover = winter.amounts.z * up * lying;
            col = mix(col, winter.snow.rgb, cover);
            pbr_input.material.perceptual_roughness = mix(pbr_input.material.perceptual_roughness, 1.0, cover);
            // snow lies in a soft blanket whose top looks more to the sky than the roof under
            // it does: without this the slope turned from a low winter sun is slate-dark in
            // its own shade and does not read as snow at all
            pbr_input.N = normalize(mix(pbr_input.N, vec3<f32>(0.0, 1.0, 0.0), 0.55 * cover));
        } else if (mark > 0.375) {
            // what grows has lost its green: the more of it there was, the barer it is now
            let green = clamp((col.g - max(col.r, col.b)) / max(col.g, 1e-4), 0.0, 1.0);
            let gone = mix(winter.winter_grass.rgb, winter.bare_earth.rgb, smoothstep(0.32, 0.5, green));
            col = mix(col, gone, 0.85 * frost_k * smoothstep(0.06, 0.2, green));
            col = mix(col, winter.frost.rgb, winter.amounts.w * frost_k * up * (0.55 + 0.45 * drift));
        } else if (mark > 0.125) {
            col = mix(col, winter.ice.rgb, 0.85 * up * smoothstep(0.25, 0.7, frost_k));
        }
    }
    pbr_input.material.base_color = vec4<f32>(col, 1.0);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
