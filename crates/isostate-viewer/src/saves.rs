//! Where saved games are kept, and how they are written so that a bad write
//! cannot lose one. What is in a save is the core's business
//! (`isostate_core::save`): the orders given, to be played again.
//!
//! There is one autosave for each island and number of families,
//! `island-7-families-6.json`, in a folder of the player's own. Beside it are
//! the last few it replaced (`.1.json` the newest) and, once a new game has
//! been started on the same island, the game before it (`.previous.json`) and
//! the few before that (`.previous.2.json` and on). A file found in the
//! autosave's place that cannot be read is set aside as `.unread.json`.
//!
//! Four things keep a game somebody is playing from being lost:
//!
//! - Only play writes there. A start that is not plain play keeps its game
//!   in the `trials` folder beside the autosaves, or nowhere if nobody is at
//!   it (`live::Opening::keeping`).
//! - One copy of the game at a time keeps a game in one place: a `Saver` holds
//!   a lock on `island-7-families-6.json.lock` for as long as it lives. The
//!   lock is the system's, on the open file, so a game that crashes lets go
//!   of it; the file itself stays, and means nothing with no game holding it.
//! - There is at every moment a latest save that is whole. A save is written
//!   under another name and renamed into place, and the game it replaces is
//!   copied aside before that, never moved.
//! - A file that says what no game could have written is not played
//!   (`Save::sound`), and a file that cannot be read is set aside, not
//!   written over.

use std::path::{Path, PathBuf};

use isostate_core::save::Save;

/// How many earlier years are kept beside the latest: three. They are there
/// to fall back on when the latest cannot be read, and to go back a year or
/// two after a mistake. Each save is written whole and renamed into place, so
/// only something outside the game spoils one (a disk fault, a file opened in
/// an editor and saved), and it would have to spoil two running before the
/// third was wanted; more would only reach further back, a year at a time,
/// than a fall-back should take a player without asking. As many files that
/// could not be read are kept (`unread`).
pub const KEPT: usize = 3;

/// How many earlier games on the same island are kept: nine. Every game on
/// the island the game opens on by default shares one autosave, so this is
/// how many times a player can begin again there before the first game is
/// let go. A game is a log of orders, a few hundred kilobytes after a busy
/// century, so the number costs nothing; nine keeps the names to one digit.
/// Only a game somebody played takes a place: a trial is kept elsewhere, and
/// a game with no orders in it is not set aside at all (`holds_orders`). A
/// place is also taken by a game's own save when it holds orders the game no
/// longer does (the rules changed and some could not be given again, or the
/// game was taken back some years and played another way): those orders are
/// in no other file.
pub const GAMES_KEPT: usize = 9;

/// The folder for saved games: `ISOSTATE_SAVES` if set; else the usual place
/// for a program's own files (on Windows under %APPDATA%); else `saves`
/// beside the working directory.
pub fn folder() -> PathBuf {
    folder_from(|name| std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from))
}

/// `folder`, given how to look up an environment variable.
fn folder_from(var: impl Fn(&str) -> Option<PathBuf>) -> PathBuf {
    if let Some(dir) = var("ISOSTATE_SAVES") {
        return dir;
    }
    let home = if cfg!(windows) {
        var("APPDATA").map(|d| d.join("Isolated State"))
    } else if cfg!(target_os = "macos") {
        var("HOME").map(|d| d.join("Library").join("Application Support").join("Isolated State"))
    } else {
        var("XDG_DATA_HOME").or_else(|| var("HOME").map(|d| d.join(".local").join("share"))).map(|d| d.join("isolated-state"))
    };
    match home {
        Some(dir) => dir.join("saves"),
        None => PathBuf::from("saves"),
    }
}

/// The autosave of the game on this island with this many families.
pub fn autosave(seed: u64, families: usize) -> PathBuf {
    folder().join(format!("island-{seed}-families-{families}.json"))
}

/// Where a trial on this island is kept: a game from a start that is not
/// plain play (`--advance`, `--demo`) but has somebody at it. It is apart
/// from the island's own game, in a folder of its own, so that trying a thing
/// never sets a played game aside; `--load` goes on with one.
pub fn trial(seed: u64, families: usize) -> PathBuf {
    folder().join("trials").join(format!("island-{seed}-families-{families}.json"))
}

/// The file a copy of the game holds a lock on while it keeps a game in `path`.
pub fn lock_file(path: &Path) -> PathBuf {
    let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    path.with_file_name(format!("{name}.lock"))
}

/// `path` with a word before its extension: `a/b.json` and "1" give `a/b.1.json`.
fn beside(path: &Path, word: &str) -> PathBuf {
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    path.with_file_name(format!("{stem}.{word}{ext}"))
}

/// The n-th autosave before the latest (1 is the newest).
pub fn earlier(path: &Path, n: usize) -> PathBuf {
    beside(path, &n.to_string())
}

/// The game that was on this island before a new one was started.
pub fn previous(path: &Path) -> PathBuf {
    beside(path, "previous")
}

/// The n-th game back that was on this island (1 is `previous`).
pub fn game_before(path: &Path, n: usize) -> PathBuf {
    if n <= 1 { previous(path) } else { beside(path, &format!("previous.{n}")) }
}

/// What was at the autosave's place and could not be read, set aside when
/// the game is first saved.
pub fn unread(path: &Path) -> PathBuf {
    beside(path, "unread")
}

/// The n-th such file back (1 is `unread`): a second file that cannot be read
/// does not take the place of the first.
pub fn unread_before(path: &Path, n: usize) -> PathBuf {
    if n <= 1 { unread(path) } else { beside(path, &format!("unread.{n}")) }
}

/// What the system said went wrong, as a clause: its own words, without the
/// full stop Windows puts in the middle ("Access is denied. (os error 5)"),
/// so that a refusal built on it is still one sentence.
pub fn plainly(e: &std::io::Error) -> String {
    e.to_string().replace(". (os error", " (os error")
}

/// The largest file that is read as a saved game. A save is a line for each
/// order, a few hundred kilobytes after the busiest of centuries; anything
/// near this size is some other file, and is not taken into memory to find out.
pub const MOST_BYTES: u64 = 64 << 20;

