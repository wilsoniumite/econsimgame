//! The look: every colour the scenery is built in, and the light and the lens
//! it is seen through, read from one file. The standard look is
//! `assets/look.toml`, built into the viewer as the font is; `--look FILE`
//! lays another file over it, which need name only what it changes.
//!
//! The scenery code takes the look as an argument and never keeps colours of
//! its own, so trying a direction is a change of data. The look is shared in
//! an `Arc`, as a year's scenery is built off the main thread.
//!
//! The ground and the sea are painted by shaders; their colours are tables of
//! the look like any other, handed to the shaders as uniforms (`Land`, `Sea`).
//!
//! Not here: the interface's colours (ui.rs, family.rs, game.rs, icons.rs).

use std::sync::Arc;

use bevy::prelude::{Color, Deref, Resource};
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::Deserialize;

/// The standard look, as the file in the source tree has it.
pub const STANDARD: &str = include_str!("../assets/look.toml");

/// The look in use, where systems can reach it.
#[derive(Resource, Clone, Deref)]
pub struct TheLook(pub Arc<Look>);

// ------------------------------------------------------------------ a colour

/// A colour from the look file: "#rrggbb", or [red, green, blue] from 0 to 1,
/// both in sRGB as a painter would pick them. It stands for its linear RGBA,
/// which is what vertex colours want (`*look.brick.red`), and gives a Bevy
/// colour for lights and materials (`look.sky.light.color()`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colour {
    linear: [f32; 4],
    srgb: [f32; 3],
}

impl Colour {
    pub fn from_srgb(srgb: [f32; 3]) -> Colour {
        let f = |s: f32| if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) };
        Colour { linear: [f(srgb[0]), f(srgb[1]), f(srgb[2]), 1.0], srgb }
    }

    fn from_hex(s: &str) -> Option<Colour> {
        let h = s.strip_prefix('#')?;
        if h.len() != 6 || !h.is_ascii() {
            return None;
        }
        let v = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok().map(|b| b as f32 / 255.0);
        Some(Colour::from_srgb([v(0)?, v(2)?, v(4)?]))
    }

    /// The colour for a light or a material.
    pub fn color(self) -> Color {
        Color::srgb(self.srgb[0], self.srgb[1], self.srgb[2])
    }

    /// The colour as a shader wants it in a uniform: sRGB, as the ground and
    /// sea shaders mix their colours, with a 1 to fill the vector.
    pub fn srgb4(self) -> bevy::math::Vec4 {
        bevy::math::Vec4::new(self.srgb[0], self.srgb[1], self.srgb[2], 1.0)
    }
}

