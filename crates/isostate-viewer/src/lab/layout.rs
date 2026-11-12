//! Laying a contact sheet out under the game's camera: where each cell
//! stands on the lab's ground, where its label goes in the picture, and how
//! many rows fit a page. Maths only, so it can be tested without a renderer.
//!
//! A sheet says nothing about spacing. Each cell is as wide as what it holds
//! or as its label, whichever is wider; each row is as deep as its deepest
//! thing, with room behind for the height of its tallest (which leans back
//! over the ground in the picture) and room in front for the labels.
//!
//! The camera stands at the real distance, so things are the size in the
//! picture that they are in the game. From play distance and beyond that is a
//! few pixels, so the middle of the true picture is shown enlarged a whole
//! number of times, each pixel a square: nothing is drawn finer than the game
//! draws it.

use bevy::math::{Vec2, Vec3};
use isostate_gen::sheets::Cell;

use crate::shots;

/// One of the three standard distances a sheet is pictured from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shot {
    Close,
    Play,
    Far,
}

impl Shot {
    pub const ALL: [Shot; 3] = [Shot::Close, Shot::Play, Shot::Far];

    pub fn name(self) -> &'static str {
        match self {
            Shot::Close => "close",
            Shot::Play => "play",
            Shot::Far => "far",
        }
    }

    /// How far the camera stands from what it looks at.
    pub fn dist(self) -> f32 {
        match self {
            Shot::Close => shots::CLOSE_DIST,
            Shot::Play => shots::PLAY_DIST,
            Shot::Far => shots::FAR_DIST,
        }
    }

    /// What the distance is, in words, for the sheet's heading.
    pub fn words(self) -> &'static str {
        match self {
            Shot::Close => "as near as the game goes",
            Shot::Play => "as a new game opens",
            Shot::Far => "about half way out to the whole island",
        }
    }

    /// The most each pixel of the true picture is enlarged.
    fn most(self) -> u32 {
        match self {
            Shot::Close => 1,
            Shot::Play => 6,
            Shot::Far => 12,
        }
    }
}

/// The text size a label is set in, and the height of a line of it.
pub const LABEL_TEXT: f32 = 13.0;
const LABEL_LINE: f32 = 17.0;
/// The picture's edges kept clear: the sheet's heading is along the top.
const CLEAR_TOP: f32 = 96.0;
const CLEAR_BOTTOM: f32 = 12.0;
const CLEAR_SIDE: f32 = 16.0;

/// How wide and how tall a label is in the picture, near enough: the
/// interface font at its label size runs to about seven pixels a letter, and
/// the label has a little paper round it. A label may be of several lines.
pub fn label_size(s: &str) -> (f32, f32) {
    let longest = s.lines().map(|l| l.chars().count()).max().unwrap_or(0);
    (longest as f32 * 7.0 + 22.0, s.lines().count().max(1) as f32 * LABEL_LINE + 5.0)
}

/// The camera of one picture: the game's lens and tilt, looking at `target`
/// from `dist`, the middle of its picture enlarged `k` times.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub target: Vec3,
    pub dist: f32,
    pub size: (f32, f32),
    pub k: f32,
}

impl View {
    /// Pixels of the finished picture to a map unit, at the point looked at.
    pub fn px_per_unit(&self) -> f32 {
        self.size.1 / (2.0 * self.dist * (crate::FOV_DEG.to_radians() * 0.5).tan()) * self.k
    }

    /// Where a point of the lab is in the finished picture, in pixels from its top left.
    pub fn project(&self, p: Vec3) -> Vec2 {
        let pitch = shots::PLAY_PITCH.to_radians();
        // the camera stands south of the point it looks at, as the game's does at the start
        let back = Vec3::new(0.0, pitch.sin(), pitch.cos());
        let up = Vec3::new(0.0, pitch.cos(), -pitch.sin());
        let v = p - (self.target + back * self.dist);
        let depth = -v.dot(back);
        let tan = (crate::FOV_DEG.to_radians() * 0.5).tan();
        let (w, h) = self.size;
        let (x, y) = (v.x / (depth * tan * w / h), v.dot(up) / (depth * tan));
        // the true picture, then its middle enlarged
        let (px, py) = ((x + 1.0) * 0.5 * w, (1.0 - y) * 0.5 * h);
        Vec2::new((px - w * 0.5) * self.k + w * 0.5, (py - h * 0.5) * self.k + h * 0.5)
    }
}

