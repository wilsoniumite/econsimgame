# Marking a version

About once a week, when main holds a game worth trying, it is marked as a version (0.1.0, 0.2.0, ...) and put on GitHub's releases page, so that the game can be played on a Windows machine, a Mac or a Linux machine with no Rust and no repository. The Windows zip is made and published by hand: the managing session chooses the week's version, runs one command, reads what it printed, and publishes. The Mac and Linux archives follow by themselves, built on GitHub's machines once the version is published.

## The routine

1. On main, at origin's main, with nothing changed: raise `version` under `[workspace.package]` in `Cargo.toml` if the last version is already tagged (0.1.0 to 0.2.0; the third number is for a mended version of the same week). Commit and push that.
2. `python tools/release.py`. It refuses unless the checkout is clean, at origin's main and its version not yet tagged.
3. It builds the game (`cargo build --profile dist -p isostate-viewer`, with the machine's folders taken out of the program, in `target/dist`): 10.6 minutes from nothing, and about as long after any new commit, because most of it is the game's own code built in one piece and optimised whole.
4. It gathers `dist/isolated-state-VERSION-windows-x64.zip`: the program, a README.txt for a player, `docs/BACKLOG.md` and `docs/DEVLOG.md` for the backlog card and the What's new card (the only pages the game reads from disk; it looks for them in `docs\` beside the program), and the licences.
5. It looks for the home folder, the user name, and git's name and address in the program and in every file of the zip, and stops if it finds any.
6. It unpacks the zip into an empty folder in the system's temporary folder, away from the repository, and has the game take one picture there with nobody at the window (a sandbox five years in); it stops unless the picture is there and is not blank, and keeps it as `dist/isolated-state-VERSION-trial.png`. Look at it. A double click (a start with no arguments) opens the start page, which the trial, started with arguments, never meets: to see what a player sees first, double-click the unpacked program once.
7. It writes `dist/isolated-state-VERSION-notes.md` from the devlog's entries since the last version's tag, sorted into what is in the game and what is in the island's economy, under a few lines on which archive is for which machine. Read it through and shorten it: it is a first draft.
8. It prints the three commands that tag the version and publish it, and runs neither: `git tag -a`, `git push origin vVERSION`, `gh release create ... --prerelease`.
9. Run them. The zip is then on the version's page, the repository's address followed by `/releases/tag/vVERSION` (the command prints it whole), to download from anywhere. Give that page, not `releases/latest`: a pre-release is not what `latest` shows, and GitHub answers 404 there while every version is a pre-release.
10. Publishing starts the Mac and Linux builds (below). About an hour later, look at their pictures, and check that the two archives are on the version's page.
11. `python tools/release.py --practice` does steps 2 to 8 from wherever the checkout stands, its refusals turned into warnings, to try the command itself; it publishes nothing either, and what it prints is not to be run.

On 26 November 2026, on a machine shared with other builds, the whole command took 11.2 minutes from an empty build folder (the build 10.6, the trial 12 seconds), 16.3 minutes after a new commit with the other crates already built (the build 15.7, the machine busier), and 0.6 minutes run again on the same commit. The zip is 28 MiB, the program in it 87.5 MiB. The build folder, `target/dist`, takes 4 GB of the disk; it may be deleted between versions, at the price of the next build starting from nothing.

## The Mac and Linux builds

A release published on GitHub starts `.github/workflows/mac-linux.yml` ("Mac and Linux builds" under the repository's Actions). On GitHub's machines it checks out the version's tag and runs the same command for the two other platforms, `python tools/release.py --ci linux-x64` and `--ci macos`: the same `dist` build with the machine's folders taken out, the same search of the program and the archive (for the home folder only: those machines have no names of ours to find), and the same trial, the archive unpacked away from the checkout and one picture taken with nobody at the window. Neither notes nor commands: those are the Windows command's.

- **Linux**: built on Ubuntu 22.04, the oldest GitHub offers, so that the program asks for glibc 2.35 and no later. `isolated-state-VERSION-linux-x64.tar.gz` holds `isolated-state` (a .tar.gz, because a zip unpacked on Linux often loses the mode that lets the program start), the README, the docs and the licences. Its trial draws in software (Mesa's lavapipe), since the machine has no graphics card.
- **Mac**: the program is built twice side by side, for Apple silicon and for Intel, on two of GitHub's Apple silicon Macs, and joined into one (`lipo`). `isolated-state-VERSION-macos.zip` holds `Isolated State.app`, with the docs inside it in `Contents/Resources/docs`, and beside it the README and the licences. The app is signed "ad hoc", by nobody in particular: Apple silicon runs nothing that is not signed at all, and the signature seals what the app holds. Its trial takes the picture twice, with the Apple silicon program and with the Intel one under Rosetta, and checks that the unpacked app's signature holds.
- When both have passed, a last job adds the two archives to the release, refusing them if their names do not carry the tag's version. The pictures stay with the run: `gh run list --workflow mac-linux.yml` gives its number, `gh run download NUMBER --pattern '*' --dir dist/run-NUMBER` fetches them (with the archives), and they can then be looked at.
- If a job fails, the release keeps what it had, and nothing half-made is added. `gh run view NUMBER --log-failed` says why. After a mend that needs no new commit (a machine of GitHub's that broke), `gh run rerun NUMBER --failed` runs the failed jobs again; otherwise the mend is a commit, and the Mac and Linux builds come with the next version.
- `gh workflow run mac-linux.yml -f tag=vVERSION` builds a version's tag again and adds its archives, replacing any there. It builds the tag's own code with the tag's own `tools/release.py`, so it cannot build a version older than the workflow: 0.1.0 has no Mac or Linux build.
- `gh workflow run mac-linux.yml -f quick=true` tries the workflow on main and publishes nothing: built in the `work` profile, which is much sooner, and kept with the run. Without `quick` it is the full build of main, also kept with the run only.
- The workflow builds with the Rust the workstation has (`RUST_VERSION` at its head, 1.97.1): raise it there when the workstation's Rust is raised.
- The builds are free: GitHub's machines cost nothing for a public repository.

What has not been done: neither build has been played at a real Mac or a real Linux desktop by us, only started on GitHub's machines. The Mac's app is not notarised (Apple's check of a developer-signed app, which needs a developer account at $99 a year), so macOS asks the player once, as below. An island is not quite the same on every machine: the simulation's exponentials, logarithms and powers come from each system's own mathematics library, which may round a last digit differently, and the years carry such differences on. Island 7's sandbox five years in had 137,329 people from the Mac's app on Apple silicon and 137,333 from its Intel program under Rosetta (27 November 2026). A save carried from one machine to another is played again there, and its opening card says if the island now stands differently.

## What a player needs

- **Windows** 10 or 11, 64-bit, with graphics that do DirectX 12 or Vulkan: nearly every card or laptop made since about 2015. The game finds whichever there is; it has not been tried on weak graphics (an old laptop's integrated chip), where it may be slow.
- **Mac**: macOS 11 (Big Sur) or later, Apple silicon or Intel.
- **Linux**: 64-bit on an x86 processor, glibc 2.35 or later (Ubuntu 22.04, Debian 12, Fedora 36 or newer, SteamOS 3), X11 or Wayland, and a Vulkan driver (Mesa's for AMD and Intel, NVIDIA's own).
- Nothing installed: the program carries its content, look, font and shaders inside it. On Windows the zip is unpacked whole, anywhere, and `isolated-state.exe` double-clicked; a black window of the game's messages opens beside the game, and closing it closes the game. On a Mac the zip opens to a folder with the app in it. On Linux `./isolated-state` is started from its folder.

## The warnings

The programs are not signed by a developer the systems know (a certificate costs money every year and is not worth it for a game tried by a few people).

- **Windows**: the first time a new version is started, SmartScreen says "Windows protected your PC" and offers only "Don't run". Click **More info**, then **Run anyway**. Windows remembers that for this program; a new version asks again. A browser may also say the zip "is not commonly downloaded": keep it. If a virus scanner holds the program back, it is the same reason, and the program can be let through in the scanner's own settings.
- **Mac**: the first time the app is opened, macOS says it could not check it for malicious software, and offers only Done and Move to Trash. Click Done, open System Settings, Privacy & Security, and under the line saying Isolated State was blocked click **Open Anyway**, then **Open Anyway** again with the password. On macOS 14 or earlier, Control-click the app, choose Open, then Open. Should macOS say instead that the app "is damaged", that is the download's quarantine mark on an app it cannot check: `xattr -dr com.apple.quarantine "Isolated State.app"` in Terminal takes it off.
- **Linux** asks nothing.

## Saves and notes

The game keeps everything a player makes in the system's usual place for a program's own files: the island's autosaves, a sandbox's session, the games set aside, and in `notes/` each note typed with N, with its picture of the screen.

- Windows: `%APPDATA%\Isolated State\saves` (paste that into Explorer's address bar to open it).
- Mac: `~/Library/Application Support/Isolated State/saves` (the Finder's Go, Go to Folder).
- Linux: `~/.local/share/isolated-state/saves`, or `$XDG_DATA_HOME/isolated-state/saves` where that is set.

The folder belongs to the machine and the account, not to the game's folder, so:

- a new version unpacked beside an old one goes on with the same saves (a save made under older rules is played again under the new ones, and the opening card says if the island now stands differently);
- deleting the game's folder or the app deletes no save and no note;
- notes taken on another machine stay on that machine. To bring them home, copy that machine's notes folder (a USB stick, or a mail to yourself) into `%APPDATA%\Isolated State\saves\notes` at home; the managing session reads them from there at the start of a session. Notes are named by the day and their number in it, so two machines' notes of one day share names: copy those into a folder of their own inside `notes\` (`notes\away\`) rather than over the ones at home.

`ISOSTATE_SAVES=FOLDER`, set before the game is started, keeps everything in another folder instead (a folder in a cloud drive, say, which would carry the notes home by itself).
