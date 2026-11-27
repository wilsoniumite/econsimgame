"""Make a marked version of the game: an archive a player can unpack and run.

    python tools/release.py                 the Windows version Cargo.toml says, from origin's main
    python tools/release.py --practice      the same from wherever the checkout stands, to try the
                                            command itself: its refusals become warnings
    python tools/release.py --ci linux-x64  on GitHub's machines (.github/workflows/mac-linux.yml):
    python tools/release.py --ci macos      the Linux or the Mac build of the commit checked out

The Windows version is made on the workstation. It refuses unless the checkout is clean, at
origin's main, and its version not tagged yet. Then it builds the game in release with no
debugging information and with the machine's folders taken out of the program (the `dist`
profile, in target/dist), gathers the program and what it needs into
dist/isolated-state-VERSION-windows-x64.zip, unpacks the zip into an empty folder away from the
repository and has it take one picture there with nobody at the window, looks for the builder's
home folder and user name in the program and the zip, writes release notes from the devlog, and
prints the commands that would tag and publish the version. It runs neither: tagging and
publishing are done by hand (docs/RELEASING.md).

The Mac and Linux builds are made on GitHub's machines by a workflow, once the version is
published, with --ci: the same build, archive, search and trial for that platform, of the commit
checked out (which must have nothing changed), and neither notes nor commands. Linux's archive
is a .tar.gz with the program in it; the Mac's is a .zip holding Isolated State.app, whose
program is built twice, for Apple silicon and for Intel, and joined into one. --build-only
TARGET builds one of those two and stops, and --slices DIR makes the app from two built so,
which lets the workflow build them side by side. --quick builds in the `work` profile instead,
much sooner, to try the workflow itself: such a build is never published.

Python 3.11 or later, git and cargo; for the Mac's app also lipo, codesign and ditto, which come
with a Mac and Xcode's command line tools.
"""

import argparse
import datetime
import getpass
import hashlib
import io
import json
import os
import plistlib
import re
import shutil
import struct
import subprocess
import sys
import tarfile
import tempfile
import time
import tomllib
import zipfile
import zlib

WINDOWS, LINUX, MAC = "windows-x64", "linux-x64", "macos"
# the program's name in each archive
PROGRAM = {WINDOWS: "isolated-state.exe", LINUX: "isolated-state", MAC: "isolated-state"}
# the Rust targets each is built for: the Windows version is built for the workstation itself, in
# target/dist as it always has been; the others with --target, in target/TARGET/dist
TRIPLES = {WINDOWS: ["x86_64-pc-windows-msvc"], LINUX: ["x86_64-unknown-linux-gnu"], MAC: ["aarch64-apple-darwin", "x86_64-apple-darwin"]}
APP = "Isolated State.app"
# macOS 11 (Big Sur), the first for Apple silicon; the Intel program is held to it as well
MAC_MINIMUM = "11.0"

# What goes into the archive beside the program: (the file in the repository, its place in the
# archive). The program carries its content, look, font and shaders inside it. The pages the game
# reads from disk as it runs are docs/BACKLOG.md, for the backlog card opened from the help card,
# and docs/DEVLOG.md, for the What's new card: crates/isostate-viewer/src/backlog_seen.rs looks
# for them in docs/ beside the program, and on a Mac in the app's Contents/Resources/docs.
DOCS = [
    ("docs/BACKLOG.md", "docs/BACKLOG.md"),
    ("docs/DEVLOG.md", "docs/DEVLOG.md"),
]
LICENCES = [
    ("LICENSE-MIT", "licences/LICENSE-MIT.txt"),
    ("LICENSE-APACHE", "licences/LICENSE-APACHE.txt"),
    ("crates/isostate-viewer/assets/fonts/OFL-Fredoka.txt", "licences/OFL-Fredoka.txt"),
]
# a picture drawn in software (Linux on GitHub's machines, which have no graphics card) took 7 minutes
TRIAL_SECONDS = 1800


def trial_args(picture):
    return ["--sandbox", "--advance", "5", "--shot", picture, "--size", "1600x1000"]


def say(line=""):
    print(line, flush=True)


def run(args, cwd=None, env=None, check=True):
    out = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if check and out.returncode != 0:
        raise SystemExit(f"failed: {' '.join(args)}\n{out.stdout}{out.stderr}")
    return out


def git(*args, check=True):
    return run(["git", *args], check=check).stdout.strip()


class Refusals:
    """A refusal stops a release, and is only a warning in a practice."""

    def __init__(self, practice):
        self.practice = practice
        self.waved = []

    def __call__(self, why):
        if self.practice:
            say(f"  (a practice, so only a warning) {why}")
            self.waved.append(why)
        else:
            raise SystemExit(f"refused: {why}\n(python tools/release.py --practice tries the command anyway, and publishes nothing)")


# ---------------------------------------------------------------- the checks before

