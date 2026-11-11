//! Random streams and the handful of numpy `Generator` methods the simulation uses.
//!
//! The Python sim draws from seven independent PCG64 streams spawned from one
//! `SeedSequence` (`sim.make_rngs`). Here each stream is a `rand_pcg::Pcg64` whose
//! 256-bit seed is derived from (seed, stream index) by SplitMix64. The streams are
//! not bit-compatible with numpy; the sampling algorithms below follow numpy's so
//! that the distributions are the same.

use std::collections::VecDeque;

use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg64;

/// One random stream: a PCG64 generator, or in parity tests a recorded sequence
/// of the draws numpy made (`parity/dump_state.py`), replayed in order so a Rust
/// function can be checked against the Python one draw for draw.
#[derive(Clone, Debug)]
pub struct Rng {
    pcg: Pcg64,
    replay: Option<VecDeque<f64>>,
}

impl Rng {
    pub fn from_seed(seed: [u8; 32]) -> Rng {
        Rng { pcg: Pcg64::from_seed(seed), replay: None }
    }

    /// A stream that returns the recorded draws: each `random` the next value,
    /// each `integers` the next value as an integer.
    pub fn replaying(draws: Vec<f64>) -> Rng {
        Rng { pcg: Pcg64::from_seed([0; 32]), replay: Some(draws.into()) }
    }

    /// Recorded draws not used yet (None for a generator).
    pub fn replay_left(&self) -> Option<usize> {
        self.replay.as_ref().map(|q| q.len())
    }

    fn next_replayed(&mut self) -> Option<f64> {
        self.replay.as_mut().map(|q| q.pop_front().expect("replayed draws exhausted"))
    }
}

/// Python: `RNG_STREAMS = ["world", "names", "firms", "tech", "people", "rail", "misc"]`.
pub const RNG_STREAMS: [&str; 7] = ["world", "names", "firms", "tech", "people", "rail", "misc"];

#[derive(Clone, Debug)]
pub struct Rngs {
    pub world: Rng,
    pub names: Rng,
    pub firms: Rng,
    pub tech: Rng,
    pub people: Rng,
    pub rail: Rng,
    pub misc: Rng,
}

fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// One independent stream per name, like `SeedSequence(seed).spawn(7)`.
pub fn make_rngs(seed: u64) -> Rngs {
    let stream = |k: u64| -> Rng {
        let mut x = seed ^ k.wrapping_add(1).wrapping_mul(0xD1B5_4A32_D192_ED03);
        let mut bytes = [0u8; 32];
        for chunk in bytes.chunks_exact_mut(8) {
            chunk.copy_from_slice(&splitmix64(&mut x).to_le_bytes());
        }
        Rng::from_seed(bytes)
    };
    Rngs {
        world: stream(0),
        names: stream(1),
        firms: stream(2),
        tech: stream(3),
        people: stream(4),
        rail: stream(5),
        misc: stream(6),
    }
}

/// Stream 8, for `peopling` (not in the Python): made from the seed and its own
/// number exactly as the seven above are, so it disturbs none of them.
pub fn people_stream(seed: u64) -> Rng {
    let mut x = seed ^ 9u64.wrapping_mul(0xD1B5_4A32_D192_ED03);
    let mut bytes = [0u8; 32];
    for chunk in bytes.chunks_exact_mut(8) {
        chunk.copy_from_slice(&splitmix64(&mut x).to_le_bytes());
    }
    Rng::from_seed(bytes)
}

/// `rng.random()`: uniform on [0, 1) with 53 random bits, as numpy.
pub fn random(rng: &mut Rng) -> f64 {
    match rng.next_replayed() {
        Some(v) => v,
        None => rng.pcg.random::<f64>(),
    }
}

/// `rng.random(n)`.
pub fn random_n(rng: &mut Rng, n: usize) -> Vec<f64> {
    (0..n).map(|_| random(rng)).collect()
}

/// `rng.uniform(lo, hi)`: numpy computes `lo + (hi - lo) * random()`.
pub fn uniform(rng: &mut Rng, lo: f64, hi: f64) -> f64 {
    lo + (hi - lo) * random(rng)
}

/// `rng.integers(0, n)`: uniform on 0..n.
pub fn integers(rng: &mut Rng, n: usize) -> usize {
    assert!(n > 0, "integers(0, 0)");
    match rng.next_replayed() {
        Some(v) => {
            assert!(v >= 0.0 && (v as usize) < n, "replayed integer {v} not in 0..{n}");
            v as usize
        }
        None => rng.pcg.random_range(0..n),
    }
}

/// `rng.choice(seq)` for a sequence of length n: numpy draws `integers(0, n)`.
pub fn choice_index(rng: &mut Rng, n: usize) -> usize {
    integers(rng, n)
}

/// `cdf.searchsorted(x, side="right")`: the first index whose value is > x.
fn searchsorted_right(cdf: &[f64], x: f64) -> usize {
    cdf.partition_point(|&v| v <= x)
}

/// Cumulative sum normalised by its last value, as numpy's `choice` builds it.
fn normalised_cdf(p: &[f64]) -> Vec<f64> {
    let mut cdf = Vec::with_capacity(p.len());
    let mut acc = 0.0;
    for &v in p {
        acc += v;
        cdf.push(acc);
    }
    let last = *cdf.last().expect("choice from an empty population");
    for v in cdf.iter_mut() {
        *v /= last;
    }
    cdf
}

/// `rng.choice(len(p), p=p)`: one draw with replacement.
pub fn choice_p(rng: &mut Rng, p: &[f64]) -> usize {
    let cdf = normalised_cdf(p);
    let x = random(rng);
    searchsorted_right(&cdf, x).min(p.len() - 1)
}

/// `rng.choice(len(p), size, replace=False, p=p)`, numpy's algorithm: draw `size`
/// points against the cdf, keep the distinct hits in the order first drawn, zero
/// the found entries and repeat for the rest.
pub fn choice_without_replacement(rng: &mut Rng, size: usize, p: &[f64]) -> Vec<usize> {
    let nonzero = p.iter().filter(|&&v| v > 0.0).count();
    assert!(nonzero >= size, "Fewer non-zero entries in p than size");
    let mut p = p.to_vec();
    let mut found: Vec<usize> = Vec::with_capacity(size);
    while found.len() < size {
        let x = random_n(rng, size - found.len());
        for &f in &found {
            p[f] = 0.0;
        }
        let cdf = normalised_cdf(&p);
        // np.unique(new, return_index=True), then the hits sorted by first position
        let mut first: Vec<(usize, usize)> = Vec::new(); // (index drawn, position of its first draw)
        for (pos, &xi) in x.iter().enumerate() {
            let idx = searchsorted_right(&cdf, xi).min(p.len() - 1);
            if !first.iter().any(|&(i, _)| i == idx) {
                first.push((idx, pos));
            }
        }
        // `first` is already in order of first position
        for (idx, _) in first {
            found.push(idx);
        }
    }
    found
}
