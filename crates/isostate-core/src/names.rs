//! Procedural names for towns, people and firms. Python: `names.py`.

use std::collections::BTreeSet;

use crate::rng::{Rng, choice_index};

pub const PREFIXES: &[&str] = &[
    "Ash", "Bar", "Bel", "Black", "Bram", "Brack", "Bright", "Bur", "Cal", "Car",
    "Cold", "Crag", "Dal", "Dun", "East", "Elm", "Fair", "Fal", "Fen", "Gar",
    "Glen", "Gold", "Gran", "Grey", "Hal", "Hart", "Haw", "High", "Hol", "Kel",
    "Ken", "Kirk", "Lang", "Lark", "Lea", "Lin", "Long", "Mar", "Mel", "Mor",
    "Nor", "Oak", "Old", "Pen", "Red", "Ros", "Rye", "Sal", "Sand", "Shaw",
    "Stan", "Stone", "Strat", "Sun", "Thorn", "Tor", "Wal", "Wel", "West", "Whit",
    "Wil", "Win", "Wolf", "Wood", "Wren", "Yar", "Brin", "Cran", "Ether", "Hask",
    "Ivel", "Mallow", "Orm", "Quen", "Ruth", "Skel", "Tarn", "Ul", "Vel", "Wyr",
];

/// Suffixes by site context ("plain", "coast", "river", "hills", "mine").
pub fn suffixes(context: &str) -> &'static [&'static str] {
    match context {
        "coast" => &["mouth", "haven", "port", "sea", "wick", "ness", "strand"],
        "river" => &["ford", "bridge", "mere", "water", "brook", "bourne", "ford"],
        "hills" => &["dale", "moor", "fell", "edge", "rigg", "side", "hope"],
        "mine" => &["pit", "delf", "heath", "side", "gate", "bank"],
        _ => &[
            "ford", "by", "ton", "ham", "wick", "stead", "ley", "bury", "worth",
            "field", "thorpe", "well", "stow", "combe", "hurst", "borough", "ing",
        ],
    }
}

pub const SURNAMES: &[&str] = &[
    "Abbot", "Ainsley", "Barlow", "Beckett", "Blythe", "Booth", "Brindle", "Carver",
    "Cawley", "Crowther", "Dent", "Denton", "Elwood", "Fairbairn", "Fenwick", "Garside",
    "Hargreave", "Holroyd", "Ingham", "Kershaw", "Lister", "Marsden", "Naylor", "Oldroyd",
    "Pickles", "Platt", "Radcliffe", "Rowntree", "Sowerby", "Tennant", "Thwaite", "Ward",
    "Whitworth", "Wilkin", "Wraith", "Ackroyd", "Bramall", "Calvert", "Dawson", "Emmott",
    "Firth", "Greaves", "Haigh", "Ibbotson", "Jowett", "Kitson", "Lumb", "Mallinson",
    "Nettleton", "Oates", "Priestley", "Quarmby", "Rushworth", "Sugden", "Tetley",
    "Uttley", "Varley", "Wadsworth", "Yeadon", "Armitage", "Birkett", "Clegg", "Dobson",
    "Eastwood", "Foulds", "Gaukroger", "Horsfall", "Illingworth", "Jagger", "Kenworthy",
];

/// Name patterns by method: `{S}` and `{S2}` are surnames, `{T}` the town.
pub fn firm_patterns(method_name: &str) -> &'static [&'static str] {
    match method_name {
        "charcoal_burning" => &["{S}'s Charcoal", "{T} Colliers' Wood"],
        "bloomery" => &["{S}'s Bloomery", "{T} Bloomery"],
        "charcoal_furnace" => &["{T} Furnace", "{S} Furnace Co."],
        "coke_furnace" => &["{T} Ironworks", "{S} & Sons Ironworks", "{T} Iron Co."],
        "smithy" => &["{S}'s Forge", "{S} Edge Tools", "{T} Smithy"],
        "hand_loom" => &["{S} Clothiers", "{T} Weavers", "{S} & Co. Cloth"],
        "textile_mill" => &["{S} Mill", "{T} Mills", "{S} Brothers Mill"],
        "brickworks" => &["{T} Brickworks", "{S}'s Brickfield"],
        "millwright" => &["{S}, Millwright", "{S} & Son Millwrights"],
        "engine_works" => &["{S} Engine Works", "{T} Foundry", "{S} & {S2} Engines"],
        "crucible_steel" => &["{S} Steel", "{S}'s Crucible Works"],
        "bessemer" => &["{T} Steel Co.", "{S} Steelworks"],
        "forge_rails" => &["{S}'s Rail Forge"],
        "rolling_mill" => &["{T} Rolling Mills", "{S} Rail Works"],
        "steel_rolling" => &["{T} Rail Mills", "{S} Steel Rail Co."],
        _ => &["{S} & Co."],
    }
}

fn pick<'a>(rng: &mut Rng, items: &[&'a str]) -> &'a str {
    items[choice_index(rng, items.len())]
}

/// A new unused town name; context picks a fitting suffix list.
pub fn town_name(rng: &mut Rng, context: &str, used: &mut BTreeSet<String>) -> String {
    let suffixes = suffixes(context);
    for _ in 0..200 {
        let name = format!("{}{}", pick(rng, PREFIXES), pick(rng, suffixes));
        if !used.contains(&name) {
            used.insert(name.clone());
            return name;
        }
    }
    let name = format!("New {}{}", pick(rng, PREFIXES), pick(rng, suffixes));
    used.insert(name.clone());
    name
}

pub fn surname(rng: &mut Rng) -> String {
    pick(rng, SURNAMES).to_string()
}

pub fn firm_name(rng: &mut Rng, method_name: &str, town: &str) -> String {
    let pat = pick(rng, firm_patterns(method_name));
    let s1 = surname(rng);
    let mut s2 = surname(rng);
    while s2 == s1 {
        s2 = surname(rng);
    }
    pat.replace("{S2}", &s2).replace("{S}", &s1).replace("{T}", town)
}
