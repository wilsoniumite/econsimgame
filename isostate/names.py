"""Procedural names for towns, people and firms."""

PREFIXES = [
    "Ash", "Bar", "Bel", "Black", "Bram", "Brack", "Bright", "Bur", "Cal", "Car",
    "Cold", "Crag", "Dal", "Dun", "East", "Elm", "Fair", "Fal", "Fen", "Gar",
    "Glen", "Gold", "Gran", "Grey", "Hal", "Hart", "Haw", "High", "Hol", "Kel",
    "Ken", "Kirk", "Lang", "Lark", "Lea", "Lin", "Long", "Mar", "Mel", "Mor",
    "Nor", "Oak", "Old", "Pen", "Red", "Ros", "Rye", "Sal", "Sand", "Shaw",
    "Stan", "Stone", "Strat", "Sun", "Thorn", "Tor", "Wal", "Wel", "West", "Whit",
    "Wil", "Win", "Wolf", "Wood", "Wren", "Yar", "Brin", "Cran", "Ether", "Hask",
    "Ivel", "Mallow", "Orm", "Quen", "Ruth", "Skel", "Tarn", "Ul", "Vel", "Wyr",
]

SUFFIX = {
    "plain": ["ford", "by", "ton", "ham", "wick", "stead", "ley", "bury", "worth",
              "field", "thorpe", "well", "stow", "combe", "hurst", "borough", "ing"],
    "coast": ["mouth", "haven", "port", "sea", "wick", "ness", "strand"],
    "river": ["ford", "bridge", "mere", "water", "brook", "bourne", "ford"],
    "hills": ["dale", "moor", "fell", "edge", "rigg", "side", "hope"],
    "mine": ["pit", "delf", "heath", "side", "gate", "bank"],
}

SURNAMES = [
    "Abbot", "Ainsley", "Barlow", "Beckett", "Blythe", "Booth", "Brindle", "Carver",
    "Cawley", "Crowther", "Dent", "Denton", "Elwood", "Fairbairn", "Fenwick", "Garside",
    "Hargreave", "Holroyd", "Ingham", "Kershaw", "Lister", "Marsden", "Naylor", "Oldroyd",
    "Pickles", "Platt", "Radcliffe", "Rowntree", "Sowerby", "Tennant", "Thwaite", "Ward",
    "Whitworth", "Wilkin", "Wraith", "Ackroyd", "Bramall", "Calvert", "Dawson", "Emmott",
    "Firth", "Greaves", "Haigh", "Ibbotson", "Jowett", "Kitson", "Lumb", "Mallinson",
    "Nettleton", "Oates", "Priestley", "Quarmby", "Rushworth", "Sugden", "Tetley",
    "Uttley", "Varley", "Wadsworth", "Yeadon", "Armitage", "Birkett", "Clegg", "Dobson",
    "Eastwood", "Foulds", "Gaukroger", "Horsfall", "Illingworth", "Jagger", "Kenworthy",
]

FIRM_PATTERNS = {
    "charcoal_burning": ["{S}'s Charcoal", "{T} Colliers' Wood"],
    "bloomery": ["{S}'s Bloomery", "{T} Bloomery"],
    "charcoal_furnace": ["{T} Furnace", "{S} Furnace Co."],
    "coke_furnace": ["{T} Ironworks", "{S} & Sons Ironworks", "{T} Iron Co."],
    "smithy": ["{S}'s Forge", "{S} Edge Tools", "{T} Smithy"],
    "hand_loom": ["{S} Clothiers", "{T} Weavers", "{S} & Co. Cloth"],
    "textile_mill": ["{S} Mill", "{T} Mills", "{S} Brothers Mill"],
    "brickworks": ["{T} Brickworks", "{S}'s Brickfield"],
    "millwright": ["{S}, Millwright", "{S} & Son Millwrights"],
    "engine_works": ["{S} Engine Works", "{T} Foundry", "{S} & {S2} Engines"],
    "crucible_steel": ["{S} Steel", "{S}'s Crucible Works"],
    "bessemer": ["{T} Steel Co.", "{S} Steelworks"],
    "forge_rails": ["{S}'s Rail Forge"],
    "rolling_mill": ["{T} Rolling Mills", "{S} Rail Works"],
    "steel_rolling": ["{T} Rail Mills", "{S} Steel Rail Co."],
}


def town_name(rng, context, used):
    """A new unused town name; context picks a fitting suffix list."""
    suffixes = SUFFIX.get(context, SUFFIX["plain"])
    for _ in range(200):
        name = rng.choice(PREFIXES) + rng.choice(suffixes)
        if name not in used:
            used.add(name)
            return name
    name = f"New {rng.choice(PREFIXES)}{rng.choice(suffixes)}"
    used.add(name)
    return name


def surname(rng):
    return str(rng.choice(SURNAMES))


def firm_name(rng, method_name, town):
    patterns = FIRM_PATTERNS.get(method_name, ["{S} & Co."])
    pat = str(rng.choice(patterns))
    s1 = surname(rng)
    s2 = surname(rng)
    while s2 == s1:
        s2 = surname(rng)
    return pat.format(S=s1, S2=s2, T=town)
