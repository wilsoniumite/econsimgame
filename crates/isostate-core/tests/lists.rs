//! Lists kept by hand that must name every rule (docs/designs/architecture.md,
//! finding 2 and change A2). For now this holds the two rules the review found
//! missing; one list of the rules, and a test that every hand-kept list agrees
//! with it, are the second half of A2 and wait for the production tree.

use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The commons and the roads each have a line in docs/rules/README.md's list of
/// mechanisms, with a page that exists and the switch a `--set` would name.
#[test]
fn the_rules_page_lists_the_commons_and_the_roads() {
    let page = read("docs/rules/README.md");
    let list = page.split("## Mechanisms under the game's rules").nth(1).expect("the list of mechanisms");
    let list = list.split("\n## ").next().unwrap();
    for (name, page, switch) in [("commons", "commons.md", "commons.carry_debt"), ("roads", "../probes/dx-roads.md", "roads.enabled")] {
        let entry = list.split("\n- ").find(|e| e.starts_with(&format!("**{name}**"))).unwrap_or_else(|| panic!("no line for {name}"));
        assert!(entry.contains(&format!("]({page})")) && entry.contains(&format!("`{switch}`")), "{name}: {entry}");
        assert!(repo().join("docs/rules").join(page).exists(), "{name}'s page {page} is missing");
        // the switch is a setting the sweep's --set takes
        let set = isostate_core::sweep::set_field(&isostate_core::game_params(), switch, &serde_json::json!(true));
        assert!(set.is_ok(), "{switch}: {:?}", set.err());
    }
}

/// The commons' setting sends its reader to the commons' own page.
#[test]
fn the_commons_setting_points_to_its_page() {
    let src = read("crates/isostate-core/src/params.rs");
    let at = src.find("pub commons:").expect("the commons' field");
    let comment = &src[src[..at].rfind("    pub ").unwrap()..at];
    assert!(comment.contains("docs/rules/commons.md"), "{comment}");
}
