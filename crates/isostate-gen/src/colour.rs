//! Colours: a token from the look file, and the linear RGBA a mesh corner carries.

use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::Deserialize;

/// The colour of a mesh corner: linear red, green, blue and alpha.
pub type Rgba = [f32; 4];

/// A colour from the look file: "#rrggbb", or [red, green, blue] from 0 to 1,
/// both in sRGB as a painter would pick them. It stands for its linear RGBA,
/// which is what mesh corners want (`*palette.brick.red`), and keeps the sRGB
/// it was written in for lights, materials and shaders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colour {
    linear: Rgba,
    srgb: [f32; 3],
}

impl Colour {
    pub fn from_srgb(srgb: [f32; 3]) -> Colour {
        let f = |s: f32| if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) };
        Colour { linear: [f(srgb[0]), f(srgb[1]), f(srgb[2]), 1.0], srgb }
    }

    /// "#rrggbb", or nothing if it is not written so.
    pub fn from_hex(s: &str) -> Option<Colour> {
        let h = s.strip_prefix('#')?;
        if h.len() != 6 || !h.is_ascii() {
            return None;
        }
        let v = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok().map(|b| b as f32 / 255.0);
        Some(Colour::from_srgb([v(0)?, v(2)?, v(4)?]))
    }

    /// The colour as it was written: sRGB red, green and blue from 0 to 1.
    pub fn srgb(self) -> [f32; 3] {
        self.srgb
    }

    /// The colour as a shader wants it in a uniform: sRGB, as the ground and
    /// sea shaders mix their colours, with a 1 to fill the vector.
    pub fn srgb4(self) -> glam::Vec4 {
        glam::Vec4::new(self.srgb[0], self.srgb[1], self.srgb[2], 1.0)
    }
}

impl std::ops::Deref for Colour {
    type Target = Rgba;
    fn deref(&self) -> &Rgba {
        &self.linear
    }
}

impl<'de> Deserialize<'de> for Colour {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Colour, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Colour;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a colour: \"#rrggbb\" or [red, green, blue] from 0 to 1")
            }
            fn visit_str<E: de::Error>(self, s: &str) -> Result<Colour, E> {
                Colour::from_hex(s).ok_or_else(|| E::custom(format!("\"{s}\" is not a colour: write \"#rrggbb\" or [red, green, blue] from 0 to 1")))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Colour, A::Error> {
                let mut c = [0f32; 3];
                for (i, v) in c.iter_mut().enumerate() {
                    *v = seq.next_element::<f32>()?.ok_or_else(|| de::Error::invalid_length(i, &"three numbers: red, green, blue"))?;
                }
                if seq.next_element::<f32>()?.is_some() {
                    return Err(de::Error::invalid_length(4, &"three numbers: red, green, blue"));
                }
                Ok(Colour::from_srgb(c))
            }
        }
        d.deserialize_any(V)
    }
}

/// One of a list of tints, by a number of the thing's own: `v` from 0 to 1
/// spreads over the list, and larger numbers go round it again.
pub fn pick(list: &[Colour], v: f32) -> Rgba {
    *list[(v * list.len() as f32) as usize % list.len()]
}

/// sRGB hex colour to linear RGBA. For colours that are not the look's to
/// name (a family's banner); scenery takes its colours from the palette.
pub fn hex(c: u32) -> Rgba {
    let f = |v: u32| {
        let s = v as f32 / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    [f((c >> 16) & 255), f((c >> 8) & 255), f(c & 255), 1.0]
}

/// The colour darker (`k` under 1) or lighter, as a face turned from or to the light.
pub fn shade(c: Rgba, k: f32) -> Rgba {
    [c[0] * k, c[1] * k, c[2] * k, c[3]]
}

/// From `a` to `b` as `t` goes from 0 to 1.
pub fn lerp(a: Rgba, b: Rgba, t: f32) -> Rgba {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, 1.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hex_token_is_the_same_linear_colour_as_the_number() {
        assert_eq!(*Colour::from_hex("#b5634c").unwrap(), hex(0xb5634c));
        assert_eq!(Colour::from_hex("#ffffff").unwrap().srgb(), [1.0, 1.0, 1.0]);
        assert!(Colour::from_hex("b5634c").is_none());
        assert!(Colour::from_hex("#b5634").is_none());
        assert!(Colour::from_hex("#b5634g").is_none());
    }

    #[test]
    fn pick_spreads_over_the_list_and_goes_round_again() {
        let list = [Colour::from_srgb([0.0; 3]), Colour::from_srgb([0.5; 3]), Colour::from_srgb([1.0; 3])];
        assert_eq!(pick(&list, 0.0), *list[0]);
        assert_eq!(pick(&list, 0.5), *list[1]);
        assert_eq!(pick(&list, 0.99), *list[2]);
        assert_eq!(pick(&list, 1.4), *list[1]);
    }

    #[test]
    fn shade_and_lerp() {
        assert_eq!(shade([1.0, 0.5, 0.25, 1.0], 0.5), [0.5, 0.25, 0.125, 1.0]);
        assert_eq!(lerp([0.0, 0.0, 0.0, 1.0], [1.0, 2.0, 4.0, 1.0], 0.5), [0.5, 1.0, 2.0, 1.0]);
    }
}
