//! Line charts for the panels. Bevy UI has no line to draw with, so a chart's
//! plot is painted into an image on the CPU, the way icons.rs paints the
//! icons, and the figures round it are ordinary text in the interface's font.
//!
//! One look for all of them: a paper ground, an ink line, a faint grid on
//! round years, the low and high figures at the left and the first and last
//! years underneath, and a marker at the year on screen. Years after the
//! marker (a replay knows them, and so does a game that has been wound back)
//! are drawn fainter. A second series, with a scale of its own on the right,
//! is in the accent colour: a thin line over a pale fill, so it sits behind
//! the ink line as a quantity rather than fighting it as a second scribble.
//!
//! `plot` is plain arithmetic on pixels, so it can be tested without a window;
//! `spawn` wraps it in nodes. A chart is drawn when its panel is rebuilt, which
//! is when the year or what is chosen changes, never per frame.

use bevy::asset::RenderAssetUsages;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::ui::{text, MUTED};

/// Image pixels per pixel of the interface, so lines stay smooth on a dense screen.
const SS: f32 = 2.0;
/// Room at the side of the plot for the low and high figures.
const GUTTER: f32 = 48.0;

const GROUND: [f32; 3] = [246.0, 238.0, 222.0];
const GRID: [f32; 3] = [229.0, 219.0, 198.0];
const INK: [f32; 3] = [51.0, 43.0, 36.0];
const MARK: [f32; 3] = [120.0, 107.0, 92.0];

pub struct Chart<'a> {
    /// the room there is, in pixels of the interface: the plot and its gutters
    pub width: f32,
    /// height of the plot
    pub height: f32,
    /// one value per year from the first; NaN where there is none
    pub line: &'a [f32],
    /// a second line with a scale of its own, drawn in `accent`
    pub second: Option<&'a [f32]>,
    pub accent: [u8; 3],
    /// the year on screen, counted from the first
    pub mark: usize,
    /// how many years lie along the bottom
    pub years: usize,
    /// the calendar year of the first, so the grid can stand on round years
    pub first_year: i32,
}

impl Chart<'_> {
    /// Width of the plot itself, between the gutters.
    fn plot_width(&self) -> f32 {
        (self.width - GUTTER - if self.second.is_some() { GUTTER } else { 0.0 }).max(40.0)
    }
}

/// The painted plot, and the figures at the bottom and top of each line's scale.
pub struct Plot {
    pub w: usize,
    pub h: usize,
    /// sRGB with alpha, four bytes a pixel
    pub px: Vec<u8>,
    pub range: Option<(f32, f32)>,
    pub range2: Option<(f32, f32)>,
}

/// The lowest and highest of a series, which are the ends of its scale. A
/// series that does not move gets a little room either side so it draws as a
/// level line; one with no figures at all has no scale.
pub fn range(vals: &[f32]) -> Option<(f32, f32)> {
    let mut it = vals.iter().copied().filter(|v| v.is_finite());
    let first = it.next()?;
    let (lo, hi) = it.fold((first, first), |(lo, hi), v| (lo.min(v), hi.max(v)));
    let least = 0.05 * hi.abs().max(lo.abs());
    if hi - lo >= least && hi > lo {
        Some((lo, hi))
    } else if hi == 0.0 && lo == 0.0 {
        Some((0.0, 1.0))
    } else {
        let mid = 0.5 * (lo + hi);
        Some((mid - 0.5 * least, mid + 0.5 * least))
    }
}

struct Canvas {
    w: usize,
    h: usize,
    /// colour 0 to 255 and cover 0 to 1
    px: Vec<[f32; 4]>,
}

impl Canvas {
    /// Lay `col` over a pixel with cover `a`.
    fn over(&mut self, x: usize, y: usize, col: [f32; 3], a: f32) {
        let d = &mut self.px[y * self.w + x];
        for i in 0..3 {
            d[i] += (col[i] - d[i]) * a;
        }
    }

    /// A straight rule across or down the whole plot, `t` pixels thick, centred on `at`.
    fn rule(&mut self, down: bool, at: f32, t: f32, col: [f32; 3], alpha: f32) {
        let n = if down { self.w } else { self.h };
        for i in (at - t).floor().max(0.0) as usize..=((at + t).ceil() as usize).min(n.saturating_sub(1)) {
            let a = (0.5 * t + 0.5 - (i as f32 + 0.5 - at).abs()).clamp(0.0, 1.0) * alpha;
            if a <= 0.0 {
                continue;
            }
            for j in 0..if down { self.h } else { self.w } {
                let (x, y) = if down { (i, j) } else { (j, i) };
                self.over(x, y, col, a);
            }
        }
    }

