//! The few `scipy.ndimage` routines the Python sim calls, on row-major (H, W) grids.
//!
//! Each follows scipy's algorithm closely enough to give the same numbers (the
//! running-sum uniform filter is reproduced step by step, because its rounding
//! leaks into firm agglomeration counts); `tests/ndimage.rs` checks them against
//! scipy outputs.

use std::collections::VecDeque;

/// Boundary handling of `uniform_filter`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// `mode="constant"`, `cval=0`: zeros outside the grid.
    Constant,
    /// `mode="reflect"` (scipy's default): `d c b a | a b c d | d c b a`.
    Reflect,
}

fn extend(line: &[f64], i: isize, mode: Mode) -> f64 {
    let n = line.len() as isize;
    if (0..n).contains(&i) {
        return line[i as usize];
    }
    match mode {
        Mode::Constant => 0.0,
        Mode::Reflect => {
            // reflect about the edges, repeating the edge value, with period 2n
            let period = 2 * n;
            let mut k = i.rem_euclid(period);
            if k >= n {
                k = period - 1 - k;
            }
            line[k as usize]
        }
    }
}

/// scipy `uniform_filter1d` on one line: a running sum of the raw values updated by
/// `entering - leaving`, divided by `size` on output (checked bit for bit against scipy 1.18).
fn uniform_filter_line(line: &[f64], size: usize, mode: Mode, out: &mut [f64]) {
    let n = line.len();
    let size1 = (size / 2) as isize;
    let size2 = size as isize - size1 - 1;
    let mut buf = Vec::with_capacity(n + size - 1);
    for k in -size1..0 {
        buf.push(extend(line, k, mode));
    }
    buf.extend_from_slice(line);
    for k in 0..size2 {
        buf.push(extend(line, n as isize + k, mode));
    }
    let fs = size as f64;
    let mut tmp = 0.0;
    for &v in &buf[..size] {
        tmp += v;
    }
    out[0] = tmp / fs;
    for ll in 1..n {
        tmp += buf[ll + size - 1] - buf[ll - 1];
        out[ll] = tmp / fs;
    }
}

/// `ndimage.uniform_filter(a.reshape(H, W), size, mode=...)`: the 1D filter along
/// axis 0 (columns), then along axis 1 (rows).
pub fn uniform_filter(a: &[f64], H: usize, W: usize, size: usize, mode: Mode) -> Vec<f64> {
    assert_eq!(a.len(), H * W);
    let mut tmp = vec![0.0; H * W];
    let mut col = vec![0.0; H];
    let mut col_out = vec![0.0; H];
    for x in 0..W {
        for y in 0..H {
            col[y] = a[y * W + x];
        }
        uniform_filter_line(&col, size, mode, &mut col_out);
        for y in 0..H {
            tmp[y * W + x] = col_out[y];
        }
    }
    let mut out = vec![0.0; H * W];
    for y in 0..H {
        uniform_filter_line(&tmp[y * W..(y + 1) * W], size, mode, &mut out[y * W..(y + 1) * W]);
    }
    out
}

/// `ndimage.label(mask)` with the default cross (4-neighbour) structure. Labels
/// run 1..=n in raster order of each component's first cell; 0 is background.
pub fn label(mask: &[bool], H: usize, W: usize) -> (Vec<i32>, usize) {
    let mut lab = vec![0i32; H * W];
    let mut n = 0;
    let mut queue = VecDeque::new();
    for start in 0..H * W {
        if !mask[start] || lab[start] != 0 {
            continue;
        }
        n += 1;
        lab[start] = n as i32;
        queue.push_back(start);
        while let Some(c) = queue.pop_front() {
            let (y, x) = (c / W, c % W);
            let mut visit = |nc: usize| {
                if mask[nc] && lab[nc] == 0 {
                    lab[nc] = n as i32;
                    queue.push_back(nc);
                }
            };
            if y > 0 {
                visit(c - W);
            }
            if y + 1 < H {
                visit(c + W);
            }
            if x > 0 {
                visit(c - 1);
            }
            if x + 1 < W {
                visit(c + 1);
            }
        }
    }
    (lab, n)
}