/// Where one cell of a sheet stands on a page.
#[derive(Clone, Debug)]
pub struct Placed {
    pub row: usize,
    pub col: usize,
    /// where the cell's own origin goes on the lab's ground
    pub origin: Vec3,
    /// the point on the ground its label hangs under: the middle of its front
    pub label_at: Vec3,
    /// how wide its place in the row is, on the ground
    pub slot: f32,
}

/// One picture's worth of a sheet.
#[derive(Clone, Debug)]
pub struct Page {
    /// the cells on it, in the sheet's order, each where it stands
    pub placed: Vec<Placed>,
    pub view: View,
    /// something on it does not fit the picture even from play distance
    pub cut_off: bool,
}

/// The box round what a cell holds, and the size of its label.
#[derive(Clone)]
struct Bx {
    /// the cell's place in its sheet
    row: usize,
    col: usize,
    lo: Vec3,
    hi: Vec3,
    label: (f32, f32),
}

/// The pages of a sheet at one of the standard distances, for a picture of
/// `size`. Close up, the true picture is shown as it is (or enlarged `loupe`
/// times, if more than 1) and the rows go over as many pages as they need; a
/// row too wide for the picture is broken into lines that fit. From further
/// off the picture is enlarged as far as still shows the whole sheet on one
/// page, or, where it does not fit one even at its true size, laid over
/// pages as it is close up.
pub fn pages(cells: &[Vec<Cell>], shot: Shot, size: (u32, u32), loupe: u32) -> Vec<Page> {
    let boxes: Vec<Vec<Bx>> = cells
        .iter()
        .enumerate()
        .map(|(row, cells)| {
            cells
                .iter()
                .enumerate()
                .map(|(col, c)| {
                    let (lo, hi) = c.model.bounds().unwrap_or((Vec3::ZERO, Vec3::ZERO));
                    Bx { row, col, lo, hi, label: label_size(&c.label) }
                })
                .collect()
        })
        .collect();
    if boxes.is_empty() {
        return Vec::new();
    }
    let size = (size.0 as f32, size.1 as f32);
    if shot == Shot::Close {
        let k = loupe.max(1);
        let lines: Vec<Vec<Bx>> = boxes.into_iter().flat_map(|row| broken(row, shot.dist(), k, size)).collect();
        return paginate(&lines, shot.dist(), k, size);
    }
    for k in (1..=shot.most()).rev() {
        let (page, fits) = fit(&boxes, 0..boxes.len(), shot.dist(), k, size);
        if fits {
            return vec![page];
        }
    }
    // a picture too small for the sheet even at its true size (the labels do
    // not shrink with it): break the rows into lines and go over pages, as close up
    let lines: Vec<Vec<Bx>> = boxes.into_iter().flat_map(|row| broken(row, shot.dist(), 1, size)).collect();
    paginate(&lines, shot.dist(), 1, size)
}

/// A row as the lines it takes to fit the picture: itself if it fits, or
/// else its two halves, each broken again if it must be.
fn broken(row: Vec<Bx>, dist: f32, k: u32, size: (f32, f32)) -> Vec<Vec<Bx>> {
    if row.len() < 2 || fit(std::slice::from_ref(&row), 0..1, dist, k, size).1 {
        return vec![row];
    }
    let mut first = row;
    let second = first.split_off(first.len().div_ceil(2));
    let mut out = broken(first, dist, k, size);
    out.extend(broken(second, dist, k, size));
    out
}

/// As many rows to a page as fit, in order.
fn paginate(boxes: &[Vec<Bx>], dist: f32, k: u32, size: (f32, f32)) -> Vec<Page> {
    let mut out = Vec::new();
    let mut start = 0;
    while start < boxes.len() {
        let mut end = start + 1;
        while end < boxes.len() && fit(boxes, start..end + 1, dist, k, size).1 {
            end += 1;
        }
        let (mut page, mut fits) = fit(boxes, start..end, dist, k, size);
        // one thing too big for the picture from here (a scene, turned): stand back until the whole of it shows
        let mut back = dist;
        while !fits && back < shots::PLAY_DIST {
            back *= 1.05;
            (page, fits) = fit(boxes, start..end, back, k, size);
        }
        page.cut_off = !fits;
        out.push(page);
        start = end;
    }
    out
}

