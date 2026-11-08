//! The scipy.ndimage stand-ins against scipy itself.
//!
//! Fixtures: `python3 crates/isostate-core/parity/dump_ndimage.py crates/isostate-core/parity/fixtures/ndimage`

mod common;

use common::{compare, fixture_dir, read_npy};
use isostate_core::ndimage::{self, Mode};

const HOW: &str = "python3 crates/isostate-core/parity/dump_ndimage.py crates/isostate-core/parity/fixtures/ndimage";
const H: usize = 84;
const W: usize = 120;

#[test]
fn uniform_filter_matches_scipy_bit_for_bit() {
    let Some(d) = fixture_dir("ndimage", HOW) else { return };
    let counts = read_npy(&d.join("uf_counts.npy")).f64();
    let vals = read_npy(&d.join("uf_vals.npy")).f64();
    for (input, size, mode, name) in [
        (&counts, 5, Mode::Constant, "uf_counts_c5"),
        (&counts, 7, Mode::Constant, "uf_counts_c7"),
        (&vals, 5, Mode::Constant, "uf_vals_c5"),
        (&vals, 11, Mode::Reflect, "uf_vals_r11"),
    ] {
        let want = read_npy(&d.join(format!("{name}.npy"))).f64();
        let got = ndimage::uniform_filter(input, H, W, size, mode);
        if let Some(e) = compare(name, &got, &want, 0.0, 0.0) {
            panic!("{e}");
        }
    }
}

#[test]
fn label_dilation_and_distance_match_scipy() {
    let Some(d) = fixture_dir("ndimage", HOW) else { return };
    let mask = read_npy(&d.join("mask.npy")).bool();
    let (lab, n) = ndimage::label(&mask, H, W);
    assert_eq!(n as i64, read_npy(&d.join("label_n.npy")).i64()[0]);
    assert_eq!(lab.iter().map(|&x| x as i64).collect::<Vec<_>>(), read_npy(&d.join("label.npy")).i64());
    assert_eq!(ndimage::binary_dilation(&mask, H, W, true), read_npy(&d.join("dilate_square.npy")).bool());
    assert_eq!(ndimage::binary_dilation(&mask, H, W, false), read_npy(&d.join("dilate_cross.npy")).bool());
    let sparse = read_npy(&d.join("sparse.npy")).bool();
    let not_sparse: Vec<bool> = sparse.iter().map(|&s| !s).collect();
    let got = ndimage::distance_transform_edt(&not_sparse, H, W);
    if let Some(e) = compare("edt_sparse", &got, &read_npy(&d.join("edt_sparse.npy")).f64(), 0.0, 0.0) {
        panic!("{e}");
    }
    let got = ndimage::distance_transform_edt(&mask, H, W);
    if let Some(e) = compare("edt_blob", &got, &read_npy(&d.join("edt_blob.npy")).f64(), 0.0, 0.0) {
        panic!("{e}");
    }
}

#[test]
fn cubic_zoom_matches_scipy() {
    let Some(d) = fixture_dir("ndimage", HOW) else { return };
    for step in [13usize, 6, 3, 2, 11] {
        let inp = read_npy(&d.join(format!("zoom_in_{step}.npy")));
        let want = read_npy(&d.join(format!("zoom_out_{step}.npy")));
        let (got, oh, ow) = ndimage::zoom_cubic(&inp.f64(), inp.shape[0], inp.shape[1], step);
        assert_eq!(vec![oh, ow], want.shape);
        // the prefilter and the weights are reproduced in scipy's order, but not every
        // product, so allow rounding at the level of a few ulps of the unit range
        if let Some(e) = compare(&format!("zoom_{step}"), &got, &want.f64(), 1e-12, 1e-12) {
            panic!("{e}");
        }
    }
}