def check_checkout(version, refuse):
    changed = git("status", "--porcelain", "--untracked-files=no")
    if changed:
        refuse("the checkout has changes not committed:\n" + changed)
    # files git does not know cannot reach the program (it is built from what the sources name),
    # so they are only listed
    loose = git("status", "--porcelain", "--untracked-files=normal")
    loose = [l for l in loose.splitlines() if l.startswith("??")]
    if loose:
        say(f"  {len(loose)} files git does not know are in the checkout; they cannot reach the program")
    # a practice asks nothing of origin and changes no ref: it goes by where origin's main was last seen
    if not refuse.practice:
        fetched = run(["git", "fetch", "--quiet", "--tags", "origin", "main"], check=False)
        if fetched.returncode != 0:
            refuse("origin could not be asked where main stands: " + fetched.stderr.strip())
    head = git("rev-parse", "HEAD")
    main = git("rev-parse", "--verify", "--quiet", "origin/main", check=False)
    if head != main:
        refuse(f"the checkout is at {head[:12]}, and origin's main at {main[:12] or 'nowhere known'}: a version is made from origin's main")
    if git("tag", "--list", f"v{version}"):
        refuse(f"v{version} is tagged already: raise the version in Cargo.toml (workspace.package), commit and push it, then make the version")


def check_ci_checkout():
    """On GitHub's machines the workflow checks out the version's tag: nothing in it may have
    changed, or the program would say so of itself (build.rs) and not be the version's."""
    changed = git("status", "--porcelain", "--untracked-files=no")
    if changed:
        raise SystemExit("refused: the checkout has changes not committed:\n" + changed)


# ---------------------------------------------------------------- the build

def remapped_flags(root):
    """The machine's folders as rustc should write them in the program: the home folder as ~,
    cargo's and rustup's folders by their names, the repository as isolated-state. When two
    match, the one given last wins, so the home folder comes first."""
    home = os.path.expanduser("~")
    cargo = os.environ.get("CARGO_HOME") or os.path.join(home, ".cargo")
    rustup = os.environ.get("RUSTUP_HOME") or os.path.join(home, ".rustup")
    pairs = [(home, "~"), (cargo, "cargo"), (rustup, "rustup"), (root, "isolated-state")]
    return [f"--remap-path-prefix={os.path.normpath(p)}={to}" for p, to in pairs]


def build(root, profile, triple=None):
    """The game built in `profile`, for `triple` (None: for this machine, as the Windows version
    is), with the machine's folders taken out of the program: its path and the seconds it took."""
    env = dict(os.environ)
    env.pop("RUSTFLAGS", None)
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(remapped_flags(root))
    args = ["cargo", "build", "--locked", "--profile", profile, "-p", "isostate-viewer"]
    folder = os.path.join(root, "target", profile)
    if triple:
        args += ["--target", triple]
        folder = os.path.join(root, "target", triple, profile)
        if triple.endswith("-apple-darwin"):
            env["MACOSX_DEPLOYMENT_TARGET"] = MAC_MINIMUM
    t = time.monotonic()
    out = subprocess.run(args, cwd=root, env=env)
    if out.returncode != 0:
        raise SystemExit("the build failed")
    windows = "windows" in triple if triple else os.name == "nt"
    return os.path.join(folder, "isostate-viewer" + (".exe" if windows else "")), time.monotonic() - t


def join_slices(slices, out):
    """One program for both kinds of Mac, from the two built for each (lipo)."""
    os.makedirs(os.path.dirname(out), exist_ok=True)
    run(["lipo", "-create", "-output", out, *slices])
    archs = sorted(run(["lipo", "-archs", out]).stdout.split())
    if archs != ["arm64", "x86_64"]:
        raise SystemExit(f"the Mac's program holds {archs}, where arm64 and x86_64 were looked for")
    # a program carried between the workflow's machines comes without its mode
    os.chmod(out, 0o755)
    return out


# ---------------------------------------------------------------- the archive

def third_party(root, triples, newline):
    """Every crate the program is built from, with its version and licence, from cargo's own
    account of the build (normal dependencies, for the platform's targets)."""
    rows = set()
    for triple in triples:
        meta = json.loads(run(["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", triple], cwd=root).stdout)
        packages = {p["id"]: p for p in meta["packages"]}
        nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
        start = next(p["id"] for p in meta["packages"] if p["name"] == "isostate-viewer")
        seen, todo = set(), [start]
        while todo:
            n = todo.pop()
            if n in seen:
                continue
            seen.add(n)
            for d in nodes[n]["deps"]:
                if any(k["kind"] is None for k in d["dep_kinds"]):
                    todo.append(d["pkg"])
        rows |= {(packages[i]["name"], packages[i]["version"], packages[i].get("license") or "see its source", packages[i].get("repository") or "") for i in seen if packages[i]["source"]}
    lines = [
        "The game is built with these crates, each under the licence it names (most under MIT or",
        "Apache-2.0, whose texts are in this folder). Each crate's copyright notices are in its source,",
        "at the address given or on crates.io.",
        "",
    ]
    lines += [f"{n} {v}  {lic}  {repo}".rstrip() for n, v, lic, repo in sorted(rows)]
    return newline.join(lines) + newline, len(rows)


