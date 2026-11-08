//! numpy semantics the port relies on, in one place.
//!
//! numpy sums a contiguous run of values (`a.sum()`, `a.sum(axis=-1)`) pairwise,
//! eight accumulators at a time; it sums along any other axis, and in `bincount`,
//! `add.at` and `einsum`, one value after another. Following both keeps the Rust
//! numbers within a few ulps of the Python ones.

/// numpy's `pairwise_sum` for a contiguous array (`np.sum`, `x.sum(axis=-1)`).
pub fn pairwise_sum(a: &[f64]) -> f64 {
    let n = a.len();
    if n < 8 {
        let mut res = 0.0;
        for &v in a {
            res += v;
        }
        res
    } else if n <= 128 {
        let mut r = [a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7]];
        let mut i = 8;
        while i < n - (n % 8) {
            for j in 0..8 {
                r[j] += a[i + j];
            }
            i += 8;
        }
        let mut res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
        while i < n {
            res += a[i];
            i += 1;
        }
        res
    } else {
        let mut n2 = n / 2;
        n2 -= n2 % 8;
        pairwise_sum(&a[..n2]) + pairwise_sum(&a[n2..])
    }
}

/// Pairwise sum of `f(i)` for i in 0..n (a contiguous temporary in numpy).
pub fn pairwise_sum_by(n: usize, f: impl Fn(usize) -> f64) -> f64 {
    if n <= 16 {
        let mut buf = [0.0f64; 16];
        for (i, b) in buf.iter_mut().enumerate().take(n) {
            *b = f(i);
        }
        pairwise_sum(&buf[..n])
    } else {
        let v: Vec<f64> = (0..n).map(f).collect();
        pairwise_sum(&v)
    }
}

/// `np.bincount(idx, weights=w, minlength=n)`: sums in index order.
pub fn bincount(idx: &[usize], w: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![0.0; n];
    for (&i, &v) in idx.iter().zip(w) {
        out[i] += v;
    }
    out
}

/// `np.argmax`: the first index of the largest value (NaN never wins here).
pub fn argmax(a: &[f64]) -> usize {
    let mut best = 0;
    for i in 1..a.len() {
        if a[i] > a[best] {
            best = i;
        }
    }
    best
}

/// `np.argmin`: the first index of the smallest value.
pub fn argmin(a: &[f64]) -> usize {
    let mut best = 0;
    for i in 1..a.len() {
        if a[i] < a[best] {
            best = i;
        }
    }
    best
}

/// `np.argsort(a)`, ascending, NaN last. numpy's default sort is not stable and its
/// tie order depends on the platform's SIMD sort; here ties are broken by index.
/// The sim's uses only see ties in measure-zero cases (noted where they matter).
pub fn argsort(a: &[f64]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..a.len()).collect();
    idx.sort_by(|&i, &j| a[i].total_cmp(&a[j]).then(i.cmp(&j)));
    idx
}

pub fn bool_f(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}
