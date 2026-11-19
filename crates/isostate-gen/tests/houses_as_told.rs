//! The houses are held to what they were before the towns' second pass.
//!
//! A house that is told only what a house was told then (its plot, storeys,
//! roof, age, whether it stands in the hills, its neighbours, its seed) comes
//! out as it did then, mesh for mesh, at all three levels of detail. What
//! was added since is told in fields of its own (the street's family, the age
//! a cottage was raised in), and each of them has a value that says "told
//! nothing": the constructors (`Cottage::new` and the rest) give it. So every
//! difference in a picture of a town is a difference in what the layout knows
//! and tells, and none is a change to the houses themselves.
//!
//! The numbers below are sums over every corner, facing, colour and triangle
//! of some thousands of houses of each kind, taken from the generators as
//! they stood when the town ground landed (commit 09c7f6a). To take them
//! again after a change that is meant to move a house told nothing, run
//! `cargo test -p isostate-gen --test houses_as_told -- --nocapture`, which
//! prints the sums it finds beside those it expects.

use glam::Vec3;
use isostate_gen::houses::roof::{Cover, Roof};
use isostate_gen::houses::{cottage, farmhouse, terrace, townhouse, villa, Cottage, Detail, Era, Farmhouse, Terrace, Townhouse, Villa, BACK_PLOT_WIDTH, PLOT};
use isostate_gen::{MeshBuf, Model, Palette, Seed, Site};

/// A running sum of everything in a model, in the order it was built.
struct Sum(u64);

