//! Winter on what is built. Houses, works, churches, farm buildings and
//! bridges are meshes rebuilt once a year, and the seasons turn once in 24
//! seconds, so winter must not rebuild them. Their material is Bevy's
//! StandardMaterial with a fragment shader (built.wesl) that is told how deep
//! the winter is and does the rest per pixel: snow on what faces the sky,
//! frost on the ground round a house, its garden gone dull and bare, a pond
//! frozen over. It goes only by which way a surface faces and by the mark
//! the palette gives each colour (`isostate_gen::colour::settles`, carried in
//! a corner's alpha), never by how a generator built the thing.

use bevy::asset::embedded_asset;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use crate::look::Look;

/// What houses and works are drawn in.
pub type BuiltMaterial = ExtendedMaterial<StandardMaterial, Winter>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct Winter {
    #[uniform(100)]
    pub params: WinterParams,
}

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
pub struct WinterParams {
    /// how much snow lies (0 none, 1 midwinter), how much frost, how much of
    /// a surface the snow covers at its deepest, how strong the frost is
    pub amounts: Vec4,
    /// linear colours: lying snow, hoar frost, a frozen pond, the earth of a
    /// garden bed that is dug over, a lawn in winter
    pub snow: Vec4,
    pub frost: Vec4,
    pub ice: Vec4,
    pub bare_earth: Vec4,
    pub winter_grass: Vec4,
}

impl MaterialExtension for Winter {
    fn fragment_shader() -> ShaderRef {
        "embedded://isostate_viewer/built.wesl".into()
    }
}

pub fn register(app: &mut App) {
    embedded_asset!(app, "built.wesl");
    app.add_plugins(MaterialPlugin::<BuiltMaterial>::default());
}

/// How much snow lies on roofs and how much frost is on the ground at a phase
/// of the year (0 midwinter, 0.5 midsummer), each from 0 to 1. The frost is
/// the land's own (`winter` in terrain.wesl): there from early November to
/// early March. Snow lies for less of it, from the end of November to the
/// middle of February, and is whole for the six weeks round midwinter.
pub fn snow_and_frost(phase: f32) -> (f32, f32) {
    let from_midwinter = phase.rem_euclid(1.0).min(1.0 - phase.rem_euclid(1.0));
    let smooth = |a: f32, b: f32, x: f32| {
        let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    (1.0 - smooth(0.055, 0.125, from_midwinter), 1.0 - smooth(0.06, 0.2, from_midwinter))
}

/// The shader's numbers at a phase of the year.
pub fn params(lk: &Look, phase: f32) -> WinterParams {
    let (snow, frost) = snow_and_frost(phase);
    let w = &lk.winter;
    let lin = |c: crate::look::Colour| Vec4::new(c[0], c[1], c[2], 1.0);
    WinterParams {
        amounts: Vec4::new(snow, frost, w.snow_cover, w.frost_cover),
        // snow is drawn brighter than a paint of its colour would be: see `winter.snow_bright`
        snow: lin(w.snow) * Vec4::new(w.snow_bright, w.snow_bright, w.snow_bright, 1.0),
        frost: lin(w.frost),
        ice: lin(w.ice),
        bare_earth: lin(w.bare_earth),
        winter_grass: lin(w.winter_grass),
    }
}

/// The material itself, at a phase of the year: `rough` is how matt it is
/// (`surfaces.buildings` for what stands, `surfaces.paths` for what lies flat).
pub fn material(lk: &Look, phase: f32, rough: f32) -> BuiltMaterial {
    BuiltMaterial {
        base: StandardMaterial { base_color: Color::WHITE, perceptual_roughness: rough, reflectance: lk.surfaces.painted_reflectance, ..default() },
        extension: Winter { params: params(lk, phase) },
    }
}

/// How solid smoke is at a phase of the year, against midsummer: it stands
/// fuller in cold air.
pub fn smoke_fullness(lk: &Look, phase: f32) -> f32 {
    1.0 + lk.winter.smoke_fuller * snow_and_frost(phase).1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snow_lies_in_the_depth_of_winter_and_frost_for_longer() {
        // 0 is midwinter: January is about 0.04, April 0.29, July 0.54, October 0.79
        assert_eq!(snow_and_frost(0.0), (1.0, 1.0));
        let (snow, frost) = snow_and_frost(0.04);
        assert!(snow == 1.0 && frost == 1.0, "January: {snow} {frost}");
        for ph in [0.29, 0.54, 0.79] {
            assert_eq!(snow_and_frost(ph), (0.0, 0.0), "no winter at {ph}");
        }
        // the frost comes first and goes last, and both go the same either side of midwinter
        for i in 0..200 {
            let ph = i as f32 / 200.0;
            let (snow, frost) = snow_and_frost(ph);
            assert!(snow <= frost, "snow without frost at {ph}");
            let (s2, f2) = snow_and_frost(1.0 - ph);
            assert!((snow - s2).abs() < 1e-4 && (frost - f2).abs() < 1e-4);
        }
        assert!(snow_and_frost(0.15).0 == 0.0 && snow_and_frost(0.15).1 > 0.0);
        // a season given the long way round is the same season
        assert_eq!(snow_and_frost(1.0), snow_and_frost(0.0));
        assert_eq!(snow_and_frost(-0.96), snow_and_frost(0.04));
    }

    #[test]
    fn the_shader_declares_the_uniform_as_the_material_has_it() {
        // the fields of WinterParams, in order, are the fields of the shader's struct
        let src = include_str!("built.wesl");
        let body = src.split("struct WinterParams {").nth(1).expect("built.wesl declares WinterParams").split('}').next().unwrap();
        let fields: Vec<&str> = body.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with("//")).map(|l| l.split(':').next().unwrap()).collect();
        assert_eq!(fields, ["amounts", "snow", "frost", "ice", "bare_earth", "winter_grass"]);
        // and the marks it reads are the palette's
        use isostate_gen::colour::settles::{FROST, ICE, NOTHING, SNOW};
        assert!(SNOW > 0.75 && FROST < 0.75 && FROST > 0.375 && ICE < 0.375 && ICE > 0.125 && NOTHING < 0.125);
        for cut in ["0.75", "0.375", "0.125"] {
            assert!(src.contains(cut), "built.wesl sorts the marks at {cut}");
        }
    }

    #[test]
    fn snow_is_drawn_brighter_than_a_paint_of_its_colour_and_brighter_than_frost() {
        let lk = Look::standard();
        let p = params(&lk, 0.0);
        assert!(lk.winter.snow_bright > 1.0);
        for i in 0..3 {
            assert!((p.snow[i] - lk.winter.snow[i] * lk.winter.snow_bright).abs() < 1e-6);
            assert!(p.snow[i] > p.frost[i], "snow is no brighter than frost in channel {i}");
        }
        // and it is a cold white: the low sun of winter is warm, and a plain white under it
        // comes out the cream of the walls
        assert!(p.snow.z > p.snow.x);
    }

    #[test]
    fn smoke_is_fuller_in_winter_and_never_more_than_solid() {
        let lk = Look::standard();
        assert_eq!(smoke_fullness(&lk, 0.5), 1.0);
        assert!(smoke_fullness(&lk, 0.0) > 1.0);
        // a plume is at most 0.85 solid where it leaves the flue (`works::plume`)
        assert!(0.85 * smoke_fullness(&lk, 0.0) <= 1.0);
    }
}
