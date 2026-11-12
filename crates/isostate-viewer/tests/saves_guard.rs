//! The saved game through the real program: the viewer is started as a
//! player or a helper would start it, and what it says, how it ends and what
//! it leaves in the folder are looked at. The unit tests (`src/saves.rs`,
//! `src/live_tests.rs`) hold the rules; these hold that the program keeps them.
//!
//! ```text
//! cargo test -p isostate-viewer --test saves_guard                                  the refusals: under a minute
//! cargo test -p isostate-viewer --test saves_guard -- --ignored --test-threads 1    those that play: about six minutes
//! ```
//!
//! The second set plays the game with `--script`, which draws it with no
//! window and so needs a graphics adapter; they are left out of a plain
//! `cargo test` for that and for their length. Every start here names its own
//! folder with `ISOSTATE_SAVES`, under `ISOSTATE_TEST_DIR` if that is set and
//! the system's folder for temporary files if not: none can reach the
//! player's own saved games.
//!
//! No test here is timed. A copy of the game is waited for until it ends, or
//! until it has said or written what is looked for; the limits on those waits
//! only stop a game that hangs, are many times what a quiet machine needs, and
//! when one runs out the test says it timed out, which is not a wrong answer.
//! On a machine too busy even for them, `ISOSTATE_TEST_PATIENCE=3` makes every
//! wait three times as long.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use isostate_core::save::Save;

/// How long a wait may go on, in seconds: `secs` times `ISOSTATE_TEST_PATIENCE`
/// if that is set. (On 12 November a whole century that takes 45 s alone was
/// still being played after the two minutes it was then given, with two dozen
/// builds running beside it.)
fn patience(secs: f32) -> f32 {
    secs * std::env::var("ISOSTATE_TEST_PATIENCE").ok().and_then(|v| v.trim().parse::<f32>().ok()).filter(|t| t.is_finite() && *t > 0.0).unwrap_or(1.0)
}

/// Stop the test because a wait ran out, saying so.
fn timed_out(secs: f32, what: &str, said: &str) -> ! {
    panic!(
        "timed out after {secs:.0} s waiting: {what}. No wrong answer was seen; a machine busy enough can run a wait out, and ISOSTATE_TEST_PATIENCE=3 waits three times as long. The game had said:\n{said}"
    )
}