def player_readme(version, commit, day, platform):
    if platform == WINDOWS:
        return windows_readme(version, commit, day)
    program = "./isolated-state" if platform == LINUX else "Isolated State.app/Contents/MacOS/isolated-state"
    head = f"""Isolated State {version}
(built on {day} from commit {commit})

An island's economy from 1750, played as a small diorama: its towns, works, roads and railways
grow as its people trade, and you watch it, change it by hand, or play one of its families.

"""
    start_page = f"""  It opens on a start page: go on with a saved game, or begin a new island, as a sandbox
  (nobody plays a family, and the tray under the clock changes the island by hand) or as the
  family game. Its foot shows the command line that does the same, and {program} --help
  lists everything else a command line can ask for.

"""
    licences = """  licences/               the game's licences (MIT or Apache-2.0, at your choice), the
                          font's (SIL Open Font Licence), and the crates it is built with
"""
    if platform == LINUX:
        return head + """TO PLAY
  Unpack the whole folder anywhere you like and start isolated-state in it: in a terminal, cd
  into the folder and type ./isolated-state, which shows the game's messages too; or
  double-click it, where the file manager runs programs. Press H in the game for every key.
  The first start makes the island, which takes a few seconds.

""" + start_page + """WHAT IT NEEDS
  A 64-bit Linux on an x86 processor with glibc 2.35 or later (Ubuntu 22.04, Debian 12, Fedora
  36 or newer, SteamOS 3), a desktop on X11 or Wayland, and graphics with a Vulkan driver: Mesa's
  for AMD and Intel, NVIDIA's own for NVIDIA. Nothing else is installed: the program carries
  what it draws with.

WHERE YOUR GAMES ARE KEPT
  The game saves itself into ~/.local/share/isolated-state/saves (or into
  $XDG_DATA_HOME/isolated-state/saves, where that is set). A note typed with N is kept there
  too, in the notes folder, with a picture of the screen. Saves and notes stay on the machine
  they were made on: to bring notes home, copy that notes folder. Deleting this game's folder
  leaves them where they are.

  The cards opened from the help card (H) read the docs folder here: the backlog, the plan of
  work as it stood when this version was made, and the devlog, what was done day by day.

WHAT IS IN THE FOLDER
  isolated-state          the game
  docs/BACKLOG.md         the plan of work, for the backlog card
  docs/DEVLOG.md          what was done, newest first, for the What's new card
""" + licences
    return head + """TO PLAY
  Open the zip (Safari opens it by itself) and the folder in it. Drag Isolated State to
  Applications if you like, or leave it where it is, and double-click it. Press H in the game
  for every key. The first start makes the island, which takes a few seconds.

""" + start_page + """WHAT IT NEEDS
  A Mac with macOS 11 (Big Sur) or later, with Apple silicon or Intel: the app holds a program
  for each. Nothing else is installed: the app carries what it draws with.

WHEN YOUR MAC WARNS YOU
  The app is not signed by a developer Apple knows (that takes a fee every year), so the first
  time it is opened macOS says it could not check it, and offers only Done and Move to Trash.
  Click Done. Then open System Settings, Privacy & Security, scroll down to where it says that
  Isolated State was blocked, and click Open Anyway, then Open Anyway again with your password.
  It asks once for each new version. On macOS 14 or earlier, Control-click the app, choose
  Open, then Open.

  If macOS says instead that the app is damaged and should be moved to the Trash, take the
  download's mark off it: in Terminal, type  xattr -dr com.apple.quarantine  and a space, drag
  the app onto the Terminal window, press Return, and open the app again.

WHERE YOUR GAMES ARE KEPT
  The game saves itself into ~/Library/Application Support/Isolated State/saves (in the
  Finder: Go, Go to Folder, and paste that). A note typed with N is kept there too, in the
  notes folder, with a picture of the screen. Saves and notes stay on the machine they were
  made on: to bring notes home, copy that notes folder. Deleting the app leaves them where
  they are.

  The cards opened from the help card (H) read the docs inside the app: the backlog, the plan
  of work as it stood when this version was made, and the devlog, what was done day by day.

WHAT IS IN THE FOLDER
  Isolated State.app      the game, with the backlog and the devlog inside it (in
                          Contents/Resources/docs: Control-click the app, Show Package Contents)
""" + licences


def windows_readme(version, commit, day):
    program = PROGRAM[WINDOWS]
    text = f"""Isolated State {version}
(built on {day} from commit {commit})

An island's economy from 1750, played as a small diorama: its towns, works, roads and railways
grow as its people trade, and you watch it, change it by hand, or play one of its families.

TO PLAY
  Unpack the whole folder anywhere you like (the Desktop, Documents) and double-click
  {program}. A black window of the game's messages opens beside it: leave it open, since closing
  it closes the game. Press H in the game for every key. The first start makes the island,
  which takes a few seconds.

  It opens on a start page: go on with a saved game, or begin a new island, as a sandbox
  (nobody plays a family, and the tray under the clock changes the island by hand) or as the
  family game. Its foot shows the command line that does the same, and {program} --help
  lists everything else a command line can ask for.

WHAT IT NEEDS
  Windows 10 or 11, 64-bit, and graphics that do DirectX 12 or Vulkan (nearly every card or
  laptop since about 2015). Nothing else is installed: the program carries what it draws with.

WHEN WINDOWS WARNS YOU
  The program is not signed, so the first time it starts Windows may say "Windows protected your
  PC". Click "More info", then "Run anyway". It asks once for each new version.

WHERE YOUR GAMES ARE KEPT
  The game saves itself into %APPDATA%\\Isolated State\\saves (paste that into Explorer's
  address bar to open it). A note typed with N is kept there too, in the notes folder, with a
  picture of the screen. Saves and notes stay on the machine they were made on: to bring notes
  home, copy that notes folder. Deleting this game's folder leaves them where they are.

  The cards opened from the help card (H) read the docs folder here: the backlog, the plan of
  work as it stood when this version was made, and the devlog, what was done day by day.

WHAT IS IN THE FOLDER
  {program}               the game
  docs\\BACKLOG.md         the plan of work, for the backlog card
  docs\\DEVLOG.md          what was done, newest first, for the What's new card
  licences\\               the game's licences (MIT or Apache-2.0, at your choice), the
                          font's (SIL Open Font Licence), and the crates it is built with
"""
    return text.replace("\n", "\r\n")