    /// A line through `pts`, `half` pixels either side, with a soft edge. Each
    /// stretch only touches the pixels near it, so a long series stays cheap.
    fn stroke(&mut self, pts: &[(f32, f32)], half: f32, col: [f32; 3]) {
        if pts.is_empty() {
            return;
        }
        let mut cover = vec![0.0f32; self.w * self.h];
        let single = [pts[0], pts[0]];
        let pairs: Vec<&[(f32, f32)]> = if pts.len() == 1 { vec![&single[..]] } else { pts.windows(2).collect() };
        for s in pairs {
            let (a, b) = (s[0], s[1]);
            let reach = half + 1.0;
            let x0 = (a.0.min(b.0) - reach).floor().max(0.0) as usize;
            let x1 = ((a.0.max(b.0) + reach).ceil() as usize).min(self.w.saturating_sub(1));
            let y0 = (a.1.min(b.1) - reach).floor().max(0.0) as usize;
            let y1 = ((a.1.max(b.1) + reach).ceil() as usize).min(self.h.saturating_sub(1));
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let len2 = (dx * dx + dy * dy).max(1e-9);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let (px, py) = (x as f32 + 0.5 - a.0, y as f32 + 0.5 - a.1);
                    let t = ((px * dx + py * dy) / len2).clamp(0.0, 1.0);
                    let d = ((px - dx * t).powi(2) + (py - dy * t).powi(2)).sqrt();
                    let c = &mut cover[y * self.w + x];
                    *c = c.max((half + 0.5 - d).clamp(0.0, 1.0));
                }
            }
        }
        for y in 0..self.h {
            for x in 0..self.w {
                let a = cover[y * self.w + x];
                if a > 0.0 {
                    self.over(x, y, col, a);
                }
            }
        }
    }

    /// A pale fill from a line through `pts` (left to right) down to `floor`.
    fn fill_under(&mut self, pts: &[(f32, f32)], floor: f32, col: [f32; 3], alpha: f32) {
        for s in pts.windows(2) {
            let (a, b) = (s[0], s[1]);
            let x0 = a.0.round().max(0.0) as usize;
            let x1 = (b.0.round() as usize).min(self.w);
            for x in x0..x1 {
                let t = ((x as f32 + 0.5 - a.0) / (b.0 - a.0).max(1e-6)).clamp(0.0, 1.0);
                let top = a.1 + (b.1 - a.1) * t;
                for y in top.floor().max(0.0) as usize..(floor.round() as usize).min(self.h) {
                    // a soft upper edge, so the fill does not saw at the line
                    let k = (y as f32 + 1.0 - top).clamp(0.0, 1.0);
                    self.over(x, y, col, alpha * k);
                }
            }
        }
    }

    /// A filled dot with a ring of the ground round it, so it stands off the line.
    fn dot(&mut self, at: (f32, f32), r: f32, col: [f32; 3]) {
        for (rad, c) in [(r + 1.4 * SS, GROUND), (r, col)] {
            let x0 = (at.0 - rad - 1.0).floor().max(0.0) as usize;
            let x1 = ((at.0 + rad + 1.0).ceil() as usize).min(self.w.saturating_sub(1));
            let y0 = (at.1 - rad - 1.0).floor().max(0.0) as usize;
            let y1 = ((at.1 + rad + 1.0).ceil() as usize).min(self.h.saturating_sub(1));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let d = ((x as f32 + 0.5 - at.0).powi(2) + (y as f32 + 0.5 - at.1).powi(2)).sqrt();
                    let a = (rad + 0.5 - d).clamp(0.0, 1.0);
                    if a > 0.0 {
                        self.over(x, y, c, a);
                    }
                }
            }
        }
    }
}

/// A colour a share `k` of the way from the ground to `col`.
fn faint(col: [f32; 3], k: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| GROUND[i] + (col[i] - GROUND[i]) * k)
}

