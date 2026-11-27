//! The island's settings (not in the Python; docs/rules/land.md, "The island's
//! settings"): its land, its riches and its people at the start, a short list
//! of dials said one way everywhere.
//!
//! Until 27 November 2026 an island was its number and nothing else: the land's
//! numbers were in the rules, where `--set land.rain=600` reached them for those
//! who knew the name, and a game had no argument for them and its save did not
//! remember them. These are the few that are safe to turn and worth turning,
//! each with a range every island 1 to 8 finishes a run at both ends of. Each is
//! a field of the parameters (`Params`) and nothing else, so there is one truth:
//! [`Island::set`] writes them there and [`Island::read`] reads them back.
//!
//! They are said `NAME=VALUE`, several with commas between them, in the same
//! words everywhere:
//!
//! - `isostate-core run --island hills=1400,rain=600`;
//! - `isostate-core sweep --island ...` for every run, and `--set
//!   island.rain=450,900,1350` to sweep one;
//! - `isostate-viewer --island ...` for a new game;
//! - the record's header, `meta.island`, and a saved game's `island`, each an
//!   object of the settings that are not as the island comes and written only
//!   when there is one, so that a run or a save at the defaults is the file it
//!   was, and every save made before reads as before.
//!
//! The start itself (settled or landfall) is the eleventh choice of the list
//! and keeps its own words (`--start landfall`, `--landfall`, a save's
//! `"landfall": true`), which came first.

use serde::{Deserialize, Serialize};

use crate::params::Params;

/// Which part of the island a setting is of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The ground and its water: acts only on the new land (`land.enabled`).
    Land,
    /// What is dug and cut: on the new land only, too.
    Riches,
    /// The people of 1750 and their towns.
    People,
}

/// One setting, as the command line, the record, a save and the start page say it.
#[derive(Clone, Copy, Debug)]
pub struct Setting {
    /// Its name, as `--island NAME=VALUE` takes it.
    pub name: &'static str,
    pub part: Part,
    /// Its term in the glossary (data/words.toml), by its key; empty where it has none yet.
    pub term: &'static str,
    /// What its number counts.
    pub unit: &'static str,
    /// As the island comes (on the settled island; see `default_of` for people).
    pub default: f64,
    /// The least and the most it may be set to, both tried on islands 1 to 8.
    pub least: f64,
    pub most: f64,
    /// What it does, in a sentence.
    pub does: &'static str,
}

/// The settings, in the order they are written. The ranges are those every
/// island 1 to 8 finished a run to 1900 at both ends of, on the start the
/// setting applies to (docs/rules/land.md, "The island's settings").
pub const SETTINGS: [Setting; 11] = [
    Setting { name: "hills", part: Part::Land, term: "hills", unit: "m", default: 1000.0, least: 500.0, most: 1600.0, does: "The height the mountain belt is lifted towards, metres (land.relief)." },
    Setting { name: "rain", part: Part::Land, term: "rain", unit: "mm a year", default: 900.0, least: 450.0, most: 1800.0, does: "Rain on the windward coast at sea level, mm a year (land.rain); more falls on the hills and less in their lee." },
    Setting { name: "wander", part: Part::Land, term: "meander", unit: "times", default: 1.0, least: 0.0, most: 2.0, does: "How freely the rivers wander: the meander steps taken this many times over (land.wander); 0 leaves them where the erosion put them." },
    Setting { name: "lakes", part: Part::Land, term: "lake", unit: "times", default: 1.0, least: 0.0, most: 3.0, does: "How many lakes: the upland lakes looked for and the chance of the lowland lake, this many times over (land.lakes)." },
    Setting { name: "coal", part: Part::Riches, term: "coal", unit: "times", default: 1.0, least: 0.25, most: 2.5, does: "How rich the coalfields are, and how far they reach (land.riches.coal)." },
    Setting { name: "ore", part: Part::Riches, term: "ore", unit: "times", default: 1.0, least: 0.25, most: 2.5, does: "How rich the ore bodies are, and how far they reach (land.riches.ore)." },
    Setting { name: "stone", part: Part::Riches, term: "stone", unit: "times", default: 1.0, least: 0.25, most: 2.5, does: "How much stone lies near the surface (land.riches.stone)." },
    Setting { name: "clay", part: Part::Riches, term: "clay", unit: "times", default: 1.0, least: 0.25, most: 2.5, does: "How much clay lies in the vales (land.riches.clay)." },
    Setting { name: "woods", part: Part::Riches, term: "woodland", unit: "times", default: 1.0, least: 0.25, most: 2.0, does: "How thick the woods would grow if left (land.riches.woods); every field stays at most full." },
    Setting { name: "people", part: Part::People, term: "", unit: "people", default: 135_000.0, least: 30_000.0, most: 540_000.0, does: "The people of 1750 (pop0 on the settled island, start.people at landfall), shared among the first towns by the corn land round each." },
    Setting { name: "towns", part: Part::People, term: "market_town", unit: "towns", default: 6.0, least: 1.0, most: 12.0, does: "The first towns of the settled island (n_towns); more than six stand closer than 17 cells. Landfall has one." },
];