def archive_name(version, platform):
    return f"isolated-state-{version}-{platform}" + (".tar.gz" if platform == LINUX else ".zip")


def pack_windows(root, exe, version, commit, day, dist):
    path = os.path.join(dist, archive_name(version, WINDOWS))
    top = f"isolated-state-{version}/"
    credits, crates = third_party(root, TRIPLES[WINDOWS], "\r\n")
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        z.write(exe, top + PROGRAM[WINDOWS])
        z.writestr(top + "README.txt", player_readme(version, commit, day, WINDOWS))
        for src, at in DOCS + LICENCES:
            z.write(os.path.join(root, src), top + at)
        z.writestr(top + "licences/crates.txt", credits)
    return path, top, crates


def pack_linux(root, exe, version, commit, day, dist):
    """A .tar.gz, which keeps the program's mode: a zip unpacked on Linux often loses it, and the
    program then will not start."""
    path = os.path.join(dist, archive_name(version, LINUX))
    top = f"isolated-state-{version}"
    credits, crates = third_party(root, TRIPLES[LINUX], "\n")
    now = int(time.time())

    def folder(tar, at):
        info = tarfile.TarInfo(at)
        info.type, info.mode, info.mtime = tarfile.DIRTYPE, 0o755, now
        tar.addfile(info)

    def file(tar, at, data, mode=0o644):
        info = tarfile.TarInfo(at)
        info.size, info.mode, info.mtime = len(data), mode, now
        tar.addfile(info, io.BytesIO(data))

    with tarfile.open(path, "w:gz", compresslevel=9) as tar:
        for at in (top, f"{top}/docs", f"{top}/licences"):
            folder(tar, at)
        file(tar, f"{top}/{PROGRAM[LINUX]}", open(exe, "rb").read(), 0o755)
        file(tar, f"{top}/README.txt", player_readme(version, commit, day, LINUX).encode())
        for src, at in DOCS + LICENCES:
            file(tar, f"{top}/{at}", open(os.path.join(root, src), "rb").read())
        file(tar, f"{top}/licences/crates.txt", credits.encode())
    return path, top + "/", crates


def github_owner():
    """The repository's owner on GitHub: GitHub's machines say it, and elsewhere origin's address does."""
    owner = os.environ.get("GITHUB_REPOSITORY_OWNER")
    if not owner:
        m = re.search(r"github\.com[:/]([^/]+)/", git("remote", "get-url", "origin", check=False))
        owner = m.group(1) if m else None
    return owner


def app_plist(version):
    # the app's name for macOS, in the reverse of an address its makers hold: the repository's on GitHub
    owner = github_owner()
    ident = f"io.github.{owner.lower()}.isolated-state" if owner else "io.github.isolated-state"
    return {
        "CFBundleDevelopmentRegion": "en",
        "CFBundleDisplayName": "Isolated State",
        "CFBundleExecutable": PROGRAM[MAC],
        "CFBundleIdentifier": ident,
        "CFBundleInfoDictionaryVersion": "6.0",
        "CFBundleName": "Isolated State",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version,
        "CFBundleVersion": version,
        "LSApplicationCategoryType": "public.app-category.simulation-games",
        "LSMinimumSystemVersion": MAC_MINIMUM,
        "NSHighResolutionCapable": True,
        "NSHumanReadableCopyright": "The Isolated State authors. MIT or Apache-2.0, at your choice.",
    }


