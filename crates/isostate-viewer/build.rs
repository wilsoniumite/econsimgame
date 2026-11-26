//! Tells the viewer which commit it was built from, so that a note typed at
//! the window (src/note.rs) can say which build it was seen in. It asks git
//! and nothing else; with no git to ask the commit is "unknown". It runs again
//! when a commit is made or checked out, and when the viewer's sources change,
//! so the words "with changes not committed" say how things stood at the last
//! build of the viewer.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    let commit = git(&["rev-parse", "--short=12", "HEAD"]).filter(|c| !c.is_empty());
    let changed = commit.is_some() && git(&["status", "--porcelain", "--untracked-files=no"]).is_some_and(|s| !s.is_empty());
    let said = match commit {
        Some(c) if changed => format!("{c}, with changes not committed"),
        Some(c) => c,
        None => "unknown".to_string(),
    };
    println!("cargo:rustc-env=ISOSTATE_COMMIT={said}");
    for path in ["HEAD", "logs/HEAD"] {
        if let Some(p) = git(&["rev-parse", "--git-path", path]).filter(|p| !p.is_empty()) {
            println!("cargo:rerun-if-changed={p}");
        }
    }
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");
}
