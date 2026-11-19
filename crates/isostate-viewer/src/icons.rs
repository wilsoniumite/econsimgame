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
    pub map: Handle<Image>,
    /// the sandbox's Hand tab
    pub pen: Handle<Image>,
    /// the sandbox's Rules tab
    pub scales: Handle<Image>,
    /// the tray's tools (`sandbox::tool_chip`): Look, Harvest
    pub eye: Handle<Image>,
    pub sheaf: Handle<Image>,
    /// tools still to come, drawn now so that their stages need not come here:
    /// an idea (the lamp), a town (the market cross), a road and a railway
    /// (works is `works`)
    #[allow(dead_code)]
    pub lamp: Handle<Image>,
    #[allow(dead_code)]
    pub cross: Handle<Image>,
    #[allow(dead_code)]
    pub road: Handle<Image>,
    #[allow(dead_code)]
    pub rail: Handle<Image>,
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

    /// Cut the shape out of what is painted, with the same soft edge.
    fn erase(&mut self, sdf: &Sdf) {
        for y in 0..N {
            for x in 0..N {
                let (u, v) = ((x as f32 + 0.5) / N as f32, (y as f32 + 0.5) / N as f32);
                let a = (0.5 - sdf(u, v) * N as f32).clamp(0.0, 1.0);
                self.px[y * N + x][3] *= 1.0 - a;
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

/// A stroke of width `2 t` from (ax, ay) to (bx, by), with round ends.
fn segment(ax: f32, ay: f32, bx: f32, by: f32, t: f32) -> impl Fn(f32, f32) -> f32 {
    move |x, y| {
        let (px, py, ex, ey) = (x - ax, y - ay, bx - ax, by - ay);
        let h = ((px * ex + py * ey) / (ex * ex + ey * ey).max(1e-9)).clamp(0.0, 1.0);
        ((px - ex * h).powi(2) + (py - ey * h).powi(2)).sqrt() - t
    }
}

/// The lower half of a circle: a pan of the scales.
fn bowl(cx: f32, cy: f32, r: f32) -> impl Fn(f32, f32) -> f32 {
    let disc = circle(cx, cy, r);
    move |x, y| disc(x, y).max(cy - y)
}

fn pen() -> Image {
    let mut c = Canvas::new();
    // a dip pen held slanting: the holder, the nib to the lower left, and the line it has drawn
    c.fill(&poly(vec![(0.52, 0.30), (0.80, 0.02), (0.98, 0.20), (0.70, 0.48)]), WHITE);
    c.fill(&poly(vec![(0.50, 0.32), (0.68, 0.50), (0.22, 0.78)]), WHITE);
    // the slit down the nib
    c.erase(&segment(0.38, 0.62, 0.56, 0.44, 0.022));
    // and the break between holder and nib
    c.erase(&segment(0.47, 0.29, 0.73, 0.55, 0.018));
    c.fill(&rbox(0.36, 0.90, 0.30, 0.03, 0.03), [1.0, 1.0, 1.0, 0.6]);
    c.image()
}

fn scales() -> Image {
    let mut c = Canvas::new();
    // a pair of scales: the post on its foot, the beam, and a pan hung from each end
    c.fill(&rbox(0.5, 0.52, 0.035, 0.36, 0.02), WHITE);
    c.fill(&rbox(0.5, 0.90, 0.20, 0.04, 0.03), WHITE);
    c.fill(&circle(0.5, 0.15, 0.06), WHITE);
    c.fill(&rbox(0.5, 0.22, 0.40, 0.025, 0.02), WHITE);
    for x in [0.18, 0.82] {
        c.fill(&segment(x, 0.22, x - 0.12, 0.56, 0.014), WHITE);
        c.fill(&segment(x, 0.22, x + 0.12, 0.56, 0.014), WHITE);
        c.fill(&bowl(x, 0.56, 0.16), WHITE);
    }
    c.image()
}

/// An oval as wide as `rx` and as tall as `ry` either side of its centre (near
/// its edge the distance is close enough for a soft edge one pixel wide).
fn oval(cx: f32, cy: f32, rx: f32, ry: f32) -> impl Fn(f32, f32) -> f32 {
    move |x, y| (((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2)).sqrt().mul_add(rx.min(ry), -rx.min(ry))
}

/// The almond of an eye: what two circles, one above and one below, have in common.
fn almond(cx: f32, cy: f32, r: f32, off: f32) -> impl Fn(f32, f32) -> f32 {
    let (top, bottom) = (circle(cx, cy + off, r), circle(cx, cy - off, r));
    move |x, y| top(x, y).max(bottom(x, y))
}

fn eye() -> Image {
    let mut c = Canvas::new();
    // an almond, solid so that it reads at the size of a chip's word, the white of the eye
    // round the iris cut out of it, and the iris with a glint
    c.fill(&almond(0.5, 0.5, 0.58, 0.4), WHITE);
    c.erase(&circle(0.5, 0.5, 0.22));
    c.fill(&circle(0.5, 0.5, 0.13), WHITE);
    c.erase(&circle(0.455, 0.455, 0.04));
    c.image()
}

fn sheaf() -> Image {
    let mut c = Canvas::new();
    // a sheaf of corn: the stalks gathered at a band and spread above and below it, an ear on each
    for i in -2..=2 {
        let s = i as f32;
        c.fill(&segment(0.5 + s * 0.03, 0.6, 0.5 + s * 0.09, 0.3, 0.035), WHITE);
        c.fill(&segment(0.5 + s * 0.03, 0.6, 0.5 + s * 0.075, 0.93, 0.035), WHITE);
        c.fill(&oval(0.5 + s * 0.1, 0.19, 0.05, 0.12), WHITE);
    }
    // the band, with a gap either side of it
    c.erase(&rbox(0.5, 0.6, 0.24, 0.075, 0.03));
    c.fill(&rbox(0.5, 0.6, 0.19, 0.045, 0.03), WHITE);
    c.image()
}

fn lamp() -> Image {
    let mut c = Canvas::new();
    // an oil lamp: the bowl on its foot, the lid, a spout to the right and a handle to the left,
    // and a flame at the spout's tip with its light about it
    c.fill(&bowl(0.38, 0.64, 0.24), WHITE);
    c.fill(&rbox(0.38, 0.64, 0.26, 0.03, 0.02), WHITE);
    c.fill(&rbox(0.38, 0.9, 0.11, 0.03, 0.02), WHITE);
    c.fill(&segment(0.38, 0.84, 0.38, 0.9, 0.04), WHITE);
    c.fill(&poly(vec![(0.55, 0.64), (0.8, 0.56), (0.8, 0.62), (0.6, 0.77)]), WHITE);
    c.fill(&ring(0.1, 0.71, 0.075, 0.025), WHITE);
    // the flame stands on the spout's tip, and three strokes of light go out from it
    let (fx, fy) = (0.78, 0.42);
    c.fill(&circle(fx, fy, 0.075), WHITE);
    c.fill(&poly(vec![(fx - 0.075, fy), (fx + 0.075, fy), (fx, fy - 0.22)]), WHITE);
    for (dx, dy) in [(-1.0f32, -0.15f32), (-0.7, -0.72), (0.7, -0.72)] {
        let (ux, uy) = (dx / dx.hypot(dy), dy / dx.hypot(dy));
        let (cx, cy) = (fx, fy - 0.06);
        c.fill(&segment(cx + ux * 0.17, cy + uy * 0.17, cx + ux * 0.25, cy + uy * 0.25, 0.018), [1.0, 1.0, 1.0, 0.75]);
    }
    c.image()
}

fn market_cross() -> Image {
    let mut c = Canvas::new();
    // a market cross: three steps, the shaft, and the cross at its head
    for (i, hw) in [0.38, 0.28, 0.18].iter().enumerate() {
        c.fill(&rbox(0.5, 0.9 - i as f32 * 0.08, *hw, 0.04, 0.015), WHITE);
    }
    c.fill(&rbox(0.5, 0.42, 0.05, 0.27, 0.02), WHITE);
    c.fill(&rbox(0.5, 0.26, 0.18, 0.045, 0.02), WHITE);
    c.fill(&circle(0.5, 0.1, 0.05), WHITE);
    c.image()
}

fn road() -> Image {
    let mut c = Canvas::new();
    // a road going off to the skyline, and the marks down its middle
    c.fill(&poly(vec![(0.06, 0.95), (0.42, 0.1), (0.58, 0.1), (0.94, 0.95)]), WHITE);
    c.erase(&segment(0.5, 0.8, 0.5, 0.9, 0.035));
    c.erase(&segment(0.5, 0.52, 0.5, 0.62, 0.026));
    c.erase(&segment(0.5, 0.28, 0.5, 0.35, 0.018));
    c.image()
}

fn rail() -> Image {
    let mut c = Canvas::new();
    // a railway going off to the skyline: the sleepers, and the two rails over them
    for (y, hw) in [(0.88, 0.36), (0.66, 0.29), (0.47, 0.23), (0.3, 0.18), (0.16, 0.14)] {
        c.fill(&rbox(0.5, y, hw, 0.025 + 0.02 * y, 0.01), [1.0, 1.0, 1.0, 0.65]);
    }
    c.fill(&segment(0.24, 0.96, 0.42, 0.06, 0.03), WHITE);
    c.fill(&segment(0.76, 0.96, 0.58, 0.06, 0.03), WHITE);
    c.image()
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

fn map() -> Image {
    let mut c = Canvas::new();
    // a folded map: three leaves, the middle one tipped the other way, and a place marked on it
    c.fill(&poly(vec![(0.10, 0.24), (0.36, 0.16), (0.36, 0.78), (0.10, 0.86)]), WHITE);
    c.fill(&poly(vec![(0.385, 0.16), (0.615, 0.24), (0.615, 0.86), (0.385, 0.78)]), [1.0, 1.0, 1.0, 0.6]);
    c.fill(&poly(vec![(0.64, 0.24), (0.90, 0.16), (0.90, 0.78), (0.64, 0.86)]), WHITE);
    c.fill(&circle(0.5, 0.5, 0.07), WHITE);
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
        map: add(map()),
        pen: add(pen()),
        scales: add(scales()),
        eye: add(eye()),
        sheaf: add(sheaf()),
        lamp: add(lamp()),
        cross: add(market_cross()),
        road: add(road()),
        rail: add(rail()),
    };
    commands.insert_resource(icons);
}

/// An icon node of the given size, tinted.
pub fn icon(handle: &Handle<Image>, size: f32, tint: Color) -> impl Bundle {
    (ImageNode { image: handle.clone(), color: tint, ..default() }, Node { width: Val::Px(size), height: Val::Px(size), flex_shrink: 0.0, ..default() })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every icon by name, as `setup` draws them.
    fn all() -> Vec<(&'static str, Image)> {
        vec![
            ("coin", coin()), ("works", works()), ("land", land()), ("people", people()), ("family", family()), ("holdings", holdings()), ("markets", markets()), ("news", news()), ("map", map()),
            ("pen", pen()), ("scales", scales()), ("eye", eye()), ("sheaf", sheaf()), ("lamp", lamp()), ("cross", market_cross()), ("road", road()), ("rail", rail()),
        ]
    }

    /// The alpha of each pixel, 0 to 1.
    fn alpha(img: &Image) -> Vec<f32> {
        img.data.as_ref().unwrap().chunks(4).map(|p| p[3] as f32 / 255.0).collect()
    }

    #[test]
    fn every_icon_is_drawn_and_keeps_room_about_it() {
        for (name, img) in all() {
            let a = alpha(&img);
            let inked = a.iter().filter(|&&v| v > 0.5).count() as f32 / a.len() as f32;
            // something to see, and not a blot
            assert!((0.06..0.6).contains(&inked), "{name}: {:.0}% of it is inked", inked * 100.0);
            // nothing touches the edge of its square but by a pixel or two
            let edge: f32 = (0..N).flat_map(|i| [a[i], a[(N - 1) * N + i], a[i * N], a[i * N + N - 1]]).sum();
            assert!(edge < 6.0, "{name} runs off its square ({edge:.1} pixels of ink on the edge)");
        }
    }

    /// With ISOSTATE_ICON_SHEET naming a PNG, every icon is written to it four
    /// times its size, once in ink on the paper and once in white on the
    /// hand's ink, to be looked at.
    #[test]
    fn the_icons_can_be_laid_out_on_a_sheet() {
        let Some(path) = std::env::var_os("ISOSTATE_ICON_SHEET") else { return };
        let icons = all();
        let (scale, pad) = (4, 16);
        let tile = N * scale + 2 * pad;
        let (w, h) = (tile * icons.len().div_ceil(2), tile * 4);
        let paper = rgb(0xfbf6ec);
        let ink = rgb(0x332b24);
        let hand = rgb(0x213d73);
        let mut px = vec![0u8; w * h * 4];
        for (i, (_, img)) in icons.iter().enumerate() {
            let data = img.data.as_ref().unwrap();
            let (col, row) = (i / 2, (i % 2) * 2);
            for (r, (ground, tint)) in [(paper, ink), (hand, WHITE)].iter().enumerate() {
                let (x0, y0) = (col * tile, (row + r) * tile);
                for y in 0..tile {
                    for x in 0..tile {
                        let (sx, sy) = (x.wrapping_sub(pad) / scale, y.wrapping_sub(pad) / scale);
                        let s = if sx < N && sy < N { &data[(sy * N + sx) * 4..(sy * N + sx) * 4 + 4] } else { &[0, 0, 0, 0][..] };
                        let a = s[3] as f32 / 255.0;
                        let d = &mut px[((y0 + y) * w + x0 + x) * 4..((y0 + y) * w + x0 + x) * 4 + 4];
                        for k in 0..3 {
                            // the coin keeps its gold; the others take the tint
                            let c = if i == 0 { s[k] as f32 / 255.0 } else { tint[k] * s[k] as f32 / 255.0 };
                            d[k] = ((c * a + ground[k] * (1.0 - a)) * 255.0).round() as u8;
                        }
                        d[3] = 255;
                    }
                }
            }
        }
        let sheet = Image::new(Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::all());
        sheet.try_into_dynamic().unwrap().save(&path).unwrap();
    }
}
