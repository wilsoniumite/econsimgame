//! The lab's sheets of the building forms: every form at one scale, the
//! years across and the storeys up, today's houses in the first column; and
//! one sheet a kind (dwellings, offices, sheds), close to.

use glam::Vec3;

use super::{form, Form, Shape, Spec, Use, FORMS};
use crate::hash::Seed;
use crate::kit::Detail;
use crate::palette::Palette;
use crate::sheets::Cell;
use crate::site::{Model, Site};

/// The ages across the sheet of every form: the houses as drawn, then the
/// four ages of the twentieth century and the one after.
const AGES: [(i32, i32, &str); 6] = [(1600, 1894, "today's houses"), (1700, 1918, "to 1918"), (1919, 1945, "1919 to 1945"), (1946, 1969, "1946 to 1969"), (1970, 1994, "1970 to 1994"), (1995, 2050, "1995 to 2050")];

/// The heights up the sheet, tallest first (the top row is the furthest back).
const HEIGHTS: [(u8, u8); 6] = [(23, 40), (14, 22), (8, 13), (5, 7), (3, 4), (1, 2)];

/// The year a form is shown as built in: the middle of its years, or of
/// the years in which its builders know all it needs.
fn shown_year(s: &Spec) -> i32 {
    let first = s.needs.iter().map(|k| k.year()).fold(s.years.0, i32::max).min(s.years.1);
    (first + s.years.1) / 2
}

/// The storeys a form is shown with on the sheet of every form: the middle of what it stands.
fn shown_storeys(s: &Spec) -> u8 {
    (s.storeys.0 + s.storeys.1).div_ceil(2)
}

/// Several forms side by side in one cell, a little apart, each on its own ground.
fn side_by_side(m: &mut Model, site: &Site, forms: &[Form], pal: &Palette) {
    let gap = 0.012;
    let total: f32 = forms.iter().map(|f| f.ground.0).sum::<f32>() + gap * forms.len().saturating_sub(1) as f32;
    let mut x = -total * 0.5;
    for f in forms {
        let at = Site::level(site.origin + Vec3::new(x + f.ground.0 * 0.5, 0.0, 0.0), site.yaw);
        form(m, &at, f, pal);
        x += f.ground.0 + gap;
    }
}

/// Every form at one scale: a row for each band of storeys, tallest at the
/// back, and across it a cell for each age that has a form of that height,
/// today's houses first; each form in the cell of the middle of its years
/// and of its storeys.
pub(crate) fn every_form(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    let mut rows = Vec::new();
    for &(lo, hi) in &HEIGHTS {
        let mut row = Vec::new();
        for &(y0, y1, age) in &AGES {
            let here: Vec<&Spec> = FORMS
                .iter()
                .filter(|s| {
                    let (y, n) = (shown_year(s), shown_storeys(s));
                    let today = s.source == super::Source::Today;
                    n >= lo && n <= hi && if y0 == 1600 { today } else { !today && y >= y0 && y <= y1 }
                })
                .collect();
            if here.is_empty() {
                continue;
            }
            let names: Vec<&str> = here.iter().map(|s| s.short).collect();
            let mut line = names.join(", ");
            if line.chars().count() > 46 {
                line = format!("{}, {} and {} more", names[0], names[1], names.len() - 2);
            }
            let head = if row.is_empty() { format!("{lo} to {hi} storeys: {age}") } else { age.to_string() };
            let forms: Vec<Form> = here.iter().enumerate().map(|(i, s)| Form { detail, ..Form::new(s.shape, shown_storeys(s), shown_year(s), Seed::of(7 + i as u32)) }).collect();
            row.push(Cell::new(format!("{head}\n{line}"), move |m, site| side_by_side(m, site, &forms, pal)));
        }
        if !row.is_empty() {
            rows.push(row);
        }
    }
    rows
}

/// One kind of form close to: a row for each form, built at its fewest
/// storeys in its first year, between, and at its most in its last; a
/// cottage of 1840 at the head of the first row for the scale.
fn of_kind(pal: &Palette, detail: Detail, use_: Use) -> Vec<Vec<Cell>> {
    let mut rows: Vec<Vec<Cell>> = Vec::new();
    for (i, s) in FORMS.iter().filter(|s| s.use_ == use_ && s.source != super::Source::Today).enumerate() {
        let first = s.needs.iter().map(|k| k.year()).fold(s.years.0, i32::max).min(s.years.1);
        let mid = (s.storeys.0 + s.storeys.1).div_ceil(2);
        let mut row = Vec::new();
        if i == 0 {
            let f = Form { detail, ..Form::new(Shape::Cottage, 2, 1840, Seed::of(3)) };
            row.push(Cell::new("a cottage of 1840\nfor the scale", move |m, site| form(m, site, &f, pal)));
        }
        let mut seen: Vec<(i32, u8)> = Vec::new();
        for (j, (year, storeys)) in [(first, s.storeys.0), ((first + s.years.1) / 2, mid), (s.years.1, s.storeys.1)].into_iter().enumerate() {
            if seen.contains(&(year, storeys)) || (j == 1 && (storeys == s.storeys.0 || storeys == s.storeys.1) && s.storeys.0 != s.storeys.1) {
                continue;
            }
            seen.push((year, storeys));
            let f = Form { detail, ..Form::new(s.shape, storeys, year, Seed::of(11 + i as u32 * 3 + j as u32)) };
            row.push(Cell::new(format!("{}\n{year}, {storeys} storeys", s.short), move |m, site| form(m, site, &f, pal)));
        }
        rows.push(row);
    }
    rows
}

pub(crate) fn dwellings(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    of_kind(pal, detail, Use::Dwellings)
}

pub(crate) fn offices(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    of_kind(pal, detail, Use::Offices)
}

pub(crate) fn sheds(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    of_kind(pal, detail, Use::Sheds)
}
