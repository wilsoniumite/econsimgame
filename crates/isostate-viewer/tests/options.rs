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
}

impl Ran {
    /// The lines the game itself said: not the renderer's log.
    fn said(&self) -> Vec<&str> {
        self.err.lines().filter(|l| !l.is_empty() && !l.contains("\u{1b}[") && !l.starts_with('\t')).collect()
    }
}

/// How long a run may go on before it is taken to have hung, in seconds: ten
/// minutes, which is many times what any run here needs on a quiet machine,
/// times `ISOSTATE_TEST_PATIENCE` if that is set (2 doubles it). No test here is
/// timed: the limit only stops a game that would never end.
fn patience() -> f32 {
    600.0 * std::env::var("ISOSTATE_TEST_PATIENCE").ok().and_then(|v| v.trim().parse::<f32>().ok()).filter(|t| t.is_finite() && *t > 0.0).unwrap_or(1.0)
}

/// Run the game with these arguments and these variables set, keeping its
/// saved games in `dir`/saves, and wait for it to end.
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
            None if began.elapsed().as_secs_f32() > patience() => {
                let _ = child.kill();
                let _ = child.wait();
                panic!(
                    "timed out after {:.0} s waiting for the game to end: {args:?}. No wrong answer was seen; a machine busy enough can run a wait out, and ISOSTATE_TEST_PATIENCE=3 waits three times as long. The game had said:\n{}",
                    patience(),
                    String::from_utf8_lossy(&std::fs::read(&err).unwrap_or_default())
                );
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    let read = |p: &Path| String::from_utf8_lossy(&std::fs::read(p).unwrap_or_default()).into_owned();
    Ran { code, out: read(&out), err: read(&err) }
}

/// A refusal: the game did not start, said why in one sentence that begins
/// as given and ends as given, and made no island first. That the refusal is
/// the first thing it said, and that nothing of an island made or a year
/// played is said at all, is what shows it came before anything was made,
/// since the game says that it is making an island, or playing years again,
/// as it begins to. (It used to be timed as well, which a busy machine could
/// fail.)
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
        (&["--shots", &text(&dir.join("sheet")), "--shots-note", "--shots-compare", "before"], "--shots-note needs a line of text after it, and --shots-compare is another option", "put the line between them"),
        (&["--shot"], "--shot needs a value after it", "--shot needs a value after it"),
    ] {
        let ran = game(&dir, args, &[]);
        refused(&ran, begins, ends);
    }
    assert!(!dir.join("saves").exists() && !dir.join("sheet").exists(), "a refused start made a folder");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_good_a_game_lacks_is_refused_before_its_island_is_made() {
    let dir = scratch("goods");
    let picture = dir.join("coal.png");
    // the word came from a --view, and the refusal says so (it used to say --overlay)
    let view = format!("{},1850,,,,,,price:cole", text(&picture));
    let ran = game(&dir, &["--view", &view], &[]);
    let said = refused(&ran, "--view: \"cole\" is not a good of this run: use one of ", "");
    assert!(said.contains("coal"), "{said}");
    assert!(!picture.exists());
    // and the good a view's Markets panel opens on, which was not checked at all
    let view = format!("{},1850,,,,,,markets,open:Cole:months", text(&picture));
    let ran = game(&dir, &["--view", &view], &[]);
    refused(&ran, "--view: \"Cole\" is not a good of this run: use one of ", "");
    assert!(!picture.exists());
    // a word in a view that is no panel, no map view and nothing the Markets panel shows
    let view = format!("{},1850,,,,,,rents", text(&picture));
    let ran = game(&dir, &["--view", &view], &[]);
    refused(&ran, "--view: rents is not a word a view takes", "for the Markets panel");
    assert!(!picture.exists());
    let ran = game(&dir, &["--overlay", "price:Cole", "--describe", "30,30"], &[]);
    refused(&ran, "--overlay: \"Cole\" is not a good of this run: use one of ", "");
    // a good the game has, by its name in any case, goes through to the island
    let ran = game(&dir, &["--families", "1", "--overlay", "price:COAL", "--describe", "30,30"], &[]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(ran.err.contains("making island 7 with 1 family\u{2026}"), "{}", ran.err);
    assert!(!ran.out.is_empty(), "nothing was described");
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