def pack_mac(root, exe, version, commit, day, dist):
    """Isolated State.app, in a folder with the README and the licences, zipped as a Mac zips
    (ditto). The app is signed "ad hoc", by nobody in particular: Apple silicon runs no program
    that is not signed at all, and the signature seals what the app holds. It is not a
    developer's signature, so macOS still asks the player once (README.txt says how)."""
    path = os.path.join(dist, archive_name(version, MAC))
    top = f"isolated-state-{version}"
    credits, crates = third_party(root, TRIPLES[MAC], "\n")
    stage = tempfile.mkdtemp(prefix="isolated-state-pack-")
    folder = os.path.join(stage, top)
    app = os.path.join(folder, APP)
    contents = os.path.join(app, "Contents")
    os.makedirs(os.path.join(contents, "MacOS"))
    program = os.path.join(contents, "MacOS", PROGRAM[MAC])
    shutil.copyfile(exe, program)
    os.chmod(program, 0o755)
    for src, at in DOCS:
        to = os.path.join(contents, "Resources", at)
        os.makedirs(os.path.dirname(to), exist_ok=True)
        shutil.copyfile(os.path.join(root, src), to)
    with open(os.path.join(contents, "Info.plist"), "wb") as f:
        plistlib.dump(app_plist(version), f)
    run(["codesign", "--force", "--sign", "-", app])
    run(["codesign", "--verify", "--strict", "--verbose=2", app])
    with open(os.path.join(folder, "README.txt"), "w", encoding="utf-8", newline="\n") as f:
        f.write(player_readme(version, commit, day, MAC))
    for src, at in LICENCES:
        to = os.path.join(folder, at)
        os.makedirs(os.path.dirname(to), exist_ok=True)
        shutil.copyfile(os.path.join(root, src), to)
    with open(os.path.join(folder, "licences", "crates.txt"), "w", encoding="utf-8", newline="\n") as f:
        f.write(credits)
    if os.path.exists(path):
        os.remove(path)
    run(["ditto", "-c", "-k", "--norsrc", "--noextattr", "--noacl", "--keepParent", folder, path])
    shutil.rmtree(stage, ignore_errors=True)
    return path, top + "/", crates


PACK = {WINDOWS: pack_windows, LINUX: pack_linux, MAC: pack_mac}


def members(path):
    """(name, bytes) of every file in the archive."""
    if path.endswith(".zip"):
        with zipfile.ZipFile(path) as z:
            for info in z.infolist():
                yield info.filename, z.read(info)
    else:
        with tarfile.open(path) as t:
            for m in t.getmembers():
                yield m.name, t.extractfile(m).read() if m.isfile() else b""


# ---------------------------------------------------------------- what the program must not carry

def views(data):
    """The bytes as text three ways: as they are, and every other byte from each of the first
    two, which reads text kept as UTF-16 (as Windows keeps paths) as plain letters."""
    return [data.decode("latin-1"), data[0::2].decode("latin-1"), data[1::2].decode("latin-1")]


def private_patterns(ci):
    """The home folder (with either slash, and doubled backslashes as code writes them); and,
    on the workstation, the user name as a word of its own and git's name and address for this
    checkout, if set. GitHub's machines have no such names (their user is `runner`, a word the
    engine's own code is full of), so there only the home folder is looked for."""
    home = os.path.normpath(os.path.expanduser("~"))
    parts = [re.escape(p) for p in re.split(r"[\\/]", home) if p]
    found = [("the home folder", re.compile(r"[\\/]{1,2}".join(parts), re.I))]
    if ci:
        return found
    user = getpass.getuser()
    if user:
        found.append(("the user name", re.compile(rf"(?<![A-Za-z0-9]){re.escape(user)}(?![A-Za-z0-9])", re.I)))
    for key, what in [("user.name", "git's name"), ("user.email", "git's address")]:
        v = git("config", "--get", key, check=False)
        if v:
            found.append((what, re.compile(re.escape(v), re.I)))
    return found


def search(label, data, patterns):
    hits = []
    for text in views(data):
        for what, pat in patterns:
            for m in pat.finditer(text):
                a, b = max(0, m.start() - 30), min(len(text), m.end() + 30)
                hits.append(f"{label}: {what}, in ...{text[a:b]!r}...")
                break
    return hits


def search_all(exe, path, patterns):
    hits = search(os.path.basename(exe), open(exe, "rb").read(), patterns)
    hits += search(os.path.basename(path), open(path, "rb").read(), patterns)
    for name, data in members(path):
        hits += search(name, name.encode() + data, patterns)
    return hits


# ---------------------------------------------------------------- the trial

