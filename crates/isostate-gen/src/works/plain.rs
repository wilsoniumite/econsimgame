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
            // a cart's way in
            c.doors(&s, super::kit::Face::Front, -len * 0.13, len * 0.13, 0.0, h * 0.68, false, *pal.timber.door);
        }
    }
    let front = -hd + 0.008 + rows as f32 * (dep + 0.012);
    let d = hd - 0.012 - front;
    if d > 0.008 {
        c.stacks(0.0, front + d * 0.5, hw * 1.2, d * 0.8, c.fit(hw * 1.2, 0.02 * g, 10), c.fit(d * 0.8, 0.012 * g, 4), 0.006 * g, *pal.timber.planks);
    }
}

/// The process buildings of a works of a kind the generators do not know,
/// from its process alone (a part of a planned works: `site`). One making
/// shop, of storeys where the work is of many hands on light goods and a
/// hall where the stuff is heavy; a preparing shed for each good that comes
/// in beyond the first; where the process burns fuel, a boiler house and a
/// chimney; where it burns a great deal (a unit and a half of heat to a unit
/// of what it makes), hearth stacks on the shop; where it burns more still
/// (two and a half: a potbank, a glasshouse, an alkali works), kilns in
/// place of the boiler house: in a row before the shop, as many as its size
/// asks, or one beside it in a small yard; and where its plant is dear
/// and it was built late, a finishing shed under north lights. Every one of
/// these is read off the process's numbers, so two trades nobody has drawn
/// still look like two trades.
pub fn process(c: &mut Ctx, p: &super::site::Process) {
    use super::kit::Face;
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*if p.heat > 0.0 { pal.ground.yard_cinders } else { pal.ground.yard_setts }, false);
    c.wall(c.yard_wall(), 0.008, 0.022 * g);
    let (wall, roof) = (c.masonry(p.heat >= 1.0), c.roofing());
    let heavy = p.inputs.iter().any(|(good, _)| good.weight >= 1.5) || p.output.weight >= 1.2;
    let storeys = if heavy { 1 } else { (1.6 + p.labour.min(4.0) * 0.3 + size.max(0.4).log10()).round().clamp(1.0, 4.0) as usize };
    let h = if heavy { 0.032 * g } else { c.storeys(storeys) };
    // where it burns a great deal its fire is in kilns, and it has no boiler house
    let kilns = p.heat >= 2.5;
    let fired = p.heat > 0.0 && !kilns;
    // (a works that fires in kilns has a shallower shop, and the kilns before it; in a yard
    // too shallow for that, a kiln beside the shop)
    let dep = (if kilns { 0.026 } else { 0.036 } * g).min(2.0 * hd - 0.02);
    let before = 2.0 * hd - 0.016 - dep - 0.01;
    let kiln = (0.013 * g).min(if before >= 0.021 { (before - 0.003) * 0.5 } else { hd - 0.009 });
    let beside = kilns && before < 0.021;
    let side = if fired { 0.03 * g } else if beside { 2.0 * kiln + 0.005 } else { 0.0 };
    let prep = p.inputs.len().saturating_sub(1).min(3);
    let len = (2.0 * hw - 0.016 - side).min(0.11 * g * size.max(0.4).powf(0.2));
    let (x0, z0) = (-hw + 0.008 + len * 0.5, -hd + 0.008 + dep * 0.5);
    let shop = c.shed(x0, z0, len, dep, h, false, wall, roof);
    c.doors(&shop, Face::Front, -len * 0.5 + 0.006, -len * 0.5 + 0.006 + 0.017, 0.0, (h * 0.68).min(0.016), false, *pal.timber.door);
    if !heavy && storeys > 1 && len > 0.05 {
        c.hoist(&shop, Face::Front, len * 0.5 - 0.011, 1);
    }
    if fired {
        // the boiler house against the shop's end, and the chimney behind it
        let (bw, bd) = (0.022 * g, (0.026 * g).min(dep));
        let bx = x0 + len * 0.5 + 0.003 + bw * 0.5;
        if bx + bw * 0.5 <= hw - 0.006 {
            let b = c.shed(bx, z0 - dep * 0.5 + bd * 0.5, bd, bw, (0.026 * g).min(h), true, wall, roof);
            if !c.going_up() {
                c.glow(&b, Face::EndB, -0.004 * g, 0.004 * g, 0.0, 0.009 * g, true);
            }
            let tall = (0.07 + 0.03 * p.heat.min(2.0)) * g * size.max(1.0).powf(0.05);
            c.chimney(bx, (z0 - dep * 0.5 + bd + 0.012 * g).min(hd - 0.02), tall, 0.0062 * g, 0.6 + 0.3 * p.heat.min(2.0));
        } else {
            c.stack(x0 + len * 0.5 - 0.006, z0, h + dep * 0.32 + 0.012 * g, 0.006 * g, c.tone(*pal.brick.chimney), 0.5);
        }
        if p.heat >= 1.5 {
            // hearths along the back of the shop, each under its own stack
            let n = c.fit(len - 0.02, 0.03 * g, 5);
            for i in 0..n {
                let sx = -len * 0.5 + len * (i as f32 + 0.5) / n as f32;
                c.slab_stack(&shop, sx, -dep * 0.5 + 0.004, 0.009 * g, 0.006 * g, h + 0.02 * g, c.tone(*pal.brick.dark), 0.35);
            }
        }
    }
    let mut front = z0 + dep * 0.5 + 0.01;
    if kilns && kiln >= 0.006 {
        // the kilns, fired in turn: in a row before the shop, as many as the square root of
        // its size asks and its ground has room for; or the one beside it
        let pitch = 2.0 * kiln + 0.006 * g;
        let n = if beside { 1 } else { c.fit(2.0 * hw - 0.016, pitch, ((1.2 * size.max(0.4).sqrt()).ceil() as usize).max(1)) };
        let brick = c.tone(*pal.brick.red);
        for k in 0..n {
            let (x, z) = if beside { (hw - 0.008 - kiln, -hd + 0.008 + kiln) } else { (-hw + 0.008 + kiln + k as f32 * pitch, front + kiln) };
            let top = c.bottle_kiln(x, z, kiln, !c.derelict(), brick);
            if !c.going_up() && k % 2 == 0 {
                c.plume(top, 0.5);
            }
        }
        if !beside {
            front += 2.0 * kiln + 0.006;
        }
    }
    // a preparing shed for each further good that comes in, across the front of the yard
    let room = hd - 0.008 - front;
    if prep > 0 && room > 0.02 {
        let (pl, pd) = ((2.0 * hw - 0.016 - 0.006 * (prep as f32 - 1.0)) / prep as f32, room.min(0.026 * g));
        for i in 0..prep {
            c.shed(-hw + 0.008 + pl * 0.5 + i as f32 * (pl + 0.006), hd - 0.008 - pd * 0.5, pl.min(0.06 * g), pd, 0.02 * g, false, c.masonry(false), roof);
        }
    } else if p.capital >= 100.0 && c.late() && room > 0.024 {
        let wl = 2.0 * hw - 0.02;
        c.sawtooth(0.0, front + room * 0.5 - 0.002, wl, room - 0.006, 0.016 * g, ((wl / (0.018 * g)).floor() as usize).max(2), false, wall, roof);
    }
}