pub fn read(path: &Path) -> Result<Save, String> {
    if let Some(len) = std::fs::metadata(path).ok().map(|m| m.len()).filter(|&len| len > MOST_BYTES) {
        return Err(format!("{}: not a saved game: it is {} megabytes, and a saved game is a few hundred kilobytes at the most", path.display(), len >> 20));
    }
    // (the system says of a folder only that access is denied, and of a picture that a stream was not valid)
    if path.is_dir() {
        return Err(format!("{}: not a saved game: it is a folder, and a saved game is a file in one", path.display()));
    }
    let text = std::fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::InvalidData => format!("{}: not a saved game: it is not a file of text", path.display()),
        _ => format!("{}: {}", path.display(), plainly(&e)),
    })?;
    Save::from_text(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Whether a save holds anything of the player's: an order carried out, or
/// one that waits. A game with none is its island's seed and a year, and the
/// seed makes it again, so it is not worth a place among the games kept.
pub fn holds_orders(save: &Save) -> bool {
    !save.orders.is_empty() || !save.waiting.is_empty()
}

/// Whether a game may be kept at a place that was named (`--save`): nothing
/// is there, or a saved game is, and a file can be written beside it.
/// Anything else there is somebody's file (or a folder), and is not moved
/// aside or written over. Asked before a year is played, so that a place that
/// cannot be written is told at once and not after a century.
pub fn may_keep(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("--save needs the name of a file to save in".to_string());
    }
    if path.is_dir() {
        return Err(format!("{} is a folder, and --save needs the name of a file: name one inside it", path.display()));
    }
    if path.exists() {
        read(path).map_err(|e| format!("{e}, and --save writes only where nothing is or over a saved game it can read: name another file"))?;
    }
    let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let cannot = |e: std::io::Error| format!("a game cannot be kept in {}, because nothing can be written in {} ({}): name a file in a folder that can be written", path.display(), dir.display(), plainly(&e));
    std::fs::create_dir_all(dir).map_err(cannot)?;
    // a file of its own name, made and taken away again: not the name a save is written under
    let trying = beside(path, "trying");
    std::fs::OpenOptions::new().write(true).create_new(true).open(&trying).map_err(cannot)?;
    let _ = std::fs::remove_file(&trying);
    Ok(())
}

/// Why a place to keep a game in could not be taken.
#[derive(Debug, PartialEq)]
pub enum Held {
    /// another copy of the game is keeping a game there
    ByAnother,
    /// the lock's file could not be made, and why: nothing can be written there
    Unwritable(String),
}

/// What is written in a lock's file, for whoever opens it.
const LOCK_WORDS: &str = "A copy of Isolated State holds this file open while it keeps the game saved beside it,\nso that a second copy does not write over that game. The file stays when the game is\nclosed; it means nothing then, and may be deleted.\n";

/// The newest autosave that can be read: the latest, or failing that the
/// ones it replaced. With it, why any newer one was passed over. None if
/// there is no save at all.
pub fn read_latest(path: &Path) -> Option<(Save, PathBuf, Vec<String>)> {
    let mut passed = Vec::new();
    for n in 0..=KEPT {
        let p = if n == 0 { path.to_path_buf() } else { earlier(path, n) };
        if !p.exists() {
            continue;
        }
        match read(&p) {
            Ok(save) => return Some((save, p, passed)),
            Err(e) => passed.push(e),
        }
    }
    for e in &passed {
        eprintln!("a saved game could not be read: {e}");
    }
    None
}

/// What a write that is stopped in a test says: the next step never happened,
/// as when the program dies there.
const STOPPED: &str = "stopped, as a crash would stop it";

/// Copy a file whole: under another name first and then renamed, so that
/// the copy is at no moment half there.
fn copy_whole(from: &Path, to: &Path) -> std::io::Result<()> {
    let by = beside(to, "copying");
    // (opened for writing: Windows will not flush a file opened only to be read)
    let copied = std::fs::copy(from, &by).and_then(|_| std::fs::OpenOptions::new().write(true).open(&by)?.sync_all()).and_then(|()| std::fs::rename(&by, to));
    if copied.is_err() {
        let _ = std::fs::remove_file(&by);
    }
    copied
}

/// Writes one game's saves to one place.
pub struct Saver {
    pub path: PathBuf,
    /// what was there before has been looked at (and set aside, if it was another game)
    settled: bool,
    /// the lock's file, held open for as long as this lives (`claim`); it is
    /// only ever held, and let go when this is dropped
    _lock: Option<std::fs::File>,
    /// changes made to the folder by the write in hand, and (in a test) the
    /// one before which it is to stop
    steps: usize,
    stop_before: Option<usize>,
}

impl Saver {
    /// A saver that takes no lock: for a place nobody else can be writing
    /// (a test's own folder), or one whose folder cannot hold a lock.
    pub fn new(path: PathBuf) -> Saver {
        Saver { path, settled: false, _lock: None, steps: 0, stop_before: None }
    }

