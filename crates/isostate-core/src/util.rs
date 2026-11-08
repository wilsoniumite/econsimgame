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
    // sort (key, index) pairs: the key orders like f64::total_cmp, the index breaks ties
    let mut keyed: Vec<(u64, u32)> = a.iter().enumerate().map(|(i, &v)| (crate::world::ord_key(v), i as u32)).collect();
    keyed.sort_unstable();
    keyed.into_iter().map(|(_, i)| i as usize).collect()
}

pub fn bool_f(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ---------------- Python number formatting used in event text and output files

/// Python's `round(x, n)` for a float: correctly rounded, ties to even (Rust's
/// fixed-precision formatting rounds the same way).
pub fn round_dp(x: f64, n: usize) -> f64 {
    if !x.is_finite() {
        return x;
    }
    format!("{x:.n$}").parse().unwrap()
}

/// Python's `f"{v:.{n}%}"`: the value times 100 in floating point, fixed, then "%".
pub fn fmt_pct(v: f64, n: usize) -> String {
    format!("{:.n$}%", v * 100.0)
}

/// Python's `f"{v:+.{n}%}"`.
pub fn fmt_pct_signed(v: f64, n: usize) -> String {
    format!("{:+.n$}%", v * 100.0)
}

fn group_thousands(digits: &str) -> String {
    let (sign, digits) = if let Some(d) = digits.strip_prefix('-') { ("-", d) } else { ("", digits) };
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    format!("{sign}{out}")
}

/// Python's `f"{n:,}"` for an integer.
pub fn fmt_thousands(n: i64) -> String {
    group_thousands(&n.to_string())
}

/// Python's `f"{x:,.0f}"`.
pub fn fmt_thousands_f0(x: f64) -> String {
    group_thousands(&format!("{x:.0}"))
}

/// Python's `f"{x:.6g}"`, as the CLI writes the monthly series.
pub fn fmt_g6(x: f64) -> String {
    if x.is_nan() {
        return "nan".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".to_string() } else { "-inf".to_string() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0".to_string() } else { "0".to_string() };
    }
    const P: i32 = 6;
    // the exponent after rounding to P significant digits decides the style
    let e_repr = format!("{:.*e}", (P - 1) as usize, x);
    let (mantissa, exp) = e_repr.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let strip = |s: &str| -> String {
        if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s.to_string() }
    };
    if (-4..P).contains(&exp) {
        strip(&format!("{:.*}", (P - 1 - exp) as usize, x))
    } else {
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{}e{}{:02}", strip(mantissa), sign, exp.abs())
    }
}
