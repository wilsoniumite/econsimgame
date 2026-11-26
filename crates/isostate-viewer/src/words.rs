//! The glossary, `data/words.toml`: the game's terms, each with the label it
//! is shown by and one to three sentences on what it is (docs/designs/words.md).
//! It is built into the viewer, as the look is, and read once.
//!
//! A tooltip's sentences are cut into pieces here: plain words, and links.
//! Links are found, not marked: the first mention of any other term's `name`
//! or `also`, as whole words in any case, the longest first, becomes a link
//! to that term. `see` and `wiki` make the line of links at a tooltip's foot.
//!
//! A test holds every entry to the rules of words.md (`the_glossary_keeps_the_rules`),
//! and the labels of the firm's card to the glossary (firm_card.rs).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

/// The glossary as the file in the source tree has it.
pub const WORDS_TOML: &str = include_str!("../../../data/words.toml");

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Term {
    pub name: String,
    pub says: String,
    #[serde(default)]
    pub also: Vec<String>,
    #[serde(default)]
    pub see: Vec<String>,
    #[serde(default)]
    pub wiki: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    term: BTreeMap<String, Term>,
}

/// Every term by its key (`bid_rent`), and the phrases that link to each.
pub struct Glossary {
    pub terms: BTreeMap<String, Term>,
    /// every name and also, lower case, as letters, with its term's key; the longest first
    phrases: Vec<(Vec<char>, String)>,
}

/// Something a piece of a tooltip leads to.
#[derive(Clone, Debug, PartialEq)]
pub enum Link {
    /// a term of the glossary, by its key: hovering opens its tooltip
    Term(String),
    /// a town, by its slot: hovering shows it, clicking goes there
    Town(usize),
    /// a method, by its place in the record's methods: shown as a term, its
    /// tooltip made from the method's own recipe in the record (firm_card.rs)
    Method(usize),
    /// a good, by its place among the goods: clicking opens it on the Markets panel
    Good(usize),
    /// an article of the English Wikipedia, by its title as the glossary has it
    Wiki(String),
}

/// A run of text, plain or a link.
#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    Text(String),
    Link(Link, String),
}

impl Piece {
    pub fn text(&self) -> &str {
        match self {
            Piece::Text(s) | Piece::Link(_, s) => s,
        }
    }
}

/// Plain words, as pieces.
pub fn plain(s: impl Into<String>) -> Vec<Piece> {
    vec![Piece::Text(s.into())]
}

impl Glossary {
    pub fn parse(text: &str) -> Result<Glossary, String> {
        let f: File = toml::from_str(text).map_err(|e| format!("data/words.toml: {e}"))?;
        let mut phrases = Vec::new();
        for (key, t) in &f.term {
            for p in std::iter::once(&t.name).chain(t.also.iter()) {
                phrases.push((p.to_lowercase().chars().collect::<Vec<char>>(), key.clone()));
            }
        }
        phrases.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
        Ok(Glossary { terms: f.term, phrases })
    }

    pub fn get(&self, key: &str) -> Option<&Term> {
        self.terms.get(key)
    }

    /// The key of the term a label names: its `name` or one of its `also`, in any case.
    pub fn key_of(&self, label: &str) -> Option<&str> {
        let l = label.to_lowercase();
        self.terms.iter().find(|(_, t)| t.name.to_lowercase() == l || t.also.iter().any(|a| a.to_lowercase() == l)).map(|(k, _)| k.as_str())
    }

    /// A text cut into pieces, its first mention of each other term a link.
    /// `own` is the term the text is about, which never links to itself.
    pub fn pieces(&self, text: &str, own: Option<&str>) -> Vec<Piece> {
        let chars: Vec<char> = text.chars().collect();
        let lower: Vec<char> = chars.iter().map(|c| c.to_lowercase().next().unwrap_or(*c)).collect();
        let mut out = Vec::new();
        let mut linked: Vec<&str> = Vec::new();
        let mut plain_from = 0;
        let mut i = 0;
        while i < chars.len() {
            let starts_word = i == 0 || !chars[i - 1].is_alphanumeric();
            let mut took = None;
            if starts_word && chars[i].is_alphanumeric() {
                // the longest phrase that is here as whole words
                for (p, key) in &self.phrases {
                    let end = i + p.len();
                    if end <= chars.len() && lower[i..end] == p[..] && (end == chars.len() || !chars[end].is_alphanumeric()) {
                        took = Some((end, key.as_str()));
                        break;
                    }
                }
            }
            match took {
                Some((end, key)) => {
                    // a mention of itself, or of a term already linked, stays plain
                    if Some(key) != own && !linked.contains(&key) {
                        if plain_from < i {
                            out.push(Piece::Text(chars[plain_from..i].iter().collect()));
                        }
                        out.push(Piece::Link(Link::Term(key.to_string()), chars[i..end].iter().collect()));
                        linked.push(key);
                        plain_from = end;
                    }
                    i = end;
                }
                None => i += 1,
            }
        }
        if plain_from < chars.len() {
            out.push(Piece::Text(chars[plain_from..].iter().collect()));
        }
        out
    }
}