    /// Take the place for this copy of the game alone, or say that another
    /// copy has it. The lock is the system's, on a file held open beside the
    /// save: it goes when this is dropped and when the program ends, however
    /// it ends, so a game that crashed does not keep the next one out.
    pub fn claim(path: PathBuf) -> Result<Saver, Held> {
        let told = |e: std::io::Error| Held::Unwritable(format!("{}: {}", path.display(), plainly(&e)));
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(told)?;
        }
        let file = std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(lock_file(&path)).map_err(told)?;
        match file.try_lock() {
            Ok(()) => {
                // (written under the lock; another copy cannot read it meanwhile, and need not)
                if file.metadata().is_ok_and(|m| m.len() == 0) {
                    use std::io::Write;
                    let _ = (&file).write_all(LOCK_WORDS.as_bytes());
                }
                Ok(Saver { _lock: Some(file), ..Saver::new(path) })
            }
            Err(std::fs::TryLockError::WouldBlock) => Err(Held::ByAnother),
            // a folder that cannot hold a lock (some network drives): the game is kept there without one
            Err(std::fs::TryLockError::Error(e)) => {
                eprintln!("{} cannot be locked ({e}): keep to one copy of the game there", lock_file(&path).display());
                Ok(Saver::new(path))
            }
        }
    }

    /// `claim`, waiting up to `patience` for another copy to let go: for a
    /// game started by the one that is closing (New game).
    pub fn claim_within(path: PathBuf, patience: std::time::Duration) -> Result<Saver, Held> {
        let t0 = std::time::Instant::now();
        loop {
            match Saver::claim(path.clone()) {
                Err(Held::ByAnother) if t0.elapsed() < patience => std::thread::sleep(std::time::Duration::from_millis(100)),
                got => return got,
            }
        }
    }

    /// Whether this copy holds the place's lock.
    #[cfg(test)]
    pub fn locked(&self) -> bool {
        self._lock.is_some()
    }

    /// Before a change to the folder. In a test a write can be told to stop
    /// here, and then leaves everything as a crash at that moment would.
    fn step(&mut self) -> Result<(), String> {
        self.steps += 1;
        if self.stop_before == Some(self.steps) { Err(STOPPED.to_string()) } else { Ok(()) }
    }

    /// Make room in the first of a row of files kept beside the save (the
    /// first, the second, and so on to `most`): each moves up a place, as far
    /// as the first gap, and with no gap the last is let go. Going only as far
    /// as a gap, a row that a crash left half moved is not moved onto itself.
    fn make_room(&mut self, name: &dyn Fn(usize) -> PathBuf, most: usize) -> Result<(), String> {
        let top = (1..=most).find(|&n| !name(n).exists()).unwrap_or(most);
        for n in (1..top).rev() {
            self.step()?;
            std::fs::rename(name(n), name(n + 1)).map_err(|e| format!("{}: {e}", self.path.display()))?;
        }
        Ok(())
    }

    /// Write the save in place of the latest. At a year's end the latest is
    /// first kept as an earlier one, so the last few years can be gone back to.
    ///
    /// The new file is written whole under another name and then renamed, so
    /// the latest is at every moment either the old one or the new one.
    pub fn write(&mut self, save: &Save, year_end: bool) -> Result<(), String> {
        let path = self.path.clone();
        // a folder of that name would be renamed like a file, with all that is in it
        if path.as_os_str().is_empty() || path.is_dir() {
            return Err(format!("{}: that is not the name of a file", path.display()));
        }
        let tmp = beside(&path, "writing");
        self.steps = 0;
        let written = self.write_by_way_of(&path, &tmp, save, year_end);
        if written.as_ref().is_err_and(|e| e != STOPPED) {
            // nothing half-written is left lying beside the save
            let _ = std::fs::remove_file(&tmp);
        }
        written
    }

    fn write_by_way_of(&mut self, path: &Path, tmp: &Path, save: &Save, year_end: bool) -> Result<(), String> {
        let told = |e: std::io::Error| format!("{}: {e}", path.display());
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(told)?;
        }
        self.step()?;
        {
            use std::io::Write;
            let mut f = std::fs::File::create(tmp).map_err(told)?;
            f.write_all(save.to_text().as_bytes()).map_err(told)?;
            // on the disk before it takes the latest's place
            f.sync_all().map_err(told)?;
        }
        // the latest is another game's: its years are not this game's to keep
        let mut others = false;
        if !self.settled {
            // the game that is here, and the file it is in: the latest, or if
            // that cannot be read the newest of the years kept beside it that
            // can, which is the one a player was offered to go on with
            let here = match read(path) {
                Ok(old) => Some((old, path.to_path_buf())),
                Err(_) => {
                    // a file that cannot be read (a save cut short, one written by
                    // a later version of the game, or one that says what no game
                    // could): it is set aside, where the years to come cannot
                    // push it out, and behind it the few set aside before
                    if path.exists() {
                        self.make_room(&|n| unread_before(path, n), KEPT)?;
                        self.step()?;
                        std::fs::rename(path, unread(path)).map_err(told)?;
                    }
                    (1..=KEPT).find_map(|n| read(&earlier(path, n)).ok().map(|old| (old, earlier(path, n))))
                }
            };
            // another game's save is here (a new game on the same island, or
            // a file named with --save): it is set aside whole, not written
            // over, and so are those set aside before it, so that starting a
            // new game by mistake does not lose the one that mattered. It is
            // copied aside and stays where it is until the new save takes its
            // place, so there is no moment with no latest save.
            if let Some((old, file)) = here {
                others = old.started != save.started || old.seed != save.seed || old.families != save.families;
                // The same game, but the save that is here holds orders this one
                // does not: the rules have changed since and some could not be
                // given again, or the game was taken back to an earlier year and
                // played another way. Those orders are in no other file, so it is
                // set aside like another game's. (A game played on only ever adds
                // to its orders.)
                let fuller = !others && !save.orders.starts_with(&old.orders);
                // (a game with nothing of the player's in it takes no place among
                // those kept, and one set aside already by a write that then
                // failed is not set aside twice)
                if (others || fuller) && holds_orders(&old) && read(&previous(path)).ok().as_ref() != Some(&old) {
                    self.make_room(&|n| game_before(path, n), GAMES_KEPT)?;
                    self.step()?;
                    copy_whole(&file, &previous(path)).map_err(told)?;
                }
                // another game's earlier saves are years the one set aside already
                // holds (and of a game with no orders, years the seed makes again):
                // the new game's years are not to push another game's out one by
                // one. The oldest goes first: a write that dies among them leaves
                // the newest, which is the one set aside, to be known again
                for n in (1..=KEPT).rev().filter(|_| others) {
                    if earlier(path, n).exists() {
                        self.step()?;
                        let _ = std::fs::remove_file(earlier(path, n));
                    }
                }
            }
        }
        // (a latest that the newest earlier save already is was kept by a write
        // that then failed: it is not kept a second time, pushing an older year out)
        let kept_already = || std::fs::read(path).is_ok_and(|latest| std::fs::read(earlier(path, 1)).is_ok_and(|one| one == latest));
        if year_end && !others && path.exists() && !kept_already() {
            self.make_room(&|n| earlier(path, n), KEPT)?;
            // copied, not moved: there is no moment without a latest
            self.step()?;
            copy_whole(path, &earlier(path, 1)).map_err(told)?;
        }
        self.step()?;
        std::fs::rename(tmp, path).map_err(told)?;
        // what was there has been dealt with once the new save is in its place, and not before:
        // a write that failed on the way looks again
        self.settled = true;
        Ok(())
    }
}

/// Seconds since 1970.
pub fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// The arguments to start this program again on a new game on the island
/// being played: the ones it was started with, less the save it was told to
/// load and the years it was told to play first, and with the island and the
/// number of families of the game on the table (a save that was loaded may
/// be of another island than the one the arguments name).
pub fn new_game_args(args: &[String], seed: u64, families: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            // (a script is of the game that was on the table, not of the new one)
            "--load" | "--seed" | "--families" | "--advance" | "--script" => i += 1,
            "--new" => {}
            a => out.push(a.to_string()),
        }
        i += 1;
    }
    out.extend(["--seed".to_string(), seed.to_string(), "--families".to_string(), families.to_string(), "--new".to_string()]);
    out
}