/// Lay these rows out on one page and point the camera at them; true if
/// everything is inside the picture.
fn fit(boxes: &[Vec<Bx>], rows: std::ops::Range<usize>, dist: f32, k: u32, size: (f32, f32)) -> (Page, bool) {
    // Things further back are drawn smaller, but a label is the same size
    // wherever it is, so the back row's labels need more ground than the
    // front row's. How much smaller the back row is depends on how deep the
    // page is, which depends on the spacing: begin by taking the back row to
    // be drawn full size and lay the page out again until that is true enough.
    let mut far = 1.0f32;
    let mut out = lay(boxes, rows.clone(), dist, k, size, far);
    for _ in 0..6 {
        let s = out.0.view.px_per_unit();
        let back = out.0.placed.first().map_or(Vec3::ZERO, |p| p.label_at);
        let drawn = (out.0.view.project(back + Vec3::X).x - out.0.view.project(back).x) / s;
        if drawn >= far - 0.004 {
            break;
        }
        far = drawn * 0.995;
        out = lay(boxes, rows.clone(), dist, k, size, far);
    }
    out
}

/// One laying out, taking the back row to be drawn `far` of full size.
fn lay(boxes: &[Vec<Bx>], rows: std::ops::Range<usize>, dist: f32, k: u32, size: (f32, f32), far: f32) -> (Page, bool) {
    let (w, h) = size;
    let pitch = shots::PLAY_PITCH.to_radians();
    let (sp, cp) = (pitch.sin(), pitch.cos());
    let mut view = View { target: Vec3::ZERO, dist, size, k: k as f32 };
    // pixels to a unit at the point looked at
    let s = view.px_per_unit();
    let half_depth = (h * 0.5) / (s * sp);
    // a cell is as wide as what it holds, or as its label where that is wider
    let slot = |b: &Bx| ((b.hi.x - b.lo.x) + 14.0 / s).max((b.label.0 + 10.0) / (s * far));
    // a sheet whose rows are all as long as each other is a grid: its columns line up
    let grid = boxes.iter().all(|r| r.len() == boxes[0].len());
    let columns: Vec<f32> = if grid { (0..boxes[0].len()).map(|c| boxes.iter().map(|r| slot(&r[c])).fold(0.0, f32::max)).collect() } else { Vec::new() };
    let row_width = |r: usize| if grid { columns.iter().sum::<f32>() } else { boxes[r].iter().map(slot).sum::<f32>() };
    let width = rows.clone().map(row_width).fold(0.0, f32::max);
    // how far in front of a row's things its labels begin, on the ground
    let gap = 5.0 / (s * sp);

    let mut placed = Vec::new();
    let mut z = 0.0f32;
    for r in rows.clone() {
        let tallest = boxes[r].iter().map(|b| b.hi.y).fold(0.0, f32::max);
        let deepest = boxes[r].iter().map(|b| b.hi.z - b.lo.z).fold(0.0, f32::max);
        let label = boxes[r].iter().map(|b| b.label.1).fold(0.0, f32::max);
        // a tall thing leans back over the ground in the picture: keep it off the labels of the row behind
        let back = z + tallest * cp / sp;
        let front = back + deepest;
        let mut x = (width - row_width(r)) * 0.5;
        for (c, b) in boxes[r].iter().enumerate() {
            let sw = if grid { columns[c] } else { slot(b) };
            let mid = x + sw * 0.5;
            placed.push(Placed {
                row: b.row,
                col: b.col,
                // the middle of the thing in the middle of its place, its front on the row's front line
                origin: Vec3::new(mid - (b.lo.x + b.hi.x) * 0.5, 0.0, front - b.hi.z),
                label_at: Vec3::new(mid, 0.0, front + gap),
                slot: sw,
            });
            x += sw;
        }
        // the labels' band in front of the row, and a little air
        z = front + (label + 14.0) / (s * sp * far);
    }

    // what the page covers of the picture, wherever the camera looks: the box
    // round what each cell holds, and each label
    let mid_x = width * 0.5;
    let all: Vec<&Bx> = rows.clone().flat_map(|r| boxes[r].iter()).collect();
    let covers = |view: &View| {
        let (mut left, mut top, mut right, mut bottom) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for (pl, b) in placed.iter().zip(&all) {
            for corner in 0..8 {
                let pick = |bit: usize, lo: f32, hi: f32| if corner >> bit & 1 == 0 { lo } else { hi };
                let p = view.project(pl.origin + Vec3::new(pick(0, b.lo.x, b.hi.x), pick(1, b.lo.y.max(0.0), b.hi.y), pick(2, b.lo.z, b.hi.z)));
                (left, top, right, bottom) = (left.min(p.x), top.min(p.y), right.max(p.x), bottom.max(p.y));
            }
            let at = view.project(pl.label_at);
            (left, right, bottom) = (left.min(at.x - b.label.0 * 0.5), right.max(at.x + b.label.0 * 0.5), bottom.max(at.y + b.label.1 + 4.0));
        }
        (left, top, right, bottom)
    };
    // look at the point that puts the page in the middle of the clear part of the picture
    let want = (CLEAR_TOP + h - CLEAR_BOTTOM) * 0.5;
    let (mut lo, mut hi) = (-4.0 * half_depth, z + 4.0 * half_depth);
    for _ in 0..40 {
        let tz = (lo + hi) * 0.5;
        view.target = Vec3::new(mid_x, 0.0, tz);
        let (_, top, _, bottom) = covers(&view);
        // looking further down the page moves it up the picture
        if (top + bottom) * 0.5 > want { lo = tz } else { hi = tz }
    }
    let (left, top, right, bottom) = covers(&view);
    let fits = top >= CLEAR_TOP - 0.5 && bottom <= h - CLEAR_BOTTOM + 0.5 && left >= CLEAR_SIDE && right <= w - CLEAR_SIDE;
    (Page { placed, view, cut_off: false }, fits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use isostate_gen::sheets::SHEETS;
    use isostate_gen::Palette;

    const SIZE: (u32, u32) = (1600, 1000);

    #[test]
    fn the_point_looked_at_is_the_middle_of_the_picture() {
        let v = View { target: Vec3::new(3.0, 0.0, 5.0), dist: 14.0, size: (1600.0, 1000.0), k: 1.0 };
        assert!((v.project(v.target) - Vec2::new(800.0, 500.0)).length() < 1e-3);
        // a unit to the east at the point looked at is px_per_unit to the right
        let east = v.project(v.target + Vec3::X);
        assert!((east.x - 800.0 - v.px_per_unit()).abs() < 0.01 && (east.y - 500.0).abs() < 0.01);
        // up is up, and south (toward the camera) is down the picture
        assert!(v.project(v.target + Vec3::Y).y < 500.0);
        assert!(v.project(v.target + Vec3::Z).y > 500.0);
        // enlarged four times, the same point is four times as far from the middle
        let big = View { k: 4.0, ..v };
        assert!((big.project(v.target + Vec3::X).x - 800.0 - 4.0 * v.px_per_unit()).abs() < 0.05);
    }

    #[test]
    fn the_projection_is_the_game_s_camera() {
        // the same point through the lens the game's camera is spawned with, and Bevy's own matrices
        use bevy::camera::CameraProjection;
        use bevy::prelude::*;
        let v = View { target: Vec3::new(1.0, 0.0, 2.0), dist: 14.0, size: (1600.0, 1000.0), k: 1.0 };
        let orbit = crate::Orbit { target: v.target, yaw: 0.0, pitch: shots::PLAY_PITCH.to_radians(), dist: v.dist };
        let cam = crate::orbit_transform(&orbit);
        let proj = PerspectiveProjection { fov: crate::FOV_DEG.to_radians(), aspect_ratio: 1.6, near: 0.05, far: 2000.0, ..default() }.get_clip_from_view();
        for p in [Vec3::new(0.3, 0.1, 1.2), Vec3::new(2.5, 0.0, 3.9), Vec3::new(1.0, 0.4, 2.0)] {
            let ndc = (proj * cam.to_matrix().inverse()).project_point3(p);
            let want = Vec2::new((ndc.x + 1.0) * 800.0, (1.0 - ndc.y) * 500.0);
            assert!((v.project(p) - want).length() < 0.01, "{p}: {} and {want}", v.project(p));
        }
    }

    #[test]
    fn every_sheet_fits_its_pictures() {
        let pal = Palette::standard();
        for sheet in SHEETS {
            let cells = crate::lab::cells_of(sheet, &pal, crate::lab::TURN, isostate_gen::houses::Detail::Near);
            let count: usize = cells.iter().map(|r| r.len()).sum();
            for shot in Shot::ALL {
                let pages = pages(&cells, shot, SIZE, 1);
                assert!(!pages.is_empty());
                // every cell is on exactly one page, in order
                assert_eq!(pages.iter().map(|p| p.placed.len()).sum::<usize>(), count, "{} {}", sheet.name, shot.name());
                let order: Vec<(usize, usize)> = pages.iter().flat_map(|p| p.placed.iter().map(|pl| (pl.row, pl.col))).collect();
                let sheet_order: Vec<(usize, usize)> = cells.iter().enumerate().flat_map(|(r, row)| (0..row.len()).map(move |c| (r, c))).collect();
                assert_eq!(order, sheet_order, "{} {}", sheet.name, shot.name());
                for (i, page) in pages.iter().enumerate() {
                    assert!(!page.cut_off, "{} {} page {} is cut off", sheet.name, shot.name(), i + 1);
                    for pl in &page.placed {
                        let cell = &cells[pl.row][pl.col];
                        // the label is inside the picture
                        let at = page.view.project(pl.label_at);
                        let (lw, lh) = label_size(&cell.label);
                        assert!(at.x - lw * 0.5 >= 0.0 && at.x + lw * 0.5 <= 1600.0 && at.y >= CLEAR_TOP && at.y + lh <= 1000.0, "{} {}: the label {:?} is at {at}", sheet.name, shot.name(), cell.label);
                        // and so is everything the cell holds
                        let (lo, hi) = cell.model.bounds().unwrap();
                        for corner in [Vec3::new(lo.x, lo.y.max(0.0), lo.z), Vec3::new(hi.x, hi.y, lo.z), Vec3::new(lo.x, hi.y, hi.z), Vec3::new(hi.x, 0.0, hi.z)] {
                            let p = page.view.project(pl.origin + corner);
                            assert!(p.x >= 0.0 && p.x <= 1600.0 && p.y >= 60.0 && p.y <= 1000.0, "{} {}: {:?} reaches {p}", sheet.name, shot.name(), cell.label);
                        }
                    }
                }
                // (the scale sheet has a row for every generator there is, and runs to several pages)
                if shot != Shot::Close && sheet.name != "scale" {
                    assert_eq!(pages.len(), 1, "{} {} does not fit one page", sheet.name, shot.name());
                }
            }
        }
    }

    #[test]
    fn a_small_picture_takes_more_pages_and_cuts_nothing_off() {
        // ten labels side by side are wider than a picture of 800, whatever the distance
        let pal = Palette::standard();
        for sheet in SHEETS {
            let cells = crate::lab::cells_of(sheet, &pal, crate::lab::TURN, isostate_gen::houses::Detail::Near);
            let count: usize = cells.iter().map(|r| r.len()).sum();
            for shot in Shot::ALL {
                let pages = pages(&cells, shot, (800, 600), 1);
                assert_eq!(pages.iter().map(|p| p.placed.len()).sum::<usize>(), count, "{} {}", sheet.name, shot.name());
                for (i, page) in pages.iter().enumerate() {
                    assert!(!page.cut_off, "{} {} page {} is cut off", sheet.name, shot.name(), i + 1);
                }
            }
        }
    }

    #[test]
    fn labels_in_a_row_do_not_overlap() {
        let pal = Palette::standard();
        for sheet in SHEETS {
            let cells = crate::lab::cells_of(sheet, &pal, crate::lab::TURN, isostate_gen::houses::Detail::Near);
            for shot in Shot::ALL {
                for page in pages(&cells, shot, SIZE, 1) {
                    for pair in page.placed.windows(2) {
                        // two cells side by side on one line of the page
                        if pair[0].row != pair[1].row || pair[0].label_at.z != pair[1].label_at.z {
                            continue;
                        }
                        let (a, b) = (page.view.project(pair[0].label_at), page.view.project(pair[1].label_at));
                        let need = (label_size(&cells[pair[0].row][pair[0].col].label).0 + label_size(&cells[pair[1].row][pair[1].col].label).0) * 0.5;
                        assert!(b.x - a.x >= need, "{} {}: {:?} and {:?} are {} apart and need {need}", sheet.name, shot.name(), cells[pair[0].row][pair[0].col].label, cells[pair[1].row][pair[1].col].label, b.x - a.x);
                    }
                }
            }
        }
    }

    #[test]
    fn from_further_off_the_picture_is_enlarged_a_whole_number_of_times() {
        let pal = Palette::standard();
        let cells = crate::lab::cells_of(isostate_gen::sheets::find("cottages").unwrap(), &pal, crate::lab::TURN, isostate_gen::houses::Detail::Near);
        let k = |shot| pages(&cells, shot, SIZE, 1)[0].view.k;
        assert_eq!(k(Shot::Close), 1.0);
        assert!(k(Shot::Play) >= 2.0 && k(Shot::Play).fract() == 0.0);
        assert!(k(Shot::Far) >= k(Shot::Play));
        // and close up, if asked
        assert_eq!(pages(&cells, Shot::Close, SIZE, 2)[0].view.k, 2.0);
    }
}
