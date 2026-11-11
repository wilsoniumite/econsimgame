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

use std::path::{Path, PathBuf};

use isostate_core::save::Save;

/// How many earlier autosaves are kept beside the latest, and how many
/// earlier games on the same island.
pub const KEPT: usize = 3;

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

pub fn read(path: &Path) -> Result<Save, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Save::from_text(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Whether a game may be kept at a place that was named (`--save`): nothing
/// is there, or a saved game is. Anything else is somebody's file (or a
/// folder), and is not moved aside or written over.
pub fn may_keep(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("--save needs the name of a file to save in".to_string());
    }
    if !path.exists() {
        return Ok(());
    }
    read(path).map(|_| ()).map_err(|e| format!("{e} (--save only writes over a saved game it can read)"))
}

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

/// Writes one game's saves to one place.
pub struct Saver {
    pub path: PathBuf,
    /// what was there before has been looked at (and set aside, if it was another game)
    settled: bool,
}

impl Saver {
    pub fn new(path: PathBuf) -> Saver {
        Saver { path, settled: false }
    }

    /// Write the save in place of the latest. At a year's end the latest is
    /// first kept as an earlier one, so the last few years can be gone back to.
    ///
    /// The new file is written whole under another name and then renamed, so
    /// the latest is at every moment either the old one or the new one.
    pub fn write(&mut self, save: &Save, year_end: bool) -> Result<(), String> {
        let path = &self.path;
        // a folder of that name would be renamed like a file, with all that is in it
        if path.as_os_str().is_empty() || path.is_dir() {
            return Err(format!("{}: that is not the name of a file", path.display()));
        }
        let tmp = beside(path, "writing");
        let written = self.write_by_way_of(&tmp, save, year_end);
        if written.is_err() {
            // nothing half-written is left lying beside the save
            let _ = std::fs::remove_file(&tmp);
        }
        written
    }

    fn write_by_way_of(&mut self, tmp: &Path, save: &Save, year_end: bool) -> Result<(), String> {
        let path = &self.path;
        let told = |e: std::io::Error| format!("{}: {e}", path.display());
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(told)?;
        }
        {
            use std::io::Write;
            let mut f = std::fs::File::create(tmp).map_err(told)?;
            f.write_all(save.to_text().as_bytes()).map_err(told)?;
            // on the disk before it takes the latest's place
            f.sync_all().map_err(told)?;
        }
        if !self.settled {
            match read(path) {
                // another game's save is here (a new game on the same island,
                // or a file named with --save): it is set aside whole, not
                // written over, and so are the few set aside before it, so
                // that starting a new game twice by mistake does not lose the
                // one that mattered
                Ok(old) if old.started != save.started || old.seed != save.seed || old.families != save.families => {
                    for n in (1..KEPT).rev() {
                        let from = game_before(path, n);
                        if from.exists() {
                            std::fs::rename(&from, game_before(path, n + 1)).map_err(told)?;
                        }
                    }
                    std::fs::rename(path, previous(path)).map_err(told)?;
                    // its earlier saves are years the one set aside already holds
                    for n in 1..=KEPT {
                        let _ = std::fs::remove_file(earlier(path, n));
                    }
                }
                Ok(_) => {}
                // a file that cannot be read (a save cut short, or one written
                // by a later version of the game): it is set aside too, where
                // the years to come cannot push it out
                Err(_) if path.exists() => std::fs::rename(path, unread(path)).map_err(told)?,
                Err(_) => {}
            }
            self.settled = true;
        }
        if year_end && path.exists() {
            for n in (1..KEPT).rev() {
                let from = earlier(path, n);
                if from.exists() {
                    std::fs::rename(&from, earlier(path, n + 1)).map_err(told)?;
                }
            }
            std::fs::rename(path, earlier(path, 1)).map_err(told)?;
        }
        std::fs::rename(tmp, path).map_err(told)
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
            "--load" | "--seed" | "--families" | "--advance" => i += 1,
            "--new" => {}
            a => out.push(a.to_string()),
        }
        i += 1;
    }
    out.extend(["--seed".to_string(), seed.to_string(), "--families".to_string(), families.to_string(), "--new".to_string()]);
    out
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
            family: "Fairfax".into(),
            seat: "Ashby".into(),
            worth: 0,
            check: String::new(),
            orders: vec![Given { year: 1750, order: Order::MoveSeat { town: 1 } }],
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("isostate-saves-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
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
        // more new games: the last few set aside are all kept, the newest first, and no more than that
        for started in 3..=6 {
            Saver::new(path.clone()).write(&save(started, 1750), true).unwrap();
        }
        assert_eq!(files(&dir), ["game.json", "game.previous.2.json", "game.previous.3.json", "game.previous.json"]);
        assert_eq!(read(&path).unwrap().started, 6);
        assert_eq!(read(&previous(&path)).unwrap().started, 5);
        assert_eq!(read(&game_before(&path, 2)).unwrap().started, 4);
        assert_eq!(read(&game_before(&path, 3)).unwrap().started, 3);
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
    fn a_save_of_a_later_version_is_not_pushed_out_by_the_years() {
        let dir = scratch("later");
        let path = dir.join("game.json");
        std::fs::create_dir_all(&dir).unwrap();
        let later = save(1, 1790).to_text().replace("\"format\": 1,", "\"format\": 2,");
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
        assert!(may_keep(Path::new("")).unwrap_err().contains("needs the name"));
        assert!(may_keep(&folder.join("new.json")).is_ok());
        let mine = folder.join("mine.json");
        Saver::new(mine.clone()).write(&save(1, 1750), true).unwrap();
        assert!(may_keep(&mine).is_ok());
        // a write that fails part of the way leaves no half-written file: here
        // an earlier year cannot move up, its name being taken by a folder
        std::fs::write(earlier(&mine, 2), b"").unwrap();
        std::fs::create_dir_all(earlier(&mine, 3)).unwrap();
        std::fs::write(earlier(&mine, 3).join("x"), b"x").unwrap();
        assert!(Saver::new(mine.clone()).write(&save(1, 1751), true).is_err());
        assert_eq!(files(&folder), ["mine.2.json", "mine.3.json", "mine.json", "run.bin"]);
        assert_eq!(read(&mine).unwrap().year, 1750, "the latest is still the one before");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_new_game_starts_the_program_again_without_the_save() {
        // the save that was loaded is of island 11 with three families, whatever the arguments said
        let args: Vec<String> = ["--families", "4", "--load", "a b.json", "--size", "1280x800", "--advance", "5"].iter().map(|s| s.to_string()).collect();
        assert_eq!(new_game_args(&args, 11, 3), ["--size", "1280x800", "--seed", "11", "--families", "3", "--new"]);
        assert_eq!(new_game_args(&["--new".to_string()], 7, 6), ["--seed", "7", "--families", "6", "--new"]);
        // a place that was named goes on being the place
        let args: Vec<String> = ["--save", "mine.json", "--seed", "9"].iter().map(|s| s.to_string()).collect();
        assert_eq!(new_game_args(&args, 9, 6), ["--save", "mine.json", "--seed", "9", "--families", "6", "--new"]);
    }
}
