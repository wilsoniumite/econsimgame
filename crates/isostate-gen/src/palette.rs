//! The palette: every colour the scenery is built in, by name. It is the
//! colour half of the look file (the viewer's `assets/look.toml`); the light
//! and the lens are the viewer's half. A generator is handed the palette and
//! takes its colours from it by token (`*pal.brick.red`), never keeping any of
//! its own, so trying a palette is a change of data.
//!
//! Tokens are named for what they are in the world, not for where the code
//! uses them: one brick, wherever it is laid. Where a thing comes in several
//! tints the token is a list, or the palette has a function that picks one by
//! a number of the thing's own (`pal.walls.town_house(v)`).

use serde::Deserialize;

use crate::colour::{Colour, Rgba};

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
    pub churchyard_grass: Colour,
    pub spoil_coal: Colour,
    pub spoil_ironstone: Colour,
    pub spoil_stone: Colour,
    pub spoil_clay: Colour,
    pub charcoal_clamp: Colour,
    pub charcoal_clamp_top: Colour,
    pub yard_cinders: Colour,
    pub yard_setts: Colour,
    pub yard_earth: Colour,
    pub shop_pavement: Colour,
    pub charcoal_hearth: Colour,
    pub yard_weeds: Colour,
    pub clay_face: Colour,
    pub clay_bottom: Colour,
    pub slag: Colour,
    pub furnace_bank: Colour,
    pub garden: Colour,
    pub garden_bed: Colour,
    pub garden_path: Colour,
    pub back_yard: Colour,
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
    pub shop_front: Colour,
    pub lighthouse_white: Colour,
    pub lighthouse_red: Colour,
    pub window_frame: Colour,
    pub door_paints: Vec<Colour>,
}

impl Walls {
    /// The wash of a separate house in town, by a number of its own; cream comes round twice.
    pub fn town_house(&self, v: f32) -> Rgba {
        *[self.cream, self.limewash, self.ochre, self.rose, self.sage, self.cream][(v * 6.0) as usize % 6]
    }