/// The people at landfall: as the start comes, and the range tried.
pub const LANDFALL_PEOPLE: (f64, f64, f64) = (5_000.0, 2_000.0, 20_000.0);

/// The settings an island is made with, each None where it is as the island
/// comes. Written to a record or a save with only those that are not None.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Island {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hills: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rain: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wander: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lakes: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coal: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ore: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stone: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clay: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub woods: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub people: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub towns: Option<f64>,
}

/// The setting by its name.
pub fn setting(name: &str) -> Option<(usize, &'static Setting)> {
    SETTINGS.iter().enumerate().find(|(_, s)| s.name == name)
}

/// The names, for a sentence that lists them.
pub fn names() -> String {
    SETTINGS.iter().map(|s| s.name).collect::<Vec<_>>().join(", ")
}

/// A setting's value as the island comes, on this start.
pub fn default_of(i: usize, landfall: bool) -> f64 {
    if SETTINGS[i].name == "people" && landfall { LANDFALL_PEOPLE.0 } else { SETTINGS[i].default }
}

/// The least and the most a setting may be, on this start.
pub fn range_of(i: usize, landfall: bool) -> (f64, f64) {
    if SETTINGS[i].name == "people" && landfall { (LANDFALL_PEOPLE.1, LANDFALL_PEOPLE.2) } else { (SETTINGS[i].least, SETTINGS[i].most) }
}

impl Island {
    /// Every setting as the island comes.
    pub fn plain(&self) -> bool {
        *self == Island::default()
    }

    /// The `i`-th setting of `SETTINGS`.
    pub fn get(&self, i: usize) -> Option<f64> {
        *self.slot_ref(i)
    }

    fn slot_ref(&self, i: usize) -> &Option<f64> {
        match i {
            0 => &self.hills,
            1 => &self.rain,
            2 => &self.wander,
            3 => &self.lakes,
            4 => &self.coal,
            5 => &self.ore,
            6 => &self.stone,
            7 => &self.clay,
            8 => &self.woods,
            9 => &self.people,
            _ => &self.towns,
        }
    }

    /// The `i`-th setting, to be set.
    pub fn slot(&mut self, i: usize) -> &mut Option<f64> {
        match i {
            0 => &mut self.hills,
            1 => &mut self.rain,
            2 => &mut self.wander,
            3 => &mut self.lakes,
            4 => &mut self.coal,
            5 => &mut self.ore,
            6 => &mut self.stone,
            7 => &mut self.clay,
            8 => &mut self.woods,
            9 => &mut self.people,
            _ => &mut self.towns,
        }
    }