/// A folder of a test's own, empty: under `ISOSTATE_TEST_DIR` if that is set
/// (so that a helper's runs stay where it was told to keep them), else under
/// the system's folder for temporary files.
#[cfg(test)]
pub fn test_dir(name: &str) -> PathBuf {
    let root = std::env::var_os("ISOSTATE_TEST_DIR").filter(|v| !v.is_empty()).map_or_else(std::env::temp_dir, PathBuf::from);
    let dir = root.join(format!("isostate-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use isostate_core::save::{Given, Order, RulesId};

    fn save(started: u64, year: i64) -> Save {
        Save {
            format: 1,
            version: "0.1.0".into(),
            rules: RulesId { name: "game".into(), fingerprint: "0".into() },
            seed: 7,
            families: 6,
            started,
            saved: String::new(),
            year,
            month: 0,
            family: "Fairfax".into(),
            seat: "Ashby".into(),
            worth: 0,
            check: String::new(),
            orders: vec![Given { year: 1750, month: 0, order: Order::MoveSeat { town: 1 } }],
            waiting: Vec::new(),
        }
    }

    fn scratch(name: &str) -> PathBuf {
        test_dir(&format!("saves-test-{name}"))
    }

    fn files(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        v.sort();
        v
    }

    #[test]
    fn names() {
        let p = Path::new("x").join("island-7-families-6.json");
        assert_eq!(earlier(&p, 2), Path::new("x").join("island-7-families-6.2.json"));
        assert_eq!(previous(&p), Path::new("x").join("island-7-families-6.previous.json"));
        assert_eq!(game_before(&p, 1), previous(&p));
        assert_eq!(game_before(&p, 2), Path::new("x").join("island-7-families-6.previous.2.json"));
        assert_eq!(autosave(7, 6).file_name().unwrap(), "island-7-families-6.json");
        // a trial has the island's name in a folder of its own beside the autosaves
        assert!(trial(7, 6).ends_with(Path::new("trials").join("island-7-families-6.json")));
        assert_eq!(unread_before(&p, 1), unread(&p));
        assert_eq!(unread_before(&p, 2), Path::new("x").join("island-7-families-6.unread.2.json"));
        assert_eq!(lock_file(&p), Path::new("x").join("island-7-families-6.json.lock"));
    }

    #[test]
    fn the_folder_is_the_players_own_or_beside_the_working_directory() {
        let env = |pairs: &'static [(&'static str, &'static str)]| move |name: &str| pairs.iter().find(|(k, _)| *k == name).map(|(_, v)| PathBuf::from(v));
        // nothing to go by: beside the working directory
        assert_eq!(folder_from(env(&[])), PathBuf::from("saves"));
        // the usual place for this system
        let usual = folder_from(env(&[("APPDATA", "C:/Users/x/AppData/Roaming"), ("HOME", "/home/x"), ("XDG_DATA_HOME", "/data/x")]));
        if cfg!(windows) {
            assert_eq!(usual, Path::new("C:/Users/x/AppData/Roaming").join("Isolated State").join("saves"));
        } else if cfg!(target_os = "macos") {
            assert_eq!(usual, Path::new("/home/x/Library/Application Support/Isolated State/saves"));
        } else {
            assert_eq!(usual, Path::new("/data/x/isolated-state/saves"));
        }
        // a folder that was named wins
        assert_eq!(folder_from(env(&[("ISOSTATE_SAVES", "D:/games"), ("APPDATA", "C:/Users/x/AppData/Roaming")])), PathBuf::from("D:/games"));
    }

    #[test]
    fn the_last_few_years_are_kept_and_nothing_else() {
        let dir = scratch("kept");
        let path = dir.join("deep").join("game.json");
        let mut s = Saver::new(path.clone());
        for year in 1750..=1756 {
            s.write(&save(1, year), true).unwrap();
        }
        // an order between years is written in place: the earlier ones stay as they are
        s.write(&save(1, 1756), false).unwrap();
        assert_eq!(files(path.parent().unwrap()), ["game.1.json", "game.2.json", "game.3.json", "game.json"]);
        assert_eq!(read(&path).unwrap().year, 1756);
        assert_eq!(read(&earlier(&path, 1)).unwrap().year, 1755);
        assert_eq!(read(&earlier(&path, 2)).unwrap().year, 1754);
        assert_eq!(read(&earlier(&path, 3)).unwrap().year, 1753);
        assert_eq!(read(&path).unwrap(), save(1, 1756), "what is read is what was written");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_new_game_sets_the_old_one_aside() {
        let dir = scratch("aside");
        let path = dir.join("game.json");
        let mut old = Saver::new(path.clone());
        for year in 1750..=1790 {
            old.write(&save(1, year), true).unwrap();
        }
        // the same game going on (after a load) keeps its history
        let mut same = Saver::new(path.clone());
        same.write(&save(1, 1791), true).unwrap();
        assert_eq!(read(&earlier(&path, 1)).unwrap().year, 1790);
        assert!(!previous(&path).exists());
        // a new game: the old one is whole in .previous, and its earlier years are gone
        let mut new = Saver::new(path.clone());
        new.write(&save(2, 1750), true).unwrap();
        assert_eq!(files(&dir), ["game.json", "game.previous.json"]);
        assert_eq!(read(&previous(&path)).unwrap(), save(1, 1791));
        assert_eq!(read(&path).unwrap().started, 2);
        new.write(&save(2, 1751), true).unwrap();
        assert_eq!(files(&dir), ["game.1.json", "game.json", "game.previous.json"]);
        assert_eq!(read(&previous(&path)).unwrap().year, 1791, "the game set aside is left alone after that");
        // more new games: the last nine set aside are all kept, the newest first, and no more than that
        for started in 3..=13 {
            Saver::new(path.clone()).write(&save(started, 1750), true).unwrap();
        }
        let mut kept: Vec<String> = (2..=GAMES_KEPT).map(|n| format!("game.previous.{n}.json")).collect();
        kept.insert(0, "game.json".into());
        kept.push("game.previous.json".into());
        assert_eq!(files(&dir), kept);
        assert_eq!(read(&path).unwrap().started, 13);
        assert_eq!(read(&previous(&path)).unwrap().started, 12);
        assert_eq!(read(&game_before(&path, 2)).unwrap().started, 11);
        assert_eq!(read(&game_before(&path, GAMES_KEPT)).unwrap().started, 4);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A game nobody gave an order in: what a test start leaves.
    fn idle(started: u64, year: i64) -> Save {
        Save { orders: Vec::new(), ..save(started, year) }
    }

    #[test]
    fn a_game_with_no_orders_takes_no_place_among_those_kept() {
        let dir = scratch("idle");
        let path = dir.join("game.json");
        let mut played = Saver::new(path.clone());
        for year in 1750..=1790 {
            played.write(&save(1, year), true).unwrap();
        }
        assert!(holds_orders(&save(1, 1790)) && !holds_orders(&idle(2, 1750)));
        assert!(holds_orders(&Save { waiting: vec![Order::MoveSeat { town: 2 }], ..idle(2, 1750) }), "an order that waits is the player's too");
        // a dozen games that nobody gave an order in are started on the island, each played some years
        for started in 2..=13 {
            let mut s = Saver::new(path.clone());
            for year in 1750..=1754 {
                s.write(&idle(started, year), true).unwrap();
            }
        }
        // the played game was set aside by the first of them and is there still: the others took no place
        assert_eq!(files(&dir), ["game.1.json", "game.2.json", "game.3.json", "game.json", "game.previous.json"]);
        assert_eq!(read(&previous(&path)).unwrap(), save(1, 1790));
        assert_eq!(read(&path).unwrap(), idle(13, 1754));
        // and the played game set aside is not pushed out by a game with orders either, until nine have followed it
        Saver::new(path.clone()).write(&save(14, 1750), false).unwrap();
        assert_eq!(read(&previous(&path)).unwrap(), save(1, 1790), "the idle game before it was not set aside");
        assert!(!game_before(&path, 2).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_write_stopped_at_any_moment_leaves_a_whole_latest_save() {
        // what is in the game's place before the write, what is written, at a year's end or not,
        // and how many games had been set aside there before
        let old = save(1, 1790);
        let another_way = Save { orders: vec![Given { year: 1750, month: 0, order: Order::MoveSeat { town: 2 } }], ..save(1, 1788) };
        let cases: [(&str, Save, Save, bool, usize); 8] = [
            ("the same game, taken back two years and played another way", old.clone(), another_way.clone(), true, 2),
            ("the same game, with an order that could not be given again, by an order", old.clone(), Save { orders: Vec::new(), ..save(1, 1790) }, false, GAMES_KEPT),
            ("the same game, a year on", old.clone(), save(1, 1791), true, 2),
            ("the same game, an order given", old.clone(), save(1, 1790), false, 2),
            ("a new game over a played one", old.clone(), save(2, 1750), true, 2),
            ("a new game over a played one, by an order", old.clone(), save(2, 1750), false, 2),
            ("a new game over a played one, with as many set aside before as are kept", old.clone(), save(2, 1750), true, GAMES_KEPT),
            ("a new game over one nobody gave an order in", idle(1, 1790), save(2, 1750), true, 2),
        ];
        for (what, before, new, year_end, aside_before) in &cases {
            let earlier_games: Vec<u64> = (1..=*aside_before as u64).map(|n| 100 + n).collect();
            let mut stopped = 0;
            for stop in 1.. {
                let dir = scratch("stopped");
                let path = dir.join("game.json");
                let mut first = Saver::new(path.clone());
                for year in 1787..=before.year {
                    first.write(&Save { year, ..before.clone() }, true).unwrap();
                }
                // games were set aside here before, and must come through as they were
                for (n, started) in earlier_games.iter().enumerate() {
                    std::fs::write(game_before(&path, n + 1), save(*started, 1760).to_text()).unwrap();
                }

                let mut s = Saver::new(path.clone());
                s.stop_before = Some(stop);
                let wrote = s.write(new, *year_end);
                if wrote.is_ok() {
                    assert!(stop > 2, "{what}: a write has at least the two steps of writing and renaming");
                    let _ = std::fs::remove_dir_all(&dir);
                    break;
                }
                stopped += 1;
                assert_eq!(wrote.unwrap_err(), STOPPED);
                let when = format!("{what}, stopped before step {stop}");
                // the latest is whole, and is the game that was there or the one being written
                let latest = read(&path).unwrap_or_else(|e| panic!("{when}: there is no latest save ({e})"));
                assert!(latest == *before || latest == *new, "{when}: the latest is neither the old save nor the new");
                // a played game that was there can still be found, whole
                let set_aside = holds_orders(before) && (before.started != new.started || !new.orders.starts_with(&before.orders));
                if set_aside {
                    let found = latest == *before || (1..=GAMES_KEPT).any(|n| read(&game_before(&path, n)).is_ok_and(|g| g == *before));
                    assert!(found, "{when}: the game that was there is nowhere");
                }
                assert!(read_latest(&path).is_some());

                // the game is started again and saved: it ends as it would have with no crash
                Saver::new(path.clone()).write(new, *year_end).unwrap_or_else(|e| panic!("{when}, and then saved again: {e}"));
                assert_eq!(read(&path).unwrap(), *new, "{when}");
                let aside: Vec<u64> = (1..=GAMES_KEPT).filter_map(|n| read(&game_before(&path, n)).ok()).map(|g| g.started).collect();
                if set_aside {
                    // set aside once, not once for the write that stopped and again for the one that did
                    // not; and of those set aside before, only the oldest is let go, if there is no room
                    let mut want = vec![before.started];
                    want.extend(earlier_games.iter().take(GAMES_KEPT - 1));
                    assert_eq!(aside, want, "{when}");
                    assert_eq!(read(&previous(&path)).unwrap(), *before);
                } else {
                    assert_eq!(aside, earlier_games, "{when}");
                }
                // the years kept beside the latest are its own game's, and no year is there twice
                let years: Vec<(u64, i64)> = (1..=KEPT).filter_map(|n| read(&earlier(&path, n)).ok()).map(|g| (g.started, g.year)).collect();
                assert!(years.iter().all(|&(started, _)| started == new.started), "{when}: {years:?}");
                assert!(years.windows(2).all(|w| w[0].1 > w[1].1), "{when}: {years:?}");
                let _ = std::fs::remove_dir_all(&dir);
            }
            assert!(stopped >= 2, "{what}: the write was stopped {stopped} times");
        }
    }

    /// A game with `n` orders in it: the size of a busy game's save, so that
    /// a write of it takes long enough to be killed in.
    fn busy(started: u64, n: usize) -> Save {
        let orders = (0..n).map(|i| Given { year: 1750 + (i as i64 / 40).min(150), month: 0, order: Order::MoveSeat { town: i } }).collect();
        Save { orders, year: 1750 + (n as i64 / 40).min(150), ..save(started, 1750) }
    }

    /// Not a check of its own: the copy of the game that
    /// `a_game_killed_while_it_saves_leaves_a_whole_latest_save` starts and
    /// kills. It takes the place as a game does, goes on with the newest save
    /// that can be read (or begins a new game over it, if told to), and saves
    /// without rest, an order and then a year's end, until it is killed.
    #[test]
    #[ignore = "started, and killed, by a_game_killed_while_it_saves_leaves_a_whole_latest_save"]
    fn the_copy_that_is_killed() {
        let Some(dir) = std::env::var_os("ISOSTATE_KILLED_IN").filter(|v| !v.is_empty()) else { return };
        let path = PathBuf::from(dir).join("game.json");
        let mut s = match Saver::claim(path.clone()) {
            Ok(s) => s,
            Err(held) => {
                eprintln!("the copy before this one was killed, and its place is still held: {held:?}");
                std::process::exit(3);
            }
        };
        let (was, _, passed) = read_latest(&path).expect("there is a save to go on with");
        assert!(passed.is_empty(), "the latest could not be read: {passed:?}");
        let new = std::env::var_os("ISOSTATE_KILLED_NEW").is_some();
        let (started, mut n) = if new { (was.started + 1, 1500) } else { (was.started, was.orders.len()) };
        loop {
            n += 1;
            s.write(&busy(started, n), false).unwrap();
            s.write(&busy(started, n), true).unwrap();
        }
    }

    #[test]
    fn saving_a_busy_game_takes_a_small_part_of_a_year() {
        // a year at the clock's fastest is half a second, and the simulation's thread writes the save
        // after it has handed the year over: what the writing costs (a file written and put on the
        // disk, the latest copied aside and put on the disk, four renames) must stay a small part of that
        let dir = scratch("timed");
        let path = dir.join("game.json");
        let game = busy(1, 8000);
        let size = game.to_text().len();
        assert!(size > 300_000, "{size}");
        let mut s = Saver::new(path.clone());
        s.write(&game, true).unwrap();
        let (mut year_ends, mut orders) = (Vec::new(), Vec::new());
        for n in 0..40 {
            let t0 = std::time::Instant::now();
            s.write(&busy(1, 8001 + n), true).unwrap();
            year_ends.push(t0.elapsed().as_secs_f32() * 1000.0);
            let t0 = std::time::Instant::now();
            s.write(&busy(1, 8001 + n), false).unwrap();
            orders.push(t0.elapsed().as_secs_f32() * 1000.0);
        }
        // and reading one, which a start does once and a save's first write once more
        let t0 = std::time::Instant::now();
        read(&path).unwrap();
        let reading = t0.elapsed().as_secs_f32() * 1000.0;
        let middle = |v: &mut Vec<f32>| {
            v.sort_by(|a, b| a.total_cmp(b));
            (v[v.len() / 2], v[v.len() - 1])
        };
        let (year_end, year_end_worst) = middle(&mut year_ends);
        let (order, order_worst) = middle(&mut orders);
        println!("a save of {} kilobytes: at a year's end {year_end:.1} ms (the slowest {year_end_worst:.1}), after an order {order:.1} ms (the slowest {order_worst:.1}), read in {reading:.1} ms", size / 1000);
        assert!(year_end < 100.0 && order < 100.0, "a save takes {year_end:.0} ms at a year's end and {order:.0} ms after an order");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_game_killed_while_it_saves_leaves_a_whole_latest_save() {
        let dir = scratch("killed");
        let path = dir.join("game.json");
        Saver::new(path.clone()).write(&busy(1, 1500), true).unwrap();
        let mut last = read(&path).unwrap();
        let mut aside: Vec<Save> = Vec::new();
        let mut in_a_write = 0;
        // thirty-six copies of the game, each killed where it stands a few milliseconds after it has got
        // going; every third begins a new game over the one before, more of them than games are kept
        for round in 0..36u64 {
            let new = round % 3 == 2;
            let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
            cmd.args(["--ignored", "--exact", "saves::tests::the_copy_that_is_killed", "--nocapture"]).env("ISOSTATE_KILLED_IN", &dir).env_remove("ISOSTATE_KILLED_NEW");
            cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
            if new {
                cmd.env("ISOSTATE_KILLED_NEW", "1");
            }
            let mut child = cmd.spawn().unwrap();
            // it has the place and is saving when the latest is no longer the one it found
            let t0 = std::time::Instant::now();
            while read(&path).is_ok_and(|now| now == last) {
                assert!(child.try_wait().unwrap().is_none(), "round {round}: the copy ended by itself: a lock left by the one killed before it kept it out, or it could not go on with what was left");
                assert!(t0.elapsed().as_secs() < 60, "round {round}: the copy never saved");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            // (a time that differs from round to round, and is not a round number of saves)
            let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos() as u64;
            std::thread::sleep(std::time::Duration::from_micros(500 + (nanos / 7 + round * 2_113) % 40_000));
            child.kill().unwrap();
            child.wait().unwrap();

            let when = format!("round {round}{}", if new { ", a new game" } else { "" });
            // the latest is whole, and is the game that was being played, no further back than it had been seen
            let latest = read(&path).unwrap_or_else(|e| panic!("{when}: there is no latest save ({e})"));
            assert_eq!(latest, busy(latest.started, latest.orders.len()), "{when}: the latest is not a save the game wrote");
            // every file set aside is whole
            let there: Vec<Save> = (1..=GAMES_KEPT).filter(|&n| game_before(&path, n).exists()).map(|n| read(&game_before(&path, n)).unwrap_or_else(|e| panic!("{when}: a game set aside is not whole ({e})"))).collect();
            if new {
                // the game before it is the latest still, or is set aside whole, exactly as it was left
                assert!(latest.started == last.started + 1 || latest == last, "{when}: the latest is neither game");
                assert!(latest == last || there.first() == Some(&last), "{when}: the game that was there was not set aside as it stood");
                // (it may have been set aside and the copy killed before the new game took its place)
                if there.first() == Some(&last) && aside.first() != Some(&last) {
                    aside.insert(0, last.clone());
                }
            } else {
                assert!(latest.started == last.started && latest.orders.len() >= last.orders.len(), "{when}: the game went back, from {} orders to {}", last.orders.len(), latest.orders.len());
            }
            // the games set aside before are all still there, in their order; a copy killed while it made
            // room among as many as are kept may have let the oldest go, and no more than that
            assert!(there.len() <= aside.len() && there.len() + 1 >= aside.len().min(GAMES_KEPT), "{when}: {} games had been set aside and {} are there", aside.len(), there.len());
            assert!(there[..] == aside[..there.len()], "{when}: the games set aside are not the ones that were, in their order");
            aside.truncate(there.len());
            // every earlier year kept is whole, and of the game in the latest's place
            for n in 1..=KEPT {
                if earlier(&path, n).exists() {
                    let year = read(&earlier(&path, n)).unwrap_or_else(|e| panic!("{when}: an earlier year is not whole ({e})"));
                    assert_eq!(year.started, latest.started, "{when}: an earlier year of another game is beside the latest");
                }
            }
            // nothing is in the folder but what a game keeps there, and at most what one write was in the middle of
            let known = |name: &str| name == "game.json" || name == "game.json.lock" || name == "game.writing.json" || name.ends_with(".copying.json")
                || (1..=KEPT).any(|n| name == format!("game.{n}.json")) || name == "game.previous.json" || (2..=GAMES_KEPT).any(|n| name == format!("game.previous.{n}.json"));
            let there = files(&dir);
            assert!(there.iter().all(|name| known(name)), "{when}: {there:?}");
            if there.iter().any(|name| name == "game.writing.json" || name.ends_with(".copying.json")) {
                in_a_write += 1;
            }
            last = latest;
        }
        assert!(aside.len() >= 5, "few of the new games got as far as their first save: {}", aside.len());
        // (not every kill lands in the middle of a write, but if none did this showed nothing)
        assert!(in_a_write >= 3, "only {in_a_write} of 36 copies were killed in the middle of a write");
        // what the last one left in the middle of is cleared by the next save, and the game goes on
        let mut s = Saver::claim(path.clone()).unwrap();
        s.write(&busy(last.started, last.orders.len() + 1), true).unwrap();
        assert!(!files(&dir).iter().any(|name| name == "game.writing.json"));
        drop(s);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_second_copy_cannot_take_the_place_and_a_crash_does_not_keep_it() {
        let dir = scratch("lock");
        let path = dir.join("deep").join("game.json");
        assert_eq!(lock_file(&path), dir.join("deep").join("game.json.lock"));
        let mut first = Saver::claim(path.clone()).unwrap();
        assert!(first.locked());
        first.write(&save(1, 1750), true).unwrap();
        // a second copy is told so, and has written nothing
        assert_eq!(Saver::claim(path.clone()).err(), Some(Held::ByAnother));
        assert_eq!(Saver::claim_within(path.clone(), std::time::Duration::from_millis(250)).err(), Some(Held::ByAnother));
        assert_eq!(files(path.parent().unwrap()), ["game.json", "game.json.lock"]);
        assert_eq!(read(&path).unwrap(), save(1, 1750));
        // a place beside it is another place
        assert!(Saver::claim(dir.join("deep").join("other.json")).is_ok());
        // the first copy closes while a second waits for it (New game): the second gets the place
        let closing = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            drop(first);
        });
        let second = Saver::claim_within(path.clone(), std::time::Duration::from_secs(20)).unwrap();
        closing.join().unwrap();
        assert!(second.locked());
        // the lock's file is left behind when a game ends, as a crash leaves it; it keeps nobody out
        drop(second);
        assert!(lock_file(&path).exists());
        assert!(std::fs::read_to_string(lock_file(&path)).unwrap().contains("may be deleted"));
        let third = Saver::claim(path.clone()).unwrap();
        assert!(third.locked());
        // a place where nothing can be written: its folder is a file
        std::fs::write(dir.join("notes.txt"), "not a folder").unwrap();
        match Saver::claim(dir.join("notes.txt").join("game.json")) {
            Err(Held::Unwritable(_)) => {}
            other => panic!("a place inside a file was taken: {:?}", other.map(|s| s.path)),
        }
        drop(third);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_second_file_that_cannot_be_read_does_not_take_the_place_of_the_first() {
        let dir = scratch("unread");
        let path = dir.join("game.json");
        std::fs::create_dir_all(&dir).unwrap();
        for (n, junk) in ["one", "two", "three", "four"].iter().enumerate() {
            std::fs::write(&path, junk).unwrap();
            Saver::new(path.clone()).write(&save(n as u64, 1750), false).unwrap();
        }
        assert_eq!(files(&dir), ["game.json", "game.unread.2.json", "game.unread.3.json", "game.unread.json"]);
        assert_eq!(std::fs::read_to_string(unread(&path)).unwrap(), "four");
        assert_eq!(std::fs::read_to_string(unread_before(&path, 2)).unwrap(), "three");
        assert_eq!(std::fs::read_to_string(unread_before(&path, 3)).unwrap(), "two");
        // so is a file far too large to be a saved game: it is not read to find out, and it is kept
        let huge = dir.join("huge.json");
        std::fs::File::create(&huge).unwrap().set_len(MOST_BYTES + 1).unwrap();
        let t0 = std::time::Instant::now();
        assert!(read(&huge).unwrap_err().ends_with("not a saved game: it is 64 megabytes, and a saved game is a few hundred kilobytes at the most"));
        assert!(may_keep(&huge).unwrap_err().contains("not a saved game"));
        assert!(t0.elapsed().as_secs_f32() < 1.0, "a file that size was read");
        std::fs::remove_file(&huge).unwrap();
        // a save that says what no game could is such a file: it is not played, and not written over
        let wild = save(9, 1790).to_text().replace("\"year\": 1790,", "\"year\": 5000,");
        std::fs::write(&path, &wild).unwrap();
        assert!(read(&path).unwrap_err().contains("the year 5000"));
        assert!(read_latest(&path).is_none());
        Saver::new(path.clone()).write(&save(10, 1750), false).unwrap();
        assert_eq!(std::fs::read_to_string(unread(&path)).unwrap(), wild);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_place_that_cannot_be_written_is_refused_before_anything_is_played() {
        let dir = scratch("unwritable");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("notes.txt"), "not a folder").unwrap();
        let e = may_keep(&dir.join("notes.txt").join("game.json")).unwrap_err();
        assert!(e.starts_with("a game cannot be kept in ") && e.contains("because nothing can be written in ") && e.ends_with("name a file in a folder that can be written"), "{e}");
        let e = may_keep(&dir).unwrap_err();
        assert!(e.ends_with("is a folder, and --save needs the name of a file: name one inside it"), "{e}");
        let e = may_keep(&dir.join("notes.txt")).unwrap_err();
        assert!(e.contains("not a saved game") && e.ends_with("name another file"), "{e}");
        // a free place in a folder still to be made is fine, and asking leaves nothing but the folder
        assert_eq!(may_keep(&dir.join("new").join("game.json")), Ok(()));
        assert_eq!(files(&dir.join("new")), Vec::<String>::new());
        assert_eq!(files(&dir), ["new", "notes.txt"]);
        assert_eq!(std::fs::read_to_string(dir.join("notes.txt")).unwrap(), "not a folder");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_spoiled_save_falls_back_to_the_one_before() {
        let dir = scratch("spoiled");
        let path = dir.join("game.json");
        assert!(read_latest(&path).is_none(), "no save, nothing to go on from");
        let mut s = Saver::new(path.clone());
        for year in 1750..=1752 {
            s.write(&save(1, year), true).unwrap();
        }
        let (got, from, passed) = read_latest(&path).unwrap();
        assert_eq!((got.year, from.clone(), passed.len()), (1752, path.clone(), 0));
        // half a file, as a write cut short without the rename would have left
        std::fs::write(&path, &save(1, 1752).to_text()[..40]).unwrap();
        let (got, from, passed) = read_latest(&path).unwrap();
        assert_eq!((got.year, from), (1751, earlier(&path, 1)));
        assert_eq!(passed.len(), 1);
        assert!(passed[0].contains("not a saved game"), "{}", passed[0]);
        // the next write sets the spoiled file aside and does not stop; the years kept stay as they were
        let mut again = Saver::new(path.clone());
        again.write(&got, true).unwrap();
        assert_eq!(read(&path).unwrap().year, 1751);
        assert_eq!(std::fs::read_to_string(unread(&path)).unwrap().len(), 40);
        assert_eq!(read(&earlier(&path, 1)).unwrap().year, 1751);
        assert_eq!(read(&earlier(&path, 2)).unwrap().year, 1750);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_new_game_begun_when_the_latest_cannot_be_read_sets_the_earlier_save_aside() {
        // a played game whose latest save was spoiled from outside: the card offers the year before
        // it ("The last save could not be read, so this is an earlier one") and New game beside it
        let dir = scratch("spoiled-new");
        let path = dir.join("game.json");
        let mut s = Saver::new(path.clone());
        for year in 1750..=1790 {
            s.write(&save(1, year), true).unwrap();
        }
        std::fs::write(&path, &save(1, 1790).to_text()[..40]).unwrap();
        assert_eq!(read_latest(&path).unwrap().0, save(1, 1789));
        // New game is chosen, and the new game is played for longer than earlier years are kept
        let mut new = Saver::new(path.clone());
        new.write(&save(2, 1750), false).unwrap();
        // the played game is set aside whole, as its latest would have been: the newest year of it that can be read
        assert_eq!(read(&previous(&path)).unwrap(), save(1, 1789), "the game that was there is nowhere but among the years kept");
        assert_eq!(files(&dir), ["game.json", "game.previous.json", "game.unread.json"]);
        for year in 1751..=1760 {
            new.write(&save(2, year), true).unwrap();
        }
        assert_eq!(read(&previous(&path)).unwrap(), save(1, 1789), "the years of the new game pushed the old one out");
        assert_eq!(std::fs::read_to_string(unread(&path)).unwrap().len(), 40);
        assert_eq!(files(&dir), ["game.1.json", "game.2.json", "game.3.json", "game.json", "game.previous.json", "game.unread.json"]);
        assert!((1..=KEPT).all(|n| read(&earlier(&path, n)).unwrap().started == 2));

        // the same with no file at all in the latest's place (a write that set the spoiled one aside and then died)
        let dir2 = scratch("spoiled-gone");
        let path2 = dir2.join("game.json");
        let mut s = Saver::new(path2.clone());
        for year in 1750..=1790 {
            s.write(&save(1, year), true).unwrap();
        }
        std::fs::remove_file(&path2).unwrap();
        let mut new = Saver::new(path2.clone());
        for year in 1750..=1760 {
            new.write(&save(2, year), true).unwrap();
        }
        assert_eq!(read(&previous(&path2)).unwrap(), save(1, 1789));
        // and the same game going on from the earlier save keeps its years and sets nothing aside
        let dir3 = scratch("spoiled-on");
        let path3 = dir3.join("game.json");
        let mut s = Saver::new(path3.clone());
        for year in 1750..=1790 {
            s.write(&save(1, year), true).unwrap();
        }
        std::fs::write(&path3, "junk").unwrap();
        Saver::new(path3.clone()).write(&save(1, 1790), true).unwrap();
        assert!(!previous(&path3).exists());
        assert_eq!((1..=KEPT).map(|n| read(&earlier(&path3, n)).unwrap().year).collect::<Vec<_>>(), [1789, 1788, 1787]);
        for d in [dir, dir2, dir3] {
            let _ = std::fs::remove_dir_all(&d);
        }

        // that first write of the new game, stopped before each of its steps as a crash would stop it: the
        // played game can be found whole at every one, and saving again ends as if nothing had happened
        let mut stopped = 0;
        for stop in 1.. {
            let dir = scratch("spoiled-stopped");
            let path = dir.join("game.json");
            let mut s = Saver::new(path.clone());
            for year in 1785..=1790 {
                s.write(&save(1, year), true).unwrap();
            }
            std::fs::write(&path, "junk").unwrap();
            let mut new = Saver::new(path.clone());
            new.stop_before = Some(stop);
            let wrote = new.write(&save(2, 1750), true);
            if wrote.is_ok() {
                let _ = std::fs::remove_dir_all(&dir);
                break;
            }
            stopped += 1;
            let found = || (1..=KEPT).map(|n| earlier(&path, n)).chain([previous(&path)]).any(|p| read(&p).is_ok_and(|g| g == save(1, 1789)));
            assert!(found(), "stopped before step {stop}: the played game is nowhere");
            Saver::new(path.clone()).write(&save(2, 1750), true).unwrap();
            assert_eq!(read(&path).unwrap(), save(2, 1750));
            assert_eq!(read(&previous(&path)).unwrap(), save(1, 1789), "stopped before step {stop}, and saved again");
            assert!(!game_before(&path, 2).exists(), "stopped before step {stop}: the game was set aside twice");
            assert_eq!(std::fs::read_to_string(unread(&path)).unwrap(), "junk");
            assert!(!unread_before(&path, 2).exists() && !earlier(&path, 1).exists());
            let _ = std::fs::remove_dir_all(&dir);
        }
        assert!(stopped >= 5, "the write was stopped {stopped} times");
    }

    #[test]
    fn a_save_of_a_later_version_is_not_pushed_out_by_the_years() {
        let dir = scratch("later");
        let path = dir.join("game.json");
        std::fs::create_dir_all(&dir).unwrap();
        let later = save(1, 1790).to_text().replace("\"format\": 1,", "\"format\": 3,");
        std::fs::write(&path, &later).unwrap();
        assert!(read_latest(&path).is_none(), "this version cannot go on with it");
        // a new game is played on the island for longer than earlier years are kept
        let mut s = Saver::new(path.clone());
        s.write(&save(2, 1750), false).unwrap();
        for year in 1751..=1760 {
            s.write(&save(2, year), true).unwrap();
        }
        assert_eq!(std::fs::read_to_string(unread(&path)).unwrap(), later);
        assert_eq!(files(&dir), ["game.1.json", "game.2.json", "game.3.json", "game.json", "game.unread.json"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_folder_or_somebody_elses_file_is_left_alone() {
        let dir = scratch("alone");
        let folder = dir.join("runs");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("run.bin"), b"a recorded run").unwrap();
        // a folder is not renamed out of the way, and nothing is left beside it
        let e = Saver::new(folder.clone()).write(&save(1, 1750), true).unwrap_err();
        assert!(e.contains("not the name of a file"), "{e}");
        assert!(Saver::new(PathBuf::new()).write(&save(1, 1750), true).is_err());
        assert_eq!(files(&dir), ["runs"]);
        assert_eq!(files(&folder), ["run.bin"]);
        // a place named with --save: free, or a saved game, and nothing else
        let run = folder.join("run.bin");
        assert!(may_keep(&run).unwrap_err().contains("not a saved game"));
        assert!(may_keep(&folder).is_err());
        // what is read as a saved game and is a folder, or is no text at all, is said to be that, in the game's words
        assert!(read(&folder).unwrap_err().ends_with("runs: not a saved game: it is a folder, and a saved game is a file in one"));
        std::fs::write(folder.join("picture.png"), [0x89, b'P', b'N', b'G', 0xff, 0xfe, 0x00]).unwrap();
        assert!(read(&folder.join("picture.png")).unwrap_err().ends_with("picture.png: not a saved game: it is not a file of text"));
        std::fs::remove_file(folder.join("picture.png")).unwrap();
        assert!(may_keep(Path::new("")).unwrap_err().contains("needs the name"));
        assert!(may_keep(&folder.join("new.json")).is_ok());
        let mine = folder.join("mine.json");
        Saver::new(mine.clone()).write(&save(1, 1750), true).unwrap();
        assert!(may_keep(&mine).is_ok());
        // a write that fails part of the way leaves no half-written file: here
        // an earlier year cannot move up, its name being taken by a folder
        std::fs::write(earlier(&mine, 1), b"").unwrap();
        std::fs::write(earlier(&mine, 2), b"").unwrap();
        std::fs::create_dir_all(earlier(&mine, 3)).unwrap();
        std::fs::write(earlier(&mine, 3).join("x"), b"x").unwrap();
        assert!(Saver::new(mine.clone()).write(&save(1, 1751), true).is_err());
        assert_eq!(files(&folder), ["mine.1.json", "mine.2.json", "mine.3.json", "mine.json", "run.bin"]);
        assert_eq!(read(&mine).unwrap().year, 1750, "the latest is still the one before");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_new_game_starts_the_program_again_without_the_save() {
        // the save that was loaded is of island 11 with three families, whatever the arguments said
        let args: Vec<String> = ["--families", "4", "--load", "a b.json", "--size", "1280x800", "--advance", "5"].iter().map(|s| s.to_string()).collect();
        assert_eq!(new_game_args(&args, 11, 3), ["--size", "1280x800", "--seed", "11", "--families", "3", "--new"]);
        assert_eq!(new_game_args(&["--new".to_string()], 7, 6), ["--seed", "7", "--families", "6", "--new"]);
        let args: Vec<String> = ["--script", "click:New game", "--size", "900x600"].iter().map(|s| s.to_string()).collect();
        assert_eq!(new_game_args(&args, 7, 6), ["--size", "900x600", "--seed", "7", "--families", "6", "--new"]);
        // a place that was named goes on being the place
        let args: Vec<String> = ["--save", "mine.json", "--seed", "9"].iter().map(|s| s.to_string()).collect();
        assert_eq!(new_game_args(&args, 9, 6), ["--save", "mine.json", "--seed", "9", "--families", "6", "--new"]);
    }
}
