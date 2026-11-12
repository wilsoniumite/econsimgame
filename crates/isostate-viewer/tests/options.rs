//! The command line through the real program, as `saves_guard.rs` does for
//! the saved game: the viewer is started as a player or a helper would start
//! it, and what it says and how it ends are looked at. The unit tests at the
//! foot of `src/main.rs` hold the reading of the options; these hold that a
//! refusal comes before anything is made or played.
//!
//! ```text
//! cargo test -p isostate-viewer --test options          about half a minute
//! ```
//!
//! None of these takes a picture, so none needs a graphics adapter. Every
//! start names its own folder of saved games with `ISOSTATE_SAVES`, under
//! `ISOSTATE_TEST_DIR` if that is set and the system's folder for temporary
//! files if not.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A folder of this test's own, empty.
fn scratch(name: &str) -> PathBuf {
    let root = std::env::var_os("ISOSTATE_TEST_DIR").filter(|v| !v.is_empty()).map_or_else(std::env::temp_dir, PathBuf::from);
    let dir = root.join(format!("isostate-options-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn text(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// How a copy of the game ended.
struct Ran {
    code: Option<i32>,
    /// what it printed
    out: String,
    /// what it told the terminal
    err: String,
    secs: f32,
}

impl Ran {
    /// The lines the game itself said: not the renderer's log.
    fn said(&self) -> Vec<&str> {
        self.err.lines().filter(|l| !l.is_empty() && !l.contains("\u{1b}[") && !l.starts_with('\t')).collect()
    }
}

/// Run the game with these arguments and these variables set, keeping its
/// saved games in `dir`/saves, and wait for it to end (two minutes at most).
fn game(dir: &Path, args: &[&str], vars: &[(&str, &str)]) -> Ran {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let (out, err) = (dir.join(format!("{n}.out.txt")), dir.join(format!("{n}.err.txt")));
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_isostate-viewer"));
    for (k, v) in vars {
        cmd.env(k, v);
    }
    let began = Instant::now();
    let mut child = cmd
        .args(args)
        // never the player's own folder
        .env("ISOSTATE_SAVES", dir.join("saves"))
        .env_remove("ISOSTATE_RESTARTED")
        .env_remove("ISOSTATE_WELCOME")
        .stdin(Stdio::null())
        .stdout(std::fs::File::create(&out).unwrap())
        .stderr(std::fs::File::create(&err).unwrap())
        .spawn()
        .expect("the viewer starts");
    let code = loop {
        match child.try_wait().unwrap() {
            Some(status) => break status.code(),
            None if began.elapsed().as_secs() > 120 => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("the game was still running after two minutes: {args:?}");
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    let read = |p: &Path| String::from_utf8_lossy(&std::fs::read(p).unwrap_or_default()).into_owned();
    Ran { code, out: read(&out), err: read(&err), secs: began.elapsed().as_secs_f32() }
}

/// A refusal: the game did not start, said why in one sentence that begins
/// as given and ends as given, and made no island first.
fn refused(ran: &Ran, begins: &str, ends: &str) -> String {
    let said = ran.said();
    let first = said.first().copied().unwrap_or_default().to_string();
    assert_eq!(ran.code, Some(2), "{first}");
    assert!(first.starts_with(begins), "it said: {first}\nand should begin: {begins}");
    assert!(first.ends_with(ends), "it said: {first}\nand should end: {ends}");
    assert_eq!(first.matches(". ").count(), 0, "one sentence: {first}");
    for l in &said {
        assert!(!l.contains("making island") && !l.contains("playing ") && !l.contains("prepared the game"), "something was made or played before the refusal:\n{}", ran.err);
    }
    assert!(ran.secs < 20.0, "refused after {} s", ran.secs);
    first
}

#[test]
fn a_name_that_is_another_option_is_refused_before_anything_is_played() {
    let dir = scratch("names");
    // --shot --new took --new for the picture's name, and played a new game to take it
    let ran = game(&dir, &["--shot", "--new"], &[]);
    assert_eq!(
        refused(&ran, "--shot needs the name of a picture after it", "(a name that begins with dashes is written ./--new)"),
        "--shot needs the name of a picture after it, and --new is another option: put the name between them (a name that begins with dashes is written ./--new)"
    );
    // --look, --view and --shots-note in the same way as --load and --save
    for (args, begins, ends) in [
        (&["--look", "--advance", "3"][..], "--look needs the name of a look file after it, and --advance is another option", "./--advance)"),
        (&["--view", "--shot", "x.png"], "--view needs a picture's name and its view after it, and --shot is another option", "./--shot)"),
        (&["--shots", &text(&dir.join("sheet")), "--shots-note", "--shots-compare", "before"], "--shots-note needs a line of text after it, and --shots-compare is another option", "./--shots-compare)"),
        (&["--shot"], "--shot needs a value after it", "--shot needs a value after it"),
    ] {
        let ran = game(&dir, args, &[]);
        refused(&ran, begins, ends);
    }
    assert!(!dir.join("saves").exists() && !dir.join("sheet").exists(), "a refused start made a folder");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_measuring_tool_does_its_work_and_the_program_stops_where_the_options_are_read() {
    let dir = scratch("plan");
    let plan = dir.join("plan.ppm");
    // the plan of a small piece of the island (plan.rs), asked for beside a square to describe:
    // the plan is written, and the program stops before it would describe the square
    let ran = game(&dir, &["--families", "1", "--describe", "30,30"], &[("ISOSTATE_PLAN", &text(&plan)), ("ISOSTATE_PLAN_VIEW", "20,20,24,24,2")]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(ran.err.contains("wrote the plan to "), "{}", ran.err);
    assert!(!ran.err.contains("prepared the game") && ran.out.is_empty(), "the program went on past the plan:\n{}\n{}", ran.out, ran.err);
    // eight by eight pixels of three bytes, after the PPM's header
    assert_eq!(std::fs::read(&plan).unwrap().len(), "P6\n8 8\n255\n".len() + 8 * 8 * 3);
    let _ = std::fs::remove_dir_all(&dir);
}
