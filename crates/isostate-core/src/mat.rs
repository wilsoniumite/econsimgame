//! A dense row-major 2D array of f64: the shape of the 2D numpy arrays in the Python state.
//!
//! `m[(r, c)]` reads like numpy's `m[r, c]`, and `m.row(r)` is `m[r]`.

use std::ops::{Index, IndexMut};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mat {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Mat {
    pub fn zeros(rows: usize, cols: usize) -> Mat {
        Mat::filled(rows, cols, 0.0)
    }

    pub fn filled(rows: usize, cols: usize, v: f64) -> Mat {
        Mat { rows, cols, data: vec![v; rows * cols] }
    }

    pub fn from_vec(rows: usize, cols: usize, data: Vec<f64>) -> Mat {
        assert_eq!(data.len(), rows * cols, "Mat::from_vec: {} values for a {rows}x{cols} matrix", data.len());
        Mat { rows, cols, data }
    }

    pub fn row(&self, r: usize) -> &[f64] {
        &self.data[r * self.cols..(r + 1) * self.cols]
    }

    pub fn row_mut(&mut self, r: usize) -> &mut [f64] {
        &mut self.data[r * self.cols..(r + 1) * self.cols]
    }

    pub fn fill(&mut self, v: f64) {
        self.data.iter_mut().for_each(|x| *x = v);
    }
}

impl Index<(usize, usize)> for Mat {
    type Output = f64;
    fn index(&self, (r, c): (usize, usize)) -> &f64 {
        debug_assert!(r < self.rows && c < self.cols);
        &self.data[r * self.cols + c]
    }
}

impl IndexMut<(usize, usize)> for Mat {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut f64 {
        debug_assert!(r < self.rows && c < self.cols);
        &mut self.data[r * self.cols + c]
    }
}
