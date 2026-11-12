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

/// Cells along each side of a terrain chunk.
const CHUNK: usize = 8;

/// The ground as one mesh per chunk of 8 x 8 cells, so the parts out of view
/// are not drawn. Chunks that are only open sea, far from any shore, are left
/// out: the sea's own surface covers them. The normal comes from the whole
/// heightfield, so chunks meet without seams. The first texture coordinate
/// carries how much each vertex lies on a river's valley floor.
pub fn terrain_chunks(g: &Ground) -> Vec<Mesh> {
    let step = 1.0 / VRES as f32;
    let (gw, gh) = (g.gw, g.gh);
    let normal = |i: usize, j: usize| {
        let hl = g.hgt[j * gw + i.saturating_sub(1)];
        let hr = g.hgt[j * gw + (i + 1).min(gw - 1)];
        let hu = g.hgt[j.saturating_sub(1) * gw + i];
        let hd = g.hgt[(j + 1).min(gh - 1) * gw + i];
        bevy::math::Vec3::new(hl - hr, 2.0 * step, hu - hd).normalize().to_array()
    };
    let mut out = Vec::new();
    let cv = CHUNK * VRES;
    for cj in 0..g.h.div_ceil(CHUNK) {
        for ci in 0..g.w.div_ceil(CHUNK) {
            let (i0, j0) = (ci * cv, cj * cv);
            let (i1, j1) = ((i0 + cv).min(gw - 1), (j0 + cv).min(gh - 1));
            // anything above the deep sea floor here?
            let mut shallow = false;
            for j in j0..=j1 {
                for i in i0..=i1 {
                    if g.hgt[j * gw + i] > -0.75 {
                        shallow = true;
                    }
                }
            }
            if !shallow {
                continue;
            }
            let (nw, nh) = (i1 - i0 + 1, j1 - j0 + 1);
            let mut pos = Vec::with_capacity(nw * nh);
            let mut nrm = Vec::with_capacity(nw * nh);
            let mut uv = Vec::with_capacity(nw * nh);
            for j in j0..=j1 {
                for i in i0..=i1 {
                    let k = j * gw + i;
                    pos.push([i as f32 * step, g.hgt[k], j as f32 * step]);
                    nrm.push(normal(i, j));
                    uv.push([g.floodplain[k], 0.0]);
                }
            }
            let mut idx = Vec::with_capacity((nw - 1) * (nh - 1) * 6);
            for j in 0..nh - 1 {
                for i in 0..nw - 1 {
                    let a = (j * nw + i) as u32;
                    let b = a + 1;
                    let c = a + nw as u32;
                    let d = c + 1;
                    idx.extend_from_slice(&[a, c, b, b, c, d]);
                }
            }
            let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
            m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
            m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
            m.insert_indices(Indices::U32(idx));
            out.push(m);
        }
    }
    out
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
    /// per parcel, this year: its class, how wooded it is and its town density (`year_parcels`)
    #[texture(101)]
    pub year: Handle<Image>,
    /// per parcel, fixed: the field depth and terrain class, the year of
    /// enclosure, the grass among its arable and its soil (`fixed_parcels`)
    #[texture(102)]
    pub fixed: Handle<Image>,
    /// the land's colours, from the look
    #[uniform(103)]
    pub paint: crate::look::LandUniform,
    /// per cell, the map overlay: a tint and the region it belongs to (`overlay.rs`)
    #[texture(104)]
    pub overlay: Handle<Image>,
    /// the rivers for the far view: the distance to the nearest and its width (`river_far`)
    #[texture(105)]
    #[sampler(106)]
    pub rivers: Handle<Image>,
    /// this year's wood and heath as two smooth fields, eight texels to a
    /// cell and read between them (`world::YearLand::field`, `land_field`)
    #[texture(107)]
    #[sampler(108)]
    pub field: Handle<Image>,
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
    /// the calendar year shown, for the open fields
    pub year: f32,
    /// how strongly the map overlay is washed over the land; 0 for none, and
    /// then the ground is drawn exactly as without one
    pub overlay: f32,
    /// how far the overlay's tints blend across the edge of a square, in squares
    pub overlay_soft: f32,
    /// 1 draws an ink line where the overlay's regions meet
    pub overlay_lines: f32,
    /// the first block of the parcel textures along each axis (`world::Land`)
    pub block_i0: i32,
    pub block_j0: i32,
    /// (task A) 1 in a run whose towns keep their ground (`Replay::kept`)
    pub town_kept: f32,
    /// 1 on the new land, whose shore is the first third of a metre above the
    /// tide stretched to a beach's height (`newland::units`): it is drawn far
    /// steeper than it is, and is not turned to shingle for that
    pub soft_shore: f32,
}