/// Paint the plot of a chart.
pub fn plot(c: &Chart) -> Plot {
    let w = (c.plot_width() * SS).round().max(8.0) as usize;
    let h = (c.height * SS).round().max(8.0) as usize;
    let mut cv = Canvas { w, h, px: vec![[GROUND[0], GROUND[1], GROUND[2], 1.0]; w * h] };
    // the paper has rounded corners
    let rad = 7.0 * SS;
    for y in 0..h {
        for x in 0..w {
            let (qx, qy) = ((x as f32 + 0.5 - w as f32 * 0.5).abs() - w as f32 * 0.5 + rad, (y as f32 + 0.5 - h as f32 * 0.5).abs() - h as f32 * 0.5 + rad);
            let d = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - rad;
            cv.px[y * w + x][3] = (0.5 - d).clamp(0.0, 1.0);
        }
    }
    // the lines keep clear of the edge by their own width and a little more
    let (pad_x, pad_y) = (6.0 * SS, 7.0 * SS);
    let last = c.years.max(2) - 1;
    let x_of = |i: usize| pad_x + i as f32 / last as f32 * (w as f32 - 2.0 * pad_x);
    let y_of = |v: f32, (lo, hi): (f32, f32)| h as f32 - pad_y - (v - lo) / (hi - lo) * (h as f32 - 2.0 * pad_y);

    // the grid: the low and high levels, the quarters between, and the round years
    for j in 0..=4 {
        cv.rule(false, pad_y + j as f32 / 4.0 * (h as f32 - 2.0 * pad_y), SS * 0.5, GRID, if j == 0 || j == 4 { 1.0 } else { 0.6 });
    }
    let every = if c.years > 60 { 25 } else if c.years > 20 { 10 } else { 5 };
    for i in 0..c.years {
        if (c.first_year + i as i32) % every == 0 && i > 0 && i < last {
            cv.rule(true, x_of(i), SS * 0.5, GRID, 0.6);
        }
    }
    let mark = c.mark.min(last);
    cv.rule(true, x_of(mark), SS * 0.5, MARK, 0.75);

    let accent = [c.accent[0] as f32, c.accent[1] as f32, c.accent[2] as f32];
    let range2 = c.second.and_then(range);
    let range1 = range(c.line);
    let mut dots = Vec::new();
    // the second line first, so the ink lies over it where they cross
    for (vals, rg, col, half, filled) in [(c.second, range2, accent, 0.7 * SS, true), (Some(c.line), range1, INK, 0.9 * SS, false)] {
        let (Some(vals), Some(rg)) = (vals, rg) else { continue };
        let n = vals.len().min(c.years);
        // each unbroken run of figures, split at the marker: what is still to come is fainter
        let mut i = 0;
        while i < n {
            if !vals[i].is_finite() {
                i += 1;
                continue;
            }
            let start = i;
            while i < n && vals[i].is_finite() {
                i += 1;
            }
            let pts: Vec<(f32, f32)> = (start..i).map(|j| (x_of(j), y_of(vals[j], rg))).collect();
            let cut = (mark + 1).saturating_sub(start).min(pts.len());
            if filled {
                // the second series: a quantity, filled down to the foot of the plot
                let floor = h as f32 - pad_y;
                cv.fill_under(&pts[..cut], floor, col, 0.2);
                cv.fill_under(&pts[cut.saturating_sub(1)..], floor, col, 0.09);
            }
            if cut < pts.len() {
                cv.stroke(&pts[cut.saturating_sub(1)..], half, faint(col, 0.42));
            }
            if cut > 0 {
                cv.stroke(&pts[..cut], half, col);
            }
        }
        if let Some(v) = vals.get(mark).filter(|v| v.is_finite()) {
            dots.push(((x_of(mark), y_of(*v, rg)), col));
        }
    }
    for (at, col) in dots {
        cv.dot(at, 2.6 * SS, col);
    }

    let mut px = Vec::with_capacity(w * h * 4);
    for p in &cv.px {
        px.extend_from_slice(&[p[0].round() as u8, p[1].round() as u8, p[2].round() as u8, (p[3] * 255.0).round() as u8]);
    }
    Plot { w, h, px, range: range1, range2 }
}