impl Sum {
    fn word(&mut self, w: u32) {
        for b in w.to_le_bytes() {
            self.0 = (self.0 ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn buf(&mut self, b: &MeshBuf) {
        self.word(b.pos.len() as u32);
        for p in b.pos.iter().chain(&b.nrm) {
            p.iter().for_each(|x| self.word(x.to_bits()));
        }
        for c in &b.col {
            c.iter().for_each(|x| self.word(x.to_bits()));
        }
        self.word(b.idx.len() as u32);
        b.idx.iter().for_each(|&i| self.word(i));
    }

    fn model(&mut self, m: &Model) {
        for b in [&m.solid, &m.smoke, &m.green.evergreen, &m.green.leaves, &m.green.bare, &m.flat] {
            self.buf(b);
        }
    }
}

/// The widths of plot the towns lay: a back lane's, a high street's, and what lies between and beyond.
const WIDTHS: [f32; 5] = [BACK_PLOT_WIDTH, PLOT.0, 0.0537, 0.062, 0.066];

/// The ground a house of the sum stands on: level for one in three, the rest on a slope, each turned its own way.
fn slope(x: f32, z: f32) -> f32 {
    0.2 + 0.11 * x - 0.07 * z
}

/// One sum for each level of detail (far, play, near) over the houses `build` makes for 240 seeds.
fn sums(build: impl Fn(&mut Model, &Site, u32, Seed, Detail, &Palette)) -> [u64; 3] {
    let pal = Palette::standard();
    Detail::ALL.map(|detail| {
        let mut sum = Sum(0xcbf2_9ce4_8422_2325);
        for n in 0..240u32 {
            let seed = Seed::of(n * 7 + 1);
            let mut m = Model::default();
            let at = Vec3::new(3.0 + 0.37 * n as f32, 0.0, 5.0 + 0.11 * n as f32);
            if n % 3 == 0 {
                build(&mut m, &Site::level(Vec3::new(at.x, 0.3, at.z), 0.4 * n as f32), n, seed, detail, &pal);
            } else {
                build(&mut m, &Site::on(Vec3::new(at.x, slope(at.x, at.z), at.z), 0.4 * n as f32, &slope), n, seed, detail, &pal);
            }
            assert!(m.is_sound());
            sum.model(&m);
        }
        sum.0
    })
}

fn era(n: u32) -> Era {
    Era::ALL[(n / 3) as usize % 4]
}

fn roof(n: u32, seed: Seed) -> Roof {
    match n % 5 {
        0 => Roof { cover: Cover::Thatch, weathering: (n % 4) as u8 },
        1 => Roof { cover: Cover::Slate, weathering: (n % 3) as u8 },
        _ => Roof::of(1700 + (n as i32 * 13) % 190, 1900, n % 7 == 0, n % 2 == 0, seed),
    }
}

fn neighbours(n: u32) -> (bool, bool) {
    (n % 4 >= 2, n % 4 == 1 || n % 4 == 2)
}

fn check(kind: &str, found: [u64; 3], expected: [u64; 3]) {
    let line = |s: [u64; 3]| format!("[{:#018x}, {:#018x}, {:#018x}]", s[0], s[1], s[2]);
    println!("{kind}: found {}, expected {}", line(found), line(expected));
    assert_eq!(found, expected, "a {kind} that is told nothing new is not the {kind} it was");
}

#[test]
fn a_cottage_told_nothing_new_is_the_cottage_it_was() {
    let found = sums(|m, site, n, seed, detail, pal| {
        let c = Cottage { era: era(n), upland: n % 7 == 0, neighbours: neighbours(n), detail, ..Cottage::new(WIDTHS[n as usize % 5], PLOT.1, 1 + (n % 11 % 3) as u8, roof(n, seed), seed) };
        cottage(&mut m.solid, &mut m.smoke, &mut m.green, site, &c, pal);
    });
    check("cottage", found, COTTAGE);
}

#[test]
fn a_townhouse_told_nothing_new_is_the_townhouse_it_was() {
    let found = sums(|m, site, n, seed, detail, pal| {
        let t = Townhouse { era: era(n), upland: n % 7 == 0, shop: n % 3 == 1, neighbours: neighbours(n), detail, ..Townhouse::new(WIDTHS[n as usize % 5], PLOT.1, 2 + (n % 11 % 3) as u8, roof(n, seed), seed) };
        townhouse(&mut m.solid, &mut m.smoke, site, &t, pal);
    });
    check("townhouse", found, TOWNHOUSE);
}

#[test]
fn a_terrace_told_nothing_new_is_the_terrace_it_was() {
    let found = sums(|m, site, n, seed, detail, pal| {
        let houses = 2 + n as usize % 5;
        let t = Terrace { era: era(n), upland: n % 7 == 0, detail, ..Terrace::new(houses, Terrace::length_of(houses, WIDTHS[n as usize % 3]), PLOT.1, 2 + (n % 4 == 3) as u8, roof(n, seed), seed) };
        terrace(&mut m.solid, &mut m.smoke, site, &t, pal);
    });
    check("terrace", found, TERRACE);
}

#[test]
fn a_villa_told_nothing_new_is_the_villa_it_was() {
    let found = sums(|m, site, n, seed, detail, pal| {
        let v = Villa { era: era(n), upland: n % 7 == 0, detail, ..Villa::new(WIDTHS[n as usize % 5], PLOT.1, roof(n, seed), seed) };
        villa(&mut m.solid, &mut m.smoke, &mut m.green, site, &v, pal);
    });
    check("villa", found, VILLA);
}

#[test]
fn a_farmhouse_is_the_farmhouse_it_was() {
    let found = sums(|m, site, n, seed, detail, pal| {
        let f = Farmhouse { storeys: 1 + (n % 2) as u8, upland: n % 7 == 0, thatch: n % 3 == 0, wing: n % 5 < 2, detail, seed };
        farmhouse(&mut m.solid, &mut m.smoke, site, &f, pal);
    });
    check("farmhouse", found, FARMHOUSE);
}

/// A number from 0 to 1 for the `k`th thing about the `n`th house of `on_any_plot`: drawn
/// apart from the seeds the houses use, so that what is tried does not follow their numbers.
fn draw(n: u32, k: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9e37_79b9) ^ k.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^= x >> 12;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// The same proof taken a second way, by a second hand: 1,500 houses of each kind on plots
/// of any width from 0.04 to 0.07 and any depth from 0.030 to 0.042 (not only those the
/// towns lay), townhouses to four storeys and rows to seven houses, every age, cover and
/// weathering, each turned its own way on level ground or a slope. The sums were taken
/// from main's generators at the same commit as those above.
#[test]
fn a_house_told_nothing_new_on_any_plot_is_the_house_it_was() {
    let pal = Palette::standard();
    let slope = |x: f32, z: f32| 0.15 - 0.13 * x + 0.09 * z;
    let mut sums = [[0xcbf2_9ce4_8422_2325u64; 3]; 4];
    for n in 0..1500u32 {
        let d = |k: u32| draw(n, k);
        let seed = Seed::new(d(1), d(2));
        let (width, depth) = (0.04 + 0.03 * d(3), 0.03 + 0.012 * d(4));
        let era = Era::ALL[(d(5) * 4.0) as usize % 4];
        let roof = match (d(6) * 4.0) as u32 {
            0 => Roof { cover: Cover::Thatch, weathering: (d(7) * 4.0) as u8 },
            1 => Roof { cover: Cover::Tile, weathering: (d(7) * 4.0) as u8 },
            _ => Roof::of(1700 + (d(8) * 200.0) as i32, 1750 + (d(9) * 150.0) as i32, d(10) < 0.3, d(11) < 0.5, seed),
        };
        let (upland, neighbours) = (d(12) < 0.25, (d(13) < 0.5, d(14) < 0.5));
        let (at, yaw) = (Vec3::new(1.0 + d(15) * 20.0, 0.0, 1.0 + d(16) * 20.0), d(17) * std::f32::consts::TAU);
        for (l, detail) in Detail::ALL.into_iter().enumerate() {
            let site = if n % 2 == 0 { Site::level(Vec3::new(at.x, 0.25, at.z), yaw) } else { Site::on(Vec3::new(at.x, slope(at.x, at.z), at.z), yaw, &slope) };
            let mut add = |kind: usize, m: &Model| {
                let mut s = Sum(sums[kind][l]);
                s.model(m);
                sums[kind][l] = s.0;
            };
            let mut m = Model::default();
            let c = Cottage { era, upland, neighbours, detail, ..Cottage::new(width, depth, 1 + (d(18) * 3.0) as u8, roof, seed) };
            cottage(&mut m.solid, &mut m.smoke, &mut m.green, &site, &c, &pal);
            add(0, &m);
            let mut m = Model::default();
            let t = Townhouse { era, upland, shop: d(19) < 0.4, neighbours, detail, ..Townhouse::new(width, depth, 2 + (d(20) * 3.0) as u8, roof, seed) };
            townhouse(&mut m.solid, &mut m.smoke, &site, &t, &pal);
            add(1, &m);
            let mut m = Model::default();
            let houses = 2 + (d(21) * 6.0) as usize;
            let t = Terrace { era, upland, detail, ..Terrace::new(houses, Terrace::length_of(houses, width), depth, 2 + (d(22) < 0.3) as u8, roof, seed) };
            terrace(&mut m.solid, &mut m.smoke, &site, &t, &pal);
            add(2, &m);
            let mut m = Model::default();
            let v = Villa { era, upland, detail, ..Villa::new(width, depth, roof, seed) };
            villa(&mut m.solid, &mut m.smoke, &mut m.green, &site, &v, &pal);
            add(3, &m);
        }
    }
    let kinds = ["cottage", "townhouse", "terrace", "villa"];
    // (every sum is printed before any is held, so that all four can be taken again at once)
    for (kind, found) in kinds.iter().zip(sums) {
        println!("{kind} on any plot: found [{:#018x}, {:#018x}, {:#018x}]", found[0], found[1], found[2]);
    }
    for (kind, (found, expected)) in kinds.into_iter().zip(sums.into_iter().zip(ON_ANY_PLOT)) {
        check(kind, found, expected);
    }
}

/// The sums of `a_house_told_nothing_new_on_any_plot_is_the_house_it_was`: cottage, townhouse, terrace, villa.
const ON_ANY_PLOT: [[u64; 3]; 4] = [
    [0x23d506334213547c, 0x66da791bde3fd66f, 0xf58dbfefe319a761],
    [0x52a8160f21950881, 0xc8d5f7c87d484a98, 0x0f9df0a2aded08b3],
    [0x992aca89905585e9, 0x9bdb52276eb2e259, 0xd51d3b6f9b8e9de5],
    [0x76611b7a110c7645, 0x4013c3d4c79079ff, 0xc0f991b89ec1dab9],
];

const COTTAGE: [u64; 3] =[0x51cdd81fabb0cac1, 0x12cd0990b9ccd8a6, 0xe955a43620921d29];
const TOWNHOUSE: [u64; 3] = [0x13b84d346227a4cb, 0xea409a29b5f8e853, 0xfbac17fb276aab0e];
const TERRACE: [u64; 3] = [0x6e968d8008cfd741, 0x01c6ca85a35088d3, 0xc08a275707f94a5e];
const VILLA: [u64; 3] = [0x7629672e9baf5f08, 0x2371da0d06c40c7d, 0x2fab13f7584f7649];
const FARMHOUSE: [u64; 3] = [0xe52acdc03ba7b5b3, 0xc4bd12b06e137bc1, 0xbcf6ae29cbd49aa5];