/// The glossary built into the viewer.
pub fn glossary() -> &'static Glossary {
    static G: OnceLock<Glossary> = OnceLock::new();
    // the file is part of the program: if it is wrong the program is, and the tests say so
    G.get_or_init(|| Glossary::parse(WORDS_TOML).unwrap_or_else(|e| panic!("{e}")))
}

// ------------------------------------------------------------------ the rules

/// The words kept out of labels and tooltips (docs/designs/words.md, the last
/// list but one), as whole words in any case.
#[cfg_attr(not(test), allow(dead_code))]
pub const KEPT_OUT: [&str; 17] =
    ["stands", "stood", "standing", "as it stands", "kept", "told", "asked", "whatever", "whoever", "the island's", "by hand", "the year's turn", "a head", "per cent", "dear", "of it", "there is no"];

/// The words of the table's "Not" column (docs/designs/words.md, Words we
/// use) that mean only the thing the glossary has another term for, so that a
/// test can hold them out too: "works" for a firm, "hands" for workers.
/// Those with other senses ("made", "found", "a month") are left to the reader.
#[cfg_attr(not(test), allow(dead_code))]
pub const NOT_USED: [&str; 12] = ["works", "concern", "hands", "plant", "square", "cell", "district", "idea", "technique", "carting", "dearest", "cheapest"];

/// The sentences of a text: cut after a full stop, a question mark or an
/// exclamation mark followed by a space or the end.
#[cfg_attr(not(test), allow(dead_code))]
pub fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        cur.push(c);
        if matches!(c, '.' | '?' | '!') && chars.get(i + 1).is_none_or(|n| n.is_whitespace()) {
            out.push(cur.trim().to_string());
            cur.clear();
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// The kept-out words a text holds, as whole words in any case.
#[cfg_attr(not(test), allow(dead_code))]
pub fn kept_out_in(text: &str) -> Vec<&'static str> {
    let words: Vec<String> = text.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '\u{2019}')).filter(|w| !w.is_empty()).map(|w| w.to_lowercase().replace('\u{2019}', "'")).collect();
    KEPT_OUT
        .iter()
        .chain(NOT_USED.iter())
        .filter(|k| {
            let kw: Vec<&str> = k.split(' ').collect();
            words.windows(kw.len()).any(|w| w.iter().zip(&kw).all(|(a, b)| a == b))
        })
        .copied()
        .collect()
}

/// Whether a label keeps the rules: one to three words, none kept out, no full stop.
#[cfg_attr(not(test), allow(dead_code))]
pub fn label_fault(label: &str) -> Option<String> {
    let n = label.split_whitespace().count();
    if !(1..=3).contains(&n) {
        return Some(format!("\"{label}\" is {n} words, where a label is one to three"));
    }
    if let Some(w) = kept_out_in(label).first() {
        return Some(format!("\"{label}\" holds \"{w}\", which the rules keep out"));
    }
    if label.ends_with('.') {
        return Some(format!("\"{label}\" ends in a full stop"));
    }
    None
}