    /// `hills=1400,rain=600`, as the command line takes it: names from
    /// `SETTINGS`, each once, each a number. The ranges are checked when the
    /// settings are laid on the rules (`set`), which know the start.
    pub fn parse(text: &str) -> Result<Island, String> {
        let mut out = Island::default();
        for pair in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let Some((name, value)) = pair.split_once('=') else {
                return Err(format!("--island {text}: write each setting as NAME=VALUE, with commas between them ({})", names()));
            };
            let (name, value) = (name.trim(), value.trim());
            let Some((i, _)) = setting(name) else {
                return Err(format!("--island: there is no setting called {name:?}; the settings are {}", names()));
            };
            let v: f64 = value.parse().ok().filter(|v: &f64| v.is_finite()).ok_or_else(|| format!("--island {name}={value}: {value} is not a number"))?;
            if out.get(i).is_some() {
                return Err(format!("--island: {name} is given twice"));
            }
            *out.slot(i) = Some(v);
        }
        Ok(out)
    }

    /// These settings with `over`'s laid on them: a setting `over` gives wins.
    pub fn and(mut self, over: &Island) -> Island {
        for i in 0..SETTINGS.len() {
            if let Some(v) = over.get(i) {
                *self.slot(i) = Some(v);
            }
        }
        self
    }

    /// The settings as the command line says them: `hills=1400,rain=600`, in
    /// the order of `SETTINGS`; empty when every one is as the island comes.
    pub fn words(&self) -> String {
        (0..SETTINGS.len()).filter_map(|i| self.get(i).map(|v| format!("{}={v}", SETTINGS[i].name))).collect::<Vec<_>>().join(",")
    }

    /// The settings `P` is made with, each None where it is as the island
    /// comes on `P`'s start. The land's and the riches' settings are read only
    /// where the land is the new land, which alone they act on; the towns only
    /// on the settled island, since landfall has one.
    pub fn read(P: &Params) -> Island {
        let landfall = P.start.landfall;
        let mut out = Island::default();
        for (i, s) in SETTINGS.iter().enumerate() {
            let v = match s.name {
                _ if s.part != Part::People && !P.land.enabled => continue,
                "hills" => P.land.relief,
                "rain" => P.land.rain,
                "wander" => P.land.wander,
                "lakes" => P.land.lakes,
                "coal" => P.land.riches.coal,
                "ore" => P.land.riches.ore,
                "stone" => P.land.riches.stone,
                "clay" => P.land.riches.clay,
                "woods" => P.land.riches.woods,
                "people" => crate::landfall::people(P),
                "towns" if landfall => continue,
                _ => P.n_towns as f64,
            };
            if v != default_of(i, landfall) {
                *out.slot(i) = Some(v);
            }
        }
        out
    }

    /// Whether these settings can be laid on `P`: each inside its range on
    /// `P`'s start, the towns a whole number and on the settled island, and
    /// the land's and the riches' only on the new land. Said in a sentence
    /// that names the setting.
    pub fn check(&self, P: &Params) -> Result<(), String> {
        let landfall = P.start.landfall;
        for (i, s) in SETTINGS.iter().enumerate() {
            let Some(v) = self.get(i) else { continue };
            let (least, most) = range_of(i, landfall);
            if !(least..=most).contains(&v) {
                let start = if s.name == "people" { if landfall { " at landfall" } else { " on the settled island" } } else { "" };
                return Err(format!("the island's {}: {v} is outside its range{start}, {least} to {most} {}", s.name, s.unit));
            }
            if s.part != Part::People && !P.land.enabled {
                return Err(format!("the island's {}: it acts on the new land only, and this island is the old one", s.name));
            }
            if s.name == "towns" {
                if v.fract() != 0.0 {
                    return Err(format!("the island's towns: {v} is not a whole number of towns"));
                }
                if landfall {
                    return Err("the island's towns: landfall begins with one town, so the number of first towns is the settled island's".to_string());
                }
                if !P.land.enabled {
                    return Err("the island's towns: it acts on the new land only, and this island is the old one".to_string());
                }
            }
        }
        Ok(())
    }

    /// Lay these settings on `P`, a start already made (the rules, and the
    /// start laid on them), and say the settings `P` then has (`read`): a
    /// setting asked for as the island comes is None. Refused, with `P` left as
    /// it was, where `check` refuses.
    pub fn set(&self, P: &mut Params) -> Result<Island, String> {
        self.check(P)?;
        for (i, s) in SETTINGS.iter().enumerate() {
            let Some(v) = self.get(i) else { continue };
            match s.name {
                "hills" => P.land.relief = v,
                "rain" => P.land.rain = v,
                "wander" => P.land.wander = v,
                "lakes" => P.land.lakes = v,
                "coal" => P.land.riches.coal = v,
                "ore" => P.land.riches.ore = v,
                "stone" => P.land.riches.stone = v,
                "clay" => P.land.riches.clay = v,
                "woods" => P.land.riches.woods = v,
                "people" if P.start.landfall => P.start.people = v,
                "people" => P.pop0 = v.round() as i64,
                _ => P.n_towns = v as usize,
            }
        }
        Ok(Island::read(P))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::game_params;

    #[test]
    fn the_island_as_it_comes_reads_as_nothing_set() {
        assert!(Island::read(&game_params()).plain());
        let mut P = game_params();
        crate::landfall::landfall(&mut P);
        assert!(Island::read(&P).plain());
        // and setting nothing changes nothing
        let mut Q = game_params();
        assert!(Island::default().set(&mut Q).unwrap().plain());
        assert_eq!(Q, game_params());
        // nor does setting each to its default
        let all = Island::parse("hills=1000,rain=900,wander=1,lakes=1,coal=1,ore=1,stone=1,clay=1,woods=1,people=135000,towns=6").unwrap();
        let mut R = game_params();
        assert!(all.set(&mut R).unwrap().plain());
        assert_eq!(R, game_params());
        assert_eq!(serde_json::to_string(&R).unwrap(), serde_json::to_string(&game_params()).unwrap());
    }

    #[test]
    fn settings_are_said_and_read_back_the_same() {
        let s = Island::parse("rain=600, hills=1400,coal=2,people=270000,towns=8").unwrap();
        assert_eq!(s.words(), "hills=1400,rain=600,coal=2,people=270000,towns=8");
        assert_eq!(Island::parse(&s.words()).unwrap(), s);
        let mut P = game_params();
        assert_eq!(s.set(&mut P).unwrap(), s);
        assert_eq!((P.land.relief, P.land.rain, P.land.riches.coal, P.pop0, P.n_towns), (1400.0, 600.0, 2.0, 270_000, 8));
        assert_eq!(Island::read(&P), s);
        // in JSON, only what is set
        assert_eq!(serde_json::to_string(&Island { rain: Some(600.0), ..Island::default() }).unwrap(), r#"{"rain":600.0}"#);
        // landfall's people are its own
        let mut L = game_params();
        crate::landfall::landfall(&mut L);
        let l = Island::parse("people=8000").unwrap();
        assert_eq!(l.set(&mut L).unwrap(), l);
        assert_eq!((L.start.people, L.pop0), (8000.0, 135_000));
    }

    #[test]
    fn settings_out_of_reach_are_refused_in_words() {
        assert!(Island::parse("hils=3").unwrap_err().contains("no setting called \"hils\""));
        assert!(Island::parse("rain=wet").unwrap_err().contains("not a number"));
        assert!(Island::parse("rain=600,rain=700").unwrap_err().contains("given twice"));
        let mut P = game_params();
        assert!(Island::parse("rain=100").unwrap().set(&mut P).unwrap_err().contains("outside its range"));
        assert!(Island::parse("towns=6.5").unwrap().set(&mut P).unwrap_err().contains("whole number"));
        assert_eq!(P, game_params());
        let mut L = game_params();
        crate::landfall::landfall(&mut L);
        assert!(Island::parse("towns=3").unwrap().set(&mut L).unwrap_err().contains("landfall begins with one town"));
        assert!(Island::parse("people=135000").unwrap().set(&mut L).unwrap_err().contains("at landfall"));
        let mut old = game_params();
        old.land = crate::landform::LandParams::default();
        assert!(Island::parse("coal=2").unwrap().set(&mut old).unwrap_err().contains("new land only"));
        assert!(Island::parse("people=60000").unwrap().set(&mut old).is_ok());
    }
}