/// `ndimage.binary_dilation(mask)` with one iteration and border value 0. With
/// `square` the structure is `np.ones((3, 3))`, otherwise scipy's default cross.
pub fn binary_dilation(mask: &[bool], H: usize, W: usize, square: bool) -> Vec<bool> {
    let mut out = vec![false; H * W];
    for y in 0..H {
        for x in 0..W {
            let mut hit = false;
            for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    if !square && dy != 0 && dx != 0 {
                        continue;
                    }
                    let (ny, nx) = (y as isize + dy, x as isize + dx);
                    if ny >= 0 && ny < H as isize && nx >= 0 && nx < W as isize && mask[ny as usize * W + nx as usize] {
                        hit = true;
                    }
                }
            }
            out[y * W + x] = hit;
        }
    }
    out
}

/// 1D squared distance transform of a sampled function (Felzenszwalb and Huttenlocher).
fn sq_dt_1d(f: &[f64], out: &mut [f64]) {
    let n = f.len();
    let mut v = vec![0usize; n];
    let mut z = vec![0.0f64; n + 1];
    let mut k = 0usize;
    // skip leading infinite samples
    let Some(first) = f.iter().position(|x| x.is_finite()) else {
        out.iter_mut().for_each(|o| *o = f64::INFINITY);
        return;
    };
    v[0] = first;
    z[0] = f64::NEG_INFINITY;
    z[1] = f64::INFINITY;
    for q in first + 1..n {
        if !f[q].is_finite() {
            continue;
        }
        loop {
            let p = v[k];
            let s = ((f[q] + (q * q) as f64) - (f[p] + (p * p) as f64)) / (2.0 * q as f64 - 2.0 * p as f64);
            // z[0] is -inf, so this never steps below the first parabola
            if s <= z[k] {
                k -= 1;
                continue;
            }
            k += 1;
            v[k] = q;
            z[k] = s;
            z[k + 1] = f64::INFINITY;
            break;
        }
    }
    let mut k = 0usize;
    for (q, o) in out.iter_mut().enumerate() {
        while z[k + 1] < q as f64 {
            k += 1;
        }
        let d = q as f64 - v[k] as f64;
        *o = d * d + f[v[k]];
    }
}

/// `ndimage.distance_transform_edt(feature)`: for each cell, the Euclidean distance
/// to the nearest cell where `feature` is false (0 on those cells). Exact: squared
/// distances are integers, so the result equals `hypot(dy, dx)` to the nearest one.
pub fn distance_transform_edt(feature: &[bool], H: usize, W: usize) -> Vec<f64> {
    assert_eq!(feature.len(), H * W);
    if feature.iter().all(|&f| f) {
        return vec![f64::INFINITY; H * W];
    }
    // columns first
    let mut g = vec![0.0; H * W];
    let mut col = vec![0.0; H];
    let mut col_out = vec![0.0; H];
    for x in 0..W {
        for y in 0..H {
            col[y] = if feature[y * W + x] { f64::INFINITY } else { 0.0 };
        }
        sq_dt_1d(&col, &mut col_out);
        for y in 0..H {
            g[y * W + x] = col_out[y];
        }
    }
    let mut out = vec![0.0; H * W];
    let mut row_out = vec![0.0; W];
    for y in 0..H {
        sq_dt_1d(&g[y * W..(y + 1) * W], &mut row_out);
        for x in 0..W {
            out[y * W + x] = row_out[x].sqrt();
        }
    }
    out
}