def png_pixels(path):
    """Width, height and the rows of an 8-bit PNG, read with nothing but zlib."""
    data = open(path, "rb").read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG")
    pos, idat, w = 8, b"", 0
    while pos < len(data):
        n, kind = struct.unpack(">I4s", data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + n]
        if kind == b"IHDR":
            w, h, depth, colour, _, _, interlace = struct.unpack(">IIBBBBB", body)
            if depth != 8 or interlace or colour not in (2, 6):
                raise ValueError(f"a PNG this check does not read (depth {depth}, colour {colour}, interlace {interlace})")
            bpp = 3 if colour == 2 else 4
        elif kind == b"IDAT":
            idat += body
        pos += 12 + n
    raw, stride, rows, prev = zlib.decompress(idat), w * bpp, [], bytearray(w * bpp)
    for y in range(h):
        f, line = raw[y * (stride + 1)], bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b, c = prev[i], prev[i - bpp] if i >= bpp else 0
            if f == 1:
                line[i] = (line[i] + a) & 255
            elif f == 2:
                line[i] = (line[i] + b) & 255
            elif f == 3:
                line[i] = (line[i] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(bytes(line))
        prev = line
    return w, h, bpp, rows


def not_blank(path):
    """A picture is not blank when its colours are many and no one of them covers most of it."""
    w, h, bpp, rows = png_pixels(path)
    counts = {}
    for y in range(0, h, 4):
        row = rows[y]
        for x in range(0, w, 4):
            px = row[x * bpp:x * bpp + 3]
            counts[px] = counts.get(px, 0) + 1
    total = sum(counts.values())
    commonest = max(counts.values()) / total
    return len(counts) >= 200 and commonest < 0.5, f"{w} by {h} pixels, {len(counts)} colours among {total} sampled, the commonest on {commonest:.0%} of them"


def reaches_repository(folder):
    """Whether the game started in `folder` could find the repository: a folder above it that
    holds the repository's Cargo.toml, .git or docs/BACKLOG.md."""
    d = os.path.dirname(folder)
    while True:
        if any(os.path.exists(os.path.join(d, x)) for x in ("Cargo.toml", ".git", os.path.join("docs", "BACKLOG.md"))):
            return d
        up = os.path.dirname(d)
        if up == d:
            return None
        d = up


def unpack(path, where, platform):
    """The archive unpacked as a player's machine would: a Mac's zip by ditto, as the Finder does."""
    if platform == MAC:
        run(["ditto", "-x", "-k", path, where])
    elif path.endswith(".zip"):
        with zipfile.ZipFile(path) as z:
            z.extractall(where)
    else:
        with tarfile.open(path) as t:
            t.extractall(where, filter="data")


def rosetta():
    """Whether this Mac can run an Intel program (Apple silicon with Rosetta, or an Intel Mac)."""
    return sys.platform == "darwin" and subprocess.run(["arch", "-x86_64", "/usr/bin/true"], capture_output=True).returncode == 0


def shoot(command, game, env, picture, log):
    """One picture taken by the game in `game` with nobody at the window: its path, how it
    looks, and the seconds it took."""
    t = time.monotonic()
    try:
        out = subprocess.run([*command, *trial_args(picture)], cwd=game, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=TRIAL_SECONDS)
    except subprocess.TimeoutExpired:
        raise SystemExit(f"the trial took more than {TRIAL_SECONDS} s; its folder is {game}")
    took = time.monotonic() - t
    at = os.path.join(game, picture)
    with open(log, "w", encoding="utf-8") as f:
        f.write(out.stdout + out.stderr)
    if out.returncode != 0 or not os.path.isfile(at):
        raise SystemExit(f"the trial ended with code {out.returncode} and {'a' if os.path.isfile(at) else 'no'} picture; what it said is in {log}:\n{(out.stdout + out.stderr)[-3000:]}")
    ok, how = not_blank(at)
    if not ok:
        raise SystemExit(f"the trial's picture looks blank ({how}): {at}")
    return at, how, took


def trial(path, top, version, commit, root, platform):
    """Unpack the archive into an empty folder away from the repository, and have the game take
    one picture there with nobody at the window; on a Mac that can, the Intel program a second.
    The pictures as (what took it, its path, how it looks, the seconds it took), and the folder."""
    where = tempfile.mkdtemp(prefix="isolated-state-trial-")
    near = reaches_repository(where)
    if near or os.path.normcase(where).startswith(os.path.normcase(root)):
        raise SystemExit(f"the trial's folder {where} is within reach of a repository at {near or root}: set TEMP to a folder outside it")
    unpack(path, where, platform)
    game = os.path.join(where, top.rstrip("/"))
    if platform == MAC:
        app = os.path.join(game, APP)
        run(["codesign", "--verify", "--strict", "--verbose=2", app])
        say("  the unpacked app's signature holds")
        exe = os.path.join(app, "Contents", "MacOS", PROGRAM[MAC])
    else:
        exe = os.path.join(game, PROGRAM[platform])
    # nothing of the build or of the repository in its surroundings, and its own saves' folder
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith(("CARGO", "RUSTUP", "RUSTFLAGS", "BEVY_", "ISOSTATE_", "WGPU_"))}
    env["PATH"] = os.pathsep.join(p for p in env.get("PATH", "").split(os.pathsep) if p and not os.path.normcase(os.path.abspath(p)).startswith(os.path.normcase(root)))
    env["ISOSTATE_SAVES"] = os.path.join(where, "saves")
    takers = [("the program", [exe], "trial.png")]
    if platform == MAC:
        if rosetta():
            takers.append(("the Intel program, under Rosetta", ["arch", "-x86_64", exe], "trial-intel.png"))
        else:
            say("  this Mac runs no Intel program (no Rosetta): the app's Intel program is not started")
    pictures = []
    for what, command, picture in takers:
        said = run([*command, "--version"], cwd=game, env=env).stdout.strip()
        if version not in said or commit not in said:
            raise SystemExit(f"{what} unpacked says {said!r}, where {version} and {commit} were looked for")
        say(f"  {what} unpacked says: {said}")
        at, how, took = shoot(command, game, env, picture, os.path.join(where, picture + ".log"))
        pictures.append((what, at, how, took))
    return pictures, where


# ---------------------------------------------------------------- release notes

ENTRY = re.compile(r"^\*\*(.+?)\*\*\s*(.*)$")
DAY = re.compile(r"^## (\d{4}-\d{2}-\d{2})")
# words that say an entry is something a player sees or does, and the sentence that says where
IN_THE_GAME = re.compile(r"\b(in the viewer|the viewer|tab|panel|card|chip|map views?|button|tray|toast|the help card|key|press|click|inspector|the bar|on screen)\b", re.I)