/// A folder of this test's own, empty.
fn scratch(name: &str) -> PathBuf {
    let root = std::env::var_os("ISOSTATE_TEST_DIR").filter(|v| !v.is_empty()).map_or_else(std::env::temp_dir, PathBuf::from);
    let dir = root.join(format!("isostate-guard-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn files(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    v.sort();
    v
}

fn text(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// A copy of the game that has been started, with where it writes what it says.
struct Started {
    child: Child,
    out: PathBuf,
    err: PathBuf,
    began: Instant,
}

/// How a copy of the game ended.
struct Ran {
    code: Option<i32>,
    /// what it printed (a script's lines)
    out: String,
    /// what it told the terminal
    err: String,
}

impl Ran {
    /// The lines the game itself said: not the renderer's log.
    fn said(&self) -> Vec<&str> {
        self.err.lines().filter(|l| !l.is_empty() && !l.contains("\u{1b}[") && !l.starts_with('\t')).collect()
    }

    /// A script's line that begins with `start`, without it.
    fn line(&self, start: &str) -> &str {
        self.out.lines().find_map(|l| l.strip_prefix(start)).unwrap_or_else(|| panic!("the script printed no line beginning {start:?}:\n{}\n{}", self.out, self.err))
    }

    /// The lines a script's `print:NOTE` printed, joined.
    fn printed(&self, note: &str) -> String {
        let from = self.out.lines().position(|l| l.starts_with(&format!("script: [{note}]"))).unwrap_or_else(|| panic!("the script printed nothing for {note}:\n{}\n{}", self.out, self.err));
        self.out.lines().skip(from).take_while(|l| l.starts_with("script:   ") || l.starts_with(&format!("script: [{note}]"))).collect::<Vec<_>>().join("\n")
    }
}

/// Start the game with these arguments, keeping its saved games in `saves`.
fn start(saves: &Path, args: &[&str]) -> Started {
    start_told(saves, args, None)
}

/// `start`, with what ISOSTATE_WELCOME says (which of the opening card's
/// buttons is pressed with nobody at the pointer).
fn start_told(saves: &Path, args: &[&str], welcome: Option<&str>) -> Started {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let logs = saves.parent().unwrap().join("said");
    std::fs::create_dir_all(&logs).unwrap();
    let (out, err) = (logs.join(format!("{n}.out.txt")), logs.join(format!("{n}.err.txt")));
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_isostate-viewer"));
    match welcome {
        Some(button) => cmd.env("ISOSTATE_WELCOME", button),
        None => cmd.env_remove("ISOSTATE_WELCOME"),
    };
    let child = cmd
        .args(args)
        // never the player's own folder
        .env("ISOSTATE_SAVES", saves)
        .env_remove("ISOSTATE_RESTARTED")
        .env_remove("ISOSTATE_HELP")
        .stdin(Stdio::null())
        .stdout(std::fs::File::create(&out).unwrap())
        .stderr(std::fs::File::create(&err).unwrap())
        .spawn()
        .expect("the viewer starts");
    Started { child, out, err, began: Instant::now() }
}

impl Started {
    fn said_so_far(&self) -> String {
        String::from_utf8_lossy(&std::fs::read(&self.err).unwrap_or_default()).into_owned()
    }

    /// Wait for it to end; one still running after `patience(limit)` seconds
    /// is stopped, and the test stops saying it timed out.
    fn end(mut self, limit: f32) -> Ran {
        let limit = patience(limit);
        let code = loop {
            match self.child.try_wait().unwrap() {
                Some(status) => break status.code(),
                None if self.began.elapsed().as_secs_f32() > limit => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    timed_out(limit, "for the game to end", &self.said_so_far());
                }
                None => std::thread::sleep(Duration::from_millis(50)),
            }
        };
        let read = |p: &Path| String::from_utf8_lossy(&std::fs::read(p).unwrap_or_default()).into_owned();
        Ran { code, out: read(&self.out), err: read(&self.err) }
    }

    /// Stop it dead, as a crash or the task manager would.
    fn crash(mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
    }
}

/// Run the game to its end. A run with no script ends by itself in seconds
/// (`--describe` prints and stops before a window is thought of), or in a
/// minute if it plays a century; a scripted one in a few minutes at most.
/// Each is given ten times that before it is taken to have hung.
fn game(saves: &Path, args: &[&str]) -> Ran {
    start(saves, args).end(if args.contains(&"--script") { 1800.0 } else { 600.0 })
}

/// A refusal: the game did not start, said why in one sentence that begins
/// as given and ends by saying what to do, and made no island first. That the
/// refusal is the first thing it said, and that nothing of an island made or a
/// year played is said at all, is what shows it came before anything was
/// made, since the game says that it is making an island, or playing years
/// again, as it begins to. (Three of these used to be timed as well, which a
/// busy machine could fail.)
fn refused(ran: &Ran, code: i32, begins: &str, ends: &str) -> String {
    let said = ran.said();
    let first = said.first().copied().unwrap_or_default().to_string();
    assert_eq!(ran.code, Some(code), "{first}");
    assert!(first.starts_with(begins), "it said: {first}\nand should begin: {begins}");
    assert!(first.ends_with(ends), "it said: {first}\nand should end: {ends}");
    assert_eq!(first.matches(". ").count(), 0, "one sentence: {first}");
    for l in &said {
        assert!(!l.contains("making island") && !l.contains("playing ") && !l.contains("prepared the game"), "something was played before the refusal:\n{}", ran.err);
    }
    first
}

/// The island's own saved game, as a played game with orders in it, made by
/// the program: the demo's works and land, saved where the autosave is kept.
fn a_played_game(saves: &Path) -> (PathBuf, Vec<u8>) {
    let file = saves.join("island-7-families-6.json");
    let made = game(saves, &["--new", "--demo", "--advance", "2", "--save", &text(&file), "--describe", "30,30"]);
    assert_eq!(made.code, Some(0), "{}", made.err);
    let played = Save::from_text(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert!(played.orders.len() >= 2 && played.year == 1752, "{played:?}");
    let bytes = std::fs::read(&file).unwrap();
    (file, bytes)
}

#[test]
fn an_option_that_is_wrong_is_refused_in_one_sentence() {
    let dir = scratch("options");
    let saves = dir.join("saves");
    // --load with no file after it used to take the next option for the file
    let ran = game(&saves, &["--load", "--size", "800x600"]);
    assert_eq!(
        refused(&ran, 2, "--load needs the name of a saved game after it", "(a name that begins with dashes is written ./--size)"),
        "--load needs the name of a saved game after it, and --size is another option: put the name between them (a name that begins with dashes is written ./--size)"
    );
    let ran = game(&saves, &["--new", "--save", "--advance", "3"]);
    refused(&ran, 2, "--save needs the name of a file to keep the game in after it, and --advance is another option", "./--advance)");
    // nothing bounded the number of families
    let ran = game(&saves, &["--families", "500"]);
    assert_eq!(refused(&ran, 2, "--families: ", "give a number in that range"), "--families: a game has from 1 to 30 families, not 500; give a number in that range");
    let ran = game(&saves, &["--families", "0"]);
    refused(&ran, 2, "--families: a game has from 1 to 30 families, not 0", "give a number in that range");
    // every option that wants something after it, given nothing: one sentence, and no island made
    for opt in ["--load", "--save", "--families", "--advance", "--seed", "--script", "--shot", "--view", "--describe", "--year", "--size", "--shots"] {
        let ran = game(&saves, &[opt]);
        refused(&ran, 2, &format!("{opt} needs "), " after it");
    }
    // and given what it cannot take
    for (args, begins) in [
        (&["--families", "six"][..], "--families: six is not a number it can take"),
        (&["--families", "-1"], "--families: -1 is not a number it can take"),
        (&["--advance", "-3"], "--advance: -3 is not a number it can take"),
        (&["--advance", "99999999999999999999"], "--advance: 99999999999999999999 is not a number it can take"),
        (&["--seed", "1e3"], "--seed: 1e3 is not a number it can take"),
        (&["--script", "--new"], "--script: --new: not a step"),
        (&["--loda", "x.json"], "--loda is not an option"),
    ] {
        let ran = game(&saves, args);
        refused(&ran, 2, begins, "");
    }
    assert!(!saves.exists(), "a refused start made the folder for saved games");
    // as many years as can be counted is the whole game and not one more than can be: this once
    // stopped the program where it stood (a family alone on the island, to be quick about it)
    let ran = game(&saves, &["--advance", "18446744073709551615", "--families", "1", "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(ran.err.contains("nobody is at this game") && ran.err.contains("prepared the game"), "{}", ran.err);
    assert!(!saves.exists(), "a run that only describes a square made the folder for saved games");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_that_is_wrong_is_refused_before_anything_is_played() {
    let dir = scratch("files");
    let saves = dir.join("saves");
    let good = dir.join("good.json");
    let made = game(&saves, &["--new", "--save", &text(&good), "--describe", "30,30"]);
    assert_eq!(made.code, Some(0), "{}", made.err);
    let whole = std::fs::read_to_string(&good).unwrap();
    assert!(whole.contains("\"year\": 1750,") && whole.contains("\"families\": 6,") && whole.contains("\"format\": 2,"), "{whole}");
    let what_to_do = "; name another saved game after --load, or leave --load out to play the island's own";
    let try_file = |name: &str, contents: Option<String>| {
        let file = dir.join(name);
        if let Some(c) = &contents {
            std::fs::write(&file, c).unwrap();
        }
        let ran = game(&saves, &["--load", &text(&file), "--advance", "150", "--describe", "30,30"]);
        let said = refused(&ran, 1, &format!("the game could not start: {}: ", file.display()), what_to_do);
        if let Some(c) = &contents {
            assert_eq!(&std::fs::read_to_string(&file).unwrap(), c, "the file that was refused was changed");
        }
        said
    };
    // a save doctored to the year 5000 was once accepted and announced as 3,251 years to play
    let said = try_file("year-5000.json", Some(whole.replace("\"year\": 1750,", "\"year\": 5000,")));
    assert!(said.contains("not a saved game the game wrote: it says the game had reached the year 5000, and the island's years are 1750 to 1900"), "{said}");
    let said = try_file("year-1066.json", Some(whole.replace("\"year\": 1750,", "\"year\": 1066,")));
    assert!(said.contains("the year 1066"), "{said}");
    // a number of families no game has
    let said = try_file("families.json", Some(whole.replace("\"families\": 6,", "\"families\": 5000,")));
    assert!(said.contains("it names 5000 families, and a game has from 1 to 30 families, not 5000"), "{said}");
    let said = try_file("month.json", Some(whole.replace("\"month\": 0,", "\"month\": 40,")));
    assert!(said.contains("it says 40 months of a year had been played"), "{said}");
    // a save cut short, something that is no save, a later layout, no file at all
    let said = try_file("half.json", Some(whole[..whole.len() / 2].to_string()));
    assert!(said.contains("not a saved game: "), "{said}");
    let said = try_file("notes.json", Some("{\"shopping\": [\"eggs\"]}".to_string()));
    assert!(said.contains("not a saved game: it does not say which layout it is in"), "{said}");
    let said = try_file("later.json", Some(whole.replace("\"format\": 2,", "\"format\": 9,")));
    assert!(said.contains("saved by a later version of the game (layout 9; this one reads up to 2)"), "{said}");
    try_file("not-there.json", None);
    let said = try_file("empty.json", Some(String::new()));
    assert!(said.contains("not a saved game: "), "{said}");
    // a folder, and a file that is no text at all: said in the game's words, not the system's
    std::fs::create_dir_all(dir.join("folder.json")).unwrap();
    let said = try_file("folder.json", None);
    assert!(said.contains("not a saved game: it is a folder, and a saved game is a file in one"), "{said}");
    std::fs::write(dir.join("picture.json"), [0x89, b'P', b'N', b'G', 0xff, 0xfe, 0x00, 0x01]).unwrap();
    let said = try_file("picture.json", None);
    assert!(said.contains("not a saved game: it is not a file of text"), "{said}");
    // orders that name what is not on the island (a square, a town, a works nobody knows, a member of
    // the family who is not there) are refused one at a time, each said, and the rest of the game is played
    let wild = dir.join("wild.json");
    let orders = r#""orders": [
    {"year":1750,"kind":"buy_land","cell":99999999},
    {"year":1750,"kind":"buy_land","cell":18446744073709551615},
    {"year":1750,"kind":"move_seat","town":4000},
    {"year":1750,"kind":"found","method":"a works nobody knows","cell":18446744073709551615},
    {"year":1750,"kind":"found","method":"a works nobody knows","cell":0},
    {"year":1750,"kind":"post","member":77,"posting":{"kind":"manager","group":"iron","town":900}},
    {"year":1750,"kind":"stance","group":"estates","town":12345,"stance":"grow"},
    {"year":1750,"kind":"decide","id":18446744073709551615,"choice":{"kind":"head","member":999}},
    {"year":-40,"kind":"move_seat","town":18446744073709551615},
    {"year":1890,"kind":"move_seat","town":1}
  ]"#;
    assert!(whole.contains("\"orders\": []"), "{whole}");
    std::fs::write(&wild, whole.replace("\"orders\": []", orders)).unwrap();
    let back = game(&saves, &["--load", &text(&wild), "--describe", "30,30"]);
    assert_eq!(back.code, Some(0), "{}", back.err);
    assert_eq!(back.err.matches("could not be given again").count(), 10, "{}", back.err);
    assert!(back.err.contains("back at the end of 1750") && back.err.contains("after the save's last year"), "{}", back.err);
    // gone on with and kept in its own place, the game has none of those orders in it: the file that has
    // is set aside whole before the game is saved over it, and the terminal says so
    let as_it_was = std::fs::read_to_string(&wild).unwrap();
    let kept = game(&saves, &["--load", &text(&wild), "--save", &text(&wild), "--describe", "30,30"]);
    assert_eq!(kept.code, Some(0), "{}", kept.err);
    assert!(kept.err.contains("the save as it was, with every order in it, is not written over: where this game is kept in its place, the file is first set aside whole (.previous.json)"), "{}", kept.err);
    assert_eq!(std::fs::read_to_string(dir.join("wild.previous.json")).unwrap(), as_it_was, "the file with every order in it was written over");
    assert!(Save::from_text(&std::fs::read_to_string(&wild).unwrap()).unwrap().orders.is_empty());
    // and the save as the game wrote it is gone on with
    let back = game(&saves, &["--load", &text(&good), "--describe", "30,30"]);
    assert_eq!(back.code, Some(0), "{}", back.err);
    assert!(back.err.contains("going on with the") && back.err.contains("back at the end of 1750"), "{}", back.err);
    assert!(!saves.exists(), "a run that only describes a square made the folder for saved games");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_place_to_save_in_that_cannot_be_written_is_refused_before_anything_is_played() {
    let dir = scratch("places");
    let saves = dir.join("saves");
    let notes = dir.join("notes.txt");
    std::fs::write(&notes, "somebody's notes").unwrap();
    // a folder that cannot be made: its name is taken by a file
    let inside = notes.join("game.json");
    let ran = game(&saves, &["--new", "--advance", "100", "--save", &text(&inside), "--describe", "30,30"]);
    refused(&ran, 1, &format!("the game could not start: a game cannot be kept in {}, because nothing can be written in {} (", inside.display(), notes.display()), "): name a file in a folder that can be written");
    // somebody's file is not written over, and a folder is not a file
    let ran = game(&saves, &["--new", "--save", &text(&notes), "--describe", "30,30"]);
    refused(&ran, 1, &format!("the game could not start: {}: not a saved game: ", notes.display()), ", and --save writes only where nothing is or over a saved game it can read: name another file");
    let ran = game(&saves, &["--new", "--save", &text(&dir), "--describe", "30,30"]);
    refused(&ran, 1, &format!("the game could not start: {} is a folder", dir.display()), "and --save needs the name of a file: name one inside it");
    assert_eq!(std::fs::read_to_string(&notes).unwrap(), "somebody's notes");
    assert_eq!(files(&dir), ["notes.txt", "said"], "a refused start left something behind");
    // a free place in a folder still to be made is taken, and written at once
    let free = dir.join("new").join("game.json");
    let ran = game(&saves, &["--new", "--save", &text(&free), "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert_eq!(files(&dir.join("new")), ["game.json", "game.json.lock"]);
    assert_eq!(Save::from_text(&std::fs::read_to_string(&free).unwrap()).unwrap().year, 1750);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_second_copy_is_refused_and_a_copy_that_crashed_keeps_nobody_out() {
    let dir = scratch("copies");
    let saves = dir.join("saves");
    let (file, played) = a_played_game(&saves);
    let beside = files(&saves);
    assert_eq!(beside, ["island-7-families-6.json", "island-7-families-6.json.lock"]);

    // the first copy: it keeps its game in the island's autosave, and is busy for half a minute
    // playing a century and a half before it would open
    let mut first = start(&saves, &["--new", "--advance", "150", "--keep", "--describe", "30,30"]);
    let t0 = Instant::now();
    while !first.said_so_far().contains("making island") {
        assert!(first.child.try_wait().unwrap().is_none(), "the first copy ended before it made its island: {}", first.said_so_far());
        if t0.elapsed().as_secs_f32() > patience(300.0) {
            let _ = first.child.kill();
            timed_out(patience(300.0), "for the first copy to begin making its island", &first.said_so_far());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    // a second copy that would keep its game in the same place, however it is started
    let second = "the game could not start: another copy of the game is open and is keeping its game in ";
    let what_to_do = "island-7-families-6.json: play on in that window, or close it and start this one again (--save FILE keeps a second game in a file of its own)";
    for args in [&["--keep", "--describe", "30,30"][..], &["--new", "--keep", "--describe", "30,30"], &["--load", &text(&file), "--keep", "--describe", "30,30"], &["--keep", "--script", "print"]] {
        let ran = game(&saves, args);
        refused(&ran, 1, second, what_to_do);
    }
    // what is not kept there is not in its way
    let ran = game(&saves, &["--advance", "1", "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(ran.err.contains("nobody is at this game (it takes pictures or is played by a script), so it is not kept"), "{}", ran.err);
    let ran = game(&saves, &["--new", "--save", &text(&dir.join("mine.json")), "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert_eq!(std::fs::read(&file).unwrap(), played, "the first copy's game was touched");
    assert_eq!(files(&saves), beside);

    // the first copy dies where it stands. Its lock goes with it; the lock's file stays and means nothing
    assert!(
        first.child.try_wait().unwrap().is_none(),
        "the first copy had played its 150 years and ended before the checks above were done, so they showed nothing: the test was outrun, which is not a wrong answer"
    );
    first.crash();
    assert!(saves.join("island-7-families-6.json.lock").exists());
    let ran = game(&saves, &["--keep", "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "after a crash the next copy was kept out: {}", ran.err);
    assert!(ran.err.contains("the game is kept in "), "{}", ran.err);
    assert_eq!(std::fs::read(&file).unwrap(), played);
    assert_eq!(files(&saves), beside);
    let _ = std::fs::remove_dir_all(&dir);
}

/// What a script's `print` says of where the game stands and what the family holds.
fn standing(ran: &Ran, note: &str) -> (String, String, String) {
    let p = ran.printed(note);
    let date = p.split("date \"").nth(1).and_then(|s| s.split('"').next()).unwrap_or_default().to_string();
    let holds = p.lines().nth(1).and_then(|l| l.rsplit("; ").next()).unwrap_or_default().to_string();
    let waits = p.split("waiting for the year's end: ").nth(1).and_then(|s| s.lines().next()).unwrap_or_default().to_string();
    assert!(holds.starts_with("cash ") && !waits.is_empty(), "{p}");
    (date, holds, waits)
}

#[test]
#[ignore = "plays the game with --script, which needs a graphics adapter: run with -- --ignored --test-threads 1"]
fn a_script_with_years_played_first_leaves_the_played_game_alone() {
    let dir = scratch("trials");
    let saves = dir.join("saves");
    let (file, played) = a_played_game(&saves);
    let beside = files(&saves);
    // four test starts of the kind that used to set the island's game aside, each played a year on
    // and given an order: the fourth pushed the played game out of the copies kept
    for n in 1..=4 {
        let ran = game(&saves, &["--demo", "--advance", &n.to_string(), "--script", "key:Escape;key:Escape;key:Escape;key:Escape;click:Next month;settle;tab:holdings;click:Hold;settle;print:ordered;tab:none;click:To January;settle;key:Escape;key:Escape;key:Escape;key:Escape;print:on"]);
        assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
        assert!(ran.err.contains("nobody is at this game (it takes pictures or is played by a script), so it is not kept and no saved game is written over (--save FILE keeps it)"), "{}", ran.err);
        assert!(ran.printed("on").contains(&format!("newest year {}", 1751 + n)), "{}", ran.out);
        assert!(ran.printed("ordered").contains("the last order went through: Told.") && ran.printed("ordered").contains(" to hold; "), "the script's order was not taken: {}", ran.out);
        assert!(!ran.out.contains("script: no click"), "{}", ran.out);
        assert_eq!(std::fs::read(&file).unwrap(), played, "test start {n} wrote where the played game is kept");
        assert_eq!(files(&saves), beside, "test start {n} left something among the saved games");
    }
    // and a script that goes on with the island's game (ISOSTATE_SAVES names the folder) plays it on and writes nothing
    let ran = game(&saves, &["--script", "print:back;key:Escape;key:Escape;key:Escape;key:Escape;click:To January;settle;key:Escape;key:Escape;key:Escape;key:Escape;print:on"]);
    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert!(ran.printed("back").contains("the welcome card says: January 1753") && ran.printed("back").contains("Welcome back."), "{}", ran.out);
    assert!(ran.printed("on").contains("newest year 1753"), "{}", ran.out);
    assert_eq!(std::fs::read(&file).unwrap(), played);
    assert_eq!(files(&saves), beside);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "plays the game with --script, which needs a graphics adapter: run with -- --ignored --test-threads 1"]
fn a_file_that_cannot_be_read_is_set_aside_and_the_card_says_when_an_earlier_save_is_used() {
    let dir = scratch("unread");
    let saves = dir.join("saves");
    std::fs::create_dir_all(&saves).unwrap();
    let file = saves.join("island-7-families-6.json");
    // something in the autosave's place that is no saved game
    let junk = "{\"format\": 2, \"this is\": \"not what the game wrote\"";
    std::fs::write(&file, junk).unwrap();
    // a game is started and kept as the island's own (--keep: nobody is at it): the first decision
    // is answered and two years are played
    let clear = "key:Escape;key:Escape;key:Escape;key:Escape";
    // (Space closes the opening card and leaves the first decision's card open behind it; Escape would put that off too)
    let play = format!("print:open;key:Space;frames:3;decide:0;settle;{clear};click:To January;settle;{clear};click:To January;settle;{clear};print:on");
    let ran = game(&saves, &["--keep", "--script", &play]);
    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert!(ran.err.contains("a saved game could not be read: ") && ran.err.contains("making island 7 with 6 families"), "{}", ran.err);
    // the new game's opening card says why it is a new game, and that the file is kept and under what name
    let says = ran.line("script:   the welcome card says: ");
    assert!(
        says.contains(" | The saved game that was here could not be read, so this is a new one. That file is not written over: it is set aside as island-7-families-6.unread.json when this game is first saved, and the terminal says what is wrong with it. | Click anywhere to begin. The game saves itself as you play."),
        "{says}"
    );
    assert!(ran.printed("on").contains("newest year 1752") && !ran.out.contains("script: no click"), "{}", ran.out);
    // the file that could not be read is beside the save, whole, where the years cannot push it out
    assert_eq!(files(&saves), ["island-7-families-6.1.json", "island-7-families-6.2.json", "island-7-families-6.json", "island-7-families-6.json.lock", "island-7-families-6.unread.json"]);
    assert_eq!(std::fs::read_to_string(saves.join("island-7-families-6.unread.json")).unwrap(), junk);
    let read = |p: &Path| Save::from_text(&std::fs::read_to_string(p).unwrap()).unwrap();
    let latest = read(&file);
    assert_eq!((latest.year, latest.month, latest.orders.len()), (1752, 0, 1));
    assert_eq!(read(&saves.join("island-7-families-6.1.json")).year, 1751);

    // the latest is cut short, as a write that died half way would have left it before saves were renamed into place
    let whole = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, &whole[..whole.len() / 2]).unwrap();
    let beside = files(&saves);
    let card = dir.join("card.png");
    // (the picture waits for the renderer's shaders: the first frames of a run are black)
    let look = format!("print:card;settle;wait:8;shot:{};wait:2;click:Continue from January 1752;settle;print:on", text(&card));
    let ran = game(&saves, &["--script", &look]);
    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert!(ran.err.contains("the latest save could not be read (") && ran.err.contains("going back to the one before it"), "{}", ran.err);
    let says = ran.line("script:   the welcome card says: ");
    assert!(says.starts_with("January 1752 | The "), "{says}");
    assert!(says.contains(" | Welcome back. "), "{says}");
    assert!(says.contains(" | The last save could not be read, so this is an earlier one, from 1751. | "), "{says}");
    assert!(says.contains(" | Continue from January 1752 | New game | "), "{says}");
    assert!(says.ends_with(" | A new game begins again in 1750 and leaves this one saved beside it."), "{says}");
    assert!(ran.printed("card").contains("the welcome card is up") && !ran.printed("on").contains("the welcome card is up"), "{}", ran.out);
    // (a black frame of this size is 26 kilobytes; the island with the card over it some hundreds)
    assert!(std::fs::metadata(&card).is_ok_and(|m| m.len() > 150_000), "no picture of the card was taken, or a black one: {:?}", std::fs::metadata(&card).map(|m| m.len()));
    // a script only looked: the folder is as it was, the spoiled file still in the latest's place
    assert_eq!(files(&saves), beside);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), whole[..whole.len() / 2]);

    // gone on with and kept: the spoiled latest is set aside in its turn, behind the first, and the game is saved again
    let ran = game(&saves, &["--keep", "--script", &format!("click:Continue from January 1752;settle;{clear};click:To January;settle;{clear};print:on")]);
    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert!(ran.printed("on").contains("newest year 1752"), "{}", ran.out);
    assert_eq!(std::fs::read_to_string(saves.join("island-7-families-6.unread.json")).unwrap(), whole[..whole.len() / 2]);
    assert_eq!(std::fs::read_to_string(saves.join("island-7-families-6.unread.2.json")).unwrap(), junk);
    assert_eq!((read(&file).year, read(&file).started), (1752, latest.started), "the same game, a year on from the earlier save");
    // (the picture is left where it was taken when ISOSTATE_TEST_DIR names the folder, to be looked at)
    if std::env::var_os("ISOSTATE_TEST_DIR").is_none() {
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
#[ignore = "plays the game with --script, which needs a graphics adapter: run with -- --ignored --test-threads 1"]
fn a_whole_game_saved_left_and_loaded_again_lands_on_the_same_month_with_the_same_holdings() {
    let dir = scratch("century");
    let saves = dir.join("saves");
    let (a, b, c) = (dir.join("a.json"), dir.join("b.json"), dir.join("c.json"));
    let read = |p: &Path| Save::from_text(&std::fs::read_to_string(p).unwrap()).unwrap();
    let unchanged = |ran: &Ran| {
        assert!(!ran.err.contains("does not stand quite as it was saved") && !ran.err.contains("could not be given again") && !ran.err.contains("saved under other rules"), "{}", ran.err);
    };
    // cards that may be up (the opening card, decisions that waited, the card at the century's end) are closed or put off
    let clear = "key:Escape;key:Escape;key:Escape;key:Escape";
    let month = "click:Next month;settle";

    // 1750 to 1898 with the demo's works and land, then five months of 1899 through the clock, and
    // an order given in June that waits for the year's end; and the game is left
    let play = format!("{clear};{month};{month};{month};{month};{month};tab:holdings;click:Hold;settle;tab:none;print:left");
    let first = game(&saves, &["--new", "--demo", "--advance", "148", "--save", &text(&a), "--script", &play]);
    assert_eq!(first.code, Some(0), "{}\n{}", first.out, first.err);
    let left = standing(&first, "left");
    assert_eq!(left.0, "June 1899");
    assert!(left.2.contains(" to hold; "), "{left:?}");
    let saved = read(&a);
    assert_eq!((saved.format, saved.year, saved.month, saved.waiting.len()), (2, 1898, 5, 1), "the second layout: the months played and the order that waits");
    assert!(saved.orders.len() >= 2);

    // loaded again and saved at once in another file: the same month, the same island to the last digit
    let again = game(&saves, &["--load", &text(&a), "--save", &text(&b), "--describe", "30,30"]);
    assert_eq!(again.code, Some(0), "{}", again.err);
    assert!(again.err.contains("playing 149 years again") && again.err.contains("back 5 months into 1899"), "{}", again.err);
    unchanged(&again);
    let resaved = read(&b);
    assert_eq!((resaved.year, resaved.month, &resaved.check, resaved.worth, &resaved.orders, &resaved.waiting), (saved.year, saved.month, &saved.check, saved.worth, &saved.orders, &saved.waiting));

    // gone on with through the clock: the same month and holdings on screen; then on to January, when what
    // waited is carried out, and through 1900 to the end; and the game is left again
    let on = format!("{clear};print:back;click:To January;settle;{clear};print:turned;click:To January;settle;{clear};tab:none;print:ended");
    let second = game(&saves, &["--load", &text(&b), "--save", &text(&c), "--script", &on]);
    assert_eq!(second.code, Some(0), "{}\n{}", second.out, second.err);
    unchanged(&second);
    assert_eq!(standing(&second, "back"), left, "the game came back on another month or with other holdings");
    let turned = standing(&second, "turned");
    assert_eq!((turned.0.as_str(), turned.2.starts_with("nothing; ")), ("January 1900", true), "{turned:?}");
    let ended = standing(&second, "ended");
    assert!(second.printed("ended").contains("newest year 1900"), "{}", second.out);
    let last = read(&c);
    assert_eq!((last.year, last.month, last.waiting.len(), last.started), (1900, 0, 0, saved.started));
    assert_eq!(last.orders.len(), saved.orders.len() + 1, "the order that waited was carried out and is in the log");
    assert_eq!((last.orders.last().unwrap().year, last.orders.last().unwrap().month), (1898, 12), "{:?}", last.orders.last());

    // the whole game, loaded once more: 1900, and the same holdings
    let third = game(&saves, &["--load", &text(&c), "--script", &format!("{clear};tab:none;print:again")]);
    assert_eq!(third.code, Some(0), "{}\n{}", third.out, third.err);
    assert!(third.err.contains("playing 151 years again") && third.err.contains("back at the end of 1900"), "{}", third.err);
    unchanged(&third);
    let again = standing(&third, "again");
    assert_eq!(again.1, ended.1, "the finished game came back with other holdings (it ended on {:?} and came back on {:?})", ended.0, again.0);
    assert_eq!(again.0, "1900");
    assert!(!saves.exists(), "games kept in named files, and scripts: nothing belongs in the folder of autosaves");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_save_in_the_first_layout_still_loads() {
    let dir = scratch("first-layout");
    let saves = dir.join("saves");
    let (file, _) = a_played_game(&saves);
    let whole = std::fs::read_to_string(&file).unwrap();
    let now = Save::from_text(&whole).unwrap();
    // as the game wrote it before the clock told the month: no months played, nothing waiting, layout 1
    assert!(whole.contains("\"format\": 2,") && whole.contains("  \"month\": 0,\n") && whole.contains(",\n  \"waiting\": []\n"), "{whole}");
    let first = whole.replace("\"format\": 2,", "\"format\": 1,").replace("  \"month\": 0,\n", "").replace(",\n  \"waiting\": []\n", "\n");
    assert!(!first.contains("month") && !first.contains("waiting"), "{first}");
    let old = dir.join("first-layout.json");
    std::fs::write(&old, &first).unwrap();
    // loaded, and saved at once in another file: the same year, the same orders, the same island to the last digit
    let again = dir.join("again.json");
    let ran = game(&saves, &["--load", &text(&old), "--save", &text(&again), "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(ran.err.contains(&format!("playing 3 years again, with {} orders", now.orders.len())) && ran.err.contains("back at the end of 1752"), "{}", ran.err);
    assert!(!ran.err.contains("does not stand quite as it was saved") && !ran.err.contains("could not be given again") && !ran.err.contains("saved under other rules"), "{}", ran.err);
    let back = Save::from_text(&std::fs::read_to_string(&again).unwrap()).unwrap();
    assert_eq!((back.format, back.year, back.month, &back.check, &back.orders, back.waiting.len(), back.started), (2, now.year, 0, &now.check, &now.orders, 0, now.started));
    // and the file in the first layout is as it was
    assert_eq!(std::fs::read_to_string(&old).unwrap(), first);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_game_begun_where_another_is_kept_says_what_becomes_of_that_one() {
    let dir = scratch("said-aside");
    let saves = dir.join("saves");
    let (file, played) = a_played_game(&saves);
    let orders = Save::from_text(&String::from_utf8_lossy(&played)).unwrap().orders.len();
    let previous = saves.join("island-7-families-6.previous.json");
    // a new game in the played one's place: said before anything is written, and then done
    let ran = game(&saves, &["--new", "--save", &text(&file), "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    let said = ran.said();
    let told = said.iter().position(|l| l.starts_with("another game is kept there")).unwrap_or_else(|| panic!("nothing was said of the played game:\n{}", ran.err));
    assert_eq!(
        said[told],
        format!(
            "another game is kept there (the Fairfax family's, which had reached 1752 with {orders} orders): it is not written over, but set aside whole as {} when this game is first saved (--load FILE goes on with it)",
            previous.display()
        )
    );
    assert!(said.iter().position(|l| l.starts_with("saved in 1750: ")).is_some_and(|saved| saved > told), "{}", ran.err);
    assert_eq!(std::fs::read(&previous).unwrap(), played);
    assert_eq!(Save::from_text(&std::fs::read_to_string(&file).unwrap()).unwrap().year, 1750);
    // a new game over that one, which nobody gave an order in: it is written over, and that is said too
    let ran = game(&saves, &["--new", "--save", &text(&file), "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(
        ran.err.contains("another game is kept there (the Fairfax family's, which had reached 1750 with 0 orders): nobody gave an order in it, so it is written over when this game is first saved, and the same island makes it again"),
        "{}",
        ran.err
    );
    assert_eq!(std::fs::read(&previous).unwrap(), played, "the played game set aside was moved by a game with no orders");
    assert!(!saves.join("island-7-families-6.previous.2.json").exists());
    // and the game that is there, gone on with, has nothing of the kind said of it
    let ran = game(&saves, &["--load", &text(&file), "--save", &text(&file), "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(!ran.err.contains("another game is kept there"), "{}", ran.err);
    // the played game is gone on with by naming it, and takes its place back; the one it finds there has no orders
    let ran = game(&saves, &["--load", &text(&previous), "--save", &text(&file), "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(ran.err.contains("back at the end of 1752") && ran.err.contains("nobody gave an order in it, so it is written over"), "{}", ran.err);
    let back = Save::from_text(&std::fs::read_to_string(&file).unwrap()).unwrap();
    let was = Save::from_text(&String::from_utf8_lossy(&played)).unwrap();
    assert_eq!((back.year, &back.orders, &back.check, back.started), (was.year, &was.orders, &was.check, was.started));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "takes pictures, which needs a graphics adapter: run with -- --ignored --test-threads 1"]
fn runs_that_take_pictures_leave_the_played_game_alone() {
    let dir = scratch("pictures");
    let saves = dir.join("saves");
    let (file, played) = a_played_game(&saves);
    let beside = files(&saves);
    let pictures = dir.join("pictures");
    let (one, two, three, sheet) = (pictures.join("one.png"), pictures.join("two.png"), pictures.join("three.png"), pictures.join("sheet"));
    let view = format!("{},,30,30,0,55,40", text(&three));
    // every kind of start that takes pictures, on the island the played game is of, with and without years
    // played first and the demo's orders: none is kept, and none goes on with the played game
    for (args, picture) in [
        (&["--shot", &text(&one)][..], one.clone()),
        (&["--advance", "3", "--demo", "--shot", &text(&two)], two.clone()),
        (&["--new", "--advance", "2", "--view", &view], three.clone()),
        (&["--shots", &text(&sheet)], sheet.join("index.html")),
    ] {
        let ran = start(&saves, args).end(1800.0);
        assert_eq!(ran.code, Some(0), "{args:?}\n{}", ran.err);
        assert!(ran.err.contains("nobody is at this game (it takes pictures or is played by a script), so it is not kept and no saved game is written over (--save FILE keeps it)"), "{args:?}\n{}", ran.err);
        assert!(!ran.err.contains("going on with"), "{args:?}: a run that takes pictures went on with the played game\n{}", ran.err);
        assert!(picture.exists(), "{args:?} took no picture");
        assert_eq!(std::fs::read(&file).unwrap(), played, "{args:?} wrote where the played game is kept");
        assert_eq!(files(&saves), beside, "{args:?} left something among the saved games");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "takes a picture, which needs a graphics adapter: run with -- --ignored --test-threads 1"]
fn new_game_hands_the_place_over_to_the_copy_it_starts() {
    let dir = scratch("handed-over");
    let saves = dir.join("saves");
    let (file, played) = a_played_game(&saves);
    let picture = dir.join("new-game.png");
    // the played game is gone on with and kept as the island's own, so this copy holds its place; the
    // opening card's New game is pressed (ISOSTATE_WELCOME=new: nobody is at the pointer), which starts
    // the program again and leaves. The copy it starts says what it does in the same place as this one.
    let first = start_told(&saves, &["--load", &text(&file), "--keep", "--shot", &text(&picture)], Some("new"));
    let err = first.err.clone();
    let ran = first.end(600.0);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(ran.err.contains("the game is kept in ") && ran.err.contains("pressing the card's button, as ISOSTATE_WELCOME asks") && ran.err.contains("starting again on a new game"), "{}", ran.err);
    // the new copy waited for the old one to let go of the place and took it: it is not refused as a second copy
    let said = || String::from_utf8_lossy(&std::fs::read(&err).unwrap_or_default()).into_owned();
    let new_copy = |all: &str| all.split("starting again on a new game").nth(1).unwrap_or_default().to_string();
    let t0 = Instant::now();
    while !new_copy(&said()).contains("the game is kept in ") {
        assert!(!said().contains("another copy of the game is open"), "the new game was kept out by the copy that started it:\n{}", said());
        if t0.elapsed().as_secs_f32() > patience(600.0) {
            timed_out(patience(600.0), "for the new game to start", &said());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(new_copy(&said()).contains("making island 7 with 6 families"), "{}", said());
    // it takes its picture and ends, and lets go of the place in its turn (a lock that is held cannot be read)
    let lock = saves.join("island-7-families-6.json.lock");
    while !picture.exists() || std::fs::read(&lock).is_err() {
        if t0.elapsed().as_secs_f32() > patience(1800.0) {
            timed_out(patience(1800.0), "for the new game to take its picture and end", &said());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    // nothing has happened in the new game, so nothing is written yet: the played game is as it was
    assert_eq!(std::fs::read(&file).unwrap(), played);
    assert_eq!(files(&saves), ["island-7-families-6.json", "island-7-families-6.json.lock"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "plays the game with --script, which needs a graphics adapter: run with -- --ignored --test-threads 1"]
fn a_game_killed_a_dozen_times_as_it_plays_is_gone_on_with_each_time() {
    let dir = scratch("killed");
    let saves = dir.join("saves");
    let file = saves.join("island-7-families-6.json");
    let read = |p: &Path| Save::from_text(&std::fs::read_to_string(p).map_err(|e| e.to_string())?);
    let clear = "key:Escape;key:Escape;key:Escape;key:Escape";
    // the first copy begins the game and answers its first decision; every copy then runs the clock at
    // its fastest, answering what comes up, until it is killed where it stands
    let begin = format!("key:Space;frames:3;decide:0;settle;{clear};click:Fastest;run:900");
    let go_on = format!("{clear};click:Fastest;run:900");
    let mut last: Option<Save> = None;
    for round in 0..12u64 {
        let copy = start(&saves, &["--keep", "--script", if round == 0 { &begin } else { &go_on }]);
        // it is playing when its save has moved on from where the last copy left it
        let t0 = Instant::now();
        loop {
            let now = read(&file).ok();
            if now.as_ref().is_some_and(|now| last.as_ref().is_none_or(|l| (now.year, now.orders.len()) > (l.year, l.orders.len()))) {
                break;
            }
            let said = copy.said_so_far();
            assert!(!said.contains("another copy of the game is open"), "round {round}: the copy killed before this one kept it out:\n{said}");
            if t0.elapsed().as_secs_f32() > patience(1200.0) {
                copy.crash();
                timed_out(patience(1200.0), &format!("round {round}: for the game to get going and save"), &said);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        // some way into a year, or into its turn, or into the save of it: a different moment each round
        std::thread::sleep(Duration::from_millis(150 + round * 137 % 900));
        let said = copy.said_so_far();
        copy.crash();
        if round > 0 {
            assert!(said.contains("going on with the") && !said.contains("the latest save could not be read") && !said.contains("does not stand quite as it was saved") && !said.contains("could not be given again"), "round {round}:\n{said}");
        }
        // the latest is whole, the same game, and no further back than it had been seen; so are the years kept beside it
        let now = read(&file).unwrap_or_else(|e| panic!("round {round}: there is no latest save ({e})"));
        if let Some(l) = &last {
            assert_eq!(now.started, l.started, "round {round}: another game is in its place");
            assert!((now.year, now.month) >= (l.year, l.month) && now.orders.len() >= l.orders.len(), "round {round}: the game went back, from {} {} with {} orders to {} {} with {}", l.year, l.month, l.orders.len(), now.year, now.month, now.orders.len());
        }
        for name in files(&saves) {
            let known = ["island-7-families-6.json", "island-7-families-6.1.json", "island-7-families-6.2.json", "island-7-families-6.3.json", "island-7-families-6.json.lock", "island-7-families-6.writing.json", "island-7-families-6.1.copying.json"];
            assert!(known.contains(&name.as_str()), "round {round}: {name} is among the saved games");
            if name.ends_with(".json") && !name.contains("writing") && !name.contains("copying") {
                let year = read(&saves.join(&name)).unwrap_or_else(|e| panic!("round {round}: {name} is not whole ({e})"));
                assert_eq!(year.started, now.started);
            }
        }
        last = Some(now);
    }
    // a dozen copies on, the game is some years in with orders in it, and what is left loads to the
    // island as it was saved, to the last digit
    let last = last.unwrap();
    assert!(last.year >= 1755 && !last.orders.is_empty(), "the game got no further than {} with {} orders", last.year, last.orders.len());
    let again = dir.join("again.json");
    let ran = game(&saves, &["--load", &text(&file), "--save", &text(&again), "--describe", "30,30"]);
    assert_eq!(ran.code, Some(0), "{}", ran.err);
    assert!(!ran.err.contains("does not stand quite as it was saved") && !ran.err.contains("could not be given again"), "{}", ran.err);
    let back = read(&again).unwrap();
    assert_eq!((back.year, back.month, &back.check, &back.orders, &back.waiting), (last.year, last.month, &last.check, &last.orders, &last.waiting));
    let _ = std::fs::remove_dir_all(&dir);
}