fn image(p: &Plot) -> Image {
    let mut img = Image::new(
        Extent3d { width: p.w as u32, height: p.h as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        p.px.clone(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::linear();
    img
}

/// Spawn a chart: its plot, the low and high figures of each line beside it
/// (written by `figure`, and by `figure2` for the second line) and the first
/// and last years underneath.
pub fn spawn(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, c: &Chart, figure: &dyn Fn(f32) -> String, figure2: &dyn Fn(f32) -> String) {
    let pl = plot(c);
    let handle = images.add(image(&pl));
    let pw = c.plot_width();
    let accent = Color::srgb_u8(c.accent[0], c.accent[1], c.accent[2]);
    // the figures at the ends of a scale: the high one level with the top of the plot, the low one with the bottom
    let ends = |row: &mut ChildSpawnerCommands, rg: Option<(f32, f32)>, fig: &dyn Fn(f32) -> String, col: Color, align: AlignItems| {
        row.spawn(Node { width: Val::Px(GUTTER - 6.0), height: Val::Px(c.height), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, align_items: align, flex_shrink: 0.0, ..default() }).with_children(|col_node| {
            if let Some((lo, hi)) = rg {
                col_node.spawn((text(f, &fig(hi), 11.5, col), TextLayout::default().with_no_wrap()));
                col_node.spawn((text(f, &fig(lo), 11.5, col), TextLayout::default().with_no_wrap()));
            }
        });
    };
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(1.0), ..default() }).with_children(|col| {
        col.spawn(Node { column_gap: Val::Px(6.0), ..default() }).with_children(|row| {
            ends(row, pl.range, figure, MUTED, AlignItems::End);
            row.spawn((ImageNode { image: handle, ..default() }, Node { width: Val::Px(pw), height: Val::Px(c.height), flex_shrink: 0.0, ..default() }));
            if c.second.is_some() {
                ends(row, pl.range2, figure2, accent, AlignItems::Start);
            }
        });
        col.spawn(Node { width: Val::Px(pw), margin: UiRect::left(Val::Px(GUTTER)), padding: UiRect::horizontal(Val::Px(2.0)), justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|row| {
            row.spawn(text(f, &format!("{}", c.first_year), 11.5, MUTED));
            row.spawn(text(f, &format!("{}", c.first_year + c.years.max(1) as i32 - 1), 11.5, MUTED));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart(line: &[f32]) -> Chart<'_> {
        Chart { width: 248.0, height: 60.0, line, second: None, accent: [178, 58, 58], mark: 5, years: line.len(), first_year: 1750 }
    }

    /// How dark a pixel is: paper is light, ink is dark.
    fn light(p: &Plot, x: f32, y: f32) -> f32 {
        let i = (y as usize * p.w + x as usize) * 4;
        (p.px[i] as f32 + p.px[i + 1] as f32 + p.px[i + 2] as f32) / 3.0
    }

    #[test]
    fn a_rising_line_runs_from_bottom_left_to_top_right() {
        let vals: Vec<f32> = (0..11).map(|i| i as f32).collect();
        let p = plot(&chart(&vals));
        assert_eq!((p.w, p.h), (400, 120));
        assert_eq!(p.range, Some((0.0, 10.0)));
        let (pad_x, pad_y) = (6.0 * SS, 7.0 * SS);
        // the ink starts at the low figure on the left; years after the marker are fainter but still drawn
        assert!(light(&p, pad_x + 3.0, p.h as f32 - pad_y - 2.0) < 110.0, "no ink at the start");
        let end = light(&p, p.w as f32 - pad_x - 3.0, pad_y + 2.0);
        assert!(end < 215.0 && end > 110.0, "the years to come should be faint, got {end}");
        // and the corners away from the line are bare paper
        assert!(light(&p, 40.0, 30.0) > 225.0);
        assert!(light(&p, p.w as f32 - 40.0, p.h as f32 - 30.0) > 225.0);
    }

    #[test]
    fn the_marker_stands_on_the_year_on_screen() {
        let vals: Vec<f32> = (0..11).map(|i| i as f32).collect();
        let p = plot(&chart(&vals));
        // year 5 of 0..=10 is the middle of the plot; the rule there is darker than the paper beside it
        let x = p.w as f32 * 0.5;
        assert!(light(&p, x, 18.0) < light(&p, x + 24.0, 18.0) - 20.0);
        // a dot sits on the line at the marker, in ink
        assert!(light(&p, x, p.h as f32 * 0.5) < 90.0);
    }

    #[test]
    fn gaps_and_level_lines_are_drawn_without_fuss() {
        // nothing to draw: no scale, and the plot is bare
        let none = [f32::NAN; 11];
        let mut c = chart(&none);
        c.mark = 0;
        let bare = plot(&c);
        assert_eq!(bare.range, None);
        // a level series sits in the middle of a small range of its own
        let (lo, hi) = range(&[2.0, 2.0, 2.0]).unwrap();
        assert!((lo - 1.95).abs() < 1e-5 && (hi - 2.05).abs() < 1e-5);
        assert_eq!(range(&[0.0, 0.0]), Some((0.0, 1.0)));
        // a gap breaks the line: the missing years are as bare as a plot with no figures
        let gap = [1.0, 1.0, f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN, 1.0, 1.0];
        let mut c = chart(&gap);
        c.mark = 0;
        let p = plot(&c);
        let mid = p.h as f32 * 0.5;
        let x = p.w as f32 * 0.5 + 30.0;
        assert_eq!(light(&p, x, mid), light(&bare, x, mid));
        let x = 6.0 * SS + 10.0;
        assert!(light(&p, x, mid) < light(&bare, x, mid) - 40.0);
    }

    #[test]
    fn a_second_line_gets_its_own_scale_and_gutter() {
        let (a, b): (Vec<f32>, Vec<f32>) = (0..11).map(|i| (i as f32, 1000.0 - 50.0 * i as f32)).unzip();
        let mut c = chart(&a);
        c.second = Some(&b);
        let p = plot(&c);
        assert_eq!(p.w, 304);
        assert_eq!(p.range2, Some((500.0, 1000.0)));
        // the accent line starts high on the left: reddish there, not paper and not ink
        let i = ((7.0 * SS + 1.0) as usize * p.w + (6.0 * SS + 3.0) as usize) * 4;
        assert!(p.px[i] as i32 - p.px[i + 2] as i32 > 60, "expected the accent colour, got {:?}", &p.px[i..i + 3]);
    }
}
