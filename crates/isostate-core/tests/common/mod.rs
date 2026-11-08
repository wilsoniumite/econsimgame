//! Test helpers: a minimal `.npy` reader, tolerant array comparison and the Python state loader.
#![allow(dead_code)]

pub mod dump;

use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub enum NpyData {
    F64(Vec<f64>),
    I64(Vec<i64>),
    U8(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct Npy {
    pub shape: Vec<usize>,
    pub data: NpyData,
}

impl Npy {
    pub fn len(&self) -> usize {
        self.shape.iter().product()
    }

    pub fn f64(&self) -> Vec<f64> {
        match &self.data {
            NpyData::F64(v) => v.clone(),
            NpyData::I64(v) => v.iter().map(|&x| x as f64).collect(),
            NpyData::U8(v) => v.iter().map(|&x| x as f64).collect(),
        }
    }

    pub fn i64(&self) -> Vec<i64> {
        match &self.data {
            NpyData::I64(v) => v.clone(),
            NpyData::U8(v) => v.iter().map(|&x| x as i64).collect(),
            NpyData::F64(_) => panic!("expected an integer array"),
        }
    }

    pub fn usize(&self) -> Vec<usize> {
        self.i64().into_iter().map(|x| usize::try_from(x).expect("negative index")).collect()
    }

    pub fn bool(&self) -> Vec<bool> {
        self.i64().into_iter().map(|x| x != 0).collect()
    }
}

/// Read a little-endian `.npy` file of dtype `<f8`, `<i8` or `|u1` (C order).
pub fn read_npy(path: &Path) -> Npy {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert_eq!(&bytes[..6], b"\x93NUMPY", "{}: not an npy file", path.display());
    let major = bytes[6];
    let (hlen, start) = if major == 1 {
        (u16::from_le_bytes([bytes[8], bytes[9]]) as usize, 10)
    } else {
        (u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize, 12)
    };
    let header = std::str::from_utf8(&bytes[start..start + hlen]).expect("npy header is ascii");
    let field = |key: &str| -> &str {
        let at = header.find(key).unwrap_or_else(|| panic!("npy header lacks {key}: {header}"));
        &header[at + key.len()..]
    };
    let descr = field("'descr':").trim_start();
    let descr = &descr[1..descr[1..].find('\'').unwrap() + 1];
    assert!(field("'fortran_order':").trim_start().starts_with("False"), "fortran order npy");
    let shape_txt = field("'shape':").trim_start();
    let shape_txt = &shape_txt[1..shape_txt.find(')').unwrap()];
    let shape: Vec<usize> = shape_txt.split(',').map(str::trim).filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect();
    let n: usize = shape.iter().product();
    let raw = &bytes[start + hlen..];
    let data = match descr {
        "<f8" => NpyData::F64(raw.chunks_exact(8).take(n).map(|c| f64::from_le_bytes(c.try_into().unwrap())).collect()),
        "<i8" => NpyData::I64(raw.chunks_exact(8).take(n).map(|c| i64::from_le_bytes(c.try_into().unwrap())).collect()),
        "|u1" | "|b1" => NpyData::U8(raw[..n].to_vec()),
        other => panic!("{}: unsupported dtype {other}", path.display()),
    };
    Npy { shape, data }
}

/// Where the Python-generated fixtures live: `$ISOSTATE_PARITY_DIR` or `parity/fixtures`.
pub fn fixtures_root() -> PathBuf {
    match std::env::var("ISOSTATE_PARITY_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => Path::new(env!("CARGO_MANIFEST_DIR")).join("parity").join("fixtures"),
    }
}

/// The fixture directory `name`, or None (with a note) when it has not been generated.
/// Set `ISOSTATE_REQUIRE_PARITY=1` to make a missing fixture a failure.
pub fn fixture_dir(name: &str, how: &str) -> Option<PathBuf> {
    let d = fixtures_root().join(name);
    if d.is_dir() {
        return Some(d);
    }
    let msg = format!("parity fixture {} not found; generate it with: {how}", d.display());
    if std::env::var("ISOSTATE_REQUIRE_PARITY").is_ok() {
        panic!("{msg}");
    }
    eprintln!("SKIPPED: {msg}");
    None
}

/// Element-wise comparison: |a - b| <= rtol * max(|a|, |b|) + atol. Returns the worst
/// offender as a message, or None when everything matches.
pub fn compare(name: &str, got: &[f64], want: &[f64], rtol: f64, atol: f64) -> Option<String> {
    if got.len() != want.len() {
        return Some(format!("{name}: length {} vs {}", got.len(), want.len()));
    }
    let mut worst: Option<(usize, f64)> = None;
    let mut bad = 0usize;
    for (i, (&g, &w)) in got.iter().zip(want).enumerate() {
        let ok = if g.is_nan() || w.is_nan() {
            g.is_nan() && w.is_nan()
        } else if g.is_infinite() || w.is_infinite() {
            g == w
        } else {
            (g - w).abs() <= rtol * g.abs().max(w.abs()) + atol
        };
        if !ok {
            bad += 1;
            let excess = if g.is_finite() && w.is_finite() { (g - w).abs() / (g.abs().max(w.abs()) + atol) } else { f64::INFINITY };
            if worst.map_or(true, |(_, e)| excess > e) {
                worst = Some((i, excess));
            }
        }
    }
    worst.map(|(i, e)| format!("{name}: {bad} of {} differ; worst at {i}: got {} want {} (rel {e:.3e})", got.len(), got[i], want[i]))
}

/// A tolerance scaled to the array: atol = scale * max |want|.
pub fn atol_for(want: &[f64], scale: f64) -> f64 {
    let m = want.iter().filter(|x| x.is_finite()).fold(0.0f64, |a, &x| a.max(x.abs()));
    m * scale
}