def devlog_entries(text):
    """The devlog's entries, newest first: (day, title, the rest of the paragraph)."""
    day, found = None, []
    for line in text.splitlines():
        m = DAY.match(line)
        if m:
            day = m.group(1)
            continue
        m = ENTRY.match(line.strip())
        if m and day:
            title = m.group(1).strip().rstrip(".:")
            found.append((day, title, m.group(2)))
    return found


def first_sentence_with(pattern, text):
    for s in re.split(r"(?<=[.!?])\s+", text):
        if pattern.search(s):
            return s.strip()
    return None


def release_notes(version, commit, day):
    now = devlog_entries(open(os.path.join("docs", "DEVLOG.md"), encoding="utf-8").read())
    last = git("describe", "--tags", "--abbrev=0", "--match", "v*", "HEAD", check=False)
    if last:
        before = git("show", f"{last}:docs/DEVLOG.md", check=False)
        known = {t for _, t, _ in devlog_entries(before)}
        new = [e for e in now if e[1] not in known]
        since = f"since {last}"
    else:
        # the first marked version: the devlog's last seven days
        newest = datetime.date.fromisoformat(now[0][0]) if now else datetime.date.today()
        new = [e for e in now if (newest - datetime.date.fromisoformat(e[0])).days < 7]
        since = f"in the devlog's last seven days, to {newest.isoformat()} (the first marked version)"
    seen, game, model = set(), [], []
    for d, title, rest in new:
        if title in seen:
            continue
        seen.add(title)
        where = first_sentence_with(IN_THE_GAME, rest)
        if where or IN_THE_GAME.search(title):
            game.append((title, where or ""))
        else:
            model.append((title, None))
    lines = [
        f"# Isolated State {version}",
        "",
        f"Built from commit {commit} on {day}. Unpack the archive for your machine anywhere and start the game in it; README.txt inside says more.",
        "",
        f"- **Windows** 10 or 11, 64-bit: `{archive_name(version, WINDOWS)}`, and `{PROGRAM[WINDOWS]}` in it.",
        f"- **Mac**, macOS 11 or later, Apple silicon or Intel: `{archive_name(version, MAC)}`, and Isolated State.app in it. The first time, macOS blocks it: System Settings, Privacy & Security, Open Anyway.",
        f"- **Linux**, x64, glibc 2.35 or later, with Vulkan: `{archive_name(version, LINUX)}`, and `{PROGRAM[LINUX]}` in it.",
        "",
        "The Mac and Linux archives are built on GitHub's machines once this page is published, and appear here about an hour later.",
        "",
        f"What is new {since}, from the devlog's headings. Written by tools/release.py: read it through and shorten it before publishing.",
        "",
    ]
    if game:
        lines += ["## In the game", ""]
        lines += [f"- **{t}.** {w}".rstrip() for t, w in game]
        lines.append("")
    if model:
        lines += ["## In the island's economy", ""]
        lines += [f"- {t}." for t, _ in model]
        lines.append("")
    return "\n".join(lines), len(game) + len(model), since


# ---------------------------------------------------------------- the command