/// Every way the glossary breaks the rules of words.md, one line each.
#[cfg_attr(not(test), allow(dead_code))]
pub fn faults(g: &Glossary) -> Vec<String> {
    let mut out = Vec::new();
    for (key, t) in &g.terms {
        if let Some(f) = label_fault(&t.name) {
            out.push(format!("{key}: its name {f}"));
        }
        if t.says.trim().is_empty() {
            out.push(format!("{key}: says nothing"));
        }
        let s = sentences(&t.says);
        if s.len() > 3 {
            out.push(format!("{key}: {} sentences, where a tooltip has at most three", s.len()));
        }
        for sen in &s {
            let n = sen.split_whitespace().count();
            if n > 18 {
                out.push(format!("{key}: a sentence of {n} words, where 18 is the most: \"{sen}\""));
            }
            // a colon or a semicolon joins two sentences; brackets are not used
            if sen.contains([':', ';', '(', ')']) {
                out.push(format!("{key}: a colon, a semicolon or a bracket in \"{sen}\""));
            }
            if !sen.ends_with('.') {
                out.push(format!("{key}: \"{sen}\" does not end in a full stop"));
            }
        }
        for w in kept_out_in(&t.says) {
            out.push(format!("{key}: says \"{w}\", which the rules keep out"));
        }
        for s in &t.see {
            if !g.terms.contains_key(s) {
                out.push(format!("{key}: sees \"{s}\", which is no term"));
            }
        }
        for w in &t.wiki {
            if w.trim().is_empty() || w.contains(['/', '?', '#']) {
                out.push(format!("{key}: \"{w}\" is not a title"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_glossary_keeps_the_rules() {
        let g = glossary();
        assert!(g.terms.len() >= 30, "{} terms", g.terms.len());
        let f = faults(g);
        assert!(f.is_empty(), "the glossary breaks the rules of docs/designs/words.md:\n{}", f.join("\n"));
    }

    #[test]
    fn a_nineteen_word_sentence_a_dead_see_and_a_kept_out_word_are_each_caught() {
        // the same glossary with three faults put in, one to a term; each is found
        let words = WORDS_TOML.replace("\r\n", "\n");
        let bad = words
            .replace("says = \"The most a firm can make in a month with its equipment.\"", "says = \"The most a firm can make in a month with its equipment if every machine runs all night long.\"")
            .replace("see = [\"capacity\", \"method\"]", "see = [\"capacity\", \"plant\"]")
            .replace("says = \"Money a firm holds to pay its costs.\"", "says = \"Money a firm has kept to pay its costs.\"");
        assert_ne!(bad, words);
        let f = faults(&Glossary::parse(&bad).unwrap());
        assert_eq!(f.len(), 3, "{f:#?}");
        assert!(f[0].starts_with("capacity: a sentence of 19 words"), "{}", f[0]);
        assert!(f[1].starts_with("cash: says \"kept\""), "{}", f[1]);
        assert!(f[2].starts_with("size: sees \"plant\""), "{}", f[2]);
        // a term with nothing to say, and a name of four words
        let bad = words.replace("name = \"Cash\"\nsays = \"Money a firm holds to pay its costs.\"", "name = \"Cash in the bank\"\nsays = \"\"");
        assert_ne!(bad, words);
        let f = faults(&Glossary::parse(&bad).unwrap());
        assert!(f.iter().any(|l| l.contains("is 4 words")) && f.iter().any(|l| l.ends_with("says nothing")), "{f:#?}");
    }

    #[test]
    fn links_are_found_whole_words_first_mention_longest_first_and_never_to_itself() {
        let g = glossary();
        let p = g.pieces("Paid monthly for the site the firm occupies. A site goes to the highest bid rent, not the bids. Bid rent again.", Some("ground_rent"));
        let links: Vec<(String, String)> = p
            .iter()
            .filter_map(|x| match x {
                Piece::Link(Link::Term(k), s) => Some((k.clone(), s.clone())),
                _ => None,
            })
            .collect();
        // "site" once (the first), "firm", "bid rent" as one link and not "bids" again
        assert_eq!(links, vec![("site".into(), "site".into()), ("firm".into(), "firm".into()), ("bid_rent".into(), "bid rent".into())]);
        // the pieces make the text again, word for word
        assert_eq!(p.iter().map(|x| x.text()).collect::<String>(), "Paid monthly for the site the firm occupies. A site goes to the highest bid rent, not the bids. Bid rent again.");
        // its own name stays plain; a word inside another word is not a mention
        let p = g.pieces("Bid rent falls with distance to market. Outbidding.", Some("bid_rent"));
        assert!(matches!(&p[1], Piece::Link(Link::Term(k), s) if k == "distance_to_market" && s == "distance to market"), "{p:?}");
        assert_eq!(p.len(), 3);
        assert_eq!(g.key_of("Workers"), Some("worker"));
        assert_eq!(g.key_of("ground rent"), Some("ground_rent"));
    }
}