/// The cubic B-spline prefilter along one line with mirror boundary conditions,
/// as scipy's `spline_filter1d(order=3)` (which uses the mirror initialisation for
/// `mode="constant"` too).
fn spline_prefilter_line(c: &mut [f64]) {
    let n = c.len();
    if n < 2 {
        return;
    }
    let z: f64 = 3.0f64.sqrt() - 2.0;
    let gain = (1.0 - z) * (1.0 - 1.0 / z);
    for v in c.iter_mut() {
        *v *= gain;
    }
    // causal initialisation (mirror)
    let mut z_i = z;
    let z_n_1 = z.powf((n - 1) as f64);
    c[0] += z_n_1 * c[n - 1];
    for i in 1..n - 1 {
        c[0] += z_i * (c[i] + z_n_1 * c[n - 1 - i]);
        z_i *= z;
    }
    c[0] /= 1.0 - z_n_1 * z_n_1;
    for i in 1..n {
        c[i] += z * c[i - 1];
    }
    // anticausal initialisation (mirror)
    c[n - 1] = (z * c[n - 2] + c[n - 1]) * z / (z * z - 1.0);
    for i in (0..n - 1).rev() {
        c[i] = z * (c[i + 1] - c[i]);
    }
}

/// Mirror an index into 0..len with period 2 * len - 2, as scipy does for spline coefficients.
fn mirror_index(idx: isize, len: isize) -> usize {
    if len <= 1 {
        return 0;
    }
    let s2 = 2 * len - 2;
    let mut i = idx;
    if i < 0 {
        i = s2 * (-i / s2) + i;
        i = if i <= 1 - len { i + s2 } else { -i };
    } else if i >= len {
        i -= s2 * (i / s2);
        if i >= len {
            i = s2 - i;
        }
    }
    i as usize
}

/// Cubic B-spline weights for the four coefficients around `x`, and the first index.
fn spline_weights(x: f64) -> (isize, [f64; 4]) {
    let start = x.floor() as isize - 1;
    let y = x - x.floor();
    let z = 1.0 - y;
    let mut w = [0.0; 4];
    w[0] = z * z * z / 6.0;
    w[1] = (y * y * (y - 2.0) * 3.0 + 4.0) / 6.0;
    w[2] = (z * z * (z - 2.0) * 3.0 + 4.0) / 6.0;
    w[3] = 1.0 - w[0] - w[1] - w[2];
    (start, w)
}

/// `ndimage.zoom(grid, factor, order=3)` for an integer factor: spline prefilter
/// along both axes, then cubic interpolation at `out_index * (n_in - 1) / (n_out - 1)`.
/// Returns the (gh * factor, gw * factor) result.
pub fn zoom_cubic(grid: &[f64], gh: usize, gw: usize, factor: usize) -> (Vec<f64>, usize, usize) {
    assert_eq!(grid.len(), gh * gw);
    let (oh, ow) = (gh * factor, gw * factor);
    // prefilter: axis 0 then axis 1
    let mut c = grid.to_vec();
    let mut col = vec![0.0; gh];
    for x in 0..gw {
        for y in 0..gh {
            col[y] = c[y * gw + x];
        }
        spline_prefilter_line(&mut col);
        for y in 0..gh {
            c[y * gw + x] = col[y];
        }
    }
    for y in 0..gh {
        spline_prefilter_line(&mut c[y * gw..(y + 1) * gw]);
    }
    let zy = if oh > 1 { (gh - 1) as f64 / (oh - 1) as f64 } else { 1.0 };
    let zx = if ow > 1 { (gw - 1) as f64 / (ow - 1) as f64 } else { 1.0 };
    let xw: Vec<(isize, [f64; 4])> = (0..ow).map(|ox| spline_weights(ox as f64 * zx)).collect();
    let mut out = vec![0.0; oh * ow];
    for oy in 0..oh {
        let (sy, wy) = spline_weights(oy as f64 * zy);
        let rows: [usize; 4] = std::array::from_fn(|k| mirror_index(sy + k as isize, gh as isize));
        for ox in 0..ow {
            let (sx, wx) = xw[ox];
            let mut t = 0.0;
            for (ky, &ry) in rows.iter().enumerate() {
                for kx in 0..4 {
                    let cx = mirror_index(sx + kx as isize, gw as isize);
                    t += c[ry * gw + cx] * wy[ky] * wx[kx];
                }
            }
            out[oy * ow + ox] = t;
        }
    }
    (out, oh, ow)
}
