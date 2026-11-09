//! Little icons for the interface, drawn at startup from signed distance
//! shapes, so there are no image files to ship and every icon shares one hand.
//!
//! Most are drawn in white and tinted where they are used (`ImageNode::color`);
//! the coin keeps its own gold.

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Resource, Clone)]
pub struct Icons {
    pub coin: Handle<Image>,
    pub works: Handle<Image>,
    pub land: Handle<Image>,
    pub people: Handle<Image>,
    pub family: Handle<Image>,
    pub holdings: Handle<Image>,
    pub markets: Handle<Image>,
    pub news: Handle<Image>,
}

const N: usize = 64;

struct Canvas {
    px: Vec<[f32; 4]>,
}

type Sdf = dyn Fn(f32, f32) -> f32;

impl Canvas {
    fn new() -> Canvas {
        Canvas { px: vec![[0.0; 4]; N * N] }
    }

    /// Paint where the shape's distance is below zero, with a one-pixel soft edge.
    fn fill(&mut self, sdf: &Sdf, col: [f32; 4]) {
        for y in 0..N {
            for x in 0..N {
                let (u, v) = ((x as f32 + 0.5) / N as f32, (y as f32 + 0.5) / N as f32);
                let a = (0.5 - sdf(u, v) * N as f32).clamp(0.0, 1.0) * col[3];
                if a <= 0.0 {
                    continue;
                }
                let d = &mut self.px[y * N + x];
                let out_a = a + d[3] * (1.0 - a);
                for i in 0..3 {
                    d[i] = (col[i] * a + d[i] * d[3] * (1.0 - a)) / out_a.max(1e-6);
                }
                d[3] = out_a;
            }
        }
    }

    fn image(self) -> Image {
        let mut bytes = Vec::with_capacity(N * N * 4);
        // colours are given in sRGB, which is what the texture holds
        for p in &self.px {
            for c in p {
                bytes.push((c.clamp(0.0, 1.0) * 255.0).round() as u8);
            }
        }
        let mut img = Image::new(
            Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: 1 },
            TextureDimension::D2,
            bytes,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        img.sampler = ImageSampler::linear();
        img
    }
}

fn rgb(h: u32) -> [f32; 4] {
    [((h >> 16) & 255) as f32 / 255.0, ((h >> 8) & 255) as f32 / 255.0, (h & 255) as f32 / 255.0, 1.0]
}

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

fn circle(cx: f32, cy: f32, r: f32) -> impl Fn(f32, f32) -> f32 {
    move |x, y| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r
}

fn rbox(cx: f32, cy: f32, hw: f32, hh: f32, rad: f32) -> impl Fn(f32, f32) -> f32 {
    move |x, y| {
        let (qx, qy) = ((x - cx).abs() - hw + rad, (y - cy).abs() - hh + rad);
        (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - rad
    }
}

/// A convex polygon, its points in either order round the edge.
fn poly(pts: Vec<(f32, f32)>) -> impl Fn(f32, f32) -> f32 {
    let area: f32 = (0..pts.len()).map(|i| {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        a.0 * b.1 - b.0 * a.1
    }).sum();
    let flip = if area < 0.0 { -1.0 } else { 1.0 };
    move |x, y| {
        let mut d = f32::MIN;
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            let (ex, ey) = (b.0 - a.0, b.1 - a.1);
            let len = (ex * ex + ey * ey).sqrt().max(1e-6);
            // outward normal
            let (nx, ny) = (flip * ey / len, -flip * ex / len);
            d = d.max((x - a.0) * nx + (y - a.1) * ny);
        }
        d
    }
}

