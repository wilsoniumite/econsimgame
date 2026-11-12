//! The look: every colour the scenery is built in, and the light and the lens
//! it is seen through, read from one file. The standard look is
//! `assets/look.toml`, built into the viewer as the font is; `--look FILE`
//! lays another file over it, which need name only what it changes.
//!
//! The scenery code takes the look as an argument and never keeps colours of
//! its own, so trying a direction is a change of data. The look is shared in
//! an `Arc`, as a year's scenery is built off the main thread.
//!
//! The look has two halves. The named colours of the scenery are the palette,
//! which belongs to the generators (the isostate-gen crate, which has no Bevy
//! in it) and is read out of the same file. The land's and the sea's paints,
//! the light and the lens are the viewer's, and are here.
//!
//! The ground and the sea are painted by shaders; their colours are tables of
//! the look like any other, handed to the shaders as uniforms (`Land`, `Sea`).
//!
//! Not here: the interface's colours (ui.rs, family.rs, game.rs, icons.rs).

use std::sync::Arc;

use bevy::prelude::{Color, Deref, Resource};
use serde::Deserialize;

/// The standard look, as the file in the source tree has it.
pub const STANDARD: &str = include_str!("../assets/look.toml");

/// The look in use, where systems can reach it.
#[derive(Resource, Clone, Deref)]
pub struct TheLook(pub Arc<Look>);

// ------------------------------------------------------------------ a colour

/// A colour token and the palette of the scenery are the generators', which
/// have no Bevy in them (isostate-gen); the look is that palette and, beside
/// it, what the viewer alone needs.
pub use isostate_gen::{Colour, Palette};

/// A look token as a Bevy colour, for a light or a material
/// (`look.sky.light.color()`).
pub trait ToColor {
    fn color(self) -> Color;
}

impl ToColor for Colour {
    fn color(self) -> Color {
        let s = self.srgb();
        Color::srgb(s[0], s[1], s[2])
    }
}

// ------------------------------------------------------------------ what the shaders paint

/// A table of colours for a shader: the table as the look file has it, and the
/// same colours laid out as the shader's uniform, field for field. The shader
/// declares a struct with the same fields in the same order (a test checks it).
macro_rules! shader_paint {
    ($(#[$doc:meta])* $table:ident as $uniform:ident { $($name:ident),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Deserialize, Clone, Debug)]
        #[serde(deny_unknown_fields)]
        pub struct $table {
            $(pub $name: Colour,)*
        }

        #[derive(Clone, Copy, Debug, bevy::reflect::Reflect, bevy::render::render_resource::ShaderType)]
        pub struct $uniform {
            $(pub $name: bevy::math::Vec4,)*
        }

        impl $table {
            /// The fields, in the order the shader must declare them.
            #[allow(dead_code)]
            pub const NAMES: &'static [&'static str] = &[$(stringify!($name)),*];

            pub fn uniform(&self) -> $uniform {
                $uniform { $($name: self.$name.srgb4(),)* }
            }
        }
    };
}

shader_paint! {
    /// The land itself, as terrain.wesl paints it.
    Land as LandUniform {
        soil,
        soil_winter,
        stubble,
        corn_green,
        wheat_ripe,
        barley_ripe,
        oats_ripe,
        roots,
        roots_winter,
        hay,
        hay_spring,
        hay_tall,
        hay_drying,
        hay_winter,
        fallow_weeds,
        pasture,
        pasture_light,
        pasture_deep,
        pasture_pale,
        grass_dry,
        grass_autumn,
        grass_winter,
        balk,
        headland,
        water_meadow,
        rock,
        snow,
        frost,
        hedge,
        hedge_autumn,
        hedge_winter,
        field_wall,
        wood,
        wood_shade,
        wood_spring,
        wood_autumn,
        wood_winter,
        heath,
        heath_hill,
        heath_mountain,
        heath_brown,
        heather_drift,
        heather_drift_bloom,
        heather,
        heather_bloom,
        gorse,
        gorse_flower,
        town_earth,
        garden,
        garden_dug,
        cobbles,
        town_far,
        yard,
        yard_cinder,
        yard_gravel,
        yard_lane,
        derelict,
        derelict_weeds,
        quarry,
        clay_pit,
        iron_mine,
        colliery,
        seabed_sand,
        seabed_shelf,
        seabed_deep,
        beach_wet,
        beach,
        shingle,
        soil_light,
        soil_red,
        rough_grass,
        wood_canopy,
        river_thread,
        shore_far,
        wood_canopy_autumn,
        moor_grass,
        bracken,
        bracken_autumn,
        rushes,
    }
}

shader_paint! {
    /// The sea's surface, as sea.wesl paints it.
    Sea as SeaUniform {
        shallows,
        shelf,
        deep,
        open,
        foam,
    }
}

