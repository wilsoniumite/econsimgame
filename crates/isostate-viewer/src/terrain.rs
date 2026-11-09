//! The island itself: a heightfield mesh, the ground material that paints the
//! land patchwork per pixel (the fragment shader is terrain.wesl), and the sea.

use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::image::{Image, ImageSampler};
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;

use crate::replay::Replay;
use crate::world::{Ground, VRES};

pub fn terrain_mesh(g: &Ground) -> Mesh {
    let (gw, gh) = (g.gw, g.gh);
    let step = 1.0 / VRES as f32;
    let mut pos = Vec::with_capacity(gw * gh);
    let mut nrm = Vec::with_capacity(gw * gh);
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
    m.insert_indices(Indices::U32(idx));
    m
}

/// Height of the terrain mesh itself, two flat triangles per quad as
/// `terrain_mesh` builds it (inside a quad this differs a little from the
/// bilinear `world::height_at`).
pub fn mesh_height(g: &Ground, x: f32, z: f32) -> f32 {
    let gx = (x * VRES as f32).clamp(0.0, (g.gw - 1) as f32 - 0.001);
    let gy = (z * VRES as f32).clamp(0.0, (g.gh - 1) as f32 - 0.001);
    let (i, j) = (gx.floor() as usize, gy.floor() as usize);
    let (tx, ty) = (gx - i as f32, gy - j as f32);
    let k = j * g.gw + i;
    let (a, b, c, d) = (g.hgt[k], g.hgt[k + 1], g.hgt[k + g.gw], g.hgt[k + g.gw + 1]);
    if tx + ty <= 1.0 { a + (b - a) * tx + (c - a) * ty } else { d + (c - d) * (1.0 - tx) + (b - d) * (1.0 - ty) }
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

// ------------------------------------------------------------------ the ground material

/// The ground: Bevy's StandardMaterial for light, shadow, ambient occlusion and
/// fog, with a fragment shader that paints the land patchwork per pixel.
pub type GroundMaterial = ExtendedMaterial<StandardMaterial, Patchwork>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct Patchwork {
    #[uniform(100)]
    pub params: PatchworkParams,
    /// per cell, this year: mix_a, mix_b, mix_s and town density (`year_cells`)
    #[texture(101)]
    pub year: Handle<Image>,
    /// per cell, fixed: the terrain class (`fixed_cells`)
    #[texture(102)]
    pub fixed: Handle<Image>,
}

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
pub struct PatchworkParams {
    /// phase of the year: 0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn
    pub season: f32,
    /// map size in cells
    pub width: f32,
    pub height: f32,
    /// 1 draws flat, unlit class colours for checking against `world::class_at`
    pub debug: u32,
    /// `Ground::quant`, the 257 quantiles of the patch noise, four to a vector
    pub quant: [Vec4; 65],
}

impl MaterialExtension for Patchwork {
    fn fragment_shader() -> ShaderRef {
        "embedded://isostate_viewer/terrain.wesl".into()
    }
}

/// Embed the ground shader in the binary and register the material.
pub fn register(app: &mut App) {
    embedded_asset!(app, "terrain.wesl");
    app.add_plugins(MaterialPlugin::<GroundMaterial>::default());
}

pub fn patchwork_params(r: &Replay, g: &Ground, season: f32, debug: bool) -> PatchworkParams {
    let mut quant = [Vec4::ZERO; 65];
    for (i, &q) in g.quant.iter().enumerate() {
        quant[i / 4][i % 4] = q;
    }
    PatchworkParams { season, width: r.w as f32, height: r.h as f32, debug: debug as u32, quant }
}

/// The year's land mix for every cell, as the shader reads it: mix_a, mix_b,
/// mix_s, and the town density that scatter.rs builds houses by.
pub fn year_cells(r: &Replay, k: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(r.n * 4);
    for c in 0..r.n {
        let off = k * r.n + c;
        let dense = if r.use_[off] == 10 { r.inten[off] } else { 115 };
        out.extend_from_slice(&[r.mix_a[off], r.mix_b[off], r.mix_s[off], dense]);
    }
    out
}

/// The terrain class of every cell (0 sea, 1 lowland, 2 hills, 3 mountains).
pub fn fixed_cells(r: &Replay) -> Vec<u8> {
    r.terrain.clone()
}

/// A texture with one texel per map cell, which the shader reads with
/// textureLoad. Replacing its data later writes into the same GPU texture.
pub fn cell_image(r: &Replay, data: Vec<u8>, format: TextureFormat) -> Image {
    let size = Extent3d { width: r.w as u32, height: r.h as u32, depth_or_array_layers: 1 };
    let mut img = Image::new(size, TextureDimension::D2, data, format, RenderAssetUsages::default());
    img.sampler = ImageSampler::nearest();
    img
}