def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--practice", action="store_true", help="try the whole Windows command from wherever the checkout stands; its refusals become warnings")
    ap.add_argument("--keep-trial", action="store_true", help="leave the trial's folder where it is, to look at")
    ap.add_argument("--ci", choices=[LINUX, MAC], help="on GitHub's machines: the Linux or the Mac build of the commit checked out, with no notes and no commands (.github/workflows/mac-linux.yml)")
    ap.add_argument("--build-only", metavar="TARGET", choices=TRIPLES[MAC], help="with --ci macos: build the Mac's program for one kind of processor into dist/slices/TARGET/, and stop")
    ap.add_argument("--slices", metavar="DIR", help="with --ci macos: make the app from the two programs in DIR/TARGET/, built with --build-only")
    ap.add_argument("--quick", action="store_true", help="with --ci: build in the work profile, much sooner, to try the workflow; such a build is never published")
    a = ap.parse_args()
    if (a.build_only or a.slices) and a.ci != MAC:
        ap.error("--build-only and --slices go with --ci macos")
    if a.quick and not a.ci:
        ap.error("--quick goes with --ci: the Windows version is always built in the dist profile")
    if a.practice and a.ci:
        ap.error("--practice is for the Windows command; a run of the workflow by hand is the practice for --ci")
    if not a.ci and os.name != "nt":
        raise SystemExit("the Windows version is made on Windows; elsewhere this is --ci linux-x64 or --ci macos, for GitHub's machines (docs/RELEASING.md)")
    platform = a.ci or WINDOWS
    profile = "work" if a.quick else "dist"
    t0 = time.monotonic()
    root = os.path.normpath(git("rev-parse", "--show-toplevel"))
    os.chdir(root)
    version = tomllib.load(open("Cargo.toml", "rb"))["workspace"]["package"]["version"]
    commit = git("rev-parse", "--short=12", "HEAD")
    day = datetime.date.today().isoformat()
    dist = os.path.join(root, "dist")
    os.makedirs(dist, exist_ok=True)
    kind = " (a practice)" if a.practice else " (a quick build, never to be published)" if a.quick else ""
    say(f"Isolated State {version} for {platform}, from commit {commit}{kind}")
    refuse = Refusals(a.practice)

    say("1. the checkout")
    if a.ci:
        check_ci_checkout()
        say("  nothing in it has changed")
    else:
        check_checkout(version, refuse)

    say(f"2. the build (cargo build --profile {profile} -p isostate-viewer, with the machine's folders taken out)")
    if platform == WINDOWS:
        exe, build_s = build(root, profile)
    elif platform == LINUX:
        exe, build_s = build(root, profile, TRIPLES[LINUX][0])
    elif a.build_only:
        exe, build_s = build(root, profile, a.build_only)
        kept = os.path.join(dist, "slices", a.build_only, "isostate-viewer")
        os.makedirs(os.path.dirname(kept), exist_ok=True)
        shutil.copyfile(exe, kept)
        say(f"  built for {a.build_only} in {build_s / 60:.1f} minutes: {os.path.getsize(kept) / 2**20:.1f} MiB, kept as {os.path.relpath(kept)}")
        return
    else:
        if a.slices:
            slices = [os.path.join(a.slices, t, "isostate-viewer") for t in TRIPLES[MAC]]
            missing = [s for s in slices if not os.path.isfile(s)]
            if missing:
                raise SystemExit("not built: " + ", ".join(missing))
            build_s = 0.0
            say("  the two programs built before: " + ", ".join(os.path.relpath(s) for s in slices))
        else:
            slices, build_s = [], 0.0
            for t in TRIPLES[MAC]:
                exe, took = build(root, profile, t)
                slices.append(exe)
                build_s += took
        exe = join_slices(slices, os.path.join(dist, "slices", "universal", "isostate-viewer"))
        say("  joined into one program for both kinds of Mac (arm64 and x86_64)")
    said = run([exe, "--version"]).stdout.strip()
    say(f"  built in {build_s:.0f} s: {os.path.getsize(exe) / 2**20:.1f} MiB; it says: {said}")
    if version not in said or commit not in said:
        refuse(f"the program says {said!r}, where {version} and {commit} were looked for")

    say("3. the archive")
    path, top, crates = PACK[platform](root, exe, version, commit, day, dist)
    size = os.path.getsize(path)
    sha = hashlib.sha256(open(path, "rb").read()).hexdigest()
    say(f"  {os.path.relpath(path)}: {size / 2**20:.1f} MiB, with the {crates} crates' licences listed")

    say("4. what the program and the archive must not carry")
    hits = search_all(exe, path, private_patterns(bool(a.ci)))
    if hits:
        raise SystemExit("found in the program or the archive:\n  " + "\n  ".join(hits[:20]))
    say("  neither the home folder" + ("" if a.ci else ", the user name nor git's name or address") + " is in the program or the archive")

    say("5. the trial: the archive unpacked into an empty folder away from the repository, and a picture taken there")
    pictures, where = trial(path, top, version, commit, root, platform)
    for what, picture, how, took in pictures:
        mark = "" if platform == WINDOWS else f"-{platform}"
        if picture.endswith("trial-intel.png"):
            mark += "-intel"
        kept = os.path.join(dist, f"isolated-state-{version}{mark}-trial.png")
        shutil.copyfile(picture, kept)
        say(f"  {what} started and took its picture in {took:.0f} s: {how}")
        say(f"  the picture is kept as {os.path.relpath(kept)}")
    if a.keep_trial:
        say(f"  the trial's folder is kept: {where}")
    else:
        shutil.rmtree(where, ignore_errors=True)

    total = time.monotonic() - t0
    if a.ci:
        say()
        say(f"Made in {total / 60:.1f} minutes (the build {build_s / 60:.1f}): {os.path.relpath(path)}, {size / 2**20:.1f} MiB, sha256 {sha}")
        return

    say("6. the release notes")
    notes, count, since = release_notes(version, commit, day)
    npath = os.path.join(dist, f"isolated-state-{version}-notes.md")
    with open(npath, "w", encoding="utf-8") as f:
        f.write(notes)
    say(f"  {os.path.relpath(npath)}: {count} entries {since}")

    say()
    say(f"Made in {total / 60:.1f} minutes (the build {build_s / 60:.1f}): {os.path.relpath(path)}, {size / 2**20:.1f} MiB, sha256 {sha}")
    say()
    if refuse.waved:
        say("This was a practice, and these would have refused a release:")
        for w in refuse.waved:
            say(f"  - {w.splitlines()[0]}")
        say("Nothing below is to be run for it.")
        say()
    say("To mark and publish the version (nothing has been run; read the notes through first):")
    tag = f"v{version}"
    say(f'  git tag -a {tag} -m "Isolated State {version}" {git("rev-parse", "HEAD")}')
    say(f"  git push origin {tag}")
    say(f'  gh release create {tag} "{os.path.relpath(path)}" --title "Isolated State {version}" --notes-file "{os.path.relpath(npath)}" --prerelease')
    origin = re.sub(r"\.git$", "", git("remote", "get-url", "origin", check=False))
    m = re.search(r"github\.com[:/](.+)$", origin)
    if m:
        # a pre-release is not what releases/latest shows: that page answers 404 while there are only pre-releases
        say(f"Then it is at https://github.com/{m.group(1)}/releases/tag/{tag}, to download on any machine;")
        say(f"the Mac and Linux builds follow there in about an hour (.github/workflows/mac-linux.yml; gh run list --workflow mac-linux.yml).")


if __name__ == "__main__":
    main()
