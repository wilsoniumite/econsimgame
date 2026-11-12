//! The hash every fixed choice hangs on: a slot's place, its thresholds and
//! its variety all come from it, so the same island always looks the same.
//! The ground shader (the viewer's terrain.wesl) repeats it on the GPU; change
//! one side and you must change the other.

/// A number from 0 to 1 fixed by two whole numbers and a seed.
pub fn hash2(x: i32, y: i32, s: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ (s as u32).wrapping_mul(1_442_695_041);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    h as f32 / 4_294_967_295.0
}

/// The two fixed numbers a slot carries for variety, each from 0 to 1. A
/// generator reads its choices off them (which wash, how wide, whether there
/// is a chimney), so the same slot always gives the same building. In the
/// game they are the slot's own hashes; in the lab and in tests `Seed::of(n)`
/// makes a pair from a whole number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seed {
    pub u: f32,
    pub v: f32,
}

impl Seed {
    pub fn new(u: f32, v: f32) -> Seed {
        Seed { u, v }
    }

    /// A further number from 0 to 1 that follows from the pair: the `n`th
    /// of as many as a thing needs, each unlike the others, for the choices
    /// it must not make all alike (a house with a rear wing need not be the
    /// one with a dormer).
    pub fn pick(&self, n: i32) -> f32 {
        hash2((self.u * 65_521.0) as i32, (self.v * 65_521.0) as i32, 1_000 + n)
    }

    /// The pair for the `n`th of anything.
    pub fn of(n: u32) -> Seed {
        Seed { u: hash2(n as i32, 1, 77), v: hash2(n as i32, 2, 77) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hash_is_what_it_always_was() {
        // every slot on the island and the ground shader depend on these exact values
        // (taken from the function as it stood in the viewer's world.rs when it moved here)
        assert_eq!(hash2(0, 0, 0), 0.0);
        assert_eq!(hash2(1, 2, 3), 0.6394013);
        assert_eq!(hash2(-7, 40, 11), 0.2295166);
        assert_eq!(hash2(4999, 3, 17), 0.4608432);
    }

    #[test]
    fn it_stays_between_nought_and_one_and_spreads() {
        let mut sum = 0.0;
        for i in 0..10_000 {
            let h = hash2(i, i * 7 + 3, 5);
            assert!((0.0..=1.0).contains(&h));
            sum += h;
        }
        assert!((sum / 10_000.0 - 0.5).abs() < 0.02);
    }

    #[test]
    fn a_seed_is_fixed_by_its_number() {
        assert_eq!(Seed::of(4), Seed::of(4));
        assert_ne!(Seed::of(4), Seed::of(5));
    }
}
