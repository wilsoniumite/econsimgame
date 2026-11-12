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

/// How many works have borne `base` as their name or as the first part of a
/// numbered one ("Ashby Mill", "Ashby Mill No. 2"), closed works among them, so
/// that the next takes the number after and no name is given twice in a run.
/// Not in the Python, which counts every name that starts with the base and so
/// numbers "Ashby Mill" on from the town's "Ashby Mills" too (`firms::spawn_sister`
/// keeps that count under the port).
pub fn works_of_name(names: &[String], base: &str) -> usize {
    names.iter().filter(|n| n.strip_prefix(base).is_some_and(|rest| rest.is_empty() || rest.starts_with(" No. "))).count()
}

// ---------------------------------------------------------------- families (not in the Python)

/// Surnames of the families who own works and estates.
pub const FAMILY_SURNAMES: &[&str] = &[
    "Ashby", "Blackwood", "Calloway", "Darnell", "Everard", "Fairfax", "Gresley", "Harcourt",
    "Ingram", "Kingsley", "Lascelles", "Mowbray", "Neville", "Ormsby", "Pemberton", "Quennell",
    "Ravensworth", "Strickland", "Tollemache", "Vavasour", "Wentworth", "Yelverton", "Arkwright",
    "Beaumont", "Courtenay", "Dalrymple", "Egerton", "Fitzroy", "Grenville", "Howard",
];

/// First names of family members.
pub const MEMBER_NAMES: &[&str] = &[
    "Arthur", "Anne", "Charles", "Caroline", "Edward", "Eleanor", "Frederick", "Georgiana",
    "Henry", "Harriet", "James", "Isabella", "John", "Louisa", "Richard", "Margaret",
    "Robert", "Mary", "Thomas", "Sophia", "William", "Charlotte", "George", "Emily",
];

/// A family surname not used yet (the list wraps with a numeral if it runs out).
pub fn family_surname(rng: &mut Rng, used: &mut BTreeSet<String>) -> String {
    for _ in 0..200 {
        let name = pick(rng, FAMILY_SURNAMES).to_string();
        if used.insert(name.clone()) {
            return name;
        }
    }
    let name = format!("{} {}", pick(rng, FAMILY_SURNAMES), used.len() + 1);
    used.insert(name.clone());
    name
}

/// A first name not already used in the family, when one is left.
pub fn member_name(rng: &mut Rng, used: &mut BTreeSet<String>) -> String {
    for _ in 0..50 {
        let name = pick(rng, MEMBER_NAMES).to_string();
        if used.insert(name.clone()) {
            return name;
        }
    }
    pick(rng, MEMBER_NAMES).to_string()
}

/// A family's works are named after it: the method's first pattern with only the
/// surname in it, or "{surname} & Co.".
pub fn family_firm_name(method_name: &str, surname: &str, town: &str) -> String {
    let pat = firm_patterns(method_name).iter().find(|p| p.contains("{S}") && !p.contains("{S2}")).copied().unwrap_or("{S} & Co.");
    pat.replace("{S}", surname).replace("{T}", town)
}