impl MaterialExtension for Patchwork {
    fn fragment_shader() -> ShaderRef {
        "embedded://isostate_viewer/terrain.wesl".into()
    }
}

/// Embed the ground and sea shaders in the binary and register the materials.
pub fn register(app: &mut App) {
    embedded_asset!(app, "terrain.wesl");
    embedded_asset!(app, "sea.wesl");
    app.add_plugins(MaterialPlugin::<GroundMaterial>::default())
        .add_plugins(MaterialPlugin::<SeaMaterial>::default())
        .add_systems(Startup, spawn_sea)
        .add_systems(Update, show_year);
}

/// Tell the ground shader which year is shown, for the open fields round the
/// towns, which are enclosed in a year of their own.
fn show_year(tl: Res<crate::Timeline>, data: Res<crate::Data>, sm: Option<Res<crate::SeasonMats>>, mut mats: ResMut<Assets<GroundMaterial>>, mut last: Local<Option<i32>>) {
    let Some(sm) = sm else { return };
    let year = data.r.y0 + (tl.t.floor() as i32).clamp(0, data.r.ny as i32 - 1);
    if *last == Some(year) {
        return;
    }
    if let Some(mut m) = mats.get_mut(&sm.terrain) {
        m.extension.params.year = year as f32;
        // (task A) a live game knows whether its towns keep their ground once its first year is in
        m.extension.params.town_kept = data.r.kept() as u8 as f32;
        *last = Some(year);
    }
}

// ------------------------------------------------------------------ the sea

/// The sea's surface: Bevy's StandardMaterial, blended, with a fragment shader
/// (sea.wesl) that reads the depth of the water from the ground's heightfield.
pub type SeaMaterial = ExtendedMaterial<StandardMaterial, Sea>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct Sea {
    #[uniform(100)]
    pub params: SeaParams,
    /// the ground's height at every vertex, one float each
    #[texture(101, sample_type = "float", filterable = false)]
    pub heights: Handle<Image>,
    /// the water's colours, from the look
    #[uniform(102)]
    pub paint: crate::look::SeaUniform,
}

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
pub struct SeaParams {
    pub grid_w: f32,
    pub grid_h: f32,
    pub res: f32,
    pub season: f32,
}

impl MaterialExtension for Sea {
    fn fragment_shader() -> ShaderRef {
        "embedded://isostate_viewer/sea.wesl".into()
    }
}

/// A sheet of water at sea level over the whole map and well beyond.
fn spawn_sea(mut commands: Commands, data: Res<crate::Data>, look: Res<crate::look::TheLook>, mut meshes: ResMut<Assets<Mesh>>, mut seas: ResMut<Assets<SeaMaterial>>, mut images: ResMut<Assets<Image>>) {
    let g = &data.g;
    let size = Extent3d { width: g.gw as u32, height: g.gh as u32, depth_or_array_layers: 1 };
    let bytes: Vec<u8> = g.hgt.iter().flat_map(|h| h.to_le_bytes()).collect();
    let mut img = Image::new(size, TextureDimension::D2, bytes, TextureFormat::R32Float, RenderAssetUsages::RENDER_WORLD);
    img.sampler = ImageSampler::nearest();
    let heights = images.add(img);
    let (w, h) = (g.w as f32, g.h as f32);
    let mat = seas.add(SeaMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: look.surfaces.sea,
            reflectance: look.surfaces.sea_reflectance,
            ..default()
        },
        extension: Sea { params: SeaParams { grid_w: g.gw as f32, grid_h: g.gh as f32, res: VRES as f32, season: 0.5 }, heights, paint: look.sea.uniform() },
    });
    commands.spawn((Mesh3d(meshes.add(plane(-4.0 * w, -4.0 * h, 5.0 * w, 5.0 * h, 0.0))), MeshMaterial3d(mat), bevy::light::NotShadowCaster));
}

pub fn patchwork_params(r: &Replay, g: &Ground, season: f32, debug: bool) -> PatchworkParams {
    PatchworkParams { season, width: r.w as f32, height: r.h as f32, debug: debug as u32, year: r.y0 as f32, overlay: 0.0, overlay_soft: 0.06, overlay_lines: 0.0, block_i0: g.land.bi0, block_j0: g.land.bj0, town_kept: r.kept() as u8 as f32, soft_shore: g.laid.is_some() as u8 as f32 }
}