    /// The wash of a lowland farmhouse.
    pub fn farmhouse(&self, v: f32) -> Rgba {
        *[self.cream, self.limewash, self.ochre, self.rose, self.sage][(v * 5.0) as usize % 5]
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Roofs {
    pub tile_red: Colour,
    pub tile_dark: Colour,
    pub tile_orange: Colour,
    pub tile_farm: Colour,
    pub moss: Colour,
    pub church_lead: Colour,
    pub barn: Colour,
    pub station_canopy: Colour,
    pub drying_shed: Colour,
    pub coke_ovens: Colour,
    pub stone_slate: Colour,
    pub awnings: Vec<Colour>,
    pub chimney_pot: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Thatch {
    pub farmhouse: Colour,
    pub weaver_cottage: Colour,
    pub cottage: Colour,
    pub weathered: Colour,
    pub rick: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Slate {
    pub house: Colour,
    pub town: Colour,
    pub stone: Colour,
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
    pub fence: Colour,
    pub planks: Colour,
    pub logs: Colour,
    pub waterwheel: Colour,
    pub belfry_louvre: Colour,
    pub house_frame: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Stone {
    pub church: Colour,
    pub church_spire: Colour,
    pub churchyard_wall: Colour,
    pub gravestone: Colour,
    pub market_cross: Colour,
    pub works: Colour,
    pub shed: Colour,
    pub yard_wall: Colour,
    pub limestone: Colour,
    pub upland_farm: Vec<Colour>,
    pub upland_barn: Colour,
    pub sill: Colour,
    pub plinth: Colour,
    pub garden_wall: Colour,
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
    pub fn terrace(&self, v: f32) -> Rgba {
        *[self.red, self.terrace_brown, self.terrace_orange, self.terrace_sooty][(v * 4.0) as usize % 4]
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Glass {
    pub window: Colour,
    pub mill_lit: Colour,
    pub roof_light: Colour,
    pub lighthouse_lamp: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Iron {
    pub winding_gear: Colour,
    pub converter: Colour,
    pub plate: Colour,
    pub glow: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Goods {
    pub coal: Colour,
    pub bales: Colour,
    pub cloth: Colour,
    pub bricks: Colour,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Foliage {
    pub trunk: Colour,
    pub bare_limb: Colour,
    pub broadleaf: Vec<Colour>,
    pub conifer: Vec<Colour>,
    pub yew: Colour,
    pub gorse: Colour,
    pub broom: Colour,
    pub hedge: Colour,
    pub vegetables: Colour,
    pub tint_winter: [f32; 3],
    pub tint_spring: [f32; 3],
    pub tint_summer: [f32; 3],
    pub tint_autumn: [f32; 3],
}

impl Foliage {
    /// What the leaves are multiplied by at a phase of the year: 0 midwinter,
    /// 0.25 spring, summer from 0.4 to 0.66, autumn at 0.78, and round to
    /// winter again. The leaves hold their summer green until late August
    /// and turn in a few weeks.
    pub fn tint(&self, phase: f32) -> [f32; 3] {
        let keys: [(f32, [f32; 3]); 6] =
            [(0.0, self.tint_winter), (0.25, self.tint_spring), (0.4, self.tint_summer), (0.66, self.tint_summer), (0.78, self.tint_autumn), (1.0, self.tint_winter)];
        let i = keys.iter().position(|k| k.0 > phase).unwrap_or(5).max(1);
        let (a, b) = (keys[i - 1], keys[i]);
        let t = ((phase - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
        [a.1[0] + (b.1[0] - a.1[0]) * t, a.1[1] + (b.1[1] - a.1[1]) * t, a.1[2] + (b.1[2] - a.1[2]) * t]
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Water {
    pub river: Colour,
    pub river_bank: Colour,
    pub river_tint_winter: [f32; 3],
    pub sea_floor: Colour,
}

impl Water {
    /// What the rivers are multiplied by at a phase of the year (0
    /// midwinter, 0.5 midsummer): the winter tint in the depth of winter,
    /// fading out by the end of February and in again from November.
    pub fn river_tint(&self, phase: f32) -> [f32; 3] {
        let winter = (((phase * std::f32::consts::TAU).cos() - 0.45) / 0.55).clamp(0.0, 1.0);
        self.river_tint_winter.map(|t| 1.0 + (t - 1.0) * winter)
    }
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
    pub bank_scree: Colour,
    pub stone: Colour,
    pub viaduct_brick: Colour,
    pub tunnel_mouth: Colour,
    pub girder: Colour,
    pub rope: Colour,
    pub buffer: Colour,
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
    pub fence_timber: Colour,
    pub post_shade: Colour,
    pub peg: Colour,
    pub hover: Colour,
    pub hover_alpha: f32,
    pub outline_plain: Colour,
    pub outline_alpha: f32,
}

/// Every named colour of the scenery, a table for each material or kind of thing.
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Palette {
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
    pub goods: Goods,
    pub foliage: Foliage,
    pub water: Water,
    pub smoke: Smoke,
    pub rail: Rail,
    pub bridges: Bridges,
    pub vehicles: Vehicles,
    pub marks: Marks,
}

/// The look file as it stands in the source tree. The palette's tables are
/// read out of it; the rest of it is the viewer's.
const LOOK_FILE: &str = include_str!("../../isostate-viewer/assets/look.toml");

impl Palette {
    /// The tables of the look file that are the palette's.
    pub const TABLES: &'static [&'static str] =
        &["ground", "walls", "roofs", "thatch", "slate", "timber", "stone", "brick", "glass", "iron", "goods", "foliage", "water", "smoke", "rail", "bridges", "vehicles", "marks"];

    /// The walls of a shed in a works yard: brick of one sort or another, or stone.
    pub fn yard_shed_wall(&self, v: f32) -> Rgba {
        *[self.brick.red, self.brick.shed, self.stone.shed, self.brick.dark][(v * 4.0) as usize % 4]
    }

    /// The palette of the standard look, as the file in the source tree has it.
    pub fn standard() -> Palette {
        // the file is part of the program: if it is wrong the program is, and the tests say so
        let mut look: toml::Table = toml::from_str(LOOK_FILE).unwrap_or_else(|e| panic!("the built-in look: {e}"));
        Palette::take_from(&mut look).unwrap_or_else(|e| panic!("the built-in look: {e}"))
    }

    /// Take the palette's tables out of a look file that has been read, leaving
    /// the viewer's tables behind.
    pub fn take_from(look: &mut toml::Table) -> Result<Palette, String> {
        let mut mine = toml::Table::new();
        for name in Palette::TABLES {
            if let Some(t) = look.remove(*name) {
                mine.insert((*name).to_string(), t);
            }
        }
        let palette: Palette = mine.try_into().map_err(|e| e.to_string())?;
        palette.check()?;
        Ok(palette)
    }

    /// What the types cannot say: a list of tints needs a tint in it.
    fn check(&self) -> Result<(), String> {
        for (name, list) in [
            ("foliage.broadleaf", &self.foliage.broadleaf),
            ("foliage.conifer", &self.foliage.conifer),
            ("stone.upland_farm", &self.stone.upland_farm),
            ("vehicles.wagons", &self.vehicles.wagons),
            ("roofs.awnings", &self.roofs.awnings),
            ("walls.door_paints", &self.walls.door_paints),
        ] {
            if list.is_empty() {
                return Err(format!("`{name}` needs at least one colour"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::hex;

    #[test]
    fn the_standard_palette_reads() {
        let pal = Palette::standard();
        assert_eq!(*pal.brick.red, hex(0xb5634c));
        assert_eq!(pal.foliage.broadleaf.len(), 5);
        assert_eq!(pal.walls.town_house(0.0), *pal.walls.cream);
        assert_eq!(pal.walls.town_house(0.99), *pal.walls.cream);
    }

    #[test]
    fn the_look_file_keeps_its_other_tables() {
        let mut look: toml::Table = toml::from_str(LOOK_FILE).unwrap();
        Palette::take_from(&mut look).unwrap();
        // what is left is the viewer's: the land and sea paints, the light and the lens
        for name in Palette::TABLES {
            assert!(!look.contains_key(*name));
        }
        assert!(look.contains_key("sun") && look.contains_key("land"));
    }

    #[test]
    fn a_missing_or_misnamed_colour_is_an_error() {
        let mut look: toml::Table = toml::from_str(LOOK_FILE).unwrap();
        look["brick"].as_table_mut().unwrap().remove("red");
        assert!(Palette::take_from(&mut look).unwrap_err().contains("red"));
        let mut look: toml::Table = toml::from_str(LOOK_FILE).unwrap();
        look["brick"].as_table_mut().unwrap().insert("rde".into(), toml::Value::String("#000000".into()));
        assert!(Palette::take_from(&mut look).is_err());
        let mut look: toml::Table = toml::from_str(LOOK_FILE).unwrap();
        look["foliage"].as_table_mut().unwrap().insert("conifer".into(), toml::Value::Array(vec![]));
        assert!(Palette::take_from(&mut look).unwrap_err().contains("foliage.conifer"));
    }

    #[test]
    fn the_rivers_are_steel_in_midwinter_only() {
        let w = Palette::standard().water;
        assert_eq!(w.river_tint(0.0), w.river_tint_winter);
        assert_eq!(w.river_tint(0.5), [1.0; 3]);
        assert_eq!(w.river_tint(0.25), [1.0; 3]);
        assert_eq!(w.river_tint(0.8), [1.0; 3]);
        assert!(w.river_tint(0.95)[0] < 1.0 && w.river_tint(0.95)[0] > w.river_tint_winter[0]);
    }

    #[test]
    fn the_leaves_turn_through_the_year() {
        let f = Palette::standard().foliage;
        assert_eq!(f.tint(0.0), f.tint_winter);
        assert_eq!(f.tint(0.5), f.tint_summer);
        assert_eq!(f.tint(0.25), f.tint_spring);
        let between = f.tint(0.125);
        assert!((between[1] - (f.tint_winter[1] + f.tint_spring[1]) * 0.5).abs() < 1e-6);
        // green through the summer, gold in October
        assert_eq!(f.tint(0.64), f.tint_summer);
        assert_eq!(f.tint(0.78), f.tint_autumn);
        assert!(f.tint(0.72)[0] > 1.2 && f.tint(0.78)[0] > f.tint(0.78)[1] * 1.5);
    }
}