impl std::ops::Deref for Colour {
    type Target = [f32; 4];
    fn deref(&self) -> &[f32; 4] {
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
pub fn pick(list: &[Colour], v: f32) -> [f32; 4] {
    *list[(v * list.len() as f32) as usize % list.len()]
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

// ------------------------------------------------------------------ the scenery

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Ground {
    pub road_gravel: Colour,
    pub lane_dust: Colour,
    pub farm_lane: Colour,
    pub street_earth: Colour,
    pub market_paving: Colour,
    pub farmyard: Colour,
    pub farmyard_edge: Colour,
    pub station_platform: Colour,
    pub spoil_coal: Colour,
    pub spoil_ironstone: Colour,
    pub spoil_stone: Colour,
    pub spoil_clay: Colour,
    pub charcoal_clamp: Colour,
    pub charcoal_clamp_top: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Walls {
    pub cream: Colour,
    pub limewash: Colour,
    pub ochre: Colour,
    pub rose: Colour,
    pub sage: Colour,
    pub station: Colour,
    pub market_hall: Colour,
    pub weaver_cottage: Colour,
    pub lighthouse_white: Colour,
    pub lighthouse_red: Colour,
}

impl Walls {
    /// The wash of a separate house in town, by a number of its own; cream comes round twice.
    pub fn town_house(&self, v: f32) -> [f32; 4] {
        *[self.cream, self.limewash, self.ochre, self.rose, self.sage, self.cream][(v * 6.0) as usize % 6]
    }

    /// The wash of a lowland farmhouse.
    pub fn farmhouse(&self, v: f32) -> [f32; 4] {
        *[self.cream, self.limewash, self.ochre, self.rose, self.sage][(v * 5.0) as usize % 5]
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Roofs {
    pub tile_red: Colour,
    pub tile_dark: Colour,
    pub tile_farm: Colour,
    pub moss: Colour,
    pub barn: Colour,
    pub station_canopy: Colour,
    pub drying_shed: Colour,
    pub coke_ovens: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Thatch {
    pub farmhouse: Colour,
    pub weaver_cottage: Colour,
    pub rick: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Slate {
    pub house: Colour,
    pub town: Colour,
    pub works: Colour,
    pub upland_farm: Colour,
    pub upland_barn: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Timber {
    pub boards: Colour,
    pub scaffold_pole: Colour,
    pub scaffold_board: Colour,
    pub headframe: Colour,
    pub barn_tarred: Colour,
    pub door: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Stone {
    pub church: Colour,
    pub market_cross: Colour,
    pub works: Colour,
    pub shed: Colour,
    pub upland_farm: Vec<Colour>,
    pub upland_barn: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Brick {
    pub red: Colour,
    pub dark: Colour,
    pub shed: Colour,
    pub terrace_brown: Colour,
    pub terrace_orange: Colour,
    pub terrace_sooty: Colour,
    pub chimney: Colour,
    pub barn: Colour,
}

impl Brick {
    /// The brick of a terrace (or of a brick house on a high street), from fresh red to sooty brown.
    pub fn terrace(&self, v: f32) -> [f32; 4] {
        *[self.red, self.terrace_brown, self.terrace_orange, self.terrace_sooty][(v * 4.0) as usize % 4]
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Glass {
    pub window: Colour,
    pub mill_lit: Colour,
    pub lighthouse_lamp: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Iron {
    pub winding_gear: Colour,
    pub converter: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Foliage {
    pub trunk: Colour,
    pub broadleaf: Vec<Colour>,
    pub conifer: Vec<Colour>,
    pub gorse: Colour,
    pub broom: Colour,
    pub tint_winter: [f32; 3],
    pub tint_spring: [f32; 3],
    pub tint_summer: [f32; 3],
    pub tint_autumn: [f32; 3],
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Water {
    pub river: Colour,
    pub river_bank: Colour,
    pub sea_floor: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Smoke {
    pub puff: Colour,
    pub steam: Colour,
    pub soot: Colour,
    pub stack_top: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Rail {
    pub ballast: Colour,
    pub rails: Colour,
    pub earthworks: Colour,
    pub bank_top: Colour,
    pub bank_grass: Colour,
    pub bank_grass_shade: Colour,
    pub stone: Colour,
    pub viaduct_brick: Colour,
    pub cutting_grass_lit: Colour,
    pub cutting_grass_shade: Colour,
    pub cutting_rock_lit: Colour,
    pub cutting_rock_shade: Colour,
    pub cutting_back: Colour,
    pub tunnel_mouth: Colour,
    pub girder: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Bridges {
    pub road_stone: Colour,
    pub lane_stone: Colour,
    pub arch_shadow: Colour,
    pub causeway: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Vehicles {
    pub engine: Colour,
    pub engine_black: Colour,
    pub wagons: Vec<Colour>,
    pub cart_wood: Colour,
    pub cart_canvas: Colour,
    pub cart_wheels: Colour,
    pub horse: Colour,
    pub boat_hull: Colour,
    pub boat_deck: Colour,
    pub boat_mast: Colour,
    pub sail: Colour,
    pub jib: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Marks {
    pub pole: Colour,
    pub finial: Colour,
    pub cloth_pale: Colour,
    pub post_shade: Colour,
    pub peg: Colour,
    pub hover: Colour,
    pub hover_alpha: f32,
    pub outline_plain: Colour,
    pub outline_alpha: f32,
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
}

// ------------------------------------------------------------------ the whole look

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Look {
    pub land: Land,
    pub sea: Sea,
    pub ground: Ground,
    pub walls: Walls,
    pub roofs: Roofs,
    pub thatch: Thatch,
    pub slate: Slate,
    pub timber: Timber,
    pub stone: Stone,
    pub brick: Brick,
    pub glass: Glass,
    pub iron: Iron,
    pub foliage: Foliage,
    pub water: Water,
    pub smoke: Smoke,
    pub rail: Rail,
    pub bridges: Bridges,
    pub vehicles: Vehicles,
    pub marks: Marks,
    pub sun: Sun,
    pub shadows: Shadows,
    pub sky: Sky,
    pub haze: Haze,
    pub post: Post,
    pub ambient_occlusion: AmbientOcclusion,
    pub tilt_shift: TiltShift,
    pub surfaces: Surfaces,
}

impl Look {
    /// The roof of a separate house in town, by a number of its own: mostly
    /// red tile, some slate, the odd one gone green.
    pub fn house_roof(&self, u: f32) -> [f32; 4] {
        *[self.roofs.tile_red, self.slate.house, self.roofs.tile_red, self.roofs.moss, self.roofs.tile_dark][(u * 13.0) as usize % 5]
    }

    /// The walls of a shed in a works yard: brick of one sort or another, or stone.
    pub fn yard_shed_wall(&self, v: f32) -> [f32; 4] {
        *[self.brick.red, self.brick.shed, self.stone.shed, self.brick.dark][(v * 4.0) as usize % 4]
    }

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
        let look: Look = whole.try_into().map_err(|e| format!("{name}: {e}"))?;
        look.check().map_err(|e| format!("{name}: {e}"))?;
        Ok(look)
    }

    /// What the types cannot say: a list of tints needs a tint in it.
    fn check(&self) -> Result<(), String> {
        for (name, list) in [
            ("foliage.broadleaf", &self.foliage.broadleaf),
            ("foliage.conifer", &self.foliage.conifer),
            ("stone.upland_farm", &self.stone.upland_farm),
            ("vehicles.wagons", &self.vehicles.wagons),
        ] {
            if list.is_empty() {
                return Err(format!("`{name}` needs at least one colour"));
            }
        }
        if self.shadows.cascades == 0 || self.shadows.cascades > 4 {
            return Err("`shadows.cascades` must be from 1 to 4".into());
        }
        Ok(())
    }
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
        for bad in ["[brick]\nred = \"red\"\n", "[brick]\nred = [1.0, 0.5]\n", "[brick]\nred = 3\n", "[post]\ntonemapping = \"film\"\n", "[foliage]\nconifer = []\n", "[brick.red]\nx = 1\n", "brick = 1\n"] {
            assert!(Look::from_layers(&[("standard", STANDARD), ("mine", bad)]).is_err(), "{bad}");
        }
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