/// The year's land for every parcel, as the shader reads it: its class
/// (`world::year_land`), how many times it is cut into fields this year
/// (`YearLand::depth`), the town density of its cell that scatter.rs builds
/// houses by, and what it is farmed as where neither wood nor heath stands
/// on it (`YearLand::under`).
pub fn year_parcels(r: &Replay, g: &Ground, k: usize, land: &crate::world::YearLand) -> Vec<u8> {
    let mut out = Vec::with_capacity(land.class.len() * 4);
    for (i, &class) in land.class.iter().enumerate() {
        let off = k * r.n + g.land.wcell[i] as usize;
        // (task A) on kept town ground, how many houses stand on the cell against the most a cell
        // comes to hold: its closes are gardens between the houses while it is young
        let kept = land.town.cells.get(&g.land.wcell[i]).filter(|_| class == crate::world::URBAN).map(|c| (c.houses as f32 / 119.0 * 255.0).min(255.0) as u8);
        let dense = kept.unwrap_or(if r.use_[off] == 10 { r.inten[off] } else { 115 });
        // (task A) a parcel of town ground is never farmed, so its last byte says which of
        // its eight closes are built up
        let last = if class == crate::world::URBAN { land.town.built[i] } else { land.under[i] };
        out.extend_from_slice(&[class, land.depth[i], dense, last]);
    }
    out
}

/// What does not change about each parcel: how finely it is cut into fields
/// (the low four bits) and the terrain class of its cell (0 sea, 1 lowland,
/// 2 hills, 3 mountains; the high four); the year its open fields are
/// enclosed, less 1750 (0 if they never were open); how much of its arable is
/// under grass; and the tone of its soil.
pub fn fixed_parcels(r: &Replay, g: &Ground) -> Vec<u8> {
    let l = &g.land;
    let mut out = Vec::with_capacity(l.at.len() * 4);
    for i in 0..l.at.len() {
        let open = if l.open_until[i] > 0 { (l.open_until[i] - 1750).clamp(1, 255) as u8 } else { 0 };
        out.extend_from_slice(&[l.depth[i] | (r.terrain[l.wcell[i] as usize] << 4), open, l.ley[i], l.soil[i]]);
    }
    out
}

/// The rivers as the far view draws them, sixteen texels to a cell and read
/// between texels: how far each spot is from the middle of the nearest river
/// (255 is a quarter of a cell or more), and that river's half width (255 is
/// 0.06 of a cell). From far off the shader rules a thread along the rivers
/// that keeps a pixel or two of width, as a map does.
pub fn river_far(g: &Ground) -> Image {
    use rayon::prelude::*;
    const RES: usize = 16;
    let (tw, th) = (g.w * RES, g.h * RES);
    let data: Vec<u8> = (0..tw * th)
        .into_par_iter()
        .flat_map_iter(|i| {
            let p = Vec2::new(((i % tw) as f32 + 0.5) / RES as f32, ((i / tw) as f32 + 0.5) / RES as f32);
            let (d, half) = g.water.nearest(p);
            [((d / 0.25).clamp(0.0, 1.0) * 255.0) as u8, ((half / 0.06).clamp(0.0, 1.0) * 255.0) as u8]
        })
        .collect();
    let size = Extent3d { width: tw as u32, height: th as u32, depth_or_array_layers: 1 };
    let mut img = Image::new(size, TextureDimension::D2, data, TextureFormat::Rg8Unorm, RenderAssetUsages::RENDER_WORLD);
    img.sampler = ImageSampler::linear();
    img
}

/// A texture with one texel per parcel, eight in a row for each block of the
/// field system, which the shader reads with textureLoad. Replacing its data
/// later writes into the same GPU texture.
pub fn parcel_image(g: &Ground, data: Vec<u8>) -> Image {
    let size = Extent3d { width: (g.land.bw * 8) as u32, height: g.land.bh as u32, depth_or_array_layers: 1 };
    let mut img = Image::new(size, TextureDimension::D2, data, TextureFormat::Rgba8Unorm, RenderAssetUsages::default());
    img.sampler = ImageSampler::nearest();
    img
}

/// The year's wood and heath (`world::YearLand::field`) as a texture the
/// shader reads between texels. Replacing its data later writes into the
/// same GPU texture.
pub fn field_image(g: &Ground, data: Vec<u8>) -> Image {
    let size = Extent3d { width: g.land.tw as u32, height: g.land.th as u32, depth_or_array_layers: 1 };
    let mut img = Image::new(size, TextureDimension::D2, data, TextureFormat::Rg8Unorm, RenderAssetUsages::default());
    img.sampler = ImageSampler::linear();
    img
}

/// A texture with one texel per map cell, which the shader reads with
/// textureLoad. Replacing its data later writes into the same GPU texture.
pub fn cell_image(r: &Replay, data: Vec<u8>, format: TextureFormat) -> Image {
    let size = Extent3d { width: r.w as u32, height: r.h as u32, depth_or_array_layers: 1 };
    let mut img = Image::new(size, TextureDimension::D2, data, format, RenderAssetUsages::default());
    img.sampler = ImageSampler::nearest();
    img
}
