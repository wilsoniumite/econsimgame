//! A works of a kind the generators do not know.

use super::kit::Ctx;

/// Plain sheds in a walled yard, more of them as it grows.
pub fn plain(c: &mut Ctx) {
    let (hw, hd, g, pal) = (c.hw, c.hd, c.g, c.pal);
    c.ground(*pal.ground.yard_setts, false);
    c.wall(c.yard_wall(), 0.008, 0.022 * g);
    let (wall, roof) = (c.masonry(false), c.roofing());
    let (len, dep, h) = ((0.062 * g).min(2.0 * hw - 0.02), 0.04 * g, 0.03 * g);
    let cols = c.fit(2.0 * hw - 0.016 + 0.012, len + 0.012, 4);
    let rows = c.fit(2.0 * hd * 0.7, dep + 0.012, 5);
    for j in 0..rows {
        for i in 0..cols {
            let x = (i as f32 - (cols as f32 - 1.0) * 0.5) * (len + 0.012);
            let s = c.shed(x, -hd + 0.008 + dep * 0.5 + j as f32 * (dep + 0.012), len, dep, h, false, wall, roof);
            c.windows(&s, 0, 1, false);
        }
    }
    let front = -hd + 0.008 + rows as f32 * (dep + 0.012);
    let d = hd - 0.012 - front;
    if d > 0.008 {
        c.stacks(0.0, front + d * 0.5, hw * 1.2, d * 0.8, c.fit(hw * 1.2, 0.02 * g, 10), c.fit(d * 0.8, 0.012 * g, 4), 0.006 * g, *pal.timber.planks);
    }
}