// ------------------------------------------------------------------ the light and the lens

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Sun {
    pub turn: f32,
    pub height_winter: f32,
    pub height_summer_adds: f32,
    pub strength_winter: f32,
    pub strength_summer_adds: f32,
    pub colour_winter: [f32; 3],
    pub colour_summer_adds: [f32; 3],
}

impl Sun {
    /// The sun's height above the horizon, in radians; `summer` runs from 0 at midwinter to 1 at midsummer.
    pub fn height(&self, summer: f32) -> f32 {
        self.height_winter + self.height_summer_adds * summer
    }

    pub fn strength(&self, summer: f32) -> f32 {
        self.strength_winter + self.strength_summer_adds * summer
    }

    pub fn colour(&self, summer: f32) -> Color {
        let (w, a) = (self.colour_winter, self.colour_summer_adds);
        Color::srgb(w[0] + a[0] * summer, w[1] + a[1] * summer, w[2] + a[2] * summer)
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Shadows {
    pub cascades: usize,
    pub first_far: f32,
    pub max_distance: f32,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Sky {
    pub background: Colour,
    pub light: Colour,
    pub brightness: f32,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Haze {
    pub colour: Colour,
    pub sun_glow: Colour,
    pub sun_glow_strength: f32,
    pub sun_glow_tightness: f32,
    pub start: f32,
    pub end: f32,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Tonemap {
    None,
    Linear,
    Reinhard,
    ReinhardLuminance,
    AcesFitted,
    Agx,
    SomewhatBoringDisplayTransform,
    TonyMcMapface,
    BlenderFilmic,
    KhronosPbrNeutral,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Post {
    pub tonemapping: Tonemap,
    pub exposure: f32,
    pub bloom: f32,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AmbientOcclusion {
    pub on: bool,
    pub quality: Quality,
    pub radius: f32,
    pub thickness: f32,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TiltShift {
    pub on: bool,
    pub focus: f32,
    pub widest: f32,
    pub narrowest: f32,
}

impl TiltShift {
    /// The lens opening at a camera distance: shallow focus when far away, gentler up close.
    pub fn f_stops(&self, dist: f32) -> f32 {
        (self.focus / dist).clamp(self.widest, self.narrowest)
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Surfaces {
    pub buildings: f32,
    pub paths: f32,
    pub foliage: f32,
    pub boats_and_marks: f32,
    pub painted_reflectance: f32,
    pub ground: f32,
    pub ground_reflectance: f32,
    pub river: f32,
    pub river_reflectance: f32,
    pub sea: f32,
    pub sea_reflectance: f32,
    pub smoke_translucence: f32,
    pub bare_limb_translucence: f32,
}

// ------------------------------------------------------------------ the whole look

/// The whole look. The scenery's named colours are the palette, which the
/// generators take; the look stands for it (`look.brick.red` is the
/// palette's brick), so the scenery code reads its colours as it always has.
#[derive(Clone, Debug)]
pub struct Look {
    pub palette: Palette,
    pub land: Land,
    pub sea: Sea,
    pub sun: Sun,
    pub shadows: Shadows,
    pub sky: Sky,
    pub haze: Haze,
    pub post: Post,
    pub ambient_occlusion: AmbientOcclusion,
    pub tilt_shift: TiltShift,
    pub surfaces: Surfaces,
}

impl std::ops::Deref for Look {
    type Target = Palette;
    fn deref(&self) -> &Palette {
        &self.palette
    }
}

/// The viewer's half of the look file: what is left once the palette's
/// tables are taken out.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewerHalf {
    land: Land,
    sea: Sea,
    sun: Sun,
    shadows: Shadows,
    sky: Sky,
    haze: Haze,
    post: Post,
    ambient_occlusion: AmbientOcclusion,
    tilt_shift: TiltShift,
    surfaces: Surfaces,
}

impl Look {
    /// The look built into the viewer.
    pub fn standard() -> Look {
        // the file is part of the program: if it is wrong the program is, and the tests say so
        Look::from_layers(&[("the built-in look", STANDARD)]).unwrap_or_else(|e| panic!("{e}"))
    }

    /// The standard look with a file laid over it. The file may name only what
    /// it changes; a name the standard look does not have is an error.
    pub fn with_file(path: &str) -> Result<Look, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read the look file {path}: {e}"))?;
        Look::from_layers(&[("the built-in look", STANDARD), (path, &text)])
    }

    /// Lay each text over the ones before it, then read the result. The first
    /// must be whole; the others may be partial.
    fn from_layers(layers: &[(&str, &str)]) -> Result<Look, String> {
        let mut whole = toml::Table::new();
        for (i, (name, text)) in layers.iter().enumerate() {
            let table: toml::Table = toml::from_str(text).map_err(|e| format!("{name}: {e}"))?;
            if i == 0 {
                whole = table;
            } else {
                overlay(&mut whole, table, "").map_err(|e| format!("{name}: {e}"))?;
            }
        }
        let name = layers.last().map_or("the look", |l| l.0);
        // TOML has nan and inf for numbers; a light or a lens set to one draws nothing, or stops the viewer
        if let Some(path) = not_a_number(&whole, "") {
            return Err(format!("{name}: `{path}` must be a number, not nan or inf"));
        }
        // the palette's tables first, then what is left, which is the viewer's
        let palette = Palette::take_from(&mut whole).map_err(|e| format!("{name}: {}", e.trim_end()))?;
        let v: ViewerHalf = whole.try_into().map_err(|e: toml::de::Error| format!("{name}: {}", e.to_string().trim_end()))?;
        let look = Look {
            palette,
            land: v.land,
            sea: v.sea,
            sun: v.sun,
            shadows: v.shadows,
            sky: v.sky,
            haze: v.haze,
            post: v.post,
            ambient_occlusion: v.ambient_occlusion,
            tilt_shift: v.tilt_shift,
            surfaces: v.surfaces,
        };
        look.check().map_err(|e| format!("{name}: {e}"))?;
        Ok(look)
    }

    /// What the types cannot say (the palette checks its own lists of tints).
    /// These are the settings that would otherwise stop the viewer part way
    /// in, where Bevy or the lens arithmetic refuses them.
    fn check(&self) -> Result<(), String> {
        let s = &self.shadows;
        if s.cascades == 0 || s.cascades > 4 {
            return Err("`shadows.cascades` must be from 1 to 4".into());
        }
        // Bevy's shadows begin 0.1 from the camera
        if s.first_far <= 0.1 || s.max_distance < s.first_far {
            return Err("`shadows.first_far` must be more than 0.1, and `shadows.max_distance` no less than it".into());
        }
        let t = &self.tilt_shift;
        if t.widest <= 0.0 || t.narrowest < t.widest {
            return Err("`tilt_shift.widest` must be more than 0, and `tilt_shift.narrowest` no less than it".into());
        }
        if self.haze.end <= self.haze.start {
            return Err("`haze.end` must be more than `haze.start`".into());
        }
        if self.ambient_occlusion.radius <= 0.0 {
            return Err("`ambient_occlusion.radius` must be more than 0".into());
        }
        Ok(())
    }
}

/// The first number in a table that is nan or inf, by its path ("sun.turn").
fn not_a_number(table: &toml::Table, at: &str) -> Option<String> {
    fn bad(v: &toml::Value) -> bool {
        match v {
            toml::Value::Float(f) => !f.is_finite(),
            toml::Value::Array(a) => a.iter().any(bad),
            _ => false,
        }
    }
    for (key, value) in table {
        let path = if at.is_empty() { key.clone() } else { format!("{at}.{key}") };
        match value {
            toml::Value::Table(t) => {
                if let Some(p) = not_a_number(t, &path) {
                    return Some(p);
                }
            }
            v if bad(v) => return Some(path),
            _ => {}
        }
    }
    None
}

/// Lay `over` on `base`, key by key. Every key laid over must be there already
/// and be of the same kind (a table over a table, a value over a value).
fn overlay(base: &mut toml::Table, over: toml::Table, at: &str) -> Result<(), String> {
    for (key, value) in over {
        let path = if at.is_empty() { key.clone() } else { format!("{at}.{key}") };
        let Some(slot) = base.get_mut(&key) else {
            let near = base.keys().map(|k| (distance(k, &key), k)).filter(|(d, _)| *d <= 3).min();
            let hint = match near {
                Some((_, k)) => format!(" (did you mean `{k}`?)"),
                None => String::new(),
            };
            let what = if at.is_empty() { "the look has no table".to_string() } else { format!("`{at}` has no") };
            return Err(format!("{what} `{key}`{hint}"));
        };
        match (slot, value) {
            (toml::Value::Table(b), toml::Value::Table(o)) => overlay(b, o, &path)?,
            (toml::Value::Table(_), _) => return Err(format!("`{path}` is a table of settings, not one value")),
            (_, toml::Value::Table(_)) => return Err(format!("`{path}` is one value, not a table")),
            (slot, value) => *slot = value,
        }
    }
    Ok(())
}

/// How many letters must change to turn one word into another.
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut prev = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let cur = row[j];
            row[j] = (prev + (a[i - 1] != b[j - 1]) as usize).min(row[j] + 1).min(row[j - 1] + 1);
            prev = cur;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_standard_look_reads() {
        let look = Look::standard();
        // a hex colour is the same linear colour meshkit::hex gives
        assert_eq!(*look.brick.red, crate::meshkit::hex(0xb5634c));
        assert_eq!(look.sky.light.color(), Color::srgb(0.78, 0.86, 1.0));
        assert_eq!(look.post.tonemapping, Tonemap::TonyMcMapface);
    }

    #[test]
    fn a_partial_file_changes_only_what_it_names() {
        let look = Look::from_layers(&[("standard", STANDARD), ("mine", "[brick]\nred = \"#000000\"\n[sun]\nturn = 1.0\n")]).unwrap();
        assert_eq!(*look.brick.red, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(look.sun.turn, 1.0);
        let standard = Look::standard();
        assert_eq!(look.brick.dark, standard.brick.dark);
        assert_eq!(look.sun.height_winter, standard.sun.height_winter);
    }

    #[test]
    fn an_unknown_name_is_an_error() {
        let e = Look::from_layers(&[("standard", STANDARD), ("mine", "[roofs]\ntile_rde = \"#ff0000\"\n")]).unwrap_err();
        assert!(e.contains("`roofs` has no `tile_rde`") && e.contains("tile_red"), "{e}");
        let e = Look::from_layers(&[("standard", STANDARD), ("mine", "[rooves]\ntile_red = \"#ff0000\"\n")]).unwrap_err();
        assert!(e.contains("no table `rooves`") && e.contains("roofs"), "{e}");
    }

    #[test]
    fn a_bad_value_is_an_error() {
        for bad in [
            "[brick]\nred = \"red\"\n",
            "[brick]\nred = \"#+f+f+f\"\n",
            "[brick]\nred = [1.0, 0.5]\n",
            "[brick]\nred = [1.5, 0.5, 0.0]\n",
            "[brick]\nred = 3\n",
            "[post]\ntonemapping = \"film\"\n",
            "[foliage]\nconifer = []\n",
            "[brick.red]\nx = 1\n",
            "brick = 1\n",
        ] {
            assert!(Look::from_layers(&[("standard", STANDARD), ("mine", bad)]).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_setting_that_would_stop_the_viewer_is_an_error() {
        let read = |over: &str| Look::from_layers(&[("standard", STANDARD), ("mine", over)]);
        // the lens opening is kept between widest and narrowest, which must be that way round
        assert!(read("[tilt_shift]\nwidest = 5.0\nnarrowest = 1.0\n").unwrap_err().contains("tilt_shift.widest"));
        assert!(read("[haze]\nstart = 4.0\nend = 1.0\n").unwrap_err().contains("haze.end"));
        assert!(read("[shadows]\ncascades = 9\n").unwrap_err().contains("shadows.cascades"));
        assert!(read("[shadows]\nmax_distance = 0.0\n").unwrap_err().contains("shadows.first_far"));
        assert!(read("[ambient_occlusion]\nradius = 0.0\n").unwrap_err().contains("ambient_occlusion.radius"));
        // nan and inf are numbers to TOML
        assert!(read("[sun]\nstrength_winter = nan\n").unwrap_err().contains("`sun.strength_winter`"));
        assert!(read("[foliage]\ntint_winter = [1.0, inf, 1.0]\n").unwrap_err().contains("`foliage.tint_winter`"));
        // and an error says where, without a tail of blank lines
        let e = read("[sun]\nturn = \"west\"\n").unwrap_err();
        assert!(e.contains("sun.turn") && !e.ends_with('\n'), "{e:?}");
    }

    /// The names of a struct's fields in a shader's source, in order.
    fn shader_fields(source: &str, name: &str) -> Vec<String> {
        let start = source.find(&format!("struct {name} {{")).unwrap_or_else(|| panic!("no struct {name}"));
        let body = &source[start..];
        let body = &body[body.find('{').unwrap() + 1..body.find('}').unwrap()];
        body.lines().filter_map(|l| l.trim().split_once(':')).map(|(n, _)| n.trim().to_string()).collect()
    }

    #[test]
    fn the_shaders_declare_the_same_colours_in_the_same_order() {
        assert_eq!(shader_fields(include_str!("terrain.wesl"), "LandPaint"), Land::NAMES);
        assert_eq!(shader_fields(include_str!("sea.wesl"), "SeaPaint"), Sea::NAMES);
    }

    #[test]
    fn colours_may_be_numbers() {
        let c: Colour = toml::from_str::<toml::Table>("c = [1.0, 0.5, 0.0]").unwrap()["c"].clone().try_into().unwrap();
        assert_eq!(c.color(), Color::srgb(1.0, 0.5, 0.0));
        assert_eq!(c[3], 1.0);
    }
}