fn ring(cx: f32, cy: f32, r: f32, t: f32) -> impl Fn(f32, f32) -> f32 {
    move |x, y| (((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r).abs() - t
}

fn coin() -> Image {
    let mut c = Canvas::new();
    c.fill(&circle(0.5, 0.53, 0.40), rgb(0x9a6a1c));
    c.fill(&circle(0.5, 0.5, 0.40), rgb(0xe8b84e));
    c.fill(&ring(0.5, 0.5, 0.29, 0.025), rgb(0xc48f2c));
    // a shine
    c.fill(&rbox(0.38, 0.34, 0.035, 0.09, 0.035), [1.0, 0.97, 0.85, 0.75]);
    c.image()
}

fn works() -> Image {
    let mut c = Canvas::new();
    // two saw-tooth roofs over a long shed, and a chimney with a puff
    c.fill(&rbox(0.68, 0.36, 0.06, 0.2, 0.02), WHITE);
    c.fill(&rbox(0.5, 0.7, 0.36, 0.17, 0.04), WHITE);
    c.fill(&poly(vec![(0.14, 0.55), (0.14, 0.38), (0.36, 0.55)]), WHITE);
    c.fill(&poly(vec![(0.36, 0.55), (0.36, 0.38), (0.58, 0.55)]), WHITE);
    c.fill(&circle(0.71, 0.13, 0.07), [1.0, 1.0, 1.0, 0.8]);
    c.fill(&circle(0.82, 0.09, 0.05), [1.0, 1.0, 1.0, 0.6]);
    c.image()
}

fn land() -> Image {
    let mut c = Canvas::new();
    // a field with furrows, and a fence post at each corner
    let field = rbox(0.5, 0.55, 0.36, 0.3, 0.06);
    c.fill(&field, [1.0, 1.0, 1.0, 0.45]);
    for i in 0..4 {
        let y = 0.36 + i as f32 * 0.125;
        c.fill(&rbox(0.5, y, 0.3, 0.025, 0.02), WHITE);
    }
    c.image()
}

fn people() -> Image {
    let mut c = Canvas::new();
    c.fill(&circle(0.35, 0.33, 0.12), [1.0, 1.0, 1.0, 0.7]);
    c.fill(&rbox(0.35, 0.72, 0.19, 0.18, 0.15), [1.0, 1.0, 1.0, 0.7]);
    c.fill(&circle(0.63, 0.38, 0.13), WHITE);
    c.fill(&rbox(0.63, 0.78, 0.21, 0.19, 0.16), WHITE);
    c.image()
}

fn family() -> Image {
    let mut c = Canvas::new();
    // a shield
    c.fill(&rbox(0.5, 0.36, 0.3, 0.2, 0.05), WHITE);
    c.fill(&poly(vec![(0.2, 0.45), (0.5, 0.9), (0.8, 0.45)]), WHITE);
    c.image()
}

fn holdings() -> Image {
    let mut c = Canvas::new();
    // a house with a pennant
    c.fill(&rbox(0.46, 0.7, 0.27, 0.17, 0.03), WHITE);
    c.fill(&poly(vec![(0.13, 0.55), (0.46, 0.28), (0.79, 0.55)]), WHITE);
    c.fill(&rbox(0.8, 0.32, 0.018, 0.26, 0.01), WHITE);
    c.fill(&poly(vec![(0.82, 0.08), (0.82, 0.26), (0.98, 0.17)]), WHITE);
    c.image()
}

fn markets() -> Image {
    let mut c = Canvas::new();
    for (i, h) in [0.22, 0.36, 0.5].iter().enumerate() {
        let x = 0.25 + i as f32 * 0.25;
        c.fill(&rbox(x, 0.86 - h * 0.5 - 0.02, 0.08, h * 0.5, 0.03), WHITE);
    }
    c.fill(&rbox(0.5, 0.88, 0.4, 0.02, 0.01), [1.0, 1.0, 1.0, 0.7]);
    c.image()
}

fn news() -> Image {
    let mut c = Canvas::new();
    c.fill(&rbox(0.5, 0.52, 0.34, 0.36, 0.06), [1.0, 1.0, 1.0, 0.45]);
    c.fill(&rbox(0.5, 0.3, 0.24, 0.05, 0.02), WHITE);
    for i in 0..3 {
        c.fill(&rbox(0.5, 0.48 + i as f32 * 0.13, 0.24, 0.025, 0.02), WHITE);
    }
    c.image()
}

pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut add = |img: Image| images.add(img);
    let icons = Icons {
        coin: add(coin()),
        works: add(works()),
        land: add(land()),
        people: add(people()),
        family: add(family()),
        holdings: add(holdings()),
        markets: add(markets()),
        news: add(news()),
    };
    commands.insert_resource(icons);
}

/// An icon node of the given size, tinted.
pub fn icon(handle: &Handle<Image>, size: f32, tint: Color) -> impl Bundle {
    (ImageNode { image: handle.clone(), color: tint, ..default() }, Node { width: Val::Px(size), height: Val::Px(size), flex_shrink: 0.0, ..default() })
}
