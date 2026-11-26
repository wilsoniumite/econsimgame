# Marking a version

About once a week, when main holds a game worth trying, it is marked as a version (0.1.0, 0.2.0, ...) and put on GitHub's releases page as one zip, so that the game can be played on any Windows machine with no Rust and no repository. Nothing about it is automatic: the managing session chooses the week's version, runs one command, reads what it printed, and publishes by hand.

## The routine

1. On main, at origin's main, with nothing changed: raise `version` under `[workspace.package]` in `Cargo.toml` if the last version is already tagged (0.1.0 to 0.2.0; the third number is for a mended version of the same week). Commit and push that.
2. `python tools/release.py`. It refuses unless the checkout is clean, at origin's main and its version not yet tagged.
3. It builds the game (`cargo build --profile dist -p isostate-viewer`, with the machine's folders taken out of the program), about ten minutes from nothing and a minute or two after a small change.
4. It gathers `dist/isolated-state-VERSION-windows-x64.zip`: the program, a README.txt for a player, `docs/BACKLOG.md` and `docs/DEVLOG.md` for the backlog card and the What's new card (the only pages the game reads from disk; it looks for them in `docs\` beside the program), and the licences.
5. It looks for the home folder, the user name, and git's name and address in the program and in every file of the zip, and stops if it finds any.
6. It unpacks the zip into an empty folder in the system's temporary folder, away from the repository, and has the game take one picture there with nobody at the window (a sandbox five years in); it stops unless the picture is there and is not blank, and keeps it as `dist/isolated-state-VERSION-trial.png`. Look at it.
7. It writes `dist/isolated-state-VERSION-notes.md` from the devlog's entries since the last version's tag, sorted into what is in the game and what is in the island's economy. Read it through and shorten it: it is a first draft.
8. It prints the three commands that tag the version and publish it, and runs neither: `git tag -a`, `git push origin vVERSION`, `gh release create ... --prerelease`.
9. Run them. The zip is then at the repository's `releases/latest` page, to download from anywhere.
10. `python tools/release.py --practice` does all of it from wherever the checkout stands, its refusals turned into warnings, to try the command itself; it publishes nothing either, and what it prints is not to be run.

The whole command took 14 minutes from an empty build folder and 4 minutes with the build already made (26 November 2026, on a busy machine; the notes of the release kit's piece have the measurements).

## What a player needs

- Windows 10 or 11, 64-bit. There is no build for macOS or Linux yet.
- Graphics that do DirectX 12 or Vulkan: nearly every card or laptop made since about 2015. The game finds whichever there is; it has not been tried on weak graphics (an old laptop's integrated chip), where it may be slow.
- Nothing installed: the program carries its content, look, font and shaders inside it. The zip is unpacked whole, anywhere, and `isolated-state.exe` double-clicked. A black window of the game's messages opens beside the game; closing it closes the game.

## The warning Windows shows

The program is not signed (a signing certificate costs money every year and is not worth it for a game tried by a few people), so the first time a new version is started Windows SmartScreen says "Windows protected your PC" and offers only "Don't run". Click **More info**, then **Run anyway**. Windows remembers that for this program; a new version asks again. A browser may also say the zip "is not commonly downloaded": keep it. If a virus scanner holds the program back, it is the same reason, and the program can be let through in the scanner's own settings.

## Saves and notes

The game keeps everything a player makes under `%APPDATA%\Isolated State\saves` (paste that into Explorer's address bar to open it): the island's autosaves, a sandbox's session, the games set aside, and in `notes\` each note typed with N, with its picture of the screen. The folder belongs to the machine and the Windows account, not to the game's folder, so:

- a new version unpacked beside an old one goes on with the same saves (a save made under older rules is played again under the new ones, and the opening card says if the island now stands differently);
- deleting the game's folder deletes no save and no note;
- notes taken on another machine stay on that machine. To bring them home, copy that machine's `%APPDATA%\Isolated State\saves\notes` folder (a USB stick, or a mail to yourself) into the same folder at home; the managing session reads them from there at the start of a session. Notes are named by the day and their number in it, so two machines' notes of one day share names: copy those into a folder of their own inside `notes\` (`notes\away\`) rather than over the ones at home.

`ISOSTATE_SAVES=FOLDER`, set before the game is started, keeps everything in another folder instead (a folder in a cloud drive, say, which would carry the notes home by itself).
